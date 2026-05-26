//! `#[tauri::command]` 어댑터 레이어. Phase 2 부터 SettingsStore 도입.

use std::process::Command;
use std::sync::Arc;

use tauri::{Builder, Manager, Runtime};

use crate::errors::{AppError, AppResult};
use crate::ollama::OllamaClient;
use crate::settings::SettingsStore;

pub mod detect;
pub mod settings;
pub mod translate;

pub use translate::TranslationRegistry;

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
    let registry = Arc::new(TranslationRegistry::default());

    builder
        .manage(registry)
        .setup(|app| {
            // Settings 영속화 위치: app_data_dir/settings.json
            // macOS 에서는 ~/Library/Application Support/<bundle id>/settings.json 으로 매핑.
            let data_dir =
                app.path()
                    .app_data_dir()
                    .map_err(|e| -> Box<dyn std::error::Error> {
                        Box::new(std::io::Error::other(format!("resolve app_data_dir: {e}")))
                    })?;
            let settings_path = data_dir.join("settings.json");
            let store =
                SettingsStore::load(&settings_path).map_err(|e| -> Box<dyn std::error::Error> {
                    Box::new(std::io::Error::other(format!("settings init: {e:?}")))
                })?;
            app.manage(Arc::new(store));

            let client = OllamaClient::new().map_err(|e| -> Box<dyn std::error::Error> {
                Box::new(std::io::Error::other(format!("ollama client init: {e:?}")))
            })?;
            app.manage(client);
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_ollama_download_page,
            translate::translate_stream,
            translate::cancel_translation,
            detect::detect_language,
            settings::get_settings,
            settings::update_settings,
        ])
}
