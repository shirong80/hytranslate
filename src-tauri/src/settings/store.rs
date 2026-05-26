//! Settings 메모리 캐시 + JSON 파일 영속화.
//!
//! - 앱 시작 시 `load(path)` 로 파일을 읽어 메모리에 캐시.
//! - 파일이 없으면 기본값으로 채우고 디스크에 한 번 flush 한다.
//! - `update` 호출 시 메모리 + 디스크 동시 갱신.
//! - 동시 접근은 `RwLock` 으로 보호. 쓰기는 드물고 읽기는 잦다.

use std::fs;
use std::io::{ErrorKind, Write as _};
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use crate::errors::{AppError, AppResult};

use super::Settings;

pub struct SettingsStore {
    state: RwLock<Settings>,
    file_path: PathBuf,
}

impl SettingsStore {
    /// `file_path` 에서 로드한다. 부재/파싱 실패 시 기본값으로 fall back 하고
    /// 정상 파일이 없는 경우 한 번 flush 한다. 파싱 실패만 발생한 경우에는
    /// 깨진 파일을 덮어쓰지 않고 메모리상으로만 기본값 사용 (사용자가 직접 복구 가능).
    pub fn load(file_path: impl Into<PathBuf>) -> AppResult<Self> {
        let file_path = file_path.into();
        let settings = match fs::read_to_string(&file_path) {
            Ok(raw) => match serde_json::from_str::<Settings>(&raw) {
                Ok(s) => s,
                Err(e) => {
                    tracing::warn!(
                        error = %e,
                        "settings.json parse failed; falling back to defaults without overwriting"
                    );
                    Settings::default()
                }
            },
            Err(err) if err.kind() == ErrorKind::NotFound => {
                let s = Settings::default();
                save_to_disk(&file_path, &s)?;
                s
            }
            Err(err) => {
                return Err(AppError::internal(format!("settings load failed: {err}")));
            }
        };
        Ok(Self {
            state: RwLock::new(settings),
            file_path,
        })
    }

    pub fn get(&self) -> Settings {
        self.state.read().expect("settings lock poisoned").clone()
    }

    /// 메모리 갱신 후 디스크에 flush. 디스크 실패 시 메모리 값은 롤백한다.
    pub fn update(&self, new_settings: Settings) -> AppResult<Settings> {
        let previous = self.get();
        {
            let mut guard = self.state.write().expect("settings lock poisoned");
            *guard = new_settings.clone();
        }
        if let Err(e) = save_to_disk(&self.file_path, &new_settings) {
            // 디스크 실패 시 메모리 상태를 직전 값으로 복구하여 inconsistent 상태 회피.
            let mut guard = self.state.write().expect("settings lock poisoned");
            *guard = previous;
            return Err(e);
        }
        Ok(new_settings)
    }
}

fn save_to_disk(path: &Path, settings: &Settings) -> AppResult<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| AppError::internal(format!("settings dir create: {e}")))?;
    }
    let json = serde_json::to_string_pretty(settings)
        .map_err(|e| AppError::internal(format!("settings serialize: {e}")))?;
    let tmp_path = path.with_extension("json.tmp");
    {
        let mut f = fs::File::create(&tmp_path)
            .map_err(|e| AppError::internal(format!("settings tmp create: {e}")))?;
        f.write_all(json.as_bytes())
            .map_err(|e| AppError::internal(format!("settings write: {e}")))?;
        f.sync_all()
            .map_err(|e| AppError::internal(format!("settings fsync: {e}")))?;
    }
    fs::rename(&tmp_path, path).map_err(|e| AppError::internal(format!("settings rename: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::settings::Theme;

    fn tmp_path(name: &str) -> PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "hytranslate-test-{}-{name}.json",
            uuid::Uuid::new_v4()
        ));
        p
    }

    #[test]
    fn load_creates_default_file_when_missing() {
        let path = tmp_path("missing");
        let store = SettingsStore::load(&path).unwrap();
        let s = store.get();
        assert_eq!(s, Settings::default());
        assert!(path.exists());
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn update_persists_to_disk_and_round_trips() {
        let path = tmp_path("round-trip");
        let store = SettingsStore::load(&path).unwrap();
        let mut new = store.get();
        new.theme = Theme::Dark;
        new.active_model = "custom-model".to_string();
        new.ollama_endpoint = "http://127.0.0.1:12345".to_string();
        store.update(new.clone()).unwrap();

        // 새 store 로 reload 해서 디스크 값을 검증.
        let reloaded = SettingsStore::load(&path).unwrap();
        assert_eq!(reloaded.get(), new);
        std::fs::remove_file(&path).ok();
    }

    #[test]
    fn corrupt_json_falls_back_to_defaults_without_overwriting() {
        let path = tmp_path("corrupt");
        fs::write(&path, "{not json}").unwrap();
        let store = SettingsStore::load(&path).unwrap();
        assert_eq!(store.get(), Settings::default());
        // 깨진 파일이 그대로 남아 있어야 한다 (사용자가 복구할 기회).
        assert_eq!(fs::read_to_string(&path).unwrap(), "{not json}");
        std::fs::remove_file(&path).ok();
    }
}
