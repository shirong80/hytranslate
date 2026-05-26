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
use tauri::{Emitter, Manager};
use tokio_util::sync::CancellationToken;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::events::{
    TRANSLATION_CANCELLED, TRANSLATION_CHUNK, TRANSLATION_COMPLETED, TRANSLATION_ERROR,
    TRANSLATION_STARTED,
};
use crate::history::{self, HistoryRepo, InsertRecord};
use crate::language::{self, SourceLanguage};
use crate::ollama::{ChunkFlow, OllamaClient};
use crate::settings::SettingsStore;

pub const MAIN_INPUT_LIMIT: usize = 30_000;

/// 요청별 cancel token + terminal-state guard.
///
/// code-review v1 follow-up review §10 (Critical 1 v3) — cancel 과 persist 의 terminal
/// 결정을 같은 mutex critical section 으로 묶어 race 를 닫는다. 한쪽이 lock 을 잡고
/// `claimed = true` 로 마킹하면 다른 쪽은 noop. 보장:
///
/// - cancel 이 먼저 잡으면: token.cancel() + claimed=true. 이후 worker 의 persist 분기는
///   lock 잡고 claimed==true 를 보고 INSERT 자체를 건너뜀 → cancelled emit.
/// - worker 가 먼저 잡으면: claimed=true 후 INSERT+commit. 그 뒤 도착한 cancel 은
///   claimed==true 를 보고 false 반환 (no-op). DB 에 이미 commit 된 row 는 stale 아님 —
///   사용자 입력 시점엔 아직 commit 전이었으므로 이 ordering 은 정상 종료.
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

    let snapshot = settings.get();
    let endpoint = snapshot.ollama_endpoint;
    let save_history = snapshot.save_history;

    let token = CancellationToken::new();
    registry.insert(request.request_id.clone(), token.clone());

    let app_handle = app.clone();
    let client = (*client).clone();
    let registry_inner = registry.inner().clone();
    let history_repo: Option<Arc<HistoryRepo>> = app
        .try_state::<Arc<HistoryRepo>>()
        .map(|s: tauri::State<'_, Arc<HistoryRepo>>| s.inner().clone());
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
            history_repo,
            save_history,
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

#[allow(clippy::too_many_arguments)]
async fn run_translation<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    client: OllamaClient,
    registry: Arc<TranslationRegistry>,
    endpoint: String,
    request: TranslateRequest,
    token: CancellationToken,
    history_repo: Option<Arc<HistoryRepo>>,
    save_history: bool,
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

    // Critical 1 v3 (code-review v1 follow-up review §10) — terminal-state guard.
    // cancel 과 persist 의 terminal 결정을 같은 mutex critical section 으로 묶는다.
    // lock 안에서 INSERT+commit 까지 수행하므로 commit 직전 race 가 닫힌다.
    let state = registry
        .get(&request_id)
        .expect("registry has the state we just inserted");

    let outcome = decide_outcome(
        &state,
        result,
        history_repo.as_deref(),
        save_history,
        &request_id,
        &request,
        started.elapsed().as_millis(),
    );
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

/// terminal mutex 를 잡고 outcome 을 결정한다. lock 안에서 INSERT+commit 까지 마쳐
/// cancel 과의 race 가 결정적으로 닫힌다.
fn decide_outcome(
    state: &RequestState,
    result: AppResult<String>,
    repo: Option<&HistoryRepo>,
    save_history: bool,
    request_id: &str,
    request: &TranslateRequest,
    duration_ms: u128,
) -> TerminalEvent {
    let mut claimed = state
        .terminal
        .lock()
        .expect("translation terminal lock poisoned");

    match result {
        Ok(full_text) => {
            if *claimed {
                // cancel 이 race 에서 이겼다. INSERT 하지 않음.
                TerminalEvent::Cancelled
            } else {
                // worker 가 race 에서 이겼다 — 같은 lock 안에서 persist 수행.
                persist_completed(
                    repo,
                    save_history,
                    request_id,
                    request,
                    &full_text,
                    duration_ms,
                );
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

/// PRD §8.7 — completed 이후 (terminal mutex 를 쥔 상태에서) 호출. cancel/error 경로는
/// 호출하지 않는다. `save_history` 가 false 거나 repo 가 미초기화면 silent skip.
/// DB 오류는 history 저장만 실패한 것이므로 worker 분기에는 영향 없음 (completed 그대로).
fn persist_completed(
    repo: Option<&HistoryRepo>,
    save_history: bool,
    request_id: &str,
    request: &TranslateRequest,
    full_text: &str,
    duration_ms: u128,
) {
    if !save_history {
        return;
    }
    let Some(repo) = repo else {
        tracing::debug!(request_id, "history repo unavailable; skip insert");
        return;
    };
    let duration_clamped = i64::try_from(duration_ms).unwrap_or(i64::MAX);
    let insert = InsertRecord {
        id: request_id.to_string(),
        source_text: request.source_text.clone(),
        source_language: request.source_language,
        translated_text: full_text.to_string(),
        model: request.model.clone(),
        duration_ms: duration_clamped,
        created_at: history::now_iso8601(),
    };
    if let Err(e) = repo.insert(insert) {
        tracing::warn!(request_id, error = ?e, "history insert failed");
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

    fn fresh_repo() -> (tempfile::TempDir, Arc<HistoryRepo>) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hytranslate.sqlite");
        let pool = crate::db::open(&path).unwrap();
        (dir, HistoryRepo::new(pool))
    }

    fn fake_request() -> TranslateRequest {
        TranslateRequest {
            source_text: "안녕하세요".to_string(),
            source_language: SourceLanguage::Korean,
            model: "test-model".to_string(),
            request_id: Uuid::new_v4().to_string(),
        }
    }

    #[test]
    fn persist_completed_writes_when_save_history_on() {
        let (_d, repo) = fresh_repo();
        let request = fake_request();
        persist_completed(
            Some(&repo),
            true,
            &request.request_id,
            &request,
            "Hello",
            100,
        );
        let got = repo.get(&request.request_id).unwrap();
        assert!(got.is_some());
    }

    #[test]
    fn persist_completed_skips_when_save_history_off() {
        let (_d, repo) = fresh_repo();
        let request = fake_request();
        persist_completed(
            Some(&repo),
            false,
            &request.request_id,
            &request,
            "Hello",
            100,
        );
        let got = repo.get(&request.request_id).unwrap();
        assert!(got.is_none());
    }

    #[test]
    fn persist_completed_no_op_when_repo_missing() {
        let request = fake_request();
        // None repo 는 panic 없이 silent skip.
        persist_completed(None, true, &request.request_id, &request, "Hello", 100);
    }

    /// Critical 1 v3 회귀 — decide_outcome 이 cancel 이 먼저 claim 한 상황에서 stream
    /// 결과를 받아도 INSERT 를 건너뛰고 Cancelled 를 반환한다.
    #[test]
    fn decide_outcome_skips_persist_when_cancel_wins() {
        let (_d, repo) = fresh_repo();
        let request = fake_request();
        let state = RequestState::new(CancellationToken::new());
        // cancel 이 먼저 잡음 — claimed=true.
        {
            let mut guard = state.terminal.lock().unwrap();
            *guard = true;
        }
        let out = decide_outcome(
            &state,
            Ok("Hello".to_string()),
            Some(&repo),
            true,
            &request.request_id,
            &request,
            100,
        );
        assert!(matches!(out, TerminalEvent::Cancelled));
        assert!(
            repo.get(&request.request_id).unwrap().is_none(),
            "cancel 이 먼저 claim 한 경우 INSERT 는 일어나면 안 된다"
        );
    }

    /// Critical 1 v3 회귀 — worker 가 lock 을 먼저 잡으면 같은 critical section 안에서
    /// INSERT+commit 까지 마치고 claimed=true. 이후 cancel 은 noop, row 는 보존.
    #[test]
    fn decide_outcome_persists_and_claims_when_worker_wins() {
        let (_d, repo) = fresh_repo();
        let request = fake_request();
        let state = RequestState::new(CancellationToken::new());
        let out = decide_outcome(
            &state,
            Ok("Hello".to_string()),
            Some(&repo),
            true,
            &request.request_id,
            &request,
            100,
        );
        match out {
            TerminalEvent::Completed { full_text, .. } => assert_eq!(full_text, "Hello"),
            other => panic!("expected Completed, got {other:?}"),
        }
        // 같은 lock 안에서 INSERT 완료.
        assert!(repo.get(&request.request_id).unwrap().is_some());
        // worker 가 이미 claim 했으니 후속 cancel 은 noop.
        assert!(*state.terminal.lock().unwrap());
    }

    /// Critical 1 v3 회귀 — stream 자체가 Err(Cancelled) 로 끝났을 때는 항상 Cancelled.
    /// claimed 가 아직 false 라도 마킹 후 cancelled emit.
    #[test]
    fn decide_outcome_handles_inner_cancel_error() {
        let (_d, repo) = fresh_repo();
        let request = fake_request();
        let state = RequestState::new(CancellationToken::new());
        let out = decide_outcome(
            &state,
            Err(AppError::Cancelled),
            Some(&repo),
            true,
            &request.request_id,
            &request,
            100,
        );
        assert!(matches!(out, TerminalEvent::Cancelled));
        assert!(repo.get(&request.request_id).unwrap().is_none());
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
