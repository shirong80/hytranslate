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
use uuid::Uuid;

use crate::commands::translate::MAIN_INPUT_LIMIT;
use crate::errors::{AppError, AppResult};
use crate::history::{
    now_iso8601, HistoryRepo, InsertRecord, ListQuery, ListResult, TranslationRecord,
};
use crate::language::SourceLanguage;
use crate::settings::SettingsStore;

/// 저장 경로 translated_text 상한. 정상 번역은 입력 cap 의 몇 배를 넘지 않으므로
/// 넉넉히 잡되, 웹뷰가 MB 단위 blob 을 직접 밀어넣는 abuse 는 차단한다.
const MAX_TRANSLATED_LEN: usize = 120_000;

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

/// Cmd+Enter 수동 저장 페이로드. `created_at` 은 백엔드가 생성하므로 FE 가 보내지 않는다.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveRecordRequest {
    pub id: String,
    pub source_text: String,
    pub source_language: SourceLanguage,
    pub translated_text: String,
    pub model: String,
    pub duration_ms: i64,
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

/// 사용자가 Cmd+Enter 로 명시 저장하는 단일 레코드. `save_history` 가 꺼져 있으면 INSERT
/// 없이 `Ok(())` 를 반환한다 (FE 는 결과와 무관하게 입력을 비운다). 저장 자체가 실패해도
/// 이력 손실은 비치명적이므로 FE 흐름은 영향받지 않는다.
#[tauri::command]
pub async fn save_translation_record(
    repo: tauri::State<'_, Arc<HistoryRepo>>,
    settings: tauri::State<'_, Arc<SettingsStore>>,
    request: SaveRecordRequest,
) -> AppResult<()> {
    save_record_inner(&repo, settings.get().save_history, request)
}

/// 테스트 가능한 코어 — tauri::State 분리. `save_history` 게이팅은 백엔드가 단일 진실원.
///
/// 웹뷰는 translate 경로의 길이 cap·UUID requestId 를 우회해 이 커맨드를 직접 호출할 수
/// 있다(XSS 포함). 따라서 저장 경로에서도 입력을 신뢰하지 않고 다시 방어한다 — 길이 cap 으로
/// DB 비대화/디스크 고갈(DoS)을, id 형식 검증으로 쓰레기 식별자 유입을 막는다.
fn save_record_inner(
    repo: &HistoryRepo,
    save_history: bool,
    request: SaveRecordRequest,
) -> AppResult<()> {
    if !save_history {
        return Ok(());
    }
    if request.source_text.chars().count() > MAIN_INPUT_LIMIT {
        return Err(AppError::InputTooLong {
            limit: MAIN_INPUT_LIMIT,
        });
    }
    if request.translated_text.chars().count() > MAX_TRANSLATED_LEN {
        return Err(AppError::InputTooLong {
            limit: MAX_TRANSLATED_LEN,
        });
    }
    Uuid::parse_str(&request.id)
        .map_err(|e| AppError::internal(format!("invalid record id (UUID expected): {e}")))?;
    repo.insert(InsertRecord {
        id: request.id,
        source_text: request.source_text,
        source_language: request.source_language,
        translated_text: request.translated_text,
        model: request.model,
        duration_ms: request.duration_ms,
        created_at: now_iso8601(),
    })
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

    fn fresh_repo() -> (tempfile::TempDir, Arc<HistoryRepo>) {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("hytranslate.sqlite");
        let pool = crate::db::open(&path).unwrap();
        (dir, HistoryRepo::new(pool))
    }

    const UUID_A: &str = "11111111-1111-4111-8111-111111111111";

    fn save_request(id: &str) -> SaveRecordRequest {
        SaveRecordRequest {
            id: id.to_string(),
            source_text: "안녕하세요".to_string(),
            source_language: SourceLanguage::Korean,
            translated_text: "Hello".to_string(),
            model: "test-model".to_string(),
            duration_ms: 100,
        }
    }

    #[test]
    fn save_record_inner_writes_when_save_history_on() {
        let (_d, repo) = fresh_repo();
        save_record_inner(&repo, true, save_request(UUID_A)).unwrap();
        assert!(repo.get(UUID_A).unwrap().is_some());
    }

    #[test]
    fn save_record_inner_skips_when_save_history_off() {
        let (_d, repo) = fresh_repo();
        save_record_inner(&repo, false, save_request(UUID_A)).unwrap();
        assert!(repo.get(UUID_A).unwrap().is_none());
    }

    #[test]
    fn save_record_inner_rejects_non_uuid_id() {
        let (_d, repo) = fresh_repo();
        let err = save_record_inner(&repo, true, save_request("not-a-uuid"));
        assert!(err.is_err());
        assert!(repo.get("not-a-uuid").unwrap().is_none());
    }

    #[test]
    fn save_record_inner_rejects_oversized_source_text() {
        let (_d, repo) = fresh_repo();
        let mut req = save_request(UUID_A);
        req.source_text = "가".repeat(MAIN_INPUT_LIMIT + 1);
        let err = save_record_inner(&repo, true, req);
        assert!(matches!(err, Err(AppError::InputTooLong { .. })));
        assert!(repo.get(UUID_A).unwrap().is_none());
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
