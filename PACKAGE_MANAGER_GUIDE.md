# Rust로 JavaScript 패키지 매니저 만들기

이 문서는 `npm` 같은 JavaScript 패키지 매니저를 Rust로 구현할 때 필요한 설계 기준을 정리한 가이드다.

여기서 말하는 대상은 이런 툴이다.

- `package.json`을 읽는다
- npm registry에서 패키지 메타데이터를 가져온다
- semver range를 해석한다
- dependency graph를 만든다
- `node_modules`를 구성한다
- `package-lock.json` 같은 lockfile을 쓴다
- `bin` 링크를 만든다
- workspace를 지원할 수 있다

즉, 일반 바이너리 설치기나 Homebrew류가 아니라, Node 생태계용 dependency manager다.

---

## 1. 먼저 현실적인 범위를 잡아라

JavaScript 패키지 매니저는 겉보기보다 훨씬 어렵다. 이유는 다음 때문이다.

- npm semver 규칙은 Cargo식 semver와 다르다
- `dependencies`, `devDependencies`, `peerDependencies`, `optionalDependencies`가 다 다르다
- `node_modules` 레이아웃과 hoisting이 복잡하다
- lifecycle scripts가 있다
- `bin` 링크가 있다
- workspace가 있다
- lockfile과 실제 설치 트리가 다를 수 있다

처음부터 npm 전체를 복제하려고 하면 실패 확률이 높다.

### 1.1 추천하는 1차 목표

1차 구현은 아래만 지원하는 쪽이 맞다.

1. `package.json` 읽기
2. `dependencies`와 `devDependencies` 파싱
3. npm registry에서 packument 조회
4. npm 방식 semver range 해석
5. tarball 다운로드
6. integrity 체크
7. 단순 dependency graph 생성
8. `node_modules` 설치
9. root `.bin` 링크 생성
10. lockfile 생성

이 정도면 이미 "작동하는 JS 패키지 매니저"다.

### 1.2 1차에서 빼는 것이 좋은 것

- peer dependency 완전 호환
- lifecycle script 전체 지원
- native addon 빌드
- workspaces 완전 지원
- overrides / resolutions
- content-addressable global store
- symlink 기반 고급 dedupe
- publish
- audit

이건 2차 이후가 맞다.

---

## 2. npm 계열 툴의 핵심 개념부터 정확히 잡아야 한다

Rust 쪽 패키지 매니저 감각으로 접근하면 자주 틀린다.

### 2.1 `package.json`

프로젝트의 의도된 의존성 선언이다.

공식 npm 문서 기준으로 `package.json`은 프로젝트 메타데이터와 dependency, script, bin, workspaces 등의 설정을 담는다.  
출처: https://docs.npmjs.com/cli/v11/configuring-npm/package-json

처음에 꼭 읽을 필드:

- `name`
- `version`
- `dependencies`
- `devDependencies`
- `optionalDependencies`
- `peerDependencies`
- `bin`
- `workspaces`
- `scripts`

### 2.2 lockfile

`package-lock.json`은 선언이 아니라 "해결 결과"다.

npm 문서상 lockfile은 설치 트리를 재현 가능하게 만들기 위한 파일이다.  
출처: https://docs.npmjs.com/cli/v8/configuring-npm/package-lock-json

즉:

- `package.json`은 원하는 범위
- `package-lock.json`은 실제 선택된 정확한 버전과 트리

### 2.3 registry packument

npm registry는 패키지 이름에 대해 여러 버전 메타데이터를 돌려준다. 이 문서를 보통 packument라고 부른다.

npm registry API 문서와 registry 문서 기준으로 패키지 메타데이터는 버전별 정보, dist 정보, tarball URL, integrity, dependencies 등을 담는다.  
출처:

- https://api-docs.npmjs.com/
- https://docs.npmjs.com/cli/v8/using-npm/registry/

### 2.4 `node_modules`

이건 단순한 "패키지 폴더 모음"이 아니다.

Node의 모듈 해석 규칙 때문에 어떤 패키지를 어느 깊이에 두느냐가 동작에 직접 영향을 준다.

즉 JS 패키지 매니저의 핵심은 사실상:

- semver 해석
- dependency tree 해석
- `node_modules` 배치 전략

이 3개다.

---

## 3. 아키텍처는 이렇게 나누는 게 좋다

추천 구조:

```text
src/
  main.rs
  lib.rs
  cli/
    mod.rs
    args.rs
  app/
    mod.rs
    install.rs
    remove.rs
    update.rs
    ci.rs
  manifest/
    mod.rs
    package_json.rs
    lockfile.rs
  registry/
    mod.rs
    npm.rs
    metadata.rs
  semver/
    mod.rs
  resolver/
    mod.rs
    graph.rs
    hoist.rs
  store/
    mod.rs
    cache.rs
    extract.rs
    tree.rs
    bin.rs
  scripts/
    mod.rs
  report/
    mod.rs
  error.rs
tests/
  fixtures/
  integration/
```

각 계층 역할은 이렇다.

### 3.1 CLI

- `install`, `add`, `remove`, `update`, `ci`
- 플래그 파싱
- 출력 모드 선택

### 3.2 App

- 실제 유스케이스 orchestration
- lock 획득
- manifest 읽기
- resolve 실행
- 다운로드/설치/lockfile 갱신

### 3.3 Manifest

- `package.json` 타입
- lockfile 타입
- read/write

### 3.4 Registry

- 패키지 메타데이터 조회
- tarball URL 획득
- dist integrity 정보 획득

### 3.5 Semver

- npm semver range 파싱
- version satisfaction 체크

### 3.6 Resolver

- dependency graph 구성
- 중복 버전 판단
- hoisting 배치 결정
- peer dependency 검사

### 3.7 Store

- tarball 캐시
- 압축 해제
- `node_modules` 구성
- `.bin` 링크 생성

---

## 4. npm 스타일 패키지 매니저에서 제일 중요한 건 semver다

여기서 많이 실수한다.

Rust `semver` crate는 Cargo 해석 기준이다. docs.rs 설명에도 이 crate는 Cargo의 semver 해석을 따른다고 되어 있고, 다른 생태계에는 적절치 않을 수 있다고 나온다.  
출처: https://docs.rs/semver

반면 `node-semver` 호환용 Rust crate는 별도로 있다. `node_semver`는 Node/NPM의 semver와 호환되도록 설계됐다고 docs.rs에 명시되어 있다.  
출처: https://docs.rs/node-semver

### 4.1 결론

JS 패키지 매니저를 만들면:

- `semver` crate를 기본 선택지로 보면 안 된다
- `node_semver` 같은 npm 호환 range 파서를 쓰는 쪽이 맞다

### 4.2 왜 중요한가

npm range에는 이런 게 많다.

- `^1.2.3`
- `~1.2.3`
- `1.x`
- `*`
- `>=1 <2`
- prerelease 관련 미묘한 규칙

이걸 Cargo식 해석으로 처리하면 resolver가 틀어진다.

---

## 5. `package.json` 데이터 모델

처음부터 모든 필드를 완벽히 모델링할 필요는 없다. 하지만 너무 적게 잡아도 안 된다.

추천 구조:

```rust
use std::collections::BTreeMap;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct PackageJson {
    pub name: Option<String>,
    pub version: Option<String>,
    #[serde(default)]
    pub dependencies: BTreeMap<String, String>,
    #[serde(default, rename = "devDependencies")]
    pub dev_dependencies: BTreeMap<String, String>,
    #[serde(default, rename = "optionalDependencies")]
    pub optional_dependencies: BTreeMap<String, String>,
    #[serde(default, rename = "peerDependencies")]
    pub peer_dependencies: BTreeMap<String, String>,
    #[serde(default)]
    pub bin: serde_json::Value,
    #[serde(default)]
    pub scripts: BTreeMap<String, String>,
    #[serde(default)]
    pub workspaces: Option<serde_json::Value>,
}
```

### 5.1 crate 선택

직접 struct를 만드는 방법도 좋고, `package_json` crate를 쓸 수도 있다. docs.rs 기준 `package_json`은 npm `package.json` 스키마에 맞는 타입을 제공한다.  
출처: https://docs.rs/package-json/latest/package_json/struct.PackageJson.html

추천은 이렇다.

- 빨리 가려면 `package_json`
- 제어를 세밀하게 하려면 직접 struct 정의

실제로는 직접 struct 정의가 유지보수에 더 나은 경우가 많다.

---

## 6. registry 쪽은 npm public API 구조를 이해해야 한다

기본적으로 필요한 건 2개다.

### 6.1 패키지 메타데이터 조회

핵심 요청:

- `GET https://registry.npmjs.org/<package-name>`

이 응답에는 보통:

- dist-tags
- versions
- 각 버전의 dependencies
- tarball URL
- integrity / shasum

같은 정보가 들어 있다.

관련 공식 문서:

- https://api-docs.npmjs.com/
- https://docs.npmjs.com/cli/v8/using-npm/registry/

### 6.2 tarball 다운로드

버전 metadata의 `dist.tarball` URL로 내려받는다.

설치 플로우는 보통 이렇다.

1. packument 요청
2. version 선택
3. `dist.tarball` URL 확인
4. tarball 다운로드
5. integrity 검증
6. 압축 해제
7. `package/` 폴더 내용 설치

---

## 7. integrity 검증은 필수다

npm 계열에서는 checksum보다 SRI 문자열을 자주 본다.

예시:

```text
sha512-BASE64...
```

Rust에선 `ssri` crate가 매우 적합하다. docs.rs 설명 그대로 SRI 문자열 파싱, 생성, 검증을 지원한다.  
출처: https://docs.rs/ssri

### 7.1 왜 `ssri`를 추천하나

- npm 메타데이터와 개념이 맞다
- 무결성 체크를 직접 구현할 필요가 없다
- 스트리밍 검증 방향으로 확장 가능하다

### 7.2 설치 파이프라인에서의 위치

반드시 아래 순서여야 한다.

1. 다운로드
2. integrity 검증
3. 압축 해제
4. `node_modules` 반영

검증 전 내용을 설치 트리에 노출하면 안 된다.

---

## 8. tarball 구조를 알아야 한다

npm tarball은 일반적으로 압축을 풀면 내부에 `package/` 디렉터리 아래 파일들이 들어 있다.

즉 설치 시에는 보통:

- tarball 다운로드
- 압축 해제
- `package/` 폴더를 실제 패키지 루트로 간주

이 흐름이 된다.

### 8.1 주의할 점

- path traversal 방지
- symlink 처리 정책
- 파일 권한 복원 정책
- `package.json` 존재 여부 확인

JavaScript 패키지 매니저는 생각보다 압축 해제 취약점에 민감하다.

---

## 9. lockfile은 처음부터 별도 타입으로 설계해라

lockfile은 설치 결과물의 스냅샷이다.

### 9.1 최소 필드

- lockfile version
- root package 정보
- resolved package 목록
- 각 package의 version
- resolved tarball URL
- integrity
- dependencies 관계
- install 위치 또는 트리 정보

### 9.2 예시 스키마

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct Lockfile {
    pub name: Option<String>,
    pub version: Option<String>,
    #[serde(rename = "lockfileVersion")]
    pub lockfile_version: u32,
    pub packages: std::collections::BTreeMap<String, LockedPackage>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LockedPackage {
    pub version: String,
    pub resolved: Option<String>,
    pub integrity: Option<String>,
    #[serde(default)]
    pub dependencies: std::collections::BTreeMap<String, String>,
}
```

이건 npm lockfile과 완전히 같을 필요는 없다. 하지만 최소한 아래는 보장해야 한다.

- 같은 lockfile이면 같은 설치 결과
- lockfile만으로 fetch 가능한 정보 확보

### 9.3 `install`과 `ci`를 분리하는 이유

권장 정책:

- `install`: `package.json` 기준으로 resolve하고 lockfile 갱신 가능
- `ci`: lockfile을 엄격히 따르고, 불일치 시 실패

이건 npm의 `install` / `ci` 역할 분리와 같은 방향이다.

---

## 10. resolver는 가장 어려운 부분이다

JS 패키지 매니저의 핵심 난이도는 resolver다.

resolver는 단순히 "버전 하나 선택"이 아니다.

- 여러 dependency가 같은 package를 서로 다른 range로 요구
- 같은 패키지가 여러 버전 필요할 수 있음
- hoisting 여부에 따라 설치 경로가 달라짐
- peer dependency는 부모 컨텍스트와 맞아야 함

### 10.1 1차 resolver 전략

처음엔 이 정도가 적당하다.

1. root `dependencies` 읽기
2. 각 의존성 버전 resolve
3. transitive dependency 재귀 확장
4. 정확한 tree 생성
5. hoisting은 최소화하거나 root-level dedupe만 제한적으로 수행

즉 처음부터 pnpm급 최적화를 노리지 말고, "정확한 트리"를 먼저 만드는 게 맞다.

### 10.2 자료구조

그래프 자료구조가 있으면 편하다.

`petgraph`는 그래프 표현과 알고리즘에 유용하다.  
출처: https://docs.rs/petgraph/latest/petgraph/

하지만 꼭 필요한 건 아니다. 트리 중심 구현이면 직접 구조체로도 충분하다.

예시:

```rust
pub struct ResolvedNode {
    pub name: String,
    pub version: String,
    pub integrity: Option<String>,
    pub tarball_url: String,
    pub dependencies: Vec<ResolvedNode>,
}
```

처음엔 이게 더 단순하다.

---

## 11. `node_modules` 배치 전략

이게 실제 동작을 결정한다.

### 11.1 가장 단순한 방법

중첩 설치:

```text
node_modules/
  a/
    package.json
    node_modules/
      b/
```

장점:

- 구현이 단순하다
- dependency isolation이 쉽다

단점:

- 디스크 사용량 증가
- hoisting 없음

### 11.2 root-level dedupe

간단한 최적화로 아래를 할 수 있다.

- 동일 버전이 이미 root에 있으면 재사용
- 아니면 nested install

이 정도만 해도 체감 품질이 올라간다.

### 11.3 완전 hoisting은 나중에

완전 hoisting은 충돌 해결, peer dependency, bin 경로 모두에 영향을 준다.

1차 버전에선:

- 정확한 nested tree
- 제한적 dedupe

정도로 끝내는 게 안전하다.

---

## 12. `.bin` 링크는 꼭 필요하다

많은 JS 패키지가 CLI 실행 파일을 `bin` 필드로 노출한다.

`package_json` 문서에도 `bin` 필드가 npm에서 executable 설치에 사용된다고 설명되어 있다.  
출처: https://docs.rs/package-json/latest/package_json/struct.PackageJson.html

### 12.1 동작 원리

패키지 설치 후:

- `node_modules/.bin/<name>` 생성
- 대상은 패키지 내부의 `bin` 스크립트

Unix에서는 symlink가 흔하고, Windows는 `.cmd` shim이 필요할 수 있다.

### 12.2 구현 시 주의점

- `bin`이 string일 수도 있고 object일 수도 있다
- shebang 유지
- 상대 경로 기준 정확히 맞추기
- Windows용 launcher 처리

---

## 13. lifecycle script는 초기에 매우 보수적으로 다뤄라

JS 패키지 매니저에서 lifecycle script는 큰 복잡도를 만든다.

예시:

- `preinstall`
- `install`
- `postinstall`
- `prepare`

이건 사실상 arbitrary code execution이다.

### 13.1 1차 권장 정책

선택지 3개 중 하나를 고르는 게 좋다.

1. 완전 비활성화
2. `--ignore-scripts` 기본값 true
3. 명시적 opt-in일 때만 실행

보안과 디버깅 관점에서 초반에는 이게 맞다.

### 13.2 나중에 지원한다면

필요한 것:

- 환경변수 규격
- cwd 설정
- PATH에 `.bin` 주입
- script 실패 시 에러 리포트
- 출력 캡처

이건 설치기보다 "프로세스 실행 플랫폼"에 가까워진다.

---

## 14. workspace는 나중에 넣되 구조는 미리 대비해라

npm 문서 기준 workspaces는 하나의 상위 프로젝트 안에 여러 패키지를 두고, 설치 시 자동 symlink되는 구조다.  
출처: https://docs.npmjs.com/cli/v8/using-npm/workspaces/

### 14.1 1차 버전에서는

- `workspaces` 필드를 파싱만 하거나
- 단일 프로젝트만 지원하고
- workspace 발견 시 "아직 미지원" 에러를 내도 된다

### 14.2 하지만 타입은 미리 분리해라

예시:

```rust
pub enum ProjectKind {
    SinglePackage,
    Workspace { members: Vec<std::path::PathBuf> },
}
```

나중에 workspace를 붙일 가능성이 매우 높기 때문이다.

---

## 15. 추천 crate

여기서는 "JS 패키지 매니저" 기준으로 추천한다.

### 15.1 핵심 추천

| crate | 용도 | 이유 |
| --- | --- | --- |
| `clap` | CLI 파싱 | 서브커맨드, help, 에러 메시지 품질이 좋다 |
| `thiserror` | typed error | 내부 에러 모델 정리에 적합 |
| `serde` | JSON/TOML 직렬화 | `package.json`, lockfile, registry metadata 처리 |
| `serde_json` | JSON 파싱 | `package.json`과 registry 응답의 핵심 |
| `reqwest` | HTTP 클라이언트 | registry 요청과 tarball 다운로드 |
| `node_semver` | npm semver 해석 | Node/NPM 호환 range 처리 |
| `ssri` | integrity 검증 | npm dist integrity와 직접 맞는다 |
| `tempfile` | 임시 디렉터리 | tarball staging, atomic install |
| `tar` | tarball 해제 | npm package tarball 처리 |
| `flate2` | gzip 해제 | `.tgz` 대응 |
| `url` | URL 처리 | registry URL, resolved URL 정규화 |

### 15.2 강하게 추천하는 보조 crate

| crate | 용도 | 이유 |
| --- | --- | --- |
| `package_json` | `package.json` 타입 | 빠르게 시작하기 좋다 |
| `camino` | UTF-8 path | JS 툴링에서는 문자열 path 취급이 많아서 편하다 |
| `fs-err` | 나은 fs 에러 | 어떤 파일 작업이 실패했는지 더 잘 나온다 |
| `fd-lock` | 파일 lock | lockfile/state 동시성 제어 |
| `indicatif` | 진행 표시 | install UX 개선 |
| `tracing` | verbose/debug 로깅 | resolver와 install 디버깅에 유용 |
| `ignore` | 파일 무시 규칙 | pack/publish, workspace 스캔, 파일 트리 처리 |
| `globset` | glob 매칭 | workspace, files whitelist 처리 |

### 15.3 경우에 따라 추천

| crate | 용도 | 메모 |
| --- | --- | --- |
| `petgraph` | dependency graph | 복잡한 resolver를 만들 때 유용 |
| `tokio` | async 다운로드 | 병렬 fetch가 필요하면 도입 |
| `miette` | rich diagnostic | CLI 진단형 에러를 강화할 때 |
| `sha2` | fallback hash | integrity 외 별도 checksum 정책이 필요할 때 |

---

## 16. 내가 추천하는 의존성 조합

### 16.1 가장 실용적인 1차 버전

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
thiserror = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.12", features = ["blocking", "json", "rustls-tls"] }
node-semver = "2"
ssri = "9"
tempfile = "3"
tar = "0.4"
flate2 = "1"
url = "2"
fs-err = "3"
fd-lock = "4"
indicatif = "0.18"
tracing = "0.1"
tracing-subscriber = "0.3"
```

특징:

- npm registry에서 resolve/install하는 최소 기능에 적합
- blocking 기반이라 디버깅이 쉽다
- semver와 integrity를 npm 호환으로 처리 가능

### 16.2 조금 더 키울 때

```toml
[dependencies]
clap = { version = "4", features = ["derive"] }
thiserror = "2"
serde = { version = "1", features = ["derive"] }
serde_json = "1"
reqwest = { version = "0.12", features = ["json", "rustls-tls", "stream"] }
tokio = { version = "1", features = ["rt-multi-thread", "macros", "fs"] }
node-semver = "2"
ssri = "9"
tempfile = "3"
tar = "0.4"
flate2 = "1"
url = "2"
camino = "1"
fs-err = "3"
fd-lock = "4"
ignore = "0.4"
globset = "0.4"
petgraph = "0.8"
indicatif = "0.18"
tracing = "0.1"
tracing-subscriber = "0.3"
miette = "7"
```

특징:

- 병렬 다운로드
- workspace 준비
- 그래프 기반 resolver 확장
- 진단 출력 강화

---

## 17. 설치 플로우는 이렇게 설계하는 게 좋다

`install`은 아래 순서가 안전하다.

1. 프로젝트 루트 탐색
2. `package.json` 읽기
3. 기존 lockfile 읽기
4. lock 획득
5. dependency resolve
6. tarball fetch
7. integrity 검증
8. staging dir에 압축 해제
9. package tree 생성
10. `node_modules` 반영
11. `.bin` 생성
12. lockfile 갱신
13. report 출력

### 17.1 중요한 원칙

- 다운로드 중 `node_modules`를 직접 건드리지 말 것
- 검증 전 파일을 노출하지 말 것
- lockfile은 마지막에 commit할 것
- 실패 시 partial install 흔적을 cleanup할 것

---

## 18. 에러 처리는 앱 레벨과 설치 레벨을 분리해라

추천 방식:

- 내부는 `AppError`, `ResolveError`, `RegistryError`, `InstallError` 같은 typed error
- 최상단 CLI는 `Report`로 요약 출력

예시:

```rust
pub struct Report {
    pub summary: String,
    pub details: Vec<String>,
}
```

좋은 에러 메시지는 아래를 포함해야 한다.

- 어떤 패키지에서 실패했는가
- 어떤 version range를 해석하던 중이었는가
- 어떤 URL을 받으려 했는가
- 어떤 path에 쓰려 했는가
- integrity mismatch인지, semver mismatch인지, peer conflict인지

JS 패키지 매니저에서는 그냥 `failed to install` 정도로는 아무 의미가 없다.

---

## 19. 꼭 필요한 테스트

### 19.1 semver 테스트

- `^`
- `~`
- `x`
- `*`
- prerelease
- npm range edge case

### 19.2 resolver 테스트

- 같은 패키지 다른 버전 충돌
- nested dependency
- root dedupe
- optional dependency 실패
- peer dependency 경고/실패

### 19.3 installer 테스트

- integrity mismatch
- corrupt tarball
- partial install recovery
- `.bin` 생성
- Windows path 처리

### 19.4 fixture 기반 registry 테스트

실전에서는 테스트용 로컬 registry fixture가 거의 필수다.

추천 방식:

- 정적 JSON packument fixture
- 정적 `.tgz` fixture
- 로컬 HTTP 서버 또는 파일 기반 mock

---

## 20. 현실적인 구현 로드맵

### 단계 1

- `package.json` 파서
- npm semver 파서
- registry metadata fetch

### 단계 2

- 단일 패키지 tarball 다운로드
- integrity 검증
- staging extract

### 단계 3

- dependency tree resolver
- nested `node_modules` 설치

### 단계 4

- root `.bin` 생성
- lockfile 생성
- `install` / `ci`

### 단계 5

- dedupe
- optional dependency
- limited peer dependency validation

### 단계 6

- workspace
- lifecycle script opt-in
- update/remove/gc

이 순서가 제일 안전하다.

---

## 21. 추천 결론

Rust로 npm 같은 JS 패키지 매니저를 만들 때 제일 중요한 건 이거다.

1. Cargo식 사고를 버리고 npm식 semver와 tree 설치 모델을 먼저 이해할 것
2. `node_semver`와 `ssri` 같은 npm 친화 crate를 쓸 것
3. resolver와 `node_modules` 배치 전략을 핵심 문제로 볼 것
4. lockfile과 integrity를 처음부터 설계에 넣을 것
5. lifecycle script와 workspace는 나중에 붙일 것

한 줄로 요약하면:

이 프로젝트의 본질은 "다운로드 툴"이 아니라 "npm registry metadata를 해석해서 정확한 `node_modules` 트리를 재현하는 엔진"이다.

---

## 22. 참고 링크

공식 npm 문서:

- `package.json`: https://docs.npmjs.com/cli/v11/configuring-npm/package-json
- `package-lock.json`: https://docs.npmjs.com/cli/v8/configuring-npm/package-lock-json
- `registry`: https://docs.npmjs.com/cli/v8/using-npm/registry/
- `workspaces`: https://docs.npmjs.com/cli/v8/using-npm/workspaces/
- `npm Registry API`: https://api-docs.npmjs.com/

추천 Rust crate 문서:

- `clap`: https://docs.rs/crate/clap/latest
- `thiserror`: https://docs.rs/crate/thiserror/latest
- `serde`: https://docs.rs/serde/latest/serde
- `serde_json`: https://docs.rs/serde_json/latest/serde_json/
- `reqwest`: https://docs.rs/reqwest/
- `node_semver`: https://docs.rs/node-semver
- `ssri`: https://docs.rs/ssri
- `package_json`: https://docs.rs/package-json/latest/package_json/struct.PackageJson.html
- `tempfile`: https://docs.rs/crate/tempfile/latest
- `tar`: https://docs.rs/crate/tar/latest
- `flate2`: https://docs.rs/crate/flate2/latest
- `url`: https://docs.rs/crate/url/latest
- `fs-err`: https://docs.rs/crate/fs-err/latest
- `fd-lock`: https://docs.rs/fd-lock
- `ignore`: https://docs.rs/ignore/latest/ignore/
- `globset`: https://docs.rs/globset/latest/globset/
- `petgraph`: https://docs.rs/petgraph/latest/petgraph/
- `indicatif`: https://docs.rs/indicatif
- `tracing`: https://docs.rs/tracing/latest/tracing/
- `miette`: https://docs.rs/crate/miette/latest
