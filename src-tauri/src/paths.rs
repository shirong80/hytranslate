//! PRD §9.4 — 사용자 데이터 디렉터리 결정 + legacy 위치에서의 안전 마이그레이션.
//!
//! 전략 (코드리뷰 Major 7 + Critical hard-stop 2차 반영):
//!
//! 1. 자동 단계 (`migrate_copy_verify`): startup 시 new 경로에 copy + verify. legacy 는
//!    **읽기 전용** 으로만 참조. 어떤 destructive 작업도 수행하지 않는다.
//! 2. 수동 단계 (`cleanup_legacy`): 사용자가 설정 UI 의 CTA 를 명시 클릭한 경우에만 호출.
//!    legacy 의 우리 파일을 `new_dir/legacy-backup-<unix-ts>/` 로 fs::rename. 빈 디렉터리면
//!    `remove_dir` 시도. 외부 파일이 남아 있으면 디렉터리는 보존.
//!
//! SQLite DB 복사는 `fs::copy` 가 아니라 rusqlite `backup::Backup` API 로 페이지 단위 backup —
//! WAL/SHM checkpoint 가 끝나지 않았어도 일관된 snapshot 을 보장. settings.json 은 직접
//! `<dst>.tmp-<uuid>` 임시 파일에 write → `sync_all` → `fs::rename` 로 atomic write.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use rusqlite::backup::Backup;
use rusqlite::{Connection, OpenFlags};
use serde::Serialize;
use tauri::{AppHandle, Manager, Runtime};
use uuid::Uuid;

use crate::errors::{AppError, AppResult};

const NEW_DIR_RELATIVE: &str = "Library/Application Support/HyTranslate Mac";
const APP_SUPPORT_PREFIX: &str = "Library/Application Support";
const APP_DIR_NAME: &str = "HyTranslate Mac";
const SETTINGS_FILENAME: &str = "settings.json";
const DB_FILENAME: &str = "hytranslate.sqlite";

/// 자동 단계의 산출물.
///
/// **불변식**: `migrate_copy_verify` 가 `Ok` 로 반환할 때 `new_dir` 은 항상 채워지며,
/// `legacy_dir` 은 새 경로와 동일하지 않은 한 채워진다. `resolve_data_dir` 는 이 값만 보고
/// active dir 를 고른다.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationOutcome {
    pub new_dir: PathBuf,
    pub legacy_dir: Option<PathBuf>,
    pub copied: Vec<PathBuf>,
    pub verified: bool,
    pub legacy_has_our_files: bool,
    pub legacy_cleanable: bool,
    pub verify_error: Option<String>,
}

/// FE 노출용 view — 절대 경로는 디스플레이만, 액션은 backend 가 outcome 기반으로 수행.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MigrationStatusView {
    pub legacy_cleanable: bool,
    pub legacy_dir: Option<String>,
    pub verified: bool,
}

/// code-review v1 follow-up §17 — variant 이름은 PascalCase 로 유지하고 (TS union 과 일치),
/// 구조체 variant 내부 필드만 camelCase 로 rename.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "kind", rename_all_fields = "camelCase")]
pub enum CleanupReport {
    Skipped,
    Completed { backup_dir: PathBuf, moved: usize },
}

/// PRD §9.4 — `~/Library/Application Support/HyTranslate Mac/`.
///
/// Tauri 의 path API 로 home 디렉터리를 해석한 뒤 Application Support 하위인지 검증한다.
/// `$HOME` 오염 시에도 의도치 않은 위치로 redirect 되지 않게 한다
/// (code-review v1 follow-up §19).
pub fn new_data_dir<R: Runtime>(app: &AppHandle<R>) -> AppResult<PathBuf> {
    let home = app
        .path()
        .home_dir()
        .map_err(|e| AppError::internal(format!("resolve home_dir: {e}")))?;
    let dir = home.join(NEW_DIR_RELATIVE);
    ensure_under_app_support(&dir, &home)?;
    fs::create_dir_all(&dir)
        .map_err(|e| AppError::internal(format!("create new data dir: {e}")))?;
    Ok(dir)
}

/// 의도된 경로가 home/Library/Application Support/HyTranslate Mac 와 정확히 일치하는지 검증.
fn ensure_under_app_support(candidate: &Path, home: &Path) -> AppResult<()> {
    let expected = home.join(APP_SUPPORT_PREFIX).join(APP_DIR_NAME);
    if candidate != expected {
        return Err(AppError::internal(format!(
            "data dir not under Application Support: candidate={} expected={}",
            candidate.display(),
            expected.display(),
        )));
    }
    Ok(())
}

/// Tauri 가 가리키는 기존 app_data_dir. 새 경로와 동일하면 None — legacy 가 없는 것.
pub fn legacy_data_dir<R: Runtime>(
    app: &AppHandle<R>,
    new_dir: &Path,
) -> AppResult<Option<PathBuf>> {
    let tauri_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| AppError::internal(format!("resolve app_data_dir: {e}")))?;
    if paths_equivalent(&tauri_dir, new_dir) {
        Ok(None)
    } else {
        Ok(Some(tauri_dir))
    }
}

fn paths_equivalent(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(x), Ok(y)) => x == y,
        _ => a == b,
    }
}

/// `migrate_copy_verify` 의 `Err` 는 path 해석/생성 자체가 실패한 fatal 케이스에 한정.
/// 그 외 (legacy 조회 실패, DB copy 실패, verify 실패) 는 모두 outcome 내부에 흡수.
pub fn migrate_copy_verify<R: Runtime>(app: &AppHandle<R>) -> AppResult<MigrationOutcome> {
    let new_dir = new_data_dir(app)?;
    let legacy_dir = match legacy_data_dir(app, &new_dir) {
        Ok(v) => v,
        Err(err) => {
            tracing::warn!(error = ?err, "legacy_data_dir resolve failed; treating as none");
            None
        }
    };

    let legacy_has_our_files = legacy_dir
        .as_deref()
        .map(legacy_has_our_files)
        .unwrap_or(false);

    if !legacy_has_our_files {
        return Ok(MigrationOutcome {
            new_dir,
            legacy_dir,
            copied: Vec::new(),
            verified: true,
            legacy_has_our_files: false,
            legacy_cleanable: false,
            verify_error: None,
        });
    }

    let legacy = legacy_dir
        .as_deref()
        .expect("legacy_has_our_files=true implies legacy_dir.is_some()");

    let mut copied: Vec<PathBuf> = Vec::new();
    let mut copy_error: Option<String> = None;

    // settings.json — 둘 다 있으면 new 가 우선 (사용자가 새 경로에서 이미 작업했을 수도).
    let legacy_settings = legacy.join(SETTINGS_FILENAME);
    let new_settings = new_dir.join(SETTINGS_FILENAME);
    if legacy_settings.exists() && !new_settings.exists() {
        if let Err(e) = copy_settings(&legacy_settings, &new_settings) {
            copy_error = Some(format!("settings copy: {e:?}"));
        } else {
            copied.push(new_settings.clone());
        }
    }

    let legacy_db = legacy.join(DB_FILENAME);
    let new_db = new_dir.join(DB_FILENAME);
    if legacy_db.exists() && !new_db.exists() && copy_error.is_none() {
        if let Err(e) = copy_database(&legacy_db, &new_db) {
            copy_error = Some(format!("db copy: {e:?}"));
        } else {
            copied.push(new_db.clone());
        }
    }

    let verify_error = if let Some(err) = copy_error.clone() {
        // copy 실패 시 verify 까지 갈 필요 없음. destination 청소.
        rollback_destination(&new_dir, &copied);
        Some(err)
    } else {
        match verify_destination(&new_dir) {
            Ok(()) => None,
            Err(e) => {
                rollback_destination(&new_dir, &copied);
                Some(format!("verify failed: {e:?}"))
            }
        }
    };

    let verified = verify_error.is_none();
    let copied_if_verified = if verified { copied } else { Vec::new() };

    Ok(MigrationOutcome {
        new_dir,
        legacy_dir,
        copied: copied_if_verified,
        verified,
        legacy_has_our_files,
        legacy_cleanable: verified && legacy_has_our_files,
        verify_error,
    })
}

fn legacy_has_our_files(dir: &Path) -> bool {
    dir.join(SETTINGS_FILENAME).exists() || dir.join(DB_FILENAME).exists()
}

/// 결정 트리는 불변식 덕에 명확하다:
///   1. verified → new_dir
///   2. !verified && legacy_dir + legacy_has_our_files → legacy_dir (안전 fallback)
///   3. 그 외 → new_dir
pub fn resolve_data_dir(outcome: &MigrationOutcome) -> PathBuf {
    if outcome.verified {
        return outcome.new_dir.clone();
    }
    if outcome.legacy_has_our_files {
        if let Some(legacy) = outcome.legacy_dir.clone() {
            return legacy;
        }
    }
    outcome.new_dir.clone()
}

pub fn migration_status_view(outcome: &MigrationOutcome) -> MigrationStatusView {
    MigrationStatusView {
        legacy_cleanable: outcome.legacy_cleanable,
        legacy_dir: outcome.legacy_dir.as_ref().map(|p| p.display().to_string()),
        verified: outcome.verified,
    }
}

/// 수동 단계. legacy_cleanable == false 면 즉시 Skipped 반환.
/// 우리 파일들을 `<new_dir>/legacy-backup-<unix-ts>/` 로 fs::rename.
/// 같은 파일시스템이면 원자적; 다른 파일시스템이면 copy + remove.
/// rename 후 legacy 디렉터리가 빈 디렉터리면 remove_dir 시도.
pub fn cleanup_legacy(outcome: &MigrationOutcome) -> AppResult<CleanupReport> {
    if !outcome.legacy_cleanable {
        return Ok(CleanupReport::Skipped);
    }
    let legacy = outcome
        .legacy_dir
        .as_deref()
        .ok_or_else(|| AppError::internal("legacy_cleanable=true but legacy_dir is None"))?;

    // code-review v1 follow-up §18 — backend 가 filesystem 상태를 재검증한다.
    // renderer compromise 또는 stale outcome 이 호출해도 legacy 가 실제로 우리 파일을
    // 가지고 있지 않으면 cleanup 을 거부한다.
    if !legacy_has_our_files(legacy) {
        return Ok(CleanupReport::Skipped);
    }

    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let backup_dir = outcome.new_dir.join(format!("legacy-backup-{ts}"));
    fs::create_dir_all(&backup_dir)
        .map_err(|e| AppError::internal(format!("create backup dir: {e}")))?;

    let mut moved = 0usize;
    for name in [SETTINGS_FILENAME, DB_FILENAME] {
        let from = legacy.join(name);
        if !from.exists() {
            continue;
        }
        let to = backup_dir.join(name);
        move_path(&from, &to).map_err(|e| AppError::internal(format!("move {name}: {e:?}")))?;
        moved += 1;
    }

    // WAL/SHM 도 옮긴다 — DB 와 짝.
    for suffix in ["-wal", "-shm"] {
        let from = legacy.join(format!("{DB_FILENAME}{suffix}"));
        if from.exists() {
            let to = backup_dir.join(format!("{DB_FILENAME}{suffix}"));
            let _ = move_path(&from, &to);
        }
    }

    // 빈 디렉터리면 정리. 외부 파일이 남아 있으면 그대로.
    if dir_is_empty(legacy).unwrap_or(false) {
        let _ = fs::remove_dir(legacy);
    }

    Ok(CleanupReport::Completed { backup_dir, moved })
}

fn dir_is_empty(dir: &Path) -> std::io::Result<bool> {
    let mut it = fs::read_dir(dir)?;
    Ok(it.next().is_none())
}

fn move_path(from: &Path, to: &Path) -> AppResult<()> {
    if let Some(parent) = to.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| AppError::internal(format!("create dst parent: {e}")))?;
    }
    if let Err(rename_err) = fs::rename(from, to) {
        // cross-fs fallback — copy + remove.
        fs::copy(from, to).map_err(|e| {
            AppError::internal(format!(
                "rename+copy fallback failed: rename={rename_err}, copy={e}"
            ))
        })?;
        fs::remove_file(from).map_err(|e| AppError::internal(format!("remove source: {e}")))?;
    }
    Ok(())
}

fn copy_settings(src: &Path, dst: &Path) -> AppResult<()> {
    let bytes = fs::read(src).map_err(|e| AppError::internal(format!("read src: {e}")))?;
    atomic_write_file(dst, &bytes)
}

fn copy_database(src: &Path, dst: &Path) -> AppResult<()> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent).map_err(|e| AppError::internal(format!("dst dir: {e}")))?;
    }
    let src_conn = Connection::open_with_flags(src, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|e| AppError::internal(format!("open src db: {e}")))?;
    let mut dst_conn =
        Connection::open(dst).map_err(|e| AppError::internal(format!("open dst db: {e}")))?;
    {
        let backup = Backup::new(&src_conn, &mut dst_conn)
            .map_err(|e| AppError::internal(format!("init backup: {e}")))?;
        backup
            .run_to_completion(64, std::time::Duration::ZERO, None)
            .map_err(|e| AppError::internal(format!("run backup: {e}")))?;
    }
    // 둘 다 명시적으로 닫아 WAL/SHM 잔여를 정리한다.
    drop(dst_conn);
    let _ = src_conn.close();
    Ok(())
}

/// 새 settings.json 이 deserialize 가능하고 새 DB 가 schema_version 을 가진다.
///
/// settings 는 `serde_json::Value` (JSON 문법) 가 아니라 실제 `Settings` 스키마로
/// deserialize 해야 한다 (code-review v1 follow-up §14 보강). legacy 가 우리 스키마와
/// 호환되지 않는 경우 verify=false 로 떨어져 안전 fallback 이 작동하도록.
fn verify_destination(dst_dir: &Path) -> AppResult<()> {
    let settings_path = dst_dir.join(SETTINGS_FILENAME);
    if settings_path.exists() {
        let bytes = fs::read(&settings_path)
            .map_err(|e| AppError::internal(format!("read settings: {e}")))?;
        let _: crate::settings::Settings = serde_json::from_slice(&bytes)
            .map_err(|e| AppError::internal(format!("settings schema parse: {e}")))?;
    }
    let db_path = dst_dir.join(DB_FILENAME);
    if db_path.exists() {
        let conn = Connection::open_with_flags(&db_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
            .map_err(|e| AppError::internal(format!("open verify db: {e}")))?;
        let version: i32 = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .map_err(|e| AppError::internal(format!("read user_version: {e}")))?;
        if version <= 0 {
            return Err(AppError::internal(format!(
                "verify: user_version={version}, expected > 0"
            )));
        }
        let _: i64 = conn
            .query_row("SELECT count(*) FROM translation_records", [], |r| r.get(0))
            .map_err(|e| AppError::internal(format!("count translation_records: {e}")))?;
    }
    Ok(())
}

/// verify 실패 시 destination 에 복사된 파일들을 best-effort 삭제. legacy 는 손대지 않음.
fn rollback_destination(_dir: &Path, copied: &[PathBuf]) {
    for p in copied {
        let _ = fs::remove_file(p);
        // SQLite 가 만들었을 수 있는 WAL/SHM.
        if let Some(name) = p.file_name().and_then(|n| n.to_str()) {
            if name == DB_FILENAME {
                let _ = fs::remove_file(p.with_file_name(format!("{DB_FILENAME}-wal")));
                let _ = fs::remove_file(p.with_file_name(format!("{DB_FILENAME}-shm")));
            }
        }
    }
}

/// std-기반 atomic write — `<dst>.tmp-<uuid>` → write_all → sync_all → fs::rename.
/// 같은 디렉터리이므로 POSIX rename 이 원자.
pub fn atomic_write_file(dst: &Path, bytes: &[u8]) -> AppResult<()> {
    if let Some(parent) = dst.parent() {
        fs::create_dir_all(parent)
            .map_err(|e| AppError::internal(format!("create dst parent: {e}")))?;
    }
    let suffix = Uuid::new_v4().simple().to_string();
    let tmp = dst.with_file_name(match dst.file_name() {
        Some(name) => format!("{}.tmp-{suffix}", name.to_string_lossy()),
        None => format!(".tmp-{suffix}"),
    });
    let write_result = (|| -> std::io::Result<()> {
        let mut f = fs::File::create(&tmp)?;
        f.write_all(bytes)?;
        f.sync_all()?;
        Ok(())
    })();
    if let Err(e) = write_result {
        let _ = fs::remove_file(&tmp);
        return Err(AppError::internal(format!("atomic write tmp: {e}")));
    }
    if let Err(e) = fs::rename(&tmp, dst) {
        let _ = fs::remove_file(&tmp);
        return Err(AppError::internal(format!("atomic rename: {e}")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn write_settings(dir: &Path, json: &str) {
        fs::create_dir_all(dir).unwrap();
        fs::write(dir.join(SETTINGS_FILENAME), json).unwrap();
    }

    fn fresh_db(path: &Path) {
        if let Some(p) = path.parent() {
            fs::create_dir_all(p).unwrap();
        }
        let _pool = crate::db::open(path).unwrap();
    }

    /// 실제 outcome 을 만들지 않고 helper 동작만 확인. tauri::App 을 띄우지 않는 단위 테스트.
    fn outcome(
        new_dir: PathBuf,
        legacy_dir: Option<PathBuf>,
        legacy_has_our_files: bool,
        verified: bool,
    ) -> MigrationOutcome {
        MigrationOutcome {
            new_dir,
            legacy_dir,
            copied: Vec::new(),
            verified,
            legacy_has_our_files,
            legacy_cleanable: verified && legacy_has_our_files,
            verify_error: None,
        }
    }

    #[test]
    fn atomic_write_creates_file_and_no_tmp_remains() {
        let dir = tempfile::tempdir().unwrap();
        let dst = dir.path().join("foo.json");
        atomic_write_file(&dst, br#"{"k":"v"}"#).unwrap();
        assert_eq!(fs::read(&dst).unwrap(), br#"{"k":"v"}"#);
        let leftovers: Vec<_> = fs::read_dir(dir.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp-"))
            .collect();
        assert!(leftovers.is_empty(), "tmp files remain: {leftovers:?}");
    }

    #[test]
    fn resolve_data_dir_prefers_new_when_verified() {
        let new_dir = PathBuf::from("/tmp/hytranslate-new");
        let legacy = PathBuf::from("/tmp/hytranslate-legacy");
        let o = outcome(new_dir.clone(), Some(legacy.clone()), true, true);
        assert_eq!(resolve_data_dir(&o), new_dir);
    }

    #[test]
    fn resolve_data_dir_falls_back_to_legacy_when_verify_failed() {
        let new_dir = PathBuf::from("/tmp/hytranslate-new");
        let legacy = PathBuf::from("/tmp/hytranslate-legacy");
        let o = outcome(new_dir, Some(legacy.clone()), true, false);
        assert_eq!(resolve_data_dir(&o), legacy);
    }

    #[test]
    fn resolve_data_dir_returns_new_when_no_legacy() {
        let new_dir = PathBuf::from("/tmp/hytranslate-new");
        let o = outcome(new_dir.clone(), None, false, true);
        assert_eq!(resolve_data_dir(&o), new_dir);
    }

    #[test]
    fn copy_settings_writes_atomically() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("legacy").join("settings.json");
        let dst = dir.path().join("new").join("settings.json");
        fs::create_dir_all(src.parent().unwrap()).unwrap();
        fs::write(&src, br#"{"theme":"System"}"#).unwrap();
        copy_settings(&src, &dst).unwrap();
        assert_eq!(fs::read(&dst).unwrap(), br#"{"theme":"System"}"#);
    }

    #[test]
    fn copy_database_uses_backup_api_and_is_readable() {
        let dir = tempfile::tempdir().unwrap();
        let src = dir.path().join("legacy").join(DB_FILENAME);
        let dst = dir.path().join("new").join(DB_FILENAME);
        fresh_db(&src);
        let pool = crate::db::open(&src).unwrap();
        let conn = pool.get().unwrap();
        conn.execute(
            "INSERT INTO translation_records (id, source_text, source_language, translated_text, model, duration_ms, created_at) \
             VALUES ('rec1','source','Korean','Hello','m',100,'2026-05-26T00:00:00Z')",
            [],
        )
        .unwrap();
        drop(conn);
        drop(pool);

        copy_database(&src, &dst).unwrap();
        verify_destination(dst.parent().unwrap()).unwrap();

        let pool2 = crate::db::open(&dst).unwrap();
        let conn2 = pool2.get().unwrap();
        let count: i64 = conn2
            .query_row("SELECT count(*) FROM translation_records", [], |r| r.get(0))
            .unwrap();
        assert_eq!(count, 1);
    }

    #[test]
    fn cleanup_skipped_when_not_cleanable() {
        let dir = tempfile::tempdir().unwrap();
        let o = outcome(dir.path().to_path_buf(), None, false, true);
        match cleanup_legacy(&o).unwrap() {
            CleanupReport::Skipped => {}
            other => panic!("expected Skipped, got {other:?}"),
        }
    }

    #[test]
    fn cleanup_moves_our_files_and_removes_empty_dir() {
        let dir = tempfile::tempdir().unwrap();
        let new_dir = dir.path().join("new");
        let legacy = dir.path().join("legacy");
        fs::create_dir_all(&new_dir).unwrap();
        write_settings(&legacy, "{}");
        fresh_db(&legacy.join(DB_FILENAME));

        let o = MigrationOutcome {
            new_dir: new_dir.clone(),
            legacy_dir: Some(legacy.clone()),
            copied: vec![],
            verified: true,
            legacy_has_our_files: true,
            legacy_cleanable: true,
            verify_error: None,
        };
        let report = cleanup_legacy(&o).unwrap();
        match report {
            CleanupReport::Completed { backup_dir, moved } => {
                assert!(moved >= 2, "expected at least 2 files moved, got {moved}");
                assert!(backup_dir.exists());
                assert!(backup_dir.join(SETTINGS_FILENAME).exists());
                assert!(backup_dir.join(DB_FILENAME).exists());
            }
            other => panic!("expected Completed, got {other:?}"),
        }
        assert!(!legacy.exists(), "empty legacy dir should be removed");
    }

    #[test]
    fn cleanup_preserves_legacy_with_foreign_file() {
        let dir = tempfile::tempdir().unwrap();
        let new_dir = dir.path().join("new");
        let legacy = dir.path().join("legacy");
        fs::create_dir_all(&new_dir).unwrap();
        write_settings(&legacy, "{}");
        fs::write(legacy.join("README.md"), b"keep me").unwrap();

        let o = MigrationOutcome {
            new_dir,
            legacy_dir: Some(legacy.clone()),
            copied: vec![],
            verified: true,
            legacy_has_our_files: true,
            legacy_cleanable: true,
            verify_error: None,
        };
        cleanup_legacy(&o).unwrap();
        assert!(
            legacy.exists(),
            "legacy dir with foreign file must be preserved"
        );
        assert!(legacy.join("README.md").exists());
    }

    #[test]
    fn verify_destination_passes_on_fresh_setup() {
        let dir = tempfile::tempdir().unwrap();
        let settings = serde_json::to_string(&crate::settings::Settings::default()).unwrap();
        fs::write(dir.path().join(SETTINGS_FILENAME), settings).unwrap();
        fresh_db(&dir.path().join(DB_FILENAME));
        verify_destination(dir.path()).unwrap();
    }

    #[test]
    fn verify_destination_rejects_corrupted_db() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(dir.path().join(DB_FILENAME), b"this is not sqlite").unwrap();
        assert!(verify_destination(dir.path()).is_err());
    }

    /// code-review v1 follow-up §14 회귀 — settings.json 이 JSON 문법은 맞지만
    /// `Settings` 스키마로 deserialize 안 되면 verify=false 가 되어야 한다.
    /// `theme` 가 enum 외 값이면 강제로 fail.
    #[test]
    fn verify_destination_rejects_settings_schema_mismatch() {
        let dir = tempfile::tempdir().unwrap();
        fs::write(
            dir.path().join(SETTINGS_FILENAME),
            br#"{"theme": "Neon", "globalHotkey": 42}"#,
        )
        .unwrap();
        assert!(verify_destination(dir.path()).is_err());
    }
}
