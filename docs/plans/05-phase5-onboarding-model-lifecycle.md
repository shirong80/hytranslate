# Phase 5 — 온보딩과 모델 lifecycle

PRD §15.5 / §6.1 / §7.4 / §8.4 / §10.4 / §10.5 구현.

## 목표

- 비개발자도 첫 실행에서 Ollama 설치 / 실행 / 모델 다운로드를 마치고 번역 화면으로 진입할 수 있다.
- 환경 (macOS 버전, 아키텍처, 메모리) 을 감지해 12 GB 미만 시 1.8B 모델을 추천한다.
- 모델 pull 은 사용자가 명시 승인할 때만 시작하고 진행률을 표시한다.
- 실패 시 inline 메시지 + 재시도 (PRD §11).

## 잠금 결정 (Phase 5 한정)

| 항목 | 결정 |
|---|---|
| 추천 임계값 | RAM ≥ 12 GB → Hy-MT2 7B, 미만 → 1.8B (PRD §6.1) |
| 시스템 감지 | `sw_vers -productVersion` + `sysctl hw.memsize`. arch 는 `std::env::consts::ARCH` |
| Ollama 실행 감지 | `/api/tags` 200 응답 = 실행 중. connect 실패 → `OllamaNotRunning` 으로 매핑되지만 status 응답에서는 `running:false` 로 변환 |
| 모델 pull | `/api/pull` stream. terminal 은 `{"status":"success"}` 한 줄 |
| 자동 실행 | v1 은 자동 실행 시도 없이 사용자가 직접 실행. 공식 다운로드 페이지 링크만 제공 |
| 권한 안내 | accessibility 권한은 settings 의 persistent CTA 와 onboarding 안내로 처리. Phase 5 에서 자동 detect/prompt 는 보류 |
| 화면 | 메인 윈도우 안에서 step indicator 가 있는 단일 panel. settings.onboarding_completed=false 일 때만 표시 |
| 저장 시점 | finish 버튼 → `complete_onboarding` → settings flag 영속화 |

## Backend

- `environment::detect()` — `EnvironmentReport { macosVersion, macosMajor, macosSupported, arch, totalMemoryGb, recommendedModel }`
- `ollama::client::list_models` — `GET /api/tags`
- `ollama::client::pull_model_stream` — `POST /api/pull` (stream), `PullChunk { status, digest, total, completed, error }`
- `commands::onboarding`
  - `detect_environment` → `EnvironmentReport`
  - `get_ollama_status` → `OllamaStatus { running, endpoint, models }`
  - `pull_model { model }` — 토큰 등록 후 비동기 worker spawn. 이벤트 `model-pull:started | progress | error | completed`
  - `cancel_model_pull { model }` — 토큰 fire. 별도 이벤트 없음
  - `complete_onboarding` — settings flag persist
- `PullRegistry`: DashMap<model, CancellationToken>

## Frontend

- `src/features/onboarding/`
  - `types.ts` — `EnvironmentReport`, `OllamaStatus`, `OnboardingStep`, `HY_MT2_7B / 1_8B` 상수
  - `ipc.ts` — invoke + listen wrappers
  - `store.ts` — zustand. `step / env / ollama / selectedModel / pullingModel / progress / installedSinceStart`
  - `components/onboarding-screen.tsx` — step indicator + welcome / environment / ollama / model / permissions / history 카드
- `src/windows/main/main.tsx` — `loaded && !onboardingCompleted` 분기로 OnboardingScreen 진입
- `src/i18n/ko.ts` — 50 여 개 `onboarding.*` 키 추가

## 완료 기준 (PRD §15.5)

- Ollama 가 없는 사용자는 공식 다운로드 링크를 받는다.
- 모델이 없는 사용자는 앱 안에서 다운로드를 시작할 수 있다.
- 다운로드 실패 시 재시도할 수 있다 (`error` 시 store 가 idle 로 돌아가고 사용자는 다시 startPull 가능).
- 온보딩 완료 후 정상 번역 화면으로 진입한다.

## 테스트

- Rust: 9 개 ollama (`list_models` x3, `pull_model_stream` x3) + 5 개 environment (parse / threshold / serialize) + 2 개 PullRegistry — 총 +16. 전체 106 통과.
- FE: 13 개 onboarding store + toProgressView 분기 — 총 55 통과.
- 수동: dev 빌드에서 첫 실행 시 onboarding 노출, `finish` 후 메인으로 진입, `onboardingCompleted=false` 로 settings.json 복원 시 다시 진입.
