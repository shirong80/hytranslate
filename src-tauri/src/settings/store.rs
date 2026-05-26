//! Settings 메모리 캐시 + JSON 파일 영속화.
//!
//! - 앱 시작 시 `load(path)` 로 파일을 읽어 메모리에 캐시.
//! - 파일이 없으면 기본값으로 채우고 디스크에 한 번 flush 한다.
//! - `update` 호출 시 디스크 → 메모리 순으로 갱신하며, 전체 critical section 을
//!   write lock 안에서 직렬화한다. 두 update 가 같은 tmp 경로를 공유하거나
//!   메모리/디스크가 어긋날 수 있는 race window 를 만들지 않는다.
//! - tmp 파일에는 operation-unique suffix 를 붙여 외부 프로세스가 동일 경로를
//!   동시에 만지는 경우에도 충돌하지 않도록 한다 (defense-in-depth).

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

    /// 디스크 → 메모리 순으로 직렬 갱신한다. write lock 을 disk save 전 구간에
    /// 걸쳐 보유해 두 update 가 동시에 같은 tmp 파일을 만지거나, 메모리만 갱신된
    /// 채로 disk 가 실패하는 상태를 만들지 않는다. disk 가 실패하면 메모리 값은
    /// 그대로 유지된다 (별도 snapshot rollback 불필요).
    pub fn update(&self, new_settings: Settings) -> AppResult<Settings> {
        let mut guard = self.state.write().expect("settings lock poisoned");
        save_to_disk(&self.file_path, &new_settings)?;
        *guard = new_settings.clone();
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
    // operation-unique tmp suffix — write lock 이 이미 직렬화하지만, 같은
    // app data dir 을 만지는 외부 도구/디버거가 동일 tmp 경로를 쓰는 사고에도
    // 충돌하지 않도록 한다.
    let tmp_path = path.with_extension(format!("json.tmp.{}", uuid::Uuid::new_v4()));
    let write_result = (|| -> AppResult<()> {
        let mut f = fs::File::create(&tmp_path)
            .map_err(|e| AppError::internal(format!("settings tmp create: {e}")))?;
        f.write_all(json.as_bytes())
            .map_err(|e| AppError::internal(format!("settings write: {e}")))?;
        f.sync_all()
            .map_err(|e| AppError::internal(format!("settings fsync: {e}")))?;
        fs::rename(&tmp_path, path).map_err(|e| AppError::internal(format!("settings rename: {e}")))
    })();
    if write_result.is_err() {
        // rename 이전 단계에서 실패한 경우 tmp 파일이 남아 있을 수 있다 — best-effort cleanup.
        let _ = fs::remove_file(&tmp_path);
    }
    write_result
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

    #[test]
    fn concurrent_updates_keep_memory_and_disk_in_sync() {
        // 코드리뷰 High 1 회귀 — 두 thread 가 동시에 update 한 뒤
        // 1) store.get() 이 마지막으로 적용된 값을 반환하고
        // 2) 디스크 파일 내용이 그 값과 정확히 일치하며
        // 3) tmp 잔여 파일이 남지 않아야 한다.
        use std::sync::Arc;
        use std::thread;

        let path = tmp_path("concurrent");
        let store = Arc::new(SettingsStore::load(&path).unwrap());

        let a = Settings {
            active_model: "model-A".to_string(),
            ollama_endpoint: "http://127.0.0.1:11111".to_string(),
            ..Settings::default()
        };
        let b = Settings {
            active_model: "model-B".to_string(),
            ollama_endpoint: "http://127.0.0.1:22222".to_string(),
            ..Settings::default()
        };

        // N 라운드를 돌려 race window 가 있다면 노출되도록 한다.
        for _ in 0..50 {
            let store_a = store.clone();
            let store_b = store.clone();
            let a_val = a.clone();
            let b_val = b.clone();
            let h1 = thread::spawn(move || store_a.update(a_val).unwrap());
            let h2 = thread::spawn(move || store_b.update(b_val).unwrap());
            h1.join().unwrap();
            h2.join().unwrap();

            // 메모리 값은 A 또는 B 중 하나여야 한다.
            let mem = store.get();
            assert!(mem == a || mem == b, "unexpected merged state: {mem:?}");

            // 디스크 값과 메모리 값이 정확히 일치해야 한다 (lost update 없음).
            let raw = fs::read_to_string(&path).expect("file exists");
            let on_disk: Settings = serde_json::from_str(&raw).expect("valid json");
            assert_eq!(mem, on_disk, "memory/disk mismatch");
        }

        // tmp 파일이 남아 있지 않은지 확인 (cleanup 또는 rename 성공).
        let dir = path.parent().unwrap();
        let stem = path.file_stem().unwrap().to_string_lossy().into_owned();
        for entry in fs::read_dir(dir).unwrap() {
            let entry = entry.unwrap();
            let name = entry.file_name().to_string_lossy().into_owned();
            assert!(
                !name.starts_with(&stem) || !name.contains(".json.tmp."),
                "leftover tmp file: {name}"
            );
        }

        std::fs::remove_file(&path).ok();
    }
}
