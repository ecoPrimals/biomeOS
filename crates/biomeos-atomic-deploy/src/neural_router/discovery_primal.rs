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
        let socket_path = nucleation.assign_socket_with_runtime_dir(
            primal_name,
            &self.family_id,
            xdg_runtime_parent,
        );

        if !socket_path.exists() {
            return Err(anyhow!(
                "Primal '{}' not found: socket {} does not exist",
                primal_name,
                socket_path.display()
            ));
        }

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
