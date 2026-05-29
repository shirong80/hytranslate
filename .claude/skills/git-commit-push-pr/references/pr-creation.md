# Pull Request 생성 참조 가이드

PR 생성은 `gh` CLI로 수행한다. PR은 **외부에 노출되고 리뷰어에게 알림이 가는 작업**이므로, 타겟 브랜치 검증 → 사전 검증 → push → 생성 → 결과 확인 순서를 지킨다. PR 제목·본문은 **한국어**로 작성한다.

---

## 1. 타겟 브랜치 확정 (인자 우선 검증 → 무효 시 명시 요청)

PR의 base(타겟) 브랜치는 **반드시 검증된 값**으로만 진행한다. 임의의 기본 브랜치(main/master/develop)로 대체하지 않는다.

```bash
git fetch --quiet origin
git ls-remote --heads origin <후보>      # 한 줄이라도 출력되면 유효
git branch -r                            # 원격 브랜치 목록(참고 제시용)
```

### 판정 규칙
| 상황 | 처리 |
|------|------|
| 인자로 타겟이 전달됨 + 원격에 존재 | 그대로 사용 |
| 인자로 타겟이 전달됨 + 원격에 **없음** | "`<후보>`는 origin에 존재하지 않습니다. 올바른 타겟 브랜치를 알려주세요." 알리고 재요청. **대체 금지.** |
| 인자가 **없음** | 사용자에게 타겟 브랜치를 명시 요청. 감지한 기본 브랜치를 참고로 제시. |

기본 브랜치 감지(참고 제시용):
```bash
git symbolic-ref --short refs/remotes/origin/HEAD 2>/dev/null   # 예: origin/main → main
# 비어 있으면:
gh repo view --json defaultBranchRef --jq .defaultBranchRef.name
```

---

## 2. PR 사전 검증

```bash
gh auth status                               # 인증 (실패 시 `! gh auth login`)
git branch --show-current                    # 현재(head) 브랜치
git log <타겟>..HEAD --oneline               # 타겟 대비 커밋
git diff <타겟>...HEAD --stat                # 변경 파일 요약 (점 3개: merge-base 기준)
gh pr list --head <현재 브랜치> --state open   # 동일 head로 열린 PR
```

### 차단 조건
- **현재 브랜치 == 타겟 브랜치** → PR 불가. 중단하고 알린다.
- **`<타겟>..HEAD` 커밋 없음** → 타겟과 차이가 없어 PR 불가. 알린다.
- **미커밋 변경 존재** → 먼저 커밋할지 확인.
- **동일 head로 열린 PR 존재** → 그 URL을 안내하고, 새로 만들지/기존을 볼지 확인(중복 생성 금지).

> `git diff A...HEAD`(점 3개)는 A와 HEAD의 공통 조상(merge-base) 기준 차이라 PR diff와 일치한다. `git log A..HEAD`(점 2개)는 A에 없고 HEAD에 있는 커밋 목록이다. 용도를 구분해 쓴다.

---

## 3. 원격 push

PR 생성 전 현재 브랜치가 origin에 올라가 있어야 한다.
```bash
git push -u origin HEAD
```
- `gh pr create`는 push되지 않은 브랜치에 대해 push 여부를 물을 수 있으나, 명시적으로 먼저 push해 동작을 예측 가능하게 한다.
- force push는 하지 않는다.

---

## 4. PR 작성 및 생성 (한국어)

타겟 대비 커밋·diff를 분석해 한국어 제목·본문을 작성한다. 양식은 `templates/pr-body.md`.

- **제목**: 70자 이내, 한국어. 여러 커밋이면 종합한 핵심 한 줄.
- **본문**: `## 요약` / `## 변경사항` / `## 테스트 계획`.

```bash
gh pr create --base <타겟> --head <현재 브랜치> --title "사용자 프로필 조회 API 추가" --body "$(cat <<'EOF'
## 요약
- 사용자 프로필 조회 엔드포인트 추가
- 로그 포매터 공통 유틸로 분리

## 변경사항
- `user/profile.controller`, `profile.service`: GET /users/{id}/profile 추가
- `common/logger`: 포매터 추출 및 재사용

## 테스트 계획
- [ ] 프로필 조회 정상 응답 확인
- [ ] 존재하지 않는 사용자 404 확인
EOF
)"
```

### 자주 쓰는 플래그
| 목적 | 플래그 |
|------|--------|
| 타겟(base) 지정 | `--base <branch>` |
| head 브랜치 명시 | `--head <branch>` |
| 초안 PR | `--draft` |
| 리뷰어 지정 | `--reviewer <user>` |
| 라벨 | `--label <label>` |
| 브라우저에서 열기 | `--web` |

---

## 5. 결과 확인

```bash
gh pr view --json url,baseRefName,headRefName,title,state
```
- `baseRefName` = 타겟 브랜치, `headRefName` = 현재 브랜치인지 확인.
- `url`을 사용자에게 출력한다.

---

## 6. 흔한 오류와 대처

| 증상 | 원인 | 대처 |
|------|------|------|
| `pull request already exists for ...` | 동일 head로 열린 PR 존재 | `gh pr view`로 기존 PR 안내, 새로 만들지 확인 |
| `GraphQL: No commits between A and B` | 타겟과 차이 없음 | 커밋 여부 확인, 타겟 브랜치 재확인 |
| `must be a collaborator` / 403 | 권한 부족(포크 등) | 포크 워크플로우(`--repo`, 포크 head) 안내 |
| `could not determine base` | base 미지정 또는 무효 | `--base <타겟>` 명시, 타겟 유효성 재검증 |
| 인증 실패 | 토큰 만료/스코프 부족 | `! gh auth login` 안내(repo 스코프 필요) |
| `head branch ... not found on remote` | push 안 됨 | `git push -u origin HEAD` 먼저 |
