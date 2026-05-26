//! 이력 명령 (PRD §10.6).
//!
//! command 는 어댑터만 — 모든 SQL 은 `crate::history::HistoryRepo` 에서 처리한다.
//! Export 두 종은 `tauri-plugin-dialog::save_file` 로 사용자가 직접 경로를 선택한 뒤
//! 백엔드가 파일을 기록한다. dialog 가 취소되면 `Ok(None)` 을 반환해 FE 가 silent 처리.

use std::fs;
use std::io::Write as _;
use std::path::PathBuf;
use std::sync::Arc;

use serde::{Deserialize, Serialize};
use tauri_plugin_dialog::{DialogExt, FilePath};
use tokio::sync::oneshot;

use crate::errors::{AppError, AppResult};
use crate::history::{HistoryRepo, ListQuery, ListResult, TranslationRecord};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ListRequest {
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub favorite_only: Option<bool>,
    pub tag: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchRequest {
    pub query: String,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
    pub favorite_only: Option<bool>,
    pub tag: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IdRequest {
    pub id: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SetTagsRequest {
    pub id: String,
    pub tags: Vec<String>,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportResult {
    /// dialog 취소면 `None` — FE 가 silent 처리.
    pub path: Option<String>,
    pub records: i64,
}

const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 500;

fn build_query(
    limit: Option<i64>,
    offset: Option<i64>,
    favorite_only: Option<bool>,
    tag: Option<String>,
) -> ListQuery {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);
    let offset = offset.unwrap_or(0).max(0);
    ListQuery {
        limit,
        offset,
        favorite_only: favorite_only.unwrap_or(false),
        tag,
    }
}

#[tauri::command]
pub async fn list_translation_records(
    repo: tauri::State<'_, Arc<HistoryRepo>>,
    request: ListRequest,
) -> AppResult<ListResult> {
    let q = build_query(
        request.limit,
        request.offset,
        request.favorite_only,
        request.tag,
    );
    repo.list(&q)
}

#[tauri::command]
pub async fn search_translation_records(
    repo: tauri::State<'_, Arc<HistoryRepo>>,
    request: SearchRequest,
) -> AppResult<ListResult> {
    let q = build_query(
        request.limit,
        request.offset,
        request.favorite_only,
        request.tag,
    );
    repo.search(&request.query, &q)
}

#[tauri::command]
pub async fn get_translation_record(
    repo: tauri::State<'_, Arc<HistoryRepo>>,
    request: IdRequest,
) -> AppResult<Option<TranslationRecord>> {
    repo.get(&request.id)
}

#[tauri::command]
pub async fn delete_translation_record(
    repo: tauri::State<'_, Arc<HistoryRepo>>,
    request: IdRequest,
) -> AppResult<bool> {
    repo.delete(&request.id)
}

#[tauri::command]
pub async fn delete_all_translation_records(
    repo: tauri::State<'_, Arc<HistoryRepo>>,
) -> AppResult<i64> {
    repo.delete_all()
}

#[tauri::command]
pub async fn toggle_favorite(
    repo: tauri::State<'_, Arc<HistoryRepo>>,
    request: IdRequest,
) -> AppResult<Option<bool>> {
    repo.toggle_favorite(&request.id)
}

#[tauri::command]
pub async fn set_tags(
    repo: tauri::State<'_, Arc<HistoryRepo>>,
    request: SetTagsRequest,
) -> AppResult<bool> {
    repo.set_tags(&request.id, &request.tags)
}

#[tauri::command]
pub async fn export_history_csv<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    repo: tauri::State<'_, Arc<HistoryRepo>>,
) -> AppResult<ExportResult> {
    let records = repo.list_all()?;
    let Some(path) = pick_save_path(&app, "hytranslate-history.csv", "CSV", &["csv"]).await else {
        return Ok(ExportResult {
            path: None,
            records: 0,
        });
    };
    write_csv(&path, &records)?;
    Ok(ExportResult {
        path: path.to_string_lossy().into_owned().into(),
        records: records.len() as i64,
    })
}

#[tauri::command]
pub async fn export_history_json<R: tauri::Runtime>(
    app: tauri::AppHandle<R>,
    repo: tauri::State<'_, Arc<HistoryRepo>>,
) -> AppResult<ExportResult> {
    let records = repo.list_all()?;
    let Some(path) = pick_save_path(&app, "hytranslate-history.json", "JSON", &["json"]).await
    else {
        return Ok(ExportResult {
            path: None,
            records: 0,
        });
    };
    write_json(&path, &records)?;
    Ok(ExportResult {
        path: path.to_string_lossy().into_owned().into(),
        records: records.len() as i64,
    })
}

async fn pick_save_path<R: tauri::Runtime>(
    app: &tauri::AppHandle<R>,
    file_name: &str,
    filter_name: &str,
    extensions: &[&str],
) -> Option<PathBuf> {
    let (tx, rx) = oneshot::channel::<Option<FilePath>>();
    app.dialog()
        .file()
        .add_filter(filter_name, extensions)
        .set_file_name(file_name)
        .save_file(move |path| {
            let _ = tx.send(path);
        });
    let chosen = rx.await.ok().flatten()?;
    chosen.into_path().ok()
}

pub(crate) fn write_csv(path: &std::path::Path, records: &[TranslationRecord]) -> AppResult<()> {
    let file =
        fs::File::create(path).map_err(|e| AppError::internal(format!("csv create: {e}")))?;
    let mut writer = csv::Writer::from_writer(file);
    writer
        .write_record([
            "id",
            "created_at",
            "source_language",
            "model",
            "duration_ms",
            "is_favorite",
            "tags",
            "source_text",
            "translated_text",
        ])
        .map_err(|e| AppError::internal(format!("csv header: {e}")))?;
    for r in records {
        let tags = r.tags.join("|");
        let lang = serde_json::to_string(&r.source_language)
            .map(|s| s.trim_matches('"').to_string())
            .unwrap_or_default();
        writer
            .write_record([
                r.id.as_str(),
                r.created_at.as_str(),
                lang.as_str(),
                r.model.as_str(),
                &r.duration_ms.to_string(),
                if r.is_favorite { "1" } else { "0" },
                tags.as_str(),
                r.source_text.as_str(),
                r.translated_text.as_str(),
            ])
            .map_err(|e| AppError::internal(format!("csv row: {e}")))?;
    }
    writer
        .flush()
        .map_err(|e| AppError::internal(format!("csv flush: {e}")))?;
    Ok(())
}

pub(crate) fn write_json(path: &std::path::Path, records: &[TranslationRecord]) -> AppResult<()> {
    let mut file =
        fs::File::create(path).map_err(|e| AppError::internal(format!("json create: {e}")))?;
    let payload = serde_json::to_vec_pretty(records)
        .map_err(|e| AppError::internal(format!("json serialize: {e}")))?;
    file.write_all(&payload)
        .map_err(|e| AppError::internal(format!("json write: {e}")))?;
    file.sync_all()
        .map_err(|e| AppError::internal(format!("json fsync: {e}")))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::language::SourceLanguage;

    fn rec(id: &str, src: &str, dst: &str, tags: Vec<&str>, fav: bool) -> TranslationRecord {
        TranslationRecord {
            id: id.to_string(),
            source_text: src.to_string(),
            source_language: SourceLanguage::Korean,
            translated_text: dst.to_string(),
            model: "m".to_string(),
            duration_ms: 42,
            created_at: "2026-05-26T00:00:00Z".to_string(),
            is_favorite: fav,
            tags: tags.into_iter().map(String::from).collect(),
        }
    }

    #[test]
    fn csv_export_writes_header_and_rows() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("h.csv");
        write_csv(
            &path,
            &[
                rec("1", "안녕", "Hi", vec!["연구"], false),
                rec("2", "감사", "Thanks", vec![], true),
            ],
        )
        .unwrap();
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.starts_with("id,created_at,source_language,model,duration_ms,is_favorite,tags,source_text,translated_text"));
        assert!(body.contains("1,2026-05-26T00:00:00Z,Korean,m,42,0,연구,안녕,Hi"));
        assert!(body.contains("2,2026-05-26T00:00:00Z,Korean,m,42,1,,감사,Thanks"));
    }

    #[test]
    fn json_export_writes_array() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("h.json");
        write_json(&path, &[rec("1", "안녕", "Hi", vec!["연구"], false)]).unwrap();
        let body = fs::read_to_string(&path).unwrap();
        let parsed: Vec<TranslationRecord> = serde_json::from_str(&body).unwrap();
        assert_eq!(parsed.len(), 1);
        assert_eq!(parsed[0].id, "1");
        assert_eq!(parsed[0].tags, vec!["연구".to_string()]);
    }

    #[test]
    fn build_query_clamps_limit_and_offset() {
        let q = build_query(Some(10_000), Some(-5), Some(true), Some("t".into()));
        assert_eq!(q.limit, MAX_LIMIT);
        assert_eq!(q.offset, 0);
        assert!(q.favorite_only);
        assert_eq!(q.tag.as_deref(), Some("t"));
    }

    #[test]
    fn build_query_applies_defaults() {
        let q = build_query(None, None, None, None);
        assert_eq!(q.limit, DEFAULT_LIMIT);
        assert_eq!(q.offset, 0);
        assert!(!q.favorite_only);
        assert!(q.tag.is_none());
    }
}
