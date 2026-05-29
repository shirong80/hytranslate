# 커밋 계획 제시 양식

커밋을 실행하기 전에, 변경을 논리적 단위로 나눈 계획을 아래 표로 제시한다. 사용자가 한눈에 "무엇이 어떤 커밋으로 묶이는지" 파악할 수 있게 한다.

---

## 양식

```markdown
## 커밋 계획

현재 브랜치: `{현재 브랜치}`

| # | 파일 | 타입 | 커밋 메시지 |
|---|------|------|------------|
| 1 | {파일 또는 파일 그룹} | {feat/fix/...} | `{type(scope): description}` |
| 2 | {파일 또는 파일 그룹} | {type} | `{type(scope): description}` |
| 3 | {파일 또는 파일 그룹} | {type} | `{type(scope): description}` |

> 제외: {민감 파일 또는 의도적으로 커밋하지 않는 파일과 이유}
```

---

## 작성 가이드

- **순서**: 선행/공통 변경(의존성·유틸·리팩토링)을 위쪽, 그 위에 기능, 문서는 아래쪽.
- **파일 열**: 같은 커밋에 묶이는 파일을 한 행에. 파일이 많으면 대표 경로 + "외 N개"로 축약.
- **타입 열**: Conventional Commits 타입.
- **메시지 열**: 실제 사용할 영어 커밋 메시지(백틱으로 감쌈).
- **제외 줄**: 민감 파일(`.env` 등)이나 커밋하지 않을 파일이 있으면 이유와 함께 명시. 없으면 생략.

---

## 예시

```markdown
## 커밋 계획

현재 브랜치: `feature/user-profile`

| # | 파일 | 타입 | 커밋 메시지 |
|---|------|------|------------|
| 1 | src/common/logger.ts | refactor | `refactor(logger): extract reusable log formatter` |
| 2 | src/user/profile.controller.ts, profile.service.ts | feat | `feat(user): add profile read endpoint` |
| 3 | README.md | docs | `docs: document profile API usage` |

> 제외: `.env.local` — 환경변수 파일이라 스테이징에서 제외했습니다.
```
