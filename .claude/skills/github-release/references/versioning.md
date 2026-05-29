# 버전 규칙 및 매니페스트 동기화 참조 가이드

릴리스 태그와 프로젝트의 버전 선언 파일이 **항상 일치**해야 한다. 불일치는 빌드 산출물의 버전과 릴리스 버전이 어긋나는 혼란을 만든다.

---

## 1. SemVer 규칙

`MAJOR.MINOR.PATCH` 형식을 따른다.

| 증가 | 언제 | 예 |
|------|------|----|
| MAJOR | 하위 호환이 깨지는 변경 | `1.4.2` → `2.0.0` |
| MINOR | 하위 호환되는 기능 추가 | `1.4.2` → `1.5.0` |
| PATCH | 하위 호환되는 버그 수정만 | `1.4.2` → `1.4.3` |

- 0.x 단계(`0.y.z`)는 초기 개발. 호환성 보장이 약하므로 MINOR를 기능 단위로 올린다.
- 사전 배포본은 `-rc.1`, `-beta.1` 등 식별자를 붙일 수 있다(`1.0.0-rc.1`). 이 경우 게시 시 `--prerelease`.

### Git 태그
- 태그에는 `v` 접두사를 붙인다: `v0.1.0`, `v1.5.0`.
- 매니페스트의 `version` 필드에는 보통 `v` 없이 숫자만 쓴다(`0.1.0`). 태그의 `v0.1.0`과 매니페스트의 `0.1.0`이 같은 버전을 가리키면 일치로 본다.

---

## 2. 프로젝트 유형별 버전 매니페스트

릴리스 전 **실제로 존재하는** 매니페스트만 골라 버전을 확인·동기화한다. 없는 파일을 가정하지 않는다.

| 프로젝트 유형 | 매니페스트 | 버전 필드 |
|--------------|-----------|----------|
| Node / Electron | `package.json` | `"version": "0.1.0"` |
| Tauri (Node + Rust) | `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json` | 각 `version` |
| Rust | `Cargo.toml` | `[package] version = "0.1.0"` |
| Python | `pyproject.toml` (또는 `setup.py`, `__init__.py`의 `__version__`) | `version = "0.1.0"` |
| JVM (Gradle) | `build.gradle` / `build.gradle.kts` | `version = "0.1.0"` |
| JVM (Maven) | `pom.xml` | `<version>0.1.0</version>` |
| Go | (태그가 곧 버전) | git 태그 자체 |
| Flutter / Dart | `pubspec.yaml` | `version: 0.1.0` |

### 동기화 점검 예시 (Tauri)
```bash
rg '"version"' package.json
rg '^version' src-tauri/Cargo.toml
rg '"version"' src-tauri/tauri.conf.json
```
세 값이 모두 태그(`v0.1.0`의 `0.1.0`)와 같아야 한다.

---

## 3. 불일치 처리

매니페스트 버전이 태그와 다르면:

1. 어느 쪽이 맞는지 사용자에게 확인한다(보통 태그를 의도한 버전으로 본다).
2. 매니페스트를 태그 버전으로 수정한다.
3. **이 수정을 커밋하고 push**한 뒤에 그 커밋을 태그한다 — 그래야 태그된 커밋의 매니페스트가 올바른 버전을 담는다.
4. push 후 `git log --oneline origin/main..HEAD`가 비어 있는지 재확인한다.

> 매니페스트 수정 커밋을 빠뜨린 채 태그하면, 릴리스 산출물의 내부 버전이 이전 값으로 남는다. 반드시 태그 **전에** 커밋한다.
