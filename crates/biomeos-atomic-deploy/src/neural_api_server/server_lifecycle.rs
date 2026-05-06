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
    /// Log a startup inventory of all discoverable graphs so that
    /// deployment problems (wrong path, unparseable TOMLs, empty dir)
    /// are immediately visible in logs.
    async fn log_graph_inventory(&self) {
        let graphs_dir = &self.graphs_dir;
        let graphs_dir_exists = graphs_dir.is_dir();

        match self.graph_handler.list().await {
            Ok(list) => {
                let count = list.as_array().map_or(0, Vec::len);
                info!(
                    "📊 Graph inventory: {} graphs available (graphs_dir={} [{}])",
                    count,
                    graphs_dir.display(),
                    if graphs_dir_exists { "OK" } else { "MISSING" },
                );
                if count == 0 && graphs_dir_exists {
                    warn!(
                        "⚠️  graphs_dir exists but graph.list returned 0 graphs — \
                         check that TOML files are parseable as neural_graph or DeploymentGraph"
                    );
                }
                if count == 0 && !graphs_dir_exists {
                    warn!(
                        "⚠️  graphs_dir {} does not exist — graph.list will return [] \
                         and the route table will be empty. \
                         Use --graphs-dir to point to the correct directory.",
                        graphs_dir.display()
                    );
                }
            }
            Err(e) => {
                warn!("⚠️  Failed to list graphs during startup inventory: {e}");
            }
        }

        let cap_count = self.router.list_capabilities().await.len();
        info!(
            "📊 Route table: {} registered capabilities after graph + primal loading",
            cap_count
        );
    }

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
            let addr = biomeos_types::constants::endpoints::tcp_bind_addr_with_host(
                self.bind_address.as_deref(),
                port,
            );
            let listener = tokio::net::TcpListener::bind(addr)
                .await
                .context(format!("Failed to bind TCP listener on {addr}"))?;
            info!("📡 Neural API TCP listener bound: {addr}");
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

        // 4a. ALWAYS load semantic translations from Tower Atomic graph
        self.load_translations_on_startup().await?;

        // 4b. Auto-scan ALL graphs in graphs_dir for capability translations.
        // This ensures primals declared in deployment graphs (not just the bootstrap
        // graph) are registered in the capability router — critical for TCP-only mode
        // where the full route table must be populated before any client connects.
        self.load_translations_from_all_graphs().await;

        // 4c. Pre-register capabilities declared in graph nodes so the route
        // table is populated even before primals are discovered via live sockets.
        // This is the critical bridge: graphs define which primals provide which
        // capabilities, and we register expected socket paths now. Live discovery
        // (step 5) will update endpoints for any primals that are already running.
        self.register_capabilities_from_graphs().await;

        // 4d. Log graph inventory so deployment issues are immediately visible.
        self.log_graph_inventory().await;

        // 5. Auto-discover running primals and register their capabilities
        self.discover_and_register_primals().await;

        // 5b. Derive coordination purpose key from security provider (if reachable)
        self.derive_coordination_key().await;

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
