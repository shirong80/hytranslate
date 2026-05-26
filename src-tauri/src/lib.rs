pub mod commands;
pub mod db;
pub mod environment;
pub mod errors;
pub mod events;
pub mod history;
pub mod language;
pub mod menubar;
pub mod ollama;
pub mod settings;
pub mod shortcuts;

pub fn run() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| tracing_subscriber::EnvFilter::new("info")),
        )
        .init();

    let builder = tauri::Builder::default();

    commands::register(builder)
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[cfg(test)]
mod tests {
    #[test]
    fn skeleton_compiles() {
        assert_eq!(2 + 2, 4);
    }
}
