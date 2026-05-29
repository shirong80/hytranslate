# 릴리스 가이드라인 (Release Guideline)

HyTranslate Mac의 릴리스 노트 작성과 GitHub 릴리스 게시 절차를 정의한다. 제품 범위·기능의 단일 출처는 [`../hytranslate-mac-prd.md`](../hytranslate-mac-prd.md)이며, 이 문서는 "이미 출시되는 코드"를 어떻게 기록·게시하는지만 다룬다.

> 최초 적용 사례: [`v0.1.0.md`](v0.1.0.md) (2026-05-29 게시). 형식·절차의 워크드 예시로 참고한다.

## 1. 버전 규칙

- SemVer를 따른다. `MAJOR.MINOR.PATCH`.
- Git 태그는 `v` 접두사를 붙인다: `v0.1.0`.
- `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`의 `version`은 태그와 일치시킨다.

## 2. 릴리스 노트 형식

[tw93/Kaku](https://github.com/tw93/Kaku/releases) 스타일을 미러링하되, 프로젝트 문서 언어가 한국어이므로 **한국어(주) + 영어(부) 이중 언어**로 작성한다.

- **위치**: `docs/releases/<version>.md` (예: `docs/releases/v0.1.0.md`)
- **헤더(1행)**: `버전 + 이모지 1개 + 짧은 테마 문구`. 예: `v0.1.0 🌅 첫 출시 / First Light`. 이 행은 게시 시 GitHub 릴리스 **제목**이 된다(§4 참고).
- **태그라인**: 한국어 1줄 + 영어 1줄. 앱 한 줄 소개.
- **`## 새로운 기능`** (한국어): 단일 번호 목록. 신규/개선/버그픽스를 카테고리로 나누지 않고 **사용자 임팩트 순**으로 정렬한다. 핵심 번역 → macOS 통합 → 이력/검색 → 온보딩/모델 → 설정/테마 순서를 기본으로 한다.
- **`## Changelog`** (영어): `새로운 기능`과 **동일 항목·동일 순서**로 1:1 병기.
- **`## 시작하기 / Getting Started`**: 설치·최초 실행 안내(Ollama 설치 → 모델 pull → `Cmd+Shift+T`). 초기 버전일수록 중요.
- **`## 개인정보 / Privacy`**: 번역이 localhost를 벗어나지 않으며 텔레메트리·클라우드가 없음을 1줄로.
- **`## 시스템 요구사항 / System Requirements`**: macOS 13 Ventura+, Apple Silicon 권장(Intel 지원).

### 불릿 작성 규칙

- 각 항목은 **굵은 기능명**: 사용자가 *무엇을 할 수 있는지*를 평서형 1~2문장으로.
- 단축키·명령·모델 ID는 백틱으로 감싼다: `Cmd+Shift+T`, `ollama pull hf.co/tencent/Hy-MT2-7B-GGUF:Q4_K_M`.
- 불릿 본문에는 이모지를 쓰지 않는다(헤더 이모지만 허용).
- 한국어는 자연스러운 어투, 영어는 간결한 현재형. 두 언어가 같은 의미를 담아야 한다.

## 3. 정확성 게이트 (필수)

릴리스 노트가 실제로 출시되지 않는 기능을 광고하면 버그로 취급한다. 작성 후 게시 전 아래를 반드시 통과시킨다.

1. **태그될 커밋 기준으로만 기술한다.** 작업 트리의 미커밋/WIP 코드, 다른 브랜치의 기능은 제외한다.
   - 의심되는 기능은 커밋본과 작업 트리를 비교해 검증한다:
     ```bash
     git show <tag-commit>:<path> | rg -i "<feature-symbol>"   # 태그본에 존재?
     rg -i "<feature-symbol>" <path>                            # 작업 트리에만 존재?
     ```
   - 예시(v0.1.0): "전체화면 Space 위 표시" 동작(`apply_fullscreen_overlay`)이 작업 트리에만 있고 태그 커밋에는 없어 노트에서 제외함.
2. **PRD 로드맵이 아니라 구현된 코드만 근거로 한다.** 명령 핸들러·store·UI에서 사용자 도달 가능 여부를 확인한다.
3. **과장 금지.** 모든 불릿은 코드 근거가 있어야 한다. stub/TODO/주석 처리/플래그 off 기능은 넣지 않는다.
4. **개인정보 보호.** 노트·예시에 실제 `source_text`/`translated_text`나 내부 경로를 노출하지 않는다(PRD §12).

### 권장 작성 방식

기능이 많을 때는 도메인별(번역 / 언어·설정 / macOS 통합 / 이력·검색 / 온보딩)로 **실제 코드를 병렬 조사 → 초안 작성 → 정확성·병기 정합 검수**의 3단계로 진행한다. 각 단계의 산출물을 위 게이트로 검증한다.

## 4. 게시 절차 (`gh`)

### 4.1 사전 점검

```bash
gh auth status                        # repo 권한 로그인 확인
git remote -v                         # origin이 대상 리포지토리인지
gh release list --limit 10            # 동일 버전 릴리스 존재 여부
git tag -l                            # 동일 태그 존재 여부
git fetch --quiet origin
git log --oneline origin/main..HEAD   # 미push 커밋 — 비어 있어야 태그 커밋이 원격에 존재
```

- **태그될 커밋은 반드시 origin에 push되어 있어야 한다.** `origin/main..HEAD`가 비어 있으면 `--target main`이 안전하다.

### 4.2 본문 파일 준비

헤더(1행)는 릴리스 **제목**으로 분리하고, 나머지를 본문으로 쓴다. `docs/releases/<version>.md`의 첫 줄을 제외한 내용을 본문 파일로 만든다(임시 파일 사용 후 삭제).

### 4.3 생성

```bash
gh release create v0.1.0 \
  --target main \
  --title "v0.1.0 🌅 첫 출시 / First Light" \
  --notes-file <body-file> \
  --latest
```

- 정식 릴리스는 `--latest`. 사전 배포본은 `--prerelease`를 추가한다.
- 초안으로 검토만 하려면 `--draft`로 만들고 GitHub에서 게시한다.

### 4.4 게시 후 검증

```bash
gh release view v0.1.0 --json tagName,name,isDraft,isPrerelease,targetCommitish,url
git ls-remote --tags origin v0.1.0    # 태그가 의도한 커밋을 가리키는지 확인
```

> 주의: 일부 `gh` 버전에는 `--json isLatest` 필드가 없다. 사용 가능 필드는 오류 메시지로 확인한다.

## 5. 체크리스트

- [ ] 버전 일치: 태그 / `package.json` / `Cargo.toml` / `tauri.conf.json`
- [ ] 태그될 커밋이 origin에 push됨 (`origin/main..HEAD` 비어 있음)
- [ ] 노트 형식: 헤더·태그라인·`새로운 기능`·`Changelog`·시작하기·개인정보·시스템 요구사항
- [ ] 한국어/영어 항목 1:1 정합, 임팩트순 정렬
- [ ] 정확성 게이트 통과: 태그 커밋 기준, 코드 근거 있음, WIP 제외, 개인정보 미노출
- [ ] `gh release create` 후 `release view` + `ls-remote --tags`로 검증
- [ ] `docs/releases/<version>.md` 커밋 여부 결정(리포지토리 보관 권장)
