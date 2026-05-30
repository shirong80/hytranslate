//! Phase 1: `translate_stream` / `cancel_translation` 명령.
//!
//! 핸들러는 30,000자 cap 검증 후 worker 를 spawn 한다. worker 는 chunk 마다
//! `translation:chunk` 이벤트를 emit 하고, 종료 시점에 `completed` / `cancelled` /
//! `error` 중 정확히 하나를 emit 한다.

use std::sync::Arc;
use std::sync::Mutex;
use std::time::Instant;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::events::{
    TRANSLATION_CANCELLED, TRANSLATION_CHUNK, TRANSLATION_COMPLETED, TRANSLATION_ERROR,
    TRANSLATION_STARTED,
};
use crate::language::{self, SourceLanguage};
use crate::ollama::{ChunkFlow, OllamaClient};
use crate::settings::SettingsStore;

pub const MAIN_INPUT_LIMIT: usize = 30_000;

/// 요청별 cancel token + terminal-state guard.
///
/// 저장은 더 이상 worker 에서 일어나지 않는다 (이력 저장은 사용자가 Cmd+Enter 로
/// `save_translation_record` 를 명시 호출). 그래도 terminal guard 는 유지한다 — 늦게 도착한
/// cancel 이 이미 성공한 stream 결과를 `completed` 로 emit 하지 못하게 막는 이벤트 정합성
/// 보장 때문. 한쪽이 lock 을 잡고 `claimed = true` 로 마킹하면 다른 쪽은 noop:
///
/// - cancel 이 먼저 잡으면: token.cancel() + claimed=true. 이후 worker 는 lock 안에서
///   claimed==true 를 보고 `cancelled` 를 emit (Ok 결과여도 completed 로 흘리지 않음).
/// - worker 가 먼저 잡으면: claimed=true 후 `completed` emit. 그 뒤 도착한 cancel 은
///   claimed==true 를 보고 false 반환 (no-op).
pub struct RequestState {
    pub token: CancellationToken,
    pub terminal: Mutex<bool>,
}

impl RequestState {
    fn new(token: CancellationToken) -> Arc<Self> {
        Arc::new(Self {
            token,
            terminal: Mutex::new(false),
        })
    }
}

/// 진행 중인 번역 요청 상태 맵. `register` 에서 만들어 `manage` 한다.
#[derive(Default)]
pub struct TranslationRegistry {
    states: DashMap<String, Arc<RequestState>>,
}

impl TranslationRegistry {
    pub fn insert(&self, request_id: String, token: CancellationToken) -> Arc<RequestState> {
        let state = RequestState::new(token);
        self.states.insert(request_id, state.clone());
        state
    }

    pub fn get(&self, request_id: &str) -> Option<Arc<RequestState>> {
        self.states.get(request_id).map(|r| r.value().clone())
    }

    pub fn remove(&self, request_id: &str) {
        self.states.remove(request_id);
    }

    /// cancel 도 같은 mutex 안에서 terminal claim 을 시도한다.
    /// 이미 worker 가 commit 으로 claim 했다면 `false` 를 반환하고 token 도 흔들지 않는다.
    pub fn cancel(&self, request_id: &str) -> bool {
        let Some(state) = self.states.get(request_id).map(|r| r.value().clone()) else {
            return false;
        };
        let mut claimed = state
            .terminal
            .lock()
            .expect("translation terminal lock poisoned");
        if *claimed {
            return false;
        }
        *claimed = true;
        state.token.cancel();
        true
    }

    pub fn is_empty(&self) -> bool {
        self.states.is_empty()
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.states.len()
    }
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslateRequest {
    pub source_text: String,
    pub source_language: SourceLanguage,
    pub model: String,
    pub request_id: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct StartedPayload {
    request_id: String,
    model: String,
    started_at_ms: u128,
    resolved_language: SourceLanguage,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ChunkPayload {
    request_id: String,
    delta: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CompletedPayload {
    request_id: String,
    full_text: String,
    duration_ms: u128,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct CancelledPayload {
    request_id: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct ErrorPayload {
    request_id: String,
    error: AppError,
}

fn validate_request_id(id: &str) -> AppResult<()> {
    Uuid::parse_str(id).map_err(|e| AppError::Internal {
        message: format!("invalid requestId (must be UUID v4): {e}"),
    })?;
    Ok(())
}

fn now_ms() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
}

#[tauri::command]
pub async fn translate_stream<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    registry: tauri::State<'_, Arc<TranslationRegistry>>,
    client: tauri::State<'_, OllamaClient>,
    settings: tauri::State<'_, Arc<SettingsStore>>,
    request: TranslateRequest,
) -> AppResult<()> {
    if request.source_text.chars().count() > MAIN_INPUT_LIMIT {
        return Err(AppError::InputTooLong {
            limit: MAIN_INPUT_LIMIT,
        });
    }
    validate_request_id(&request.request_id)?;

    // PRD §8.2 — Auto 입력은 backend 가 즉시 detect 한다. UI 가 다시 보내지 않으므로
    // race 가 없다. detector 가 모호하다고 결정한 경우에도 `Auto` 그대로 두어
    // prompt 가 generic `Chinese` 라벨을 쓰게 한다.
    let resolved_language = if request.source_language == SourceLanguage::Auto {
        language::detect(&request.source_text).language
    } else {
        request.source_language
    };

    let endpoint = settings.get().ollama_endpoint;

    let token = CancellationToken::new();
    registry.insert(request.request_id.clone(), token.clone());

    let app_handle = app.clone();
    let client = (*client).clone();
    let registry_inner = registry.inner().clone();
    let resolved_request = TranslateRequest {
        source_language: resolved_language,
        ..request
    };

    tokio::spawn(async move {
        run_translation(
            app_handle,
            client,
            registry_inner,
            endpoint,
            resolved_request,
            token,
        )
        .await;
    });

    Ok(())
}

#[tauri::command]
pub async fn cancel_translation(
    registry: tauri::State<'_, Arc<TranslationRegistry>>,
    request_id: String,
) -> AppResult<()> {
    validate_request_id(&request_id)?;
    registry.cancel(&request_id);
    Ok(())
}

async fn run_translation<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    client: OllamaClient,
    registry: Arc<TranslationRegistry>,
    endpoint: String,
    request: TranslateRequest,
    token: CancellationToken,
) {
    let request_id = request.request_id.clone();
    let model = request.model.clone();
    let started = Instant::now();

    let _ = app.emit(
        TRANSLATION_STARTED,
        StartedPayload {
            request_id: request_id.clone(),
            model: model.clone(),
            started_at_ms: now_ms(),
            resolved_language: request.source_language,
        },
    );
    tracing::info!(
        request_id = %request_id,
        model = %model,
        source_language = ?request.source_language,
        source_len = request.source_text.chars().count(),
        "translation:started"
    );

    let emit_app = app.clone();
    let emit_request_id = request_id.clone();
    let emit_token = token.clone();

    let result = client
        .generate_stream(
            &endpoint,
            &model,
            request.source_language,
            &request.source_text,
            &token,
            move |delta| {
                if emit_token.is_cancelled() {
                    return ChunkFlow::Stop;
                }
                let _ = emit_app.emit(
                    TRANSLATION_CHUNK,
                    ChunkPayload {
                        request_id: emit_request_id.clone(),
                        delta: delta.to_string(),
                    },
                );
                ChunkFlow::Continue
            },
        )
        .await;

    // terminal-state guard — 늦게 도착한 cancel 이 성공한 stream 결과를 completed 로
    // 덮어쓰지 못하게 cancel 과 worker 의 terminal 결정을 같은 mutex critical section 으로 묶는다.
    let state = registry
        .get(&request_id)
        .expect("registry has the state we just inserted");

    let outcome = decide_outcome(&state, result, started.elapsed().as_millis());
    registry.remove(&request_id);

    match outcome {
        TerminalEvent::Completed {
            full_text,
            duration_ms,
        } => {
            let _ = app.emit(
                TRANSLATION_COMPLETED,
                CompletedPayload {
                    request_id: request_id.clone(),
                    full_text,
                    duration_ms,
                },
            );
            tracing::info!(
                request_id = %request_id,
                duration_ms,
                "translation:completed"
            );
        }
        TerminalEvent::Cancelled => {
            let _ = app.emit(
                TRANSLATION_CANCELLED,
                CancelledPayload {
                    request_id: request_id.clone(),
                },
            );
            tracing::info!(request_id = %request_id, "translation:cancelled");
        }
        TerminalEvent::Error(err) => {
            tracing::warn!(request_id = %request_id, error.kind = ?err, "translation:error");
            let _ = app.emit(
                TRANSLATION_ERROR,
                ErrorPayload {
                    request_id: request_id.clone(),
                    error: err,
                },
            );
        }
    }
}

#[derive(Debug)]
enum TerminalEvent {
    Completed {
        full_text: String,
        duration_ms: u128,
    },
    Cancelled,
    Error(AppError),
}

/// terminal mutex 를 잡고 outcome 을 결정한다. 저장은 하지 않는다 — cancel 과 worker 의
/// terminal 결정을 같은 critical section 으로 묶어, 늦게 도착한 cancel 이 성공 결과를
/// completed 로 흘리지 못하게 한다.
fn decide_outcome(
    state: &RequestState,
    result: AppResult<String>,
    duration_ms: u128,
) -> TerminalEvent {
    let mut claimed = state
        .terminal
        .lock()
        .expect("translation terminal lock poisoned");

    match result {
        Ok(full_text) => {
            if *claimed {
                // cancel 이 race 에서 이겼다.
                TerminalEvent::Cancelled
            } else {
                *claimed = true;
                TerminalEvent::Completed {
                    full_text,
                    duration_ms,
                }
            }
        }
        Err(AppError::Cancelled) => {
            // stream 내부 cancel — cancel() 호출이 이미 claimed=true 로 마킹했을 것.
            *claimed = true;
            TerminalEvent::Cancelled
        }
        Err(err) => {
            if *claimed {
                TerminalEvent::Cancelled
            } else {
                *claimed = true;
                TerminalEvent::Error(err)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registry_inserts_and_cancels() {
        let reg = TranslationRegistry::default();
        let token = CancellationToken::new();
        reg.insert("abc".to_string(), token.clone());
        assert_eq!(reg.len(), 1);
        assert!(reg.cancel("abc"));
        assert!(token.is_cancelled());
        // Critical 1 v3 — registry entry 는 worker 가 정리할 때까지 살아 있다.
        // cancel 은 token 만 흔들고 entry 는 그대로 두어, worker 가 lock 안에서
        // terminal 상태를 보고 emit cancelled 로 마무리할 수 있게 한다.
        assert_eq!(reg.len(), 1);
        reg.remove("abc");
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn registry_cancel_missing_returns_false() {
        let reg = TranslationRegistry::default();
        assert!(!reg.cancel("nope"));
    }

    /// Critical 1 v3 회귀 — cancel 이 lock 을 잡고 claimed=true 로 마킹한 뒤에는
    /// 두 번째 cancel 이 false 를 반환한다. worker 가 commit 으로 claim 한 후의
    /// cancel 도 동일하게 false (no-op) — 이미 settled 인 terminal state 를 다시 흔들지 않음.
    #[test]
    fn registry_cancel_is_idempotent_under_terminal_guard() {
        let reg = TranslationRegistry::default();
        let token = CancellationToken::new();
        reg.insert("abc".to_string(), token.clone());
        assert!(reg.cancel("abc"));
        assert!(token.is_cancelled());
        // 두 번째 cancel — 이미 claimed, false.
        assert!(!reg.cancel("abc"));
    }

    /// Critical 1 v3 회귀 — worker 가 먼저 terminal lock 을 잡고 claim 하면 cancel 은
    /// false 를 반환해 token 도 흔들지 않는다. 같은 critical section 내 ordering 보장.
    #[test]
    fn worker_claim_blocks_subsequent_cancel() {
        let reg = TranslationRegistry::default();
        let token = CancellationToken::new();
        reg.insert("abc".to_string(), token.clone());
        // worker 가 lock 을 잡아 commit 했다고 가정 — claimed=true.
        let state = reg.get("abc").unwrap();
        {
            let mut guard = state.terminal.lock().unwrap();
            *guard = true;
        }
        // 이후 cancel 은 false. token 도 그대로.
        assert!(!reg.cancel("abc"));
        assert!(!token.is_cancelled());
    }

    #[test]
    fn validate_request_id_rejects_non_uuid() {
        assert!(validate_request_id("not-a-uuid").is_err());
        assert!(validate_request_id(&Uuid::new_v4().to_string()).is_ok());
    }

    /// 늦게 도착한 cancel 이 먼저 claim 하면 stream 이 Ok 여도 Cancelled 를 반환한다.
    #[test]
    fn decide_outcome_yields_cancelled_when_cancel_wins() {
        let state = RequestState::new(CancellationToken::new());
        // cancel 이 먼저 잡음 — claimed=true.
        {
            let mut guard = state.terminal.lock().unwrap();
            *guard = true;
        }
        let out = decide_outcome(&state, Ok("Hello".to_string()), 100);
        assert!(matches!(out, TerminalEvent::Cancelled));
    }

    /// worker 가 lock 을 먼저 잡으면 Completed 를 반환하고 claimed=true 로 마킹한다.
    #[test]
    fn decide_outcome_completes_and_claims_when_worker_wins() {
        let state = RequestState::new(CancellationToken::new());
        let out = decide_outcome(&state, Ok("Hello".to_string()), 100);
        match out {
            TerminalEvent::Completed {
                full_text,
                duration_ms,
            } => {
                assert_eq!(full_text, "Hello");
                assert_eq!(duration_ms, 100);
            }
            other => panic!("expected Completed, got {other:?}"),
        }
        // worker 가 이미 claim 했으니 후속 cancel 은 noop.
        assert!(*state.terminal.lock().unwrap());
    }

    /// stream 자체가 Err(Cancelled) 로 끝났을 때는 항상 Cancelled (claimed 마킹).
    #[test]
    fn decide_outcome_handles_inner_cancel_error() {
        let state = RequestState::new(CancellationToken::new());
        let out = decide_outcome(&state, Err(AppError::Cancelled), 100);
        assert!(matches!(out, TerminalEvent::Cancelled));
        assert!(*state.terminal.lock().unwrap());
    }

    /// 비-cancel 에러는 claimed 가 false 면 Error 를 반환하고 claimed 마킹한다.
    #[test]
    fn decide_outcome_yields_error_when_not_claimed() {
        let state = RequestState::new(CancellationToken::new());
        let out = decide_outcome(
            &state,
            Err(AppError::Internal {
                message: "boom".to_string(),
            }),
            100,
        );
        assert!(matches!(out, TerminalEvent::Error(_)));
        assert!(*state.terminal.lock().unwrap());
    }

    #[test]
    fn started_payload_includes_resolved_language() {
        let payload = StartedPayload {
            request_id: "abc".to_string(),
            model: "test-model".to_string(),
            started_at_ms: 0,
            resolved_language: SourceLanguage::ChineseSimplified,
        };
        let json = serde_json::to_string(&payload).expect("serializable");
        assert!(
            json.contains(r#""resolvedLanguage":"ChineseSimplified""#),
            "json: {json}"
        );
    }
}
