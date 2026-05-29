---
name: git-commit-push-pr
description: "현재 브랜치의 작업을 논리적 단위로 나눠 커밋하고(영어 Conventional Commits), 원격(origin)에 push하고, 사용자가 지정한 타겟 브랜치로 Pull Request를 생성하는 git 워크플로우 스킬. 커밋 메시지는 영어 컨벤션, PR 제목·본문은 한국어로 작성한다. 기술 스택(Node/Java/Python/Go/Rust 등)과 무관하게 모든 GitHub 저장소에 보편적으로 적용된다. 반드시 사용해야 하는 경우: '커밋', 'commit', '커밋해줘', '변경사항 커밋', '작업 커밋', '푸시', 'push', '푸시해줘', '커밋하고 푸시', 'PR', 'pr 생성', 'PR 올려줘', 'PR 만들어줘', 'PR 요청', '풀리퀘스트', 'pull request', '머지 요청', '리뷰 요청', 'commit and push', 'open a PR', 'create a pull request', 'raise a PR' 등의 키워드가 포함된 요청. 사용자가 'develop 브랜치로 PR 올려줘', '지금까지 작업 커밋하고 푸시해줘', '변경사항 정리해서 커밋해줘'처럼 커밋·푸시·PR 중 하나라도 원할 때 이 스킬을 사용한다. 단, 버전 태깅·릴리스 게시(gh release, 릴리스 노트)와는 다른 작업이며 그 경우엔 이 스킬을 쓰지 않는다."
---

# Git 커밋·푸시·PR 스킬

현재 브랜치의 변경사항을 **논리적 단위로 커밋 → origin에 push → 타겟 브랜치로 PR 생성**하는 git 워크플로우를 수행한다. 요청 범위에 따라 커밋만, 커밋+푸시, 또는 PR까지 단계를 선택해 실행한다. 특정 언어·프레임워크를 가정하지 않고 저장소 상태를 직접 읽어 동작하므로 모든 GitHub 저장소에 적용된다.

---

## 역할

당신은 git 워크플로우를 안전하고 깔끔하게 수행하는 어시스턴트다. 커밋 이력은 협업자가 변경을 추적하는 단일 출처이고, push와 PR은 한번 나가면 외부에 노출되는 작업임을 이해한다.

**핵심 원칙**:

1. **요청 범위에 맞는 단계만 실행한다.** "커밋"이면 커밋까지, "푸시"면 push까지, "PR/풀리퀘스트"면 PR까지 수행한다. 사용자가 명시하지 않은 외부 작업(push, PR)을 임의로 진행하지 않는다. ("왜": push·PR은 되돌리기 번거로운 외부 노출 작업이라, 요청하지 않은 단계까지 진행하면 의도치 않은 공개·리뷰 요청이 발생한다.)

2. **커밋은 논리적 단위로 나눈다.** 관련된 변경끼리 묶어 각 커밋이 독립적으로 의미를 갖게 한다. 기능 추가·버그 수정·리팩토링·문서·설정은 서로 다른 커밋으로 분리한다. ("왜": 한 커밋에 무관한 변경이 섞이면 리뷰·되돌리기·체리픽이 어려워진다.)

3. **커밋 메시지는 영어 Conventional Commits, PR 제목·본문은 한국어로 쓴다.** 커밋은 `type(scope): description` 형식의 영어, PR은 한국어로 작성한다. (상세: `references/commit-conventions.md`)

4. **기술 스택과 기본 브랜치를 가정하지 않고 저장소에서 직접 감지한다.** 기본 브랜치가 `main`인지 `master`인지, 커밋 컨벤션이 무엇인지, 어떤 파일이 민감한지를 저장소의 실제 상태(`git`, 기존 커밋 로그)에서 확인한다. (상세: `references/branch-and-remote.md`)

5. **민감 파일은 절대 커밋하지 않는다.** `.env`, 자격증명, 비밀키, 토큰 등이 변경 목록에 있으면 스테이징에서 제외하고 사용자에게 알린다. (패턴: `checklists/pre-commit.md`)

6. **PR 타겟 브랜치는 반드시 검증된 값으로만 진행한다.** 인자로 받은 타겟 브랜치를 우선 검증하고, 부재하거나 유효하지 않으면 사용자에게 명시적으로 다시 요청한다. 임의의 기본 브랜치로 대체하지 않는다. (상세: `references/pr-creation.md`)

사용자와의 커뮤니케이션은 한국어를 기본으로 한다. 단, 커밋 메시지·브랜치명·명령어·코드는 원문(영어/코드)을 유지한다.

---

## 프로세스

작업은 **0단계 범위 판별 → 1단계 커밋 → 2단계 push → 3단계 PR** 순서로 진행한다. 0단계에서 결정한 범위까지만 실행한다.

### 0단계: 요청 범위 판별 및 사전 점검

먼저 **어디까지 실행할지** 결정한다.

| 요청 신호 | 실행 범위 |
|----------|----------|
| "커밋", "commit", "변경사항 정리" (push·PR 언급 없음) | 1단계까지 (커밋) |
| "푸시", "push", "커밋하고 푸시" | 1~2단계 (커밋 + push) |
| "PR", "pull request", "풀리퀘스트", "머지 요청" 또는 **타겟 브랜치 인자 존재** | 1~3단계 (커밋 + push + PR) |

이어서 저장소 사전 점검을 수행한다:

```bash
git rev-parse --is-inside-work-tree     # git 저장소인지 확인
git branch --show-current               # 현재 브랜치
git remote -v                           # origin 원격 확인
```

- git 저장소가 아니면 중단하고 알린다.
- 현재 브랜치가 detached HEAD이면 사용자에게 알리고 브랜치 체크아웃을 권한다.
- origin 원격이 없으면 push·PR 단계는 진행 불가임을 알린다(커밋은 가능).

### 1단계: 커밋 (현재 브랜치)

현재 브랜치에서 변경사항을 논리적 단위로 나눠 커밋한다.

1. **변경사항 확인**:
   ```bash
   git status                # staged/unstaged/untracked
   git diff                  # unstaged 변경 내용
   git diff --staged         # 이미 staged된 변경 내용
   ```
   각 파일이 무엇을 바꿨는지 diff를 읽어 파악한다. 커밋 메시지를 변경 내용 근거로 작성하기 위함이다.

2. **민감 파일 스크리닝**: 변경/untracked 목록에 `.env*`, `*.pem`, `*.key`, `id_rsa`, `credentials`, `*.p12`, `secrets.*` 등이 있으면 **스테이징에서 제외**하고 사용자에게 알린다. (전체 패턴: `checklists/pre-commit.md`)

3. **논리적 단위로 분류 및 계획 제시**: 관련 변경을 그룹화해 커밋 계획을 표로 제시한다. 각 그룹 = 하나의 완성된 변경 단위. (양식: `templates/commit-plan.md`)

4. **그룹별 순차 커밋**: 핵심 변경부터, 그룹별로 `git add <관련 파일>` 후 Conventional Commit으로 커밋한다.
   ```bash
   git add <관련 파일들>
   git commit -m "feat(scope): add user profile endpoint"
   ```
   - 타입: `feat`, `fix`, `refactor`, `docs`, `style`, `test`, `chore`, `perf`, `build`, `ci`
   - 메시지는 **영어**, 본문 설명이 필요하면 한국어 본문을 덧붙일 수 있다. (상세: `references/commit-conventions.md`)
   - 저장소 기존 커밋 로그(`git log --oneline -20`)에 다른 컨벤션이 강하게 자리잡았으면 그 스타일을 따른다.

5. **커밋 결과 확인**:
   ```bash
   git log --oneline -10
   git status                # 남은 변경 확인
   ```

요청 범위가 1단계까지면 여기서 결과를 보고하고 종료한다.

### 2단계: push (현재 브랜치)

요청 범위가 2단계 이상일 때만 수행한다.

1. **미커밋 변경 확인**: `git status`에 커밋되지 않은 변경이 남아 있으면 1단계를 먼저 수행할지 사용자에게 확인한다(또는 의도된 잔여 변경인지 확인).
2. **현재 브랜치를 origin에 push**:
   ```bash
   git push -u origin HEAD
   ```
   - 현재 브랜치명을 그대로 사용해 origin에 push하고 upstream을 설정한다.
   - **force push(`--force`)는 사용하지 않는다.** 거부되면 원인(원격 선행 커밋 등)을 확인하고 사용자에게 알린다. (상세: `checklists/pre-push.md`)
3. push 결과(원격 브랜치, 커밋 범위)를 보고한다.

요청 범위가 2단계까지면 여기서 종료한다.

### 3단계: Pull Request 생성

요청 범위가 PR일 때만 수행한다. 절차 상세·엣지케이스는 `references/pr-creation.md`, 점검은 `checklists/pre-pr.md`.

#### 3-1. 타겟 브랜치 확정 (인자 우선 검증 → 무효 시 명시 요청)

1. **인자 확인**: 스킬 호출 시 전달된 인자(타겟 브랜치 후보)를 먼저 확인한다.
2. **유효성 검증**: 후보가 origin에 실제 존재하는 브랜치인지 확인한다.
   ```bash
   git fetch --quiet origin
   git ls-remote --heads origin <후보>     # 결과가 있으면 유효한 원격 브랜치
   ```
3. **판정**:
   - 인자가 **없으면** → 사용자에게 타겟 브랜치를 명시적으로 요청한다. 이때 감지한 기본 브랜치를 참고로 제시한다(예: "타겟 브랜치를 알려주세요. 이 저장소의 기본 브랜치는 `main`입니다.").
   - 인자가 **유효하지 않으면**(원격에 없는 브랜치) → "`<후보>`는 origin에 존재하지 않는 브랜치입니다. 올바른 타겟 브랜치를 알려주세요." 라고 알리고 다시 요청한다. **임의의 기본 브랜치로 대체하지 않는다.**
   - 인자가 **유효하면** → 그대로 사용한다.

#### 3-2. PR 사전 검증

```bash
gh auth status                              # gh 인증 (실패 시 `! gh auth login` 안내)
git branch --show-current                   # 현재 브랜치
git log <타겟>..HEAD --oneline              # 타겟 대비 커밋 목록
git diff <타겟>...HEAD --stat               # 변경 파일 요약
gh pr list --head <현재 브랜치> --state open  # 동일 브랜치로 열린 PR 존재 여부
```

- **현재 브랜치 == 타겟 브랜치**면 PR을 만들 수 없으므로 중단하고 알린다.
- `<타겟>..HEAD`에 **커밋이 없으면** PR 생성 불가임을 알린다(타겟과 차이가 없음).
- 미커밋 변경이 있으면 먼저 커밋할지 확인한다.
- **동일 브랜치로 이미 열린 PR이 있으면** 그 URL을 안내하고, 새로 만들지/기존 PR을 볼지 확인한다.

#### 3-3. 원격 push (아직 안 했으면)

2단계를 건너뛴 경우 현재 브랜치를 origin에 push한다.
```bash
git push -u origin HEAD
```

#### 3-4. PR 작성 및 생성

타겟 대비 커밋·diff를 분석해 **한국어** PR 제목·본문을 작성한다. (본문 양식: `templates/pr-body.md`)

- 제목: 70자 이내, 한국어. 변경의 핵심을 요약(여러 커밋이면 종합).
- 본문: `## 요약`, `## 변경사항`, `## 테스트 계획` 구성.

```bash
gh pr create --base <타겟> --head <현재 브랜치> --title "PR 제목" --body "$(cat <<'EOF'
## 요약
- 변경사항 요약 (bullet)

## 변경사항
- 주요 변경 파일 및 내용

## 테스트 계획
- [ ] 테스트 항목
EOF
)"
```

#### 3-5. 결과 확인

생성된 PR URL을 출력하고, base/head/제목을 함께 보고한다.
```bash
gh pr view --json url,baseRefName,headRefName,title
```

---

## 출력 형식

수행한 단계에 해당하는 항목만 순서대로 보고한다.

1. **범위·사전 점검**: 실행할 단계, 현재 브랜치, origin 상태.
2. **커밋 계획 및 결과**: 논리적 단위 표(파일↔타입↔메시지) + 실제 생성된 커밋 로그(`git log --oneline`).
3. **push 결과**: 원격 브랜치명, push된 커밋 범위.
4. **PR 결과**: 타겟 브랜치, PR 제목, **PR URL**.

---

## 제약 조건

- **커밋 메시지는 영어 Conventional Commits, PR 제목·본문은 한국어**로 작성한다.
- **요청한 단계까지만 실행한다.** "커밋"만 요청했는데 push·PR을 임의로 진행하지 않는다.
- **각 커밋은 독립적으로 의미가 있어야 한다.** 무관한 변경을 한 커밋에 섞지 않는다.
- **민감 파일(`.env`, 자격증명, 비밀키 등)은 커밋하지 않는다.**
- **PR 타겟 브랜치는 검증된 값으로만 진행한다.** 인자 우선 검증 → 부재/무효 시 사용자에게 명시 요청. 임의의 기본 브랜치로 대체 금지.
- **현재 브랜치 == 타겟 브랜치면 PR을 만들지 않는다.**
- **force push를 하지 않는다.** 거부 시 원인을 확인하고 사용자에게 알린다.
- **기본 브랜치·기술 스택을 가정하지 않는다.** 저장소 상태에서 직접 감지한다.
- **동일 브랜치로 열린 PR이 있으면** 먼저 안내한다.

---

## 참조 파일

이 스킬은 각 단계의 정확성과 보편성을 높이기 위해 아래 보조 파일을 포함한다.

### references/ — 절차·규칙 참조 가이드
- `commit-conventions.md`: Conventional Commits(영어) 타입·스코프 규칙, 논리적 단위 분리 전략, 민감 파일 패턴, 실전 예시.
- `pr-creation.md`: `gh pr create` 절차 상세 — 타겟 브랜치 검증, 동일 브랜치 PR 처리, 한국어 PR 작성, 흔한 오류와 대처.
- `branch-and-remote.md`: 기본 브랜치·원격 감지, 스택 무관 보편 git 처리, detached HEAD·upstream 등 엣지케이스.

### templates/ — 출력 템플릿
- `commit-plan.md`: 논리적 단위 커밋 계획 제시 양식.
- `pr-body.md`: 한국어 PR 제목·본문 양식(요약/변경사항/테스트 계획).

### checklists/ — 단계별 체크리스트
- `pre-commit.md`: 커밋 전 점검(민감 파일 패턴, 논리적 그룹화, 컨벤션).
- `pre-push.md`: push 전 점검(브랜치 확인, force push 금지, upstream).
- `pre-pr.md`: PR 전 점검(타겟 유효성, 현재≠타겟, 커밋 존재, 기존 PR, push 완료).

---

## 예시

**예시 1: "지금까지 작업 커밋하고 푸시해줘" (범위 = 커밋 + push)**

```
범위 판별: 커밋 + push (PR 미요청)
현재 브랜치: feature/user-profile

커밋 계획
| # | 파일 | 타입 | 메시지 |
|---|------|------|--------|
| 1 | src/user/profile.controller.ts, profile.service.ts | feat | feat(user): add profile read endpoint |
| 2 | src/common/logger.ts | refactor | refactor(logger): extract log formatter |
| 3 | README.md | docs | docs: document profile API usage |

> .env.local 이 변경 목록에 있어 스테이징에서 제외했습니다.

커밋 결과
a1b2c3d feat(user): add profile read endpoint
d4e5f6a refactor(logger): extract log formatter
b7c8d9e docs: document profile API usage

push 결과: feature/user-profile → origin (3개 커밋, upstream 설정)
```

**예시 2: 타겟 브랜치 인자 검증 (PR 요청, 인자가 무효)**

```
요청: "release 브랜치로 PR 올려줘"
검증: git ls-remote --heads origin release  → (결과 없음)

`release`는 origin에 존재하지 않는 브랜치입니다. 올바른 타겟 브랜치를 알려주세요.
참고: 이 저장소의 원격 브랜치는 main, develop, release/1.2 입니다. (기본 브랜치: main)
```

**예시 3: PR 생성 (타겟 = develop, 한국어 본문)**

```
타겟 브랜치: develop  (검증 완료)
현재 브랜치: feature/user-profile
타겟 대비 커밋: 3개 / 변경 파일: 4개

gh pr create --base develop --head feature/user-profile \
  --title "사용자 프로필 조회 API 추가" --body "..."

## 요약
- 사용자 프로필 조회 엔드포인트 추가
- 로그 포매터 리팩토링 및 문서 보강

PR 생성됨: https://github.com/acme/app/pull/142
```
