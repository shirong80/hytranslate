//! 마이그레이션 outcome 노출 + 사용자 명시 cleanup CTA 처리 (Major 7 수동 단계).
//!
//! code-review v1 follow-up review §24 v3 — destructive cleanup 은 backend 가 발급하는
//! 1회용 nonce 토큰 + 사용자가 직접 타이핑한 confirmation phrase 두 단계 검증을 요구한다.
//!
//! 1. UI 는 backend 의 `cleanup_confirmation_phrase()` 로 폴더 이름을 받아 사용자에게 표시.
//! 2. 사용자가 입력박스에 phrase 를 직접 타이핑하면 UI 가 `issue_cleanup_token({confirmation})`
//!    호출. backend 는 outcome.legacy_dir 의 base name 과 비교해 일치할 때만 토큰 발급.
//! 3. 같은 confirmation 을 `cleanup_legacy_data_dir({token, confirmation})` 에 다시 전달.
//!    backend 는 token + confirmation 둘 다 검증.
//!
//! 두 단계 검증이 모두 backend 에서 수행되어 renderer 가 `issue_cleanup_token` → `cleanup_*`
//! 만으로 cleanup 을 trigger 하는 우회를 차단한다.

use std::sync::Arc;
use std::sync::Mutex;
use std::sync::RwLock;
use std::time::{Duration, Instant};

use serde::Deserialize;
use uuid::Uuid;

use crate::errors::{AppError, AppResult};
use crate::paths::{self, CleanupReport, MigrationOutcome, MigrationStatusView};

/// 발급한 토큰의 TTL. 모달 확인 → cleanup 호출까지 사용자의 RTT 를 충분히 덮으면서
/// 토큰 leak 시 노출 창은 짧게 유지.
const TOKEN_TTL: Duration = Duration::from_secs(60);

#[derive(Debug, Clone)]
struct PendingToken {
    value: String,
    expires_at: Instant,
}

/// 발급/검증을 직렬화하는 mutex. 동시에 두 토큰이 살아있지 않게 — 마지막 발급만 유효.
#[derive(Default)]
pub struct CleanupTokenStore {
    inner: Mutex<Option<PendingToken>>,
}

impl CleanupTokenStore {
    fn issue(&self) -> String {
        let value = Uuid::new_v4().to_string();
        let mut guard = self.inner.lock().expect("cleanup token lock poisoned");
        *guard = Some(PendingToken {
            value: value.clone(),
            expires_at: Instant::now() + TOKEN_TTL,
        });
        value
    }

    fn take_if_valid(&self, candidate: &str) -> bool {
        let mut guard = self.inner.lock().expect("cleanup token lock poisoned");
        let Some(pending) = guard.take() else {
            return false;
        };
        // 만료된 토큰이면 사용 불가. 어쨌든 take() 했으므로 invalidate 됨.
        if Instant::now() >= pending.expires_at {
            return false;
        }
        // 상수시간 비교 — uuid v4 이므로 substring 공격 사실상 무관하지만 방어선.
        constant_time_eq(pending.value.as_bytes(), candidate.as_bytes())
    }
}

fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff: u8 = 0;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// `outcome.legacy_dir` 의 마지막 경로 component. backend 가 보내고 backend 가 검증.
/// renderer 가 보내는 값을 신뢰하지 않는다. base name 추출 불가면 `None`.
fn expected_confirmation_phrase(outcome: &MigrationOutcome) -> Option<String> {
    outcome
        .legacy_dir
        .as_ref()
        .and_then(|p| p.file_name())
        .and_then(|n| n.to_str())
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct IssueCleanupTokenRequest {
    pub confirmation: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CleanupLegacyRequest {
    pub token: String,
    pub confirmation: String,
}

pub type MigrationOutcomeState = Arc<RwLock<MigrationOutcome>>;
pub type CleanupTokenState = Arc<CleanupTokenStore>;

#[tauri::command]
pub async fn get_legacy_migration_status(
    outcome: tauri::State<'_, MigrationOutcomeState>,
) -> AppResult<MigrationStatusView> {
    let guard = outcome
        .read()
        .map_err(|_| AppError::internal("migration outcome lock poisoned"))?;
    Ok(paths::migration_status_view(&guard))
}

/// UI 가 사용자에게 보여줄 phrase. backend 가 권위적인 출처이며, renderer 가 임의 phrase 를
/// 주입할 수 없게 한다. legacy_dir 가 없으면 `None` 을 반환 — UI 는 banner 자체를 숨긴다.
#[tauri::command]
pub async fn cleanup_confirmation_phrase(
    outcome: tauri::State<'_, MigrationOutcomeState>,
) -> AppResult<Option<String>> {
    let guard = outcome
        .read()
        .map_err(|_| AppError::internal("migration outcome lock poisoned"))?;
    Ok(expected_confirmation_phrase(&guard))
}

/// 사용자가 settings UI 의 confirmation 입력박스에 phrase 를 직접 타이핑한 직후 호출.
/// backend 가 phrase 일치를 검증해 user-intent 를 강제. 통과해야 1회용 토큰 발급.
#[tauri::command]
pub async fn issue_cleanup_token(
    outcome: tauri::State<'_, MigrationOutcomeState>,
    tokens: tauri::State<'_, CleanupTokenState>,
    request: IssueCleanupTokenRequest,
) -> AppResult<String> {
    {
        let guard = outcome
            .read()
            .map_err(|_| AppError::internal("migration outcome lock poisoned"))?;
        let expected = expected_confirmation_phrase(&guard).ok_or_else(|| {
            AppError::internal("issue_cleanup_token: no legacy_dir to confirm against")
        })?;
        if !confirmation_matches(&expected, &request.confirmation) {
            return Err(AppError::internal(
                "issue_cleanup_token: confirmation phrase mismatch",
            ));
        }
    }
    Ok(tokens.issue())
}

#[tauri::command]
pub async fn cleanup_legacy_data_dir(
    outcome: tauri::State<'_, MigrationOutcomeState>,
    tokens: tauri::State<'_, CleanupTokenState>,
    request: CleanupLegacyRequest,
) -> AppResult<CleanupReport> {
    // defense in depth — confirmation 을 cleanup 시점에 다시 검증.
    {
        let guard = outcome
            .read()
            .map_err(|_| AppError::internal("migration outcome lock poisoned"))?;
        let expected = expected_confirmation_phrase(&guard).ok_or_else(|| {
            AppError::internal("cleanup_legacy_data_dir: no legacy_dir to confirm against")
        })?;
        if !confirmation_matches(&expected, &request.confirmation) {
            return Err(AppError::internal(
                "cleanup_legacy_data_dir: confirmation phrase mismatch",
            ));
        }
    }
    if !tokens.take_if_valid(&request.token) {
        return Err(AppError::internal(
            "cleanup_legacy_data_dir: invalid or expired token",
        ));
    }
    // snapshot outcome under read lock so the actual fs work runs without holding the lock.
    let snapshot = {
        let guard = outcome
            .read()
            .map_err(|_| AppError::internal("migration outcome lock poisoned"))?;
        guard.clone()
    };
    let report = paths::cleanup_legacy(&snapshot)?;
    if let CleanupReport::Completed { .. } = &report {
        let mut guard = outcome
            .write()
            .map_err(|_| AppError::internal("migration outcome lock poisoned"))?;
        guard.legacy_has_our_files = false;
        guard.legacy_cleanable = false;
    }
    Ok(report)
}

/// trim 후 정확히 같아야 한다. backend 가 권위. constant-time 비교.
fn confirmation_matches(expected: &str, candidate: &str) -> bool {
    constant_time_eq(expected.trim().as_bytes(), candidate.trim().as_bytes())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    fn outcome_with_legacy(legacy_dir: Option<PathBuf>) -> MigrationOutcome {
        MigrationOutcome {
            new_dir: PathBuf::from("/tmp/new"),
            legacy_dir,
            copied: vec![],
            verified: true,
            legacy_has_our_files: true,
            legacy_cleanable: true,
            verify_error: None,
        }
    }

    #[test]
    fn token_is_one_shot() {
        let store = CleanupTokenStore::default();
        let t = store.issue();
        assert!(store.take_if_valid(&t));
        // 두 번째 호출은 거부.
        assert!(!store.take_if_valid(&t));
    }

    #[test]
    fn issuing_again_invalidates_previous_token() {
        let store = CleanupTokenStore::default();
        let t1 = store.issue();
        let _t2 = store.issue();
        // t1 은 더 이상 유효하지 않음.
        assert!(!store.take_if_valid(&t1));
    }

    #[test]
    fn random_token_rejected() {
        let store = CleanupTokenStore::default();
        let _ = store.issue();
        assert!(!store.take_if_valid("not-the-real-token"));
    }

    #[test]
    fn empty_token_rejected_without_issuing() {
        let store = CleanupTokenStore::default();
        assert!(!store.take_if_valid(""));
    }

    #[test]
    fn constant_time_eq_matches_eq() {
        assert!(constant_time_eq(b"abc", b"abc"));
        assert!(!constant_time_eq(b"abc", b"abd"));
        assert!(!constant_time_eq(b"abc", b"abcd"));
        assert!(constant_time_eq(b"", b""));
    }

    #[test]
    fn expected_confirmation_phrase_is_legacy_base_name() {
        let o = outcome_with_legacy(Some(PathBuf::from(
            "/Users/me/Library/Application Support/HyTranslate Mac",
        )));
        assert_eq!(
            expected_confirmation_phrase(&o).as_deref(),
            Some("HyTranslate Mac")
        );
    }

    #[test]
    fn expected_confirmation_phrase_is_none_without_legacy() {
        let o = outcome_with_legacy(None);
        assert!(expected_confirmation_phrase(&o).is_none());
    }

    /// Major 1 v3 회귀 — confirmation phrase 가 backend expected 와 일치해야만 통과.
    /// trim 은 허용하지만 그 외 mismatch 는 거부.
    #[test]
    fn confirmation_matches_exact_after_trim() {
        assert!(confirmation_matches("HyTranslate Mac", "HyTranslate Mac"));
        assert!(confirmation_matches(
            "HyTranslate Mac",
            "  HyTranslate Mac  "
        ));
        assert!(!confirmation_matches("HyTranslate Mac", "hytranslate mac"));
        assert!(!confirmation_matches("HyTranslate Mac", "HyTranslate"));
        assert!(!confirmation_matches("HyTranslate Mac", ""));
    }
}
