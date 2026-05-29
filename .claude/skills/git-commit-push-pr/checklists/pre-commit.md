# 커밋 전 체크리스트

커밋을 실행하기 전에 아래 항목을 점검한다.

## 변경 파악
- [ ] `git status`로 staged/unstaged/untracked를 모두 확인했다
- [ ] `git diff` / `git diff --staged`로 각 변경의 실제 내용을 읽었다 (메시지를 근거 있게 쓰기 위함)

## 민감 파일 차단 (스테이징 제외)
- [ ] `.env`, `.env.*`, `*.env` (단 `.env.example`/`.env.sample`은 허용)
- [ ] `*.pem`, `*.key`, `*.p12`, `*.pfx`, `id_rsa`, `id_ed25519`
- [ ] `credentials`, `credentials.json`, `*-credentials*`, `secrets.*`, `*.secret`
- [ ] `service-account*.json`, `*.gserviceaccount.json`, `aws_credentials`
- [ ] `.npmrc`(토큰 포함 시), `.pypirc`, `.netrc`
- [ ] 위 패턴 발견 시 스테이징에서 제외하고 사용자에게 알렸다
- [ ] 이미 추적 중인 민감 파일이면 별도로 경고했다

## 논리적 단위
- [ ] 관련 변경끼리 그룹화했고, 무관한 변경을 한 커밋에 섞지 않았다
- [ ] 기능/수정/리팩토링/문서/설정을 목적별로 분리했다
- [ ] 각 커밋이 독립적으로 의미를 갖는다
- [ ] 커밋 순서가 합리적이다(선행·공통 변경 → 기능 → 문서)

## 메시지 컨벤션
- [ ] 영어 Conventional Commits(`type(scope): description`)를 따랐다
- [ ] 타입이 변경 성격과 일치한다(feat/fix/refactor/docs/style/test/build/ci/chore/perf)
- [ ] description이 명령형·소문자·마침표 없음
- [ ] 저장소 기존 컨벤션이 다르면 그 스타일에 맞췄다

## 계획 제시
- [ ] 커밋 실행 전 논리적 단위 계획을 표로 제시했다 (`templates/commit-plan.md`)
