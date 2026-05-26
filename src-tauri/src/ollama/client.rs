//! Ollama HTTP streaming client (`POST /api/generate`, `stream: true`).
//!
//! 핵심 책임:
//! - chunked NDJSON 응답을 한 줄씩 파싱한다.
//! - UTF-8 partial codepoint 가 chunk 경계를 넘어가도 깨진 문자를 emit 하지 않는다.
//! - 취소 토큰을 chunk 사이마다 확인한다.

use std::time::Duration;

use bytes::Bytes;
use futures_util::StreamExt;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio_util::sync::CancellationToken;

use crate::errors::{AppError, AppResult};
use crate::language::SourceLanguage;
use crate::ollama::prompt::{build_prompt, num_predict_for};

const GENERATE_PATH: &str = "/api/generate";

#[derive(Debug, Clone)]
pub struct OllamaClient {
    http: Client,
    base_url: String,
}

#[derive(Debug, Serialize)]
struct GenerateRequest<'a> {
    model: &'a str,
    prompt: String,
    stream: bool,
    options: GenerateOptions,
}

#[derive(Debug, Serialize)]
struct GenerateOptions {
    temperature: f32,
    top_p: f32,
    num_predict: u32,
}

#[derive(Debug, Deserialize)]
struct GenerateChunk {
    #[serde(default)]
    response: String,
    #[serde(default)]
    done: bool,
}

/// chunk callback 결과. `Stop` 이면 worker 는 더 이상 chunk 를 emit 하지 않는다.
pub enum ChunkFlow {
    Continue,
    Stop,
}

impl OllamaClient {
    pub fn new(endpoint: impl Into<String>) -> AppResult<Self> {
        let http = Client::builder()
            .pool_idle_timeout(Duration::from_secs(30))
            .build()
            .map_err(AppError::from)?;
        Ok(Self {
            http,
            base_url: endpoint.into(),
        })
    }

    /// `model` 로 streaming generate 호출. `on_chunk` 는 `done:false` 인 chunk 의
    /// `response` delta 마다 호출된다. `cancel` 이 cancelled 이면 worker 는 즉시 중단한다.
    ///
    /// 반환값은 누적된 full text. 호출자가 `duration_ms` 와 함께 `translation:completed` 를 emit 한다.
    pub async fn generate_stream<F>(
        &self,
        model: &str,
        source_language: SourceLanguage,
        source_text: &str,
        cancel: &CancellationToken,
        mut on_chunk: F,
    ) -> AppResult<String>
    where
        F: FnMut(&str) -> ChunkFlow,
    {
        let body = GenerateRequest {
            model,
            prompt: build_prompt(source_language, source_text),
            stream: true,
            options: GenerateOptions {
                temperature: 0.3,
                top_p: 0.9,
                num_predict: num_predict_for(source_text),
            },
        };

        let url = format!("{}{}", self.base_url.trim_end_matches('/'), GENERATE_PATH);
        let response = tokio::select! {
            _ = cancel.cancelled() => return Err(AppError::Cancelled),
            res = self.http.post(&url).json(&body).send() => res.map_err(AppError::from)?,
        };

        // status별 의미 보존: 404 는 모델 미설치, 그 외는 일반 reqwest 매핑.
        let response = match response.error_for_status_ref() {
            Ok(_) => response,
            Err(err) => {
                if err.status() == Some(reqwest::StatusCode::NOT_FOUND) {
                    return Err(AppError::ModelMissing {
                        model: model.to_string(),
                    });
                }
                return Err(AppError::from(err));
            }
        };
        let mut stream = response.bytes_stream();
        let mut line_buf = LineBuffer::new();
        let mut full_text = String::new();

        loop {
            tokio::select! {
                biased;
                _ = cancel.cancelled() => return Err(AppError::Cancelled),
                next = stream.next() => {
                    let Some(item) = next else { break };
                    let bytes: Bytes = item.map_err(AppError::from)?;
                    line_buf.feed(&bytes);
                    while let Some(line) = line_buf.next_line() {
                        if line.is_empty() { continue; }
                        let chunk: GenerateChunk = serde_json::from_str(&line)
                            .map_err(|e| AppError::internal(format!("ollama chunk parse: {e}")))?;
                        if !chunk.response.is_empty() {
                            full_text.push_str(&chunk.response);
                            if matches!(on_chunk(&chunk.response), ChunkFlow::Stop) {
                                return Err(AppError::Cancelled);
                            }
                        }
                        if chunk.done {
                            // `done:true` chunk 가 곧 정상 종료. 잔여 partial line 은
                            // Ollama 가 done 이후 더 보내지 않으므로 무시 가능.
                            return Ok(full_text);
                        }
                    }
                }
            }
        }

        // EOF 인데 `done:true` 를 보지 못함 → incomplete stream.
        // 부분 번역을 정상 완료로 보고하지 않도록 명시적 에러를 반환한다.
        Err(AppError::Internal {
            message: if line_buf.is_empty() {
                "ollama stream ended before `done: true`".to_string()
            } else {
                "ollama stream ended with trailing partial line".to_string()
            },
        })
    }
}

/// NDJSON line buffer with UTF-8 boundary safety.
///
/// `feed` 는 임의 바이트를 받아들이고, `next_line` 은 newline 단위로 완성된 라인만 반환한다.
/// 라인 내부 partial UTF-8 은 valid UTF-8 prefix 만 가져가고 잔여 바이트는 다음 chunk 와 합친다.
struct LineBuffer {
    buf: Vec<u8>,
}

impl LineBuffer {
    fn new() -> Self {
        Self {
            buf: Vec::with_capacity(512),
        }
    }

    fn feed(&mut self, bytes: &[u8]) {
        self.buf.extend_from_slice(bytes);
    }

    fn is_empty(&self) -> bool {
        self.buf.is_empty()
    }

    fn next_line(&mut self) -> Option<String> {
        let pos = self.buf.iter().position(|b| *b == b'\n')?;
        let mut line: Vec<u8> = self.buf.drain(..=pos).collect();
        line.pop();
        if line.last() == Some(&b'\r') {
            line.pop();
        }
        Some(String::from_utf8_lossy(&line).into_owned())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use wiremock::matchers::{method, path};
    use wiremock::{Mock, MockServer, ResponseTemplate};

    fn ndjson_body() -> String {
        let mut buf = String::new();
        for piece in ["Hello", ", ", "world", "!"] {
            buf.push_str(&format!(
                r#"{{"model":"x","response":"{piece}","done":false}}"#
            ));
            buf.push('\n');
        }
        buf.push_str(r#"{"model":"x","response":"","done":true,"done_reason":"stop"}"#);
        buf.push('\n');
        buf
    }

    #[tokio::test]
    async fn accumulates_chunks_and_returns_full_text() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(GENERATE_PATH))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(ndjson_body(), "application/x-ndjson"),
            )
            .mount(&server)
            .await;

        let client = OllamaClient::new(server.uri()).unwrap();
        let cancel = CancellationToken::new();
        let mut seen: Vec<String> = Vec::new();
        let result = client
            .generate_stream(
                "test-model",
                SourceLanguage::Korean,
                "안녕",
                &cancel,
                |delta| {
                    seen.push(delta.to_string());
                    ChunkFlow::Continue
                },
            )
            .await
            .unwrap();

        assert_eq!(result, "Hello, world!");
        assert_eq!(seen, vec!["Hello", ", ", "world", "!"]);
    }

    #[tokio::test]
    async fn cancellation_returns_cancelled_error() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(GENERATE_PATH))
            .respond_with(
                ResponseTemplate::new(200).set_body_raw(ndjson_body(), "application/x-ndjson"),
            )
            .mount(&server)
            .await;

        let client = OllamaClient::new(server.uri()).unwrap();
        let cancel = CancellationToken::new();
        let token_for_cb = cancel.clone();
        let result = client
            .generate_stream(
                "test-model",
                SourceLanguage::Korean,
                "안녕",
                &cancel,
                |_delta| {
                    token_for_cb.cancel();
                    ChunkFlow::Stop
                },
            )
            .await;
        assert!(matches!(result, Err(AppError::Cancelled)));
    }

    #[tokio::test]
    async fn http_error_is_mapped_to_internal() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(GENERATE_PATH))
            .respond_with(ResponseTemplate::new(500))
            .mount(&server)
            .await;

        let client = OllamaClient::new(server.uri()).unwrap();
        let cancel = CancellationToken::new();
        let result = client
            .generate_stream(
                "test-model",
                SourceLanguage::Korean,
                "안녕",
                &cancel,
                |_| ChunkFlow::Continue,
            )
            .await;
        assert!(matches!(result, Err(AppError::Internal { .. })));
    }

    #[tokio::test]
    async fn http_404_maps_to_model_missing() {
        let server = MockServer::start().await;
        Mock::given(method("POST"))
            .and(path(GENERATE_PATH))
            .respond_with(ResponseTemplate::new(404).set_body_string("model not found"))
            .mount(&server)
            .await;

        let client = OllamaClient::new(server.uri()).unwrap();
        let cancel = CancellationToken::new();
        let result = client
            .generate_stream(
                "ghost-model",
                SourceLanguage::Korean,
                "안녕",
                &cancel,
                |_| ChunkFlow::Continue,
            )
            .await;
        match result {
            Err(AppError::ModelMissing { model }) => assert_eq!(model, "ghost-model"),
            other => panic!("expected ModelMissing, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn stream_without_done_true_returns_error() {
        let server = MockServer::start().await;
        // `done:true` chunk 없이 끊긴 응답.
        let body = "{\"model\":\"x\",\"response\":\"hi\",\"done\":false}\n".to_string();
        Mock::given(method("POST"))
            .and(path(GENERATE_PATH))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/x-ndjson"))
            .mount(&server)
            .await;

        let client = OllamaClient::new(server.uri()).unwrap();
        let cancel = CancellationToken::new();
        let result = client
            .generate_stream(
                "test-model",
                SourceLanguage::Korean,
                "안녕",
                &cancel,
                |_| ChunkFlow::Continue,
            )
            .await;
        assert!(matches!(result, Err(AppError::Internal { .. })));
    }

    #[tokio::test]
    async fn stream_with_trailing_partial_line_returns_error() {
        let server = MockServer::start().await;
        // `done` 도 보지 못하고 newline 없는 잔여 데이터로 끝남.
        let body = "{\"model\":\"x\",\"response\":\"hi\",\"done\":false}\n{\"model\":\"x\",\"response\":\"oops".to_string();
        Mock::given(method("POST"))
            .and(path(GENERATE_PATH))
            .respond_with(ResponseTemplate::new(200).set_body_raw(body, "application/x-ndjson"))
            .mount(&server)
            .await;

        let client = OllamaClient::new(server.uri()).unwrap();
        let cancel = CancellationToken::new();
        let result = client
            .generate_stream(
                "test-model",
                SourceLanguage::Korean,
                "안녕",
                &cancel,
                |_| ChunkFlow::Continue,
            )
            .await;
        assert!(matches!(result, Err(AppError::Internal { .. })));
    }

    #[test]
    fn line_buffer_handles_split_lines() {
        let mut buf = LineBuffer::new();
        buf.feed(b"{\"response\":\"hi");
        assert!(buf.next_line().is_none());
        buf.feed(b"\",\"done\":false}\n");
        assert_eq!(
            buf.next_line().unwrap(),
            r#"{"response":"hi","done":false}"#
        );
    }
}
