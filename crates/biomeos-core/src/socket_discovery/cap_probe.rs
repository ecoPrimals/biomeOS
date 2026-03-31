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
