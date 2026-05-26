//! SQLite 풀 + 마이그레이션 러너.
//!
//! - `r2d2` + `r2d2_sqlite::SqliteConnectionManager` 단일 풀.
//! - 풀 build 직후 단일 connection 으로 forward-only 마이그레이션 실행.
//! - `PRAGMA user_version` 으로 적용 상태 추적.
//! - 각 connection 은 `with_init` 으로 WAL + foreign_keys + busy_timeout 을 켠다.

use std::path::Path;

use r2d2::PooledConnection;
use r2d2_sqlite::SqliteConnectionManager;
use rusqlite::OpenFlags;

use crate::errors::{AppError, AppResult};

pub type Pool = r2d2::Pool<SqliteConnectionManager>;
pub type Conn = PooledConnection<SqliteConnectionManager>;

/// 현재 schema 가 따라가야 할 마이그레이션 목록.
/// 각 entry 는 `(target_user_version, sql)` — apply 후 `user_version` 을 그 값으로 갱신한다.
/// 새 migration 추가 시 이 배열 끝에 push 만 한다. 절대 reorder/edit 하지 않는다.
const MIGRATIONS: &[(i32, &str)] = &[(1, include_str!("migrations/0001_init.sql"))];

/// 풀을 build 하고 마이그레이션을 끝까지 적용한 뒤 반환한다.
pub fn open(path: impl AsRef<Path>) -> AppResult<Pool> {
    if let Some(parent) = path.as_ref().parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| AppError::internal(format!("db dir create: {e}")))?;
    }

    let manager = SqliteConnectionManager::file(path.as_ref())
        .with_flags(OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE)
        .with_init(|c| {
            c.execute_batch(
                "PRAGMA journal_mode=WAL;\
                 PRAGMA foreign_keys=ON;\
                 PRAGMA synchronous=NORMAL;\
                 PRAGMA busy_timeout=5000;",
            )
        });

    let pool = r2d2::Pool::builder()
        .max_size(8)
        .build(manager)
        .map_err(|e| AppError::internal(format!("db pool build: {e}")))?;

    {
        let mut conn = pool
            .get()
            .map_err(|e| AppError::internal(format!("db conn acquire: {e}")))?;
        run_migrations(&mut conn)?;
    }

    Ok(pool)
}

/// 마이그레이션 러너 — 단일 connection 에서 PRAGMA user_version 미만의 entry 만 transaction 으로 적용한다.
fn run_migrations(conn: &mut rusqlite::Connection) -> AppResult<()> {
    let current: i32 = conn
        .pragma_query_value(None, "user_version", |r| r.get(0))
        .map_err(|e| AppError::internal(format!("read user_version: {e}")))?;

    for &(target, sql) in MIGRATIONS {
        if current >= target {
            continue;
        }
        let tx = conn
            .transaction()
            .map_err(|e| AppError::internal(format!("begin migration tx: {e}")))?;
        tx.execute_batch(sql)
            .map_err(|e| AppError::internal(format!("apply migration {target}: {e}")))?;
        tx.pragma_update(None, "user_version", target)
            .map_err(|e| AppError::internal(format!("bump user_version {target}: {e}")))?;
        tx.commit()
            .map_err(|e| AppError::internal(format!("commit migration {target}: {e}")))?;
        tracing::info!(target_version = target, "db migration applied");
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn open_creates_db_and_applies_migrations() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hytranslate.sqlite");
        let pool = open(&path).expect("open succeeds");
        let conn = pool.get().unwrap();

        let version: i32 = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(version, MIGRATIONS.last().unwrap().0);

        let names: Vec<String> = conn
            .prepare("SELECT name FROM sqlite_master WHERE type IN ('table','view') ORDER BY name")
            .unwrap()
            .query_map([], |r| r.get::<_, String>(0))
            .unwrap()
            .collect::<rusqlite::Result<Vec<_>>>()
            .unwrap();
        assert!(names.contains(&"translation_records".to_string()));
        assert!(names.contains(&"translation_records_fts".to_string()));
    }

    #[test]
    fn rerun_is_idempotent() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hytranslate.sqlite");
        let _ = open(&path).unwrap();
        let pool = open(&path).unwrap();
        let conn = pool.get().unwrap();
        let v: i32 = conn
            .pragma_query_value(None, "user_version", |r| r.get(0))
            .unwrap();
        assert_eq!(v, MIGRATIONS.last().unwrap().0);
    }

    #[test]
    fn fts_triggers_sync_on_insert_and_delete() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hytranslate.sqlite");
        let pool = open(&path).unwrap();
        let conn = pool.get().unwrap();

        conn.execute(
            "INSERT INTO translation_records (id, source_text, source_language, translated_text, model, duration_ms, created_at) \
             VALUES (?1,'안녕하세요','Korean','Hello','m',100,'2026-05-26T00:00:00Z')",
            [uuid::Uuid::new_v4().to_string()],
        )
        .unwrap();
        let hits: i64 = conn
            .query_row(
                "SELECT count(*) FROM translation_records_fts WHERE translation_records_fts MATCH 'hello'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(hits, 1);

        conn.execute("DELETE FROM translation_records", []).unwrap();
        let hits_after: i64 = conn
            .query_row(
                "SELECT count(*) FROM translation_records_fts WHERE translation_records_fts MATCH 'hello'",
                [],
                |r| r.get(0),
            )
            .unwrap();
        assert_eq!(hits_after, 0);
    }
}
