# 커밋 컨벤션 참조 가이드

커밋 메시지는 **영어 Conventional Commits**로 작성하고, 변경을 **논리적 단위**로 나눈다. 이 가이드는 타입·스코프 규칙, 분리 전략, 민감 파일 패턴, 예시를 다룬다.

---

## 1. Conventional Commits 형식 (영어)

```
type(scope): description

[optional body — 한국어 설명 가능]

[optional footer — BREAKING CHANGE, refs #123]
```

- **type**: 변경의 성격 (아래 표).
- **scope**: 영향 범위(모듈/디렉토리/기능). 선택이지만 권장. 예: `auth`, `user`, `api`, `ui`, `deps`.
- **description**: 영어, 소문자 시작, 명령형 현재시제(`add`, `fix`, `remove`), 마침표 없음. 50자 내외 권장.
- **body**: 필요 시 "왜" 변경했는지를 설명. 한국어로 작성해도 된다.
- **footer**: 호환성 깨짐은 `BREAKING CHANGE:`, 이슈 연결은 `refs #123` / `closes #123`.

### 타입 표

| type | 사용 시점 | 예시 |
|------|----------|------|
| `feat` | 새 기능 | `feat(user): add profile read endpoint` |
| `fix` | 버그 수정 | `fix(auth): prevent token refresh loop` |
| `refactor` | 동작 변화 없는 구조 개선 | `refactor(logger): extract formatter` |
| `perf` | 성능 개선 | `perf(query): batch n+1 lookups` |
| `docs` | 문서만 | `docs: document API usage in README` |
| `style` | 포매팅·세미콜론 등(로직 무관) | `style: apply prettier` |
| `test` | 테스트 추가/수정 | `test(user): cover profile edge cases` |
| `build` | 빌드 시스템·의존성 | `build(deps): bump axios to 1.7.0` |
| `ci` | CI 설정 | `ci: add coverage upload step` |
| `chore` | 기타 잡무(빌드·CI 외) | `chore: update .gitignore` |

> 저장소의 기존 `git log --oneline -20`에 다른 컨벤션(예: gitmoji, 한국어 메시지, 대문자 시작)이 일관되게 자리잡았으면 **그 스타일을 따른다.** 컨벤션 일관성이 새 규칙 강제보다 중요하다.

---

## 2. 논리적 단위 분리 전략

각 커밋이 **독립적으로 의미를 갖고 빌드/리뷰/되돌리기가 가능**하도록 나눈다.

### 분리 기준
- **목적별로 분리**: 기능 추가 / 버그 수정 / 리팩토링 / 문서 / 설정은 서로 다른 커밋.
- **관심사별로 분리**: 무관한 두 기능은 한 커밋에 섞지 않는다.
- **순서**: 핵심·선행 변경(예: 의존성·공통 유틸)을 먼저, 그 위에 기능, 마지막에 문서.

### 한 파일이 여러 목적을 담을 때
한 파일에 기능 추가와 무관한 리팩토링이 섞였다면, 가능하면 `git add -p`로 hunk 단위로 나눠 별도 커밋한다. 분리가 과하게 복잡하면 더 우세한 목적의 타입으로 한 커밋에 담되, 메시지에 명확히 기술한다.

### 분리 예시
```
변경: 결제 기능 추가 + 로깅 유틸 리팩토링 + README 갱신 + .env.local 수정

→ 3개 커밋 (.env.local 제외)
1. refactor(logger): extract reusable log formatter
2. feat(payment): add checkout session endpoint
3. docs: add payment setup guide to README
```

---

## 3. 민감 파일 — 커밋 금지

아래 패턴은 변경/untracked 목록에 있어도 **스테이징에서 제외**하고 사용자에게 알린다.

| 분류 | 패턴 예시 |
|------|----------|
| 환경변수 | `.env`, `.env.*`, `*.env` (단 `.env.example`/`.env.sample`은 허용) |
| 비밀키/인증서 | `*.pem`, `*.key`, `*.p12`, `*.pfx`, `id_rsa`, `id_ed25519` |
| 자격증명 | `credentials`, `credentials.json`, `*-credentials*`, `secrets.*`, `*.secret` |
| 클라우드/서비스 키 | `service-account*.json`, `*.gserviceaccount.json`, `aws_credentials` |
| 토큰/계정 | `.npmrc`(토큰 포함 시), `.pypirc`, `.netrc` |

- 의심되면 커밋하지 말고 사용자에게 확인한다.
- 이미 추적 중인 민감 파일을 발견하면 별도로 알린다(이번 커밋에서 제외하는 것만으로는 이력에 이미 남아 있을 수 있음).
- `.gitignore`에 누락된 민감 파일은 `.gitignore` 추가를 제안할 수 있다.

---

## 4. 실전 예시

```bash
# 그룹 1: 공통 유틸 리팩토링 먼저
git add src/common/logger.ts
git commit -m "refactor(logger): extract reusable log formatter"

# 그룹 2: 기능 (관련 파일만)
git add src/payment/checkout.controller.ts src/payment/checkout.service.ts
git commit -m "feat(payment): add checkout session endpoint"

# 그룹 3: 문서
git add README.md
git commit -m "docs: add payment setup guide"

# 확인
git log --oneline -5
git status
```

body가 필요한 경우(왜 바꿨는지):
```bash
git commit -m "fix(auth): prevent infinite token refresh

만료 임박 토큰에서 refresh가 재귀 호출되던 문제를 수정.
refresh 진행 중 플래그로 중복 호출을 차단한다."
```
