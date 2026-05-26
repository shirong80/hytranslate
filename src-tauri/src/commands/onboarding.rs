//! Phase 5: 온보딩 + 모델 lifecycle (PRD §6.1, §8.4, §10.4, §10.5).
//!
//! 노출 command:
//! - `detect_environment` — macOS 버전 / arch / 메모리 / 추천 모델.
//! - `get_ollama_status` — 설치/실행 여부 + 설치 모델 조회.
//! - `try_start_ollama` — 설치돼 있고 실행되지 않은 경우 자동 실행 시도 (PRD §8.4).
//! - `pull_model` — 사용자 승인 후 model pull 시작 (streaming).
//! - `cancel_model_pull` — 진행 중 pull 취소.
//! - `complete_onboarding` — 선택 모델을 `active_model` 로 영속화 + flag set.

use std::path::Path;
use std::process::Command;
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
use crate::ollama::{ChunkFlow, OllamaClient, PullChunk, MODEL_HY_MT2_1_8B, MODEL_HY_MT2_7B};
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

    /// 같은 모델 pull 이 이미 등록돼 있으면 true.
    pub fn contains(&self, model: &str) -> bool {
        self.tokens.contains_key(model)
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
    /// Ollama 가 디스크에 설치되어 있는지 (실행과 별개).
    pub installed: bool,
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

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CompleteOnboardingRequest {
    /// 사용자가 onboarding 의 model step 에서 선택한 활성 모델.
    /// 허용되는 값: `MODEL_HY_MT2_7B` 또는 `MODEL_HY_MT2_1_8B` (PRD §8.4).
    pub active_model: String,
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

/// 코드리뷰 Med 2 — backend trust boundary. FE radio 만 UI 제약. 백엔드도 지원 모델만 허용.
fn is_supported_model(model: &str) -> bool {
    model == MODEL_HY_MT2_7B || model == MODEL_HY_MT2_1_8B
}

/// 코드리뷰 Med 1 — Ollama 가 디스크에 설치돼 있는지 감지. 두 군데 중 하나라도 있으면 설치된 것으로 본다.
/// 1) `/Applications/Ollama.app` — GUI 설치 (PKG 공식 배포본 기본 위치).
/// 2) `which ollama` 가 0 으로 종료 — CLI / brew 설치본.
#[cfg(target_os = "macos")]
fn detect_ollama_installed() -> bool {
    if Path::new("/Applications/Ollama.app").exists() {
        return true;
    }
    Command::new("which")
        .arg("ollama")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

#[cfg(not(target_os = "macos"))]
fn detect_ollama_installed() -> bool {
    Command::new("which")
        .arg("ollama")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
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
    let installed = detect_ollama_installed();
    match client.list_models(&endpoint).await {
        Ok(models) => Ok(OllamaStatus {
            installed: true,
            running: true,
            endpoint,
            models,
        }),
        Err(AppError::OllamaNotRunning) => Ok(OllamaStatus {
            installed,
            running: false,
            endpoint,
            models: Vec::new(),
        }),
        Err(err) => Err(err),
    }
}

/// PRD §8.4 — Ollama 가 실행되지 않은 경우 자동 실행을 시도한다. 실패 시 사용자에게 직접
/// 실행 안내를 표시한다 (FE 가 status 재조회 결과로 판단).
///
/// macOS 에서는 `open -a Ollama` 로 GUI 앱을 띄운다. Ollama.app 이 실행되면 백그라운드
/// 데몬도 함께 기동된다. `which ollama` 만 있는 경우 (`ollama serve` CLI) 는 v1 범위 외.
#[tauri::command]
pub async fn try_start_ollama() -> AppResult<()> {
    if !detect_ollama_installed() {
        return Err(AppError::OllamaUnavailable);
    }
    #[cfg(target_os = "macos")]
    {
        Command::new("open")
            .args(["-a", "Ollama"])
            .spawn()
            .map_err(AppError::internal)?;
    }
    Ok(())
}

#[tauri::command]
pub async fn pull_model<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    registry: tauri::State<'_, Arc<PullRegistry>>,
    client: tauri::State<'_, OllamaClient>,
    settings: tauri::State<'_, Arc<SettingsStore>>,
    request: PullModelRequest,
) -> AppResult<()> {
    if !is_supported_model(&request.model) {
        return Err(AppError::internal(format!(
            "pull_model: unsupported model '{}'",
            request.model
        )));
    }
    // 같은 모델 pull 이 이미 진행 중이면 새로 spawn 하지 않는다. token 을 교체하면
    // 기존 worker 는 cancel 없이 살아남아 progress 이벤트를 emit 한 뒤 success 시 두 번
    // completed 가 발생할 수 있다 — 사용자의 명시 cancel + 재시작 흐름이 아니면 거부.
    if registry.contains(&request.model) {
        return Err(AppError::internal(format!(
            "pull_model: already in progress for '{}'",
            request.model
        )));
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

/// 코드리뷰 High 1 — onboarding 종료 시 사용자가 선택/다운로드한 모델을 `active_model`
/// 로 함께 영속화한다. flag 만 켜 두면 1.8B 다운로드 / 8 GB Mac 사용자가 첫 번역에서
/// `ModelMissing` 으로 실패한다.
#[tauri::command]
pub async fn complete_onboarding(
    store: tauri::State<'_, Arc<SettingsStore>>,
    request: CompleteOnboardingRequest,
) -> AppResult<()> {
    if !is_supported_model(&request.active_model) {
        return Err(AppError::internal(format!(
            "complete_onboarding: unsupported model '{}'",
            request.active_model
        )));
    }
    let mut current = store.get();
    if current.onboarding_completed && current.active_model == request.active_model {
        return Ok(());
    }
    current.onboarding_completed = true;
    current.active_model = request.active_model;
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

    #[test]
    fn pull_registry_detects_duplicate() {
        let r = PullRegistry::default();
        let t = CancellationToken::new();
        r.insert("m".to_string(), t);
        assert!(r.contains("m"));
        assert!(!r.contains("other"));
    }

    #[test]
    fn is_supported_model_accepts_only_hy_mt2() {
        assert!(is_supported_model(MODEL_HY_MT2_7B));
        assert!(is_supported_model(MODEL_HY_MT2_1_8B));
        assert!(!is_supported_model(""));
        assert!(!is_supported_model("hf.co/some/random:tag"));
        assert!(!is_supported_model("llama3:8b"));
        // PRD 모델 이름의 부분 매칭도 거부.
        assert!(!is_supported_model("Hy-MT2-7B"));
    }
}
