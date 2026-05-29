# 브랜치·원격 처리 참조 가이드 (스택 무관)

이 스킬은 특정 언어·프레임워크·브랜치 전략을 가정하지 않는다. 기본 브랜치명, 원격 구성, 컨벤션을 **저장소 상태에서 직접 감지**해 모든 GitHub 저장소에 동일하게 동작한다.

---

## 1. 기본 브랜치 감지 (main/master 가정 금지)

`main`을 하드코딩하지 않는다. 다음 순서로 감지한다.

```bash
# 1) 원격 HEAD가 가리키는 기본 브랜치
git symbolic-ref --short refs/remotes/origin/HEAD 2>/dev/null   # → origin/main 등

# 2) 비어 있으면 원격에서 갱신
git remote set-head origin --auto 2>/dev/null
git symbolic-ref --short refs/remotes/origin/HEAD 2>/dev/null

# 3) gh로 직접 조회
gh repo view --json defaultBranchRef --jq .defaultBranchRef.name
```

> 기본 브랜치는 **PR 타겟을 사용자에게 제안할 때 참고용**으로만 쓴다. 타겟 브랜치는 사용자 인자/입력으로 확정한다(임의 대체 금지).

---

## 2. 현재 브랜치·원격 확인

```bash
git branch --show-current        # 현재 브랜치 (detached면 빈 출력)
git remote -v                    # 원격 목록 (origin 존재 여부)
git rev-parse --abbrev-ref HEAD  # detached HEAD면 'HEAD' 출력
```

### 엣지케이스
| 상황 | 처리 |
|------|------|
| detached HEAD | 커밋은 가능하나 push/PR 전에 브랜치 체크아웃 권고: `git switch -c <새 브랜치>` |
| origin 원격 없음 | 커밋만 가능. push/PR은 원격 추가(`git remote add origin <url>`) 필요 안내 |
| 원격이 origin이 아닌 이름 | `git remote -v`로 확인 후 실제 원격명 사용 |
| upstream 미설정 | `git push -u origin HEAD`로 첫 push 시 설정 |

---

## 3. push 동작 (스택 무관)

```bash
git push -u origin HEAD          # 현재 브랜치명 그대로 origin에 push + upstream 설정
```

- `HEAD`를 쓰면 현재 브랜치명을 자동 사용해 브랜치명 오타를 막는다.
- **force push 금지**: `--force`/`--force-with-lease`를 임의로 쓰지 않는다. push가 거부되면(non-fast-forward) 원격에 선행 커밋이 있다는 뜻이므로, 원인을 확인하고 사용자에게 알린다(`git fetch` 후 rebase/merge 여부는 사용자 결정).
- 보호 브랜치(protected branch)로의 직접 push가 거부될 수 있다. 그 경우 PR 워크플로우를 안내한다.

---

## 4. 커밋 컨벤션 감지

저장소가 이미 따르는 컨벤션을 존중한다.
```bash
git log --oneline -20            # 기존 메시지 스타일 파악
```
- Conventional Commits가 보이면 그대로 따른다(이 스킬의 기본).
- 일관된 다른 스타일(gitmoji, 한국어 메시지, 티켓 prefix `[JIRA-123]` 등)이 강하게 자리잡았으면 그 스타일에 맞춘다.
- 혼재되어 있으면 영어 Conventional Commits를 기본으로 적용한다.

---

## 5. 보편성 체크 (어떤 저장소든)

- 언어/프레임워크별 빌드·테스트 명령을 **가정하지 않는다.** 테스트 실행은 이 스킬의 범위가 아니다(요청 시에만).
- 민감 파일 패턴은 스택에 독립적인 일반 패턴으로 점검한다(`references/commit-conventions.md` §3).
- `.gitignore`·기본 브랜치·원격명을 저장소에서 읽어 동작하므로, 모노레포·포크·다중 원격 환경에서도 동일하게 적용된다.
