// SPDX-License-Identifier: AGPL-3.0-or-later
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

/// Probe a primal UDS for advertised capability names.
///
/// Tries `capabilities.list` first, then `capability.list` (singular) as a
/// fallback — primals in the ecosystem use both method names. Uses a short
/// connect/write/read timeout; returns an empty list on any failure.
pub async fn probe_unix_socket_capabilities_list(socket_path: impl AsRef<Path>) -> Vec<String> {
    let socket_path = socket_path.as_ref();
    let socket_path_str = socket_path.to_string_lossy();

    // Try plural first (biomeOS convention), then singular (primal convention).
    for method in &["capabilities.list", "capability.list"] {
        let stream =
            match tokio::time::timeout(timeouts::PROBE_TIMEOUT, UnixStream::connect(socket_path))
                .await
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
            "method": *method,
            "id": 1
        });
        let Ok(mut request_line) = serde_json::to_string(&request) else {
            return vec![];
        };
        request_line.push('\n');

        let mut reader = BufReader::new(stream);
        let w = reader.get_mut();
        if w.write_all(request_line.as_bytes()).await.is_err() {
            continue;
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
            _ => continue,
        }

        let resp: serde_json::Value = match serde_json::from_str(response_line.trim()) {
            Ok(v) => v,
            Err(e) => {
                debug!(
                    "probe {}: {} returned invalid JSON ({} bytes): {}",
                    socket_path_str,
                    method,
                    response_line.len(),
                    e
                );
                continue;
            }
        };

        if resp.get("error").is_some() {
            debug!(
                "probe {}: {} returned error, trying next method",
                socket_path_str, method
            );
            continue;
        }

        let caps = extract_capabilities_from_response(&resp);
        if !caps.is_empty() {
            return caps;
        }
    }

    vec![]
}

/// Parse capability names from a `capabilities.list` / `capability.list` JSON-RPC response.
///
/// Handles all 5 ecosystem wire formats (aligned with primalSpring's
/// `extract_capability_names` reference parser):
///
/// - **Format A** — `result` is a flat string array: `["crypto.sign", ...]`
/// - **Format B** — `result` is an object array: `[{"method": "crypto.sign"}]`
///   (also accepts `{"name": ...}` for `result.capabilities` sub-arrays)
/// - **Format C** — `result.method_info`: `[{"name": "crypto.sign", ...}]`
/// - **Format D** — `result.semantic_mappings`: `{"crypto": {"sign": {}}}`
/// - **Format E** — `result.provided_capabilities`: `[{"type": "security", "methods": [...]}]`
///   (BearDog wire format — each entry is a capability group with typed methods)
///
/// Legacy shapes (`result.capabilities`, `result.methods`) are tried first
/// for backwards compatibility.
pub fn extract_capabilities_from_response(resp: &serde_json::Value) -> Vec<String> {
    let result = &resp["result"];

    // --- Legacy / shortcut: result.capabilities (string or {name} objects) ---
    if let Some(caps) = result["capabilities"].as_array() {
        let parsed = extract_from_array(caps);
        if !parsed.is_empty() {
            return parsed;
        }
    }

    // --- Format A / B: result is a top-level array ---
    if let Some(arr) = result.as_array() {
        let parsed = extract_from_array(arr);
        if !parsed.is_empty() {
            return parsed;
        }
    }

    // --- Legacy / L2: result.methods (strings or {name}/{method} objects) ---
    if let Some(caps) = result["methods"].as_array() {
        let parsed = extract_from_array(caps);
        if !parsed.is_empty() {
            return parsed;
        }
    }

    // --- Format C: result.method_info [{name: "..."}] ---
    if let Some(info) = result["method_info"].as_array() {
        let parsed: Vec<String> = info
            .iter()
            .filter_map(|item| item["name"].as_str().map(String::from))
            .collect();
        if !parsed.is_empty() {
            return parsed;
        }
    }

    // --- Format D: result.semantic_mappings {domain: {verb: ...}} ---
    if let Some(domains) = result["semantic_mappings"].as_object() {
        let parsed: Vec<String> = domains
            .iter()
            .flat_map(|(domain, verbs)| {
                if let Some(verb_map) = verbs.as_object() {
                    verb_map
                        .keys()
                        .map(|verb| format!("{domain}.{verb}"))
                        .collect::<Vec<_>>()
                } else {
                    vec![domain.clone()]
                }
            })
            .collect();
        if !parsed.is_empty() {
            return parsed;
        }
    }

    // --- Format E: result.provided_capabilities [{type: "security", methods: [...]}] ---
    // BearDog wire format: each entry is a capability group with a type and method list.
    // Emits both the group type ("security") and qualified methods ("security.encrypt").
    if let Some(groups) = result["provided_capabilities"].as_array() {
        let parsed: Vec<String> = groups
            .iter()
            .flat_map(|group| {
                let cap_type = group["type"].as_str().unwrap_or_default();
                let mut names = Vec::new();
                if !cap_type.is_empty() {
                    names.push(cap_type.to_string());
                }
                if let Some(methods) = group["methods"].as_array() {
                    for m in methods {
                        let method_name = m.as_str().map(String::from).or_else(|| {
                            m.get("name")
                                .or_else(|| m.get("method"))
                                .and_then(serde_json::Value::as_str)
                                .map(String::from)
                        });
                        if let Some(name) = method_name {
                            names.push(format!("{cap_type}.{name}"));
                        }
                    }
                }
                names
            })
            .collect();
        if !parsed.is_empty() {
            return parsed;
        }
    }

    warn!(
        "Unrecognized capabilities.list response shape: {}",
        serde_json::to_string(resp).unwrap_or_else(|e| format!("<serialize error: {e}>"))
    );
    vec![]
}

/// Extract names from a JSON array (Formats A and B).
fn extract_from_array(arr: &[serde_json::Value]) -> Vec<String> {
    arr.iter()
        .filter_map(|v| {
            // Format A: bare strings
            if let Some(s) = v.as_str() {
                return Some(s.to_owned());
            }
            // Format B: {"method": "name"} or {"name": "name"} objects
            v.get("method")
                .or_else(|| v.get("name"))
                .and_then(serde_json::Value::as_str)
                .map(String::from)
        })
        .collect()
}

#[cfg(test)]
#[path = "cap_probe_tests.rs"]
mod tests;
