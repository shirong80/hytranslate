//! `#[tauri::command]` 어댑터 레이어. Phase 1 부터 핸들러 추가.

use std::process::Command;

use tauri::{Builder, Runtime};

use crate::errors::{AppError, AppResult};

/// Ollama 공식 설치 페이지 URL. FE 로 노출되지 않는 백엔드 상수다.
const OLLAMA_DOWNLOAD_URL: &str = "https://ollama.com/download";

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
    builder.invoke_handler(tauri::generate_handler![open_ollama_download_page])
}
