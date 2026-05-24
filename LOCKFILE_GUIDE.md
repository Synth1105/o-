# npm 스타일 Lockfile 만들기

이 문서는 Rust로 JavaScript 패키지 매니저를 만들 때 `package-lock.json` 같은 lockfile을 어떻게 설계하고 생성할지 정리한 가이드다.

대상은 다음이다.

- `package.json`의 range를 exact version으로 고정하고 싶다
- 설치한 `node_modules` 트리를 재현 가능하게 만들고 싶다
- 다음 `install`에서 resolve 결과를 재사용하고 싶다
- `ci` 모드에서 lockfile만 믿고 설치하고 싶다

즉, 핵심은 "설치 결과를 파일로 고정하는 법"이다.

---

## 1. lockfile이 왜 필요한가

`package.json`은 의도이고, lockfile은 결과다.

예를 들어:

```json
{
  "dependencies": {
    "react": "^18.3.0"
  }
}
```

여기서 `^18.3.0`은 range다. 실제 설치 시점에는 예를 들면 `18.3.1`이 골라질 수 있다. 그리고 그 `react`가 의존하는 하위 패키지 버전도 같이 결정된다.

lockfile은 이 결과를 기록한다.

즉 lockfile이 있어야:

- 같은 프로젝트를 다른 머신에서 같은 트리로 설치 가능
- `ci`에서 resolve 없이 재현 가능
- registry 상태가 바뀌어도 install 결과가 덜 흔들림
- integrity를 다시 검증 가능

npm 공식 문서도 `package-lock.json`을 설치 트리 재현용 파일로 설명한다.  
출처: https://docs.npmjs.com/cli/v11/configuring-npm/package-lock-json/

---

## 2. 어떤 파일을 만들 것인가

npm 생태계에는 보통 두 종류가 있다.

### 2.1 프로젝트 루트 lockfile

경로:

```text
<project>/package-lock.json
```

이건 git에 들어갈 수 있고, 사용자가 보는 주 lockfile이다.

### 2.2 hidden lockfile

경로:

```text
<project>/node_modules/.package-lock.json
```

npm 문서상 npm v7+는 `node_modules`를 매번 다시 스캔하지 않기 위해 hidden lockfile도 사용한다.  
출처: https://docs.npmjs.com/cli/v11/configuring-npm/package-lock-json/

### 2.3 추천 순서

처음 구현할 때는:

1. 먼저 `package-lock.json`만 만든다
2. 그 다음 성능 최적화 단계에서 hidden lockfile을 붙인다

이 순서가 맞다.

---

## 3. 최소한의 목표

1차 구현에서는 npm lockfile 완전 호환보다 아래를 만족하는 것이 더 중요하다.

1. `package.json`의 range가 exact version으로 고정된다
2. 모든 resolved package에 대해 `resolved`와 `integrity`가 저장된다
3. dependency 관계가 기록된다
4. 다음 install에서 같은 트리를 재현할 수 있다

즉 형식보다 의미가 먼저다.

---

## 4. lockfile에 들어가야 하는 핵심 필드

npm 공식 문서 기준으로 lockfile에는 최소한 이런 개념이 있다.

- `name`
- `version`
- `lockfileVersion`
- `packages`
- dependency flags와 resolved metadata

출처: https://docs.npmjs.com/cli/v11/configuring-npm/package-lock-json/

너의 구현 기준으로 최소 필드는 이 정도면 충분하다.

### 4.1 루트 필드

```json
{
  "name": "my-app",
  "version": "1.0.0",
  "lockfileVersion": 3,
  "packages": {}
}
```

권장 필드:

- `name`
- `version`
- `lockfileVersion`
- `packages`

### 4.2 각 패키지 엔트리 필드

각 패키지 엔트리에는 최소한 아래가 있어야 한다.

- `version`
- `resolved`
- `integrity`
- `dependencies`

상황에 따라:

- `dev`
- `optional`
- `bin`
- `engines`

도 나중에 추가할 수 있다.

---

## 5. 추천 스키마

너처럼 npm registry에서 tarball을 받고 nested `node_modules`를 만드는 구현이라면, `packages` 맵을 경로 기반으로 저장하는 쪽이 좋다.

예시:

```json
{
  "name": "my-app",
  "version": "1.0.0",
  "lockfileVersion": 3,
  "packages": {
    "": {
      "name": "my-app",
      "version": "1.0.0",
      "dependencies": {
        "react": "^18.3.0"
      }
    },
    "node_modules/react": {
      "version": "18.3.1",
      "resolved": "https://registry.npmjs.org/react/-/react-18.3.1.tgz",
      "integrity": "sha512-...",
      "dependencies": {
        "loose-envify": "^1.1.0"
      }
    },
    "node_modules/react/node_modules/loose-envify": {
      "version": "1.4.0",
      "resolved": "https://registry.npmjs.org/loose-envify/-/loose-envify-1.4.0.tgz",
      "integrity": "sha512-..."
    }
  }
}
```

여기서 중요한 점:

- key는 설치 경로다
- 루트 프로젝트는 빈 문자열 `""`을 key로 둔다
- range는 루트만 들고, 설치된 패키지는 exact version을 가진다

이 구조가 현재 구현한 nested install과 가장 잘 맞는다.

---

## 6. 왜 `packages`를 경로 기반으로 두는가

이유는 단순하다.

- JS dependency는 같은 이름의 패키지가 트리 여러 위치에 중복 설치될 수 있다
- 이름만으로는 설치 위치를 구분할 수 없다
- `foo@1.0.0`이 루트에도 있고 `bar/node_modules/foo`에도 있을 수 있다

그래서 key는 보통:

- package name이 아니라
- install path

로 두는 게 맞다.

이 방식이면 nested dependency도 정확히 복원할 수 있다.

---

## 7. lockfileVersion은 뭘 써야 하나

npm 공식 문서 기준으로:

- npm v7+에서 hidden lockfile은 `lockfileVersion: 3`
- 일반 lockfile도 최신 문서 기준 v3 계열을 쓴다

출처: https://docs.npmjs.com/cli/v11/configuring-npm/package-lock-json/

### 7.1 추천

새 구현이면 `3`으로 시작하는 게 맞다.

이유:

- 하위 호환성 레이어를 억지로 넣지 않아도 된다
- 너는 npm v6 호환용 lockfile을 만들려는 게 아니다
- path 기반 `packages` 구조와 잘 맞다

즉:

```json
{
  "lockfileVersion": 3
}
```

로 시작하면 된다.

---

## 8. 루트 엔트리와 설치 엔트리를 구분해야 한다

lockfile에서 루트 프로젝트 엔트리는 일반 dependency 엔트리와 역할이 다르다.

### 8.1 루트 엔트리

예시:

```json
"": {
  "name": "my-app",
  "version": "1.0.0",
  "dependencies": {
    "react": "^18.3.0"
  }
}
```

루트 엔트리에는:

- 프로젝트 이름
- 프로젝트 버전
- 선언된 range

를 저장한다.

### 8.2 설치 엔트리

예시:

```json
"node_modules/react": {
  "version": "18.3.1",
  "resolved": "https://registry.npmjs.org/react/-/react-18.3.1.tgz",
  "integrity": "sha512-...",
  "dependencies": {
    "loose-envify": "^1.1.0"
  }
}
```

설치 엔트리에는:

- exact version
- tarball URL
- integrity
- 그 패키지가 선언한 child dependency range

를 저장한다.

---

## 9. 어떤 시점에 lockfile 데이터를 모아야 하나

lockfile은 install이 끝난 뒤 아무렇게나 역으로 스캔해서 만들 수도 있지만, 그 방식은 비효율적이고 오류가 많다.

더 나은 방식은 설치 중간에 resolved metadata를 수집하는 것이다.

### 9.1 추천 구조

```rust
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Lockfile {
    pub name: Option<String>,
    pub version: Option<String>,
    #[serde(rename = "lockfileVersion")]
    pub lockfile_version: u32,
    pub packages: std::collections::BTreeMap<String, LockedPackage>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct LockedPackage {
    pub version: Option<String>,
    pub resolved: Option<String>,
    pub integrity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependencies: Option<std::collections::BTreeMap<String, String>>,
}
```

### 9.2 설치 파이프라인에서 언제 push하나

이 시점이 좋다.

1. 패키지 resolve 완료
2. tarball/integrity 확보
3. 설치 경로 결정
4. 설치 성공
5. lockfile collector에 엔트리 추가

즉 "설치 성공 후 기록"이 맞다.

---

## 10. 현재 코드 기준으로 필요한 collector

지금 구현은 `ResolvedPackage`를 만든 뒤 tarball을 받고 nested install을 재귀적으로 돈다. 여기에 collector를 끼우면 된다.

예시:

```rust
pub struct LockCollector {
    pub packages: std::collections::BTreeMap<String, LockedPackage>,
}
```

그리고 설치 함수 시그니처를 이런 식으로 확장한다.

```rust
fn install_dependency(
    client: &Client,
    name: &str,
    range: &str,
    node_modules_dir: &Path,
    installed: &mut HashSet<String>,
    lock: &mut LockCollector,
) -> io::Result<()>
```

설치가 끝난 뒤:

- 실제 install path를 relative path로 바꾸고
- 그 key에 `LockedPackage`를 넣는다

예시 key:

- `node_modules/react`
- `node_modules/react/node_modules/loose-envify`

---

## 11. 경로를 어떻게 key로 만들까

프로젝트 루트 기준 상대경로로 만드는 게 맞다.

예시 함수:

```rust
fn lockfile_key(project_root: &Path, package_dir: &Path) -> String {
    if project_root == package_dir {
        return String::new();
    }

    package_dir
        .strip_prefix(project_root)
        .unwrap()
        .to_string_lossy()
        .replace('\\', "/")
}
```

결과 예시:

- 프로젝트 루트: `""`
- `project/node_modules/react`: `node_modules/react`
- `project/node_modules/react/node_modules/loose-envify`: `node_modules/react/node_modules/loose-envify`

Windows도 생각하면 separator는 `/`로 정규화하는 게 좋다.

---

## 12. 설치 중 어떤 데이터를 저장해야 하나

`ResolvedPackage`에서 바로 얻을 수 있는 것:

- name
- version
- tarball_url
- integrity
- dependencies

설치 단계에서만 알 수 있는 것:

- 실제 설치 경로
- nested 위치 여부
- 루트 기준 상대 key

그래서 lockfile 엔트리 생성은 resolve 단계만으로 끝나지 않고, install 단계 정보까지 합쳐야 한다.

---

## 13. 쓰기 순서

lockfile은 마지막에 한 번만 쓰는 게 안전하다.

추천 순서:

1. 루트 manifest 읽기
2. collector 초기화
3. 루트 엔트리 생성
4. 모든 dependency 설치
5. collector를 `Lockfile`로 변환
6. temp file에 JSON 쓰기
7. `package-lock.json`으로 atomic rename

### 13.1 왜 마지막에 써야 하나

중간에 실패했는데 lockfile이 먼저 써지면:

- lockfile은 완성됐는데 `node_modules`는 덜 깔린 상태

가 생긴다.

즉:

- `node_modules`
- lockfile

둘 다 커밋 가능한 상태가 되었을 때만 기록해야 한다.

---

## 14. JSON 출력 정책

lockfile은 사람이 직접 자주 수정하는 파일이 아니므로, 안정적인 정렬과 예측 가능한 출력이 더 중요하다.

권장:

- `BTreeMap` 사용
- `serde_json::to_vec_pretty` 또는 `to_string_pretty`
- 마지막 newline 추가

예시:

```rust
let json = serde_json::to_vec_pretty(&lockfile)?;
fs::write(temp_path, json)?;
```

### 14.1 key 정렬이 중요한 이유

- git diff 안정화
- 테스트 snapshot 안정화
- 재생성 시 노이즈 감소

---

## 15. `install`과 `ci`의 차이

lockfile을 만들면 곧바로 `ci` 모드도 설계해야 한다.

### 15.1 install

- `package.json`을 기준으로 resolve 가능
- lockfile이 있으면 참고할 수 있음
- 필요하면 lockfile 갱신 가능

### 15.2 ci

- `package-lock.json`이 반드시 있어야 함
- `package.json`과 lockfile이 어긋나면 실패
- resolve 재계산을 최대한 하지 않음
- lockfile에 적힌 exact tree를 그대로 설치

즉 lockfile 생성은 결국 `ci`를 위한 기반이기도 하다.

---

## 16. hidden lockfile은 나중에 어떻게 붙이나

npm 문서상 hidden lockfile은 `node_modules/.package-lock.json`에 저장된다.  
출처: https://docs.npmjs.com/cli/v11/configuring-npm/package-lock-json/

너도 나중에 이걸 붙일 수 있다.

### 16.1 추천 전략

처음에는:

- 루트 `package-lock.json`만 생성

그 다음 단계에서:

- 설치 직후 같은 정보로 hidden lockfile도 생성

### 16.2 hidden lockfile의 목적

- 다음 install에서 `node_modules`를 빠르게 신뢰
- 전체 디렉터리 스캔 비용 감소

이건 correctness보다 성능 최적화에 가깝다.

---

## 17. 현재 코드에 바로 넣는 방법

현재 `src/pm.rs` 기준으로는 이렇게 붙이면 된다.

### 17.1 새 타입 추가

- `Lockfile`
- `LockedPackage`
- `LockCollector`

### 17.2 `install_from`에서 collector 생성

```rust
let mut lock = LockCollector::new();
lock.insert_root(&manifest);
```

### 17.3 `install_dependency`에 `lock` 전달

설치 성공 직후:

```rust
lock.insert_package(
    project_root,
    &target_dir,
    LockedPackage {
        version: Some(resolved.version.clone()),
        resolved: Some(resolved.tarball_url.clone()),
        integrity: resolved.integrity.clone(),
        dependencies: Some(resolved.dependencies.clone().into_iter().collect()),
    },
);
```

### 17.4 마지막에 write

```rust
write_lockfile(project_root, lock.into_lockfile(root_name, root_version))?;
```

이 구조가 제일 덜 꼬인다.

---

## 18. 추천 helper 함수들

필요한 함수는 대략 이 정도다.

```rust
fn build_root_lock_entry(manifest: &PackageJson) -> LockedPackage;
fn lockfile_key(project_root: &Path, package_dir: &Path) -> String;
fn write_lockfile(project_root: &Path, lockfile: &Lockfile) -> io::Result<()>;
fn write_json_atomic(path: &Path, bytes: &[u8]) -> io::Result<()>;
```

원자적 쓰기를 위해선:

1. 같은 디렉터리에 temp file 생성
2. write + flush
3. rename

패턴을 쓰면 된다.

---

## 19. 실수하기 쉬운 부분

- 루트 엔트리와 설치 엔트리를 같은 방식으로 저장
- 경로 key를 절대경로로 저장
- Windows path separator를 그대로 저장
- 설치 실패 전에 lockfile 먼저 쓰기
- `resolved`나 `integrity` 없이 version만 저장
- nested dependency를 root key에 덮어쓰기
- 루트의 declared range와 installed exact version을 섞어 쓰기

이건 나중에 `ci` 구현 때 바로 문제 된다.

---

## 20. 가장 실용적인 결론

너의 현재 구현에는 아래가 가장 잘 맞는다.

1. `lockfileVersion`은 `3`
2. `packages`는 path 기반 key
3. 루트 엔트리는 `""`
4. 설치 엔트리는 exact version + resolved + integrity + dependencies
5. collector는 install 과정에서 채움
6. 모든 설치가 성공한 뒤 `package-lock.json`을 마지막에 한 번만 씀

이 방식이면 지금 구현한 nested `node_modules` 설치기와 가장 자연스럽게 이어진다.

---

## 참고 링크

- npm `package-lock.json` 문서: https://docs.npmjs.com/cli/v11/configuring-npm/package-lock-json/
- npm v8 `package-lock.json` 문서: https://docs.npmjs.com/cli/v8/configuring-npm/package-lock-json
- npm v6 `package-lock.json` 문서: https://docs.npmjs.com/cli/v6/configuring-npm/package-lock-json/?v=true
