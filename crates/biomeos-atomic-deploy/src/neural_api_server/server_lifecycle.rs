// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Server lifecycle management: `serve()` orchestration
//!
//! Binds listeners early, runs mode detection, then delegates bootstrap,
//! coordinated join, translation loading, and primal discovery to sibling
//! modules (`bootstrap`, `translation_startup`, `discovery_init`, `listeners`).

use anyhow::{Context, Result};
use tracing::{info, warn};

use super::NeuralApiServer;
use crate::mode::BiomeOsMode;

// Re-export coordinated-mode helpers at the historical `server_lifecycle` path.
#[expect(
    unused_imports,
    reason = "re-exports at historical path for downstream consumers"
)]
pub use super::bootstrap::{
    is_explicit_coordinated_mode, is_explicit_coordinated_mode_str,
    is_explicit_coordinated_mode_with,
};

impl NeuralApiServer {
    /// Start the Neural API server
    ///
    /// Binds the socket **first** so external probes (primalSpring, health
    /// monitors) can connect immediately, then performs mode detection,
    /// bootstrap, and translation loading before accepting requests.
    ///
    /// Supports three transport modes:
    /// - UDS only (default)
    /// - UDS + TCP (when `tcp_port` is set)
    /// - TCP only (when `tcp_only` is true — mobile substrates)
    pub async fn serve(&self) -> Result<()> {
        // 0. Validate BTSP insecure guard (GAP-MATRIX-11)
        if let Err(msg) = biomeos_core::btsp_client::validate_insecure_guard() {
            anyhow::bail!(msg);
        }
        biomeos_core::btsp_client::log_security_posture();

        // 1. Bind listeners EARLY so health probes see us immediately
        let uds_listener = if self.tcp_only {
            info!("📡 TCP-only mode — skipping Unix socket bind");
            None
        } else {
            Some(self.bind_socket()?)
        };

        let tcp_listener = if let Some(port) = self.tcp_port {
            let addr = biomeos_types::constants::endpoints::production_tcp_bind_addr(port);
            let listener = tokio::net::TcpListener::bind(addr)
                .await
                .context(format!("Failed to bind TCP listener on port {port}"))?;
            info!(
                "📡 Neural API TCP listener bound: {}:{}",
                biomeos_types::constants::endpoints::PRODUCTION_BIND_ADDRESS,
                port
            );
            Some(listener)
        } else {
            None
        };

        // Tell the router our own socket so lazy rescan excludes it (GAP-MATRIX-08)
        self.router
            .set_self_socket_path(self.socket_path.clone())
            .await;

        // 2. Detect operating mode
        info!("🔍 Detecting biomeOS operating mode...");
        let detected_mode = BiomeOsMode::detect(&self.family_id).await;
        {
            let mut mode = self.mode.write().await;
            *mode = detected_mode;
        }

        // 3. Bootstrap if needed
        if detected_mode == BiomeOsMode::Bootstrap {
            self.handle_bootstrap_mode().await?;
        } else {
            self.handle_coordinated_mode().await?;
        }

        // Start lifecycle monitoring
        info!("🔍 Starting primal lifecycle monitoring...");
        if let Err(e) = self.lifecycle_handler.start_monitoring().await {
            warn!("⚠️ Failed to start lifecycle monitoring: {}", e);
        }

        // ALWAYS load semantic translations from Tower Atomic graph
        self.load_translations_on_startup().await?;

        // 4b. Auto-scan ALL graphs in graphs_dir for capability translations.
        // This ensures primals declared in deployment graphs (not just the bootstrap
        // graph) are registered in the capability router — critical for TCP-only mode
        // where the full route table must be populated before any client connects.
        self.load_translations_from_all_graphs().await;

        // 5. Auto-discover running primals and register their capabilities
        self.discover_and_register_primals().await;

        // 6. Accept connections on bound listener(s)
        match (uds_listener, tcp_listener) {
            (Some(uds), Some(tcp)) => {
                tokio::select! {
                    r = self.accept_connections(uds) => r,
                    r = self.accept_tcp_connections(tcp) => r,
                }
            }
            (Some(uds), None) => self.accept_connections(uds).await,
            (None, Some(tcp)) => self.accept_tcp_connections(tcp).await,
            (None, None) => {
                anyhow::bail!("No listeners configured — set a socket path or TCP port")
            }
        }
    }
}
