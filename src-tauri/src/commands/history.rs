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
        // 코드리뷰 Med 2 — CSV formula injection 방어.
        // model 출력 / 사용자 입력 / 태그 모두 신뢰 불가. `=`, `+`, `-`, `@`, tab, CR 로
        // 시작하는 셀에 single-quote 를 선행 부착해 spreadsheet 가 formula 로 해석하지 않게 한다.
        let model = neutralize_csv_cell(r.model.as_str());
        let tags_neutralized = neutralize_csv_cell(tags.as_str());
        let source = neutralize_csv_cell(r.source_text.as_str());
        let translated = neutralize_csv_cell(r.translated_text.as_str());
        writer
            .write_record([
                r.id.as_str(),
                r.created_at.as_str(),
                lang.as_str(),
                model.as_str(),
                &r.duration_ms.to_string(),
                if r.is_favorite { "1" } else { "0" },
                tags_neutralized.as_str(),
                source.as_str(),
                translated.as_str(),
            ])
            .map_err(|e| AppError::internal(format!("csv row: {e}")))?;
    }
    writer
        .flush()
        .map_err(|e| AppError::internal(format!("csv flush: {e}")))?;
    Ok(())
}

/// OWASP "CSV Injection" 방어 — `=`, `+`, `-`, `@`, tab, CR 로 시작하는 셀에
/// single-quote (`'`) prefix 를 붙여 Excel/Numbers/Sheets 가 formula 로 해석하지 못하게 한다.
/// 빈 문자열은 그대로.
fn neutralize_csv_cell(value: &str) -> String {
    let first = value.chars().next();
    match first {
        Some('=') | Some('+') | Some('-') | Some('@') | Some('\t') | Some('\r') => {
            let mut out = String::with_capacity(value.len() + 1);
            out.push('\'');
            out.push_str(value);
            out
        }
        _ => value.to_string(),
    }
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
    fn csv_neutralizes_formula_leading_cells() {
        // 코드리뷰 Med 2 회귀 — 모델 출력 / 사용자 입력에 formula 가 끼어도
        // Excel/Numbers/Sheets 가 실행하지 않도록 single-quote 가 선행해야 한다.
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("h.csv");
        let mut r = rec("1", "src", "dst", vec![], false);
        r.source_text = "=HYPERLINK(\"http://evil\",\"x\")".to_string();
        r.translated_text = "+cmd|'/c calc'!A1".to_string();
        r.tags = vec!["@SUM(A1:A2)".to_string()];
        r.model = "-malicious".to_string();
        write_csv(&path, &[r]).unwrap();
        let body = fs::read_to_string(&path).unwrap();
        assert!(body.contains("'=HYPERLINK"), "= should be quoted: {body:?}");
        assert!(body.contains("'+cmd|"), "+ should be quoted: {body:?}");
        assert!(body.contains("'@SUM"), "@ should be quoted: {body:?}");
        assert!(body.contains("'-malicious"), "- should be quoted: {body:?}");
    }

    #[test]
    fn csv_does_not_double_quote_benign_cells() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("h.csv");
        write_csv(
            &path,
            &[rec("id", "안녕하세요", "Hello", vec!["연구"], false)],
        )
        .unwrap();
        let body = fs::read_to_string(&path).unwrap();
        assert!(!body.contains("'안녕"));
        assert!(!body.contains("'Hello"));
        assert!(!body.contains("'연구"));
    }

    #[test]
    fn neutralize_csv_cell_matrix() {
        assert_eq!(neutralize_csv_cell("=A1"), "'=A1");
        assert_eq!(neutralize_csv_cell("+1"), "'+1");
        assert_eq!(neutralize_csv_cell("-1"), "'-1");
        assert_eq!(neutralize_csv_cell("@x"), "'@x");
        assert_eq!(neutralize_csv_cell("\tx"), "'\tx");
        assert_eq!(neutralize_csv_cell("\rx"), "'\rx");
        assert_eq!(neutralize_csv_cell(""), "");
        assert_eq!(neutralize_csv_cell("hello"), "hello");
        assert_eq!(neutralize_csv_cell("안녕"), "안녕");
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
