// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Socket Discovery Engine
//!
//! The main discovery engine implementing capability-based socket discovery
//! with multi-transport support per Universal IPC Standard v3.0.

use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::net::{TcpStream, UnixStream};
use tokio::sync::RwLock;
use tracing::{debug, info, trace, warn};

use super::result::{DiscoveredSocket, DiscoveryMethod};
use super::strategy::DiscoveryStrategy;
use super::transport::TransportEndpoint;

/// Cached socket entry
struct CachedSocket {
    socket: DiscoveredSocket,
    cached_at: std::time::Instant,
}

/// Socket discovery engine
///
/// Provides capability-based socket discovery without hardcoded paths.
pub struct SocketDiscovery {
    /// Family ID for namespace isolation
    pub(crate) family_id: String,

    /// Discovery strategy
    pub(crate) strategy: DiscoveryStrategy,

    /// Discovery cache
    cache: Arc<RwLock<HashMap<String, CachedSocket>>>,

    /// Neural API socket (for capability registry queries)
    pub(crate) neural_api_socket: Option<PathBuf>,
}

impl SocketDiscovery {
    /// Create new socket discovery with default strategy
    pub fn new(family_id: impl Into<String>) -> Self {
        Self {
            family_id: family_id.into(),
            strategy: DiscoveryStrategy::default(),
            cache: Arc::new(RwLock::new(HashMap::new())),
            neural_api_socket: None,
        }
    }

    /// Create with custom strategy
    pub fn with_strategy(family_id: impl Into<String>, strategy: DiscoveryStrategy) -> Self {
        Self {
            family_id: family_id.into(),
            strategy,
            cache: Arc::new(RwLock::new(HashMap::new())),
            neural_api_socket: None,
        }
    }

    /// Set Neural API socket for registry queries
    pub fn with_neural_api(mut self, socket: PathBuf) -> Self {
        self.neural_api_socket = Some(socket);
        self
    }

    // ========================================================================
    // DISCOVERY METHODS
    // ========================================================================

    /// Discover socket for a primal by name
    pub async fn discover_primal(&self, primal_name: &str) -> Option<DiscoveredSocket> {
        let cache_key = format!("primal:{primal_name}");

        // 1. Check cache
        if self.strategy.enable_cache {
            if let Some(cached) = self.check_cache(&cache_key).await {
                return Some(cached);
            }
        }

        // 2. Environment hint
        if self.strategy.check_env_hints {
            if let Some(socket) = self.discover_via_env_hint(primal_name).await {
                self.cache_socket(&cache_key, &socket).await;
                return Some(socket);
            }
        }

        // 3. XDG runtime dir
        if self.strategy.use_xdg_runtime {
            if let Some(socket) = self.discover_via_xdg(primal_name).await {
                self.cache_socket(&cache_key, &socket).await;
                return Some(socket);
            }
        }

        // 4. Family-scoped /tmp
        if self.strategy.use_family_tmp {
            if let Some(socket) = self.discover_via_family_tmp(primal_name).await {
                self.cache_socket(&cache_key, &socket).await;
                return Some(socket);
            }
        }

        // 5. Capability registry
        if self.strategy.query_registry {
            if let Some(socket) = self.discover_via_registry_by_name(primal_name).await {
                self.cache_socket(&cache_key, &socket).await;
                return Some(socket);
            }
        }

        warn!("Socket not found for primal: {}", primal_name);
        None
    }

    /// Discover socket by capability
    pub async fn discover_capability(&self, capability: &str) -> Option<DiscoveredSocket> {
        let cache_key = format!("capability:{capability}");

        if self.strategy.enable_cache {
            if let Some(cached) = self.check_cache(&cache_key).await {
                return Some(cached);
            }
        }

        if self.strategy.query_registry {
            if let Some(socket) = self.discover_via_registry_by_capability(capability).await {
                self.cache_socket(&cache_key, &socket).await;
                return Some(socket);
            }
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

    // ========================================================================
    // MULTI-TRANSPORT DISCOVERY WITH FALLBACK
    // ========================================================================

    /// Discover primal with automatic Tier 1 → Tier 2 fallback
    ///
    /// **Universal IPC Standard v3.0**: Implements graceful transport fallback.
    pub async fn discover_with_fallback(&self, primal_name: &str) -> Option<TransportEndpoint> {
        let cache_key = format!("endpoint:{primal_name}");

        // 1. Check cache
        if self.strategy.enable_cache {
            if let Some(cached) = self.check_cache(&cache_key).await {
                return Some(cached.endpoint);
            }
        }

        // 2. Try environment hint
        if self.strategy.check_env_hints {
            if let Some(endpoint) = self.discover_endpoint_via_env(primal_name).await {
                trace!("Discovered {} via environment: {}", primal_name, endpoint);
                let socket = DiscoveredSocket::from_endpoint(
                    endpoint.clone(),
                    DiscoveryMethod::EnvironmentHint(format!("{}_*", primal_name.to_uppercase())),
                )
                .with_primal_name(primal_name);
                self.cache_socket(&cache_key, &socket).await;
                return Some(endpoint);
            }
        }

        // === TIER 1: Native Transports ===

        // 3. Try Unix socket (XDG)
        if self.strategy.use_xdg_runtime {
            if let Some(path) = self.try_unix_socket_xdg(primal_name).await {
                let endpoint = TransportEndpoint::UnixSocket { path: path.clone() };
                let socket =
                    DiscoveredSocket::from_endpoint(endpoint.clone(), DiscoveryMethod::XdgRuntime)
                        .with_primal_name(primal_name);
                self.cache_socket(&cache_key, &socket).await;
                return Some(endpoint);
            }
        }

        // 4. Try abstract socket (Linux/Android only)
        #[cfg(target_os = "linux")]
        if self.strategy.try_abstract_sockets {
            if let Some(name) = self.try_abstract_socket(primal_name).await {
                let endpoint = TransportEndpoint::AbstractSocket { name: name.clone() };
                let socket = DiscoveredSocket::from_endpoint(
                    endpoint.clone(),
                    DiscoveryMethod::AbstractSocket,
                )
                .with_primal_name(primal_name);
                self.cache_socket(&cache_key, &socket).await;
                return Some(endpoint);
            }
        }

        // 5. Try Unix socket (family /tmp)
        if self.strategy.use_family_tmp {
            if let Some(path) = self.try_unix_socket_tmp(primal_name).await {
                let endpoint = TransportEndpoint::UnixSocket { path: path.clone() };
                let socket =
                    DiscoveredSocket::from_endpoint(endpoint.clone(), DiscoveryMethod::FamilyTmp)
                        .with_primal_name(primal_name);
                self.cache_socket(&cache_key, &socket).await;
                return Some(endpoint);
            }
        }

        // 6. Query capability registry
        if self.strategy.query_registry {
            if let Some(socket) = self.discover_via_registry_by_name(primal_name).await {
                self.cache_socket(&cache_key, &socket).await;
                return Some(socket.endpoint);
            }
        }

        // === TIER 2: Universal Fallback ===

        // 7. Try TCP fallback
        if self.strategy.enable_tcp_fallback {
            if let Some((host, port)) = self.try_tcp_fallback(primal_name).await {
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
        }

        warn!("Primal '{}' not found via any transport", primal_name);
        None
    }

    /// Get the transport endpoint for a primal (convenience method)
    pub async fn get_endpoint(&self, primal_name: &str) -> Option<TransportEndpoint> {
        self.discover_with_fallback(primal_name).await
    }

    // ========================================================================
    // TRANSPORT-SPECIFIC DISCOVERY HELPERS
    // ========================================================================

    pub(crate) async fn discover_endpoint_via_env(
        &self,
        primal_name: &str,
    ) -> Option<TransportEndpoint> {
        let prefix = primal_name.to_uppercase().replace('-', "_");

        // Check TCP first
        if let Ok(tcp) = env::var(format!("{prefix}_TCP")) {
            if let Some(endpoint) = TransportEndpoint::parse(&tcp) {
                if matches!(endpoint, TransportEndpoint::TcpSocket { .. }) {
                    return Some(endpoint);
                }
            }
            if let Some(endpoint) = TransportEndpoint::parse(&format!("tcp://{tcp}")) {
                return Some(endpoint);
            }
        }

        // Check generic endpoint
        if let Ok(endpoint_str) = env::var(format!("{prefix}_ENDPOINT")) {
            if let Some(endpoint) = TransportEndpoint::parse(&endpoint_str) {
                return Some(endpoint);
            }
        }

        // Check socket
        for var_name in [
            format!("{prefix}_SOCKET"),
            format!("{prefix}_SOCKET_PATH"),
            format!("BIOMEOS_{prefix}_SOCKET"),
        ] {
            if let Ok(value) = env::var(&var_name) {
                if let Some(endpoint) = TransportEndpoint::parse(&value) {
                    if let TransportEndpoint::UnixSocket { ref path } = endpoint {
                        if path.exists() {
                            return Some(endpoint);
                        }
                    } else {
                        return Some(endpoint);
                    }
                }
            }
        }

        None
    }

    async fn try_unix_socket_xdg(&self, primal_name: &str) -> Option<PathBuf> {
        let runtime_dir = self.get_xdg_runtime_dir()?;
        let biomeos_dir = runtime_dir.join("biomeos");

        let socket_path = biomeos_dir.join(format!("{}-{}.sock", primal_name, self.family_id));
        if self.verify_unix_socket(&socket_path).await {
            return Some(socket_path);
        }

        let legacy_path = biomeos_dir.join(format!("{primal_name}.sock"));
        if self.verify_unix_socket(&legacy_path).await {
            return Some(legacy_path);
        }

        None
    }

    async fn try_unix_socket_tmp(&self, primal_name: &str) -> Option<PathBuf> {
        // Use std::env::temp_dir() for portable temp directory
        let temp_dir = std::env::temp_dir();

        let socket_path = temp_dir.join(format!("{}-{}.sock", primal_name, self.family_id));
        if self.verify_unix_socket(&socket_path).await {
            return Some(socket_path);
        }

        let legacy_path = temp_dir.join(format!("{primal_name}.sock"));
        if self.verify_unix_socket(&legacy_path).await {
            return Some(legacy_path);
        }

        None
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

    #[cfg(target_os = "linux")]
    async fn try_abstract_socket(&self, primal_name: &str) -> Option<String> {
        use std::os::linux::net::SocketAddrExt;
        use std::os::unix::net::SocketAddr;

        let abstract_name = format!("biomeos_{}_{}", primal_name, self.family_id);

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
                    primal_name,
                    abstract_name,
                    e
                );
                None
            }
        }
    }

    async fn try_tcp_fallback(&self, primal_name: &str) -> Option<(String, u16)> {
        let host = &self.strategy.tcp_fallback_host;
        let prefix = primal_name.to_uppercase().replace('-', "_");

        if let Ok(tcp_env) = env::var(format!("{prefix}_TCP")) {
            if let Some(TransportEndpoint::TcpSocket { host: h, port: p }) =
                TransportEndpoint::parse(&tcp_env)
            {
                if self.verify_tcp_connection(&h, p).await {
                    return Some((h, p));
                }
            }
            if let Ok(port) = tcp_env.parse::<u16>() {
                if self.verify_tcp_connection(host, port).await {
                    return Some((host.clone(), port));
                }
            }
        }

        let port = self.calculate_primal_port(primal_name);
        if self.verify_tcp_connection(host, port).await {
            return Some((host.clone(), port));
        }

        None
    }

    pub(crate) fn calculate_primal_port(&self, primal_name: &str) -> u16 {
        let hash: u32 = primal_name.bytes().map(|b| b as u32).sum();
        let offset = (hash % 100) as u16;
        self.strategy.tcp_port_start + offset
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

    /// Build deterministic socket path for a primal
    ///
    /// Implements 5-tier socket resolution per PRIMAL_DEPLOYMENT_STANDARD.
    pub fn build_socket_path(&self, primal_name: &str) -> PathBuf {
        let socket_name = format!("{}-{}.sock", primal_name, self.family_id);

        // Tier 1: Explicit override via PRIMAL_SOCKET
        if let Ok(primal_socket) = env::var("PRIMAL_SOCKET") {
            let path = PathBuf::from(&primal_socket);
            if path.is_dir() || !path.exists() {
                return path.join(&socket_name);
            }
            return path;
        }

        // Tier 2: XDG runtime directory
        if let Some(runtime_dir) = self.get_xdg_runtime_dir() {
            let biomeos_dir = runtime_dir.join("biomeos");
            std::fs::create_dir_all(&biomeos_dir).ok();
            return biomeos_dir.join(&socket_name);
        }

        // Tier 3: Linux /run/user/$UID/biomeos/
        if let Ok(uid) = env::var("UID") {
            let run_user = PathBuf::from(format!("/run/user/{uid}/biomeos"));
            if run_user.parent().map(|p| p.exists()).unwrap_or(false) {
                std::fs::create_dir_all(&run_user).ok();
                return run_user.join(&socket_name);
            }
        }

        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if let Ok(meta) = std::fs::metadata("/proc/self") {
                let uid = meta.uid();
                let run_user = PathBuf::from(format!("/run/user/{uid}/biomeos"));
                if run_user.parent().map(|p| p.exists()).unwrap_or(false) {
                    std::fs::create_dir_all(&run_user).ok();
                    return run_user.join(&socket_name);
                }
            }
        }

        // Tier 4: Android /data/local/tmp/biomeos/
        let android_dir = PathBuf::from("/data/local/tmp/biomeos");
        if android_dir.parent().map(|p| p.exists()).unwrap_or(false) {
            std::fs::create_dir_all(&android_dir).ok();
            return android_dir.join(&socket_name);
        }

        // Tier 5: Fallback to /tmp/biomeos/
        let fallback_dir = PathBuf::from("/tmp/biomeos");
        std::fs::create_dir_all(&fallback_dir).ok();
        fallback_dir.join(&socket_name)
    }

    // ========================================================================
    // DISCOVERY IMPLEMENTATIONS
    // ========================================================================

    pub(crate) async fn discover_via_env_hint(
        &self,
        primal_name: &str,
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

        for env_var in env_patterns {
            if let Ok(path_str) = env::var(&env_var) {
                let path = PathBuf::from(&path_str);
                if path.exists() {
                    debug!("Discovered {} via env hint: {}", primal_name, env_var);
                    return Some(
                        DiscoveredSocket::from_unix_path(
                            path,
                            DiscoveryMethod::EnvironmentHint(env_var),
                        )
                        .with_primal_name(primal_name),
                    );
                }
            }
        }

        None
    }

    async fn discover_via_xdg(&self, primal_name: &str) -> Option<DiscoveredSocket> {
        let runtime_dir = self.get_xdg_runtime_dir()?;
        let biomeos_dir = runtime_dir.join("biomeos");

        let socket_path = biomeos_dir.join(format!("{}-{}.sock", primal_name, self.family_id));
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

    async fn discover_via_family_tmp(&self, primal_name: &str) -> Option<DiscoveredSocket> {
        // Use portable temp_dir() instead of hardcoded /tmp/
        let temp_dir = std::env::temp_dir();

        let socket_path = temp_dir.join(format!("{}-{}.sock", primal_name, self.family_id));
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

    async fn discover_via_registry_by_name(&self, primal_name: &str) -> Option<DiscoveredSocket> {
        let neural_api = self.get_neural_api_socket()?;

        match self
            .query_registry(
                "primal.discover",
                &serde_json::json!({ "name": primal_name }),
                &neural_api,
            )
            .await
        {
            Ok(result) => {
                let endpoint =
                    if let Some(socket_path) = result.get("socket_path").and_then(|s| s.as_str()) {
                        TransportEndpoint::UnixSocket {
                            path: PathBuf::from(socket_path),
                        }
                    } else if let Some(tcp) = result.get("tcp_endpoint").and_then(|s| s.as_str()) {
                        TransportEndpoint::parse(tcp)?
                    } else if let Some(abstract_name) =
                        result.get("abstract_socket").and_then(|s| s.as_str())
                    {
                        TransportEndpoint::AbstractSocket {
                            name: abstract_name.to_string(),
                        }
                    } else {
                        return None;
                    };

                let capabilities = result
                    .get("capabilities")
                    .and_then(|c| c.as_array())
                    .map(|arr| {
                        arr.iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect()
                    })
                    .unwrap_or_default();

                return Some(
                    DiscoveredSocket::from_endpoint(endpoint, DiscoveryMethod::CapabilityRegistry)
                        .with_primal_name(primal_name)
                        .with_capabilities(capabilities),
                );
            }
            Err(e) => {
                debug!("Registry query failed for {}: {}", primal_name, e);
            }
        }

        None
    }

    async fn discover_via_registry_by_capability(
        &self,
        capability: &str,
    ) -> Option<DiscoveredSocket> {
        let neural_api = self.get_neural_api_socket()?;

        match self
            .query_registry(
                "capability.discover",
                &serde_json::json!({ "capability": capability }),
                &neural_api,
            )
            .await
        {
            Ok(result) => {
                let endpoint = if let Some(socket_path) =
                    result.get("primary_socket").and_then(|s| s.as_str())
                {
                    TransportEndpoint::UnixSocket {
                        path: PathBuf::from(socket_path),
                    }
                } else if let Some(tcp) = result.get("tcp_endpoint").and_then(|s| s.as_str()) {
                    TransportEndpoint::parse(tcp)?
                } else {
                    return None;
                };

                let primal_name = result
                    .get("provider")
                    .and_then(|p| p.as_str())
                    .map(String::from);

                let mut socket =
                    DiscoveredSocket::from_endpoint(endpoint, DiscoveryMethod::CapabilityRegistry)
                        .with_capabilities(vec![capability.to_string()]);

                if let Some(name) = primal_name {
                    socket = socket.with_primal_name(name);
                }

                return Some(socket);
            }
            Err(e) => {
                debug!("Registry query failed for capability {}: {}", capability, e);
            }
        }

        None
    }

    async fn query_registry(
        &self,
        method: &str,
        params: &serde_json::Value,
        neural_api_socket: &Path,
    ) -> Result<serde_json::Value, String> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;
        use tokio::time::{timeout, Duration};

        let stream = timeout(
            Duration::from_secs(5),
            UnixStream::connect(neural_api_socket),
        )
        .await
        .map_err(|_| "Connection timeout")?
        .map_err(|e| format!("Connection failed: {e}"))?;

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        let request = biomeos_types::JsonRpcRequest::new(method, params.clone());

        let request_str = serde_json::to_string(&request).map_err(|e| e.to_string())? + "\n";
        writer
            .write_all(request_str.as_bytes())
            .await
            .map_err(|e| e.to_string())?;
        writer.flush().await.map_err(|e| e.to_string())?;

        let mut response_line = String::new();
        timeout(Duration::from_secs(5), reader.read_line(&mut response_line))
            .await
            .map_err(|_| "Response timeout")?
            .map_err(|e| format!("Read failed: {e}"))?;

        let response: serde_json::Value =
            serde_json::from_str(response_line.trim()).map_err(|e| format!("Parse failed: {e}"))?;

        if let Some(error) = response.get("error") {
            return Err(format!("Registry error: {error}"));
        }

        response
            .get("result")
            .cloned()
            .ok_or_else(|| "No result in response".to_string())
    }

    // ========================================================================
    // HELPERS
    // ========================================================================

    pub(crate) fn get_xdg_runtime_dir(&self) -> Option<PathBuf> {
        env::var("XDG_RUNTIME_DIR")
            .ok()
            .map(PathBuf::from)
            .filter(|p| p.exists())
    }

    pub(crate) fn get_neural_api_socket(&self) -> Option<PathBuf> {
        if let Some(ref socket) = self.neural_api_socket {
            if socket.exists() {
                return Some(socket.clone());
            }
        }

        if let Ok(path) = env::var("NEURAL_API_SOCKET") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Some(path);
            }
        }

        // Use portable temp_dir() instead of hardcoded /tmp/
        let temp_dir = std::env::temp_dir();
        let standard_locations = vec![
            temp_dir.join(format!("neural-api-{}.sock", self.family_id)),
            temp_dir.join("neural-api.sock"),
        ];

        standard_locations.into_iter().find(|path| path.exists())
    }

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
                key.to_string(),
                CachedSocket {
                    socket: socket.clone(),
                    cached_at: std::time::Instant::now(),
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

// Tests are in engine_tests.rs to keep this file under 1000 lines
