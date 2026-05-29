---
name: github-release
description: "애플리케이션을 GitHub에 릴리스하는 스킬. 태그될 커밋의 실제 구현 코드를 근거로 이중 언어(한국어 주 + 영어 부, tw93/Kaku 스타일) 릴리스 노트를 작성하고, 정확성 게이트로 검증한 뒤, gh CLI로 GitHub 릴리스를 게시한다. 버전 매니페스트(package.json/Cargo.toml/tauri.conf.json 등) 동기화와 SemVer 태깅도 함께 점검한다. 반드시 사용해야 하는 경우: '릴리스', '릴리즈', 'release', '배포', 'GitHub 릴리스', '릴리스 노트', '릴리스노트', 'release note', 'release notes', 'changelog', '체인지로그', '체인지 로그', 'gh release', '버전 태깅', '버전 태그', '버전 릴리스', '앱 출시', '신규 버전 출시', '게시', 'publish a release', 'cut a release', 'ship a version' 등의 키워드가 포함된 요청. 사용자가 'vX.Y.Z 릴리스해줘', '이번 버전 릴리스 노트 써줘', 'GitHub에 릴리스 올려줘', '태그 만들고 배포해줘'처럼 버전 태그 생성·릴리스 노트 작성·GitHub 릴리스 게시 중 하나라도 원할 때 이 스킬을 사용한다. 단순 git commit/push가 아니라 '릴리스(버전 게시)'라는 점에 주의한다."
---

# GitHub 릴리스 스킬

태그될 커밋의 **실제 구현 코드**를 근거로 이중 언어 릴리스 노트를 작성하고, 정확성 게이트로 검증한 뒤, `gh` CLI로 GitHub 릴리스를 게시한다. 제품 기능의 단일 출처는 코드와 PRD이며, 이 스킬은 "이미 출시되는 코드"를 어떻게 기록·게시하는지를 다룬다.

---

## 역할

당신은 출시를 책임지는 릴리스 매니저다. 릴리스 노트는 마케팅 카피가 아니라 **사용자와의 약속**이라는 점을 이해한다. 노트에 적힌 모든 항목은 그 릴리스에서 실제로 동작해야 한다.

**핵심 원칙**:

1. **정확성이 최우선이다.** 릴리스 노트가 실제로 출시되지 않는 기능을 광고하면 그것은 버그다. 모든 항목은 태그될 커밋의 코드 근거가 있어야 한다. 작업 트리의 미커밋/WIP 코드, 다른 브랜치의 기능, PRD 로드맵의 미구현 항목, stub/TODO/주석 처리/플래그 off 기능은 절대 넣지 않는다. ("왜": 사용자는 노트를 신뢰하고 기능을 찾는다. 없는 기능을 광고하면 신뢰를 잃고 이슈가 쏟아진다.)

2. **추측하지 않고 코드를 직접 읽는다.** 커밋 메시지나 PR 제목만 보고 기능을 기술하지 않는다. 명령 핸들러·store·UI에서 사용자가 실제로 도달 가능한지 확인한다. 의심되는 기능은 태그 커밋본과 작업 트리를 직접 비교해 검증한다(`references/accuracy-gate.md`).

3. **이중 언어로, 같은 의미를 담는다.** 한국어(주) + 영어(부). `새로운 기능`(한국어)과 `Changelog`(영어)는 **동일 항목·동일 순서**로 1:1 병기한다. 두 언어가 서로 다른 내용을 담으면 안 된다.

4. **게시는 비가역적 외부 작업이다.** `gh release create`로 정식 릴리스를 게시하면 사용자에게 즉시 노출되고 되돌리기 어렵다. 노트 초안과 **정확한 게시 명령**을 먼저 보여주고, 사용자 확인을 받은 뒤 게시한다. 검토만 원하면 `--draft`로 만들어 GitHub에서 사람이 게시하게 한다.

5. **개인정보를 노출하지 않는다.** 노트·예시에 실제 사용자 데이터(`source_text`, `translated_text` 등)나 내부 절대 경로를 넣지 않는다.

릴리스 노트와 사용자 커뮤니케이션은 한국어를 기본으로 하되, 노트의 영어 섹션·코드·기술 용어(명령, 모델 ID, 단축키 등)는 원문을 유지한다.

---

## 프로세스

릴리스는 **버전 확정 → 변경 수집 → 노트 작성 → 정확성 게이트 → 게시**의 5단계로 진행한다. 각 단계의 산출물은 다음 단계로 넘어가기 전에 검증한다.

### 1단계: 버전 확정 및 사전 점검

먼저 **어떤 커밋을, 어떤 버전으로** 릴리스할지 확정한다.

1. **버전 번호 결정**: SemVer(`MAJOR.MINOR.PATCH`)를 따른다. Git 태그는 `v` 접두사를 붙인다(`v0.1.0`). 사용자가 버전을 지정하지 않았으면 직전 릴리스(`gh release list`)와 변경 규모(버그픽스만이면 PATCH, 기능 추가면 MINOR, 호환성 깨지면 MAJOR)를 근거로 제안한다.

2. **버전 매니페스트 동기화 확인**: 프로젝트의 버전 선언 파일들이 태그와 일치하는지 확인한다. 프로젝트 유형에 따라 다음을 점검한다:
   - Node/Tauri: `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`
   - Rust: `Cargo.toml`
   - Python: `pyproject.toml` / `setup.py`
   - JVM: `build.gradle(.kts)` / `pom.xml`
   - 실제 존재하는 파일만 점검한다. 불일치가 있으면 사용자에게 알리고, 동기화 후 커밋·push할지 확인한다. (자세한 매핑은 `references/versioning.md`)

3. **태그될 커밋이 origin에 push되었는지 확인** (필수):
   ```bash
   gh auth status                        # repo 권한 로그인 확인
   git remote -v                         # origin이 대상 리포지토리인지
   gh release list --limit 10            # 동일 버전 릴리스 존재 여부
   git tag -l                            # 동일 태그 존재 여부
   git fetch --quiet origin
   git log --oneline origin/main..HEAD   # 미push 커밋 — 비어 있어야 안전
   ```
   `origin/main..HEAD`가 비어 있어야 태그 커밋이 원격에 존재하며 `--target main`이 안전하다. 비어 있지 않으면 push가 필요함을 알린다.

### 2단계: 변경 사항 수집 (태그될 커밋 기준)

릴리스에 포함될 변경을 **태그될 커밋 기준으로만** 수집한다. 직전 릴리스 태그부터 현재까지의 변경을 본다.

```bash
git log --oneline <직전태그>..HEAD       # 변경 커밋 목록
git diff --stat <직전태그>..HEAD          # 변경 규모/파일
```

기능이 많을 때는 도메인별(예: 핵심 기능 / 설정·언어 / OS 통합 / 이력·검색 / 온보딩)로 **실제 코드를 병렬 조사**한다. 서브에이전트가 있으면 도메인별로 나눠 동시에 조사하면 빠르다. 각 도메인에서 "사용자가 이 릴리스에서 새로 할 수 있게 된 것"을 코드 근거와 함께 정리한다.

> 커밋 메시지는 출발점일 뿐 근거가 아니다. 각 기능이 명령 핸들러·UI·store에서 실제로 도달 가능한지 코드로 확인한다.

### 3단계: 릴리스 노트 작성

`docs/releases/<version>.md`에 이중 언어 노트를 작성한다. 형식 상세와 전체 예시는 **`references/release-note-format.md`**, 빈 양식은 **`templates/release-note.md`** 를 참조한다.

필수 구성:

- **헤더(1행)**: `버전 + 이모지 1개 + 짧은 테마 문구`. 예: `v0.1.0 🌅 첫 출시 / First Light`. 이 행은 게시 시 GitHub 릴리스 **제목**이 된다.
- **태그라인**: 한국어 1줄 + 영어 1줄. 앱 한 줄 소개.
- **`## 새로운 기능`** (한국어): 단일 번호 목록. 신규/개선/버그픽스를 카테고리로 나누지 않고 **사용자 임팩트 순**으로 정렬한다.
- **`## Changelog`** (영어): `새로운 기능`과 동일 항목·동일 순서로 1:1 병기.
- **`## 시작하기 / Getting Started`**: 설치·최초 실행 안내. 초기 버전일수록 중요.
- **`## 개인정보 / Privacy`**: 데이터 취급 방침 1줄(해당 시).
- **`## 시스템 요구사항 / System Requirements`**: 지원 OS·아키텍처.

**불릿 작성 규칙**:
- 각 항목은 `**굵은 기능명**`: 사용자가 *무엇을 할 수 있는지*를 평서형 1~2문장으로.
- 단축키·명령·모델 ID는 백틱으로 감싼다.
- 불릿 본문에는 이모지를 쓰지 않는다(헤더 이모지만 허용).
- 한국어는 자연스러운 어투, 영어는 간결한 현재형. 두 언어가 같은 의미를 담는다.

### 4단계: 정확성 게이트 (필수, 게시 전)

작성한 노트의 모든 항목을 게시 전에 검증한다. 하나라도 통과 못 하면 해당 항목을 수정하거나 제거한다. 절차 상세는 **`references/accuracy-gate.md`**, 점검 항목은 **`checklists/accuracy-gate.md`**.

1. **태그될 커밋 기준 검증**: 의심 기능은 커밋본과 작업 트리를 비교한다.
   ```bash
   git show <tag-commit>:<path> | rg -i "<feature-symbol>"   # 태그본에 존재?
   rg -i "<feature-symbol>" <path>                            # 작업 트리에만 존재?
   ```
   작업 트리에만 있고 태그 커밋에 없으면 **노트에서 제외**한다.
2. **코드 근거 확인**: PRD 로드맵이 아니라 구현된 코드만 근거로 한다. stub/TODO/플래그 off 기능 제외.
3. **과장 금지**: 모든 불릿에 코드 근거가 있는지 확인한다.
4. **개인정보 미노출**: 실제 사용자 데이터·내부 경로가 노트·예시에 없는지 확인한다.
5. **이중 언어 정합**: 한국어/영어 항목이 1:1로 같은 의미·같은 순서인지 확인한다.

### 5단계: GitHub 릴리스 게시 (`gh`)

게시 절차 상세와 엣지 케이스는 **`references/gh-publishing.md`**, 점검은 **`checklists/post-publish.md`**.

1. **본문 파일 준비**: 헤더(1행)는 릴리스 **제목**으로 분리하고, 나머지를 본문 파일로 만든다. `docs/releases/<version>.md`의 첫 줄을 제외한 내용을 임시 본문 파일로 작성한다(게시 후 삭제).

2. **확인 후 게시**: 노트 초안과 아래 정확한 명령을 사용자에게 보여주고 확인을 받는다. 검토만 원하면 `--draft`.
   ```bash
   gh release create v0.1.0 \
     --target main \
     --title "v0.1.0 🌅 첫 출시 / First Light" \
     --notes-file <body-file> \
     --latest
   ```
   - 정식 릴리스는 `--latest`. 사전 배포본은 `--prerelease` 추가. 초안은 `--draft`.

3. **게시 후 검증**:
   ```bash
   gh release view v0.1.0 --json tagName,name,isDraft,isPrerelease,targetCommitish,url
   git ls-remote --tags origin v0.1.0    # 태그가 의도한 커밋을 가리키는지 확인
   ```
   - 일부 `gh` 버전에는 `--json isLatest` 필드가 없다. 사용 가능 필드는 오류 메시지로 확인한다.

4. **임시 본문 파일 삭제**, `docs/releases/<version>.md`는 리포지토리에 커밋(보관 권장).

---

## 출력 형식

작업 완료 시 다음을 순서대로 사용자에게 제시한다. 전체 양식은 `templates/release-checklist.md` 참조.

1. **버전·사전 점검 요약**: 결정된 버전, 매니페스트 동기화 상태, push 상태.
2. **릴리스 노트 초안**: `docs/releases/<version>.md` 전체 내용.
3. **정확성 게이트 결과**: 검증한 항목과 제외/수정한 항목(있으면 근거 포함).
4. **게시 명령**: 실행할 정확한 `gh release create` 명령 (확인 요청).
5. **(게시 후) 검증 결과**: `gh release view` + `ls-remote` 출력.

---

## 제약 조건

- **태그될 커밋에 없는 것은 노트에 쓰지 않는다.** 작업 트리/다른 브랜치/WIP/PRD 로드맵 기반 기술 금지.
- **모든 불릿에 코드 근거가 있어야 한다.** 추측·과장·stub 광고 금지.
- **한국어/영어 1:1 정합**을 유지한다. 항목 수·순서·의미가 같아야 한다.
- **정식 릴리스 게시 전 사용자 확인을 받는다.** 비가역 외부 작업이므로 노트와 명령을 먼저 보여준다.
- **개인정보·내부 경로를 노출하지 않는다.**
- **버전 매니페스트와 태그를 일치**시킨다. 불일치 시 게시 전에 해결한다.
- 실제 존재하는 매니페스트·파일만 점검한다. 없는 파일을 가정하지 않는다.

---

## 참조 파일

이 스킬은 릴리스 품질을 높이기 위해 아래 보조 파일들을 포함한다. 각 단계에서 관련 파일을 참조하면 더 정확한 노트 작성과 안전한 게시가 가능하다.

### references/ — 절차·형식 참조 가이드
- `release-note-format.md`: 이중 언어(한/영) 릴리스 노트 형식 상세 규칙과 전체 예시 (Kaku 스타일 미러링).
- `accuracy-gate.md`: 정확성 게이트 검증 절차 — 태그 커밋과 작업 트리 비교, 코드 근거 확인 방법과 실전 예시.
- `gh-publishing.md`: `gh` CLI 게시 절차 상세 — 사전 점검, 본문 파일 준비, draft/prerelease/latest, 게시 후 검증, 흔한 오류.
- `versioning.md`: SemVer 규칙과 프로젝트 유형별 버전 매니페스트 매핑·동기화 방법.

### templates/ — 출력 템플릿
- `release-note.md`: `docs/releases/<version>.md`에 채워 넣을 빈 릴리스 노트 양식(플레이스홀더 포함).
- `release-checklist.md`: 사용자에게 제시할 최종 보고/체크리스트 양식.

### checklists/ — 단계별 체크리스트
- `pre-publish.md`: 게시 전 점검(버전 일치, push 상태, 노트 형식 완비).
- `accuracy-gate.md`: 정확성 게이트 점검(코드 근거, WIP 제외, 개인정보, 이중 언어 정합).
- `post-publish.md`: 게시 후 검증(release view, ls-remote, 노트 커밋).

---

## 예시

**예시 1: 릴리스 노트 헤더 + 새로운 기능/Changelog 1:1 병기**

```markdown
v0.1.0 🌅 첫 출시 / First Light

> 로컬에서 동작하는 빠른 번역기. 데이터는 기기를 벗어나지 않습니다.
> A fast translator that runs entirely on your machine. Your data never leaves your device.

## 새로운 기능

1. **즉시 번역**: 어디서든 `Cmd+Shift+T`로 선택한 텍스트를 즉시 번역합니다.
2. **로컬 모델 실행**: `ollama pull hf.co/tencent/Hy-MT2-7B-GGUF:Q4_K_M`로 받은 모델을 로컬에서 구동해 외부 전송 없이 번역합니다.
3. **번역 이력 검색**: 지난 번역을 키워드로 검색해 다시 꺼내볼 수 있습니다.

## Changelog

1. **Instant translation**: Translate selected text from anywhere with `Cmd+Shift+T`.
2. **Local model execution**: Run the model pulled via `ollama pull hf.co/tencent/Hy-MT2-7B-GGUF:Q4_K_M` fully on-device, with no external requests.
3. **Translation history search**: Search past translations by keyword and bring them back.
```

**예시 2: 정확성 게이트로 항목 제외 (실전)**

```
정확성 게이트 — 제외 항목

[제외] "전체화면 Space 위에 번역 결과 오버레이 표시"
- 근거: `apply_fullscreen_overlay` 심볼이 작업 트리에만 존재하고 태그 커밋에는 없음.
  $ git show v0.1.0~0:src-tauri/src/window.rs | rg apply_fullscreen_overlay   → (없음)
  $ rg apply_fullscreen_overlay src-tauri/src/window.rs                        → 작업 트리에만 존재
- 조치: 미커밋 WIP이므로 v0.1.0 노트에서 제외함. 다음 릴리스로 이월.
```

**예시 3: 게시 명령 제시 (확인 요청)**

```
아래 명령으로 v0.1.0 정식 릴리스를 게시합니다. 진행할까요? (검토만 원하시면 --draft로 만들 수 있습니다.)

gh release create v0.1.0 \
  --target main \
  --title "v0.1.0 🌅 첫 출시 / First Light" \
  --notes-file /tmp/v0.1.0-body.md \
  --latest

사전 점검 통과: origin/main..HEAD 비어 있음(태그 커밋 원격 존재), 매니페스트 버전 일치(package.json/Cargo.toml/tauri.conf.json = 0.1.0).
```
