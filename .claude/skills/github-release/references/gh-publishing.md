# gh CLI 게시 절차 참조 가이드

GitHub 릴리스 게시는 `gh` CLI로 수행한다. 게시는 **비가역적이고 사용자에게 즉시 노출되는 외부 작업**이므로, 사전 점검 → 본문 준비 → (확인) → 생성 → 검증의 순서를 지킨다.

---

## 1. 사전 점검

```bash
gh auth status                        # repo 권한으로 로그인되어 있는지
git remote -v                         # origin이 대상 리포지토리인지
gh release list --limit 10            # 동일 버전 릴리스가 이미 있는지
git tag -l                            # 동일 태그가 이미 있는지
git fetch --quiet origin
git log --oneline origin/main..HEAD   # 미push 커밋 — 비어 있어야 태그 커밋이 원격에 존재
```

### 판정
- `gh auth status`가 실패하면 → 사용자에게 `gh auth login`을 안내(`! gh auth login`).
- `gh release list` / `git tag -l`에 동일 버전이 **이미 있으면** → 사용자에게 알리고 진행 여부 확인(덮어쓰기 금지).
- **`origin/main..HEAD`가 비어 있어야** 태그될 커밋이 원격에 존재하며 `--target main`이 안전하다. 비어 있지 않으면 push가 필요하다고 알린다.

---

## 2. 본문 파일 준비

헤더(1행)는 릴리스 **제목**(`--title`)으로 분리하고, 나머지를 본문(`--notes-file`)으로 쓴다.

`docs/releases/<version>.md`의 **첫 줄(헤더)을 제외한 내용**을 임시 본문 파일로 만든다. 게시 후 삭제한다.

```bash
# 첫 줄(헤더)을 제외한 본문을 임시 파일로
tail -n +2 docs/releases/v0.1.0.md > /tmp/v0.1.0-body.md
```

> 헤더 다음의 빈 줄까지 포함되어도 무방하다. 본문 첫 줄이 태그라인이 되도록 한다.

---

## 3. 생성

게시 전, 노트 초안과 아래 **정확한 명령**을 사용자에게 보여주고 확인을 받는다.

```bash
gh release create v0.1.0 \
  --target main \
  --title "v0.1.0 🌅 첫 출시 / First Light" \
  --notes-file /tmp/v0.1.0-body.md \
  --latest
```

### 플래그 선택
| 상황 | 플래그 |
|------|--------|
| 정식 릴리스 (최신으로 표시) | `--latest` |
| 사전 배포본 (베타/RC) | `--prerelease` (추가) |
| 검토만, 사람이 나중에 게시 | `--draft` |
| 특정 커밋/브랜치를 태그 | `--target <branch-or-sha>` |

- `gh release create`는 태그가 없으면 `--target`이 가리키는 커밋에 태그를 새로 만든다. 태그를 미리 만들어 두었다면 그 태그를 그대로 사용한다.
- **정식(`--latest`) 게시는 사용자 확인 후 실행.** 확신이 서지 않으면 `--draft`로 만들고 GitHub UI에서 사람이 게시하게 한다.

---

## 4. 게시 후 검증

```bash
gh release view v0.1.0 --json tagName,name,isDraft,isPrerelease,targetCommitish,url
git ls-remote --tags origin v0.1.0    # 태그가 의도한 커밋을 가리키는지 확인
```

확인 항목:
- `tagName` = 의도한 태그(`v0.1.0`)
- `name` = 헤더(제목)
- `isDraft` / `isPrerelease` = 의도한 값
- `targetCommitish` = 의도한 브랜치/커밋
- `url` 접속 시 본문이 올바르게 렌더링되는가
- `ls-remote`의 SHA가 태그될 커밋의 SHA와 일치하는가

> 주의: 일부 `gh` 버전에는 `--json isLatest` 필드가 없다. 오류가 나면 메시지에 사용 가능한 필드 목록이 나오므로 그중에서 고른다.

---

## 5. 정리

```bash
rm -f /tmp/v0.1.0-body.md             # 임시 본문 파일 삭제
git add docs/releases/v0.1.0.md && git commit -m "docs: v0.1.0 릴리스 노트"   # 노트 보관(권장)
```

---

## 6. 흔한 오류와 대처

| 증상 | 원인 | 대처 |
|------|------|------|
| `release already exists` | 동일 태그 릴리스 존재 | `gh release delete`(신중) 또는 `gh release edit`로 수정. 사용자 확인 필수 |
| `tag exists but release missing` | 태그만 있고 릴리스 없음 | `gh release create <tag>`로 기존 태그에 릴리스 생성(태그 새로 안 만듦) |
| `--target` 무시됨 | 이미 태그가 존재 | 태그가 가리키는 커밋이 사용됨. `git ls-remote --tags`로 확인 |
| 본문에 헤더가 중복 노출 | 첫 줄을 제외하지 않음 | `tail -n +2`로 본문 파일 재생성 |
| 인증 실패 | 토큰 만료/권한 부족 | `! gh auth login` 안내(repo 스코프 필요) |
