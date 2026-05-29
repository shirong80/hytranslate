# push 전 체크리스트

현재 브랜치를 origin에 push하기 전에 점검한다.

## 범위 확인
- [ ] 요청 범위가 push 이상이다 (커밋만 요청이면 push하지 않는다)

## 상태 확인
- [ ] `git branch --show-current`로 현재 브랜치를 확인했다 (detached HEAD 아님)
- [ ] 미커밋 변경이 남아 있는지 확인했다 (있으면 먼저 커밋할지 사용자 확인)
- [ ] origin 원격이 존재한다 (`git remote -v`)

## push 실행
- [ ] `git push -u origin HEAD`로 현재 브랜치명 그대로 push했다
- [ ] **force push(`--force`/`--force-with-lease`)를 사용하지 않았다**
- [ ] push가 거부되면(non-fast-forward) 원인을 확인하고 사용자에게 알렸다 (임의 rebase/force 금지)
- [ ] 보호 브랜치 직접 push 거부 시 PR 워크플로우를 안내했다

## 결과 보고
- [ ] 원격 브랜치명과 push된 커밋 범위를 보고했다
