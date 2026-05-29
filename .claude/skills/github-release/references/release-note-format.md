# 릴리스 노트 형식 참조 가이드

이중 언어(한국어 주 + 영어 부) 릴리스 노트 작성 규칙. [tw93/Kaku](https://github.com/tw93/Kaku/releases) 스타일을 미러링하되, 프로젝트 문서 언어가 한국어이면 한국어를 주, 영어를 부로 병기한다.

> 파일 위치는 `docs/releases/<version>.md` (예: `docs/releases/v0.1.0.md`).

---

## 1. 전체 구조

순서를 지킨다. 헤더 → 태그라인 → 새로운 기능 → Changelog → 시작하기 → 개인정보 → 시스템 요구사항.

```
<헤더 1행>            ← GitHub 릴리스 제목이 됨 (본문 파일에서는 제외)

<태그라인 한국어>
<태그라인 영어>

## 새로운 기능
## Changelog
## 시작하기 / Getting Started
## 개인정보 / Privacy
## 시스템 요구사항 / System Requirements
```

---

## 2. 헤더 (1행)

`버전 + 이모지 1개 + 짧은 테마 문구` 형식. 이모지는 정확히 1개만.

### ✅ 안전 패턴
```
v0.1.0 🌅 첫 출시 / First Light
v1.2.0 ⚡ 더 빨라진 번역 / Faster Translations
v2.0.0 🧭 새로워진 워크플로 / A New Workflow
```
- 게시 시 이 행이 `--title`이 된다. 본문 파일(`--notes-file`)에는 넣지 않는다.

### ❌ 취약 패턴
```
v0.1.0 🎉🚀🌅 첫 출시!!! 드디어 공개합니다 정말 기대해주세요   # 이모지 과다, 장황, 과장
Release v0.1.0                                              # 테마 문구 없음, 무미건조
```

---

## 3. 태그라인

한국어 1줄 + 영어 1줄. 앱이 무엇인지 한 문장으로. 인용(`>`) 블록으로 작성하면 깔끔하다.

```markdown
> 로컬에서 동작하는 빠른 번역기. 데이터는 기기를 벗어나지 않습니다.
> A fast translator that runs entirely on your machine. Your data never leaves your device.
```

---

## 4. 새로운 기능 (한국어) / Changelog (영어)

두 섹션은 **동일 항목·동일 순서·동일 의미**로 1:1 병기한다. 카테고리(신규/개선/버그픽스)로 나누지 않고 **단일 번호 목록**으로, **사용자 임팩트 순**으로 정렬한다.

### 임팩트 순서 기본값
핵심 기능 → OS/플랫폼 통합 → 이력/검색 → 온보딩/모델 → 설정/테마. 프로젝트 성격에 맞게 조정하되, "사용자가 가장 크게 체감하는 것"이 위로 온다.

### 불릿 작성 규칙
- 각 항목: `**굵은 기능명**`: 사용자가 *무엇을 할 수 있는지* 평서형 1~2문장.
- 단축키·명령·모델 ID·플래그는 백틱으로: `Cmd+Shift+T`, `ollama pull ...`.
- **불릿 본문에 이모지 금지** (헤더 이모지만 허용).
- 한국어는 자연스러운 어투, 영어는 간결한 현재형.

### ✅ 안전 패턴
```markdown
## 새로운 기능

1. **즉시 번역**: 어디서든 `Cmd+Shift+T`로 선택한 텍스트를 즉시 번역합니다.
2. **로컬 모델 실행**: 받은 모델을 로컬에서 구동해 외부 전송 없이 번역합니다.

## Changelog

1. **Instant translation**: Translate selected text from anywhere with `Cmd+Shift+T`.
2. **Local model execution**: Run the model fully on-device, with no external requests.
```

### ❌ 취약 패턴
```markdown
## 새로운 기능
### 🎯 신규 기능        # 카테고리 분리 금지, 본문 이모지 금지
- 🚀 엄청 빠른 번역!    # 과장, 코드 근거 없음, 이모지
### 🐛 버그 수정
- 여러 버그를 고쳤습니다  # 무엇이 고쳐졌는지 사용자 임팩트 불명확
```
(한국어 항목 수와 영어 항목 수가 다르면 정합 실패 — 게시 전에 맞춘다.)

---

## 5. 시작하기 / Getting Started

설치·최초 실행을 순서대로. 초기 버전일수록 중요. 의존 도구 설치 → 모델/리소스 준비 → 첫 실행/단축키 순.

```markdown
## 시작하기 / Getting Started

1. [Ollama](https://ollama.com)를 설치합니다. / Install [Ollama](https://ollama.com).
2. 모델을 받습니다 / Pull the model: `ollama pull hf.co/tencent/Hy-MT2-7B-GGUF:Q4_K_M`
3. 앱을 실행하고 `Cmd+Shift+T`로 첫 번역을 해봅니다. / Launch the app and translate with `Cmd+Shift+T`.
```

---

## 6. 개인정보 / Privacy

데이터 취급을 1줄로. 로컬 처리·텔레메트리 없음 등 해당 시에만.

```markdown
## 개인정보 / Privacy

번역은 localhost를 벗어나지 않으며, 텔레메트리나 클라우드 전송이 없습니다.
Translation never leaves localhost — no telemetry, no cloud.
```

---

## 7. 시스템 요구사항 / System Requirements

지원 OS·아키텍처를 명확히. 실제 빌드 타깃과 일치해야 한다.

```markdown
## 시스템 요구사항 / System Requirements

- macOS 13 Ventura 이상 / macOS 13 Ventura or later
- Apple Silicon 권장 (Intel 지원) / Apple Silicon recommended (Intel supported)
```

---

## 8. 작성 3단계 권장 방식

기능이 많을 때:

1. **병렬 조사**: 도메인별(번역 / 언어·설정 / OS 통합 / 이력·검색 / 온보딩)로 실제 코드를 동시 조사.
2. **초안 작성**: 임팩트순 단일 목록으로 한국어 노트 작성 → 영어 1:1 병기.
3. **검수**: `accuracy-gate.md`의 게이트로 코드 근거·WIP 제외·이중 언어 정합을 검증.
