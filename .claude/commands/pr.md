---
description: "타겟 브랜치로 Pull Request 생성"
---

현재 브랜치의 변경사항을 `$ARGUMENTS` 브랜치로 PR 요청해주세요.

## 수행 단계

1. **현재 상태 확인**
   - `git status`로 커밋되지 않은 변경사항 확인

   - 커밋되지 않은 변경사항이 있으면 먼저 커밋할지 사용자에게 확인

   - `git branch --show-current`로 현재 브랜치 확인

   - 현재 브랜치와 타겟 브랜치가 동일하면 에러 안내

2. **변경사항 분석**
   - `git log $ARGUMENTS..HEAD --oneline`으로 타겟 대비 커밋 목록 확인

   - `git diff $ARGUMENTS...HEAD --stat`으로 변경 파일 요약 확인

   - 커밋이 없으면 PR 생성 불가 안내

3. **리모트 푸시**
   - `git push -u origin HEAD`로 현재 브랜치를 리모트에 푸시

4. **PR 생성**
   - 커밋 내역과 변경사항을 분석하여 PR 제목과 본문 작성

   - PR 제목은 70자 이내, 한국어로 작성

   - `gh pr create` 명령으로 PR 생성:

   ```
   gh pr create --base $ARGUMENTS --title "PR 제목" --body "$(cat <<'EOF'
   ## Summary
   - 변경사항 요약 (bullet points)

   ## Changes
   - 주요 변경 파일 및 내용

   ## Test plan
   - [ ] 테스트 항목
   EOF
   )"
   ```

5. **결과 확인**
   - 생성된 PR URL 출력

## 주의사항

- `$ARGUMENTS`가 비어있으면 사용자에게 타겟 브랜치를 입력하도록 안내

- PR 제목은 커밋 메시지를 기반으로 간결하게 작성

- PR 본문은 모든 커밋의 변경사항을 종합하여 작성

- 이미 동일 브랜치로 열린 PR이 있는지 확인하고, 있으면 안내
