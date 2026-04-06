// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! JSON-RPC transport and primal queries used during protocol escalation (JSON-RPC → tarpc).

#![deny(unsafe_code)]

use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

use crate::living_graph::LivingGraph;

use super::config::TarpcEndpoint;

pub(super) async fn send_json_rpc(socket_path: &PathBuf, request: &Value) -> Result<Value, String> {
    let mut stream = UnixStream::connect(socket_path)
        .await
        .map_err(|e| format!("Failed to connect to {}: {}", socket_path.display(), e))?;

    let request_str =
        serde_json::to_string(request).map_err(|e| format!("Failed to serialize request: {e}"))?;

    stream
        .write_all(request_str.as_bytes())
        .await
        .map_err(|e| format!("Failed to write request: {e}"))?;
    stream
        .write_all(b"\n")
        .await
        .map_err(|e| format!("Failed to write newline: {e}"))?;

    let mut reader = BufReader::new(stream);
    let mut response_line = String::new();

    match tokio::time::timeout(Duration::from_secs(5), reader.read_line(&mut response_line)).await {
        Ok(Ok(_)) => {}
        Ok(Err(e)) => return Err(format!("Failed to read response: {e}")),
        Err(_) => return Err("Response timeout (>5s)".to_string()),
    }

    serde_json::from_str(&response_line).map_err(|e| format!("Failed to parse response: {e}"))
}

pub(super) async fn query_tarpc_endpoint(
    graph: &Arc<LivingGraph>,
    primal: &str,
) -> Result<TarpcEndpoint, String> {
    let state = graph
        .get_primal_state(primal)
        .await
        .ok_or_else(|| format!("Primal not found: {primal}"))?;

    if let Some(socket) = &state.tarpc_socket {
        return Ok(TarpcEndpoint {
            available: true,
            socket: Some(socket.clone()),
            services: state.capabilities.clone(),
        });
    }

    let request = json!({
        "jsonrpc": "2.0",
        "method": "rpc.tarpc_endpoint",
        "params": {},
        "id": graph.next_request_id(),
    });

    match send_json_rpc(&state.json_rpc_socket, &request).await {
        Ok(response) => {
            if let Some(result) = response.get("result") {
                let endpoint: TarpcEndpoint = serde_json::from_value(result.clone())
                    .map_err(|e| format!("Invalid tarpc endpoint response: {e}"))?;
                Ok(endpoint)
            } else if let Some(_error) = response.get("error") {
                tracing::debug!("Primal {} doesn't support tarpc: {:?}", primal, _error);
                Ok(TarpcEndpoint {
                    available: false,
                    socket: None,
                    services: vec![],
                })
            } else {
                Err("Invalid JSON-RPC response".to_string())
            }
        }
        Err(e) => {
            tracing::debug!("Failed to query {} for tarpc endpoint: {}", primal, e);
            Ok(TarpcEndpoint {
                available: false,
                socket: None,
                services: vec![],
            })
        }
    }
}

pub(super) async fn notify_escalation(
    graph: &Arc<LivingGraph>,
    from: &str,
    to: &str,
    tarpc_info: &TarpcEndpoint,
) -> Result<(), String> {
    let from_state = graph
        .get_primal_state(from)
        .await
        .ok_or_else(|| format!("Source primal not found: {from}"))?;

    let request = json!({
        "jsonrpc": "2.0",
        "method": "rpc.escalate_to",
        "params": {
            "target": to,
            "tarpc_socket": tarpc_info.socket,
            "services": tarpc_info.services,
        },
        "id": graph.next_request_id(),
    });

    let response = send_json_rpc(&from_state.json_rpc_socket, &request).await?;

    if response.get("error").is_some() {
        let error = response
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");
        return Err(format!("Escalation notification failed: {error}"));
    }

    Ok(())
}

pub(super) async fn verify_tarpc_connection(
    graph: &Arc<LivingGraph>,
    from: &str,
    to: &str,
) -> Result<(), String> {
    let state = graph
        .get_primal_state(to)
        .await
        .ok_or_else(|| format!("Primal not found: {to}"))?;

    let tarpc_socket = state
        .tarpc_socket
        .as_ref()
        .ok_or_else(|| format!("No tarpc socket for {to}"))?;

    if !tarpc_socket.exists() {
        return Err(format!(
            "tarpc socket does not exist: {}",
            tarpc_socket.display()
        ));
    }

    let client = crate::tarpc_client::connect_tarpc_health(tarpc_socket)
        .await
        .map_err(|e| format!("tarpc connect failed: {e}"))?;

    let ctx = tarpc::context::current();
    client
        .health_check(ctx)
        .await
        .map_err(|e| format!("tarpc health_check failed: {e}"))?;

    tracing::debug!("tarpc verification passed: {} → {}", from, to);
    Ok(())
}

pub(super) async fn notify_fallback(
    graph: &Arc<LivingGraph>,
    from: &str,
    to: &str,
    reason: &str,
) -> Result<(), String> {
    let from_state = graph
        .get_primal_state(from)
        .await
        .ok_or_else(|| format!("Source primal not found: {from}"))?;

    let request = json!({
        "jsonrpc": "2.0",
        "method": "rpc.fallback_to_json_rpc",
        "params": {
            "target": to,
            "reason": reason,
        },
        "id": graph.next_request_id(),
    });

    let response = send_json_rpc(&from_state.json_rpc_socket, &request).await?;

    if response.get("error").is_some() {
        let error = response
            .get("error")
            .and_then(|e| e.get("message"))
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");
        return Err(format!("Fallback notification failed: {error}"));
    }

    Ok(())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::super::config::TarpcEndpoint;
    use super::*;
    use crate::living_graph::{LivingGraph, PrimalProtocolState};
    use serde_json::{Value, json};
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;

    #[tokio::test]
    async fn send_json_rpc_missing_socket() {
        let path = PathBuf::from("/nonexistent/rpc-missing-test.sock");
        let err = send_json_rpc(&path, &json!({"x": 1}))
            .await
            .expect_err("expected connect error");
        assert!(
            err.contains("Failed to connect") || err.contains("No such file"),
            "{err}"
        );
    }

    #[tokio::test]
    async fn send_json_rpc_roundtrip() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("rpc.sock");
        let listener = UnixListener::bind(&path).expect("bind");
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.expect("read");
            let req: Value = serde_json::from_str(line.trim()).expect("parse req");
            let id = req.get("id").cloned().unwrap_or(Value::Null);
            let resp = json!({
                "jsonrpc": "2.0",
                "result": { "pong": true },
                "id": id
            });
            let mut stream = reader.into_inner();
            stream
                .write_all(format!("{}\n", serde_json::to_string(&resp).unwrap()).as_bytes())
                .await
                .expect("write");
        });

        tokio::task::yield_now().await;
        let out = send_json_rpc(&path, &json!({"jsonrpc":"2.0","method":"ping","id":7}))
            .await
            .expect("ok");
        assert_eq!(out.pointer("/result/pong"), Some(&json!(true)));
    }

    #[tokio::test]
    async fn send_json_rpc_invalid_json_response() {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("bad.sock");
        let listener = UnixListener::bind(&path).expect("bind");
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept");
            stream
                .write_all(b"NOT JSON AT ALL\n")
                .await
                .expect("write garbage");
        });
        tokio::task::yield_now().await;
        let err = send_json_rpc(&path, &json!({"jsonrpc":"2.0","method":"m","id":1}))
            .await
            .expect_err("parse error");
        assert!(err.contains("parse") || err.contains("Parse"), "{err}");
    }

    #[tokio::test]
    async fn query_tarpc_endpoint_uses_cached_tarpc_socket() {
        let graph = Arc::new(LivingGraph::new("fam"));
        let state = PrimalProtocolState::new("alpha", PathBuf::from("/tmp/ignored.json.sock"))
            .with_tarpc_socket(PathBuf::from("/tmp/alpha-tarpc.sock"));
        graph.register_primal(state).await;

        let ep = query_tarpc_endpoint(&graph, "alpha")
            .await
            .expect("endpoint");
        assert!(ep.available);
        assert_eq!(ep.socket, Some(PathBuf::from("/tmp/alpha-tarpc.sock")));
    }

    #[tokio::test]
    async fn query_tarpc_endpoint_primal_missing() {
        let graph = Arc::new(LivingGraph::new("fam"));
        let err = query_tarpc_endpoint(&graph, "ghost")
            .await
            .expect_err("missing primal");
        assert!(err.contains("not found"), "{err}");
    }

    #[tokio::test]
    async fn query_tarpc_endpoint_via_jsonrpc_result() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("primal.sock");
        let listener = UnixListener::bind(&sock).expect("bind");
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.expect("read");
            let req: Value = serde_json::from_str(line.trim()).expect("parse");
            assert_eq!(
                req.get("method").and_then(|m| m.as_str()),
                Some("rpc.tarpc_endpoint")
            );
            let id = req.get("id").cloned().unwrap_or(Value::Null);
            let endpoint = TarpcEndpoint {
                available: true,
                socket: Some(PathBuf::from("/rpc/t.sock")),
                services: vec!["svc".to_string()],
            };
            let resp = json!({
                "jsonrpc": "2.0",
                "result": endpoint,
                "id": id
            });
            let mut stream = reader.into_inner();
            stream
                .write_all(format!("{}\n", serde_json::to_string(&resp).unwrap()).as_bytes())
                .await
                .expect("write");
        });

        let graph = Arc::new(LivingGraph::new("fam"));
        let state = PrimalProtocolState::new("beta", sock.clone());
        graph.register_primal(state).await;

        tokio::task::yield_now().await;
        let ep = query_tarpc_endpoint(&graph, "beta").await.expect("parsed");
        assert!(ep.available);
        assert_eq!(ep.socket, Some(PathBuf::from("/rpc/t.sock")));
        assert_eq!(ep.services, vec!["svc".to_string()]);
    }

    #[tokio::test]
    async fn query_tarpc_endpoint_jsonrpc_error_returns_unavailable() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("primal2.sock");
        let listener = UnixListener::bind(&sock).expect("bind");
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.expect("read");
            let req: Value = serde_json::from_str(line.trim()).expect("parse");
            let id = req.get("id").cloned().unwrap_or(Value::Null);
            let resp = json!({
                "jsonrpc": "2.0",
                "error": { "code": -32601, "message": "no tarpc" },
                "id": id
            });
            let mut stream = reader.into_inner();
            stream
                .write_all(format!("{}\n", serde_json::to_string(&resp).unwrap()).as_bytes())
                .await
                .expect("write");
        });

        let graph = Arc::new(LivingGraph::new("fam"));
        graph
            .register_primal(PrimalProtocolState::new("gamma", sock.clone()))
            .await;

        tokio::task::yield_now().await;
        let ep = query_tarpc_endpoint(&graph, "gamma")
            .await
            .expect("fallback");
        assert!(!ep.available);
        assert!(ep.socket.is_none());
    }

    #[tokio::test]
    async fn query_tarpc_endpoint_invalid_response_shape() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("primal3.sock");
        let listener = UnixListener::bind(&sock).expect("bind");
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.expect("read");
            let req: Value = serde_json::from_str(line.trim()).expect("parse");
            let id = req.get("id").cloned().unwrap_or(Value::Null);
            let resp = json!({ "jsonrpc": "2.0", "id": id });
            let mut stream = reader.into_inner();
            stream
                .write_all(format!("{}\n", serde_json::to_string(&resp).unwrap()).as_bytes())
                .await
                .expect("write");
        });

        let graph = Arc::new(LivingGraph::new("fam"));
        graph
            .register_primal(PrimalProtocolState::new("delta", sock.clone()))
            .await;

        tokio::task::yield_now().await;
        let err = query_tarpc_endpoint(&graph, "delta")
            .await
            .expect_err("invalid");
        assert!(err.contains("Invalid JSON-RPC"), "{err}");
    }

    #[tokio::test]
    async fn notify_escalation_source_missing() {
        let graph = Arc::new(LivingGraph::new("fam"));
        let tarpc = TarpcEndpoint {
            available: true,
            socket: Some(PathBuf::from("/t.sock")),
            services: vec![],
        };
        let err = notify_escalation(&graph, "missing", "to", &tarpc)
            .await
            .expect_err("no source");
        assert!(err.contains("Source primal not found"), "{err}");
    }

    #[tokio::test]
    async fn notify_fallback_source_missing() {
        let graph = Arc::new(LivingGraph::new("fam"));
        let err = notify_fallback(&graph, "nope", "tgt", "reason")
            .await
            .expect_err("no source");
        assert!(err.contains("Source primal not found"), "{err}");
    }

    #[tokio::test]
    async fn notify_escalation_rpc_error_message() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("from.sock");
        let listener = UnixListener::bind(&sock).expect("bind");
        tokio::spawn(async move {
            let (stream, _) = listener.accept().await.expect("accept");
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.expect("read");
            let req: Value = serde_json::from_str(line.trim()).expect("parse");
            let id = req.get("id").cloned().unwrap_or(Value::Null);
            let resp = json!({
                "jsonrpc": "2.0",
                "error": { "code": -1, "message": "escalation denied" },
                "id": id
            });
            let mut stream = reader.into_inner();
            stream
                .write_all(format!("{}\n", serde_json::to_string(&resp).unwrap()).as_bytes())
                .await
                .expect("write");
        });

        let graph = Arc::new(LivingGraph::new("fam"));
        graph
            .register_primal(PrimalProtocolState::new("from", sock.clone()))
            .await;

        let tarpc = TarpcEndpoint {
            available: true,
            socket: Some(PathBuf::from("/x.sock")),
            services: vec![],
        };

        tokio::task::yield_now().await;
        let err = notify_escalation(&graph, "from", "to", &tarpc)
            .await
            .expect_err("rpc error");
        assert!(err.contains("escalation denied"), "{err}");
    }

    #[tokio::test]
    async fn verify_tarpc_primal_or_socket_missing() {
        let graph = Arc::new(LivingGraph::new("fam"));
        let err = verify_tarpc_connection(&graph, "a", "b")
            .await
            .expect_err("missing primal");
        assert!(err.contains("not found"), "{err}");

        let graph = Arc::new(LivingGraph::new("fam2"));
        graph
            .register_primal(PrimalProtocolState::new(
                "solo",
                PathBuf::from("/tmp/x.sock"),
            ))
            .await;
        let err = verify_tarpc_connection(&graph, "a", "solo")
            .await
            .expect_err("no tarpc");
        assert!(err.contains("No tarpc") || err.contains("tarpc"), "{err}");
    }
}
