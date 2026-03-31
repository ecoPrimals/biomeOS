// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Transport probe implementations for `SocketDiscovery`.
//!
//! Each method implements one step in the multi-transport discovery chain:
//! environment hints, XDG runtime, family tmp, filesystem manifests,
//! socket registry, abstract sockets, TCP fallback, and transport verification.

use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use biomeos_types::primal_names;
use tokio::net::{TcpStream, UnixStream};
use tracing::{debug, trace};

use super::result::{DiscoveredSocket, DiscoveryMethod};
use super::transport::TransportEndpoint;

impl super::engine::SocketDiscovery {
    // ========================================================================
    // ENVIRONMENT HINT DISCOVERY
    // ========================================================================

    pub(crate) fn discover_via_env_hint(&self, primal_name: &str) -> Option<DiscoveredSocket> {
        self.discover_via_env_hint_with(primal_name, None)
    }

    #[expect(
        clippy::unused_self,
        reason = "method for future use or API consistency"
    )]
    pub(crate) fn discover_via_env_hint_with(
        &self,
        primal_name: &str,
        env_overrides: Option<&HashMap<String, String>>,
    ) -> Option<DiscoveredSocket> {
        let env_patterns = vec![
            format!("{}_SOCKET", primal_name.to_uppercase().replace('-', "_")),
            format!(
                "{}_SOCKET_PATH",
                primal_name.to_uppercase().replace('-', "_")
            ),
            format!(
                "BIOMEOS_{}_SOCKET",
                primal_name.to_uppercase().replace('-', "_")
            ),
        ];

        let get_env = |key: &str| {
            env_overrides
                .and_then(|m| m.get(key).cloned())
                .or_else(|| env::var(key).ok())
        };

        for env_var in env_patterns {
            if let Some(path_str) = get_env(&env_var) {
                let path = PathBuf::from(&path_str);
                if path.exists() {
                    debug!("Discovered {} via env hint: {}", primal_name, env_var);
                    return Some(
                        DiscoveredSocket::from_unix_path(
                            path,
                            DiscoveryMethod::EnvironmentHint(Arc::from(env_var.as_str())),
                        )
                        .with_primal_name(primal_name),
                    );
                }
            }
        }

        None
    }

    #[expect(
        clippy::unused_self,
        reason = "method for future use or API consistency"
    )]
    pub(crate) fn discover_endpoint_via_env_with(
        &self,
        primal_name: &str,
        env_overrides: Option<&HashMap<String, String>>,
    ) -> Option<TransportEndpoint> {
        let prefix = primal_name.to_uppercase().replace('-', "_");
        let get_env = |key: &str| {
            env_overrides
                .and_then(|m| m.get(key).cloned())
                .or_else(|| env::var(key).ok())
        };

        if let Some(tcp) = get_env(&format!("{prefix}_TCP")) {
            if let Some(endpoint) = TransportEndpoint::parse(&tcp)
                && matches!(endpoint, TransportEndpoint::TcpSocket { .. })
            {
                return Some(endpoint);
            }
            if let Some(endpoint) = TransportEndpoint::parse(&format!("tcp://{tcp}")) {
                return Some(endpoint);
            }
        }

        if let Some(endpoint_str) = get_env(&format!("{prefix}_ENDPOINT"))
            && let Some(endpoint) = TransportEndpoint::parse(&endpoint_str)
        {
            return Some(endpoint);
        }

        for var_name in [
            format!("{prefix}_SOCKET"),
            format!("{prefix}_SOCKET_PATH"),
            format!("BIOMEOS_{prefix}_SOCKET"),
        ] {
            if let Some(value) = get_env(&var_name)
                && let Some(endpoint) = TransportEndpoint::parse(&value)
            {
                if let TransportEndpoint::UnixSocket { ref path } = endpoint {
                    if path.exists() {
                        return Some(endpoint);
                    }
                } else {
                    return Some(endpoint);
                }
            }
        }

        None
    }

    // ========================================================================
    // FILESYSTEM DISCOVERY (XDG, /tmp, capability sockets)
    // ========================================================================

    pub(super) async fn discover_via_xdg(&self, primal_name: &str) -> Option<DiscoveredSocket> {
        let runtime_dir = self.xdg_runtime_dir()?;
        let biomeos_dir = runtime_dir.join(primal_names::BIOMEOS);

        for cap_name in Self::capability_socket_names(primal_name) {
            let cap_path = biomeos_dir.join(format!("{cap_name}.sock"));
            if self.verify_unix_socket(&cap_path).await {
                debug!(
                    "Discovered {} via capability socket {}.sock (XDG)",
                    primal_name, cap_name
                );
                return Some(
                    DiscoveredSocket::from_unix_path(cap_path, DiscoveryMethod::XdgRuntime)
                        .with_primal_name(primal_name),
                );
            }
        }

        let socket_path =
            biomeos_dir.join(format!("{}-{}.sock", primal_name, self.family_id.as_str()));
        if socket_path.exists() {
            debug!("Discovered {} via XDG runtime", primal_name);
            return Some(
                DiscoveredSocket::from_unix_path(socket_path, DiscoveryMethod::XdgRuntime)
                    .with_primal_name(primal_name),
            );
        }

        let legacy_path = biomeos_dir.join(format!("{primal_name}.sock"));
        if legacy_path.exists() {
            debug!("Discovered {} via XDG runtime (legacy)", primal_name);
            return Some(
                DiscoveredSocket::from_unix_path(legacy_path, DiscoveryMethod::XdgRuntime)
                    .with_primal_name(primal_name),
            );
        }

        None
    }

    pub(super) async fn discover_via_family_tmp(
        &self,
        primal_name: &str,
    ) -> Option<DiscoveredSocket> {
        let temp_dir = self.temp_dir();

        for cap_name in Self::capability_socket_names(primal_name) {
            let cap_path = temp_dir.join(format!("{cap_name}.sock"));
            if self.verify_unix_socket(&cap_path).await {
                debug!(
                    "Discovered {} via capability socket {}.sock (tmp)",
                    primal_name, cap_name
                );
                return Some(
                    DiscoveredSocket::from_unix_path(cap_path, DiscoveryMethod::FamilyTmp)
                        .with_primal_name(primal_name),
                );
            }
        }

        let socket_path =
            temp_dir.join(format!("{}-{}.sock", primal_name, self.family_id.as_str()));
        if socket_path.exists() {
            debug!("Discovered {} via family temp dir", primal_name);
            return Some(
                DiscoveredSocket::from_unix_path(socket_path, DiscoveryMethod::FamilyTmp)
                    .with_primal_name(primal_name),
            );
        }

        let legacy_path = temp_dir.join(format!("{primal_name}.sock"));
        if legacy_path.exists() {
            debug!("Discovered {} via temp dir (legacy)", primal_name);
            return Some(
                DiscoveredSocket::from_unix_path(legacy_path, DiscoveryMethod::FamilyTmp)
                    .with_primal_name(primal_name),
            );
        }

        None
    }

    pub(super) async fn discover_capability_socket(
        &self,
        capability: &str,
    ) -> Option<DiscoveredSocket> {
        let sock_name = format!("{capability}.sock");

        if let Some(runtime_dir) = self.xdg_runtime_dir() {
            let path = runtime_dir.join(primal_names::BIOMEOS).join(&sock_name);
            if self.verify_unix_socket(&path).await {
                debug!("Discovered capability '{}' via XDG socket", capability);
                return Some(DiscoveredSocket::from_unix_path(
                    path,
                    DiscoveryMethod::XdgRuntime,
                ));
            }
        }

        let tmp_path = self.temp_dir().join(&sock_name);
        if self.verify_unix_socket(&tmp_path).await {
            debug!("Discovered capability '{}' via tmp socket", capability);
            return Some(DiscoveredSocket::from_unix_path(
                tmp_path,
                DiscoveryMethod::FamilyTmp,
            ));
        }

        None
    }

    // ========================================================================
    // MANIFEST + REGISTRY DISCOVERY
    // ========================================================================

    pub(super) async fn discover_via_manifest(
        &self,
        primal_name: &str,
    ) -> Option<DiscoveredSocket> {
        use super::result::PrimalManifest;

        let manifest_name = format!("{primal_name}.json");

        let mut candidates = Vec::new();
        if let Some(xdg) = self.xdg_runtime_dir() {
            candidates.push(xdg.join("ecoPrimals/manifests").join(&manifest_name));
        }
        candidates.push(
            self.temp_dir()
                .join("ecoPrimals/manifests")
                .join(&manifest_name),
        );

        for path in candidates {
            if let Ok(contents) = tokio::fs::read_to_string(&path).await {
                match serde_json::from_str::<PrimalManifest>(&contents) {
                    Ok(manifest) => {
                        let socket_path = PathBuf::from(manifest.socket.as_ref());
                        if self.verify_unix_socket(&socket_path).await {
                            debug!(
                                "Discovered {} via manifest at {}",
                                primal_name,
                                path.display()
                            );
                            return Some(
                                DiscoveredSocket::from_unix_path(
                                    socket_path,
                                    DiscoveryMethod::Manifest,
                                )
                                .with_primal_name(primal_name)
                                .with_capabilities(manifest.capabilities),
                            );
                        }
                        trace!(
                            "Manifest for {} found but socket not connectable: {}",
                            primal_name,
                            manifest.socket.as_ref()
                        );
                    }
                    Err(e) => {
                        trace!("Invalid manifest at {}: {}", path.display(), e);
                    }
                }
            }
        }

        None
    }

    pub(super) async fn discover_via_socket_registry(
        &self,
        primal_name: &str,
    ) -> Option<DiscoveredSocket> {
        use super::result::SocketRegistry;

        let registry_path = self
            .xdg_runtime_dir()?
            .join(primal_names::BIOMEOS)
            .join("socket-registry.json");

        let contents = tokio::fs::read_to_string(&registry_path).await.ok()?;
        let registry: SocketRegistry = serde_json::from_str(&contents).ok()?;

        for entry in &registry.entries {
            if entry.primal.eq_ignore_ascii_case(primal_name) {
                let socket_path = PathBuf::from(&entry.socket);
                if self.verify_unix_socket(&socket_path).await {
                    debug!(
                        "Discovered {} via socket-registry at {}",
                        primal_name,
                        registry_path.display()
                    );
                    return Some(
                        DiscoveredSocket::from_unix_path(
                            socket_path,
                            DiscoveryMethod::SocketRegistry,
                        )
                        .with_primal_name(primal_name)
                        .with_capabilities(entry.capabilities.clone()),
                    );
                }
            }
        }

        None
    }

    // ========================================================================
    // TRANSPORT VERIFICATION + PROBES
    // ========================================================================

    pub(super) async fn try_unix_socket_xdg(&self, primal_name: &str) -> Option<PathBuf> {
        let runtime_dir = self.xdg_runtime_dir()?;
        let biomeos_dir = runtime_dir.join(primal_names::BIOMEOS);

        let socket_path =
            biomeos_dir.join(format!("{}-{}.sock", primal_name, self.family_id.as_str()));
        if self.verify_unix_socket(&socket_path).await {
            return Some(socket_path);
        }

        let legacy_path = biomeos_dir.join(format!("{primal_name}.sock"));
        if self.verify_unix_socket(&legacy_path).await {
            return Some(legacy_path);
        }

        None
    }

    pub(super) async fn try_unix_socket_tmp(&self, primal_name: &str) -> Option<PathBuf> {
        let temp_dir = self.temp_dir();

        let socket_path =
            temp_dir.join(format!("{}-{}.sock", primal_name, self.family_id.as_str()));
        if self.verify_unix_socket(&socket_path).await {
            return Some(socket_path);
        }

        let legacy_path = temp_dir.join(format!("{primal_name}.sock"));
        if self.verify_unix_socket(&legacy_path).await {
            return Some(legacy_path);
        }

        None
    }

    #[cfg(target_os = "linux")]
    pub(super) fn try_abstract_socket(&self, primal_name: &str) -> Option<String> {
        use std::os::linux::net::SocketAddrExt;
        use std::os::unix::net::SocketAddr;

        let abstract_name = format!("biomeos_{}_{}", primal_name, self.family_id.as_str());

        let addr = match SocketAddr::from_abstract_name(&abstract_name) {
            Ok(addr) => addr,
            Err(e) => {
                trace!("Failed to create abstract socket addr: {}", e);
                return None;
            }
        };

        match std::os::unix::net::UnixStream::connect_addr(&addr) {
            Ok(_) => {
                debug!(
                    "Abstract socket available for {}: @{}",
                    primal_name, abstract_name
                );
                Some(abstract_name)
            }
            Err(e) => {
                trace!(
                    "Abstract socket not available for {}: @{} - {}",
                    primal_name, abstract_name, e
                );
                None
            }
        }
    }

    /// Tier 2 TCP fallback; `tcp_env_override` replaces `{PRIMAL}_TCP` when set (for tests).
    pub(super) async fn try_tcp_fallback_with(
        &self,
        primal_name: &str,
        tcp_env_override: Option<&str>,
    ) -> Option<(Arc<str>, u16)> {
        let host = &self.strategy.tcp_fallback_host;
        let prefix = primal_name.to_uppercase().replace('-', "_");

        let tcp_env = tcp_env_override
            .map(ToString::to_string)
            .or_else(|| env::var(format!("{prefix}_TCP")).ok());

        if let Some(tcp_env) = tcp_env {
            if let Some(TransportEndpoint::TcpSocket { host: h, port: p }) =
                TransportEndpoint::parse(&tcp_env)
                && self.verify_tcp_connection(h.as_ref(), p).await
            {
                return Some((h, p));
            }
            if let Ok(port) = tcp_env.parse::<u16>()
                && self.verify_tcp_connection(host.as_ref(), port).await
            {
                return Some((Arc::clone(host), port));
            }
        }

        let port = self.calculate_primal_port(primal_name);
        if self.verify_tcp_connection(host.as_ref(), port).await {
            return Some((Arc::clone(host), port));
        }

        None
    }

    pub(crate) fn calculate_primal_port(&self, primal_name: &str) -> u16 {
        let hash: u32 = primal_name.bytes().map(u32::from).sum();
        let offset = (hash % 100) as u16;
        self.strategy.tcp_port_start + offset
    }

    pub(crate) async fn verify_unix_socket(&self, path: &Path) -> bool {
        if !path.exists() {
            return false;
        }

        match tokio::time::timeout(
            std::time::Duration::from_millis(500),
            UnixStream::connect(path),
        )
        .await
        {
            Ok(Ok(_)) => true,
            Ok(Err(e)) => {
                trace!(
                    "Unix socket exists but connection failed: {} - {}",
                    path.display(),
                    e
                );
                false
            }
            Err(_) => {
                trace!("Unix socket connection timed out: {}", path.display());
                false
            }
        }
    }

    pub(crate) async fn verify_tcp_connection(&self, host: &str, port: u16) -> bool {
        let addr = format!("{host}:{port}");
        match tokio::time::timeout(
            std::time::Duration::from_millis(500),
            TcpStream::connect(&addr),
        )
        .await
        {
            Ok(Ok(_)) => {
                trace!("TCP connection verified: {}", addr);
                true
            }
            Ok(Err(e)) => {
                trace!("TCP connection failed: {} - {}", addr, e);
                false
            }
            Err(_) => {
                trace!("TCP connection timed out: {}", addr);
                false
            }
        }
    }
}
