// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Probe Unix primal sockets for `capabilities.list` (newline-delimited JSON-RPC).
//!
//! Shared by Neural API auto-discovery and other callers so capability probing is not
//! duplicated across crates.

use std::path::Path;

use biomeos_types::constants::timeouts;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tracing::{debug, warn};

/// Probe a primal UDS for advertised capability names via `capabilities.list`.
///
/// Uses a short connect/write/read timeout; returns an empty list on any failure.
pub async fn probe_unix_socket_capabilities_list(socket_path: impl AsRef<Path>) -> Vec<String> {
    let socket_path = socket_path.as_ref();
    let socket_path_str = socket_path.to_string_lossy();

    let stream =
        match tokio::time::timeout(timeouts::PROBE_TIMEOUT, UnixStream::connect(socket_path)).await
        {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                debug!("probe {}: connect failed: {}", socket_path_str, e);
                return vec![];
            }
            Err(_) => {
                debug!("probe {}: connect timed out", socket_path_str);
                return vec![];
            }
        };

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "capabilities.list",
        "id": 1
    });
    let Ok(mut request_line) = serde_json::to_string(&request) else {
        return vec![];
    };
    request_line.push('\n');

    let mut reader = BufReader::new(stream);
    let w = reader.get_mut();
    if w.write_all(request_line.as_bytes()).await.is_err() {
        return vec![];
    }
    let _ = w.flush().await;

    let mut response_line = String::new();
    match tokio::time::timeout(
        timeouts::PROBE_TIMEOUT,
        reader.read_line(&mut response_line),
    )
    .await
    {
        Ok(Ok(n)) if n > 0 => {}
        _ => return vec![],
    }

    let resp: serde_json::Value = serde_json::from_str(&response_line).unwrap_or_default();
    extract_capabilities_from_response(&resp)
}

/// Parse capability names from a `capabilities.list` JSON-RPC response.
pub fn extract_capabilities_from_response(resp: &serde_json::Value) -> Vec<String> {
    if let Some(caps) = resp["result"]["capabilities"].as_array() {
        let parsed: Vec<String> = caps
            .iter()
            .filter_map(|c| {
                c.as_str()
                    .map(String::from)
                    .or_else(|| c["name"].as_str().map(String::from))
            })
            .collect();
        if !parsed.is_empty() {
            return parsed;
        }
    }

    if let Some(caps) = resp["result"].as_array() {
        let parsed: Vec<String> = caps
            .iter()
            .filter_map(|c| c.as_str().map(String::from))
            .collect();
        if !parsed.is_empty() {
            return parsed;
        }
    }

    if let Some(caps) = resp["result"]["methods"].as_array() {
        let parsed: Vec<String> = caps
            .iter()
            .filter_map(|c| c.as_str().map(String::from))
            .collect();
        if !parsed.is_empty() {
            return parsed;
        }
    }

    warn!(
        "Unrecognized capabilities.list response shape: {}",
        serde_json::to_string(resp).unwrap_or_default()
    );
    vec![]
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::or_fun_call,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::{extract_capabilities_from_response, probe_unix_socket_capabilities_list};
    use serde_json::json;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;

    #[test]
    fn extract_capabilities_prefers_string_entries() {
        let resp = json!({
            "result": {
                "capabilities": ["alpha", "beta"]
            }
        });
        assert_eq!(
            extract_capabilities_from_response(&resp),
            vec!["alpha".to_string(), "beta".to_string()]
        );
    }

    #[test]
    fn extract_capabilities_accepts_object_entries_with_name_field() {
        let resp = json!({
            "result": {
                "capabilities": [
                    {"name": "from_object"},
                    "plain"
                ]
            }
        });
        assert_eq!(
            extract_capabilities_from_response(&resp),
            vec!["from_object".to_string(), "plain".to_string()]
        );
    }

    #[test]
    fn extract_capabilities_accepts_result_as_top_level_string_array() {
        let resp = json!({
            "result": ["rpc.a", "rpc.b"]
        });
        assert_eq!(
            extract_capabilities_from_response(&resp),
            vec!["rpc.a".to_string(), "rpc.b".to_string()]
        );
    }

    #[test]
    fn extract_capabilities_falls_back_to_methods_array() {
        let resp = json!({
            "result": {
                "capabilities": [],
                "methods": ["mesh.peers", "health.ping"]
            }
        });
        assert_eq!(
            extract_capabilities_from_response(&resp),
            vec!["mesh.peers".to_string(), "health.ping".to_string()]
        );
    }

    #[test]
    fn extract_capabilities_returns_empty_for_unrecognized_shape() {
        let resp = json!({
            "result": {
                "capabilities": [],
                "methods": []
            }
        });
        assert!(extract_capabilities_from_response(&resp).is_empty());
    }

    #[tokio::test]
    async fn probe_nonexistent_socket_returns_empty() {
        let path = std::env::temp_dir().join("biomeos_cap_probe_absent_sock.sock");
        let _ = std::fs::remove_file(&path);
        assert!(!path.exists());
        let caps = probe_unix_socket_capabilities_list(&path).await;
        assert!(caps.is_empty());
    }

    #[tokio::test]
    async fn probe_reads_capabilities_from_capabilities_list_response() {
        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("probe.sock");
        let path_for_client = sock_path.clone();
        // Bind before spawning so the client never races a not-yet-created socket path.
        let listener = UnixListener::bind(&sock_path).unwrap();

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let req_id = serde_json::from_str::<serde_json::Value>(&line)
                .ok()
                .and_then(|v| v.get("id").cloned())
                .unwrap_or(json!(1));
            let body = json!({
                "jsonrpc": "2.0",
                "id": req_id,
                "result": {
                    "capabilities": ["cap.one", "cap.two"]
                }
            });
            let mut stream = reader.into_inner();
            let line = format!("{}\n", serde_json::to_string(&body).unwrap());
            stream.write_all(line.as_bytes()).await.unwrap();
            stream.flush().await.unwrap();
        });

        let caps = probe_unix_socket_capabilities_list(&path_for_client).await;
        server.await.expect("server task");
        assert_eq!(caps, vec!["cap.one".to_string(), "cap.two".to_string()]);
    }

    #[tokio::test]
    async fn probe_invalid_json_line_returns_empty() {
        let dir = tempfile::tempdir().unwrap();
        let sock_path = dir.path().join("bad-json.sock");
        let path_for_client = sock_path.clone();
        let listener = UnixListener::bind(&sock_path).unwrap();

        let server = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let mut stream = reader.into_inner();
            stream.write_all(b"not-json\n").await.unwrap();
            stream.flush().await.unwrap();
        });

        let caps = probe_unix_socket_capabilities_list(&path_for_client).await;
        server.await.expect("server task");
        assert!(caps.is_empty());
    }

    #[test]
    fn extract_capabilities_object_entries_skip_invalid() {
        let resp = json!({
            "result": {
                "capabilities": [
                    {"not_name": "x"},
                    "keep"
                ]
            }
        });
        assert_eq!(
            extract_capabilities_from_response(&resp),
            vec!["keep".to_string()]
        );
    }

    #[test]
    fn extract_capabilities_methods_empty_then_warn_empty() {
        let resp = json!({
            "result": { "methods": [] }
        });
        assert!(extract_capabilities_from_response(&resp).is_empty());
    }
}
