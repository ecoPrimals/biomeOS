// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! JSON-RPC endpoint probing over Unix sockets and stub TCP/HTTP handling.

use std::path::Path;

use anyhow::Result;
use biomeos_primal_sdk::PrimalCapability;
use biomeos_types::Health;
use biomeos_types::constants::timeouts;

use super::super::discovery::ProbeResult;

/// Probe an endpoint string: Unix paths, `unix://`, or stub TCP/HTTP.
pub(crate) async fn probe_endpoint(endpoint: &str) -> Result<ProbeResult> {
    tracing::debug!("🔍 Probing endpoint: {}", endpoint);

    let probe_result = if let Some(path) = endpoint.strip_prefix("unix://") {
        probe_unix_endpoint(path).await
    } else if endpoint.starts_with("tcp://") || endpoint.starts_with("http://") {
        Ok(ProbeResult {
            name: "unknown".to_string(),
            version: "unknown".to_string(),
            capabilities: vec![],
            health: Health::Healthy,
        })
    } else if Path::new(endpoint)
        .extension()
        .is_some_and(|ext| ext == "sock")
        || endpoint.starts_with('/')
    {
        probe_unix_endpoint(endpoint).await
    } else {
        anyhow::bail!("Unsupported endpoint scheme: {endpoint}")
    };

    match &probe_result {
        Ok(r) => tracing::info!(
            "✅ Probed {}: {} v{} ({:?})",
            endpoint,
            r.name,
            r.version,
            r.health
        ),
        Err(e) => tracing::warn!("❌ Probe {} failed: {}", endpoint, e),
    }
    probe_result
}

/// Probe a Unix socket endpoint with real JSON-RPC requests.
///
/// Sends `identity.get` first (to learn name + version), then
/// `capabilities.list` (to discover advertised capabilities). If
/// `identity.get` returns an error the primal is still "reachable" — we just
/// lack its self-reported identity.
async fn probe_unix_endpoint(socket_path: &str) -> Result<ProbeResult> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;

    let path = Path::new(socket_path);

    let stream = tokio::time::timeout(timeouts::PROBE_TIMEOUT, UnixStream::connect(path))
        .await
        .map_err(|_| anyhow::anyhow!("connect timed out"))?
        .map_err(|e| anyhow::anyhow!("connect failed: {e}"))?;

    let mut reader = BufReader::new(stream);

    // --- identity.get ---
    let identity_req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "identity.get",
        "id": 1
    });
    let line = serde_json::to_string(&identity_req)? + "\n";
    reader.get_mut().write_all(line.as_bytes()).await?;
    let _ = reader.get_mut().flush().await;

    let mut resp_line = String::new();
    let (name, version) =
        match tokio::time::timeout(timeouts::PROBE_TIMEOUT, reader.read_line(&mut resp_line)).await
        {
            Ok(Ok(n)) if n > 0 => {
                let v: serde_json::Value = serde_json::from_str(&resp_line).unwrap_or_default();
                let name = v["result"]["name"]
                    .as_str()
                    .or_else(|| v["result"]["primal"].as_str())
                    .unwrap_or("unknown")
                    .to_string();
                let version = v["result"]["version"]
                    .as_str()
                    .unwrap_or("unknown")
                    .to_string();
                (name, version)
            }
            _ => ("unknown".to_string(), "unknown".to_string()),
        };

    // --- capabilities.list ---
    let caps = crate::socket_discovery::probe_unix_socket_capabilities_list(path).await;
    let capabilities = caps
        .into_iter()
        .map(|c| PrimalCapability::new(&c, "", ""))
        .collect();

    Ok(ProbeResult {
        name,
        version,
        capabilities,
        health: Health::Healthy,
    })
}
