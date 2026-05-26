//! 번역 기록 레포지토리 + FTS5 검색 (PRD §9.1 / §6.6 / §10.6).
//!
//! - `HistoryRepo` 가 풀에서 connection 을 빌려 한 connection 안에서 작업한다.
//! - 모든 SQL 은 repo 안에서만 작성하고, command 레이어는 typed 함수만 호출한다.
//! - FTS 검색은 `translation_records_fts MATCH ?` + JOIN 으로 본 행을 다시 가져온다.
//! - tags 는 JSON 문자열 1 컬럼. 필터는 `tags_json LIKE '%"tag"%'` 로 v1 에 충분.

use std::sync::Arc;

use chrono::SecondsFormat;
use rusqlite::{params, OptionalExtension, ToSql};
use serde::{Deserialize, Serialize};

use crate::db::{Conn, Pool};
use crate::errors::{AppError, AppResult};
use crate::language::SourceLanguage;

/// PRD §9.1 TranslationRecord — FE 와 1:1 매핑되는 외부 shape.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TranslationRecord {
    pub id: String,
    pub source_text: String,
    pub source_language: SourceLanguage,
    pub translated_text: String,
    pub model: String,
    pub duration_ms: i64,
    pub created_at: String,
    pub is_favorite: bool,
    pub tags: Vec<String>,
}

/// `list` / `search` 입력 파라미터.
#[derive(Debug, Clone, Default)]
pub struct ListQuery {
    pub limit: i64,
    pub offset: i64,
    pub favorite_only: bool,
    pub tag: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ListResult {
    pub records: Vec<TranslationRecord>,
    pub total: i64,
}

#[derive(Debug, Clone)]
pub struct InsertRecord {
    pub id: String,
    pub source_text: String,
    pub source_language: SourceLanguage,
    pub translated_text: String,
    pub model: String,
    pub duration_ms: i64,
    pub created_at: String,
}

/// 풀 wrapper — `tauri::State` 에 `Arc<HistoryRepo>` 로 보관해 commands 가 공유한다.
pub struct HistoryRepo {
    pool: Pool,
}

impl HistoryRepo {
    pub fn new(pool: Pool) -> Arc<Self> {
        Arc::new(Self { pool })
    }

    fn conn(&self) -> AppResult<Conn> {
        self.pool
            .get()
            .map_err(|e| AppError::internal(format!("db conn acquire: {e}")))
    }

    /// 단일 INSERT — cancel/persist atomicity 는 호출처 (commands/translate.rs) 의 terminal
    /// mutex 가 보장한다 (code-review v1 follow-up review §10, Critical 1 v3).
    pub fn insert(&self, record: InsertRecord) -> AppResult<()> {
        let conn = self.conn()?;
        let lang = serde_json::to_string(&record.source_language)
            .map_err(|e| AppError::internal(format!("source_language serialize: {e}")))?
            .trim_matches('"')
            .to_string();
        conn.prepare_cached(
            "INSERT INTO translation_records (id, source_text, source_language, translated_text, model, duration_ms, created_at) \
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        )
        .map_err(map_sql_err)?
        .execute(params![
            record.id,
            record.source_text,
            lang,
            record.translated_text,
            record.model,
            record.duration_ms,
            record.created_at,
        ])
        .map_err(map_sql_err)?;
        Ok(())
    }

    pub fn list(&self, query: &ListQuery) -> AppResult<ListResult> {
        let conn = self.conn()?;
        let (where_sql, params_vec) = build_filter(query.favorite_only, query.tag.as_deref());

        let count_sql = format!("SELECT count(*) FROM translation_records {where_sql}");
        let total: i64 = conn
            .prepare_cached(&count_sql)
            .map_err(map_sql_err)?
            .query_row(rusqlite::params_from_iter(params_vec.iter()), |r| r.get(0))
            .map_err(map_sql_err)?;

        let sql = format!(
            "SELECT id, source_text, source_language, translated_text, model, duration_ms, created_at, is_favorite, tags_json \
             FROM translation_records {where_sql} \
             ORDER BY created_at DESC, rowid DESC LIMIT ?{limit_idx} OFFSET ?{offset_idx}",
            where_sql = where_sql,
            limit_idx = params_vec.len() + 1,
            offset_idx = params_vec.len() + 2,
        );
        let mut stmt = conn.prepare_cached(&sql).map_err(map_sql_err)?;
        let mut p: Vec<Box<dyn ToSql>> = params_vec.into_iter().collect();
        p.push(Box::new(query.limit));
        p.push(Box::new(query.offset));
        let records = stmt
            .query_map(
                rusqlite::params_from_iter(p.iter().map(|b| b.as_ref())),
                row_to_record,
            )
            .map_err(map_sql_err)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(map_sql_err)?;

        Ok(ListResult { records, total })
    }

    pub fn search(&self, q: &str, query: &ListQuery) -> AppResult<ListResult> {
        let conn = self.conn()?;
        let fts_query = build_fts_query(q);
        if fts_query.is_empty() {
            // 검색어가 의미 있는 토큰이 없으면 list 와 동일 동작.
            return self.list(query);
        }

        let (filter_sql, filter_params) = build_filter(query.favorite_only, query.tag.as_deref());

        let count_sql = format!(
            "SELECT count(*) FROM translation_records r \
             JOIN translation_records_fts f ON f.rowid = r.rowid \
             WHERE translation_records_fts MATCH ?1 \
             {extra}",
            extra = if filter_sql.is_empty() {
                String::new()
            } else {
                format!("AND ({})", filter_sql.trim_start_matches("WHERE "))
            },
        );
        let mut count_params: Vec<Box<dyn ToSql>> = vec![Box::new(fts_query.clone())];
        count_params.extend(
            filter_params
                .iter()
                .map(|b| -> Box<dyn ToSql> { Box::new(b.as_string_for_clone()) }),
        );
        let total: i64 = conn
            .prepare_cached(&count_sql)
            .map_err(map_sql_err)?
            .query_row(
                rusqlite::params_from_iter(count_params.iter().map(|b| b.as_ref())),
                |r| r.get(0),
            )
            .map_err(map_sql_err)?;

        let sql = format!(
            "SELECT r.id, r.source_text, r.source_language, r.translated_text, r.model, r.duration_ms, r.created_at, r.is_favorite, r.tags_json \
             FROM translation_records r \
             JOIN translation_records_fts f ON f.rowid = r.rowid \
             WHERE translation_records_fts MATCH ?1 \
             {extra} \
             ORDER BY rank LIMIT ?{limit_idx} OFFSET ?{offset_idx}",
            extra = if filter_sql.is_empty() {
                String::new()
            } else {
                format!("AND ({})", filter_sql.trim_start_matches("WHERE "))
            },
            limit_idx = 2 + filter_params.len(),
            offset_idx = 3 + filter_params.len(),
        );
        let mut p: Vec<Box<dyn ToSql>> = vec![Box::new(fts_query)];
        p.extend(filter_params);
        p.push(Box::new(query.limit));
        p.push(Box::new(query.offset));

        let mut stmt = conn.prepare_cached(&sql).map_err(map_sql_err)?;
        let records = stmt
            .query_map(
                rusqlite::params_from_iter(p.iter().map(|b| b.as_ref())),
                row_to_record,
            )
            .map_err(map_sql_err)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(map_sql_err)?;

        Ok(ListResult { records, total })
    }

    pub fn get(&self, id: &str) -> AppResult<Option<TranslationRecord>> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare_cached(
                "SELECT id, source_text, source_language, translated_text, model, duration_ms, created_at, is_favorite, tags_json \
                 FROM translation_records WHERE id = ?1",
            )
            .map_err(map_sql_err)?;
        let row = stmt
            .query_row([id], row_to_record)
            .optional()
            .map_err(map_sql_err)?;
        Ok(row)
    }

    pub fn delete(&self, id: &str) -> AppResult<bool> {
        let conn = self.conn()?;
        let affected = conn
            .prepare_cached("DELETE FROM translation_records WHERE id = ?1")
            .map_err(map_sql_err)?
            .execute([id])
            .map_err(map_sql_err)?;
        Ok(affected > 0)
    }

    pub fn delete_all(&self) -> AppResult<i64> {
        let conn = self.conn()?;
        let affected = conn
            .execute("DELETE FROM translation_records", [])
            .map_err(map_sql_err)?;
        Ok(affected as i64)
    }

    /// favorite 비트를 토글하고 새 값을 반환. 행이 없으면 `Ok(None)`.
    pub fn toggle_favorite(&self, id: &str) -> AppResult<Option<bool>> {
        let conn = self.conn()?;
        let current: Option<i64> = conn
            .prepare_cached("SELECT is_favorite FROM translation_records WHERE id = ?1")
            .map_err(map_sql_err)?
            .query_row([id], |r| r.get(0))
            .optional()
            .map_err(map_sql_err)?;
        let Some(current) = current else {
            return Ok(None);
        };
        let next: i64 = if current == 0 { 1 } else { 0 };
        conn.prepare_cached("UPDATE translation_records SET is_favorite = ?1 WHERE id = ?2")
            .map_err(map_sql_err)?
            .execute(params![next, id])
            .map_err(map_sql_err)?;
        Ok(Some(next != 0))
    }

    pub fn set_tags(&self, id: &str, tags: &[String]) -> AppResult<bool> {
        let conn = self.conn()?;
        let normalized: Vec<String> = normalize_tags(tags);
        let json = serde_json::to_string(&normalized)
            .map_err(|e| AppError::internal(format!("tags serialize: {e}")))?;
        let affected = conn
            .prepare_cached("UPDATE translation_records SET tags_json = ?1 WHERE id = ?2")
            .map_err(map_sql_err)?
            .execute(params![json, id])
            .map_err(map_sql_err)?;
        Ok(affected > 0)
    }

    /// Export 용 — 전체 기록을 created_at DESC 순서로 모두 반환한다.
    /// v1 에서는 row 수가 보통 < 50k 이므로 단발 알로케이션이 단순하다.
    pub fn list_all(&self) -> AppResult<Vec<TranslationRecord>> {
        let conn = self.conn()?;
        let mut stmt = conn
            .prepare_cached(
                "SELECT id, source_text, source_language, translated_text, model, duration_ms, created_at, is_favorite, tags_json \
                 FROM translation_records ORDER BY created_at DESC, rowid DESC",
            )
            .map_err(map_sql_err)?;
        let rows = stmt
            .query_map([], row_to_record)
            .map_err(map_sql_err)?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(map_sql_err)?;
        Ok(rows)
    }
}

/// ISO 8601 UTC timestamp helper — repo 사용자가 직접 만들지 않아도 되도록.
pub fn now_iso8601() -> String {
    chrono::Utc::now().to_rfc3339_opts(SecondsFormat::Millis, true)
}

fn row_to_record(row: &rusqlite::Row<'_>) -> rusqlite::Result<TranslationRecord> {
    let lang_raw: String = row.get(2)?;
    let source_language = parse_language(&lang_raw).ok_or_else(|| {
        rusqlite::Error::FromSqlConversionFailure(
            2,
            rusqlite::types::Type::Text,
            format!("unknown source_language: {lang_raw}").into(),
        )
    })?;
    let tags_json: String = row.get(8)?;
    let tags: Vec<String> = serde_json::from_str(&tags_json).unwrap_or_default();
    let is_favorite: i64 = row.get(7)?;
    Ok(TranslationRecord {
        id: row.get(0)?,
        source_text: row.get(1)?,
        source_language,
        translated_text: row.get(3)?,
        model: row.get(4)?,
        duration_ms: row.get(5)?,
        created_at: row.get(6)?,
        is_favorite: is_favorite != 0,
        tags,
    })
}

fn parse_language(s: &str) -> Option<SourceLanguage> {
    match s {
        "Korean" => Some(SourceLanguage::Korean),
        "ChineseSimplified" => Some(SourceLanguage::ChineseSimplified),
        "ChineseTraditional" => Some(SourceLanguage::ChineseTraditional),
        "Auto" => Some(SourceLanguage::Auto),
        _ => None,
    }
}

fn map_sql_err(e: rusqlite::Error) -> AppError {
    AppError::internal(format!("sqlite: {e}"))
}

/// favorite / tag 필터를 WHERE 구문 + ToSql 매개 변수 묶음으로 변환한다.
/// 결과 SQL 은 `WHERE ...` 로 시작하거나 빈 문자열.
fn build_filter(favorite_only: bool, tag: Option<&str>) -> (String, Vec<Box<dyn ToSql>>) {
    let mut clauses: Vec<String> = Vec::new();
    let mut params_vec: Vec<Box<dyn ToSql>> = Vec::new();

    if favorite_only {
        clauses.push("is_favorite = 1".to_string());
    }
    if let Some(t) = tag {
        let trimmed = t.trim();
        if !trimmed.is_empty() {
            params_vec.push(Box::new(format!("%\"{}\"%", trimmed)));
            clauses.push(format!("tags_json LIKE ?{}", params_vec.len()));
        }
    }
    if clauses.is_empty() {
        (String::new(), params_vec)
    } else {
        (format!("WHERE {}", clauses.join(" AND ")), params_vec)
    }
}

/// 사용자 입력 검색어를 FTS5 안전 형태로 변환. 빈 토큰만 있으면 빈 문자열을 반환해
/// caller 가 list fallback 으로 빠질 수 있게 한다.
fn build_fts_query(input: &str) -> String {
    let tokens: Vec<String> = input
        .split_whitespace()
        .filter_map(|raw| {
            let cleaned: String = raw
                .chars()
                .filter(|c| c.is_alphanumeric() || c.is_whitespace())
                .collect();
            let cleaned = cleaned.trim();
            if cleaned.is_empty() {
                None
            } else {
                Some(format!("\"{cleaned}\"*"))
            }
        })
        .collect();
    tokens.join(" ")
}

/// 태그를 trim + dedupe + 빈 값 제거.
fn normalize_tags(tags: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::with_capacity(tags.len());
    for t in tags {
        let trimmed = t.trim().to_string();
        if trimmed.is_empty() {
            continue;
        }
        if !out.iter().any(|existing| existing == &trimmed) {
            out.push(trimmed);
        }
    }
    out
}

/// ToSql 박스를 string 으로 다시 한 번 빌려야 할 때 쓰는 헬퍼 trait.
/// search() 의 count 쿼리에서 filter 매개 변수를 재사용하기 위함.
trait ToSqlClone {
    fn as_string_for_clone(&self) -> String;
}

impl ToSqlClone for Box<dyn ToSql> {
    fn as_string_for_clone(&self) -> String {
        let v = self.to_sql().ok();
        match v {
            Some(rusqlite::types::ToSqlOutput::Owned(rusqlite::types::Value::Text(s))) => s,
            Some(rusqlite::types::ToSqlOutput::Borrowed(rusqlite::types::ValueRef::Text(b))) => {
                String::from_utf8_lossy(b).into_owned()
            }
            _ => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db;

    fn fresh_repo() -> (tempfile::TempDir, Arc<HistoryRepo>) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hytranslate.sqlite");
        let pool = db::open(&path).unwrap();
        (dir, HistoryRepo::new(pool))
    }

    fn sample(id: &str, source: &str, translated: &str) -> InsertRecord {
        InsertRecord {
            id: id.to_string(),
            source_text: source.to_string(),
            source_language: SourceLanguage::Korean,
            translated_text: translated.to_string(),
            model: "test-model".to_string(),
            duration_ms: 100,
            created_at: now_iso8601(),
        }
    }

    #[test]
    fn insert_and_list_round_trip() {
        let (_d, repo) = fresh_repo();
        repo.insert(sample("a", "안녕하세요", "Hello")).unwrap();
        repo.insert(sample("b", "감사합니다", "Thank you")).unwrap();
        let res = repo
            .list(&ListQuery {
                limit: 10,
                offset: 0,
                ..Default::default()
            })
            .unwrap();
        assert_eq!(res.total, 2);
        assert_eq!(res.records.len(), 2);
        // 가장 마지막에 insert 된 b 가 위에 오거나 동률이면 rowid DESC.
        assert!(res.records.iter().any(|r| r.id == "a"));
        assert!(res.records.iter().any(|r| r.id == "b"));
    }

    #[test]
    fn search_uses_fts_and_returns_matching_rows() {
        let (_d, repo) = fresh_repo();
        repo.insert(sample("a", "안녕하세요", "Hello world"))
            .unwrap();
        repo.insert(sample("b", "감사합니다", "Thank you very much"))
            .unwrap();
        repo.insert(sample("c", "안녕", "Hi")).unwrap();

        let res = repo
            .search(
                "thank",
                &ListQuery {
                    limit: 10,
                    offset: 0,
                    ..Default::default()
                },
            )
            .unwrap();
        let ids: Vec<&str> = res.records.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, vec!["b"]);
        assert_eq!(res.total, 1);
    }

    #[test]
    fn search_empty_query_falls_back_to_list() {
        let (_d, repo) = fresh_repo();
        repo.insert(sample("a", "hi", "안녕")).unwrap();
        let res = repo
            .search(
                "   ",
                &ListQuery {
                    limit: 10,
                    offset: 0,
                    ..Default::default()
                },
            )
            .unwrap();
        assert_eq!(res.total, 1);
    }

    #[test]
    fn delete_individual_and_all() {
        let (_d, repo) = fresh_repo();
        repo.insert(sample("a", "x", "y")).unwrap();
        repo.insert(sample("b", "x", "y")).unwrap();
        assert!(repo.delete("a").unwrap());
        assert_eq!(repo.list(&q(10)).unwrap().total, 1);
        let n = repo.delete_all().unwrap();
        assert_eq!(n, 1);
        assert_eq!(repo.list(&q(10)).unwrap().total, 0);
    }

    #[test]
    fn toggle_favorite_flips_bit() {
        let (_d, repo) = fresh_repo();
        repo.insert(sample("a", "x", "y")).unwrap();
        assert_eq!(repo.toggle_favorite("a").unwrap(), Some(true));
        assert_eq!(repo.toggle_favorite("a").unwrap(), Some(false));
        assert_eq!(repo.toggle_favorite("missing").unwrap(), None);
    }

    #[test]
    fn set_tags_round_trip_and_normalize() {
        let (_d, repo) = fresh_repo();
        repo.insert(sample("a", "x", "y")).unwrap();
        repo.set_tags(
            "a",
            &[
                "  법무 ".to_string(),
                "법무".to_string(),
                "".to_string(),
                "연구".to_string(),
            ],
        )
        .unwrap();
        let got = repo.get("a").unwrap().unwrap();
        assert_eq!(got.tags, vec!["법무".to_string(), "연구".to_string()]);
    }

    #[test]
    fn favorite_filter_restricts_list() {
        let (_d, repo) = fresh_repo();
        repo.insert(sample("a", "x", "y")).unwrap();
        repo.insert(sample("b", "x", "y")).unwrap();
        repo.toggle_favorite("a").unwrap();
        let res = repo
            .list(&ListQuery {
                limit: 10,
                offset: 0,
                favorite_only: true,
                tag: None,
            })
            .unwrap();
        assert_eq!(res.total, 1);
        assert_eq!(res.records[0].id, "a");
    }

    #[test]
    fn tag_filter_uses_like() {
        let (_d, repo) = fresh_repo();
        repo.insert(sample("a", "x", "y")).unwrap();
        repo.insert(sample("b", "x", "y")).unwrap();
        repo.set_tags("a", &["법무".to_string()]).unwrap();
        let res = repo
            .list(&ListQuery {
                limit: 10,
                offset: 0,
                favorite_only: false,
                tag: Some("법무".to_string()),
            })
            .unwrap();
        assert_eq!(res.total, 1);
        assert_eq!(res.records[0].id, "a");
    }

    #[test]
    fn list_all_returns_all_rows_descending() {
        let (_d, repo) = fresh_repo();
        for i in 0..3 {
            let mut rec = sample(&format!("r{i}"), "src", "dst");
            // 시간을 살짝 어긋나게 — created_at DESC 검증.
            rec.created_at = format!("2026-05-26T00:00:0{i}Z");
            repo.insert(rec).unwrap();
        }
        let all = repo.list_all().unwrap();
        let ids: Vec<&str> = all.iter().map(|r| r.id.as_str()).collect();
        assert_eq!(ids, vec!["r2", "r1", "r0"]);
    }

    fn q(limit: i64) -> ListQuery {
        ListQuery {
            limit,
            offset: 0,
            favorite_only: false,
            tag: None,
        }
    }

    #[test]
    fn fts_query_strips_punctuation_and_quotes_tokens() {
        assert_eq!(build_fts_query("thank you!"), "\"thank\"* \"you\"*");
        assert_eq!(build_fts_query("   "), "");
        assert_eq!(build_fts_query("\"hello\""), "\"hello\"*");
    }

    #[test]
    fn normalize_tags_dedups_and_trims() {
        let out = normalize_tags(&["a".into(), " a ".into(), "".into(), "b".into()]);
        assert_eq!(out, vec!["a".to_string(), "b".to_string()]);
    }
}
