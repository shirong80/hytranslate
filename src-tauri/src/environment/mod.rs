//! 온보딩 환경 감지. macOS 시스템 정보로부터 모델 추천을 산출한다.
//!
//! 외부 명령 (`sw_vers`, `sysctl`) 호출은 module-local helper 로 분리. 단위 테스트는
//! pure logic (추천 규칙, 메모리 임계값) 만 검증하고 system probe 결과는 실제 macOS
//! 에서 manual 로 검증한다.

use std::process::Command;

use serde::Serialize;

use crate::errors::AppResult;
use crate::ollama::{MODEL_HY_MT2_1_8B, MODEL_HY_MT2_7B};

/// 12 GB. 미만이면 1.8B 모델 추천 (PRD §6.1 step 2/4).
pub const LIGHT_MODEL_RAM_THRESHOLD_GB: u32 = 12;

/// 13.0. 미만이면 unsupported macOS — 온보딩에서 안내 (PRD §18 macOS 13+).
pub const MIN_MACOS_MAJOR: u32 = 13;

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum Arch {
    AppleSilicon,
    Intel,
    Unknown,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentReport {
    /// macOS 버전 문자열 (e.g., "14.4.1").
    pub macos_version: String,
    pub macos_major: u32,
    pub macos_supported: bool,
    pub arch: Arch,
    pub total_memory_gb: u32,
    /// PRD §6.1 step 4 — 12 GB 미만이면 1.8B 추천.
    pub recommended_model: String,
}

pub fn recommended_model_for(total_memory_gb: u32) -> &'static str {
    if total_memory_gb < LIGHT_MODEL_RAM_THRESHOLD_GB {
        MODEL_HY_MT2_1_8B
    } else {
        MODEL_HY_MT2_7B
    }
}

pub fn parse_macos_major(version: &str) -> u32 {
    version
        .split('.')
        .next()
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(0)
}

/// 실제 시스템에서 환경을 수집. 실패 시 안전한 fallback 으로 보고.
/// 호출자는 `EnvironmentReport` 를 그대로 FE 로 전달한다.
pub fn detect() -> AppResult<EnvironmentReport> {
    let macos_version = read_macos_version().unwrap_or_else(|| "unknown".to_string());
    let macos_major = parse_macos_major(&macos_version);
    let macos_supported = macos_major >= MIN_MACOS_MAJOR;
    let arch = detect_arch();
    let total_memory_gb = read_total_memory_gb().unwrap_or(0);
    let recommended_model = recommended_model_for(total_memory_gb).to_string();

    Ok(EnvironmentReport {
        macos_version,
        macos_major,
        macos_supported,
        arch,
        total_memory_gb,
        recommended_model,
    })
}

#[cfg(target_os = "macos")]
fn read_macos_version() -> Option<String> {
    let output = Command::new("sw_vers")
        .arg("-productVersion")
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let raw = String::from_utf8(output.stdout).ok()?;
    Some(raw.trim().to_string())
}

#[cfg(not(target_os = "macos"))]
fn read_macos_version() -> Option<String> {
    None
}

#[cfg(target_os = "macos")]
fn detect_arch() -> Arch {
    // `std::env::consts::ARCH` 는 빌드 타깃 기준. macOS-only 빌드라 신뢰 가능.
    match std::env::consts::ARCH {
        "aarch64" | "arm64" => Arch::AppleSilicon,
        "x86_64" => Arch::Intel,
        _ => Arch::Unknown,
    }
}

#[cfg(not(target_os = "macos"))]
fn detect_arch() -> Arch {
    Arch::Unknown
}

#[cfg(target_os = "macos")]
fn read_total_memory_gb() -> Option<u32> {
    // `sysctl -n hw.memsize` 는 bytes 를 stdout 으로 emit.
    let output = Command::new("sysctl")
        .args(["-n", "hw.memsize"])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let raw = String::from_utf8(output.stdout).ok()?;
    let bytes: u64 = raw.trim().parse().ok()?;
    let gb = bytes / 1_073_741_824; // 1024^3
    Some(gb as u32)
}

#[cfg(not(target_os = "macos"))]
fn read_total_memory_gb() -> Option<u32> {
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_macos_major_from_dotted_version() {
        assert_eq!(parse_macos_major("13.0"), 13);
        assert_eq!(parse_macos_major("14.4.1"), 14);
        assert_eq!(parse_macos_major("15"), 15);
        assert_eq!(parse_macos_major(""), 0);
        assert_eq!(parse_macos_major("unknown"), 0);
    }

    #[test]
    fn recommends_light_model_below_threshold() {
        assert_eq!(recommended_model_for(0), MODEL_HY_MT2_1_8B);
        assert_eq!(recommended_model_for(8), MODEL_HY_MT2_1_8B);
        assert_eq!(recommended_model_for(11), MODEL_HY_MT2_1_8B);
    }

    #[test]
    fn recommends_full_model_at_or_above_threshold() {
        assert_eq!(recommended_model_for(12), MODEL_HY_MT2_7B);
        assert_eq!(recommended_model_for(16), MODEL_HY_MT2_7B);
        assert_eq!(recommended_model_for(64), MODEL_HY_MT2_7B);
    }

    #[test]
    fn threshold_constant_matches_prd() {
        assert_eq!(LIGHT_MODEL_RAM_THRESHOLD_GB, 12);
    }

    #[test]
    fn min_macos_major_matches_prd() {
        assert_eq!(MIN_MACOS_MAJOR, 13);
    }

    #[test]
    fn report_serializes_camel_case() {
        let r = EnvironmentReport {
            macos_version: "14.4.1".to_string(),
            macos_major: 14,
            macos_supported: true,
            arch: Arch::AppleSilicon,
            total_memory_gb: 16,
            recommended_model: MODEL_HY_MT2_7B.to_string(),
        };
        let json = serde_json::to_string(&r).unwrap();
        assert!(json.contains(r#""macosVersion":"14.4.1""#));
        assert!(json.contains(r#""macosMajor":14"#));
        assert!(json.contains(r#""macosSupported":true"#));
        assert!(json.contains(r#""arch":"AppleSilicon""#));
        assert!(json.contains(r#""totalMemoryGb":16"#));
        assert!(json.contains(r#""recommendedModel":"#));
    }
}
