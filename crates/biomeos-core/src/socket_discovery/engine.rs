// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Socket Discovery Engine
//!
//! The main discovery engine implementing capability-based socket discovery
//! with multi-transport support per Universal IPC Standard v3.0.
//!
//! Transport probe implementations (env hints, XDG, abstract sockets, TCP,
//! manifests, registry) are in `engine_probes.rs`.

use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use biomeos_types::identifiers::FamilyId;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::neural_api;
use super::path_builder;
use super::result::{DiscoveredSocket, DiscoveryMethod};
use super::strategy::DiscoveryStrategy;
use super::transport::TransportEndpoint;

/// Cached socket entry
pub(super) struct CachedSocket {
    pub(super) socket: DiscoveredSocket,
    pub(super) cached_at: tokio::time::Instant,
}

/// Socket discovery engine
///
/// Provides capability-based socket discovery without hardcoded paths.
pub struct SocketDiscovery {
    /// Family ID for namespace isolation
    pub(crate) family_id: FamilyId,

    /// Discovery strategy
    pub(crate) strategy: DiscoveryStrategy,

    /// Discovery cache
    pub(super) cache: Arc<RwLock<HashMap<Arc<str>, CachedSocket>>>,

    /// Neural API socket (for capability registry queries)
    pub(crate) neural_api_socket: Option<PathBuf>,

    /// Override for `XDG_RUNTIME_DIR` (for testing without env mutation)
    pub(crate) xdg_runtime_dir_override: Option<PathBuf>,

    /// Override for temp dir / TMPDIR (for testing without env mutation)
    pub(crate) temp_dir_override: Option<PathBuf>,
}

impl SocketDiscovery {
    /// Create new socket discovery with default strategy
    pub fn new(family_id: impl AsRef<str>) -> Self {
        Self {
            family_id: FamilyId::new(family_id),
            strategy: DiscoveryStrategy::default(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            neural_api_socket: None,
            xdg_runtime_dir_override: None,
            temp_dir_override: None,
        }
    }

    /// Create with custom strategy
    pub fn with_strategy(family_id: impl AsRef<str>, strategy: DiscoveryStrategy) -> Self {
        Self {
            family_id: FamilyId::new(family_id),
            strategy,
            cache: Arc::new(RwLock::new(HashMap::new())),
            neural_api_socket: None,
            xdg_runtime_dir_override: None,
            temp_dir_override: None,
        }
    }

    /// Set Neural API socket for registry queries
    #[must_use]
    pub fn with_neural_api(mut self, socket: PathBuf) -> Self {
        self.neural_api_socket = Some(socket);
        self
    }

    /// Set `XDG_RUNTIME_DIR` override (for testing without env mutation)
    pub fn with_xdg_override(mut self, path: impl AsRef<Path>) -> Self {
        self.xdg_runtime_dir_override = Some(path.as_ref().to_path_buf());
        self
    }

    /// Set temp dir override / TMPDIR (for testing without env mutation)
    pub fn with_temp_dir_override(mut self, path: impl AsRef<Path>) -> Self {
        self.temp_dir_override = Some(path.as_ref().to_path_buf());
        self
    }

    // ========================================================================
    // PUBLIC DISCOVERY API
    // ========================================================================

    /// Discover socket for a primal by name
    pub async fn discover_primal(&self, primal_name: &str) -> Option<DiscoveredSocket> {
        let cache_key = format!("primal:{primal_name}");

        if self.strategy.enable_cache
            && let Some(cached) = self.check_cache(&cache_key).await
        {
            return Some(cached);
        }

        if self.strategy.check_env_hints
            && let Some(socket) = self.discover_via_env_hint(primal_name)
        {
            self.cache_socket(&cache_key, &socket).await;
            return Some(socket);
        }

        if self.strategy.use_xdg_runtime
            && let Some(socket) = self.discover_via_xdg(primal_name).await
        {
            self.cache_socket(&cache_key, &socket).await;
            return Some(socket);
        }

        if self.strategy.use_family_tmp
            && let Some(socket) = self.discover_via_family_tmp(primal_name).await
        {
            self.cache_socket(&cache_key, &socket).await;
            return Some(socket);
        }

        if let Some(socket) = self.discover_via_manifest(primal_name).await {
            self.cache_socket(&cache_key, &socket).await;
            return Some(socket);
        }

        if let Some(socket) = self.discover_via_socket_registry(primal_name).await {
            self.cache_socket(&cache_key, &socket).await;
            return Some(socket);
        }

        if self.strategy.query_registry
            && let Some(socket) = self.discover_via_registry_by_name(primal_name).await
        {
            self.cache_socket(&cache_key, &socket).await;
            return Some(socket);
        }

        warn!("Socket not found for primal: {}", primal_name);
        None
    }

    /// Discover socket by capability domain name.
    ///
    /// Tries capability-first filesystem sockets (e.g. `security.sock`) before
    /// falling back to the Neural API registry. Absorbed from Squirrel alpha.13.
    pub async fn discover_capability(&self, capability: &str) -> Option<DiscoveredSocket> {
        let cache_key = format!("capability:{capability}");

        if self.strategy.enable_cache
            && let Some(cached) = self.check_cache(&cache_key).await
        {
            return Some(cached);
        }

        if let Some(socket) = self.discover_capability_socket(capability).await {
            self.cache_socket(&cache_key, &socket).await;
            return Some(socket);
        }

        if let Some(primal) =
            biomeos_types::capability_taxonomy::CapabilityTaxonomy::resolve_to_primal(capability)
        {
            if let Some(socket) = self.discover_primal(primal).await {
                self.cache_socket(&cache_key, &socket).await;
                return Some(socket);
            }
        }

        if self.strategy.query_registry
            && let Some(socket) = self.discover_via_registry_by_capability(capability).await
        {
            self.cache_socket(&cache_key, &socket).await;
            return Some(socket);
        }

        warn!(
            "Socket not found for capability '{}'. Start a primal or set {}_SOCKET env var.",
            capability,
            capability.to_uppercase()
        );
        None
    }

    /// Get socket path for a primal (convenience method)
    pub async fn get_socket_path(&self, primal_name: &str) -> Option<PathBuf> {
        self.discover_primal(primal_name).await.map(|s| s.path)
    }

    /// Discover primal with automatic Tier 1 → Tier 2 fallback
    ///
    /// **Universal IPC Standard v3.0**: Implements graceful transport fallback.
    pub async fn discover_with_fallback(&self, primal_name: &str) -> Option<TransportEndpoint> {
        self.discover_with_fallback_with_env_overrides(primal_name, None, None)
            .await
    }

    /// Like [`Self::discover_with_fallback`], but supplies optional per-process env overrides
    /// (e.g. `{PRIMAL}_TCP`) without mutating the process environment.
    pub async fn discover_with_fallback_with_env_overrides(
        &self,
        primal_name: &str,
        env_overrides: Option<&HashMap<String, String>>,
        tcp_tier2_override: Option<&str>,
    ) -> Option<TransportEndpoint> {
        let cache_key = format!("endpoint:{primal_name}");

        if self.strategy.enable_cache
            && let Some(cached) = self.check_cache(&cache_key).await
        {
            return Some(cached.endpoint);
        }

        if self.strategy.check_env_hints
            && let Some(endpoint) = self.discover_endpoint_via_env_with(primal_name, env_overrides)
        {
            let socket = DiscoveredSocket::from_endpoint(
                endpoint.clone(),
                DiscoveryMethod::EnvironmentHint(Arc::from(format!(
                    "{}_*",
                    primal_name.to_uppercase()
                ))),
            )
            .with_primal_name(primal_name);
            self.cache_socket(&cache_key, &socket).await;
            return Some(endpoint);
        }

        // === TIER 1: Native Transports ===

        if self.strategy.use_xdg_runtime
            && let Some(path) = self.try_unix_socket_xdg(primal_name).await
        {
            let endpoint = TransportEndpoint::UnixSocket { path: path.clone() };
            let socket =
                DiscoveredSocket::from_endpoint(endpoint.clone(), DiscoveryMethod::XdgRuntime)
                    .with_primal_name(primal_name);
            self.cache_socket(&cache_key, &socket).await;
            return Some(endpoint);
        }

        #[cfg(target_os = "linux")]
        if self.strategy.try_abstract_sockets
            && let Some(name) = self.try_abstract_socket(primal_name)
        {
            let endpoint = TransportEndpoint::AbstractSocket {
                name: Arc::from(name.as_str()),
            };
            let socket =
                DiscoveredSocket::from_endpoint(endpoint.clone(), DiscoveryMethod::AbstractSocket)
                    .with_primal_name(primal_name);
            self.cache_socket(&cache_key, &socket).await;
            return Some(endpoint);
        }

        if self.strategy.use_family_tmp
            && let Some(path) = self.try_unix_socket_tmp(primal_name).await
        {
            let endpoint = TransportEndpoint::UnixSocket { path: path.clone() };
            let socket =
                DiscoveredSocket::from_endpoint(endpoint.clone(), DiscoveryMethod::FamilyTmp)
                    .with_primal_name(primal_name);
            self.cache_socket(&cache_key, &socket).await;
            return Some(endpoint);
        }

        if let Some(socket) = self.discover_via_manifest(primal_name).await {
            self.cache_socket(&cache_key, &socket).await;
            return Some(socket.endpoint);
        }

        if self.strategy.query_registry
            && let Some(socket) = self.discover_via_registry_by_name(primal_name).await
        {
            self.cache_socket(&cache_key, &socket).await;
            return Some(socket.endpoint);
        }

        // === TIER 2: Universal Fallback ===

        if self.strategy.enable_tcp_fallback
            && let Some((host, port)) = self
                .try_tcp_fallback_with(primal_name, tcp_tier2_override)
                .await
        {
            let endpoint = TransportEndpoint::TcpSocket { host, port };
            info!(
                "Discovered {} via TCP fallback (Tier 2): {}",
                primal_name, endpoint
            );
            let socket =
                DiscoveredSocket::from_endpoint(endpoint.clone(), DiscoveryMethod::TcpFallback)
                    .with_primal_name(primal_name);
            self.cache_socket(&cache_key, &socket).await;
            return Some(endpoint);
        }

        warn!("Primal '{}' not found via any transport", primal_name);
        None
    }

    /// Get the transport endpoint for a primal (convenience method)
    pub async fn get_endpoint(&self, primal_name: &str) -> Option<TransportEndpoint> {
        self.discover_with_fallback(primal_name).await
    }

    // ========================================================================
    // PATH BUILDING
    // ========================================================================

    /// Build deterministic socket path for a primal
    ///
    /// Implements 5-tier socket resolution per `PRIMAL_DEPLOYMENT_STANDARD`.
    #[must_use]
    pub fn build_socket_path(&self, primal_name: &str) -> PathBuf {
        path_builder::build_socket_path(primal_name, self.family_id.as_str(), None, None)
    }

    /// Build socket path with explicit env overrides (for testing without env mutation).
    #[cfg(test)]
    pub(crate) fn build_socket_path_with(
        &self,
        primal_name: &str,
        primal_socket: Option<&str>,
        xdg_runtime_dir: Option<&Path>,
    ) -> PathBuf {
        path_builder::build_socket_path(
            primal_name,
            self.family_id.as_str(),
            primal_socket,
            xdg_runtime_dir,
        )
    }

    /// Enumerate Unix socket paths in `$XDG_RUNTIME_DIR/biomeos` matching
    /// `{primal}-{family_id}.sock`.
    ///
    /// Name-agnostic: every matching file is a candidate for capability or health
    /// probing (no compiled primal list).
    #[must_use]
    pub fn list_family_scoped_unix_sockets(&self) -> Vec<PathBuf> {
        let Some(xdg) = self.xdg_runtime_dir() else {
            return vec![];
        };
        let biomeos_dir = xdg.join("biomeos");
        let suffix = format!("-{}.sock", self.family_id.as_str());
        let Ok(entries) = std::fs::read_dir(&biomeos_dir) else {
            return vec![];
        };

        let mut paths: Vec<PathBuf> = entries
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.ends_with(&suffix))
            })
            .collect();
        paths.sort();
        paths
    }

    // ========================================================================
    // HELPERS
    // ========================================================================

    pub(super) fn capability_socket_names(primal_name: &str) -> Vec<String> {
        super::capability_sockets::names_for_primal(primal_name)
    }

    pub(crate) fn get_neural_api_socket(&self) -> Option<PathBuf> {
        self.get_neural_api_socket_with(None)
    }

    pub(crate) fn get_neural_api_socket_with(
        &self,
        neural_api_env_override: Option<&Path>,
    ) -> Option<PathBuf> {
        neural_api::resolve_neural_api_socket(
            self.family_id.as_str(),
            self.neural_api_socket.as_ref(),
            neural_api_env_override,
        )
    }

    /// Get XDG runtime dir: override if set, else env var.
    pub(super) fn xdg_runtime_dir(&self) -> Option<PathBuf> {
        self.xdg_runtime_dir_override
            .clone()
            .filter(|p| p.exists())
            .or_else(|| {
                env::var("XDG_RUNTIME_DIR")
                    .ok()
                    .map(PathBuf::from)
                    .filter(|p| p.exists())
            })
    }

    /// Get temp dir: override if set, else `std::env::temp_dir()`.
    pub(super) fn temp_dir(&self) -> PathBuf {
        self.temp_dir_override
            .clone()
            .unwrap_or_else(std::env::temp_dir)
    }

    #[cfg(test)]
    pub(crate) fn get_xdg_runtime_dir() -> Option<PathBuf> {
        env::var("XDG_RUNTIME_DIR")
            .ok()
            .map(PathBuf::from)
            .filter(|p| p.exists())
    }

    // ========================================================================
    // CACHE
    // ========================================================================

    /// Check cache for a socket
    pub(crate) async fn check_cache(&self, key: &str) -> Option<DiscoveredSocket> {
        let cache = self.cache.read().await;
        if let Some(cached) = cache.get(key) {
            let age = cached.cached_at.elapsed().as_secs();
            if age < self.strategy.cache_ttl_secs {
                debug!("Cache hit for {} (age: {}s)", key, age);
                return Some(DiscoveredSocket {
                    discovered_via: DiscoveryMethod::Cached,
                    ..cached.socket.clone()
                });
            }
        }
        None
    }

    /// Cache a discovered socket
    pub(crate) async fn cache_socket(&self, key: &str, socket: &DiscoveredSocket) {
        if self.strategy.enable_cache {
            let mut cache = self.cache.write().await;
            cache.insert(
                Arc::from(key),
                CachedSocket {
                    socket: socket.clone(),
                    cached_at: tokio::time::Instant::now(),
                },
            );
        }
    }

    /// Clear the discovery cache
    pub async fn clear_cache(&self) {
        let mut cache = self.cache.write().await;
        cache.clear();
        info!("Socket discovery cache cleared");
    }
}
