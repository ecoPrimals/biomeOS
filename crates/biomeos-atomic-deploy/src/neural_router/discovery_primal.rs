// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Socket-based primal lookup and transport health probing for discovery.

use anyhow::{Context, Result, anyhow};
use std::sync::Arc;
use tracing::debug;

use crate::nucleation::SocketNucleation;
use biomeos_core::TransportEndpoint;
use biomeos_core::atomic_client::AtomicClient;

use super::NeuralRouter;
use super::types::DiscoveredPrimal;

impl NeuralRouter {
    /// Find primal by socket pattern (runtime discovery)
    pub(crate) async fn find_primal_by_socket(
        &self,
        primal_name: &str,
    ) -> Result<DiscoveredPrimal> {
        self.find_primal_by_socket_with_runtime_dir(primal_name, None)
            .await
    }

    /// Like [`Self::find_primal_by_socket`], but supplies `$XDG_RUNTIME_DIR` parent explicitly (tests).
    pub(crate) async fn find_primal_by_socket_with_runtime_dir(
        &self,
        primal_name: &str,
        xdg_runtime_parent: Option<&std::path::Path>,
    ) -> Result<DiscoveredPrimal> {
        {
            let cache = self.discovered_primals.read().await;
            if let Some(primal) = cache.get(primal_name) {
                debug!("   📦 Cache hit: {}", primal_name);
                return Ok(primal.clone());
            }
        }

        let mut nucleation = SocketNucleation::default();
        let family_suffixed = nucleation.assign_socket_with_runtime_dir(
            primal_name,
            &self.family_id,
            xdg_runtime_parent,
        );

        // Try family-suffixed path first, then unsuffixed, then full resolve_primal_socket
        let socket_path = if family_suffixed.exists() {
            family_suffixed
        } else {
            let unsuffixed = family_suffixed
                .parent()
                .map(|dir| dir.join(format!("{primal_name}.sock")))
                .unwrap_or_else(|| std::path::PathBuf::from(format!("{primal_name}.sock")));
            if unsuffixed.exists() {
                debug!(
                    "   📍 {} found at unsuffixed path: {}",
                    primal_name,
                    unsuffixed.display()
                );
                unsuffixed
            } else {
                let resolved =
                    std::path::PathBuf::from(crate::capability_translation::resolve_primal_socket(
                        primal_name,
                        &self.family_id,
                    ));
                if resolved.exists() {
                    resolved
                } else {
                    return Err(anyhow!(
                        "Primal '{}' not found: tried {} and {}",
                        primal_name,
                        family_suffixed.display(),
                        unsuffixed.display(),
                    ));
                }
            }
        };

        let endpoint = TransportEndpoint::UnixSocket {
            path: socket_path.clone(),
        };
        let healthy = self.quick_health_check(&endpoint).await;

        let primal = DiscoveredPrimal {
            name: Arc::from(primal_name),
            endpoint: endpoint.clone(),
            capabilities: vec![],
            healthy,
            last_check: chrono::Utc::now(),
        };

        {
            let mut cache = self.discovered_primals.write().await;
            cache.insert(primal_name.to_string(), primal.clone());
        }

        debug!(
            "   ✅ Discovered: {} @ {} (healthy: {})",
            primal_name,
            endpoint.display_string(),
            healthy
        );

        Ok(primal)
    }

    /// Refresh a stale endpoint for a primal by re-scanning the socket directory.
    ///
    /// Called after a forward failure to a graph-bootstrap registered endpoint.
    /// If the primal's actual socket is at a different path than what the registry
    /// recorded (e.g., unsuffixed vs family-suffixed), this finds and updates it.
    ///
    /// Returns `Some(new_endpoint)` if a live socket was found, `None` otherwise.
    pub(crate) async fn refresh_stale_endpoint(
        &self,
        primal_name: &str,
    ) -> Option<TransportEndpoint> {
        let socket_dirs = crate::handlers::TopologyHandler::get_socket_directories();

        for dir in &socket_dirs {
            let entries = match std::fs::read_dir(dir) {
                Ok(e) => e,
                Err(_) => continue,
            };

            for entry in entries.flatten() {
                let path = entry.path();
                let filename = match path.file_name().and_then(|f| f.to_str()) {
                    Some(f) => f,
                    None => continue,
                };

                if !filename.ends_with(".sock") {
                    continue;
                }

                let stem = match filename.strip_suffix(".sock") {
                    Some(s) => s,
                    None => continue,
                };

                // Match: exact primal name or primal-{anything} prefix
                let is_match = stem == primal_name
                    || stem.starts_with(&format!("{primal_name}-"))
                    || stem.starts_with(&format!("{primal_name}."));

                if !is_match {
                    continue;
                }

                let endpoint = TransportEndpoint::UnixSocket { path: path.clone() };
                if Self::check_endpoint_health(&endpoint).await {
                    debug!(
                        "   🔄 Refreshed stale endpoint for {}: {}",
                        primal_name,
                        path.display()
                    );

                    // Update discovery cache
                    {
                        let mut cache = self.discovered_primals.write().await;
                        if let Some(cached) = cache.get_mut(primal_name) {
                            cached.endpoint = endpoint.clone();
                            cached.healthy = true;
                            cached.last_check = chrono::Utc::now();
                        }
                    }

                    // Update capability registry
                    let registry = self.capability_registry.read().await;
                    let caps_to_update: Vec<String> = registry
                        .iter()
                        .filter(|(_cap, providers)| {
                            providers
                                .iter()
                                .any(|p| p.primal_name.as_ref() == primal_name)
                        })
                        .map(|(cap, _)| cap.clone())
                        .collect();
                    drop(registry);

                    for cap in caps_to_update {
                        let _ = self
                            .register_capability_unix(&cap, primal_name, &path, "endpoint-refresh")
                            .await;
                    }

                    return Some(endpoint);
                }
            }
        }

        None
    }

    /// Transport-aware health check via `AtomicClient`
    ///
    /// Any successful JSON-RPC response means the primal is alive. Only
    /// connection failures, timeouts, or JSON-RPC error responses indicate death.
    /// This prevents "socket evaporation" where primals that respond with
    /// non-standard formats (e.g. `{"status":"alive"}` instead of `{"healthy":true}`)
    /// are falsely marked dead.
    ///
    /// Dual-protocol: tries plain JSON-RPC first, BTSP fallback for secure primals.
    pub(crate) async fn quick_health_check(&self, endpoint: &TransportEndpoint) -> bool {
        let health_timeout = std::time::Duration::from_millis(500);

        let client = AtomicClient::from_endpoint(endpoint.clone()).with_timeout(health_timeout);

        // Plain JSON-RPC first (works for most primals)
        if client
            .call("health.check", serde_json::json!({}))
            .await
            .is_ok()
        {
            return true;
        }

        // BTSP fallback (secure primals)
        match client
            .call_btsp("health.check", serde_json::json!({}))
            .await
        {
            Ok(_) => true,
            Err(_) => {
                debug!(
                    "   ⚠️ Health check failed for {} (both plain and BTSP)",
                    endpoint.display_string()
                );
                false
            }
        }
    }

    /// Transport-aware health check (static, for use without `&self`)
    ///
    /// Dual-protocol: tries plain JSON-RPC first, BTSP fallback. Any successful
    /// response = alive. Connection failures and timeouts indicate death.
    pub(crate) async fn check_endpoint_health(endpoint: &TransportEndpoint) -> bool {
        use tokio::time::{Duration, timeout};

        let probe = async {
            let client =
                AtomicClient::from_endpoint(endpoint.clone()).with_timeout(Duration::from_secs(2));

            // Plain JSON-RPC first (works for most primals)
            if client
                .call("health.check", serde_json::json!({}))
                .await
                .is_ok()
            {
                return Ok::<bool, anyhow::Error>(true);
            }

            // BTSP fallback (secure primals)
            client
                .call_btsp("health.check", serde_json::json!({}))
                .await
                .context("health.check call failed (both plain and BTSP)")?;
            Ok(true)
        };

        match timeout(Duration::from_secs(3), probe).await {
            Ok(Ok(alive)) => alive,
            _ => false,
        }
    }
}
