// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! JSON-RPC endpoint probing over Unix sockets, TCP, and HTTP.

use std::path::Path;

use anyhow::Result;
use biomeos_primal_sdk::PrimalCapability;
use biomeos_types::Health;
use biomeos_types::constants::timeouts;

use super::super::discovery::ProbeResult;

/// Probe an endpoint string: Unix paths, `unix://`, TCP, or HTTP.
pub(crate) async fn probe_endpoint(endpoint: &str) -> Result<ProbeResult> {
    tracing::debug!("Probing endpoint: {}", endpoint);

    let probe_result = if let Some(path) = endpoint.strip_prefix("unix://") {
        probe_unix_endpoint(path).await
    } else if let Some(authority) = endpoint.strip_prefix("tcp://") {
        probe_tcp_endpoint(authority).await
    } else if endpoint.starts_with("http://") || endpoint.starts_with("https://") {
        probe_http_endpoint(endpoint).await
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
            "Probed {}: {} v{} ({:?})",
            endpoint,
            r.name,
            r.version,
            r.health
        ),
        Err(e) => tracing::warn!("Probe {} failed: {}", endpoint, e),
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
    use tokio::io::BufReader;
    use tokio::net::UnixStream;

    let path = Path::new(socket_path);

    let stream = tokio::time::timeout(timeouts::PROBE_TIMEOUT, UnixStream::connect(path))
        .await
        .map_err(|_| anyhow::anyhow!("connect timed out"))?
        .map_err(|e| anyhow::anyhow!("connect failed: {e}"))?;

    let mut reader = BufReader::new(stream);

    let (name, version) = send_identity_get_line(&mut reader).await;

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

/// Probe a TCP endpoint with line-delimited JSON-RPC (same framing as Unix).
async fn probe_tcp_endpoint(authority: &str) -> Result<ProbeResult> {
    use tokio::io::BufReader;
    use tokio::net::TcpStream;

    let stream = tokio::time::timeout(timeouts::PROBE_TIMEOUT, TcpStream::connect(authority))
        .await
        .map_err(|_| anyhow::anyhow!("TCP connect timed out: {authority}"))?
        .map_err(|e| anyhow::anyhow!("TCP connect failed ({authority}): {e}"))?;

    let mut reader = BufReader::new(stream);

    let (name, version) = send_identity_get_line(&mut reader).await;

    let capabilities = send_capabilities_list_line(&mut reader).await;

    Ok(ProbeResult {
        name,
        version,
        capabilities,
        health: Health::Healthy,
    })
}

/// Probe an HTTP endpoint by POSTing JSON-RPC to the root URL.
async fn probe_http_endpoint(url: &str) -> Result<ProbeResult> {
    use tokio::net::TcpStream;

    let parsed: url::Url = url
        .parse()
        .map_err(|e| anyhow::anyhow!("invalid HTTP endpoint URL ({url}): {e}"))?;

    let host = parsed
        .host_str()
        .ok_or_else(|| anyhow::anyhow!("no host in URL: {url}"))?;
    let port = parsed.port_or_known_default().unwrap_or(80);
    let authority = format!("{host}:{port}");

    let stream = tokio::time::timeout(timeouts::PROBE_TIMEOUT, TcpStream::connect(&authority))
        .await
        .map_err(|_| anyhow::anyhow!("HTTP connect timed out: {authority}"))?
        .map_err(|e| anyhow::anyhow!("HTTP connect failed ({authority}): {e}"))?;

    let (name, version) = http_jsonrpc_call(&stream, &parsed, "identity.get", 1).await;

    let capabilities = match http_jsonrpc_call_raw(&stream, &parsed, "capabilities.list", 2).await {
        Some(val) => extract_capabilities(&val),
        None => vec![],
    };

    Ok(ProbeResult {
        name,
        version,
        capabilities,
        health: Health::Healthy,
    })
}

// ---------------------------------------------------------------------------
// Shared helpers
// ---------------------------------------------------------------------------

/// Send `identity.get` over a line-delimited stream and parse name/version.
async fn send_identity_get_line<S>(reader: &mut tokio::io::BufReader<S>) -> (String, String)
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

    let identity_req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "identity.get",
        "id": 1
    });
    let line = serde_json::to_string(&identity_req).unwrap_or_default() + "\n";
    if reader.get_mut().write_all(line.as_bytes()).await.is_err() {
        return ("unknown".to_string(), "unknown".to_string());
    }
    let _ = reader.get_mut().flush().await;

    let mut resp_line = String::new();
    match tokio::time::timeout(timeouts::PROBE_TIMEOUT, reader.read_line(&mut resp_line)).await {
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
    }
}

/// Send `capabilities.list` over a line-delimited stream and parse capabilities.
async fn send_capabilities_list_line<S>(
    reader: &mut tokio::io::BufReader<S>,
) -> Vec<PrimalCapability>
where
    S: tokio::io::AsyncRead + tokio::io::AsyncWrite + Unpin,
{
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt};

    let caps_req = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "capabilities.list",
        "id": 2
    });
    let line = serde_json::to_string(&caps_req).unwrap_or_default() + "\n";
    if reader.get_mut().write_all(line.as_bytes()).await.is_err() {
        return vec![];
    }
    let _ = reader.get_mut().flush().await;

    let mut resp_line = String::new();
    match tokio::time::timeout(timeouts::PROBE_TIMEOUT, reader.read_line(&mut resp_line)).await {
        Ok(Ok(n)) if n > 0 => {
            let v: serde_json::Value = serde_json::from_str(&resp_line).unwrap_or_default();
            extract_capabilities(&v)
        }
        _ => vec![],
    }
}

/// Extract capabilities from a JSON-RPC response, handling both array-of-strings
/// and array-of-objects (`{name, ...}`) formats.
fn extract_capabilities(v: &serde_json::Value) -> Vec<PrimalCapability> {
    let arr = v["result"]["capabilities"]
        .as_array()
        .or_else(|| v["result"].as_array());
    match arr {
        Some(items) => items
            .iter()
            .filter_map(|item| {
                item.as_str()
                    .or_else(|| item["name"].as_str())
                    .map(|name| PrimalCapability::new(name, "", ""))
            })
            .collect(),
        None => vec![],
    }
}

/// Send a JSON-RPC call over a raw HTTP/1.1 POST and parse identity fields.
///
/// Uses a minimal hand-rolled HTTP/1.1 request to avoid pulling in a full
/// HTTP client just for probing.
async fn http_jsonrpc_call(
    stream: &tokio::net::TcpStream,
    parsed: &url::Url,
    method: &str,
    id: u64,
) -> (String, String) {
    match http_jsonrpc_call_raw(stream, parsed, method, id).await {
        Some(v) => {
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
        None => ("unknown".to_string(), "unknown".to_string()),
    }
}

/// Low-level HTTP/1.1 POST of a JSON-RPC request, returning the parsed JSON response.
async fn http_jsonrpc_call_raw(
    _stream: &tokio::net::TcpStream,
    parsed: &url::Url,
    method: &str,
    id: u64,
) -> Option<serde_json::Value> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;

    let host = parsed.host_str()?;
    let port = parsed.port_or_known_default().unwrap_or(80);
    let authority = format!("{host}:{port}");
    let path = parsed.path();

    let mut conn = tokio::time::timeout(timeouts::PROBE_TIMEOUT, TcpStream::connect(&authority))
        .await
        .ok()?
        .ok()?;

    let body = serde_json::json!({
        "jsonrpc": "2.0",
        "method": method,
        "id": id
    })
    .to_string();

    let request = format!(
        "POST {path} HTTP/1.1\r\nHost: {host}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
        body.len()
    );

    conn.write_all(request.as_bytes()).await.ok()?;
    let _ = conn.flush().await;

    let mut response_buf = Vec::with_capacity(4096);
    let _ =
        tokio::time::timeout(timeouts::PROBE_TIMEOUT, conn.read_to_end(&mut response_buf)).await;

    let response_str = String::from_utf8_lossy(&response_buf);
    let body_start = response_str.find("\r\n\r\n").map(|i| i + 4)?;
    let json_body = &response_str[body_start..];
    serde_json::from_str(json_body).ok()
}
