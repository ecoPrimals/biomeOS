// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use anyhow::Result;
use biomeos_types::SystemPaths;
use std::path::{Path, PathBuf};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::Duration;
use tracing::debug;

pub(super) fn family_id_for_sockets() -> String {
    std::env::var("FAMILY_ID")
        .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
        .unwrap_or_else(|_| "default".to_string())
}

fn is_ai_capability(name: &str) -> bool {
    name == "ai" || name.starts_with("ai.") || matches!(name, "mcp" | "ml" | "assistance")
}

/// Discover the Unix socket path for a primal that advertises AI-related capabilities.
pub(super) async fn discover_ai_socket_path() -> Result<PathBuf> {
    let paths = SystemPaths::new_lazy();
    let runtime = paths.runtime_dir().to_path_buf();
    let family = family_id_for_sockets();
    let suffix = format!("-{family}.sock");

    let mut candidates: Vec<PathBuf> = std::fs::read_dir(&runtime)
        .map(|entries| {
            entries
                .flatten()
                .map(|e| e.path())
                .filter(|p| {
                    p.file_name()
                        .and_then(|n| n.to_str())
                        .is_some_and(|n| n.ends_with(&suffix))
                })
                .collect()
        })
        .unwrap_or_default();
    candidates.sort();

    for socket_path in candidates {
        let caps = probe_capabilities_list(&socket_path).await;
        if caps.iter().any(|c| is_ai_capability(c)) {
            return Ok(socket_path);
        }
    }

    anyhow::bail!(
        "no primal advertising AI capability found under {}",
        runtime.display()
    )
}

async fn probe_capabilities_list(socket_path: &Path) -> Vec<String> {
    let socket_str = socket_path.to_string_lossy();
    let stream =
        match tokio::time::timeout(Duration::from_millis(500), UnixStream::connect(socket_path))
            .await
        {
            Ok(Ok(s)) => s,
            Ok(Err(e)) => {
                debug!("AI probe {}: connect failed: {}", socket_str, e);
                return vec![];
            }
            Err(_) => {
                debug!("AI probe {}: connect timed out", socket_str);
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
        Duration::from_millis(500),
        reader.read_line(&mut response_line),
    )
    .await
    {
        Ok(Ok(n)) if n > 0 => {}
        _ => return vec![],
    }

    let resp: serde_json::Value = serde_json::from_str(&response_line).unwrap_or_default();
    extract_capabilities_from_list_response(&resp)
}

/// Parse capability names from a JSON-RPC response, handling all 5 ecosystem wire formats.
pub(super) fn extract_capabilities_from_list_response(resp: &serde_json::Value) -> Vec<String> {
    let result = &resp["result"];

    if let Some(caps) = result["capabilities"].as_array() {
        let parsed = extract_names_from_array(caps);
        if !parsed.is_empty() {
            return parsed;
        }
    }
    if let Some(arr) = result.as_array() {
        let parsed = extract_names_from_array(arr);
        if !parsed.is_empty() {
            return parsed;
        }
    }
    if let Some(caps) = result["methods"].as_array() {
        let parsed: Vec<String> = caps
            .iter()
            .filter_map(|c| c.as_str().map(String::from))
            .collect();
        if !parsed.is_empty() {
            return parsed;
        }
    }
    if let Some(info) = result["method_info"].as_array() {
        let parsed: Vec<String> = info
            .iter()
            .filter_map(|item| item["name"].as_str().map(String::from))
            .collect();
        if !parsed.is_empty() {
            return parsed;
        }
    }
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
                        if let Some(method_name) = m.as_str() {
                            names.push(format!("{cap_type}.{method_name}"));
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
    vec![]
}

fn extract_names_from_array(arr: &[serde_json::Value]) -> Vec<String> {
    arr.iter()
        .filter_map(|v| {
            if let Some(s) = v.as_str() {
                return Some(s.to_owned());
            }
            v.get("method")
                .or_else(|| v.get("name"))
                .and_then(serde_json::Value::as_str)
                .map(String::from)
        })
        .collect()
}
