// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Unix/TCP socket binding, accept loops, and JSON-RPC health probes.

use anyhow::{Context, Result};
use tokio::net::UnixListener;
use tracing::{debug, error, info};

use super::NeuralApiServer;
use crate::mode::BiomeOsMode;

impl NeuralApiServer {
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

    /// Escalate BTSP enforcement at runtime.
    ///
    /// Called after Tower (security + mesh orchestration) is confirmed healthy.
    /// All subsequent UDS connections will require BTSP authentication.
    /// This is a one-way transition: once escalated, cannot be de-escalated.
    ///
    /// JSON-RPC method: `btsp.escalate`
    pub(crate) fn btsp_escalate(&self) -> Result<serde_json::Value> {
        let was = self
            .btsp_escalated
            .swap(true, std::sync::atomic::Ordering::SeqCst);
        if was {
            info!("🔐 BTSP escalation: already enforced (no-op)");
        } else {
            info!("🔐 BTSP escalated: new connections now require BTSP authentication");
        }
        Ok(serde_json::json!({
            "escalated": true,
            "previously_escalated": was,
            "family_id": self.family_id,
        }))
    }

    /// Report current BTSP enforcement state including Phase 3 readiness.
    ///
    /// JSON-RPC method: `btsp.status`
    pub(crate) async fn btsp_status(&self) -> Result<serde_json::Value> {
        let has_family = biomeos_core::btsp_client::has_family_id();
        let static_enforce = if self.btsp_optional {
            false
        } else {
            biomeos_core::btsp_client::btsp_enforce()
        };
        let runtime_escalated = !self.btsp_optional
            && self
                .btsp_escalated
                .load(std::sync::atomic::Ordering::Relaxed);
        let effective = static_enforce || runtime_escalated;
        Ok(serde_json::json!({
            "has_family_id": has_family,
            "static_enforce": static_enforce,
            "runtime_escalated": runtime_escalated,
            "effective_enforce": effective,
            "phase": if effective { "phase2_enforced" } else if has_family { "phase2_available" } else { "phase1_cleartext" },
            "post_handshake_cipher": "chacha20-poly1305",
            "phase3_ready": true,
            "supported_ciphers": ["chacha20-poly1305", "null"],
            "active_sessions": self.btsp_sessions.read().await.len(),
        }))
    }

    /// Bind the Unix socket so the path exists for external probes.
    ///
    /// Called early in `serve()` before bootstrap or translation loading,
    /// so that primalSpring and other health monitors can discover the
    /// socket immediately after the process starts.
    pub(crate) fn bind_socket(&self) -> Result<UnixListener> {
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
    ///
    /// **BTSP Phase 2**: When `FAMILY_ID` is set, each accepted connection
    /// undergoes a BTSP handshake before JSON-RPC processing begins. The
    /// handshake crypto is delegated to the security provider via `btsp.session.create` /
    /// `btsp.session.verify`. Clients that do not initiate a handshake
    /// (legacy JSON-RPC) are handled according to [`btsp_enforce`]:
    /// - enforce = true (default): connection rejected
    /// - enforce = false: accepted with a warning (rollout compatibility)
    ///
    /// [`btsp_enforce`]: biomeos_core::btsp_client::btsp_enforce
    pub(crate) async fn accept_connections(&self, listener: UnixListener) -> Result<()> {
        let btsp_active = biomeos_core::btsp_client::has_family_id();
        let static_enforce = if self.btsp_optional {
            false
        } else {
            biomeos_core::btsp_client::btsp_enforce()
        };

        info!(
            "🧠 Neural API server accepting UDS connections (mode: {}, BTSP: {})",
            self.mode_display_str().await,
            if self.btsp_optional {
                "optional (--btsp-optional)"
            } else if btsp_active {
                if static_enforce {
                    "enforced"
                } else {
                    "warn-only (escalatable)"
                }
            } else {
                "off (dev)"
            }
        );

        let escalated = self.btsp_escalated.clone();
        let btsp_opt = self.btsp_optional;
        loop {
            match listener.accept().await {
                Ok((stream, _addr)) => {
                    let server = self.clone();
                    let enforce = !btsp_opt
                        && (static_enforce || escalated.load(std::sync::atomic::Ordering::Relaxed));
                    tokio::spawn(async move {
                        if let Err(e) = server.handle_connection_with_btsp(stream, enforce).await {
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
    pub(crate) async fn accept_tcp_connections(
        &self,
        listener: tokio::net::TcpListener,
    ) -> Result<()> {
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
#[expect(clippy::expect_used, reason = "test")]
mod tests {
    use super::super::NeuralApiServer;

    #[test]
    fn test_health_liveness_reports_alive_and_version() {
        let temp = tempfile::tempdir().expect("tempdir");
        let sock = temp.path().join("neural-api.sock");
        let server = NeuralApiServer::new(temp.path(), "fam-liveness", sock);
        let v = server.health_liveness().expect("liveness");
        assert_eq!(v["status"], "alive");
        assert_eq!(v["version"], env!("CARGO_PKG_VERSION"));
    }

    #[tokio::test]
    async fn test_health_check_includes_family_socket_and_capability_count() {
        let temp = tempfile::tempdir().expect("tempdir");
        let sock = temp.path().join("api-health.sock");
        let server = NeuralApiServer::new(temp.path(), "fam-health", &sock);
        let j = server.health_check().await.expect("health check");
        assert_eq!(j["status"], "healthy");
        assert_eq!(j["family_id"], "fam-health");
        assert_eq!(j["socket_path"], sock.display().to_string());
        assert_eq!(j["registered_capabilities"], serde_json::json!(0));
    }

    #[tokio::test]
    async fn test_health_readiness_false_without_registered_capabilities() {
        let temp = tempfile::tempdir().expect("tempdir");
        let sock = temp.path().join("api-ready.sock");
        let server = NeuralApiServer::new(temp.path(), "fam-ready", sock);
        let j = server.health_readiness().await.expect("readiness");
        assert_eq!(j["ready"], serde_json::json!(false));
        assert_eq!(j["registered_capabilities"], serde_json::json!(0));
    }

    #[tokio::test]
    async fn test_bind_socket_replaces_stale_path_and_binds() {
        let temp = tempfile::tempdir().expect("tempdir");
        let sock = temp.path().join("bind-test.sock");
        std::fs::write(&sock, b"stale").expect("seed stale file");
        let server = NeuralApiServer::new(temp.path(), "fam-bind", &sock);
        let listener = server.bind_socket().expect("bind unix socket");
        drop(listener);
        assert!(sock.exists());
    }

    #[test]
    fn test_with_tcp_port_and_tcp_only_builder_chain() {
        let temp = tempfile::tempdir().expect("tempdir");
        let sock = temp.path().join("neural.sock");
        let _ = NeuralApiServer::new(temp.path(), "fam-tcp", &sock)
            .with_tcp_port(0)
            .with_tcp_only(0);
    }
}
