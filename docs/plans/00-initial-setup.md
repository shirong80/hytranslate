# HyTranslate Mac — 초기 셋업 계획서

> **상태**: 리뷰 v4 반영 완료 — 사용자 승인 후 코드 진행
> **리뷰 출처**:
> - `docs/review/00-initial-setup-review.md` (계획서 검토) — v1: High 1 / Medium 3 / Low 2 → 모두 반영. v2: Medium 1 → 반영 후 v3 에서 supersede
> - `docs/review/00-initial-setup-code-review.md` (산출물 코드리뷰) — v3 (구판): High 1 / Medium 2 / Low 2 → 모두 반영. v4 (현행): Medium 1 (shell-plugin 잔재 정리) → 반영
> **참조**: `docs/hytranslate-mac-prd.md` (Source of Truth), `.claude/CLAUDE.md`, `.claude/rules/*`
> **범위 정의**: 본 문서가 다루는 "초기 셋업"은 **Phase 1 기능 구현 직전까지의 인프라**를 가리킨다. Tauri 2 scaffold, 빌드 시스템, 디렉터리 구조, dev tooling, 설정 파일, 스켈레톤 진입점을 포함하며 **번역 기능 코드는 포함하지 않는다**.

---

## 1. 목적

PRD §15.1 (Phase 1) 의 "Tauri 2 프로젝트 scaffold" 단계와 그 전제가 되는 모든 기반을 준비한다.
완료 시점에 **§8 산출물 체크리스트의 모든 항목**이 통과해야 하며, 요약하면 다음과 같다.

- `npm run tauri:dev` 으로 빈 React 메인 창이 macOS 위에 뜬다 (`popup` / `menubar` 는 hidden 상태).
- FE 검증 6종 통과: `format:check`, `lint`, `typecheck`, `test` (Vitest), `test:e2e` (Playwright sanity)
- BE 검증 4종 통과: `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo check`, `cargo test`
- `.claude/rules/architecture.md` 가 명시한 디렉터리 구조가 그대로 존재한다.
- Phase 1 기능 코드를 작성할 수 있는 "빈 슬롯"이 모두 마련된다 (공개 `AppError` 7 variant, 이벤트 상수, 도메인 module 스켈레톤).

본 문서가 **하지 않는 것**:

- Ollama HTTP 클라이언트 구현 ❌ (Phase 1 작업)
- 언어 감지 / prompt builder 구현 ❌ (Phase 2)
- SQLite 스키마 / migration ❌ (Phase 4)
- 온보딩 화면 ❌ (Phase 5)

---

## 2. 아키텍처 한눈에 보기 (PRD + rules 기반)

```
hytranslate/
├── src/                          # React frontend
│   ├── windows/                  # main / popup / menubar 진입점
│   │   ├── main/
│   │   │   ├── index.html
│   │   │   └── main.tsx
│   │   ├── popup/
│   │   │   ├── index.html
│   │   │   └── popup.tsx
│   │   └── menubar/
│   │       ├── index.html
│   │       └── menubar.tsx
│   ├── components/               # 재사용 UI
│   ├── features/                 # 기능 슬라이스 (Phase별로 채움)
│   ├── lib/
│   │   ├── ipc/
│   │   │   ├── client.ts         # invoke/listen 기본 wrapper
│   │   │   ├── events.ts         # 이벤트 이름 단일 정의 (placeholder)
│   │   │   └── errors.ts         # AppError discriminated union 미러
│   │   └── hooks/
│   ├── i18n/
│   │   └── ko.ts                 # 한국어 문자열 (빈 스텁)
│   ├── styles/
│   │   └── globals.css           # Tailwind 레이어
│   └── types/                    # 공용 타입 (필요 시)
├── src-tauri/
│   ├── Cargo.toml
│   ├── tauri.conf.json
│   ├── build.rs
│   ├── icons/
│   └── src/
│       ├── main.rs
│       ├── lib.rs
│       ├── errors.rs             # AppError enum (placeholder 변형들)
│       ├── events.rs             # 이벤트 이름 단일 정의 (placeholder)
│       ├── commands/             # mod.rs 만, 핸들러 0개
│       ├── ollama/               # mod.rs 만
│       ├── language/             # mod.rs 만
│       ├── history/              # mod.rs 만
│       ├── settings/             # mod.rs 만
│       ├── db/                   # mod.rs 만
│       ├── shortcuts/            # mod.rs 만
│       └── menubar/              # mod.rs 만
├── tests/e2e/                    # Playwright (빈 설정만)
├── evals/
│   └── translation-quality.md    # PRD §14.1 요구 파일 (헤더만)
├── docs/
│   ├── hytranslate-mac-prd.md    # 기존
│   └── plans/                    # 본 문서 위치
├── package.json
├── tsconfig.json
├── tsconfig.node.json
├── vite.config.ts
├── tailwind.config.ts
├── postcss.config.cjs
├── eslint.config.mjs             # flat config (ESLint v9)
├── .prettierrc
├── .editorconfig
├── .gitignore
├── .nvmrc                        # Node LTS
├── rust-toolchain.toml           # Rust stable
├── playwright.config.ts
└── vitest.config.ts
```

근거:
- `.claude/rules/architecture.md` 의 "Feature owns store + ipc.ts + types" 원칙 → `features/` 만 비워두면 Phase별 작업이 즉시 가능
- `.claude/rules/tauri-ipc.md` 의 "이벤트 이름은 `src/lib/ipc/events.ts` AND `src-tauri/src/events.rs` 에서 단일 정의" → 두 파일은 초기 셋업 단계에서 빈 스텁으로 만들어둔다
- `.claude/rules/error-handling.md` 의 `AppError` 단일 소스 + FE 미러 원칙 → `errors.rs` 와 `lib/ipc/errors.ts` 를 짝지어 생성

---

## 3. 단계별 작업 (Step Plan)

### Step 1 — 툴체인 고정
- `.nvmrc` → Node `20` LTS (Tauri 2 + Vite 5 호환)
- `rust-toolchain.toml` → `stable` + `rustfmt`, `clippy` components
- `.gitignore` 작성 (`node_modules/`, `dist/`, `src-tauri/target/`, `.DS_Store`, `*.log`, `coverage/`, `playwright-report/`, `test-results/`)
- `.editorconfig` (utf-8, LF, 2-space; Rust 파일은 4-space)

**왜 먼저인가**: Tauri CLI를 받기 전에 Node 버전을 못박아두지 않으면 머신마다 lockfile 불일치가 생긴다.

### Step 2 — Tauri 2 scaffold
- `npm create tauri-app@latest` 가 아닌 **수동 셋업** 권장. 이유:
  - PRD 가 path alias, 멀티 윈도우, 한국어 UI, 외부 capability allowlist 등 디폴트와 다른 결정을 이미 가지고 있음
  - 수동 셋업으로 의도된 의존성만 깔끔히 들어간다
- **초기 셋업 의존성** (리뷰 v4 Medium 1 반영 — shell plugin 제거 후 단일 정책):
  - `@tauri-apps/cli` (devDep), `@tauri-apps/api` (dep)
  - Tauri plugins: **추가 plugin 없음**. 외부 URL 열기는 백엔드 Rust command (`open_ollama_download_page`) 가 `std::process::Command::new("open")` 으로 처리 (리뷰 v3 High 1, §4 잠금 결정 "외부 URL 열기" 참조)
  - `tauri-plugin-global-shortcut`, `tauri-plugin-clipboard-manager`, `tauri-plugin-fs`, `tauri-plugin-autostart` 는 **Step 6 의 "Phase 진입 시 추가 예정" 표대로 Phase 3/4 진입 시 추가**한다. 본 셋업에는 포함하지 않음
  - **주의**: 정확한 버전은 `library-docs-fetcher` 로 Tauri 2 최신 plugin 매트릭스를 한 번 더 확인한 뒤 확정
- `src-tauri/tauri.conf.json` (Tauri 2 표준 위치. `.claude/CLAUDE.md` 의 구조 예시가 루트 경로를 보여주는 것은 후속 문서 정리 대상으로 §10에 기록):
  - `identifier`: `com.shiron.hytranslate`
  - `productName`: `HyTranslate Mac`
  - `version`: `0.1.0`
  - macOS only 빌드 타겟
  - **CSP 는 dev / prod 분리** (리뷰 #4 반영 — 실제 구현 단계로 격상):
    - **prod CSP** (`build.csp`): `default-src 'self'; connect-src 'self' http://localhost:11434 ipc: http://ipc.localhost; script-src 'self'; style-src 'self' 'unsafe-inline'` — `.claude/rules/security.md` 의 localhost-only 원칙 그대로
    - **dev CSP** (`build.devCsp` 또는 dev 빌드 분기): prod 셋에 더해 `connect-src` 에 `ws://localhost:1420 http://localhost:1420` 추가 — Vite HMR websocket 허용
    - `'unsafe-inline'` 은 style 에만 한정. script 는 dev/prod 모두 거부
  - **capabilities (리뷰 v4 Medium 1 반영 — shell capability 제거 후 단일 정책)**:
    - 초기 셋업에는 **`core:default` 만** 부여한다. 외부 URL 열기는 FE 에 `shell.open` 권한을 부여하지 않고 백엔드 Rust command 가 고정 URL 만 열도록 처리한다 (리뷰 v3 High 1)
    - `fs` (app-data 스코프), `clipboard-manager`, `global-shortcut` capability 는 해당 plugin crate 가 Cargo 에 들어가는 **Phase 3/4 진입 시 동시에 추가**한다 (capability 만 열고 plugin 이 없으면 런타임 설정 불일치 발생)
  - `windows`: `main`, `popup`, `menubar` 3개를 미리 정의하되 `popup` / `menubar` 는 `visible: false` 로 시작

### Step 3 — 프론트엔드 빌드 시스템

- `package.json`
  - scripts:
    - `dev`, `build`, `preview`
    - `tauri:dev`, `tauri:build`
    - `lint` (`eslint .`), `lint:fix` (`eslint . --fix`)
    - `typecheck` (`tsc --noEmit`)
    - `test` (`vitest run`), `test:watch` (`vitest`)
    - `test:e2e` (`playwright test`)
    - `format` (`prettier --write .`), `format:check` (`prettier --check .`)
- **npm 의존성 명시** (리뷰 #2 반영 — `.claude/CLAUDE.md` §Tech Stack 잠금 버전 기준):

  | 분류 | 패키지 | 메모 |
  |---|---|---|
  | runtime `dependencies` | `react@^18`, `react-dom@^18` | UI 런타임 |
  | | `zustand@^4` | feature store (rule: 셀렉터 사용) |
  | | `@tauri-apps/api@^2` | IPC client base |
  | | `lucide-react` | 아이콘 (`.claude/rules/styling.md`) |
  | build/dev `devDependencies` | `vite@^5`, `@vitejs/plugin-react@^4` | 멀티 entry |
  | | `typescript@^5` | strict 모드 |
  | | `@tauri-apps/cli@^2` | `tauri dev/build` |
  | style | `tailwindcss@^3`, `postcss`, `autoprefixer` | dark mode class 토글 |
  | lint/format | `eslint@^9`, `typescript-eslint@^8` | flat config |
  | | `eslint-plugin-react`, `eslint-plugin-react-hooks` | React 규칙 |
  | | `eslint-plugin-jsx-a11y` | 접근성 (`.claude/rules/accessibility.md`) |
  | | `eslint-plugin-import` | import 순서 (`.claude/rules/code-style.md`) |
  | | `prettier@^3` | 포맷 (hook 과 충돌 검증) |
  | test | `vitest@^2`, `jsdom`, `@testing-library/react`, `@testing-library/jest-dom` | FE unit |
  | | `@playwright/test` | E2E config 만 (sanity spec) |
  | type defs | `@types/react`, `@types/react-dom`, `@types/node` | |

  - 정확한 minor 버전은 Step 1 직후 `library-docs-fetcher` 로 호환 매트릭스 확인 후 lockfile 잠금
  - 위 목록에 **없는** 패키지는 셋업 단계에서 추가하지 않는다 (Phase별 필요 시 별도 PR)
- `vite.config.ts`
  - React plugin
  - `build.rollupOptions.input` 으로 멀티 entry 등록 (`main`, `popup`, `menubar`)
  - path aliases (`@/*`, `@components/*`, `@features/*`, `@windows/*`, `@lib/*`, `@i18n/*`, `@styles/*`) ← `.claude/CLAUDE.md` 의 표 그대로 반영
  - `server.strictPort: true`, `server.port: 1420`, HMR을 Tauri 와 맞춤
- `tsconfig.json`
  - `strict: true`, path alias 동일하게 반영, `noUncheckedIndexedAccess: true`, `noFallthroughCasesInSwitch: true`
- `tsconfig.node.json` (Vite / config 파일용 분리)

### Step 4 — Tailwind + 디자인 토큰
- `tailwind.config.ts`
  - `darkMode: 'class'`
  - `content`: `src/**/*.{ts,tsx,html}`
  - `theme.extend.fontFamily.sans`: SF Pro 스택 (`.claude/rules/styling.md`)
  - 색/spacing 토큰은 Phase 1 디자인 진행 시 채움 (지금은 default + brand placeholder 1개)
- `postcss.config.cjs` 표준 셋업
- `src/styles/globals.css`: `@tailwind base/components/utilities` + 다크모드 토글 호환을 위한 CSS 변수 정의 자리

### Step 5 — 린트 / 포맷
- ESLint (flat config, v9): `@typescript-eslint`, `react`, `react-hooks`, `import` 순서, accessibility (`jsx-a11y`)
- Prettier: 2-space, single quote, semi, trailing-comma `es5`, print-width 100 (`.claude/rules/code-style.md`)
- 이미 존재하는 `PostToolUse` hook (`prettier --write`, `rustfmt --edition 2021`) 과 충돌 없는지 검증

### Step 6 — Rust 빌드 시스템 & 의존성

**의존성 추가 정책** (리뷰 #5 반영): 초기 셋업에는 **빈 앱이 컴파일·실행되고 `AppError` / 이벤트 상수 스켈레톤이 빌드되는 데 필요한 최소 의존성**만 잠근다. Ollama / DB / streaming / mock 의존성은 각 Phase 진입 시 별도 추가한다.

- `src-tauri/Cargo.toml`
  - edition `2021`
  - **초기 셋업에 포함하는 의존성** (지금 잠금):

    | crate | 용도 | 근거 |
    |---|---|---|
    | `tauri@^2` (`macos-private-api` feature) | 앱 진입점 / 윈도우 등록 + 투명 popup 등 macOS 전용 효과 | `.claude/CLAUDE.md` Tech Stack, §4 잠금 결정 "macOS private API" |
    | `tauri-build@^2` | `build.rs` Tauri 빌드 스크립트 | Tauri 2 필수 build-dep (계획서 v4 보강) |
    | `tokio` (`rt-multi-thread`, `macros`, `sync`) | 모든 `#[tauri::command]` 가 async | `.claude/rules/rust-style.md` Async/Tokio |
    | `serde`, `serde_json` | 모든 IPC 페이로드 | tauri-ipc 규칙 |
    | `thiserror` | `AppError` derive | `.claude/rules/rust-style.md` Error handling |
    | `tracing`, `tracing-subscriber` | 라이프사이클 로그 | `.claude/rules/rust-style.md` Logging |
    | `uuid@^1` (`v4`) | `requestId` 생성 / 검증 | tauri-ipc 규칙 |

  - dev-dependencies (지금 잠금): `pretty_assertions`, `tempfile`
  - **Phase 진입 시 추가 예정** (지금은 잠그지 않음):

    | crate | 추가 시점 | PRD/Rule 근거 |
    |---|---|---|
    | `reqwest` (`stream`, `json`, `rustls-tls`) | Phase 1 — Ollama HTTP client | PRD §8.1, `.claude/rules/api-handling.md` |
    | `tokio-util` (CancellationToken) | Phase 1 — 요청 취소 | PRD §10.2, tauri-ipc cancellation |
    | `dashmap` | Phase 1 — in-flight request 토큰 맵 | tauri-ipc cancellation |
    | `futures-util` | Phase 1 — `bytes_stream` 처리 | `.claude/rules/api-handling.md` |
    | `rusqlite` (`bundled`) | Phase 4 — DB 도입 | PRD §9.4 |
    | `r2d2`, `r2d2_sqlite` | Phase 4 — pool | `.claude/rules/rust-style.md` Persistence |
    | `tauri-plugin-global-shortcut` | Phase 3 — Cmd+Shift+T | PRD §8.5 |
    | `tauri-plugin-clipboard-manager` | Phase 3 — 클립보드 | PRD §6.4 |
    | `tauri-plugin-autostart` | Phase 3 — 로그인 시 실행 | PRD §9.2 |
    | `tauri-plugin-fs` (app-data scope) | Phase 4 — DB 경로 / 로그 | `.claude/rules/security.md` |
    | dev: `wiremock` | Phase 1 — Ollama mock streaming | `.claude/rules/testing.md` |

  - **검증 순서**: 각 Phase 첫 작업 직전에 `library-docs-fetcher` 로 해당 crate 의 Tauri 2 호환성 확인
- `src-tauri/build.rs` 표준 Tauri 빌드 스크립트

### Step 7 — Rust 스켈레톤 코드
- `main.rs`: `tauri::Builder::default()` 로 빈 앱 실행. 단축키/메뉴바/DB는 비활성
- `lib.rs`: 모듈 선언만
- `errors.rs` — **공개 vs 내부 에러 분리** (리뷰 #3 반영):
  - **공개 `AppError`** (= `#[tauri::command]` 가 반환하는 직렬화 shape) 는 `.claude/rules/tauri-ipc.md` 의 **7 variant 만** 노출한다:
    - `OllamaUnavailable`, `OllamaNotRunning`, `ModelMissing { model }`, `InputTooLong { limit }`, `Cancelled`, `NetworkBlocked`, `Internal { message }`
    - `#[serde(tag = "kind")]` 로 discriminated union 직렬화. derive: `thiserror::Error`, `serde::Serialize`
  - **내부 Rust 에러** (`Db(rusqlite::Error)`, `Network(reqwest::Error)`, `Serialization(serde_json::Error)` 등) 는 **별도 enum** (예: `InternalError`) 에 둔다. Phase 진입 시 실제로 필요한 시점에 추가
  - **boundary 매핑**: `commands/` 어댑터가 `InternalError` → `AppError::Internal { message }` 로 변환. 또는 의미가 분명한 케이스 (예: `reqwest::Error::connect` → `OllamaNotRunning`) 는 명시적 분기
  - 초기 셋업에서는 공개 `AppError` 7 variant 와 빈 매핑 헬퍼 자리만 만들고, `From<reqwest::Error>` / `From<rusqlite::Error>` 는 **Phase 1 / Phase 4 진입 시** 추가
- `events.rs`: 이벤트 이름 상수 (`TRANSLATION_STARTED`, `TRANSLATION_CHUNK`, ...). FE `events.ts` 와 1:1
- 각 도메인 모듈 (`ollama/`, `language/`, `history/`, `settings/`, `db/`, `shortcuts/`, `menubar/`) 은 `mod.rs` 만 두고 빈 `pub` 표면
- `commands/mod.rs`: 핸들러 0개, register 헬퍼만

**`src/lib/ipc/errors.ts` 제약**: 공개 `AppError` 7 variant 만 mirror 한다. 내부 Rust 에러 종류는 FE 에 노출되지 않는다.

### Step 8 — Frontend 스켈레톤 코드
- `src/windows/main/main.tsx`: `<App />` 렌더, `<App />` 은 "HyTranslate Mac" 만 출력
- `src/windows/popup/popup.tsx`, `src/windows/menubar/menubar.tsx`: 각각 동일 수준의 placeholder
- `src/lib/ipc/client.ts`: `invoke<T>(name, payload)` 얇은 래퍼 + `listen` 래퍼
- `src/lib/ipc/events.ts`: 이벤트 이름 상수 (Rust 와 1:1)
- `src/lib/ipc/errors.ts`: `AppError` discriminated union 미러
- `src/i18n/ko.ts`: 빈 객체 + `t(key)` 헬퍼 시그니처만

### Step 9 — 테스트 인프라
- Vitest: `vitest.config.ts` (jsdom 환경, path alias 동기화, `setup.ts` 스텁)
- Playwright: `playwright.config.ts` (macOS only, base URL Tauri 환경 변수로 후술)
  - **주의**: Tauri 2 + Playwright 연결은 `tauri-driver` 또는 `WebDriver` 기반 → `library-docs-fetcher` 로 v2 최신 패턴 재확인 필요
- 빈 sanity 테스트 1개씩 (`describe('sanity', () => it('passes', () => expect(true).toBe(true)))`)

### Step 10 — Eval 자리 마련
- `evals/translation-quality.md` 헤더 + 빈 sample 표 (PRD §14.1 의 40/40/20 슬롯, 도메인 5개) → Phase 4 ~ Phase 5 사이에 평가팀이 채울 자리

### Step 11 — 첫 커밋 전 검증 (리뷰 #1 반영 — §1 완료 기준과 1:1 동기화)

**FE 검증** (모두 통과해야 완료):
- `npm install` → `package-lock.json` 생성 확인
- `npm run format:check` (`prettier --check .`)
- `npm run lint`
- `npm run typecheck`
- `npm run test` (Vitest sanity spec 통과)
- `npx playwright install --with-deps chromium` 후 `npm run test:e2e` (sanity spec 1개. tauri-driver 없이 vitest 수준 sanity 가능한 spec)
  - tauri-driver 연결을 미루는 결정 (§7) 에 따라 본 단계의 E2E sanity 는 **Playwright 자체가 실행되고 config 가 파싱되는지** 확인하는 수준의 빈 spec 으로 한정. 실제 Tauri 윈도우 driving 은 Phase 1 후반에 활성화

**BE 검증** (모두 통과해야 완료):
- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- `cargo clippy --manifest-path src-tauri/Cargo.toml -- -D warnings`
- `cargo check --manifest-path src-tauri/Cargo.toml`
- `cargo test --manifest-path src-tauri/Cargo.toml` (빈 테스트 모듈 통과)

**런타임 smoke**:
- `npm run tauri:dev` 으로 빈 메인 창이 뜨는지 사용자 화면에서 수동 확인
- `popup`, `menubar` 윈도우는 `visible: false` 로 등록되어 있을 뿐 표시되지 않음을 확인

---

## 4. 잠금된 결정 — 그대로 따른다 (PRD §18 & rules)

| 항목 | 결정 |
|---|---|
| 패키지 매니저 | **npm** (`.claude/CLAUDE.md` Commands 섹션이 `npm` 으로 통일) |
| Tauri 버전 | **2.x latest stable** |
| Node 버전 | **20 LTS** |
| Rust edition | **2021** |
| TLS | **rustls** (system OpenSSL 회피) |
| DB 드라이버 | **`rusqlite` bundled + `r2d2_sqlite` pool** |
| 멀티 윈도우 | main / popup / menubar **3개 초기 등록** (popup, menubar 는 invisible start) |
| UI 언어 | 한국어 (코드 내부 placeholder 도 한국어 i18n key 기준) |
| Bundle identifier | **`com.shiron.hytranslate`** (인터뷰 확정) |
| productName | **`HyTranslate Mac`** (인터뷰 확정) |
| 코드 서명 | **dev 빌드만, 서명/notarization 미설정** (v1 배포 직전 별도 작업) |
| Playwright 범위 | **config + 빈 sanity spec 1개만**. `tauri-driver` 연결은 Phase 1 후반에 켬 |
| 배포 채널 | **DMG 직접 배포 only**. App Store 배포는 현재 범위 외 (리뷰 v3 Low 1) |
| macOS private API | **허용** (`tauri.app.macOSPrivateApi: true` + Cargo `macos-private-api` feature). popup `transparent: true` 등 macOS 전용 UI 효과를 위해 필요. App Store 미고려 결정에 따른 trade-off 명시 (리뷰 v3 Low 1) |
| 외부 URL 열기 | **백엔드 Rust command 로 감싼다** (예: `open_ollama_download_page`). `tauri-plugin-shell` 미사용. FE 에 `shell.open` 권한 직접 부여 안 함. 백엔드는 `std::process::Command::new("open")` 으로 macOS 시스템 열기 수행 (리뷰 v3 High 1) |
| 문서 포맷 정책 | `prettier --check .` 의 대상에서 **`.claude/agents/`, `.claude/commands/`, `.claude/rules/`, `.claude/CLAUDE.md`, `CLAUDE.local.md`, `docs/hytranslate-mac-prd.md`, `docs/plans/00-initial-setup.md`** 만 제외. 신규 plan / review 문서는 format:check 대상에 포함 (리뷰 v3 Low 2) |
| 셋업 완료 후 흐름 | **셋업 완료 → 단일 커밋 → 검토 대기**. Phase 1 진입은 별도 지시 후 |

## 5. 명시적으로 본 단계에서 **하지 않을** 일

- 실제 Ollama HTTP 호출 코드
- 실제 SQLite 스키마/migration 작성
- 단축키 등록 / popup 표시 로직
- 메뉴바 popover 동작
- 디자인 토큰 채우기 (브랜드 컬러 미정)
- 코드 서명 / notarization 설정 (배포 단계)

## 6. 위험 요인 & 완화

| 위험 | 완화 |
|---|---|
| Tauri 2 plugin 버전 매트릭스가 빠르게 변함 | Step 2/6 직전에 `library-docs-fetcher` 호출로 최신 호환표 확인 후 의존성 잠금 |
| Playwright + Tauri 2 연결 패턴이 v1 과 다름 | Step 9 직전에 동일하게 docs fetch 확인. 초기에는 webdriver 없이 sanity spec 만 두고 tauri-driver 연결은 Phase 1 후반에 켬 |
| dev HMR websocket 차단 | **Step 2 의 dev/prod CSP 분리로 해결 — 더 이상 리스크 아님**. dev CSP 는 `ws://localhost:1420` 추가 |
| 멀티 entry Vite 설정이 Tauri windows config 와 불일치 가능 | Step 3 와 Step 2 의 url 매핑 표를 한 번 더 검토 |
| Phase별 의존성 추가 누락 | Step 6 의 "Phase 진입 시 추가 예정" 표를 Phase 진입 계획서에서 한 번 더 인용 |

---

## 7. 인터뷰 결과 (확정)

| # | 항목 | 결정 |
|---|---|---|
| 1 | Bundle identifier | `com.shiron.hytranslate` |
| 2 | productName | `HyTranslate Mac` |
| 3 | 셋업 완료 후 진입 | 셋업 완료 후 멈추고 사용자 검토 대기. Phase 1 계획서는 별도 지시 후 작성 |
| 4 | 코드 서명 | dev only, 서명/notarization 미설정 |
| 5 | Playwright 범위 | config + 빈 sanity spec 만. tauri-driver 연결은 Phase 1 후반 |

---

## 8. 산출물 체크리스트 (사용자 검토용 — Step 11 검증 1:1 동기화)

**파일 생성**
- [ ] `.gitignore`, `.editorconfig`, `.nvmrc`, `rust-toolchain.toml`
- [ ] `package.json` (deps/devDeps Step 3 명시 그대로), `package-lock.json`
- [ ] `tsconfig.json`, `tsconfig.node.json`
- [ ] `vite.config.ts`, `tailwind.config.ts`, `postcss.config.cjs`, `src/styles/globals.css`
- [ ] `eslint.config.mjs`, `.prettierrc`
- [ ] `vitest.config.ts`, `tests/setup.ts`, `playwright.config.ts`
- [ ] `tests/e2e/sanity.spec.ts` (빈 spec)
- [ ] `src/windows/{main,popup,menubar}/{index.html, *.tsx}`
- [ ] `src/lib/ipc/{client,events,errors}.ts`, `src/i18n/ko.ts`
- [ ] `src-tauri/Cargo.toml`, `src-tauri/build.rs`, `src-tauri/tauri.conf.json` (dev/prod CSP 분리 반영)
- [ ] `src-tauri/src/{main,lib,errors,events}.rs` (공개 `AppError` 7 variant 만 노출)
- [ ] `src-tauri/src/{commands,ollama,language,history,settings,db,shortcuts,menubar}/mod.rs`
- [ ] `evals/translation-quality.md` (헤더만)

**FE 검증 통과**
- [ ] `npm install` lockfile 생성
- [ ] `npm run format:check`
- [ ] `npm run lint`
- [ ] `npm run typecheck`
- [ ] `npm run test`
- [ ] `npm run test:e2e` (sanity spec)

**BE 검증 통과**
- [ ] `cargo fmt -- --check`
- [ ] `cargo clippy -- -D warnings`
- [ ] `cargo check`
- [ ] `cargo test`

**런타임 smoke**
- [ ] `npm run tauri:dev` 으로 빈 메인 창 표시. `popup`/`menubar` 는 hidden 확인

---

## 9. 다음 액션

본 계획서가 승인되면 다음 순서로 진행한다.

1. `library-docs-fetcher` 로 Tauri 2 / plugins / Vitest / Playwright-Tauri 최신 호환 정보 확인
2. Step 1 ~ Step 11 순차 실행
3. 매 Step 후 `quality-gate` 호출 (`.ts/.tsx/.rs/.css` 변경 발생 시)
4. 최종 smoke 통과 후 conventional commit (`chore(scaffold): tauri 2 + react + rust 초기 셋업`) 으로 단일 커밋
5. **여기서 멈추고 사용자 검토 대기**. Phase 1 계획서 (`docs/plans/01-phase1-core-translation.md`) 는 별도 지시 후 작성

---

## 10. 리뷰 반영 기록 (`docs/review/00-initial-setup-review.md`)

### v1 리뷰 (모두 닫힘)

| # | 심각도 | 발견 | 반영 위치 |
|---|---|---|---|
| 1 | High | Step 11 검증이 §1 완료 기준과 불일치 | Step 11 전면 재작성 + §8 체크리스트 동기화 + §1 요약 갱신 |
| 2 | Medium | npm 의존성 목록 누락 | Step 3 에 deps/devDeps 카테고리별 표 추가 |
| 3 | Medium | `AppError` 공개 직렬화 계약 모호 | Step 7 에 공개 7 variant vs 내부 Rust 에러 분리, boundary 매핑 명시. `errors.ts` mirror 범위 제약 |
| 4 | Medium | dev/prod CSP 분리가 리스크 표에만 존재 | Step 2 에 prod/dev CSP 셋을 구체 문자열로 명시. §6 리스크 표에서 해당 항목 해결 표시 |
| 5 | Low | 초기 셋업에서 Phase별 의존성을 과도하게 잠금 | Step 6 을 "초기 셋업 최소" vs "Phase 진입 시 추가" 두 표로 재구성 |
| 6 | Low | `tauri.conf.json` 위치 불일치 | Step 2 에 "Tauri 2 표준 위치" 주석 추가. `.claude/CLAUDE.md` 구조 예시 정리는 후속 문서 작업으로 표시 |

### v2 재리뷰 (이후 v3 코드리뷰에서 다시 변경됨)

| # | 심각도 | 발견 | 반영 위치 |
|---|---|---|---|
| 1 | Medium | Step 2 의 Tauri plugin/capability 목록이 Step 6 의 "초기 최소" 정책과 충돌 | (당시) **Step 2 plugin 목록을 `tauri-plugin-shell` 만으로 축소, capability 도 `shell.open` 만**. → **v3 코드리뷰 High 1 에 의해 다시 변경**: shell plugin 완전 제거, capability 는 `core:default` 만 |

### v3 코드리뷰 — `docs/review/00-initial-setup-code-review.md` (구판)

| # | 심각도 | 발견 | 반영 위치 |
|---|---|---|---|
| 1 | High | `shell:allow-open` 이 Ollama 설치 페이지로 제한되지 않음 | **`tauri-plugin-shell` crate / `@tauri-apps/plugin-shell` npm 의존성 / `shell:allow-open` capability 전부 제거**. `commands/mod.rs` 에 `open_ollama_download_page` 추가, 백엔드 상수 `OLLAMA_DOWNLOAD_URL` 만 `std::process::Command::new("open")` 으로 연다. §4 잠금 결정 "외부 URL 열기" 행 추가 |
| 2 | Medium | 계획서의 디렉터리 구조 일부가 산출물로 남지 않음 | `src/{components,features,lib/hooks,types}/README.md` 추가 (인터뷰 결정: `.gitkeep` 대신 `README.md`) |
| 3 | Medium | FE `isAppError` 가 variant 별 필수 필드를 검증하지 않음 | `src/lib/ipc/errors.ts` 의 `isAppError` 를 `kind` switch + variant payload 타입 검증으로 강화. `src/lib/ipc/errors.test.ts` 5 케이스 추가 |
| 4 | Low | `macOSPrivateApi` 사용 결정이 계획서에 기록되어 있지 않음 | §4 잠금 결정에 "배포 채널 = DMG only", "macOS private API = 허용" 행 추가 |
| 5 | Low | `.prettierignore` 가 계획서보다 넓어 문서 변경 포맷 검증을 우회 | `.prettierignore` 를 디렉터리 통째에서 구체 파일 경로 7개로 좁힘. §4 잠금 결정에 "문서 포맷 정책" 행 추가 |

### v4 코드리뷰 — `docs/review/00-initial-setup-code-review.md` (현행)

| # | 심각도 | 발견 | 반영 위치 |
|---|---|---|---|
| 1 | Medium | `tauri-plugin-shell` 미사용 결정이 계획서와 npm 의존성에 완전히 반영되지 않음 | `package.json` 에서 `@tauri-apps/plugin-shell` 제거 + lockfile 갱신. Step 2 plugin 목록을 "추가 plugin 없음" 으로 정정. Step 2 capability 설명을 `core:default` 만 으로 정정. Step 6 초기 의존성 표에서 `tauri-plugin-shell` 행 제거 (대신 누락되어 있던 `tauri-build@^2` 행을 보강). §10 v2 기록을 v3 으로 supersede 처리. v3 / v4 기록 신규 추가 |

후속 문서 작업 (본 셋업 범위 외):
- `.claude/CLAUDE.md` 의 프로젝트 구조 예시에서 `tauri.conf.json` 위치를 `src-tauri/tauri.conf.json` 로 정정
