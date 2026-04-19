// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Auto-discovery of running primals and capability registration on startup / rescan.

use tracing::{debug, info, warn};

use super::NeuralApiServer;

/// Probe a TCP endpoint for capabilities via JSON-RPC `capabilities.list`.
pub(crate) async fn probe_tcp_capabilities_public(addr: &str) -> Vec<String> {
    probe_tcp_capabilities(addr).await
}

async fn probe_tcp_capabilities(addr: &str) -> Vec<String> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::TcpStream;
    use tokio::time::{Duration, timeout};

    let request = serde_json::json!({
        "jsonrpc": "2.0",
        "method": "capabilities.list",
        "params": {},
        "id": 1
    });

    let Ok(Ok(mut stream)) = timeout(Duration::from_millis(500), TcpStream::connect(addr)).await
    else {
        return vec![];
    };

    let payload = format!("{}\n", request);
    if stream.write_all(payload.as_bytes()).await.is_err() {
        return vec![];
    }

    let mut buf = vec![0u8; 8192];
    let Ok(Ok(n)) = timeout(Duration::from_secs(2), stream.read(&mut buf)).await else {
        return vec![];
    };

    let Ok(resp) = serde_json::from_slice::<serde_json::Value>(&buf[..n]) else {
        return vec![];
    };

    resp.get("result")
        .and_then(|r| r.get("capabilities"))
        .and_then(|c| c.as_array())
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect()
        })
        .or_else(|| {
            resp.get("result").and_then(|r| r.as_array()).map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
        })
        .unwrap_or_default()
}

impl NeuralApiServer {
    /// Scan for running primals and register their capabilities.
    ///
    /// **UDS mode** (default): scans `$XDG_RUNTIME_DIR/biomeos/` for `.sock` files,
    /// probes each via `capabilities.list`, and registers Unix endpoints.
    ///
    /// **TCP-only mode**: probes well-known TCP ports (9900+) for primals that were
    /// spawned with `--port` by the graph executor. Registers TCP endpoints.
    ///
    /// This is the sovereign auto-discovery path: no startup ordering dependency.
    pub(crate) async fn discover_and_register_primals(&self) {
        let mut total_caps = 0usize;
        let mut total_primals = 0usize;

        if self.tcp_only {
            // TCP-only: probe ports 9900..9920 for primals launched by bootstrap
            let host: std::sync::Arc<str> = std::env::var("BIOMEOS_BIND_ADDRESS")
                .unwrap_or_else(|_| {
                    biomeos_types::constants::endpoints::DEFAULT_LOCALHOST.to_string()
                })
                .into();

            for port in 9900..9920u16 {
                let addr = format!("{}:{}", &host, port);
                let capabilities = probe_tcp_capabilities(&addr).await;
                if capabilities.is_empty() {
                    continue;
                }

                let primal_name = capabilities
                    .first()
                    .map(|c| c.split('.').next().unwrap_or("unknown"))
                    .unwrap_or("unknown")
                    .to_string();

                let endpoint = biomeos_core::TransportEndpoint::TcpSocket {
                    host: host.clone(),
                    port,
                };

                for cap in &capabilities {
                    if let Err(e) = self
                        .router
                        .register_capability(cap, &primal_name, endpoint.clone(), "tcp-discovery")
                        .await
                    {
                        warn!("   Failed to register {}.{}: {}", primal_name, cap, e);
                    }
                }

                info!(
                    "   🔍 Discovered {} — {} capabilities via TCP :{port}",
                    primal_name,
                    capabilities.len(),
                );
                total_caps += capabilities.len();
                total_primals += 1;
            }
        } else {
            // UDS mode: scan socket directories
            let socket_dirs = crate::handlers::TopologyHandler::get_socket_directories();

            for socket_dir in &socket_dirs {
                let entries = match std::fs::read_dir(socket_dir) {
                    Ok(e) => e,
                    Err(_) => continue,
                };

                for entry in entries.flatten() {
                    let path = entry.path();
                    let filename = match path.file_name().and_then(|n| n.to_str()) {
                        Some(f) => f.to_string(),
                        None => continue,
                    };

                    if !filename.ends_with(".sock") {
                        continue;
                    }

                    if path == self.socket_path {
                        continue;
                    }

                    let primal_name = match filename.strip_suffix(".sock") {
                        Some(base) => base.split('-').next().unwrap_or(base).to_string(),
                        None => continue,
                    };

                    let socket_str = path.to_string_lossy().to_string();
                    let capabilities = self.probe_primal_capabilities(&socket_str).await;

                    if capabilities.is_empty() {
                        debug!("   {} — no capabilities (not responsive?)", primal_name);
                        continue;
                    }

                    for cap in &capabilities {
                        if let Err(e) = self
                            .router
                            .register_capability_unix(cap, &primal_name, &path, "auto-discovery")
                            .await
                        {
                            warn!("   Failed to register {}.{}: {}", primal_name, cap, e);
                        }
                    }

                    info!(
                        "   🔍 Discovered {} — {} capabilities via {}",
                        primal_name,
                        capabilities.len(),
                        socket_str,
                    );
                    total_caps += capabilities.len();
                    total_primals += 1;
                }
            }
        }

        if total_primals > 0 {
            info!(
                "✅ Auto-discovery registered {} capabilities from {} primals",
                total_caps, total_primals
            );
        } else {
            info!("🔍 Auto-discovery: no running primals found (they will register dynamically)");
        }
    }

    /// Probe a primal's UDS socket for its capabilities via `capabilities.list`.
    ///
    /// Delegates to `probe_primal_capabilities_standalone` (shared with lazy
    /// rescan). Returns an empty vec on connection failure (non-fatal).
    pub(crate) async fn probe_primal_capabilities(&self, socket_path: &str) -> Vec<String> {
        crate::neural_router::probe_primal_capabilities_standalone(socket_path).await
    }

    /// Re-scan socket directories and register any newly-appeared primals.
    ///
    /// JSON-RPC method: `topology.rescan`
    ///
    /// Use case: deploy biomeOS into an existing system with running primals,
    /// or trigger re-discovery after new primals start. This is the on-demand
    /// complement to startup auto-discovery (Option 1) and `capability.register`
    /// (Option 2). All three paths converge at the same `NeuralRouter`.
    pub(crate) async fn rescan_primals(&self) -> anyhow::Result<serde_json::Value> {
        let before = self.router.list_capabilities().await.len();
        self.discover_and_register_primals().await;
        self.router.reset_lazy_rescan();
        let after = self.router.list_capabilities().await.len();
        let new_caps = after.saturating_sub(before);
        Ok(serde_json::json!({
            "rescanned": true,
            "new_capabilities_registered": new_caps,
            "total_capabilities": after,
        }))
    }
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test")]
mod tests {
    use super::super::NeuralApiServer;

    #[tokio::test]
    async fn test_rescan_primals_returns_json_shape() {
        let temp = tempfile::tempdir().expect("tempdir");
        let sock = temp.path().join("rescan.sock");
        let server = NeuralApiServer::new(temp.path(), "fam-rescan", sock);
        let v = server.rescan_primals().await.expect("rescan");
        assert_eq!(v["rescanned"], true);
        assert!(v.get("new_capabilities_registered").is_some());
        assert!(v.get("total_capabilities").is_some());
    }
}
