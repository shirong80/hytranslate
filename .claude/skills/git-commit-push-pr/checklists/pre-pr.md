# PR 전 체크리스트

Pull Request를 생성하기 전에 점검한다.

## 타겟 브랜치 검증 (가장 중요)
- [ ] 인자로 전달된 타겟 브랜치 후보를 먼저 확인했다
- [ ] `git ls-remote --heads origin <후보>`로 원격 존재를 검증했다
- [ ] 인자가 없으면 사용자에게 명시적으로 요청했다 (기본 브랜치를 참고로 제시)
- [ ] 인자가 무효(원격에 없음)이면 알리고 재요청했다
- [ ] **임의의 기본 브랜치(main/master/develop)로 대체하지 않았다**

## 차단 조건 점검
- [ ] 현재 브랜치 != 타겟 브랜치 (같으면 PR 불가)
- [ ] `git log <타겟>..HEAD --oneline`에 커밋이 1개 이상 있다 (없으면 PR 불가)
- [ ] 미커밋 변경이 없다 (있으면 먼저 커밋 확인)
- [ ] `gh pr list --head <현재 브랜치> --state open`로 동일 브랜치 열린 PR을 확인했다 (있으면 안내)

## 인증·push
- [ ] `gh auth status`로 인증을 확인했다 (실패 시 `! gh auth login` 안내)
- [ ] 현재 브랜치를 origin에 push했다 (`git push -u origin HEAD`)

## PR 작성 (한국어)
- [ ] 제목은 70자 이내 한국어, 여러 커밋이면 핵심을 종합했다
- [ ] 본문은 `## 요약` / `## 변경사항` / `## 테스트 계획` 구성 (`templates/pr-body.md`)
- [ ] `--base <타겟> --head <현재 브랜치>`를 명시했다

## 결과 보고
- [ ] 생성된 PR URL을 출력했다
- [ ] `gh pr view --json url,baseRefName,headRefName,title`로 base/head를 확인했다
