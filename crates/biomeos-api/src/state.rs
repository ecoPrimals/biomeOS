// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Modern application state with builder pattern
//!
//! Provides a clean, type-safe way to configure the API server.
//!
//! # Architecture
//!
//! The [`AppState`] holds all shared state for the API server, including
//! the discovery service, genome factory, and configuration. It's built using
//! the builder pattern for ergonomic and type-safe initialization.
//!
//! # Examples
//!
//! ## Basic Usage
//!
//! ```rust,no_run
//! use biomeos_api::{AppState, Config};
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! // Create with defaults
//! let state = AppState::builder()
//!     .config_from_env()
//!     .build_with_defaults()?;
//!
//! println!("Standalone mode: {}", state.is_standalone_mode());
//! # Ok(())
//! # }
//! ```
//!
//! ## Custom Configuration
//!
//! ```rust,no_run
//! use biomeos_api::{AppState, Config};
//! use biomeos_core::CompositeDiscovery;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let config = Config {
//!     standalone_mode: true,
//!     ..Default::default()
//! };
//!
//! let discovery = CompositeDiscovery::new();
//!
//! let state = AppState::builder()
//!     .discovery(discovery)
//!     .config(config)
//!     .build()?;
//! # Ok(())
//! # }
//! ```
//!
//! # Design Philosophy
//!
//! - **Type Safety**: Builder pattern prevents invalid configuration
//! - **Encapsulation**: State fields are private, accessed via methods
//! - **Defaults**: Sensible defaults with `build_with_defaults()`
//! - **Environment**: Load configuration from environment variables

use crate::handlers::genome::GenomeState;
use biomeos_core::{CompositeDiscovery, PrimalDiscovery};
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use thiserror::Error;

/// Default bind address for Tier 2 TCP fallback transport (IPC Protocol v3.0).
///
/// Reads from environment variables via `RuntimeConfig`.
/// Falls back to constant only for development.
fn default_bind_addr() -> String {
    use biomeos_types::defaults::RuntimeConfig;
    let config = RuntimeConfig::from_env();
    format!("{}:{}", config.bind_address(), config.mcp_port())
}

/// Application state (shared across handlers)
#[derive(Clone)]
pub struct AppState {
    discovery: Arc<dyn PrimalDiscovery>,
    genome: Arc<GenomeState>,
    config: Config,
    event_broadcaster: Arc<biomeos_graph::GraphEventBroadcaster>,
}

impl AppState {
    /// Create a new builder
    #[must_use]
    pub fn builder() -> AppStateBuilder {
        AppStateBuilder::default()
    }

    /// Get the discovery service
    #[must_use]
    pub fn discovery(&self) -> &dyn PrimalDiscovery {
        &*self.discovery
    }

    /// Get the genome factory state
    #[must_use]
    pub fn genome(&self) -> &GenomeState {
        &self.genome
    }

    /// Get the configuration
    #[must_use]
    pub const fn config(&self) -> &Config {
        &self.config
    }

    /// Check if standalone mode is enabled (graceful degradation)
    #[must_use]
    pub const fn is_standalone_mode(&self) -> bool {
        self.config.standalone_mode
    }

    /// Get the graph event broadcaster for push-based event streaming
    #[must_use]
    pub fn event_broadcaster(&self) -> &biomeos_graph::GraphEventBroadcaster {
        &self.event_broadcaster
    }
}

/// Application configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Enable standalone mode (graceful degradation when primals unavailable)
    ///
    /// When `true`, the API provides standalone fallback responses for demos
    /// and development without requiring full primal infrastructure.
    ///
    /// **Production**: Set to `false` (default) for full primal discovery
    /// **Development**: Set to `true` for standalone operation
    ///
    /// Set via `BIOMEOS_STANDALONE_MODE=true` environment variable.
    pub standalone_mode: bool,

    /// Unix socket path (PRIMARY transport)
    pub socket_path: PathBuf,

    /// Server bind address (DEPRECATED — HTTP bridge only!)
    ///
    /// **DEPRECATED since 0.3.0**: This is only for the temporary HTTP bridge
    /// to support legacy `PetalTongue` clients. Will be removed in v0.5.0.
    pub bind_addr: Option<SocketAddr>,

    /// Enable HTTP bridge (DEPRECATED — for `PetalTongue` transition)
    ///
    /// **DEPRECATED since 0.3.0**: Will be removed in v0.5.0 when `PetalTongue`
    /// migrates to Unix socket JSON-RPC.
    pub enable_http_bridge: bool,

    /// Request timeout
    pub request_timeout: std::time::Duration,

    /// Enable CORS (HTTP bridge only)
    pub enable_cors: bool,
}

impl Default for Config {
    fn default() -> Self {
        // Get runtime directory for Unix socket
        let socket_path = Self::default_socket_path();

        Self {
            standalone_mode: false, // Production default: require primals
            socket_path,
            bind_addr: None,           // HTTP deprecated by default!
            enable_http_bridge: false, // Disabled by default (secure!)
            request_timeout: std::time::Duration::from_secs(30),
            enable_cors: true,
        }
    }
}

impl Config {
    /// Get default Unix socket path
    ///
    /// Uses 5-tier socket resolution per `PRIMAL_DEPLOYMENT_STANDARD.md`:
    /// 1. Environment variable (`BIOMEOS_API_SOCKET`)
    /// 2. `XDG_RUNTIME_DIR/biomeos`/
    /// 3. /run/user/{uid}/biomeos/
    /// 4. /data/local/tmp/biomeos/ (Android)
    /// 5. /tmp/biomeos/ (fallback)
    fn default_socket_path() -> PathBuf {
        use biomeos_core::socket_discovery::SocketDiscovery;

        // Check environment variable first
        if let Ok(path) = std::env::var("BIOMEOS_API_SOCKET") {
            return PathBuf::from(path);
        }

        // Use SocketDiscovery for 5-tier resolution
        let family_id = std::env::var("FAMILY_ID")
            .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
            .unwrap_or_else(|_| biomeos_core::family_discovery::get_family_id());

        let discovery = SocketDiscovery::new(family_id);
        discovery.build_socket_path("biomeos-api")
    }

    /// Load configuration from environment
    pub fn from_env() -> Self {
        // Standalone mode
        let standalone_mode = std::env::var("BIOMEOS_STANDALONE_MODE")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(false);

        // Unix socket path (PRIMARY)
        let socket_path = std::env::var("BIOMEOS_API_SOCKET_PATH")
            .map_or_else(|_| Self::default_socket_path(), PathBuf::from);

        // HTTP bridge (DEPRECATED — for PetalTongue transition, removal in v0.5.0)
        let enable_http_bridge = std::env::var("BIOMEOS_API_HTTP_BRIDGE")
            .ok()
            .and_then(|v| v.parse::<bool>().ok())
            .unwrap_or(false);

        // HTTP bind address (only if bridge enabled)
        let bind_addr = if enable_http_bridge {
            std::env::var("BIOMEOS_API_BIND_ADDR")
                .ok()
                .and_then(|v| v.parse().ok())
                .or_else(|| default_bind_addr().parse().ok())
        } else {
            None
        };

        Self {
            standalone_mode,
            socket_path,
            bind_addr,
            enable_http_bridge,
            ..Default::default()
        }
    }
}

/// Builder for `AppState`
#[derive(Default)]
pub struct AppStateBuilder {
    discovery: Option<Arc<dyn PrimalDiscovery>>,
    genome: Option<Arc<GenomeState>>,
    config: Option<Config>,
}

impl AppStateBuilder {
    /// Set the discovery service
    pub fn discovery(mut self, discovery: impl PrimalDiscovery + 'static) -> Self {
        self.discovery = Some(Arc::new(discovery));
        self
    }

    /// Set the discovery service from an Arc
    pub fn discovery_arc(mut self, discovery: Arc<dyn PrimalDiscovery>) -> Self {
        self.discovery = Some(discovery);
        self
    }

    /// Set the configuration
    #[must_use]
    pub fn config(mut self, config: Config) -> Self {
        self.config = Some(config);
        self
    }

    /// Load config from environment
    #[must_use]
    pub fn config_from_env(mut self) -> Self {
        self.config = Some(Config::from_env());
        self
    }

    /// Build the `AppState`
    pub fn build(self) -> Result<AppState, BuildError> {
        let discovery = self.discovery.ok_or(BuildError::MissingDiscovery)?;

        // Initialize genome factory
        let genome = if let Some(g) = self.genome {
            g
        } else {
            tracing::info!("🧬 Initializing default genome factory");
            Arc::new(GenomeState::new().map_err(BuildError::ConfigError)?)
        };

        let config = self.config.unwrap_or_default();

        Ok(AppState {
            discovery,
            genome,
            config,
            event_broadcaster: Arc::new(biomeos_graph::GraphEventBroadcaster::new(1024)),
        })
    }

    /// Build with default local discovery if none provided
    ///
    /// NOTE: HTTP-based discovery (`create_local_discovery`) has been deprecated.
    /// For live primal discovery, use Unix socket JSON-RPC via `live_discovery` module.
    pub fn build_with_defaults(self) -> Result<AppState, BuildError> {
        let discovery = if let Some(d) = self.discovery {
            d
        } else {
            // EVOLUTION: HTTP-based discovery deprecated
            // Live discovery now uses Unix sockets via handlers/live_discovery.rs
            // For AppState, we provide an empty composite discovery
            // Real discovery happens via JSON-RPC to running primals
            tracing::info!("📡 Creating default composite discovery (Unix socket based)");
            tracing::info!("   Use /api/v1/discovery/live for active primal discovery");

            let composite = CompositeDiscovery::new();
            Arc::new(composite)
        };

        // Initialize genome factory
        let genome = if let Some(g) = self.genome {
            g
        } else {
            tracing::info!("🧬 Initializing default genome factory");
            Arc::new(GenomeState::new().map_err(BuildError::ConfigError)?)
        };

        let config = self.config.unwrap_or_default();

        Ok(AppState {
            discovery,
            genome,
            config,
            event_broadcaster: Arc::new(biomeos_graph::GraphEventBroadcaster::new(1024)),
        })
    }
}

/// Builder errors
#[derive(Debug, Error)]
pub enum BuildError {
    #[error("Discovery service not configured")]
    MissingDiscovery,

    #[error("Discovery configuration error: {0}")]
    DiscoveryError(String),

    #[error("Configuration error: {0}")]
    ConfigError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use biomeos_core::{DiscoveryResult, HealthStatus};
    use biomeos_test_utils::env_helpers::TestEnvGuard;
    use biomeos_types::PrimalId;
    use serial_test::serial;

    struct MockDiscovery;

    #[async_trait::async_trait]
    impl PrimalDiscovery for MockDiscovery {
        async fn discover(
            &self,
            _endpoint: &biomeos_types::Endpoint,
        ) -> DiscoveryResult<biomeos_core::DiscoveredPrimal> {
            Err(biomeos_core::DiscoveryError::NotFound {
                endpoint: "test_mock".to_string(),
            })
        }

        async fn discover_all(&self) -> DiscoveryResult<Vec<biomeos_core::DiscoveredPrimal>> {
            Ok(vec![])
        }

        async fn check_health(&self, _id: &PrimalId) -> DiscoveryResult<HealthStatus> {
            Ok(HealthStatus::Healthy)
        }
    }

    #[test]
    fn test_builder_requires_discovery() {
        let result = AppStateBuilder::default().build();
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_with_discovery() {
        let result = AppStateBuilder::default().discovery(MockDiscovery).build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_config_from_env() {
        let config = Config::from_env();
        // Should not panic and use defaults
        // bind_addr is None by default (HTTP bridge disabled for security)
        assert!(config.bind_addr.is_none() || config.bind_addr.unwrap().port() == 3000);
        // Socket path should always be set
        assert!(config.socket_path.to_string_lossy().contains("biomeos"));
    }

    #[test]
    fn test_config_default() {
        let config = Config::default();
        assert!(!config.standalone_mode);
        assert!(config.socket_path.to_string_lossy().contains("biomeos"));
        assert!(config.bind_addr.is_none());
        assert!(!config.enable_http_bridge);
        assert_eq!(config.request_timeout, std::time::Duration::from_secs(30));
        assert!(config.enable_cors);
    }

    #[test]
    #[serial]
    fn test_config_from_env_standalone_mode() {
        let _guard = TestEnvGuard::set("BIOMEOS_STANDALONE_MODE", "true");
        let config = Config::from_env();
        assert!(config.standalone_mode);
    }

    /// `BIOMEOS_API_HTTP_BRIDGE=true` enables deprecated HTTP bridge and resolves `bind_addr`
    /// (from `BIOMEOS_API_BIND_ADDR` or runtime default bind address).
    #[test]
    #[serial]
    fn test_config_from_env_http_bridge_true_uses_default_bind_when_addr_unset() {
        let _g_bridge = TestEnvGuard::set("BIOMEOS_API_HTTP_BRIDGE", "true");
        let _g_bind = TestEnvGuard::remove("BIOMEOS_API_BIND_ADDR");
        // `default_bind_addr()` uses `RuntimeConfig::bind_address()`; default `::1:3000` does not
        // parse as `SocketAddr` without brackets — use IPv4 so `or_else` resolves to `Some`.
        let _g_runtime_bind = TestEnvGuard::set("BIOMEOS_BIND_ADDRESS", "127.0.0.1");
        let config = Config::from_env();
        assert!(config.enable_http_bridge);
        let addr = config
            .bind_addr
            .expect("bind_addr set when HTTP bridge enabled");
        assert_eq!(addr.port(), biomeos_types::constants::ports::API_DEFAULT);
        assert!(addr.ip().is_loopback());
    }

    #[test]
    #[serial]
    fn test_config_from_env_http_bridge_with_explicit_bind_addr() {
        let _g_bridge = TestEnvGuard::set("BIOMEOS_API_HTTP_BRIDGE", "true");
        let _g_bind = TestEnvGuard::set("BIOMEOS_API_BIND_ADDR", "127.0.0.1:9876");
        let config = Config::from_env();
        assert!(config.enable_http_bridge);
        let addr = config.bind_addr.expect("bind_addr");
        assert_eq!(addr.to_string(), "127.0.0.1:9876");
    }

    #[test]
    #[serial]
    fn test_config_from_env_http_bridge_invalid_bind_addr_falls_back_to_default() {
        let _g_bridge = TestEnvGuard::set("BIOMEOS_API_HTTP_BRIDGE", "true");
        let _g_bind = TestEnvGuard::set("BIOMEOS_API_BIND_ADDR", "not-a-valid-socket-addr");
        let _g_runtime_bind = TestEnvGuard::set("BIOMEOS_BIND_ADDRESS", "127.0.0.1");
        let config = Config::from_env();
        assert!(config.enable_http_bridge);
        let addr = config.bind_addr.expect("fallback bind_addr");
        assert_eq!(addr.port(), biomeos_types::constants::ports::API_DEFAULT);
        assert!(addr.ip().is_loopback());
    }

    /// `default_socket_path` returns `BIOMEOS_API_SOCKET` verbatim when set (via `Config::default`).
    #[test]
    #[serial]
    fn test_config_default_respects_biomeos_api_socket() {
        let _g_sock = TestEnvGuard::set("BIOMEOS_API_SOCKET", "/tmp/biomeos-api-test.sock");
        let config = Config::default();
        assert_eq!(
            config.socket_path,
            std::path::PathBuf::from("/tmp/biomeos-api-test.sock")
        );
    }

    /// Custom `FAMILY_ID` flows into socket discovery (`biomeos-api-{family}.sock`).
    #[test]
    #[serial]
    fn test_config_default_socket_path_uses_family_id_env() {
        let _g_family = TestEnvGuard::set("FAMILY_ID", "test_family_state");
        let _g_sock = TestEnvGuard::remove("BIOMEOS_API_SOCKET");
        let config = Config::default();
        let name = config.socket_path.file_name().expect("file name");
        assert!(
            name.to_string_lossy().contains("test_family_state"),
            "expected family in socket file name, got {:?}",
            config.socket_path
        );
    }

    /// `BIOMEOS_FAMILY_ID` is used when `FAMILY_ID` is unset.
    #[test]
    #[serial]
    fn test_config_default_socket_path_uses_biomeos_family_id_fallback() {
        let _g_fid = TestEnvGuard::remove("FAMILY_ID");
        let _g_bf = TestEnvGuard::set("BIOMEOS_FAMILY_ID", "fallback_family_xyz");
        let _g_sock = TestEnvGuard::remove("BIOMEOS_API_SOCKET");
        let config = Config::default();
        let name = config.socket_path.file_name().expect("file name");
        assert!(
            name.to_string_lossy().contains("fallback_family_xyz"),
            "expected BIOMEOS_FAMILY_ID in socket name, got {:?}",
            config.socket_path
        );
    }

    #[test]
    #[serial]
    fn test_config_from_env_invalid_standalone_bool_defaults_false() {
        let _g = TestEnvGuard::set("BIOMEOS_STANDALONE_MODE", "not-a-bool");
        let config = Config::from_env();
        assert!(!config.standalone_mode);
    }

    #[test]
    #[serial]
    fn test_config_from_env_invalid_http_bridge_bool_defaults_false() {
        let _g = TestEnvGuard::set("BIOMEOS_API_HTTP_BRIDGE", "maybe");
        let config = Config::from_env();
        assert!(!config.enable_http_bridge);
    }

    #[test]
    #[serial]
    fn test_config_from_env_biomeos_api_socket_path_overrides_default_socket() {
        let _g = TestEnvGuard::set("BIOMEOS_API_SOCKET_PATH", "/var/run/biomeos/override.sock");
        let config = Config::from_env();
        assert_eq!(
            config.socket_path,
            std::path::PathBuf::from("/var/run/biomeos/override.sock")
        );
    }

    #[test]
    fn test_build_error_display() {
        let err = BuildError::MissingDiscovery;
        assert!(format!("{err}").contains("Discovery"));

        let err = BuildError::ConfigError("test error".to_string());
        assert!(format!("{err}").contains("test error"));
    }

    #[tokio::test]
    async fn test_app_state_accessors() {
        let state = AppState::builder()
            .discovery(MockDiscovery)
            .build()
            .expect("build state");

        assert!(state.discovery().discover_all().await.is_ok());
        let _ = state.genome();
        assert!(!state.config().standalone_mode);
        assert!(!state.is_standalone_mode());
    }

    #[test]
    fn test_build_with_defaults_no_discovery() {
        let result = AppState::builder().config_from_env().build_with_defaults();
        assert!(result.is_ok());
        let state = result.expect("state");
        assert!(!state.is_standalone_mode() || state.config().standalone_mode);
    }

    #[test]
    fn test_config_clone() {
        let config = Config::default();
        let cloned = config.clone();
        assert_eq!(config.standalone_mode, cloned.standalone_mode);
        assert_eq!(config.socket_path, cloned.socket_path);
    }
}
