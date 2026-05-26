//! `#[tauri::command]` 어댑터 레이어.
//!
//! Phase 3 부터 추가: 글로벌 단축키, 트레이 popover, dock activation policy,
//! autostart 토글. 모두 settings 영속화 시점에 reconcile 호출로 묶인다.

use std::process::Command;
use std::sync::Arc;

use tauri::{Builder, Manager, Runtime};

use crate::db;
use crate::errors::{AppError, AppResult};
use crate::history::HistoryRepo;
use crate::ollama::OllamaClient;
use crate::settings::SettingsStore;
use crate::{menubar, shortcuts};

pub mod detect;
pub mod history;
pub mod onboarding;
pub mod popup;
pub mod settings;
pub mod system;
pub mod translate;

pub use onboarding::PullRegistry;
pub use translate::TranslationRegistry;

const OLLAMA_DOWNLOAD_URL: &str = "https://ollama.com/download";

/// macOS 손쉬운 사용 설정 패널 deep-link. URL 은 백엔드 상수로 고정 — FE 가 임의 URL 을 못 연다.
const ACCESSIBILITY_SETTINGS_URL: &str =
    "x-apple.systempreferences:com.apple.preference.security?Privacy_Accessibility";

#[tauri::command]
async fn open_ollama_download_page() -> AppResult<()> {
    Command::new("open")
        .arg(OLLAMA_DOWNLOAD_URL)
        .spawn()
        .map_err(AppError::internal)?;
    Ok(())
}

#[tauri::command]
async fn open_accessibility_settings() -> AppResult<()> {
    Command::new("open")
        .arg(ACCESSIBILITY_SETTINGS_URL)
        .spawn()
        .map_err(AppError::internal)?;
    Ok(())
}

pub fn register<R: Runtime>(builder: Builder<R>) -> Builder<R> {
    let registry = Arc::new(TranslationRegistry::default());
    let pull_registry = Arc::new(PullRegistry::default());

    builder
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_autostart::init(
            tauri_plugin_autostart::MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .plugin(tauri_plugin_dialog::init())
        .manage(registry)
        .manage(pull_registry)
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
            let initial = store.get();
            app.manage(Arc::new(store));

            let client = OllamaClient::new().map_err(|e| -> Box<dyn std::error::Error> {
                Box::new(std::io::Error::other(format!("ollama client init: {e:?}")))
            })?;
            app.manage(client);

            // SQLite 풀 + 이력 레포지토리. DB 가 망가졌더라도 앱 자체는 계속 떠 있어야
            // 하므로 실패는 로그만 남기고 setup 은 통과. 이력 관련 명령은 풀 부재 시
            // State 미발견으로 자동 실패 → FE 에 inline 에러로 노출.
            let db_path = data_dir.join("hytranslate.sqlite");
            match db::open(&db_path) {
                Ok(pool) => {
                    app.manage(HistoryRepo::new(pool));
                }
                Err(e) => {
                    tracing::warn!(error = ?e, "history db open failed; history disabled");
                }
            }

            // 글로벌 단축키 + 트레이는 설치 실패 시 앱 자체는 살아 있어야 한다.
            // 실패는 로그로 남기고 setup 은 통과.
            if let Err(e) = shortcuts::install(app, &initial.global_hotkey) {
                tracing::warn!(error = ?e, hotkey = %initial.global_hotkey, "global shortcut install failed");
            }
            if let Err(e) = menubar::install(app) {
                tracing::warn!(error = ?e, "menubar tray install failed");
            }
            if let Err(e) = system::apply_dock_hidden(app.handle(), initial.hide_dock_icon) {
                tracing::warn!(error = ?e, "dock activation policy apply failed");
            }
            if let Err(e) = system::apply_autostart(app.handle(), initial.start_at_login) {
                tracing::warn!(error = ?e, "autostart apply failed");
            }

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_ollama_download_page,
            open_accessibility_settings,
            translate::translate_stream,
            translate::cancel_translation,
            detect::detect_language,
            settings::get_settings,
            settings::update_settings,
            popup::show_popup,
            popup::hide_popup,
            popup::toggle_popup,
            history::list_translation_records,
            history::search_translation_records,
            history::get_translation_record,
            history::delete_translation_record,
            history::delete_all_translation_records,
            history::toggle_favorite,
            history::set_tags,
            history::export_history_csv,
            history::export_history_json,
            onboarding::detect_environment,
            onboarding::get_ollama_status,
            onboarding::try_start_ollama,
            onboarding::pull_model,
            onboarding::cancel_model_pull,
            onboarding::complete_onboarding,
        ])
}
