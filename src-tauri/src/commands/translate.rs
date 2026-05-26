//! Phase 1: `translate_stream` / `cancel_translation` 명령.
//!
//! 핸들러는 30,000자 cap 검증 후 worker 를 spawn 한다. worker 는 chunk 마다
//! `translation:chunk` 이벤트를 emit 하고, 종료 시점에 `completed` / `cancelled` /
//! `error` 중 정확히 하나를 emit 한다.

use std::sync::Arc;
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

/// 진행 중인 번역 요청 토큰 맵. `register` 에서 만들어 `manage` 한다.
#[derive(Default)]
pub struct TranslationRegistry {
    tokens: DashMap<String, CancellationToken>,
}

impl TranslationRegistry {
    pub fn insert(&self, request_id: String, token: CancellationToken) {
        self.tokens.insert(request_id, token);
    }

    pub fn remove(&self, request_id: &str) {
        self.tokens.remove(request_id);
    }

    pub fn cancel(&self, request_id: &str) -> bool {
        if let Some((_, token)) = self.tokens.remove(request_id) {
            token.cancel();
            true
        } else {
            false
        }
    }

    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.tokens.len()
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

    registry.remove(&request_id);

    match result {
        Ok(full_text) => {
            if token.is_cancelled() {
                let _ = app.emit(
                    TRANSLATION_CANCELLED,
                    CancelledPayload {
                        request_id: request_id.clone(),
                    },
                );
                tracing::info!(request_id = %request_id, "translation:cancelled");
                return;
            }
            let duration_ms = started.elapsed().as_millis();
            // 이력 영속화 — completed 이후, save_history 가 켜진 경우에만 (PRD §8.7).
            // 실패해도 사용자 경로에는 영향을 주지 않는다 → warn 로그만.
            persist_completed(
                history_repo.as_deref(),
                save_history,
                &request_id,
                &request,
                &full_text,
                duration_ms,
            );
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
        Err(AppError::Cancelled) => {
            let _ = app.emit(
                TRANSLATION_CANCELLED,
                CancelledPayload {
                    request_id: request_id.clone(),
                },
            );
            tracing::info!(request_id = %request_id, "translation:cancelled");
        }
        Err(err) => {
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

/// PRD §8.7 — completed 이후에만 호출. cancel/error 경로는 호출하지 않는다.
/// `save_history` 가 false 거나 repo 가 미초기화면 silently skip.
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
        assert_eq!(reg.len(), 0);
    }

    #[test]
    fn registry_cancel_missing_returns_false() {
        let reg = TranslationRegistry::default();
        assert!(!reg.cancel("nope"));
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
        // panic 없이 그냥 통과해야 한다.
        persist_completed(None, true, &request.request_id, &request, "Hello", 100);
    }
}
