# npm Registry 다운로드와 `node_modules` 배치 가이드

이 문서는 Rust로 JavaScript 패키지 매니저를 만들 때, `npm registry`에서 패키지를 내려받아 `node_modules`에 배치하는 과정을 구현 관점에서 정리한 가이드다.

범위는 아래에 집중한다.

- npm registry 메타데이터 조회
- tarball 다운로드
- integrity 검증
- 압축 해제
- 패키지 루트 추출
- `node_modules` 경로 계산
- `.bin` 링크 생성
- lockfile에 반영할 데이터 정리

이 문서는 resolver 전체보다는 "선택된 패키지를 어떻게 안전하게 설치하느냐"에 초점을 둔다.

---

## 1. 전체 흐름

패키지 하나를 설치하는 최소 흐름은 이렇다.

1. 패키지 이름과 version range를 받는다
2. npm registry에서 packument를 가져온다
3. range에 맞는 정확한 버전을 고른다
4. 해당 버전의 `dist.tarball`과 `dist.integrity`를 읽는다
5. tarball을 다운로드한다
6. integrity를 검증한다
7. 임시 디렉터리에 압축을 푼다
8. tarball 내부의 `package/` 디렉터리를 실제 패키지 루트로 잡는다
9. 설치 대상 `node_modules/...` 경로로 복사 또는 이동한다
10. 패키지 내부 `package.json`을 읽는다
11. dependency와 `bin` 정보를 후속 설치 단계에 넘긴다

중요한 점은 이거다.

- 다운로드가 끝났다고 바로 설치하면 안 된다
- integrity 검증 전에는 `node_modules`에 노출하면 안 된다
- 압축 해제도 staging dir에서 해야 한다

---

## 2. registry에서 무엇을 가져오나

### 2.1 packument

npm registry는 패키지 전체 메타데이터 문서를 돌려준다. 이 문서를 흔히 packument라고 부른다.

대표 요청:

```text
GET https://registry.npmjs.org/react
GET https://registry.npmjs.org/@types/node
```

registry 문서와 API 문서 기준으로 이 응답에는 보통 아래가 들어 있다.

- `dist-tags`
- `versions`
- 각 버전의 `dependencies`
- 각 버전의 `dist.tarball`
- 각 버전의 `dist.integrity`
- 경우에 따라 `dist.shasum`

공식 참고:

- https://api-docs.npmjs.com/
- https://docs.npmjs.com/cli/v8/using-npm/registry/
- https://github.com/npm/registry/blob/main/docs/REGISTRY-API.md

### 2.2 설치에 실제로 필요한 필드

버전 하나를 고른 뒤에는 이 정도만 있으면 설치가 가능하다.

```json
{
  "name": "left-pad",
  "version": "1.3.0",
  "dependencies": {},
  "bin": null,
  "dist": {
    "tarball": "https://registry.npmjs.org/left-pad/-/left-pad-1.3.0.tgz",
    "integrity": "sha512-...",
    "shasum": "..."
  }
}
```

1차 구현에서는 `dist.tarball`과 `dist.integrity`를 가장 중요하게 보면 된다.

---

## 3. scoped package URL과 설치 경로

`@scope/name` 패키지는 URL과 설치 디렉터리를 둘 다 신경 써야 한다.

### 3.1 registry 요청 URL

scoped package는 URL 인코딩이 필요하다.

예시:

- 패키지명: `@types/node`
- 요청 경로: `%40types%2Fnode`

즉 요청은 대략 이렇게 된다.

```text
GET https://registry.npmjs.org/%40types%2Fnode
```

### 3.2 `node_modules` 설치 경로

npm 문서 기준 scoped package는 `node_modules/@scope/name` 형태로 설치된다.  
출처: https://docs.npmjs.com/cli/v11/configuring-npm/folders/

예시:

```text
node_modules/@types/node
node_modules/@babel/core
```

따라서 설치 경로 계산 함수는 평범한 패키지와 scoped package를 분리해야 한다.

예시:

```rust
fn install_dir(node_modules: &Path, package_name: &str) -> PathBuf {
    if let Some((scope, name)) = package_name.split_once('/') {
        node_modules.join(scope).join(name)
    } else {
        node_modules.join(package_name)
    }
}
```

전제:

- `package_name`이 `@scope/name` 또는 `name` 형식이라는 검증이 먼저 있어야 한다

---

## 4. 다운로드 전에 먼저 결정해야 하는 것

다운로드 함수에 들어가기 전에 아래 값은 이미 확정되어 있어야 한다.

- 패키지 이름
- 정확한 버전
- tarball URL
- integrity 문자열
- 설치 루트 경로
- 캐시 경로

이걸 모아서 구조체로 들고 가면 흐름이 깔끔하다.

```rust
pub struct ResolvedPackage {
    pub name: String,
    pub version: String,
    pub tarball_url: String,
    pub integrity: Option<String>,
    pub dependencies: std::collections::BTreeMap<String, String>,
}
```

이 구조체는 resolver와 installer의 경계가 된다.

---

## 5. tarball 다운로드

### 5.1 저장 위치

권장 방식:

- 먼저 캐시 파일 또는 temp 파일에 다운로드
- 완료 후 integrity 검증
- 검증 성공 시 cache에 확정

추천 디렉터리 예시:

```text
.yourpm/
  cache/
    tarballs/
      sha512-<digest>.tgz
  tmp/
```

파일명을 패키지 이름이 아니라 integrity 기반으로 잡으면 dedupe에 유리하다.

### 5.2 구현 순서

1. temp file 생성
2. HTTP GET
3. response body를 temp file에 스트리밍 기록
4. integrity 검증
5. 성공 시 cache path로 rename

### 5.3 왜 스트리밍이 좋은가

- 큰 tarball에서 메모리 사용량 감소
- 다운로드와 무결성 검증을 결합하기 쉬움
- 캐시 작성 흐름이 단순함

---

## 6. integrity 검증

npm ecosystem에서는 `integrity`가 핵심이다. `package-lock.json` 문서에도 registry source는 registry가 제공한 `integrity` 또는 없으면 `shasum`을 사용한다고 설명되어 있다.  
출처: https://docs.npmjs.com/cli/v6/configuring-npm/package-lock-json/?v=true

### 6.1 추천 방식

Rust에서는 `ssri` crate를 쓰는 쪽이 맞다.

이유:

- npm의 SRI 문자열 형식과 직접 맞는다
- 직접 base64/hash 조립할 필요가 없다
- 나중에 lockfile 검증에도 재사용 가능하다

### 6.2 검증 정책

우선순위:

1. `dist.integrity`가 있으면 그것을 검증
2. 없고 `dist.shasum`만 있으면 fallback 검증
3. 둘 다 없으면 기본 실패 또는 명시적 allow-unsafe 정책

실전에서는 `integrity` 없는 설치를 조용히 통과시키지 않는 편이 낫다.

---

## 7. tarball 압축 해제

npm tarball은 보통 내부에 `package/` 디렉터리를 가진다. 설치 시 그 디렉터리 아래가 실제 패키지 루트다.

예시:

```text
package/
  package.json
  index.js
  lib/
```

### 7.1 권장 절차

1. 새 staging dir 생성
2. tar.gz를 staging dir에 압축 해제
3. `staging/package` 존재 확인
4. `staging/package/package.json` 존재 확인
5. 해당 디렉터리를 설치 후보 루트로 사용

### 7.2 path traversal 방지

압축 해제 시 반드시 entry path를 검증해야 한다.

막아야 할 것:

- `../` 포함 경로
- 절대 경로
- 설치 루트 밖으로 빠져나가는 심볼릭 링크

tarball 처리 코드는 공격 표면이다. 여기서 대충 하면 안 된다.

---

## 8. `package.json`은 tarball 안의 것을 다시 읽어야 한다

registry metadata만 믿고 끝내면 안 된다. 실제 설치된 패키지는 tarball 안의 `package.json`을 기준으로 봐야 한다.

이유:

- `bin` 정보가 필요하다
- dependency 정보가 실제 tarball과 맞는지 확인해야 한다
- `main`, `exports`, `type` 등 실행 관련 정보는 실제 패키지 루트 기준으로 봐야 한다

따라서 설치 흐름 중 반드시:

```text
staging/package/package.json
```

을 읽는 단계가 있어야 한다.

---

## 9. 설치 대상 경로 계산

설치 경로는 "어느 패키지의 dependency로 들어가느냐"에 따라 달라진다.

### 9.1 root dependency

프로젝트 루트 dependency면:

```text
<project>/node_modules/<name>
<project>/node_modules/@scope/<name>
```

### 9.2 nested dependency

예를 들어 `a`의 dependency로 `b`를 설치하면:

```text
<project>/node_modules/a/node_modules/b
```

즉 installer는 보통 `target_node_modules_dir`를 직접 받아야 한다.

추천 시그니처:

```rust
fn install_package_into(
    target_node_modules_dir: &Path,
    package: &ResolvedPackage,
) -> Result<InstalledPackage, InstallError>
```

이 방식이 root install과 nested install을 같은 코드로 처리하기 좋다.

---

## 10. `node_modules`에 반영하는 순서

가장 안전한 방식은 atomic move다.

### 10.1 권장 방식

1. 최종 경로의 상위 디렉터리 생성
2. 같은 상위 디렉터리에 임시 디렉터리 생성
3. `staging/package` 내용을 임시 디렉터리로 옮김
4. 임시 디렉터리를 최종 패키지 디렉터리명으로 rename

예시:

```text
project/node_modules/.tmp-left-pad-12345
project/node_modules/left-pad
```

### 10.2 왜 그냥 copy하지 않나

직접 최종 경로에 쓰면:

- 도중 실패 시 반쯤 설치된 패키지가 남는다
- 다른 프로세스가 잘못된 상태를 볼 수 있다

atomic rename 기반이면 훨씬 안전하다.

---

## 11. 이미 설치된 패키지가 있을 때

정책을 먼저 정해야 한다.

### 11.1 단순 정책

- 같은 이름, 같은 버전이면 skip
- 같은 이름, 다른 버전이면 교체 또는 nested install

### 11.2 root install 기준 권장

1차 구현에서는 아래가 단순하다.

- 최종 경로가 비어 있으면 설치
- 있으면 내부 `package.json` 읽기
- 버전이 같으면 재다운로드 없이 성공 처리
- 버전이 다르면 resolver 결정에 따라 교체 또는 더 깊은 `node_modules`에 설치

installer가 resolver 역할을 겸하면 설계가 망가지기 쉽다. 버전 충돌 정책은 resolver 쪽에서 미리 끝내는 게 좋다.

---

## 12. `.bin` 생성

많은 패키지는 `package.json`의 `bin` 필드로 실행 파일을 노출한다.

생성 위치는 보통:

```text
<target_node_modules_dir>/.bin/
```

root dependency면:

```text
project/node_modules/.bin
```

`a` 아래에 설치된 dependency면:

```text
project/node_modules/a/node_modules/.bin
```

### 12.1 `bin` 필드 형식

`bin`은 보통 둘 중 하나다.

1. 문자열
2. 객체

예시:

```json
{
  "bin": "cli.js"
}
```

또는:

```json
{
  "bin": {
    "tsc": "./bin/tsc",
    "tsserver": "./bin/tsserver"
  }
}
```

### 12.2 생성 규칙

- 문자열이면 패키지 이름을 bin 이름으로 사용
- 객체면 key가 bin 이름
- 대상은 설치된 패키지 내부 파일 경로

Unix에선 symlink가 단순하다. Windows는 `.cmd` shim을 따로 생성하는 쪽이 현실적이다.

---

## 13. lockfile에 무엇을 기록해야 하나

다운로드와 설치가 끝나면 lockfile에 최소한 아래는 남겨야 한다.

- 정확한 패키지 이름
- 정확한 버전
- resolved tarball URL
- integrity
- dependency 목록
- 설치 트리 상 위치 또는 parent 관계

이유:

- 다음 `install`에서 재현 가능해야 한다
- `ci` 모드에서 resolve 없이 그대로 fetch/install 가능해야 한다
- integrity mismatch를 다시 검증할 수 있어야 한다

---

## 14. 캐시 전략

### 14.1 tarball cache

최소한 tarball cache는 두는 편이 좋다.

장점:

- 재설치 속도 향상
- offline 또는 flaky network 대응
- integrity 기반 dedupe 가능

### 14.2 extract cache는 신중하게

압축 해제 결과까지 캐시할 수도 있지만 1차 구현에선 과하다.

처음엔:

- tarball만 cache
- extract는 설치 시마다 staging dir에서 수행

정도가 적당하다.

---

## 15. 동시성 제어

`node_modules`는 공유 상태이기 때문에 install 중 lock이 필요하다.

최소 권장:

- 프로젝트 단위 global lock

예시:

```text
project/node_modules/.install.lock
```

lock 범위:

- lockfile 읽기/쓰기
- `node_modules` 변경
- `.bin` 생성/삭제

1차에선 fine-grained lock보다 coarse lock이 더 안전하다.

---

## 16. 실패 복구

중간 실패는 반드시 예상해야 한다.

### 16.1 자주 실패하는 지점

- 네트워크 끊김
- integrity mismatch
- 손상된 tarball
- 압축 해제 실패
- 권한 문제
- 기존 디렉터리와 충돌

### 16.2 복구 원칙

- temp file은 삭제
- staging dir은 삭제
- 최종 디렉터리는 성공 직전까지 건드리지 않음
- lockfile은 마지막에만 갱신

이 원칙만 지켜도 반쯤 설치된 상태를 많이 줄일 수 있다.

---

## 17. 추천 구현 단위

코드를 쪼개면 대략 이 정도가 좋다.

```text
registry/
  fetch_packument.rs
  select_version.rs
download/
  fetch_tarball.rs
  verify_integrity.rs
extract/
  unpack_tgz.rs
install/
  install_package.rs
  write_bin_links.rs
  place_in_node_modules.rs
lockfile/
  write_lockfile.rs
```

함수 경계는 대충 이렇게 잡으면 된다.

```rust
fn fetch_packument(name: &str) -> Result<Packument, RegistryError>;
fn resolve_version(packument: &Packument, range: &str) -> Result<VersionMeta, ResolveError>;
fn download_tarball(pkg: &ResolvedPackage, cache: &Path) -> Result<PathBuf, DownloadError>;
fn extract_package_tarball(tgz: &Path, staging: &Path) -> Result<PathBuf, ExtractError>;
fn install_package_into(node_modules: &Path, pkg_root: &Path, meta: &ResolvedPackage) -> Result<InstalledPackage, InstallError>;
fn create_bin_links(target_node_modules: &Path, installed: &InstalledPackage) -> Result<(), InstallError>;
```

---

## 18. 구현 순서 추천

실제로 만들 때는 아래 순서가 가장 덜 꼬인다.

1. packument fetch
2. exact version 선택
3. tarball 다운로드
4. integrity 검증
5. staging extract
6. `package.json` 재파싱
7. root `node_modules` 설치
8. root `.bin` 생성
9. nested dependency 설치
10. lockfile 출력

즉, 처음엔 hoisting 없이 root와 nested 설치만 정확하게 만드는 게 맞다.

---

## 19. 구현 시 자주 하는 실수

- Rust `semver`를 npm semver 대신 사용
- scoped package URL 인코딩을 빼먹음
- `package/` 디렉터리를 무시하고 tarball 루트를 바로 설치
- integrity 검증 전에 압축 해제 결과를 노출
- `bin` 필드가 문자열과 객체 둘 다 올 수 있다는 점을 놓침
- scoped package 설치 경로를 `node_modules/@scope-name`처럼 잘못 계산
- nested dependency를 전부 root에만 설치
- lockfile에 resolved URL과 integrity를 저장하지 않음

이 항목들은 거의 실제 버그로 이어진다.

---

## 20. 추천 결론

핵심은 이거다.

1. registry에선 packument를 가져온다
2. 선택된 버전의 `dist.tarball`과 `dist.integrity`를 사용한다
3. 다운로드는 cache/temp에 받고 integrity를 먼저 검증한다
4. tarball 내부 `package/`를 실제 패키지 루트로 사용한다
5. 설치는 target `node_modules`에 atomic하게 반영한다
6. `package.json`의 `bin`을 읽어 `.bin` 링크를 만든다
7. resolved URL과 integrity를 lockfile에 기록한다

이 순서를 지키면 npm registry에서 받아 `node_modules`에 배치하는 핵심 경로는 꽤 안정적으로 만들 수 있다.

---

## 참고 링크

- npm registry API: https://api-docs.npmjs.com/
- npm registry 문서: https://docs.npmjs.com/cli/v8/using-npm/registry/
- npm `package-lock.json`: https://docs.npmjs.com/cli/v6/configuring-npm/package-lock-json/?v=true
- npm folders: https://docs.npmjs.com/cli/v11/configuring-npm/folders/
- npm registry metadata 문서: https://github.com/npm/registry/blob/main/docs/REGISTRY-API.md
- Node.js modules 문서: https://nodejs.org/api/modules.html
