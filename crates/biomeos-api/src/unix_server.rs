// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Unix socket server for biomeOS API
//!
//! Provides secure, port-free communication via Unix sockets.
//! Supports HTTP (axum), raw JSON-RPC (NDJSON), and BTSP recognition
//! via first-byte auto-detection: if the first byte is `{`, the first
//! line is inspected — BTSP `ClientHello` is answered with a redirect
//! to the neural-api socket; otherwise the connection is handled as
//! newline-delimited JSON-RPC.  Non-`{` bytes are passed to hyper/axum
//! as HTTP.

use anyhow::{Context, Result};
use axum::Router;
use std::path::Path;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;
use tracing::{debug, info, warn};

/// Serve an Axum router over a Unix socket
///
/// This is the PRIMARY transport for biomeOS API, providing:
/// - Port-free architecture (no TCP ports!)
/// - Secure by default (filesystem permissions)
/// - Fast (0.1ms overhead vs 10ms HTTP)
/// - Dual-protocol: HTTP (axum) and raw JSON-RPC via first-byte auto-detect
///
/// # Arguments
///
/// * `socket_path` - Path to Unix socket
/// * `app` - Axum router to serve
/// * `on_ready` - Optional callback invoked after bind (for tests)
///
/// # Security
///
/// The socket is created with 0600 permissions (owner-only).
pub async fn serve_unix_socket<P: AsRef<Path>>(
    socket_path: P,
    app: Router,
    on_ready: Option<Box<dyn FnOnce() + Send>>,
) -> Result<()> {
    let socket_path = socket_path.as_ref();

    // Remove old socket if exists
    if socket_path.exists() {
        std::fs::remove_file(socket_path).context("Failed to remove old Unix socket")?;
    }

    // Create Unix listener
    let listener = UnixListener::bind(socket_path).context("Failed to bind Unix socket")?;

    // Set permissions (0600 - owner only)
    #[cfg(unix)]
    {
        use std::fs;
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(socket_path, fs::Permissions::from_mode(0o600))
            .context("Failed to set socket permissions")?;
    }

    info!(
        "📡 biomeOS API listening on Unix socket: {}",
        socket_path.display()
    );
    info!("   Security: Owner-only (0600 permissions)");
    info!("   Protocol: HTTP + raw JSON-RPC (auto-detect)");
    info!("   Port-free: ✅ TRUE PRIMAL architecture!");

    if let Some(f) = on_ready {
        f();
    }

    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let app = app.clone();

                tokio::spawn(async move {
                    let mut reader = BufReader::new(stream);

                    let is_jsonrpc = match reader.fill_buf().await {
                        Ok(buf) if !buf.is_empty() => buf[0] == b'{' || buf[0] == b'[',
                        _ => false,
                    };

                    if is_jsonrpc {
                        debug!("UDS auto-detect: raw JSON-RPC connection");
                        if let Err(e) = handle_raw_jsonrpc(reader).await {
                            debug!("Raw JSON-RPC connection ended: {e}");
                        }
                    } else {
                        // BufReader<UnixStream> implements AsyncRead + AsyncWrite,
                        // so any bytes already buffered by fill_buf are replayed to
                        // hyper transparently.
                        serve_http_connection(reader, app).await;
                    }
                });
            }
            Err(e) => {
                warn!("Failed to accept connection: {}", e);
            }
        }
    }
}

async fn serve_http_connection(stream: BufReader<tokio::net::UnixStream>, app: Router) {
    let stream = hyper_util::rt::TokioIo::new(stream);
    let hyper_service =
        hyper::service::service_fn(move |request: hyper::Request<hyper::body::Incoming>| {
            let (parts, body) = request.into_parts();
            let body = axum::body::Body::new(body);
            let request = axum::http::Request::from_parts(parts, body);
            let mut app = app.clone();

            async move {
                use tower::Service;
                let response = match app.call(request).await {
                    Ok(resp) => resp,
                    Err(infallible) => match infallible {},
                };
                Ok::<_, hyper::Error>(response)
            }
        });

    if let Err(e) =
        hyper_util::server::conn::auto::Builder::new(hyper_util::rt::TokioExecutor::new())
            .serve_connection(stream, hyper_service)
            .await
    {
        warn!("Error serving HTTP connection: {}", e);
    }
}

/// Handle a raw JSON-RPC (NDJSON) connection.
///
/// Responds to standard primal discovery methods (`health.check`,
/// `health.liveness`, `identity.get`, `capabilities.list`) so that
/// spring probes and discovery sweeps see biomeOS as alive rather than
/// receiving HTTP 400.
async fn handle_raw_jsonrpc(mut reader: BufReader<tokio::net::UnixStream>) -> Result<()> {
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        let response = dispatch_jsonrpc_line_async(trimmed).await;
        let mut out = serde_json::to_string(&response)?;
        out.push('\n');
        reader.get_mut().write_all(out.as_bytes()).await?;
        reader.get_mut().flush().await?;
    }

    Ok(())
}

/// Neural API methods that should be proxied to the Neural API socket
/// instead of returning -32601. These are the cross-gate critical path methods.
const NEURAL_API_PROXY_METHODS: &[&str] = &["capability.call", "graph.execute", "topology.primals"];

async fn dispatch_jsonrpc_line_async(line: &str) -> serde_json::Value {
    let null = serde_json::Value::Null;
    let Ok(req) = serde_json::from_str::<serde_json::Value>(line) else {
        return jsonrpc_error(&null, -32700, "Parse error");
    };
    let id = req.get("id").unwrap_or(&null);
    let method = req
        .get("method")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");

    if NEURAL_API_PROXY_METHODS.contains(&method) {
        let params = req
            .get("params")
            .cloned()
            .unwrap_or_else(|| serde_json::Value::Object(serde_json::Map::default()));
        return proxy_to_neural_api(id, method, &params).await;
    }

    dispatch_jsonrpc_line(line)
}

/// Forward a JSON-RPC request to the Neural API socket and return the response.
async fn proxy_to_neural_api(
    id: &serde_json::Value,
    method: &str,
    params: &serde_json::Value,
) -> serde_json::Value {
    let family_id = biomeos_types::env_config::family_id().unwrap_or_else(|| "default".to_string());

    let socket_path = neural_api_client::NeuralApiClient::discover_socket(&family_id);

    if !socket_path.exists() {
        debug!(
            "Neural API socket not found at {} — cannot proxy {method}",
            socket_path.display()
        );
        return jsonrpc_error(
            id,
            -32002,
            &format!(
                "Neural API not running (socket not found: {}). \
                 Start with `biomeos nucleus` or `biomeos neural-api`",
                socket_path.display()
            ),
        );
    }

    let client = match neural_api_client::NeuralApiClient::new(&socket_path) {
        Ok(c) => c.with_request_timeout(tokio::time::Duration::from_secs(30)),
        Err(e) => {
            warn!("Failed to create Neural API client for proxy: {e}");
            return jsonrpc_error(id, -32002, &format!("Neural API client error: {e}"));
        }
    };

    match neural_api_client::connection::json_rpc_call(
        &client.socket_path,
        method,
        params,
        client.request_timeout,
        client.connection_timeout,
        &client.retry_config,
    )
    .await
    {
        Ok(result) => {
            debug!("Proxied {method} to Neural API → success");
            serde_json::json!({
                "jsonrpc": "2.0",
                "result": result,
                "id": id
            })
        }
        Err(e) => {
            debug!("Proxied {method} to Neural API → error: {e}");
            jsonrpc_error(id, -32603, &format!("Neural API proxy error: {e}"))
        }
    }
}

fn dispatch_jsonrpc_line(line: &str) -> serde_json::Value {
    let null = serde_json::Value::Null;
    let Ok(req) = serde_json::from_str::<serde_json::Value>(line) else {
        return jsonrpc_error(&null, -32700, "Parse error");
    };

    // BTSP ClientHello detection: `{"protocol":"btsp",...}` has no `method`
    // or `jsonrpc` field. Respond with a structured redirect so the caller
    // knows to use the neural-api socket for BTSP-authenticated channels.
    if req.get("protocol").and_then(serde_json::Value::as_str) == Some("btsp") {
        debug!("BTSP ClientHello received on API socket — redirecting to neural-api");
        let id = req.get("id").unwrap_or(&null);
        return serde_json::json!({
            "jsonrpc": "2.0",
            "error": {
                "code": -32001,
                "message": "BTSP not supported on biomeOS API socket. \
                    Use the neural-api socket for BTSP-authenticated connections.",
                "data": {
                    "protocol": "btsp",
                    "redirect": "neural-api"
                }
            },
            "id": id
        });
    }

    let id = req.get("id").unwrap_or(&null);
    let method = req
        .get("method")
        .and_then(serde_json::Value::as_str)
        .unwrap_or("");

    match method {
        "health" | "health.check" | "health.liveness" | "health.status" => {
            let result = serde_json::json!({ "status": "healthy", "primal": biomeos_types::primal_names::BIOMEOS });
            jsonrpc_ok(id, &result)
        }
        "identity.get" | "identity" => {
            let result = serde_json::json!({
                "primal": biomeos_types::primal_names::BIOMEOS,
                "role": "orchestrator",
                "version": env!("CARGO_PKG_VERSION"),
                "transport": "http+jsonrpc",
                "note": "biomeOS API socket proxies capability.call and graph.execute to Neural API automatically"
            });
            jsonrpc_ok(id, &result)
        }
        "primal.list" | "topology.primals" => {
            let result = serde_json::json!({
                "primals": [
                    {
                        "name": biomeos_types::primal_names::BIOMEOS,
                        "socket": "self",
                        "status": "running",
                        "capabilities": ["orchestration"],
                        "pid": std::process::id(),
                    }
                ],
                "count": 1,
                "note": "biomeOS API socket reports itself only. \
                         For full topology use the neural-api socket (topology.primals)"
            });
            jsonrpc_ok(id, &result)
        }
        "capabilities.list" | "capability.list" => {
            let result = serde_json::json!({
                "primal": biomeos_types::primal_names::BIOMEOS,
                "methods": [
                    "health.check",
                    "health.liveness",
                    "identity.get",
                    "capabilities.list",
                    "capability.call",
                    "graph.execute",
                    "primal.list"
                ],
                "http_api": [
                    "/api/v1/health",
                    "/api/v1/topology",
                    "/api/v1/genome/list",
                    "/api/v1/trust/identity",
                    "/api/v1/events/stream"
                ],
                "note": "capability.call and graph.execute are proxied to the Neural API socket automatically. Full HTTP API also available."
            });
            jsonrpc_ok(id, &result)
        }
        _ => jsonrpc_error(
            id,
            -32601,
            &format!(
                "Method not found: {method}. \
                 capability.call and graph.execute are auto-proxied; \
                 other Neural API methods require the neural-api socket"
            ),
        ),
    }
}

fn jsonrpc_ok(id: &serde_json::Value, result: &serde_json::Value) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "result": result,
        "id": id
    })
}

fn jsonrpc_error(id: &serde_json::Value, code: i32, message: &str) -> serde_json::Value {
    serde_json::json!({
        "jsonrpc": "2.0",
        "error": { "code": code, "message": message },
        "id": id
    })
}

#[cfg(test)]
#[cfg(unix)]
#[path = "unix_server_tests.rs"]
mod tests;
