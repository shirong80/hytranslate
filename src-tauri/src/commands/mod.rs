//! `#[tauri::command]` 어댑터 레이어. Phase 1 부터 핸들러 추가.

use std::process::Command;
use std::sync::Arc;

use tauri::{Builder, Runtime};

use crate::errors::{AppError, AppResult};
use crate::ollama::OllamaClient;

pub mod translate;

pub use translate::TranslationRegistry;

/// Ollama 공식 설치 페이지 URL. FE 로 노출되지 않는 백엔드 상수다.
const OLLAMA_DOWNLOAD_URL: &str = "https://ollama.com/download";

/// 기본 Ollama endpoint. Phase 2 에서 Settings 도입 시 사용자가 override 한다.
pub const DEFAULT_OLLAMA_ENDPOINT: &str = "http://localhost:11434";

/// FE 에 `shell.open` 권한을 직접 부여하지 않고, "Ollama 설치 페이지 열기"
/// 라는 제한된 intent 만 호출하도록 백엔드에서 URL 을 고정해 열어준다.
#[tauri::command]
async fn open_ollama_download_page() -> AppResult<()> {
    Command::new("open")
        .arg(OLLAMA_DOWNLOAD_URL)
        .spawn()
        .map_err(AppError::internal)?;
    Ok(())
}

pub fn register<R: Runtime>(builder: Builder<R>) -> Builder<R> {
    let registry = Arc::new(TranslationRegistry::default());
    let client = OllamaClient::new(DEFAULT_OLLAMA_ENDPOINT)
        .expect("OllamaClient must build with default endpoint");

    builder
        .manage(registry)
        .manage(client)
        .invoke_handler(tauri::generate_handler![
            open_ollama_download_page,
            translate::translate_stream,
            translate::cancel_translation,
        ])
}
