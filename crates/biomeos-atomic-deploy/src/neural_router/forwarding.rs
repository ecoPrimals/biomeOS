// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Request forwarding - JSON-RPC and tarpc protocol escalation

use anyhow::{Context, Result};
use serde_json::Value;
use std::path::PathBuf;
use tracing::debug;

use biomeos_core::atomic_client::AtomicClient;
use biomeos_types::tarpc_types::ProtocolPreference;

use crate::living_graph::ProtocolMode;

use super::NeuralRouter;

impl NeuralRouter {
    /// Forward JSON-RPC request to primal
    ///
    /// **Pure Rust**: Async I/O, no unsafe code, idiomatic error handling
    ///
    /// **JSON-RPC AND tarpc first**: Checks protocol availability and preferences.
    pub async fn forward_request(
        &self,
        socket_path: &PathBuf,
        method: &str,
        params: &Value,
    ) -> Result<Value> {
        let start = std::time::Instant::now();

        // Determine protocol based on preference and availability
        let use_tarpc = self.should_use_tarpc(socket_path).await;

        if use_tarpc {
            match self.forward_via_tarpc(socket_path, method, params).await {
                Ok(response) => return Ok(response),
                Err(e) => {
                    debug!(
                        "tarpc forwarding failed for {}, falling back to JSON-RPC: {e}",
                        socket_path.display()
                    );
                }
            }
        }

        debug!(
            "   → Forwarding via JSON-RPC: {} to {}",
            method,
            socket_path.display()
        );

        let client = AtomicClient::unix(socket_path).with_timeout(self.request_timeout);

        let result = client.call(method, params.clone()).await.context(format!(
            "Failed to forward {} to {}",
            method,
            socket_path.display()
        ))?;

        let latency = start.elapsed().as_millis() as u64;
        debug!("   ✓ Forwarded successfully in {}ms", latency);

        if let Some(graph) = &self.living_graph {
            if let Some(primal_name) = socket_path.file_stem().and_then(|s| s.to_str()) {
                graph
                    .record_request("neural-api", primal_name, latency * 1000, true)
                    .await;
            }
        }

        Ok(result)
    }

    /// Forward a request via tarpc binary protocol for high-performance primal communication.
    pub async fn forward_via_tarpc(
        &self,
        socket_path: &std::path::Path,
        method: &str,
        params: &Value,
    ) -> Result<Value, String> {
        use crate::tarpc_client;
        use biomeos_types::tarpc_types::ServiceRegistration;
        use bytes::Bytes;
        use tarpc::context;

        let tarpc_path = biomeos_primal_sdk::tarpc_transport::tarpc_socket_path(socket_path);

        if !tarpc_path.exists() {
            return Err(format!("tarpc socket not found: {}", tarpc_path.display()));
        }

        let ctx = context::current();

        // Health methods
        if method == "health.check" || method == "health_check" {
            let client = tarpc_client::connect_tarpc_health(&tarpc_path)
                .await
                .map_err(|e| format!("tarpc health connect failed: {e}"))?;
            let status = client
                .health_check(ctx)
                .await
                .map_err(|e| format!("tarpc health_check failed: {e}"))?;
            return serde_json::to_value(&status).map_err(|e| e.to_string());
        }
        if method == "health.metrics" || method == "health_metrics" {
            let client = tarpc_client::connect_tarpc_health(&tarpc_path)
                .await
                .map_err(|e| format!("tarpc health connect failed: {e}"))?;
            let metrics = client
                .health_metrics(ctx)
                .await
                .map_err(|e| format!("tarpc health_metrics failed: {e}"))?;
            return serde_json::to_value(&metrics).map_err(|e| e.to_string());
        }
        if method == "health.version" || method == "version" {
            let client = tarpc_client::connect_tarpc_health(&tarpc_path)
                .await
                .map_err(|e| format!("tarpc health connect failed: {e}"))?;
            let info = client
                .version(ctx)
                .await
                .map_err(|e| format!("tarpc version failed: {e}"))?;
            return serde_json::to_value(&info).map_err(|e| e.to_string());
        }

        // Discovery methods
        if method.starts_with("discovery.") || method.starts_with("discovery_") {
            let client = tarpc_client::connect_tarpc_discovery(&tarpc_path)
                .await
                .map_err(|e| format!("tarpc discovery connect failed: {e}"))?;
            match method {
                "discovery.discover" | "discovery_discover" => {
                    let capability = params
                        .get("capability")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let result = client
                        .discover(ctx, capability)
                        .await
                        .map_err(|e| format!("tarpc discovery failed: {e}"))?;
                    return serde_json::to_value(&result).map_err(|e| e.to_string());
                }
                "discovery.discover_all" | "discovery_discover_all" => {
                    let result = client
                        .discover_all(ctx)
                        .await
                        .map_err(|e| format!("tarpc discovery failed: {e}"))?;
                    return serde_json::to_value(&result).map_err(|e| e.to_string());
                }
                "discovery.protocols" | "discovery_protocols" => {
                    let result = client
                        .protocols(ctx)
                        .await
                        .map_err(|e| format!("tarpc discovery failed: {e}"))?;
                    return serde_json::to_value(&result).map_err(|e| e.to_string());
                }
                "discovery.unregister" | "discovery_unregister" => {
                    let primal_id = params
                        .get("primal_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let ok = client
                        .unregister(ctx, primal_id)
                        .await
                        .map_err(|e| e.to_string())?;
                    return serde_json::to_value(ok).map_err(|e| e.to_string());
                }
                "discovery.register" | "discovery_register" => {
                    let reg: ServiceRegistration =
                        serde_json::from_value(params.clone()).map_err(|e| e.to_string())?;
                    let res = client.register(ctx, reg).await.map_err(|e| e.to_string())?;
                    return serde_json::to_value(&res).map_err(|e| e.to_string());
                }
                _ => return Err(format!("unknown discovery method: {method}")),
            };
        }

        // Security methods
        if method.starts_with("security.") || method.starts_with("security_") {
            let client = tarpc_client::connect_tarpc_security(&tarpc_path)
                .await
                .map_err(|e| format!("tarpc security connect failed: {e}"))?;

            let bytes_from_param = |key: &str| -> Result<Bytes, String> {
                let v = params
                    .get(key)
                    .ok_or_else(|| format!("missing param: {key}"))?;
                if let Some(s) = v.as_str() {
                    use base64::Engine;
                    base64::engine::general_purpose::STANDARD
                        .decode(s)
                        .map(Bytes::from)
                        .map_err(|e| e.to_string())
                } else if let Some(arr) = v.as_array() {
                    Ok(arr
                        .iter()
                        .filter_map(|x| x.as_u64().map(|u| u as u8))
                        .collect::<Bytes>())
                } else {
                    Err(format!("param {key} must be base64 string or byte array"))
                }
            };

            match method {
                "security.sign" | "security_sign" => {
                    let data = bytes_from_param("data")?;
                    let result = client
                        .sign(ctx, data)
                        .await
                        .map_err(|e| format!("tarpc security failed: {e}"))?;
                    return serde_json::to_value(&result).map_err(|e| e.to_string());
                }
                "security.verify" | "security_verify" => {
                    let data = bytes_from_param("data")?;
                    let signature = bytes_from_param("signature")?;
                    let public_key = bytes_from_param("public_key")?;
                    let ok = client
                        .verify(ctx, data, signature, public_key)
                        .await
                        .map_err(|e| format!("tarpc security failed: {e}"))?;
                    return serde_json::to_value(ok).map_err(|e| e.to_string());
                }
                "security.get_jwt_secret" | "security_get_jwt_secret" => {
                    let service_name = params
                        .get("service_name")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let result = client
                        .get_jwt_secret(ctx, service_name)
                        .await
                        .map_err(|e| format!("tarpc security failed: {e}"))?;
                    return serde_json::to_value(&result).map_err(|e| e.to_string());
                }
                "security.verify_lineage" | "security_verify_lineage" => {
                    let primal_id = params
                        .get("primal_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("")
                        .to_string();
                    let result = client
                        .verify_lineage(ctx, primal_id)
                        .await
                        .map_err(|e| format!("tarpc security failed: {e}"))?;
                    return serde_json::to_value(&result).map_err(|e| e.to_string());
                }
                _ => return Err(format!("unknown security method: {method}")),
            };
        }

        Err(format!(
            "method {method} has no tarpc mapping, use JSON-RPC"
        ))
    }

    /// Check if tarpc should be used for this request
    pub(super) async fn should_use_tarpc(&self, socket_path: &std::path::Path) -> bool {
        match self.protocol_preference {
            ProtocolPreference::JsonRpcOnly => return false,
            ProtocolPreference::TarpcOnly => return true,
            ProtocolPreference::PreferJsonRpc => return false,
            ProtocolPreference::PreferTarpc | ProtocolPreference::Auto => {}
        }

        if let Some(graph) = &self.living_graph {
            if let Some(primal_name) = socket_path.file_stem().and_then(|s| s.to_str()) {
                if let Some(state) = graph.get_primal_state(primal_name).await {
                    return state.tarpc_available()
                        && matches!(
                            state.current_mode,
                            ProtocolMode::Tarpc | ProtocolMode::Hybrid
                        );
                }
            }
        }

        false
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use crate::living_graph::{LivingGraph, PrimalProtocolState, ProtocolMode};
    use biomeos_types::tarpc_types::ProtocolPreference;
    use std::path::PathBuf;
    use std::sync::Arc;
    use tempfile::TempDir;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;
    use tokio::sync::oneshot;

    fn create_router(family_id: &str) -> NeuralRouter {
        NeuralRouter::new(family_id)
    }

    // --- should_use_tarpc tests ---

    #[tokio::test]
    async fn test_should_use_tarpc_jsonrpc_only_returns_false() {
        let router =
            create_router("test").with_protocol_preference(ProtocolPreference::JsonRpcOnly);
        let path = PathBuf::from("/tmp/test-primal.sock");
        assert!(!router.should_use_tarpc(&path).await);
    }

    #[tokio::test]
    async fn test_should_use_tarpc_tarpc_only_returns_true() {
        let router = create_router("test").with_protocol_preference(ProtocolPreference::TarpcOnly);
        let path = PathBuf::from("/tmp/test-primal.sock");
        assert!(router.should_use_tarpc(&path).await);
    }

    #[tokio::test]
    async fn test_should_use_tarpc_prefer_jsonrpc_returns_false() {
        let router =
            create_router("test").with_protocol_preference(ProtocolPreference::PreferJsonRpc);
        let path = PathBuf::from("/tmp/test-primal.sock");
        assert!(!router.should_use_tarpc(&path).await);
    }

    #[tokio::test]
    async fn test_should_use_tarpc_prefer_tarpc_no_graph_returns_false() {
        let router =
            create_router("test").with_protocol_preference(ProtocolPreference::PreferTarpc);
        let path = PathBuf::from("/tmp/test-primal.sock");
        // No living graph, so falls through to false
        assert!(!router.should_use_tarpc(&path).await);
    }

    #[tokio::test]
    async fn test_should_use_tarpc_auto_with_graph_tarpc_available() {
        let temp = TempDir::new().expect("temp dir");
        let json_sock = temp.path().join("beardog.sock");
        let tarpc_sock = temp.path().join("beardog.tarpc.sock");
        let _ = std::fs::File::create(&tarpc_sock);

        let graph = Arc::new(LivingGraph::new("test"));
        let mut state = PrimalProtocolState::new("beardog", json_sock.clone())
            .with_tarpc_socket(tarpc_sock)
            .with_capabilities(vec!["security".to_string()]);
        state.current_mode = ProtocolMode::Tarpc;
        graph.register_primal(state).await;

        let router = create_router("test")
            .with_protocol_preference(ProtocolPreference::Auto)
            .with_living_graph(graph);

        assert!(router.should_use_tarpc(&json_sock).await);
    }

    #[tokio::test]
    async fn test_should_use_tarpc_auto_with_graph_jsonrpc_mode_returns_false() {
        let temp = TempDir::new().expect("temp dir");
        let json_sock = temp.path().join("beardog.sock");
        let tarpc_sock = temp.path().join("beardog.tarpc.sock");
        let _ = std::fs::File::create(&tarpc_sock);

        let graph = Arc::new(LivingGraph::new("test"));
        let state = PrimalProtocolState::new("beardog", json_sock.clone())
            .with_tarpc_socket(tarpc_sock)
            .with_capabilities(vec!["security".to_string()]);
        // Default mode is JsonRpc - not Tarpc or Hybrid
        graph.register_primal(state).await;

        let router = create_router("test")
            .with_protocol_preference(ProtocolPreference::Auto)
            .with_living_graph(graph);

        // JsonRpc mode -> returns false (only Tarpc or Hybrid use tarpc)
        assert!(!router.should_use_tarpc(&json_sock).await);
    }

    #[tokio::test]
    async fn test_should_use_tarpc_auto_with_graph_tarpc_mode() {
        let temp = TempDir::new().expect("temp dir");
        let json_sock = temp.path().join("beardog.sock");
        let tarpc_sock = temp.path().join("beardog.tarpc.sock");
        let _ = std::fs::File::create(&tarpc_sock);

        let graph = Arc::new(LivingGraph::new("test"));
        let mut state = PrimalProtocolState::new("beardog", json_sock.clone())
            .with_tarpc_socket(tarpc_sock)
            .with_capabilities(vec!["security".to_string()]);
        state.current_mode = ProtocolMode::Tarpc;
        graph.register_primal(state).await;

        let router = create_router("test")
            .with_protocol_preference(ProtocolPreference::Auto)
            .with_living_graph(graph);

        assert!(router.should_use_tarpc(&json_sock).await);
    }

    #[tokio::test]
    async fn test_should_use_tarpc_auto_with_graph_hybrid_mode() {
        let temp = TempDir::new().expect("temp dir");
        let json_sock = temp.path().join("beardog.sock");
        let tarpc_sock = temp.path().join("beardog.tarpc.sock");
        let _ = std::fs::File::create(&tarpc_sock);

        let graph = Arc::new(LivingGraph::new("test"));
        let mut state = PrimalProtocolState::new("beardog", json_sock.clone())
            .with_tarpc_socket(tarpc_sock)
            .with_capabilities(vec!["security".to_string()]);
        state.current_mode = ProtocolMode::Hybrid;
        graph.register_primal(state).await;

        let router = create_router("test")
            .with_protocol_preference(ProtocolPreference::Auto)
            .with_living_graph(graph);

        assert!(router.should_use_tarpc(&json_sock).await);
    }

    #[tokio::test]
    async fn test_should_use_tarpc_auto_with_graph_no_tarpc_socket_returns_false() {
        let temp = TempDir::new().expect("temp dir");
        let json_sock = temp.path().join("beardog.sock");
        // No tarpc socket - tarpc_available() is false
        let graph = Arc::new(LivingGraph::new("test"));
        let state = PrimalProtocolState::new("beardog", json_sock.clone());
        graph.register_primal(state).await;

        let router = create_router("test")
            .with_protocol_preference(ProtocolPreference::Auto)
            .with_living_graph(graph);

        assert!(!router.should_use_tarpc(&json_sock).await);
    }

    // --- forward_via_tarpc error path tests ---

    #[tokio::test]
    async fn test_forward_via_tarpc_socket_not_found() {
        let router = create_router("test");
        let temp = TempDir::new().expect("temp dir");
        let socket_path = temp.path().join("nonexistent.sock");
        // No socket file - tarpc path won't exist
        let result = router
            .forward_via_tarpc(&socket_path, "health.check", &serde_json::json!({}))
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("tarpc socket not found") || err.contains("not found"));
    }

    #[tokio::test]
    async fn test_forward_via_tarpc_discovery_method_requires_tarpc_server() {
        // discovery.* methods require a real tarpc server; without one we get connect error
        let router = create_router("test");
        let temp = TempDir::new().expect("temp dir");
        let socket_path = temp.path().join("primal.sock");
        let tarpc_path = temp.path().join("primal.tarpc.sock");
        let _ = std::fs::File::create(&tarpc_path);

        let result = router
            .forward_via_tarpc(
                &socket_path,
                "discovery.unknown_method",
                &serde_json::json!({}),
            )
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("discovery") || err.contains("connect") || err.contains("tarpc"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn test_forward_via_tarpc_security_method_requires_tarpc_server() {
        // security.* methods require a real tarpc server; without one we get connect error
        let router = create_router("test");
        let temp = TempDir::new().expect("temp dir");
        let socket_path = temp.path().join("primal.sock");
        let tarpc_path = temp.path().join("primal.tarpc.sock");
        let _ = std::fs::File::create(&tarpc_path);

        let result = router
            .forward_via_tarpc(
                &socket_path,
                "security.unknown_method",
                &serde_json::json!({}),
            )
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(
            err.contains("security") || err.contains("connect") || err.contains("tarpc"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn test_forward_via_tarpc_no_tarpc_mapping() {
        let router = create_router("test");
        let temp = TempDir::new().expect("temp dir");
        let socket_path = temp.path().join("primal.sock");
        let tarpc_path = temp.path().join("primal.tarpc.sock");
        let _ = std::fs::File::create(&tarpc_path);

        let result = router
            .forward_via_tarpc(&socket_path, "custom.unknown", &serde_json::json!({}))
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.contains("no tarpc mapping"));
    }

    // --- forward_request tests (JSON-RPC path) ---

    async fn run_mock_jsonrpc_server(
        socket_path: &std::path::Path,
        response: serde_json::Value,
        ready_tx: Option<oneshot::Sender<()>>,
    ) -> tokio::task::JoinHandle<()> {
        let path = socket_path.to_path_buf();
        let response_json = serde_json::to_string(&response).expect("serialize");

        tokio::spawn(async move {
            let listener = UnixListener::bind(&path).expect("bind");
            if let Some(tx) = ready_tx {
                let _ = tx.send(());
            }
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = vec![0u8; 4096];
                let n = stream.read(&mut buf).await.expect("read");
                let _request = &buf[..n];

                let response_line = format!("{response_json}\n");
                stream
                    .write_all(response_line.as_bytes())
                    .await
                    .expect("write");
                stream.flush().await.expect("flush");
            }
        })
    }

    #[tokio::test]
    async fn test_forward_request_jsonrpc_success() {
        let temp = TempDir::new().expect("temp dir");
        let socket_path = temp.path().join("test-primal.sock");
        let rpc_response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {"healthy": true, "uptime_secs": 42},
            "id": 1
        });

        let (ready_tx, ready_rx) = oneshot::channel();
        let _server = run_mock_jsonrpc_server(&socket_path, rpc_response, Some(ready_tx)).await;
        ready_rx.await.expect("server ready");

        let router =
            create_router("test").with_protocol_preference(ProtocolPreference::JsonRpcOnly);

        let result = router
            .forward_request(&socket_path, "health.check", &serde_json::json!({}))
            .await;

        assert!(result.is_ok());
        let value = result.unwrap();
        assert_eq!(value["healthy"], true);
        assert_eq!(value["uptime_secs"], 42);
    }

    #[tokio::test]
    async fn test_forward_request_jsonrpc_socket_not_found() {
        let temp = TempDir::new().expect("temp dir");
        let socket_path = temp.path().join("nonexistent.sock");

        let router =
            create_router("test").with_protocol_preference(ProtocolPreference::JsonRpcOnly);

        let result = router
            .forward_request(&socket_path, "health.check", &serde_json::json!({}))
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("Failed to forward")
                || err.contains("connect")
                || err.contains("No such file"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn test_forward_request_tarpc_fallback_to_jsonrpc() {
        // When tarpc is preferred but fails (no tarpc socket), should fall back to JSON-RPC
        let temp = TempDir::new().expect("temp dir");
        let socket_path = temp.path().join("test-primal.sock");
        let rpc_response = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {"ok": true},
            "id": 1
        });

        let (ready_tx, ready_rx) = oneshot::channel();
        let _server = run_mock_jsonrpc_server(&socket_path, rpc_response, Some(ready_tx)).await;
        ready_rx.await.expect("server ready");

        let router =
            create_router("test").with_protocol_preference(ProtocolPreference::PreferTarpc);
        // No tarpc socket - will try tarpc, fail, fall back to JSON-RPC

        let result = router
            .forward_request(&socket_path, "some.method", &serde_json::json!({}))
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap()["ok"], true);
    }
}
