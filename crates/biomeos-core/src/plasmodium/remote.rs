// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Remote gate queries via HTTP JSON-RPC (Songbird gateway).

use anyhow::Result;
use biomeos_types::primal_names;
use serde_json::{Value, json};

use crate::atomic_client::AtomicClient;

use super::types::{BondType, ComputeInfo, GateInfo, PrimalStatus};

/// Parse `host:port` from a mesh peer address; uses `default_port` when no port is present or when
/// the port segment fails to parse as `u16`.
#[must_use]
pub(crate) fn parse_mesh_peer_address(address: &str, default_port: u16) -> (String, u16) {
    if let Some(idx) = address.rfind(':') {
        let h = &address[..idx];
        let p = address[idx + 1..].parse::<u16>().unwrap_or(default_port);
        (h.to_string(), p)
    } else {
        (address.to_string(), default_port)
    }
}

impl super::Plasmodium {
    /// Query a remote gate's NUCLEUS status via HTTP JSON-RPC gateway
    ///
    /// Uses HTTP POST to `/jsonrpc` on the remote discovery provider.
    /// The port is runtime-discovered from the `mesh.peers` response
    /// (beacon exchange), with env var and constants as fallbacks.
    ///
    /// Fetches health, primals, compute info, and load from the remote NUCLEUS.
    pub(crate) async fn query_remote_gate(&self, address: &str, node_id: &str) -> Result<GateInfo> {
        let default_port: u16 = std::env::var(biomeos_types::env_config::vars::MESH_PORT)
            .ok()
            .and_then(|p| p.parse().ok())
            .unwrap_or(biomeos_types::constants::network::DEFAULT_HTTP_PORT);

        let (host, port) = parse_mesh_peer_address(address, default_port);

        let client = AtomicClient::http(&host, port);

        let health_result: Result<Value> = client.call("health", json!({})).await;
        let reachable = health_result.is_ok();

        if !reachable {
            anyhow::bail!("Gate {node_id} not reachable at {host}:{port}");
        }

        let primals = Self::query_remote_primals(&client).await;
        let compute = Self::query_remote_compute(&client, node_id).await;
        let load = Self::query_remote_load(&client).await;

        Ok(GateInfo {
            gate_id: node_id.to_string(),
            address: address.to_string(),
            is_local: false,
            primals,
            compute,
            models: vec![],
            load,
            reachable: true,
            bond_type: BondType::Covalent,
        })
    }

    /// Query remote compute info via `system.compute` RPC.
    /// Falls back to empty `ComputeInfo` if the remote doesn't support the call.
    async fn query_remote_compute(client: &AtomicClient, node_id: &str) -> ComputeInfo {
        let Ok(result) = client.call("system.compute", json!({})).await else {
            return ComputeInfo::default();
        };

        let gpus = result
            .get("gpus")
            .and_then(|g| g.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|g| {
                        Some(super::types::GpuInfo {
                            name: g.get("name")?.as_str()?.to_string(),
                            vram_mb: g.get("vram_mb").and_then(Value::as_u64).unwrap_or(0),
                            gate_id: node_id.to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default();

        let ram_gb = result.get("ram_gb").and_then(Value::as_u64).unwrap_or(0);
        let cpu_cores = result.get("cpu_cores").and_then(Value::as_u64).unwrap_or(0) as usize;

        ComputeInfo {
            gpus,
            ram_gb,
            cpu_cores,
        }
    }

    /// Query remote system load via `system.load` RPC.
    /// Falls back to 0.0 if the remote doesn't support the call.
    async fn query_remote_load(client: &AtomicClient) -> f64 {
        client
            .call("system.load", json!({}))
            .await
            .ok()
            .and_then(|v| v.get("load").and_then(Value::as_f64))
            .unwrap_or(0.0)
    }

    /// Query remote primals via Songbird TCP
    pub(crate) async fn query_remote_primals(client: &AtomicClient) -> Vec<PrimalStatus> {
        let mut primals = Vec::new();

        // Try lifecycle.status first (if neural API is running)
        if let Ok(result) = client.call("lifecycle.status", json!({})).await
            && let Some(services) = result.get("services").and_then(|s| s.as_object())
        {
            for (name, status) in services {
                primals.push(PrimalStatus {
                    name: name.clone(),
                    healthy: status
                        .get("status")
                        .and_then(|s| s.as_str())
                        .is_some_and(|s| s == "healthy"),
                    version: status
                        .get("version")
                        .and_then(|v| v.as_str())
                        .map(std::string::ToString::to_string),
                });
            }
        }

        if primals.is_empty() {
            let discovery_provider =
                std::env::var(biomeos_types::env_config::vars::DISCOVERY_PROVIDER)
                    .ok()
                    .or_else(|| {
                        biomeos_types::capability_taxonomy::CapabilityTaxonomy::resolve_to_primal(
                            "discovery",
                        )
                        .map(String::from)
                    })
                    .unwrap_or_else(|| primal_names::SONGBIRD.to_string());
            primals.push(PrimalStatus {
                name: discovery_provider,
                healthy: true,
                version: None,
            });
        }

        primals
    }
}

#[cfg(test)]
mod tests {
    #![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

    use std::sync::Arc;
    use std::time::Duration;

    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpListener;
    use tokio::time;

    use biomeos_types::env_config::vars;
    use biomeos_types::primal_names;
    use serde_json::json;

    use crate::atomic_client::AtomicClient;
    use crate::plasmodium::Plasmodium;
    use crate::plasmodium::types::BondType;

    use super::parse_mesh_peer_address;

    fn http_json_response(body: &str) -> String {
        format!(
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        )
    }

    fn jsonrpc_result(result: &serde_json::Value) -> String {
        serde_json::json!({
            "jsonrpc": "2.0",
            "result": result,
            "id": 1
        })
        .to_string()
    }

    fn jsonrpc_method_from_http_request(raw: &str) -> Option<String> {
        let body = raw
            .split("\r\n\r\n")
            .nth(1)
            .or_else(|| raw.split("\n\n").nth(1))?;
        serde_json::from_str::<serde_json::Value>(body)
            .ok()?
            .get("method")?
            .as_str()
            .map(str::to_owned)
    }

    async fn start_jsonrpc_server<F>(handler: F) -> u16
    where
        F: Fn(&str) -> String + Send + Sync + 'static,
    {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        let handler = Arc::new(handler);
        tokio::spawn(async move {
            loop {
                let Ok((mut stream, _)) = listener.accept().await else {
                    break;
                };
                let handler = Arc::clone(&handler);
                tokio::spawn(async move {
                    let mut buf = vec![0u8; 16_384];
                    let n = stream.read(&mut buf).await.unwrap_or(0);
                    let method = jsonrpc_method_from_http_request(
                        std::str::from_utf8(&buf[..n]).unwrap_or(""),
                    )
                    .unwrap_or_default();
                    let body = handler(&method);
                    let _ = stream.write_all(http_json_response(&body).as_bytes()).await;
                });
            }
        });
        tokio::task::yield_now().await;
        port
    }

    async fn start_hanging_jsonrpc_server() -> u16 {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            loop {
                let Ok((mut stream, _)) = listener.accept().await else {
                    break;
                };
                tokio::spawn(async move {
                    let mut buf = vec![0u8; 16_384];
                    let _ = stream.read(&mut buf).await;
                    time::sleep(Duration::from_secs(60)).await;
                });
            }
        });
        tokio::task::yield_now().await;
        port
    }

    #[test]
    fn parse_mesh_peer_host_and_port() {
        let (h, p) = parse_mesh_peer_address("10.0.0.1:9443", 8080);
        assert_eq!(h, "10.0.0.1");
        assert_eq!(p, 9443);
    }

    #[test]
    fn parse_mesh_peer_no_port_uses_default() {
        let (h, p) = parse_mesh_peer_address("gateway.local", 8080);
        assert_eq!(h, "gateway.local");
        assert_eq!(p, 8080);
    }

    #[test]
    fn parse_mesh_peer_invalid_port_uses_default() {
        let (h, p) = parse_mesh_peer_address("host:999999", 3000);
        assert_eq!(h, "host");
        assert_eq!(p, 3000);
    }

    #[test]
    fn parse_mesh_peer_last_colon_splits_port() {
        let (h, p) = parse_mesh_peer_address("a:b:c:9000", 1);
        assert_eq!(h, "a:b:c");
        assert_eq!(p, 9000);
    }

    #[tokio::test]
    async fn query_remote_gate_connection_failure() {
        let p = Plasmodium::new();
        let err = p
            .query_remote_gate("127.0.0.1:59998", "offline-gate")
            .await
            .expect_err("unreachable gate should fail");
        let msg = err.to_string();
        assert!(msg.contains("offline-gate"), "{msg}");
        assert!(msg.contains("not reachable"), "{msg}");
    }

    #[tokio::test]
    async fn query_remote_gate_success_response() {
        let port = start_jsonrpc_server(|method| match method {
            "health" => jsonrpc_result(&json!({"status": "ok"})),
            _ => jsonrpc_result(&json!({})),
        })
        .await;

        temp_env::async_with_vars([(vars::DISCOVERY_PROVIDER, None::<&str>)], async move {
            let p = Plasmodium::new();
            let gate = p
                .query_remote_gate(&format!("127.0.0.1:{port}"), "remote-node")
                .await
                .expect("healthy gate");

            assert_eq!(gate.gate_id, "remote-node");
            assert_eq!(gate.address, format!("127.0.0.1:{port}"));
            assert!(!gate.is_local);
            assert!(gate.reachable);
            assert_eq!(gate.bond_type, BondType::Covalent);
            assert_eq!(gate.primals.len(), 1);
            assert_eq!(gate.primals[0].name, primal_names::SONGBIRD);
            assert!(gate.primals[0].healthy);
        })
        .await;
    }

    #[tokio::test]
    async fn query_remote_gate_malformed_health_response() {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = listener.local_addr().unwrap().port();
        tokio::spawn(async move {
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = vec![0u8; 16_384];
                let _ = stream.read(&mut buf).await;
                let _ = stream
                    .write_all(b"HTTP/1.1 200 OK\r\noops no body sep")
                    .await;
            }
        });
        tokio::task::yield_now().await;

        let p = Plasmodium::new();
        let err = p
            .query_remote_gate(&format!("127.0.0.1:{port}"), "bad-http")
            .await
            .expect_err("malformed health should fail");
        let msg = err.to_string();
        assert!(msg.contains("bad-http"), "{msg}");
        assert!(msg.contains("not reachable"), "{msg}");
    }

    #[tokio::test]
    async fn query_remote_primals_empty_response_uses_discovery_fallback() {
        let port = start_jsonrpc_server(|_method| jsonrpc_result(&json!({}))).await;

        temp_env::async_with_vars([(vars::DISCOVERY_PROVIDER, None::<&str>)], async move {
            let client = AtomicClient::http("127.0.0.1", port);
            let primals = Plasmodium::query_remote_primals(&client).await;

            assert_eq!(primals.len(), 1);
            assert_eq!(primals[0].name, primal_names::SONGBIRD);
            assert!(primals[0].healthy);
            assert!(primals[0].version.is_none());
        })
        .await;
    }

    #[tokio::test]
    async fn query_remote_primals_populated_lifecycle_services() {
        let port = start_jsonrpc_server(|method| match method {
            "lifecycle.status" => jsonrpc_result(&json!({
                "services": {
                    "beardog": {
                        "status": "healthy",
                        "version": "2.1.0"
                    },
                    "toadstool": {
                        "status": "degraded"
                    }
                }
            })),
            _ => jsonrpc_result(&json!({})),
        })
        .await;

        let client = AtomicClient::http("127.0.0.1", port);
        let primals = Plasmodium::query_remote_primals(&client).await;

        assert_eq!(primals.len(), 2);
        let beardog = primals
            .iter()
            .find(|p| p.name == "beardog")
            .expect("beardog");
        assert!(beardog.healthy);
        assert_eq!(beardog.version.as_deref(), Some("2.1.0"));
        let toadstool = primals
            .iter()
            .find(|p| p.name == "toadstool")
            .expect("toadstool");
        assert!(!toadstool.healthy);
        assert!(toadstool.version.is_none());
    }

    #[tokio::test]
    async fn query_remote_primals_timeout_falls_back_to_discovery_provider() {
        let port = start_hanging_jsonrpc_server().await;
        let client = AtomicClient::http("127.0.0.1", port).with_timeout(Duration::from_millis(200));

        temp_env::async_with_vars([(vars::DISCOVERY_PROVIDER, None::<&str>)], async move {
            let started = time::Instant::now();
            let primals = time::timeout(
                Duration::from_secs(2),
                Plasmodium::query_remote_primals(&client),
            )
            .await
            .expect("query should not hang");

            assert!(
                started.elapsed() < Duration::from_secs(1),
                "expected timeout path to finish quickly, took {:?}",
                started.elapsed()
            );
            assert_eq!(primals.len(), 1);
            assert_eq!(primals[0].name, primal_names::SONGBIRD);
        })
        .await;
    }

    #[tokio::test]
    async fn query_remote_primals_respects_discovery_provider_env() {
        let port = start_jsonrpc_server(|_method| jsonrpc_result(&json!({}))).await;

        temp_env::async_with_vars(
            [(vars::DISCOVERY_PROVIDER, Some("custom-discovery"))],
            async move {
                let client = AtomicClient::http("127.0.0.1", port);
                let primals = Plasmodium::query_remote_primals(&client).await;
                assert_eq!(primals.len(), 1);
                assert_eq!(primals[0].name, "custom-discovery");
            },
        )
        .await;
    }
}
