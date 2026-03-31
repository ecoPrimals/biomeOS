// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Server lifecycle management: bootstrap, mode detection, and initialization
//!
//! Handles the server startup sequence including:
//! - Operating mode detection (Bootstrap vs Coordinated)
//! - Bootstrap sequence execution
//! - Mode transitions
//! - Socket setup and listening

use anyhow::{Context, Result};
use tokio::net::UnixListener;
use tracing::{debug, error, info, warn};

use super::NeuralApiServer;
use crate::mode::BiomeOsMode;

/// Bundled bootstrap graph TOML, compiled into the binary so biomeOS can
/// load capability translations even when the filesystem copy is absent.
const BUNDLED_BOOTSTRAP_GRAPH: &str =
    include_str!("../../../../graphs/tower_atomic_bootstrap.toml");

/// Check if mode string indicates explicit coordinated mode
#[must_use]
pub fn is_explicit_coordinated_mode_str(mode: &str) -> bool {
    let m = mode.to_lowercase();
    m == "coordinated" || m == "coord" || m == "join"
}

/// Check if `BIOMEOS_MODE` env var indicates explicit coordinated mode
#[must_use]
pub fn is_explicit_coordinated_mode() -> bool {
    is_explicit_coordinated_mode_with(None)
}

/// Check coordinated mode with optional override (for testing without env mutation).
///
/// - `Some(s)` — use the given mode string
/// - `None` — read from `BIOMEOS_MODE` env var (or false if unset)
#[must_use]
pub fn is_explicit_coordinated_mode_with(env_override: Option<&str>) -> bool {
    let mode: Option<String> = env_override
        .map(String::from)
        .or_else(|| std::env::var("BIOMEOS_MODE").ok());
    mode.as_deref()
        .map(is_explicit_coordinated_mode_str)
        .unwrap_or(false)
}

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
        // 1. Bind listeners EARLY so health probes see us immediately
        let uds_listener = if self.tcp_only {
            info!("📡 TCP-only mode — skipping Unix socket bind");
            None
        } else {
            Some(self.bind_socket()?)
        };

        let tcp_listener = if let Some(port) = self.tcp_port {
            let addr: std::net::SocketAddr = ([0, 0, 0, 0], port).into();
            let listener = tokio::net::TcpListener::bind(addr)
                .await
                .context(format!("Failed to bind TCP listener on port {port}"))?;
            info!("📡 Neural API TCP listener bound: 0.0.0.0:{}", port);
            Some(listener)
        } else {
            None
        };

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

    /// Handle bootstrap mode: execute bootstrap sequence and transition to coordinated
    async fn handle_bootstrap_mode(&self) -> Result<()> {
        info!("🌱 === BIOMEOS BOOTSTRAP MODE ===");
        info!("🌍 No existing ecosystem detected");
        info!("🏗️  Creating ecosystem foundation...");

        // Register biomeOS in its own capability registry
        crate::bootstrap::register_self_in_registry(
            &self.router,
            &self.family_id,
            &self.socket_path,
            &self.mode,
        )
        .await?;

        // Execute bootstrap sequence (germinate Tower Atomic)
        info!("");
        info!("🏰 Germinating Tower Atomic (ecosystem genesis)...");

        // Load translations before executing bootstrap sequence
        let bootstrap_graph_path = self.graphs_dir.join("tower_atomic_bootstrap.toml");
        if bootstrap_graph_path.exists() {
            if let Ok(graph) = crate::neural_graph::Graph::from_toml_file(&bootstrap_graph_path) {
                if let Err(e) = self.load_translations_from_graph(&graph).await {
                    warn!("⚠️  Failed to load translations before bootstrap: {}", e);
                }
            }
        }

        match crate::bootstrap::execute_bootstrap_sequence(
            &self.graphs_dir,
            &self.family_id,
            &self.nucleation,
        )
        .await
        {
            Ok(_) => {
                info!("✅ Tower Atomic genesis complete!");
                info!("🔄 Transitioning to COORDINATED MODE...");

                // Transition to coordinated mode
                if let Err(e) = crate::bootstrap::transition_to_coordinated(&self.family_id).await {
                    error!("⚠️  Mode transition failed: {}", e);
                    warn!("   Continuing in bootstrap mode (Tower Atomic may be unhealthy)");
                } else {
                    // Update mode
                    let mut mode = self.mode.write().await;
                    *mode = BiomeOsMode::Coordinated;
                    info!("✅ biomeOS now operating in COORDINATED MODE (gen 1)");
                }
            }
            Err(e) => {
                error!("❌ Bootstrap sequence failed: {}", e);
                error!("   biomeOS will continue in bootstrap mode");
                error!("   Manual intervention may be required");
            }
        }
        info!("");

        Ok(())
    }

    /// Handle coordinated mode: join existing ecosystem
    async fn handle_coordinated_mode(&self) -> Result<()> {
        info!("🔄 === BIOMEOS COORDINATED MODE ===");
        info!("🏰 Tower Atomic detected or explicit coordinated mode");
        info!("🌍 Joining existing ecosystem");

        // Check if this is explicit coordinated mode (primals will auto-register)
        let explicit_mode = is_explicit_coordinated_mode();

        if explicit_mode {
            // Explicit coordinated mode: don't wait for sockets
            // Primals will register themselves via auto-registration
            info!("📋 Explicit coordinated mode - primals will auto-register");
            info!("   Neural API will accept capability registrations dynamically");
        } else {
            // Auto-detected coordinated mode: establish connection
            if let Err(e) = crate::bootstrap::transition_to_coordinated(&self.family_id).await {
                warn!("⚠️  Failed to establish BTSP tunnel: {}", e);
                warn!("   Operating without inherited security");
            }
        }

        // Register in ecosystem
        crate::bootstrap::register_self_in_registry(
            &self.router,
            &self.family_id,
            &self.socket_path,
            &self.mode,
        )
        .await?;

        Ok(())
    }

    /// Load translations from Tower Atomic graph on startup
    async fn load_translations_on_startup(&self) -> Result<()> {
        // 1. Load hardcoded default translations (always available)
        {
            let mut registry = self.translation_registry.write().await;
            let default_count = registry.load_defaults();
            info!(
                "📚 Loaded {} default capability translations",
                default_count
            );
        }

        // 2. Overlay with config/capability_registry.toml if present
        {
            let config_path = self.graphs_dir.join("../config/capability_registry.toml");
            if config_path.exists() {
                let mut registry = self.translation_registry.write().await;
                match registry.load_from_config(&config_path, |provider, family_id| {
                    crate::capability_translation::resolve_primal_socket(provider, family_id)
                }) {
                    Ok(count) => info!(
                        "📚 Loaded {} translations from capability_registry.toml",
                        count
                    ),
                    Err(e) => warn!("⚠️  Failed to load capability_registry.toml: {}", e),
                }

                // Bridge domain providers into the NeuralRouter so capability.call
                // can discover which primal handles each capability domain.
                if let Ok(config_content) = std::fs::read_to_string(&config_path) {
                    if let Ok(config) = config_content.parse::<toml::Value>() {
                        if let Some(domains) = config.get("domains").and_then(|d| d.as_table()) {
                            let family_id = biomeos_core::family_discovery::get_family_id();
                            for (domain_name, domain_cfg) in domains {
                                let provider = domain_cfg
                                    .get("provider")
                                    .and_then(|v| v.as_str())
                                    .unwrap_or_default();
                                if provider.is_empty() || provider == "*" {
                                    continue;
                                }
                                let socket = crate::capability_translation::resolve_primal_socket(
                                    provider, &family_id,
                                );
                                let caps = domain_cfg
                                    .get("capabilities")
                                    .and_then(|v| v.as_array())
                                    .map(|arr| {
                                        arr.iter().filter_map(|v| v.as_str()).collect::<Vec<_>>()
                                    })
                                    .unwrap_or_default();

                                for cap in caps {
                                    if let Err(e) = self
                                        .router
                                        .register_capability_unix(
                                            cap,
                                            provider,
                                            &socket,
                                            "config_registry",
                                        )
                                        .await
                                    {
                                        warn!(
                                            "⚠️  Failed to register domain capability {} → {}: {}",
                                            cap, provider, e
                                        );
                                    }
                                }
                                info!(
                                    "📝 Registered domain '{}' → {} ({})",
                                    domain_name, provider, socket
                                );
                            }
                        }
                    }
                }
            }
        }

        // 3. Load translations from Tower Atomic graph (filesystem or bundled)
        info!("📝 Loading semantic translations from Tower Atomic graph...");
        let bootstrap_graph_path = self.graphs_dir.join("tower_atomic_bootstrap.toml");
        let graph_result = if bootstrap_graph_path.exists() {
            crate::neural_graph::Graph::from_toml_file(&bootstrap_graph_path)
        } else {
            info!("   Filesystem graph not found — using bundled bootstrap graph");
            crate::neural_graph::Graph::from_toml_str(BUNDLED_BOOTSTRAP_GRAPH)
        };
        match graph_result {
            Ok(graph) => match self.load_translations_from_graph(&graph).await {
                Ok(_) => info!("✅ Semantic translations loaded from graph"),
                Err(e) => warn!("⚠️  Failed to load translations: {}", e),
            },
            Err(e) => warn!("⚠️  Failed to parse bootstrap graph: {}", e),
        }
        Ok(())
    }

    /// Scan `$XDG_RUNTIME_DIR/biomeos/` for running primals, probe each
    /// socket's `capabilities.list`, and register every discovered capability
    /// with the `NeuralRouter`.
    ///
    /// Any `.sock` (except this server's own path) is considered: registration is
    /// gated by a successful capability probe, not by a compiled primal name list.
    ///
    /// This is the sovereign auto-discovery path (Option 1 from ludoSpring V35):
    /// no startup ordering dependency — biomeOS discovers what is already running.
    async fn discover_and_register_primals(&self) {
        let socket_dirs = crate::handlers::TopologyHandler::get_socket_directories();
        let mut total_caps = 0usize;
        let mut total_primals = 0usize;

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

                // Skip our own socket
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
    async fn probe_primal_capabilities(&self, socket_path: &str) -> Vec<String> {
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

    /// Full health status including mode, registered capabilities, and uptime.
    ///
    /// JSON-RPC method: `health.check`
    pub(crate) async fn health_check(&self) -> Result<serde_json::Value> {
        let mode = self.mode.read().await;
        let cap_count = self.router.list_capabilities().await.len();
        Ok(serde_json::json!({
            "status": "healthy",
            "mode": format!("{mode:?}"),
            "family_id": self.family_id,
            "socket_path": self.socket_path.display().to_string(),
            "registered_capabilities": cap_count,
            "version": env!("CARGO_PKG_VERSION"),
        }))
    }

    /// Minimal liveness probe — confirms the process is running and responsive.
    ///
    /// JSON-RPC method: `health.liveness`
    pub(crate) fn health_liveness(&self) -> Result<serde_json::Value> {
        Ok(serde_json::json!({
            "status": "alive",
            "version": env!("CARGO_PKG_VERSION"),
        }))
    }

    /// Readiness probe — confirms the server has finished bootstrapping and can
    /// serve requests (capabilities loaded, mode resolved).
    ///
    /// JSON-RPC method: `health.readiness`
    pub(crate) async fn health_readiness(&self) -> Result<serde_json::Value> {
        let mode = self.mode.read().await;
        let cap_count = self.router.list_capabilities().await.len();
        let ready = cap_count > 0;
        Ok(serde_json::json!({
            "ready": ready,
            "mode": format!("{mode:?}"),
            "registered_capabilities": cap_count,
        }))
    }

    /// Bind the Unix socket so the path exists for external probes.
    ///
    /// Called early in `serve()` before bootstrap or translation loading,
    /// so that primalSpring and other health monitors can discover the
    /// socket immediately after the process starts.
    fn bind_socket(&self) -> Result<UnixListener> {
        if self.socket_path.exists() {
            std::fs::remove_file(&self.socket_path).context("Failed to remove old socket")?;
        }

        let listener =
            UnixListener::bind(&self.socket_path).context("Failed to bind Unix socket")?;

        info!("🧠 Neural API socket bound: {}", self.socket_path.display());
        info!("   Graphs directory: {}", self.graphs_dir.display());
        info!("   Family ID: {}", self.family_id);

        Ok(listener)
    }

    /// Accept UDS connections on a previously-bound listener.
    async fn accept_connections(&self, listener: UnixListener) -> Result<()> {
        info!(
            "🧠 Neural API server accepting UDS connections (mode: {})",
            self.mode_display_str().await
        );

        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let server = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_connection(stream).await {
                            error!("Connection error: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept connection: {}", e);
                }
            }
        }
    }

    /// Accept TCP connections (mobile / cross-gate).
    async fn accept_tcp_connections(&self, listener: tokio::net::TcpListener) -> Result<()> {
        info!(
            "📡 Neural API server accepting TCP connections (mode: {})",
            self.mode_display_str().await
        );

        loop {
            match listener.accept().await {
                Ok((stream, addr)) => {
                    debug!("TCP connection from {}", addr);
                    let server = self.clone();
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_tcp_connection(stream).await {
                            error!("TCP connection error from {}: {}", addr, e);
                        }
                    });
                }
                Err(e) => {
                    error!("Failed to accept TCP connection: {}", e);
                }
            }
        }
    }

    async fn mode_display_str(&self) -> &'static str {
        let mode = self.mode.read().await;
        match *mode {
            BiomeOsMode::Bootstrap => "BOOTSTRAP (genesis)",
            BiomeOsMode::Coordinated => "COORDINATED (gen 1)",
        }
    }
}

#[cfg(test)]
impl NeuralApiServer {
    /// Exercise [`NeuralApiServer::load_translations_on_startup`] in unit tests (private otherwise).
    pub(crate) async fn test_load_translations_on_startup(&self) -> Result<()> {
        self.load_translations_on_startup().await
    }
}

#[cfg(test)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    #[test]
    fn test_is_explicit_coordinated_mode_str_coordinated() {
        assert!(is_explicit_coordinated_mode_str("coordinated"));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_str_coord() {
        assert!(is_explicit_coordinated_mode_str("coord"));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_str_join() {
        assert!(is_explicit_coordinated_mode_str("join"));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_str_case_insensitive() {
        assert!(is_explicit_coordinated_mode_str("COORDINATED"));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_str_bootstrap_not_explicit() {
        assert!(!is_explicit_coordinated_mode_str("bootstrap"));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_str_empty_not_explicit() {
        assert!(!is_explicit_coordinated_mode_str(""));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_str_join_uppercase() {
        assert!(is_explicit_coordinated_mode_str("JOIN"));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_str_unknown_not_explicit() {
        assert!(!is_explicit_coordinated_mode_str("unknown"));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_str_mixed_case() {
        assert!(is_explicit_coordinated_mode_str("Coordinated"));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_str_whitespace_not_explicit() {
        assert!(!is_explicit_coordinated_mode_str(" coordinated"));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_env_coordinated() {
        assert!(is_explicit_coordinated_mode_with(Some("coordinated")));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_env_coord() {
        assert!(is_explicit_coordinated_mode_with(Some("coord")));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_env_join() {
        assert!(is_explicit_coordinated_mode_with(Some("join")));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_env_bootstrap_returns_false() {
        assert!(!is_explicit_coordinated_mode_with(Some("bootstrap")));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_env_unset_returns_false() {
        // Use Some("") to simulate unset - both yield false without env mutation
        assert!(!is_explicit_coordinated_mode_with(Some("")));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_env_unknown_returns_false() {
        assert!(!is_explicit_coordinated_mode_with(Some("unknown")));
    }

    #[test]
    fn test_is_explicit_coordinated_mode_str_partial_no_match() {
        assert!(!is_explicit_coordinated_mode_str("coordinatedx"));
        assert!(!is_explicit_coordinated_mode_str("xcoordinated"));
        assert!(!is_explicit_coordinated_mode_str("coordinat")); // partial, no match
    }

    #[test]
    fn test_is_explicit_coordinated_mode_str_join_suffix() {
        // "join" is valid; "joiner" would not match (different string)
        assert!(is_explicit_coordinated_mode_str("join"));
        assert!(!is_explicit_coordinated_mode_str("joiner"));
    }

    #[tokio::test]
    async fn test_load_translations_on_startup_defaults_only() {
        let temp = tempfile::tempdir().expect("tempdir");
        let sock = temp.path().join("neural-api.sock");
        let server = NeuralApiServer::new(temp.path(), "test-fam", sock);
        server
            .test_load_translations_on_startup()
            .await
            .expect("load translations");
    }

    #[tokio::test]
    async fn test_load_translations_on_startup_with_tower_atomic_graph() {
        let temp = tempfile::tempdir().expect("tempdir");
        let graph_toml = r#"
[graph]
id = "tower_atomic_bootstrap"
version = "1.0.0"
description = "Test graph for translations"

[[nodes]]
id = "log1"
[nodes.operation]
name = "log.info"
[nodes.config]
message = "test"
"#;
        std::fs::write(temp.path().join("tower_atomic_bootstrap.toml"), graph_toml)
            .expect("write graph");
        let sock = temp.path().join("neural-api.sock");
        let server = NeuralApiServer::new(temp.path(), "test-fam", sock);
        server
            .test_load_translations_on_startup()
            .await
            .expect("load translations with graph");
    }

    #[test]
    fn test_is_explicit_coordinated_mode_str_numeric_not_explicit() {
        assert!(!is_explicit_coordinated_mode_str("123"));
    }

    #[tokio::test]
    async fn test_load_translations_graph_parse_warn_branch_still_ok() {
        let temp = tempfile::tempdir().expect("tempdir");
        std::fs::write(
            temp.path().join("tower_atomic_bootstrap.toml"),
            "[[[ not valid graph",
        )
        .expect("write");
        let sock = temp.path().join("neural-api.sock");
        let server = NeuralApiServer::new(temp.path(), "test-fam", sock);
        server
            .test_load_translations_on_startup()
            .await
            .expect("startup load tolerates bad graph file");
    }

    /// `load_translations_on_startup` overlays `graphs_dir/../config/capability_registry.toml` when present.
    #[tokio::test]
    async fn test_load_translations_on_startup_with_capability_registry_overlay() {
        let base = tempfile::tempdir().expect("tempdir");
        let graphs_dir = base.path().join("graphs");
        std::fs::create_dir_all(&graphs_dir).expect("graphs dir");

        let config_path = graphs_dir.join("../config/capability_registry.toml");
        std::fs::create_dir_all(config_path.parent().expect("parent")).expect("config dir");
        std::fs::write(
            &config_path,
            r#"
[translations.crypto]
"crypto.unit.test_ping" = { provider = "beardog", method = "ping" }
"#,
        )
        .expect("write capability_registry.toml");

        let sock = graphs_dir.join("neural-api.sock");
        let server = NeuralApiServer::new(&graphs_dir, "test-fam", sock);
        server
            .test_load_translations_on_startup()
            .await
            .expect("load with overlay");
    }

    /// Invalid TOML at `graphs_dir/../config/capability_registry.toml` triggers the warn branch.
    #[tokio::test]
    async fn test_load_translations_on_startup_capability_registry_toml_parse_error() {
        let base = tempfile::tempdir().expect("tempdir");
        let graphs_dir = base.path().join("graphs");
        std::fs::create_dir_all(&graphs_dir).expect("graphs dir");
        let config_path = graphs_dir.join("../config/capability_registry.toml");
        std::fs::create_dir_all(config_path.parent().expect("parent")).expect("config dir");
        std::fs::write(&config_path, "[[[ not valid toml").expect("write broken toml");
        let sock = graphs_dir.join("neural-api.sock");
        let server = NeuralApiServer::new(&graphs_dir, "test-fam", sock);
        server
            .test_load_translations_on_startup()
            .await
            .expect("startup tolerates bad capability_registry.toml");
    }
}
