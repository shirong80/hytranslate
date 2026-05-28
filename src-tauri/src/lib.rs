pub mod commands;
pub mod db;
pub mod environment;
pub mod errors;
pub mod events;
pub mod history;
pub mod language;
pub mod menubar;
pub mod ollama;
pub mod paths;
pub mod settings;
pub mod shortcuts;

#[cfg(target_os = "macos")]
use tauri::Manager;

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let builder = tauri::Builder::default();

    let app = commands::register(builder)
        .build(tauri::generate_context!())
        .expect("error while building tauri application");

    app.run(|_app_handle, _event| {
        #[cfg(target_os = "macos")]
        if let tauri::RunEvent::Reopen {
            has_visible_windows: _,
            ..
        } = &_event
        {
            if let Some(window) = _app_handle.get_webview_window("main") {
                let _ = window.show();
                let _ = window.unminimize();
                let _ = window.set_focus();
                tracing::debug!(window = "main", "reopen requested");
            }
        }
    });
}

#[cfg(test)]
mod tests {
    #[test]
    fn skeleton_compiles() {
        assert_eq!(2 + 2, 4);
    }
}
