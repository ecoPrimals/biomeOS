// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Runtime capability cache and socket probes for primal discovery.

use std::collections::HashMap;
use std::path::Path;
use std::sync::{OnceLock, RwLock};
#[cfg(unix)]
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
#[cfg(unix)]
use tokio::net::UnixStream;

use crate::PrimalCapability;

/// Runtime capability hints learned from live socket probes (`capability.list`).
static RUNTIME_CAPABILITY_HINTS: OnceLock<RwLock<HashMap<String, PrimalCapability>>> =
    OnceLock::new();

fn runtime_capability_cache() -> &'static RwLock<HashMap<String, PrimalCapability>> {
    RUNTIME_CAPABILITY_HINTS.get_or_init(|| RwLock::new(HashMap::new()))
}

fn normalized_primal_key(name: &str) -> String {
    name.to_lowercase()
}

/// Record a capability learned from socket discovery for later bootstrap lookups.
pub(crate) fn store_runtime_capability_hint(name: &str, capability: PrimalCapability) {
    let key = normalized_primal_key(name);
    if let Ok(mut cache) = runtime_capability_cache().write() {
        cache.insert(key, capability);
    }
}

fn cached_capability_hint(name: &str) -> Option<PrimalCapability> {
    runtime_capability_cache()
        .read()
        .ok()
        .and_then(|cache| cache.get(&normalized_primal_key(name)).cloned())
}

/// Resolve a primal's primary capability: runtime cache first, static hints last.
///
/// Prefer capability-based APIs when the caller can express intent by capability
/// rather than primal name.
pub(crate) fn bootstrap_capability_hint(name: &str) -> PrimalCapability {
    if let Some(cap) = cached_capability_hint(name) {
        return cap;
    }
    static_bootstrap_capability_hint(name)
}

/// Last-resort fallback when runtime discovery has not yet learned this primal's capability.
///
/// Static name→capability hints exist only for cold-start bootstrap before any socket probe
/// has succeeded. Do not add new entries here — primals should self-report via `capability.list`.
fn static_bootstrap_capability_hint(name: &str) -> PrimalCapability {
    match name.to_lowercase().as_str() {
        biomeos_types::primal_names::BEARDOG => PrimalCapability::encryption(),
        biomeos_types::primal_names::SONGBIRD => PrimalCapability::networking(),
        biomeos_types::primal_names::TOADSTOOL => PrimalCapability::compute(),
        biomeos_types::primal_names::NESTGATE => PrimalCapability::storage(),
        biomeos_types::primal_names::SQUIRREL => PrimalCapability::ai(),
        biomeos_types::primal_names::WETSPRING | biomeos_types::primal_names::NEURALSPRING => {
            PrimalCapability::science()
        }
        _ => PrimalCapability::custom(name),
    }
}

/// Probe a healthy UDS for `capabilities.list` / `capability.list` and map the first entry.
#[cfg(unix)]
pub(crate) async fn probe_primary_capability(socket_path: &Path) -> Option<PrimalCapability> {
    use biomeos_types::constants::timeouts;

    for method in &["capabilities.list", "capability.list"] {
        let stream = match tokio::time::timeout(timeouts::PROBE_TIMEOUT, UnixStream::connect(socket_path))
            .await
        {
            Ok(Ok(stream)) => stream,
            _ => return None,
        };

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": *method,
            "id": 1
        });
        let Ok(mut request_line) = serde_json::to_string(&request) else {
            continue;
        };
        request_line.push('\n');

        let mut reader = BufReader::new(stream);
        let writer = reader.get_mut();
        if writer.write_all(request_line.as_bytes()).await.is_err() {
            continue;
        }
        let _ = writer.flush().await;

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
            Err(_) => continue,
        };
        if resp.get("error").is_some() {
            continue;
        }

        let caps = extract_advertised_capability_names(&resp);
        if let Some(first) = caps.first() {
            return Some(capability_from_advertised_name(first));
        }
    }

    None
}

#[cfg(windows)]
pub(crate) async fn probe_primary_capability(_socket_path: &Path) -> Option<PrimalCapability> {
    None
}

fn extract_advertised_capability_names(resp: &serde_json::Value) -> Vec<String> {
    let result = &resp["result"];

    if let Some(caps) = result["capabilities"].as_array() {
        let parsed = names_from_capability_array(caps);
        if !parsed.is_empty() {
            return parsed;
        }
    }

    if let Some(arr) = result.as_array() {
        let parsed = names_from_capability_array(arr);
        if !parsed.is_empty() {
            return parsed;
        }
    }

    if let Some(groups) = result["provided_capabilities"].as_array() {
        let parsed: Vec<String> = groups
            .iter()
            .filter_map(|group| group["type"].as_str().map(str::to_owned))
            .collect();
        if !parsed.is_empty() {
            return parsed;
        }
    }

    Vec::new()
}

fn names_from_capability_array(arr: &[serde_json::Value]) -> Vec<String> {
    arr.iter()
        .filter_map(|value| {
            if let Some(name) = value.as_str() {
                return Some(name.to_owned());
            }
            value
                .get("method")
                .or_else(|| value.get("name"))
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned)
        })
        .collect()
}

fn capability_from_advertised_name(cap: &str) -> PrimalCapability {
    let lower = cap.to_lowercase();
    match lower.as_str() {
        "encryption" | "crypto" | "security" => PrimalCapability::encryption(),
        "networking" | "discovery" | "network" => PrimalCapability::networking(),
        "compute" => PrimalCapability::compute(),
        "storage" => PrimalCapability::storage(),
        "ai" => PrimalCapability::ai(),
        "science" => PrimalCapability::science(),
        _ => match lower.split_once('.') {
            Some((category, name)) => PrimalCapability::new(category, name, "1.0"),
            None => PrimalCapability::new(&lower, &lower, "1.0"),
        },
    }
}

#[cfg(test)]
pub(crate) fn clear_runtime_capability_cache_for_tests() {
    if let Ok(mut cache) = runtime_capability_cache().write() {
        cache.clear();
    }
}
