//! Ollama endpoint allowlist (PRD §12, `.claude/rules/security.md`).
//!
//! Translation 요청은 오직 localhost 로만 나가야 한다. 사용자가 endpoint 를 바꾸어도
//! host 는 loopback 으로 강제. 포트/경로는 자유.

pub fn is_endpoint_allowed(url: &str) -> bool {
    let Ok(parsed) = reqwest::Url::parse(url) else {
        return false;
    };
    // Minor 3 — Ollama 기본은 http://localhost:11434. HTTPS loopback 은 v1 흐름이 아니므로
    // 의도되지 않은 입력으로 본다. 외부 송신 차단 원칙 강화.
    if parsed.scheme() != "http" {
        return false;
    }
    matches!(
        parsed.host_str(),
        Some("localhost") | Some("127.0.0.1") | Some("::1") | Some("[::1]")
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn localhost_variants_allowed() {
        assert!(is_endpoint_allowed("http://localhost:11434"));
        assert!(is_endpoint_allowed("http://127.0.0.1:11434"));
        assert!(is_endpoint_allowed("http://[::1]:11434"));
        assert!(is_endpoint_allowed("http://localhost"));
    }

    /// Minor 3 회귀 — Ollama 는 plaintext http only. HTTPS loopback 도 거부한다.
    #[test]
    fn https_loopback_rejected() {
        assert!(!is_endpoint_allowed("https://localhost:11434"));
        assert!(!is_endpoint_allowed("https://127.0.0.1:11434"));
    }

    #[test]
    fn non_loopback_hosts_rejected() {
        assert!(!is_endpoint_allowed("http://example.com:11434"));
        assert!(!is_endpoint_allowed("http://192.168.1.10:11434"));
        assert!(!is_endpoint_allowed("http://10.0.0.5:11434"));
        // 0.0.0.0 은 loopback 이 아님.
        assert!(!is_endpoint_allowed("http://0.0.0.0:11434"));
    }

    #[test]
    fn non_http_schemes_rejected() {
        assert!(!is_endpoint_allowed("file:///etc/passwd"));
        assert!(!is_endpoint_allowed("ftp://localhost"));
    }

    #[test]
    fn malformed_urls_rejected() {
        assert!(!is_endpoint_allowed("not a url"));
        assert!(!is_endpoint_allowed(""));
        assert!(!is_endpoint_allowed("localhost:11434"));
    }
}
