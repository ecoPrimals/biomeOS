//! Socket Discovery - Capability-Based Runtime Discovery
//!
//! **Deep Debt Solution**: Replaces hardcoded `/tmp/{primal}.sock` paths with
//! capability-based discovery that respects system conventions and primal self-knowledge.
//!
//! ## Universal IPC Standard v3.0 Compliance
//!
//! This module implements the Universal IPC Standard v3.0 for biomeOS:
//! - Multi-transport support (Unix, Abstract, TCP)
//! - Tier 1 → Tier 2 graceful fallback
//! - Platform-agnostic transport selection
//! - Runtime discovery (no hardcoded primal knowledge)
//!
//! ## Principles
//!
//! 1. **No Hardcoding**: Socket paths discovered at runtime
//! 2. **XDG Compliance**: Respects XDG_RUNTIME_DIR when available
//! 3. **Family-Based Isolation**: Sockets namespaced by family_id
//! 4. **Capability Discovery**: Find primals by what they do, not where they are
//! 5. **Platform Agnostic**: Works across Linux, macOS, Android, and other systems
//! 6. **Graceful Fallback**: Tier 1 (Unix/Abstract) → Tier 2 (TCP) automatically
//!
//! ## Transport Tiers
//!
//! - **Tier 1 (Native)**: Unix sockets, Abstract sockets (Linux/Android)
//! - **Tier 2 (Universal)**: TCP sockets (cross-device, WASM, restricted environments)
//!
//! ## Discovery Order
//!
//! 1. Environment variable hint (e.g., `BEARDOG_SOCKET`, `BEARDOG_TCP`)
//! 2. XDG_RUNTIME_DIR (e.g., `/run/user/1000/biomeos/beardog-nat0.sock`)
//! 3. Abstract socket (Android: `@biomeos_beardog_nat0`)
//! 4. Family-scoped /tmp (e.g., `/tmp/beardog-nat0.sock`)
//! 5. Capability registry query via Neural API
//! 6. TCP fallback (e.g., `127.0.0.1:9100`)
//!
//! ## Usage
//!
//! ```ignore
//! use biomeos_core::socket_discovery::{SocketDiscovery, TransportEndpoint};
//!
//! let discovery = SocketDiscovery::new("nat0");
//!
//! // Discover with automatic fallback
//! let endpoint = discovery.discover_with_fallback("beardog").await?;
//! match endpoint {
//!     TransportEndpoint::UnixSocket { path } => { /* connect via Unix */ }
//!     TransportEndpoint::AbstractSocket { name } => { /* connect via abstract */ }
//!     TransportEndpoint::TcpSocket { host, port } => { /* connect via TCP */ }
//! }
//! ```

use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::net::{TcpStream, UnixStream};
use tokio::sync::RwLock;
use tracing::{debug, info, trace, warn};

// ============================================================================
// TRANSPORT ENDPOINT - Universal IPC Standard v3.0
// ============================================================================

/// Transport endpoint for connecting to a primal
///
/// Implements the Universal IPC Standard v3.0 transport tiers:
/// - **Tier 1 (Native)**: `UnixSocket`, `AbstractSocket` - highest performance
/// - **Tier 2 (Universal)**: `TcpSocket` - cross-device, WASM compatible
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportEndpoint {
    /// Unix domain socket (Tier 1 - Linux, macOS)
    UnixSocket {
        /// Path to the socket file
        path: PathBuf,
    },

    /// Abstract socket (Tier 1 - Linux, Android)
    /// Bypasses filesystem, immune to SELinux restrictions
    AbstractSocket {
        /// Abstract socket name (without leading `@`)
        name: String,
    },

    /// TCP socket (Tier 2 - Universal fallback)
    TcpSocket {
        /// Host address
        host: String,
        /// Port number
        port: u16,
    },
}

impl TransportEndpoint {
    /// Get the tier level (1 = native, 2 = universal)
    pub fn tier(&self) -> u8 {
        match self {
            Self::UnixSocket { .. } | Self::AbstractSocket { .. } => 1,
            Self::TcpSocket { .. } => 2,
        }
    }

    /// Check if this is a Tier 1 (native) transport
    pub fn is_native(&self) -> bool {
        self.tier() == 1
    }

    /// Get a display string for logging
    pub fn display_string(&self) -> String {
        match self {
            Self::UnixSocket { path } => format!("unix://{}", path.display()),
            Self::AbstractSocket { name } => format!("abstract://@{}", name),
            Self::TcpSocket { host, port } => format!("tcp://{}:{}", host, port),
        }
    }

    /// Parse from environment variable value
    ///
    /// Supports formats:
    /// - `/path/to/socket.sock` → UnixSocket
    /// - `@abstract_name` → AbstractSocket  
    /// - `host:port` or `tcp://host:port` → TcpSocket
    pub fn parse(value: &str) -> Option<Self> {
        let value = value.trim();

        // Abstract socket: starts with @
        if value.starts_with('@') {
            return Some(Self::AbstractSocket {
                name: value[1..].to_string(),
            });
        }

        // TCP: explicit prefix or host:port format
        if let Some(stripped) = value.strip_prefix("tcp://") {
            return Self::parse_tcp(stripped);
        }

        // TCP: contains colon and doesn't look like a path
        if value.contains(':') && !value.starts_with('/') {
            return Self::parse_tcp(value);
        }

        // Unix socket: path
        if value.starts_with('/') || value.contains(".sock") {
            return Some(Self::UnixSocket {
                path: PathBuf::from(value),
            });
        }

        None
    }

    fn parse_tcp(value: &str) -> Option<Self> {
        let parts: Vec<&str> = value.rsplitn(2, ':').collect();
        if parts.len() == 2 {
            let port: u16 = parts[0].parse().ok()?;
            let host = parts[1].to_string();
            return Some(Self::TcpSocket { host, port });
        }
        None
    }
}

impl std::fmt::Display for TransportEndpoint {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.display_string())
    }
}

/// Socket discovery result
#[derive(Debug, Clone)]
pub struct DiscoveredSocket {
    /// Path to the socket (deprecated - use `endpoint` for multi-transport)
    pub path: PathBuf,

    /// Transport endpoint (Universal IPC v3.0)
    pub endpoint: TransportEndpoint,

    /// How it was discovered
    pub discovered_via: DiscoveryMethod,

    /// Primal name (if known)
    pub primal_name: Option<String>,

    /// Capabilities provided (if known)
    pub capabilities: Vec<String>,
}

impl DiscoveredSocket {
    /// Create from a Unix socket path (convenience constructor)
    pub fn from_unix_path(path: PathBuf, via: DiscoveryMethod) -> Self {
        Self {
            endpoint: TransportEndpoint::UnixSocket { path: path.clone() },
            path,
            discovered_via: via,
            primal_name: None,
            capabilities: Vec::new(),
        }
    }

    /// Create from a transport endpoint
    pub fn from_endpoint(endpoint: TransportEndpoint, via: DiscoveryMethod) -> Self {
        let path = match &endpoint {
            TransportEndpoint::UnixSocket { path } => path.clone(),
            _ => PathBuf::new(), // Non-path transports
        };
        Self {
            endpoint,
            path,
            discovered_via: via,
            primal_name: None,
            capabilities: Vec::new(),
        }
    }

    /// Set the primal name
    pub fn with_primal_name(mut self, name: impl Into<String>) -> Self {
        self.primal_name = Some(name.into());
        self
    }

    /// Set capabilities
    pub fn with_capabilities(mut self, caps: Vec<String>) -> Self {
        self.capabilities = caps;
        self
    }
}

/// How a socket was discovered
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryMethod {
    /// Via environment variable hint
    EnvironmentHint(String),

    /// Via XDG runtime directory
    XdgRuntime,

    /// Via abstract socket (Linux/Android)
    AbstractSocket,

    /// Via family-scoped /tmp
    FamilyTmp,

    /// Via capability registry query
    CapabilityRegistry,

    /// Via TCP fallback
    TcpFallback,

    /// Via socket scanning
    SocketScan,

    /// Cached from previous discovery
    Cached,
}

/// Discovery strategy configuration
#[derive(Debug, Clone)]
pub struct DiscoveryStrategy {
    /// Check environment hints first
    pub check_env_hints: bool,

    /// Use XDG runtime dir
    pub use_xdg_runtime: bool,

    /// Try abstract sockets (Linux/Android)
    pub try_abstract_sockets: bool,

    /// Use family-scoped /tmp
    pub use_family_tmp: bool,

    /// Query capability registry
    pub query_registry: bool,

    /// Enable TCP fallback (Tier 2)
    pub enable_tcp_fallback: bool,

    /// Default TCP port range start for auto-discovery
    pub tcp_port_start: u16,

    /// Default TCP host for fallback
    pub tcp_fallback_host: String,

    /// Scan for sockets
    pub scan_sockets: bool,

    /// Cache discovered sockets
    pub enable_cache: bool,

    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,
}

impl Default for DiscoveryStrategy {
    fn default() -> Self {
        Self {
            check_env_hints: true,
            use_xdg_runtime: true,
            try_abstract_sockets: cfg!(target_os = "linux"), // Only on Linux/Android
            use_family_tmp: true,
            query_registry: true,
            enable_tcp_fallback: true, // Universal IPC v3.0: always try TCP
            tcp_port_start: 9100,      // Default port range for primals
            tcp_fallback_host: "127.0.0.1".to_string(),
            scan_sockets: false, // Expensive, disabled by default
            enable_cache: true,
            cache_ttl_secs: 60,
        }
    }
}

impl DiscoveryStrategy {
    /// Create a strategy optimized for Android
    pub fn android() -> Self {
        Self {
            check_env_hints: true,
            use_xdg_runtime: false, // Android doesn't use XDG
            try_abstract_sockets: true, // Prefer abstract sockets
            use_family_tmp: false,  // SELinux may block /tmp sockets
            query_registry: true,
            enable_tcp_fallback: true,
            tcp_port_start: 9100,
            tcp_fallback_host: "127.0.0.1".to_string(),
            scan_sockets: false,
            enable_cache: true,
            cache_ttl_secs: 60,
        }
    }

    /// Create a strategy optimized for cross-device communication
    pub fn cross_device() -> Self {
        Self {
            check_env_hints: true,
            use_xdg_runtime: false,
            try_abstract_sockets: false, // Not cross-device
            use_family_tmp: false,       // Not cross-device
            query_registry: true,
            enable_tcp_fallback: true,   // TCP is primary for cross-device
            tcp_port_start: 9100,
            tcp_fallback_host: "0.0.0.0".to_string(), // Listen on all interfaces
            scan_sockets: false,
            enable_cache: true,
            cache_ttl_secs: 30, // Shorter TTL for dynamic environments
        }
    }
}

/// Socket discovery engine
///
/// Provides capability-based socket discovery without hardcoded paths.
pub struct SocketDiscovery {
    /// Family ID for namespace isolation
    family_id: String,

    /// Discovery strategy
    strategy: DiscoveryStrategy,

    /// Discovery cache
    cache: Arc<RwLock<HashMap<String, CachedSocket>>>,

    /// Neural API socket (for capability registry queries)
    neural_api_socket: Option<PathBuf>,
}

/// Cached socket entry
struct CachedSocket {
    socket: DiscoveredSocket,
    cached_at: std::time::Instant,
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
    ///
    /// Follows discovery order:
    /// 1. Check cache
    /// 2. Environment hint (e.g., BEARDOG_SOCKET)
    /// 3. XDG runtime dir
    /// 4. Family-scoped /tmp
    /// 5. Capability registry
    pub async fn discover_primal(&self, primal_name: &str) -> Option<DiscoveredSocket> {
        let cache_key = format!("primal:{}", primal_name);

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
    ///
    /// E.g., `discover_capability("crypto")` finds BearDog
    pub async fn discover_capability(&self, capability: &str) -> Option<DiscoveredSocket> {
        let cache_key = format!("capability:{}", capability);

        // 1. Check cache
        if self.strategy.enable_cache {
            if let Some(cached) = self.check_cache(&cache_key).await {
                return Some(cached);
            }
        }

        // 2. Query capability registry
        if self.strategy.query_registry {
            if let Some(socket) = self.discover_via_registry_by_capability(capability).await {
                self.cache_socket(&cache_key, &socket).await;
                return Some(socket);
            }
        }

        // 3. No hardcoded fallback - require explicit configuration
        warn!(
            "Socket not found for capability '{}'. \
             To resolve: (1) Start a primal providing '{}' capability, \
             (2) Set {}_SOCKET or {}_ENDPOINT environment variable, or \
             (3) Ensure Neural API registry is accessible for runtime discovery.",
            capability,
            capability,
            capability.to_uppercase(),
            capability.to_uppercase()
        );
        None
    }

    /// Get socket path for a primal (convenience method)
    ///
    /// Returns the path directly, or None if not found.
    pub async fn get_socket_path(&self, primal_name: &str) -> Option<PathBuf> {
        self.discover_primal(primal_name).await.map(|s| s.path)
    }

    // ========================================================================
    // UNIVERSAL IPC v3.0: MULTI-TRANSPORT DISCOVERY WITH FALLBACK
    // ========================================================================

    /// Discover primal with automatic Tier 1 → Tier 2 fallback
    ///
    /// **Universal IPC Standard v3.0**: Implements graceful transport fallback:
    /// 1. Try Tier 1 transports (Unix socket, Abstract socket) - highest performance
    /// 2. Fall back to Tier 2 (TCP) if Tier 1 fails - universal compatibility
    ///
    /// This ensures cross-device and cross-platform communication works even when
    /// native IPC is unavailable (Android SELinux, WASM, network-isolated containers).
    ///
    /// # Arguments
    /// * `primal_name` - Name of the primal to discover
    ///
    /// # Returns
    /// `TransportEndpoint` ready for connection, or `None` if discovery failed
    ///
    /// # Example
    /// ```ignore
    /// let endpoint = discovery.discover_with_fallback("beardog").await?;
    /// match endpoint {
    ///     TransportEndpoint::UnixSocket { path } => connect_unix(&path).await,
    ///     TransportEndpoint::AbstractSocket { name } => connect_abstract(&name).await,
    ///     TransportEndpoint::TcpSocket { host, port } => connect_tcp(&host, port).await,
    /// }
    /// ```
    pub async fn discover_with_fallback(&self, primal_name: &str) -> Option<TransportEndpoint> {
        let cache_key = format!("endpoint:{}", primal_name);

        // 1. Check cache
        if self.strategy.enable_cache {
            if let Some(cached) = self.check_cache(&cache_key).await {
                return Some(cached.endpoint);
            }
        }

        // 2. Try environment hint (supports all transport types)
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
                trace!("Discovered {} via XDG Unix socket: {}", primal_name, endpoint);
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
                trace!(
                    "Discovered {} via abstract socket: {}",
                    primal_name,
                    endpoint
                );
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
                trace!("Discovered {} via /tmp Unix socket: {}", primal_name, endpoint);
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

        warn!(
            "Primal '{}' not found via any transport. Tried: env, XDG, abstract, /tmp, registry, TCP",
            primal_name
        );
        None
    }

    /// Get the transport endpoint for a primal (convenience method)
    ///
    /// Returns `TransportEndpoint` with automatic fallback.
    pub async fn get_endpoint(&self, primal_name: &str) -> Option<TransportEndpoint> {
        self.discover_with_fallback(primal_name).await
    }

    // ========================================================================
    // TRANSPORT-SPECIFIC DISCOVERY HELPERS
    // ========================================================================

    /// Discover endpoint via environment variable
    ///
    /// Checks for:
    /// - `{PRIMAL}_SOCKET` - Unix socket path or abstract socket (@name)
    /// - `{PRIMAL}_TCP` - TCP host:port
    /// - `{PRIMAL}_ENDPOINT` - Any endpoint format
    async fn discover_endpoint_via_env(&self, primal_name: &str) -> Option<TransportEndpoint> {
        let prefix = primal_name.to_uppercase().replace('-', "_");

        // Check TCP first (explicit preference)
        if let Ok(tcp) = env::var(format!("{}_TCP", prefix)) {
            if let Some(endpoint) = TransportEndpoint::parse(&tcp) {
                if matches!(endpoint, TransportEndpoint::TcpSocket { .. }) {
                    return Some(endpoint);
                }
            }
            // Try parsing as host:port directly
            if let Some(endpoint) = TransportEndpoint::parse(&format!("tcp://{}", tcp)) {
                return Some(endpoint);
            }
        }

        // Check generic endpoint
        if let Ok(endpoint_str) = env::var(format!("{}_ENDPOINT", prefix)) {
            if let Some(endpoint) = TransportEndpoint::parse(&endpoint_str) {
                return Some(endpoint);
            }
        }

        // Check socket (could be Unix or abstract)
        for var_name in [
            format!("{}_SOCKET", prefix),
            format!("{}_SOCKET_PATH", prefix),
            format!("BIOMEOS_{}_SOCKET", prefix),
        ] {
            if let Ok(value) = env::var(&var_name) {
                if let Some(endpoint) = TransportEndpoint::parse(&value) {
                    // For Unix sockets, verify the path exists
                    if let TransportEndpoint::UnixSocket { ref path } = endpoint {
                        if path.exists() {
                            return Some(endpoint);
                        }
                    } else {
                        // Abstract and TCP don't need path existence check
                        return Some(endpoint);
                    }
                }
            }
        }

        None
    }

    /// Try to find Unix socket in XDG runtime directory
    async fn try_unix_socket_xdg(&self, primal_name: &str) -> Option<PathBuf> {
        let runtime_dir = self.get_xdg_runtime_dir()?;
        let biomeos_dir = runtime_dir.join("biomeos");

        // Family-namespaced first
        let socket_path = biomeos_dir.join(format!("{}-{}.sock", primal_name, self.family_id));
        if self.verify_unix_socket(&socket_path).await {
            return Some(socket_path);
        }

        // Legacy without family namespace
        let legacy_path = biomeos_dir.join(format!("{}.sock", primal_name));
        if self.verify_unix_socket(&legacy_path).await {
            return Some(legacy_path);
        }

        None
    }

    /// Try to find Unix socket in /tmp
    async fn try_unix_socket_tmp(&self, primal_name: &str) -> Option<PathBuf> {
        // Family-namespaced
        let socket_path = PathBuf::from(format!("/tmp/{}-{}.sock", primal_name, self.family_id));
        if self.verify_unix_socket(&socket_path).await {
            return Some(socket_path);
        }

        // Legacy
        let legacy_path = PathBuf::from(format!("/tmp/{}.sock", primal_name));
        if self.verify_unix_socket(&legacy_path).await {
            return Some(legacy_path);
        }

        None
    }

    /// Verify Unix socket is connectable
    async fn verify_unix_socket(&self, path: &Path) -> bool {
        if !path.exists() {
            return false;
        }

        // Actually try to connect to verify it's a live socket
        match tokio::time::timeout(
            std::time::Duration::from_millis(500),
            UnixStream::connect(path),
        )
        .await
        {
            Ok(Ok(_)) => true,
            Ok(Err(e)) => {
                trace!("Unix socket exists but connection failed: {} - {}", path.display(), e);
                false
            }
            Err(_) => {
                trace!("Unix socket connection timed out: {}", path.display());
                false
            }
        }
    }

    /// Try to connect via abstract socket (Linux/Android)
    ///
    /// Abstract sockets bypass the filesystem entirely, avoiding SELinux issues.
    /// Format: `@biomeos_{primal_name}_{family_id}`
    #[cfg(target_os = "linux")]
    async fn try_abstract_socket(&self, primal_name: &str) -> Option<String> {
        use std::os::linux::net::SocketAddrExt;
        use std::os::unix::net::SocketAddr;

        // Build abstract socket name
        let abstract_name = format!("biomeos_{}_{}", primal_name, self.family_id);

        // Try to connect to verify it's listening
        // Abstract sockets use a null byte prefix internally
        let addr = match SocketAddr::from_abstract_name(&abstract_name) {
            Ok(addr) => addr,
            Err(e) => {
                trace!("Failed to create abstract socket addr: {}", e);
                return None;
            }
        };

        // Use tokio's connect_addr if available, or fall back to sync check
        // For now, we'll use a sync check since tokio doesn't expose abstract socket connect directly
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

    /// Try TCP fallback for the primal
    ///
    /// Checks:
    /// 1. Environment variable `{PRIMAL}_TCP`
    /// 2. Well-known port based on primal name
    /// 3. Port range scanning (if enabled)
    async fn try_tcp_fallback(&self, primal_name: &str) -> Option<(String, u16)> {
        let host = &self.strategy.tcp_fallback_host;

        // Check environment variable first
        let prefix = primal_name.to_uppercase().replace('-', "_");
        if let Ok(tcp_env) = env::var(format!("{}_TCP", prefix)) {
            if let Some(TransportEndpoint::TcpSocket { host: h, port: p }) =
                TransportEndpoint::parse(&tcp_env)
            {
                if self.verify_tcp_connection(&h, p).await {
                    return Some((h, p));
                }
            }
            // Try as just port number
            if let Ok(port) = tcp_env.parse::<u16>() {
                if self.verify_tcp_connection(host, port).await {
                    return Some((host.clone(), port));
                }
            }
        }

        // Try well-known port assignment based on primal name
        // This is a deterministic port assignment: hash(primal_name) % range + base
        let port = self.calculate_primal_port(primal_name);
        if self.verify_tcp_connection(host, port).await {
            return Some((host.clone(), port));
        }

        None
    }

    /// Calculate deterministic port for a primal
    ///
    /// Uses a simple hash to assign ports in range [tcp_port_start, tcp_port_start + 100)
    fn calculate_primal_port(&self, primal_name: &str) -> u16 {
        // Simple hash: sum of bytes mod 100 + base port
        let hash: u32 = primal_name.bytes().map(|b| b as u32).sum();
        let offset = (hash % 100) as u16;
        self.strategy.tcp_port_start + offset
    }

    /// Verify TCP connection is available
    async fn verify_tcp_connection(&self, host: &str, port: u16) -> bool {
        let addr = format!("{}:{}", host, port);
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
    /// This is used when we KNOW the socket should exist at a specific location.
    /// Used by primals to register their own sockets.
    ///
    /// Implements 5-tier socket resolution per PRIMAL_DEPLOYMENT_STANDARD:
    /// Tier 1: $PRIMAL_SOCKET (explicit override)
    /// Tier 2: $XDG_RUNTIME_DIR/biomeos/
    /// Tier 3: /run/user/$UID/biomeos/
    /// Tier 4: /data/local/tmp/biomeos/ (Android)
    /// Tier 5: /tmp/biomeos/ (fallback)
    ///
    /// Socket naming convention: `{primal_name}-{family_id}.sock`
    pub fn build_socket_path(&self, primal_name: &str) -> PathBuf {
        let socket_name = format!("{}-{}.sock", primal_name, self.family_id);

        // Tier 1: Explicit override via PRIMAL_SOCKET
        if let Ok(primal_socket) = env::var("PRIMAL_SOCKET") {
            let path = PathBuf::from(&primal_socket);
            // If it's a directory, append socket name; otherwise use as-is
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
            let run_user = PathBuf::from(format!("/run/user/{}/biomeos", uid));
            if run_user.parent().map(|p| p.exists()).unwrap_or(false) {
                std::fs::create_dir_all(&run_user).ok();
                return run_user.join(&socket_name);
            }
        }

        // Also try /proc/self for UID (Linux-specific)
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if let Ok(meta) = std::fs::metadata("/proc/self") {
                let uid = meta.uid();
                let run_user = PathBuf::from(format!("/run/user/{}/biomeos", uid));
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

    /// Discover via environment variable hint
    async fn discover_via_env_hint(&self, primal_name: &str) -> Option<DiscoveredSocket> {
        // Try various environment variable patterns
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

    /// Discover via XDG runtime directory
    async fn discover_via_xdg(&self, primal_name: &str) -> Option<DiscoveredSocket> {
        let runtime_dir = self.get_xdg_runtime_dir()?;
        let biomeos_dir = runtime_dir.join("biomeos");

        // Try family-namespaced path
        let socket_path = biomeos_dir.join(format!("{}-{}.sock", primal_name, self.family_id));
        if socket_path.exists() {
            debug!("Discovered {} via XDG runtime", primal_name);
            return Some(
                DiscoveredSocket::from_unix_path(socket_path, DiscoveryMethod::XdgRuntime)
                    .with_primal_name(primal_name),
            );
        }

        // Try without family namespace (legacy)
        let legacy_path = biomeos_dir.join(format!("{}.sock", primal_name));
        if legacy_path.exists() {
            debug!("Discovered {} via XDG runtime (legacy)", primal_name);
            return Some(
                DiscoveredSocket::from_unix_path(legacy_path, DiscoveryMethod::XdgRuntime)
                    .with_primal_name(primal_name),
            );
        }

        None
    }

    /// Discover via family-scoped /tmp
    async fn discover_via_family_tmp(&self, primal_name: &str) -> Option<DiscoveredSocket> {
        // Family-namespaced path (preferred)
        let socket_path = PathBuf::from(format!("/tmp/{}-{}.sock", primal_name, self.family_id));
        if socket_path.exists() {
            debug!("Discovered {} via family /tmp", primal_name);
            return Some(
                DiscoveredSocket::from_unix_path(socket_path, DiscoveryMethod::FamilyTmp)
                    .with_primal_name(primal_name),
            );
        }

        // Legacy path without family namespace
        let legacy_path = PathBuf::from(format!("/tmp/{}.sock", primal_name));
        if legacy_path.exists() {
            debug!("Discovered {} via /tmp (legacy)", primal_name);
            return Some(
                DiscoveredSocket::from_unix_path(legacy_path, DiscoveryMethod::FamilyTmp)
                    .with_primal_name(primal_name),
            );
        }

        None
    }

    /// Query capability registry by primal name
    async fn discover_via_registry_by_name(&self, primal_name: &str) -> Option<DiscoveredSocket> {
        let neural_api = self.get_neural_api_socket()?;

        match self
            .query_registry(
                "primal.discover",
                &serde_json::json!({
                    "name": primal_name
                }),
                &neural_api,
            )
            .await
        {
            Ok(result) => {
                // Parse endpoint from registry response
                // Supports: socket_path, tcp_endpoint, abstract_socket
                let endpoint = if let Some(socket_path) =
                    result.get("socket_path").and_then(|s| s.as_str())
                {
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

    /// Query capability registry by capability
    async fn discover_via_registry_by_capability(
        &self,
        capability: &str,
    ) -> Option<DiscoveredSocket> {
        let neural_api = self.get_neural_api_socket()?;

        match self
            .query_registry(
                "capability.discover",
                &serde_json::json!({
                    "capability": capability
                }),
                &neural_api,
            )
            .await
        {
            Ok(result) => {
                // Parse endpoint from registry response
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

                let mut socket = DiscoveredSocket::from_endpoint(
                    endpoint,
                    DiscoveryMethod::CapabilityRegistry,
                )
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

    /// Query the capability registry via Neural API
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
        .map_err(|e| format!("Connection failed: {}", e))?;

        let (reader, mut writer) = stream.into_split();
        let mut reader = BufReader::new(reader);

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": method,
            "params": params,
            "id": 1
        });

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
            .map_err(|e| format!("Read failed: {}", e))?;

        let response: serde_json::Value = serde_json::from_str(response_line.trim())
            .map_err(|e| format!("Parse failed: {}", e))?;

        if let Some(error) = response.get("error") {
            return Err(format!("Registry error: {}", error));
        }

        response
            .get("result")
            .cloned()
            .ok_or_else(|| "No result in response".to_string())
    }

    // ========================================================================
    // HELPERS
    // ========================================================================

    /// Get XDG_RUNTIME_DIR
    fn get_xdg_runtime_dir(&self) -> Option<PathBuf> {
        env::var("XDG_RUNTIME_DIR")
            .ok()
            .map(PathBuf::from)
            .filter(|p| p.exists())
    }

    /// Get Neural API socket path
    fn get_neural_api_socket(&self) -> Option<PathBuf> {
        // Use configured socket
        if let Some(ref socket) = self.neural_api_socket {
            if socket.exists() {
                return Some(socket.clone());
            }
        }

        // Try environment hint
        if let Ok(path) = env::var("NEURAL_API_SOCKET") {
            let path = PathBuf::from(path);
            if path.exists() {
                return Some(path);
            }
        }

        // Try standard locations
        let standard_locations = vec![
            PathBuf::from(format!("/tmp/neural-api-{}.sock", self.family_id)),
            PathBuf::from("/tmp/neural-api.sock"),
        ];

        standard_locations.into_iter().find(|path| path.exists())
    }

    /// REMOVED: Hardcoded capability→primal mappings
    ///
    /// This function has been removed to adhere to the TRUE PRIMAL principle:
    /// "Primals do NOT have hardcoded knowledge of other primals"
    ///
    /// Instead, capability discovery now relies solely on:
    /// 1. Runtime registry queries (Neural API)
    /// 2. Environment variables (e.g., SECURITY_SOCKET, AI_ENDPOINT)
    /// 3. Explicit configuration
    ///
    /// This ensures biomeOS remains agnostic and capability-based, with no
    /// compile-time coupling to specific primal implementations.

    /// Check cache for a socket
    async fn check_cache(&self, key: &str) -> Option<DiscoveredSocket> {
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
    async fn cache_socket(&self, key: &str, socket: &DiscoveredSocket) {
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

// ============================================================================
// CONVENIENCE FUNCTIONS (for quick migrations from hardcoded paths)
// ============================================================================

/// Discover socket for a primal (convenience function)
///
/// Uses default family_id from FAMILY_ID or BIOMEOS_FAMILY_ID environment.
pub async fn discover_socket(primal_name: &str) -> Option<PathBuf> {
    let family_id = env::var("FAMILY_ID")
        .or_else(|_| env::var("BIOMEOS_FAMILY_ID"))
        .unwrap_or_else(|_| "default".to_string());

    let discovery = SocketDiscovery::new(family_id);
    discovery.get_socket_path(primal_name).await
}

/// Build socket path for a primal (convenience function)
///
/// Deterministic path building for primals to register their own sockets.
pub fn build_socket(primal_name: &str, family_id: &str) -> PathBuf {
    let discovery = SocketDiscovery::new(family_id);
    discovery.build_socket_path(primal_name)
}

// ============================================================================
// CONVENIENCE FUNCTIONS - Universal IPC v3.0
// ============================================================================

/// Discover transport endpoint with automatic fallback (convenience function)
///
/// **Universal IPC Standard v3.0**: Use this for cross-platform discovery.
pub async fn discover_endpoint(primal_name: &str) -> Option<TransportEndpoint> {
    let family_id = env::var("FAMILY_ID")
        .or_else(|_| env::var("BIOMEOS_FAMILY_ID"))
        .unwrap_or_else(|_| "default".to_string());

    let discovery = SocketDiscovery::new(family_id);
    discovery.discover_with_fallback(primal_name).await
}

// ============================================================================
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // ========================================================================
    // TransportEndpoint Tests
    // ========================================================================

    #[test]
    fn test_transport_endpoint_parse_unix() {
        let endpoint = TransportEndpoint::parse("/tmp/beardog.sock").unwrap();
        assert!(matches!(endpoint, TransportEndpoint::UnixSocket { .. }));
        if let TransportEndpoint::UnixSocket { path } = endpoint {
            assert_eq!(path, PathBuf::from("/tmp/beardog.sock"));
        }
    }

    #[test]
    fn test_transport_endpoint_parse_abstract() {
        let endpoint = TransportEndpoint::parse("@biomeos_beardog_nat0").unwrap();
        assert!(matches!(endpoint, TransportEndpoint::AbstractSocket { .. }));
        if let TransportEndpoint::AbstractSocket { name } = endpoint {
            assert_eq!(name, "biomeos_beardog_nat0");
        }
    }

    #[test]
    fn test_transport_endpoint_parse_tcp() {
        let endpoint = TransportEndpoint::parse("127.0.0.1:9100").unwrap();
        assert!(matches!(endpoint, TransportEndpoint::TcpSocket { .. }));
        if let TransportEndpoint::TcpSocket { host, port } = endpoint {
            assert_eq!(host, "127.0.0.1");
            assert_eq!(port, 9100);
        }
    }

    #[test]
    fn test_transport_endpoint_parse_tcp_with_prefix() {
        let endpoint = TransportEndpoint::parse("tcp://192.168.1.100:8080").unwrap();
        assert!(matches!(endpoint, TransportEndpoint::TcpSocket { .. }));
        if let TransportEndpoint::TcpSocket { host, port } = endpoint {
            assert_eq!(host, "192.168.1.100");
            assert_eq!(port, 8080);
        }
    }

    #[test]
    fn test_transport_endpoint_tier() {
        let unix = TransportEndpoint::UnixSocket {
            path: PathBuf::from("/tmp/test.sock"),
        };
        assert_eq!(unix.tier(), 1);
        assert!(unix.is_native());

        let abstract_sock = TransportEndpoint::AbstractSocket {
            name: "test".to_string(),
        };
        assert_eq!(abstract_sock.tier(), 1);
        assert!(abstract_sock.is_native());

        let tcp = TransportEndpoint::TcpSocket {
            host: "127.0.0.1".to_string(),
            port: 9100,
        };
        assert_eq!(tcp.tier(), 2);
        assert!(!tcp.is_native());
    }

    #[test]
    fn test_transport_endpoint_display() {
        let unix = TransportEndpoint::UnixSocket {
            path: PathBuf::from("/tmp/test.sock"),
        };
        assert_eq!(unix.display_string(), "unix:///tmp/test.sock");

        let tcp = TransportEndpoint::TcpSocket {
            host: "localhost".to_string(),
            port: 9100,
        };
        assert_eq!(tcp.display_string(), "tcp://localhost:9100");
    }

    // ========================================================================
    // DiscoveryStrategy Tests
    // ========================================================================

    #[test]
    fn test_build_socket_path() {
        let discovery = SocketDiscovery::new("test-family");
        let path = discovery.build_socket_path("beardog");

        // Should be /tmp/beardog-test-family.sock (unless XDG_RUNTIME_DIR is set)
        assert!(path.to_string_lossy().contains("beardog"));
        assert!(path.to_string_lossy().contains("test-family"));
    }

    #[test]
    fn test_discovery_strategy_defaults() {
        let strategy = DiscoveryStrategy::default();

        assert!(strategy.check_env_hints);
        assert!(strategy.use_xdg_runtime);
        assert!(strategy.use_family_tmp);
        assert!(strategy.query_registry);
        assert!(strategy.enable_tcp_fallback); // Universal IPC v3.0
        assert!(!strategy.scan_sockets); // Expensive, disabled by default
        assert!(strategy.enable_cache);
    }

    #[test]
    fn test_discovery_strategy_android() {
        let strategy = DiscoveryStrategy::android();

        assert!(strategy.try_abstract_sockets);
        assert!(!strategy.use_xdg_runtime); // Android doesn't use XDG
        assert!(!strategy.use_family_tmp);  // SELinux may block
        assert!(strategy.enable_tcp_fallback);
    }

    #[test]
    fn test_discovery_strategy_cross_device() {
        let strategy = DiscoveryStrategy::cross_device();

        assert!(!strategy.try_abstract_sockets); // Not cross-device
        assert!(!strategy.use_family_tmp);       // Not cross-device
        assert!(strategy.enable_tcp_fallback);   // Primary for cross-device
        assert_eq!(strategy.tcp_fallback_host, "0.0.0.0");
    }

    // ========================================================================
    // DiscoveredSocket Tests
    // ========================================================================

    #[test]
    fn test_discovered_socket_from_unix_path() {
        let socket =
            DiscoveredSocket::from_unix_path(PathBuf::from("/tmp/test.sock"), DiscoveryMethod::FamilyTmp);

        assert_eq!(socket.path, PathBuf::from("/tmp/test.sock"));
        assert!(matches!(socket.endpoint, TransportEndpoint::UnixSocket { .. }));
        assert_eq!(socket.discovered_via, DiscoveryMethod::FamilyTmp);
    }

    #[test]
    fn test_discovered_socket_from_endpoint() {
        let endpoint = TransportEndpoint::TcpSocket {
            host: "127.0.0.1".to_string(),
            port: 9100,
        };
        let socket = DiscoveredSocket::from_endpoint(endpoint.clone(), DiscoveryMethod::TcpFallback);

        assert!(matches!(socket.endpoint, TransportEndpoint::TcpSocket { .. }));
        assert_eq!(socket.discovered_via, DiscoveryMethod::TcpFallback);
        // TCP sockets have empty path
        assert!(socket.path.as_os_str().is_empty());
    }

    #[test]
    fn test_discovered_socket_builders() {
        let socket = DiscoveredSocket::from_unix_path(
            PathBuf::from("/tmp/test.sock"),
            DiscoveryMethod::FamilyTmp,
        )
        .with_primal_name("beardog")
        .with_capabilities(vec!["crypto".to_string(), "identity".to_string()]);

        assert_eq!(socket.primal_name, Some("beardog".to_string()));
        assert_eq!(socket.capabilities.len(), 2);
        assert!(socket.capabilities.contains(&"crypto".to_string()));
    }

    // ========================================================================
    // Async Discovery Tests
    // ========================================================================

    #[tokio::test]
    async fn test_env_hint_discovery() {
        // Set environment variable
        std::env::set_var("TEST_PRIMAL_SOCKET", "/tmp/test-primal.sock");

        let discovery = SocketDiscovery::new("test");

        // This will return None because the socket doesn't exist
        // but we can verify the logic path
        let result = discovery.discover_via_env_hint("test_primal").await;

        // Clean up
        std::env::remove_var("TEST_PRIMAL_SOCKET");

        // Result is None because socket doesn't exist
        assert!(result.is_none());
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let discovery = SocketDiscovery::new("test");

        // Manually insert into cache using the new constructor
        let socket = DiscoveredSocket::from_unix_path(
            PathBuf::from("/tmp/test.sock"),
            DiscoveryMethod::FamilyTmp,
        )
        .with_primal_name("test")
        .with_capabilities(vec!["test".to_string()]);

        discovery.cache_socket("test:key", &socket).await;

        // Should retrieve from cache
        let cached = discovery.check_cache("test:key").await;
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().discovered_via, DiscoveryMethod::Cached);

        // Clear cache
        discovery.clear_cache().await;
        let cleared = discovery.check_cache("test:key").await;
        assert!(cleared.is_none());
    }

    #[test]
    fn test_calculate_primal_port() {
        let discovery = SocketDiscovery::new("test");

        // Port should be deterministic for same primal name
        let port1 = discovery.calculate_primal_port("beardog");
        let port2 = discovery.calculate_primal_port("beardog");
        assert_eq!(port1, port2);

        // Different primals should (likely) get different ports
        let port_songbird = discovery.calculate_primal_port("songbird");
        let port_beardog = discovery.calculate_primal_port("beardog");
        // They could theoretically collide but very unlikely
        assert!(port_beardog >= 9100 && port_beardog < 9200);
        assert!(port_songbird >= 9100 && port_songbird < 9200);
    }
}
