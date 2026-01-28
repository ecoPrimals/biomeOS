//! Socket Discovery - Capability-Based Runtime Discovery
//!
//! **Deep Debt Solution**: Replaces hardcoded `/tmp/{primal}.sock` paths with
//! capability-based discovery that respects system conventions and primal self-knowledge.
//!
//! ## Principles
//!
//! 1. **No Hardcoding**: Socket paths discovered at runtime
//! 2. **XDG Compliance**: Respects XDG_RUNTIME_DIR when available
//! 3. **Family-Based Isolation**: Sockets namespaced by family_id
//! 4. **Capability Discovery**: Find primals by what they do, not where they are
//! 5. **Platform Agnostic**: Works across Linux, macOS, and other Unix systems
//!
//! ## Discovery Order
//!
//! 1. Environment variable hint (e.g., `BEARDOG_SOCKET`)
//! 2. XDG_RUNTIME_DIR (e.g., `/run/user/1000/biomeos/beardog-nat0.sock`)
//! 3. Family-scoped /tmp (e.g., `/tmp/beardog-nat0.sock`)
//! 4. Capability registry query via Neural API
//!
//! ## Usage
//!
//! ```ignore
//! use biomeos_core::socket_discovery::{SocketDiscovery, DiscoveryStrategy};
//!
//! let discovery = SocketDiscovery::new("nat0");
//!
//! // Discover by primal name
//! let socket = discovery.discover_primal("beardog").await?;
//!
//! // Discover by capability
//! let socket = discovery.discover_capability("crypto").await?;
//! ```

use std::collections::HashMap;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// Socket discovery result
#[derive(Debug, Clone)]
pub struct DiscoveredSocket {
    /// Path to the socket
    pub path: PathBuf,

    /// How it was discovered
    pub discovered_via: DiscoveryMethod,

    /// Primal name (if known)
    pub primal_name: Option<String>,

    /// Capabilities provided (if known)
    pub capabilities: Vec<String>,
}

/// How a socket was discovered
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DiscoveryMethod {
    /// Via environment variable hint
    EnvironmentHint(String),

    /// Via XDG runtime directory
    XdgRuntime,

    /// Via family-scoped /tmp
    FamilyTmp,

    /// Via capability registry query
    CapabilityRegistry,

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

    /// Use family-scoped /tmp
    pub use_family_tmp: bool,

    /// Query capability registry
    pub query_registry: bool,

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
            use_family_tmp: true,
            query_registry: true,
            scan_sockets: false, // Expensive, disabled by default
            enable_cache: true,
            cache_ttl_secs: 60,
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

        // 3. Try known capability→primal mappings (fallback)
        let primal_name = self.capability_to_primal(capability);
        if let Some(name) = primal_name {
            return self.discover_primal(&name).await;
        }

        warn!("Socket not found for capability: {}", capability);
        None
    }

    /// Get socket path for a primal (convenience method)
    ///
    /// Returns the path directly, or None if not found.
    pub async fn get_socket_path(&self, primal_name: &str) -> Option<PathBuf> {
        self.discover_primal(primal_name)
            .await
            .map(|s| s.path)
    }

    /// Build deterministic socket path for a primal
    ///
    /// This is used when we KNOW the socket should exist at a specific location.
    /// Used by primals to register their own sockets.
    pub fn build_socket_path(&self, primal_name: &str) -> PathBuf {
        // Prefer XDG runtime dir if available
        if let Some(runtime_dir) = self.get_xdg_runtime_dir() {
            let biomeos_dir = runtime_dir.join("biomeos");
            std::fs::create_dir_all(&biomeos_dir).ok();
            return biomeos_dir.join(format!("{}-{}.sock", primal_name, self.family_id));
        }

        // Fallback to /tmp with family namespace
        PathBuf::from(format!("/tmp/{}-{}.sock", primal_name, self.family_id))
    }

    // ========================================================================
    // DISCOVERY IMPLEMENTATIONS
    // ========================================================================

    /// Discover via environment variable hint
    async fn discover_via_env_hint(&self, primal_name: &str) -> Option<DiscoveredSocket> {
        // Try various environment variable patterns
        let env_patterns = vec![
            format!("{}_SOCKET", primal_name.to_uppercase().replace("-", "_")),
            format!("{}_SOCKET_PATH", primal_name.to_uppercase().replace("-", "_")),
            format!("BIOMEOS_{}_SOCKET", primal_name.to_uppercase().replace("-", "_")),
        ];

        for env_var in env_patterns {
            if let Ok(path_str) = env::var(&env_var) {
                let path = PathBuf::from(&path_str);
                if path.exists() {
                    debug!("Discovered {} via env hint: {}", primal_name, env_var);
                    return Some(DiscoveredSocket {
                        path,
                        discovered_via: DiscoveryMethod::EnvironmentHint(env_var),
                        primal_name: Some(primal_name.to_string()),
                        capabilities: Vec::new(),
                    });
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
            return Some(DiscoveredSocket {
                path: socket_path,
                discovered_via: DiscoveryMethod::XdgRuntime,
                primal_name: Some(primal_name.to_string()),
                capabilities: Vec::new(),
            });
        }

        // Try without family namespace (legacy)
        let legacy_path = biomeos_dir.join(format!("{}.sock", primal_name));
        if legacy_path.exists() {
            debug!("Discovered {} via XDG runtime (legacy)", primal_name);
            return Some(DiscoveredSocket {
                path: legacy_path,
                discovered_via: DiscoveryMethod::XdgRuntime,
                primal_name: Some(primal_name.to_string()),
                capabilities: Vec::new(),
            });
        }

        None
    }

    /// Discover via family-scoped /tmp
    async fn discover_via_family_tmp(&self, primal_name: &str) -> Option<DiscoveredSocket> {
        // Family-namespaced path (preferred)
        let socket_path = PathBuf::from(format!("/tmp/{}-{}.sock", primal_name, self.family_id));
        if socket_path.exists() {
            debug!("Discovered {} via family /tmp", primal_name);
            return Some(DiscoveredSocket {
                path: socket_path,
                discovered_via: DiscoveryMethod::FamilyTmp,
                primal_name: Some(primal_name.to_string()),
                capabilities: Vec::new(),
            });
        }

        // Legacy path without family namespace
        let legacy_path = PathBuf::from(format!("/tmp/{}.sock", primal_name));
        if legacy_path.exists() {
            debug!("Discovered {} via /tmp (legacy)", primal_name);
            return Some(DiscoveredSocket {
                path: legacy_path,
                discovered_via: DiscoveryMethod::FamilyTmp,
                primal_name: Some(primal_name.to_string()),
                capabilities: Vec::new(),
            });
        }

        None
    }

    /// Query capability registry by primal name
    async fn discover_via_registry_by_name(&self, primal_name: &str) -> Option<DiscoveredSocket> {
        let neural_api = self.get_neural_api_socket()?;

        match self.query_registry("primal.discover", &serde_json::json!({
            "name": primal_name
        }), &neural_api).await {
            Ok(result) => {
                if let Some(socket_path) = result.get("socket_path").and_then(|s| s.as_str()) {
                    let capabilities = result.get("capabilities")
                        .and_then(|c| c.as_array())
                        .map(|arr| arr.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                        .unwrap_or_default();

                    return Some(DiscoveredSocket {
                        path: PathBuf::from(socket_path),
                        discovered_via: DiscoveryMethod::CapabilityRegistry,
                        primal_name: Some(primal_name.to_string()),
                        capabilities,
                    });
                }
            }
            Err(e) => {
                debug!("Registry query failed for {}: {}", primal_name, e);
            }
        }

        None
    }

    /// Query capability registry by capability
    async fn discover_via_registry_by_capability(&self, capability: &str) -> Option<DiscoveredSocket> {
        let neural_api = self.get_neural_api_socket()?;

        match self.query_registry("capability.discover", &serde_json::json!({
            "capability": capability
        }), &neural_api).await {
            Ok(result) => {
                if let Some(socket_path) = result.get("primary_socket").and_then(|s| s.as_str()) {
                    let primal_name = result.get("provider")
                        .and_then(|p| p.as_str())
                        .map(String::from);

                    return Some(DiscoveredSocket {
                        path: PathBuf::from(socket_path),
                        discovered_via: DiscoveryMethod::CapabilityRegistry,
                        primal_name,
                        capabilities: vec![capability.to_string()],
                    });
                }
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

        let stream = timeout(Duration::from_secs(5), UnixStream::connect(neural_api_socket))
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
        writer.write_all(request_str.as_bytes()).await.map_err(|e| e.to_string())?;
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

        response.get("result")
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

        for path in standard_locations {
            if path.exists() {
                return Some(path);
            }
        }

        None
    }

    /// Map capability to known primal name (fallback)
    fn capability_to_primal(&self, capability: &str) -> Option<String> {
        // Known mappings (fallback when registry unavailable)
        let mapping = match capability {
            "crypto" | "security" | "tls" | "genetic" => Some("beardog"),
            "http" | "discovery" | "network" | "mesh" => Some("songbird"),
            "ai" | "inference" | "learning" => Some("squirrel"),
            "compute" | "workload" | "orchestration" => Some("toadstool"),
            "storage" | "data" | "persistence" => Some("nestgate"),
            _ => None,
        };

        mapping.map(String::from)
    }

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
// TESTS
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_build_socket_path() {
        let discovery = SocketDiscovery::new("test-family");
        let path = discovery.build_socket_path("beardog");

        // Should be /tmp/beardog-test-family.sock (unless XDG_RUNTIME_DIR is set)
        assert!(path.to_string_lossy().contains("beardog"));
        assert!(path.to_string_lossy().contains("test-family"));
    }

    #[test]
    fn test_capability_to_primal_mapping() {
        let discovery = SocketDiscovery::new("nat0");

        assert_eq!(discovery.capability_to_primal("crypto"), Some("beardog".to_string()));
        assert_eq!(discovery.capability_to_primal("http"), Some("songbird".to_string()));
        assert_eq!(discovery.capability_to_primal("ai"), Some("squirrel".to_string()));
        assert_eq!(discovery.capability_to_primal("compute"), Some("toadstool".to_string()));
        assert_eq!(discovery.capability_to_primal("storage"), Some("nestgate".to_string()));
        assert_eq!(discovery.capability_to_primal("unknown"), None);
    }

    #[test]
    fn test_discovery_strategy_defaults() {
        let strategy = DiscoveryStrategy::default();

        assert!(strategy.check_env_hints);
        assert!(strategy.use_xdg_runtime);
        assert!(strategy.use_family_tmp);
        assert!(strategy.query_registry);
        assert!(!strategy.scan_sockets); // Expensive, disabled by default
        assert!(strategy.enable_cache);
    }

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

        // Manually insert into cache
        let socket = DiscoveredSocket {
            path: PathBuf::from("/tmp/test.sock"),
            discovered_via: DiscoveryMethod::FamilyTmp,
            primal_name: Some("test".to_string()),
            capabilities: vec!["test".to_string()],
        };

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
}

