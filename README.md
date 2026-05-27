# HyTranslate Mac

> Tencent Hy-MT2 모델을 Ollama로 로컬 실행하여 한국어·중국어를 영어로 번역하는 macOS 메뉴바 번역 앱

HyTranslate Mac은 macOS 전용 로컬 번역 데스크톱 앱이다. 입력하거나 붙여넣은 **한국어 / 중국어 간체 / 중국어 번체** 텍스트를 **영어**로 번역하며, 번역 과정 전체가 사용자의 Mac 안에서만 실행된다. 원문과 결과는 기기 밖으로 나가지 않는다.

> **상세 명세**: 모든 제품 요구사항·화면·API 계약의 단일 출처는 [`docs/hytranslate-mac-prd.md`](docs/hytranslate-mac-prd.md) 이다.

## 핵심 가치

- **개인정보 보호** — 번역 요청은 `localhost:11434`(Ollama)로만 전송된다. 텔레메트리·원격 로깅·클라우드 동기화 없음.
- **빠른 피드백** — Ollama streaming 응답으로 번역 결과를 생성되는 즉시 표시한다.
- **오프라인 사용** — 모델 다운로드 후에는 인터넷 없이 동작한다.
- **작업 흐름 통합** — 전역 단축키(`Cmd+Shift+T`)와 메뉴바 팝업으로 어떤 앱에서든 호출한다.
- **반복 사용성** — 번역 이력, FTS5 전문 검색, 즐겨찾기, 태그로 과거 번역을 재사용한다.

## 주요 기능

| 영역 | 내용 |
|---|---|
| 번역 | 한국어 / 중국어 간체 / 중국어 번체 → 영어, 실시간 streaming, 요청 취소 |
| 언어 감지 | 자동 감지 + 수동 입력 언어 override |
| macOS 통합 | 전역 단축키, 플로팅 팝업, 메뉴바 팝오버, 클립보드 번역, 자동 시작 |
| 이력 | SQLite 영속화, FTS5 검색, 즐겨찾기·태그, CSV / JSON export |
| 온보딩 | 환경 감지, Ollama 설치·실행 상태 확인, 권장 모델 추천 + pull 진행률 |
| 모양새 | 한국어 UI, light / dark / system 테마 |

## 기술 스택

| 레이어 | 기술 |
|---|---|
| 데스크톱 셸 | Tauri 2 |
| 프론트엔드 | React 18 + TypeScript 5 + Tailwind CSS 3 + Zustand |
| 백엔드 | Rust (edition 2021) + Tokio + reqwest + serde + rusqlite (FTS5) |
| 모델 런타임 | Ollama HTTP API (`http://localhost:11434`) |
| 모델 | Tencent Hy-MT2 7B / 1.8B GGUF (Q4_K_M) |
| 빌드 | Vite (FE) + cargo (Rust) via `tauri` CLI |

## 시스템 요구사항

| 항목 | 요구사항 |
|---|---|
| OS | macOS 13 Ventura 이상 |
| 아키텍처 | Apple Silicon (권장) / Intel (지원, 성능 경고 표시) |
| 메모리 | 12GB 이상 권장 (Hy-MT2 7B). 8GB 환경은 Hy-MT2 1.8B 권장 |

> 앱이 첫 실행 온보딩에서 메모리를 감지해 12GB 미만이면 1.8B 모델을 자동으로 권장한다.

---

## 설치 — 사용자

### 1. Ollama 설치

HyTranslate는 Ollama를 **번들하지 않는다**. 공식 설치 파일을 직접 설치해야 한다.

- [https://ollama.com/download](https://ollama.com/download) 에서 macOS용 Ollama 설치
- 설치 후 Ollama가 백그라운드에서 실행 중인지 확인

```bash
# 실행 여부 확인 (모델 목록이 출력되면 정상)
ollama list
```

### 2. HyTranslate 설치

서명·notarization된 DMG를 내려받아 `Applications`로 드래그한다. *(v1 배포 채널 확정 후 링크 추가 예정)*

### 3. 모델 준비

앱 **온보딩 화면**이 Ollama 상태를 점검하고 권장 모델을 다운로드(`pull`)하며 진행률을 표시한다. 별도 조작 없이 안내를 따르면 된다.

수동으로 받으려면:

```bash
# 7B (권장, 12GB+ RAM)
ollama pull hf.co/tencent/Hy-MT2-7B-GGUF:Q4_K_M

# 1.8B (경량, 8GB RAM)
ollama pull hf.co/tencent/Hy-MT2-1.8B-GGUF:Q4_K_M
```

준비가 끝나면 `Cmd+Shift+T`로 어디서든 번역 팝업을 띄울 수 있다.

---

## 설치 — 개발자

### 사전 요구사항

| 도구 | 버전 / 비고 |
|---|---|
| Node.js | 18 이상 |
| Rust | stable (`rust-toolchain.toml`이 `rustfmt` + `clippy` 포함 stable 채널 고정) |
| Xcode Command Line Tools | `xcode-select --install` |
| Ollama | 위 사용자 설치 절차와 동일 (개발 중에도 로컬 실행 필요) |

Rust가 없다면:

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

### 클론 & 의존성 설치

```bash
git clone <repo-url> hytranslate
cd hytranslate
npm install        # 프론트엔드 의존성. Rust crate는 최초 cargo 빌드 시 자동 설치
```

### 개발 실행

```bash
npm run tauri:dev    # FE + Rust 통합 핫 리로드 (권장)
npm run dev          # 프론트엔드 단독 (Tauri API 미제공 — 제한적으로만 사용)
```

### 릴리스 빌드

```bash
npm run tauri:build  # 서명된 DMG 생성 (release)
```

## 개발 명령어

```bash
# 프론트엔드
npm run lint         # ESLint
npm run typecheck    # tsc --noEmit
npm run test         # Vitest (단위)
npm run test:e2e     # Playwright (E2E — 빌드 또는 실행 중인 앱 필요)
npm run format       # Prettier

# Rust (저장소 루트 기준)
cargo fmt   --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings
cargo test  --manifest-path src-tauri/Cargo.toml
```

## 프로젝트 구조

```
hytranslate/
├── src/              # React 프론트엔드 (windows / components / features / lib / i18n)
├── src-tauri/        # Rust 백엔드 (commands / ollama / language / history / settings / db ...)
├── docs/             # PRD(단일 출처) 및 계획·리뷰 문서
├── evals/            # 번역 품질 평가셋
└── tests/e2e/        # Playwright 스펙
```

레이어링·코딩 규칙·IPC 계약 등 기여 가이드는 [`.claude/CLAUDE.md`](.claude/CLAUDE.md)와 [`.claude/rules/`](.claude/rules/)를 참고한다.

## 개인정보 / 네트워크 정책

- 번역 요청은 설정된 `ollama_endpoint`(기본 `localhost:11434`) 외 호스트로 전송되지 않는다.
- 네트워크 사용은 **모델 다운로드**(사용자 승인), **Ollama 설치 링크**(사용자 클릭)로 제한된다.
- 로그에 원문·번역 결과를 기록하지 않으며, 로그는 로컬 파일로만 남는다.
- SQLite DB는 v1에서 암호화하지 않는다. 설정에서 이력 저장을 끄거나 전체 삭제할 수 있다.

## 라이선스

UNLICENSED — 비공개 프로젝트.
