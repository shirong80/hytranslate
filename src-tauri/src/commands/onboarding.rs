//! Phase 5: 온보딩 + 모델 lifecycle (PRD §6.1, §8.4, §10.4, §10.5).
//!
//! 노출 command:
//! - `detect_environment` — macOS 버전 / arch / 메모리 / 추천 모델.
//! - `get_ollama_status` — `/api/tags` 호출로 실행 여부 + 설치 모델 조회.
//! - `pull_model` — 사용자 승인 후 model pull 시작 (streaming).
//! - `cancel_model_pull` — 진행 중 pull 취소.
//! - `complete_onboarding` — `Settings.onboarding_completed = true` flush.

use std::sync::Arc;

use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use tauri::Emitter;
use tokio_util::sync::CancellationToken;

use crate::environment::{self, EnvironmentReport};
use crate::errors::{AppError, AppResult};
use crate::events::{
    MODEL_PULL_COMPLETED, MODEL_PULL_ERROR, MODEL_PULL_PROGRESS, MODEL_PULL_STARTED,
};
use crate::ollama::{ChunkFlow, OllamaClient, PullChunk};
use crate::settings::SettingsStore;

/// 진행 중인 model pull token. request 별로 등록되며 `cancel_model_pull` 로 cancel.
#[derive(Default)]
pub struct PullRegistry {
    tokens: DashMap<String, CancellationToken>,
}

impl PullRegistry {
    pub fn insert(&self, model: String, token: CancellationToken) {
        self.tokens.insert(model, token);
    }

    pub fn remove(&self, model: &str) {
        self.tokens.remove(model);
    }

    pub fn cancel(&self, model: &str) -> bool {
        if let Some((_, token)) = self.tokens.remove(model) {
            token.cancel();
            true
        } else {
            false
        }
    }

    #[cfg(test)]
    pub fn len(&self) -> usize {
        self.tokens.len()
    }

    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.tokens.is_empty()
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OllamaStatus {
    /// `/api/tags` 가 200 으로 응답하면 실행 중으로 본다.
    pub running: bool,
    pub endpoint: String,
    /// 로컬에 설치된 모델 ollama_name 목록. running == false 면 빈 vec.
    pub models: Vec<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PullModelRequest {
    pub model: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CancelPullRequest {
    pub model: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PullStartedPayload {
    model: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PullProgressPayload {
    model: String,
    status: String,
    digest: Option<String>,
    total: Option<u64>,
    completed: Option<u64>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PullCompletedPayload {
    model: String,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
struct PullErrorPayload {
    model: String,
    error: AppError,
}

#[tauri::command]
pub async fn detect_environment() -> AppResult<EnvironmentReport> {
    environment::detect()
}

#[tauri::command]
pub async fn get_ollama_status(
    client: tauri::State<'_, OllamaClient>,
    settings: tauri::State<'_, Arc<SettingsStore>>,
) -> AppResult<OllamaStatus> {
    let endpoint = settings.get().ollama_endpoint;
    match client.list_models(&endpoint).await {
        Ok(models) => Ok(OllamaStatus {
            running: true,
            endpoint,
            models,
        }),
        Err(AppError::OllamaNotRunning) => Ok(OllamaStatus {
            running: false,
            endpoint,
            models: Vec::new(),
        }),
        Err(err) => Err(err),
    }
}

#[tauri::command]
pub async fn pull_model<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    registry: tauri::State<'_, Arc<PullRegistry>>,
    client: tauri::State<'_, OllamaClient>,
    settings: tauri::State<'_, Arc<SettingsStore>>,
    request: PullModelRequest,
) -> AppResult<()> {
    if request.model.trim().is_empty() {
        return Err(AppError::internal("pull_model: model is empty"));
    }
    let endpoint = settings.get().ollama_endpoint;
    let token = CancellationToken::new();
    registry.insert(request.model.clone(), token.clone());

    let app_handle = app.clone();
    let client = (*client).clone();
    let registry_inner = registry.inner().clone();
    let model = request.model.clone();

    tokio::spawn(async move {
        run_pull(app_handle, client, registry_inner, endpoint, model, token).await;
    });

    Ok(())
}

#[tauri::command]
pub async fn cancel_model_pull(
    registry: tauri::State<'_, Arc<PullRegistry>>,
    request: CancelPullRequest,
) -> AppResult<()> {
    registry.cancel(&request.model);
    Ok(())
}

#[tauri::command]
pub async fn complete_onboarding(store: tauri::State<'_, Arc<SettingsStore>>) -> AppResult<()> {
    let mut current = store.get();
    if current.onboarding_completed {
        return Ok(());
    }
    current.onboarding_completed = true;
    store.update(current)?;
    Ok(())
}

async fn run_pull<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    client: OllamaClient,
    registry: Arc<PullRegistry>,
    endpoint: String,
    model: String,
    token: CancellationToken,
) {
    let _ = app.emit(
        MODEL_PULL_STARTED,
        PullStartedPayload {
            model: model.clone(),
        },
    );
    tracing::info!(model = %model, "model-pull:started");

    let emit_app = app.clone();
    let emit_model = model.clone();
    let emit_token = token.clone();

    let result = client
        .pull_model_stream(&endpoint, &model, &token, move |chunk: &PullChunk| {
            if emit_token.is_cancelled() {
                return ChunkFlow::Stop;
            }
            let _ = emit_app.emit(
                MODEL_PULL_PROGRESS,
                PullProgressPayload {
                    model: emit_model.clone(),
                    status: chunk.status.clone(),
                    digest: chunk.digest.clone(),
                    total: chunk.total,
                    completed: chunk.completed,
                },
            );
            ChunkFlow::Continue
        })
        .await;

    registry.remove(&model);

    match result {
        Ok(()) => {
            let _ = app.emit(
                MODEL_PULL_COMPLETED,
                PullCompletedPayload {
                    model: model.clone(),
                },
            );
            tracing::info!(model = %model, "model-pull:completed");
        }
        Err(AppError::Cancelled) => {
            tracing::info!(model = %model, "model-pull:cancelled");
            // 취소는 별도 이벤트 없이 stream 정지로 처리한다. FE 는 사용자 액션 시점에
            // 이미 store 상태를 비웠으므로 추가 이벤트가 필요 없다.
        }
        Err(err) => {
            tracing::warn!(model = %model, error.kind = ?err, "model-pull:error");
            let _ = app.emit(
                MODEL_PULL_ERROR,
                PullErrorPayload {
                    model: model.clone(),
                    error: err,
                },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pull_registry_cancels_token() {
        let r = PullRegistry::default();
        let t = CancellationToken::new();
        r.insert("m".to_string(), t.clone());
        assert_eq!(r.len(), 1);
        assert!(r.cancel("m"));
        assert!(t.is_cancelled());
        assert_eq!(r.len(), 0);
    }

    #[test]
    fn pull_registry_cancel_missing_returns_false() {
        let r = PullRegistry::default();
        assert!(!r.cancel("missing"));
    }
}
