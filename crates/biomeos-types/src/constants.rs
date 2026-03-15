// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Unified Constants Module
//!
//! This module centralizes all constants that were previously scattered across
//! the biomeOS ecosystem, providing a single source of truth for:
//! - Default endpoints and URLs
//! - Timeout durations
//! - Size limits and thresholds
//! - Version information
//! - Network configurations

use std::time::Duration;

/// Version and build information
pub mod version {
    /// Build timestamp for debugging
    pub const BUILD_TIMESTAMP: &str = "2025-01-XX"; // Will be updated by build system

    /// Unified version constant - single source of truth
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");

    /// Types version (alias for VERSION)
    pub const TYPES_VERSION: &str = VERSION;

    /// Build information for the entire BiomeOS ecosystem
    pub use crate::BUILD_INFO;

    /// API version for biomeOS ecosystem
    pub const API_VERSION: &str = "biomeOS/v1";

    /// Protocol version for MCP communication
    pub const MCP_PROTOCOL_VERSION: &str = "1.0";
}

/// Endpoint configuration
///
/// DESIGN PRINCIPLE: Primals do NOT have hardcoded knowledge of other primals.
/// - Each primal only knows its own identity and capabilities
/// - Primal endpoints are discovered at runtime via Songbird discovery
/// - These constants are FALLBACK values for local development only
///
/// Production systems MUST use capability-based discovery:
/// 1. Primal starts and advertises its capabilities
/// 2. Songbird mDNS/registry discovers all primals
/// 3. Primals query for capabilities they need (e.g., "security", "storage")
/// 4. No primal contains knowledge of specific other primal endpoints
pub mod endpoints {
    use super::env_vars;
    use std::env;

    /// Default localhost address (for binding) - fallback only
    pub const DEFAULT_LOCALHOST: &str = "127.0.0.1";

    /// Production bind address (for accepting connections) - fallback only
    pub const PRODUCTION_BIND_ADDRESS: &str = "0.0.0.0";

    /// Get bind address from environment or fallback to default
    ///
    /// Checks `BIND_ADDRESS` environment variable first.
    /// Falls back to `DEFAULT_LOCALHOST` for development.
    pub fn bind_address() -> String {
        env::var(env_vars::BIND_ADDRESS).unwrap_or_else(|_| DEFAULT_LOCALHOST.to_string())
    }

    /// Get production bind address from environment or fallback
    ///
    /// Checks `BIND_ADDRESS` environment variable first.
    /// Falls back to `PRODUCTION_BIND_ADDRESS` for production.
    pub fn production_bind_address() -> String {
        env::var(env_vars::BIND_ADDRESS).unwrap_or_else(|_| PRODUCTION_BIND_ADDRESS.to_string())
    }

    // REMOVED: FALLBACK_*_ENDPOINT constants
    //
    // These hardcoded endpoints violated BiomeOS's architecture principle:
    // "Primals do NOT have hardcoded knowledge of other primals"
    //
    // Instead, use:
    // 1. Environment variables (e.g., TOADSTOOL_ENDPOINT, SONGBIRD_ENDPOINT)
    // 2. Capability-based discovery via Songbird
    // 3. mDNS automatic discovery
    //
    // For local development, set environment variables:
    //   export SONGBIRD_ENDPOINT="http://localhost:3000"
    //   export TOADSTOOL_ENDPOINT="http://localhost:8080"
    //   export NESTGATE_ENDPOINT="http://localhost:8002"
    //   export BEARDOG_ENDPOINT="http://localhost:9000"
    //   export SQUIRREL_ENDPOINT="http://localhost:8001"

    // API PATH CONSTANTS - These ARE appropriate as constants
    // since they define the primal's own API contract

    /// Health check endpoint path (each primal exposes this)
    pub const HEALTH_ENDPOINT: &str = "/health";

    /// Metrics endpoint path (each primal exposes this)
    pub const METRICS_ENDPOINT: &str = "/metrics";

    /// Admin endpoint path (each primal exposes this)
    pub const ADMIN_ENDPOINT: &str = "/admin";

    /// WebSocket endpoint path (each primal exposes this)
    pub const WS_ENDPOINT: &str = "/ws";

    /// Service discovery endpoint path (Songbird exposes this)
    pub const DISCOVERY_ENDPOINT: &str = "/discovery";

    /// Service registration endpoint path (Songbird exposes this)
    pub const REGISTRATION_ENDPOINT: &str = "/register";

    /// Capability query endpoint path (Songbird exposes this)
    pub const CAPABILITY_QUERY_ENDPOINT: &str = "/capabilities";
}

/// Timeout and duration constants
pub mod timeouts {
    use super::Duration;

    /// Default connection timeout
    pub const DEFAULT_CONNECTION_TIMEOUT: Duration = Duration::from_secs(30);

    /// Default request timeout
    pub const DEFAULT_REQUEST_TIMEOUT: Duration = Duration::from_secs(30);

    /// Default operation timeout
    pub const DEFAULT_OPERATION_TIMEOUT: Duration = Duration::from_secs(60);

    /// Default validation timeout
    pub const DEFAULT_VALIDATION_TIMEOUT: Duration = Duration::from_secs(30);

    /// Default health check timeout
    pub const DEFAULT_HEALTH_CHECK_TIMEOUT: Duration = Duration::from_secs(10);

    /// Default health check interval
    pub const DEFAULT_HEALTH_CHECK_INTERVAL: Duration = Duration::from_secs(30);

    /// Default metrics collection interval
    pub const DEFAULT_METRICS_COLLECTION_INTERVAL: Duration = Duration::from_secs(60);

    /// Default session timeout
    pub const DEFAULT_SESSION_TIMEOUT: Duration = Duration::from_secs(3600); // 1 hour

    /// Default cache TTL
    pub const DEFAULT_CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes

    /// Default service startup timeout
    pub const DEFAULT_SERVICE_START_TIMEOUT: Duration = Duration::from_secs(60);

    /// Default service shutdown timeout
    pub const DEFAULT_SERVICE_STOP_TIMEOUT: Duration = Duration::from_secs(30);

    /// Default retry delay
    pub const DEFAULT_RETRY_DELAY: Duration = Duration::from_millis(1000);

    /// Default heartbeat interval
    pub const DEFAULT_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

    /// Default ping interval
    pub const DEFAULT_PING_INTERVAL: Duration = Duration::from_secs(30);

    /// Default pong timeout
    pub const DEFAULT_PONG_TIMEOUT: Duration = Duration::from_secs(10);
}

/// Resource limits and thresholds
pub mod limits {
    /// Default maximum connections
    pub const DEFAULT_MAX_CONNECTIONS: u32 = 1000;

    /// Default buffer size
    pub const DEFAULT_BUFFER_SIZE: usize = 8192;

    /// Default maximum message size
    pub const DEFAULT_MAX_MESSAGE_SIZE: usize = 1024 * 1024; // 1MB

    /// Default rate limit per minute
    pub const DEFAULT_RATE_LIMIT_PER_MINUTE: u64 = 100;

    /// Default maximum services in service mesh
    pub const DEFAULT_SERVICE_MESH_MAX_SERVICES: u32 = 100;

    /// Default memory limit (MB)
    pub const DEFAULT_MEMORY_LIMIT_MB: u64 = 1024;

    /// Default CPU limit (millicores)
    pub const DEFAULT_CPU_LIMIT_MILLICORES: u64 = 1000;

    /// Default disk space limit (GB)
    pub const DEFAULT_DISK_LIMIT_GB: u64 = 10;
}

/// Network configuration constants
///
/// **DESIGN PRINCIPLE**: These are FALLBACK defaults only.
/// Production systems MUST use environment variables or capability discovery.
pub mod network {
    use super::env_vars;
    use std::env;

    /// Default HTTP port (fallback only)
    pub const DEFAULT_HTTP_PORT: u16 = 8080;

    /// Default HTTPS port (fallback only)
    pub const DEFAULT_HTTPS_PORT: u16 = 8443;

    /// Default WebSocket port (fallback only)
    pub const DEFAULT_WS_PORT: u16 = 8081;

    /// Default MCP port (fallback only)
    pub const DEFAULT_MCP_PORT: u16 = 3000;

    /// Default discovery port (fallback only)
    pub const DEFAULT_DISCOVERY_PORT: u16 = 8001;

    /// Default BearDog (security) port (fallback only)
    pub const DEFAULT_BEARDOG_PORT: u16 = 9000;

    /// Default Songbird (universal adapter) port (fallback only)
    pub const DEFAULT_SONGBIRD_PORT: u16 = 3000;

    /// Default broadcast discovery port (fallback only)
    pub const DEFAULT_BROADCAST_DISCOVERY_PORT: u16 = 9199;

    /// Default dev server port (common Flask/alternative HTTP fallback)
    pub const DEFAULT_DEV_PORT: u16 = 5000;

    /// Get HTTP port from environment or fallback to default
    ///
    /// Checks `HTTP_PORT` environment variable first.
    pub fn http_port() -> u16 {
        env::var(env_vars::HTTP_PORT)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_HTTP_PORT)
    }

    /// Get HTTPS port from environment or fallback to default
    ///
    /// Checks `HTTPS_PORT` environment variable first.
    pub fn https_port() -> u16 {
        env::var(env_vars::HTTPS_PORT)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_HTTPS_PORT)
    }

    /// Get WebSocket port from environment or fallback to default
    ///
    /// Checks `WEBSOCKET_PORT` environment variable first.
    pub fn websocket_port() -> u16 {
        env::var(env_vars::WEBSOCKET_PORT)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_WS_PORT)
    }

    /// Get MCP port from environment or fallback to default
    ///
    /// Checks `MCP_PORT` or `MCP_WEBSOCKET_PORT` environment variable first.
    pub fn mcp_port() -> u16 {
        env::var(env_vars::MCP_WEBSOCKET_PORT)
            .or_else(|_| env::var("MCP_PORT"))
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_MCP_PORT)
    }

    /// Get discovery port from environment or fallback to default
    ///
    /// Checks `DISCOVERY_PORT` environment variable first.
    pub fn discovery_port() -> u16 {
        env::var("DISCOVERY_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_DISCOVERY_PORT)
    }

    /// Get BearDog port from environment or fallback to default
    ///
    /// Checks `BEARDOG_PORT` environment variable first.
    pub fn beardog_port() -> u16 {
        env::var(env_vars::BEARDOG_PORT)
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_BEARDOG_PORT)
    }

    /// Get Songbird port from environment or fallback to default
    ///
    /// Checks `SONGBIRD_PORT` or `MCP_PORT` environment variable first.
    pub fn songbird_port() -> u16 {
        env::var(env_vars::SONGBIRD_PORT)
            .or_else(|_| env::var(env_vars::MCP_WEBSOCKET_PORT))
            .or_else(|_| env::var("MCP_PORT"))
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(DEFAULT_SONGBIRD_PORT)
    }

    /// Link local address range
    pub const LINK_LOCAL_RANGE: &str = "169.254.0.0/16";

    /// Multicast address range
    pub const MULTICAST_RANGE: &str = "224.0.0.0/4";

    /// Private Class A network
    pub const PRIVATE_CLASS_A: &str = "10.0.0.0/8";

    /// Private Class B network
    pub const PRIVATE_CLASS_B: &str = "172.16.0.0/12";

    /// Private Class C network
    pub const PRIVATE_CLASS_C: &str = "192.168.0.0/16";

    /// Default MCP subprotocol
    pub const DEFAULT_MCP_SUBPROTOCOL: &str = "mcp";

    /// Default user agent
    pub const DEFAULT_USER_AGENT: &str = "biomeOS/1.0";

    /// Default content type
    pub const DEFAULT_CONTENT_TYPE: &str = "application/json";
}

/// Security and authentication constants
pub mod security {
    use super::Duration;

    /// Default authentication timeout
    pub const DEFAULT_AUTH_TIMEOUT: Duration = Duration::from_secs(300); // 5 minutes

    /// Default token expiry
    pub const DEFAULT_TOKEN_EXPIRY: Duration = Duration::from_secs(3600); // 1 hour

    /// Default key rotation interval
    pub const DEFAULT_KEY_ROTATION_INTERVAL: Duration = Duration::from_secs(86400); // 24 hours

    /// Default lockout duration
    pub const DEFAULT_LOCKOUT_DURATION: Duration = Duration::from_secs(1800); // 30 minutes

    /// Default key cache TTL
    pub const DEFAULT_KEY_CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes

    /// Default audit retention period
    pub const DEFAULT_AUDIT_RETENTION: Duration = Duration::from_secs(2592000); // 30 days

    /// Default compliance check interval
    pub const DEFAULT_COMPLIANCE_CHECK_INTERVAL: Duration = Duration::from_secs(86400); // 24 hours

    /// Default threat scan interval
    pub const DEFAULT_THREAT_SCAN_INTERVAL: Duration = Duration::from_secs(3600);
    // 1 hour
}

/// File and data constants
pub mod files {
    /// Default charset for encoding
    pub const DEFAULT_CHARSET: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";

    /// Base64 alphabet
    pub const BASE64_ALPHABET: &[u8] =
        b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

    /// Size units for formatting
    pub const SIZE_UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];

    /// Default configuration file name
    pub const DEFAULT_CONFIG_FILE: &str = "biome.yaml";

    /// Default rules directory
    pub const DEFAULT_RULES_DIR: &str = ".rules";

    /// Get default plugins directory for a specific primal
    ///
    /// # Example
    /// ```
    /// # use biomeos_types::constants::files;
    /// let dir = files::default_plugin_dir("squirrel");
    /// assert_eq!(dir, ".squirrel/plugins");
    /// ```
    pub fn default_plugin_dir(primal_name: &str) -> String {
        format!(".{primal_name}/plugins")
    }

    /// Get default plugins directory for current primal (from environment)
    ///
    /// Reads the PRIMAL_NAME environment variable. Falls back to "unknown" if not set.
    ///
    /// # Example
    /// ```
    /// # use biomeos_types::constants::files;
    /// // Assumes PRIMAL_NAME is set in environment
    /// let dir = files::current_primal_plugin_dir();
    /// ```
    pub fn current_primal_plugin_dir() -> String {
        let primal_name = std::env::var("PRIMAL_NAME").unwrap_or_else(|_| "unknown".to_string());
        default_plugin_dir(&primal_name)
    }

    /// Default history file
    pub const DEFAULT_HISTORY_FILE: &str = "command_history.json";

    /// Default journal file
    pub const DEFAULT_JOURNAL_FILE: &str = "command_journal.json";
}

/// Event system constants
pub mod events {
    /// Plugin initialized event
    pub const PLUGIN_INITIALIZED: &str = "plugin.initialized";

    /// Plugin started event
    pub const PLUGIN_STARTED: &str = "plugin.started";

    /// Plugin stopped event
    pub const PLUGIN_STOPPED: &str = "plugin.stopped";

    /// Plugin error event
    pub const PLUGIN_ERROR: &str = "plugin.error";

    /// Command executed event
    pub const COMMAND_EXECUTED: &str = "command.executed";

    /// Command failed event
    pub const COMMAND_FAILED: &str = "command.failed";

    /// System ready event
    pub const SYSTEM_READY: &str = "system.ready";

    /// System shutdown event
    pub const SYSTEM_SHUTDOWN: &str = "system.shutdown";

    /// Custom event prefix
    pub const CUSTOM_EVENT: &str = "custom.event";
}

/// Capability constants for discovery
///
/// **DESIGN PRINCIPLE**: Query by capability, not by primal name.
///
/// These constants are used for capability-based discovery through the
/// universal adapter (Songbird). No primal should hardcode knowledge of
/// other primals by name.
///
/// # Example
/// ```
/// use biomeos_types::constants::capabilities;
///
/// // Query by capability, not by name
/// let compute_capability = capabilities::COMPUTE;
/// let storage_capability = capabilities::STORAGE;
/// let security_capability = capabilities::SECURITY;
///
/// assert_eq!(compute_capability, "compute");
/// assert_eq!(storage_capability, "storage");
/// assert_eq!(security_capability, "security");
/// ```
pub mod capabilities {
    /// Compute and execution capability (e.g., ToadStool)
    pub const COMPUTE: &str = "compute";

    /// Storage and persistence capability (e.g., NestGate)
    pub const STORAGE: &str = "storage";

    /// Security and cryptography capability (e.g., BearDog)
    pub const SECURITY: &str = "security";

    /// AI and intelligence capability (e.g., Squirrel)
    pub const AI: &str = "ai";

    /// Discovery and service mesh capability (e.g., Songbird)
    pub const DISCOVERY: &str = "discovery";

    /// Orchestration capability (e.g., BiomeOS, Songbird)
    pub const ORCHESTRATION: &str = "orchestration";

    /// UI and visualization capability (e.g., PetalTongue)
    pub const VISUALIZATION: &str = "visualization";

    /// Networking capability
    pub const NETWORKING: &str = "networking";

    /// Monitoring and observability capability
    pub const MONITORING: &str = "monitoring";

    /// Data processing capability
    pub const DATA_PROCESSING: &str = "data-processing";

    // =====================================================================
    // Spring capability domains — registered by springs at runtime.
    // =====================================================================

    /// Ecology and agriculture capability (airSpring)
    pub const ECOLOGY: &str = "ecology";

    /// Life science, analytical chemistry, microbial ecology (wetSpring)
    pub const SCIENCE: &str = "science";

    /// Medical, PK/PD, microbiome, biosignal (healthSpring)
    pub const MEDICAL: &str = "medical";

    /// Game science, HCI, procedural content (ludoSpring)
    pub const GAME: &str = "game";

    /// Computational physics, nuclear EOS, GPU compute (hotSpring)
    pub const PHYSICS: &str = "physics";

    /// Measurement, signal processing, inverse problems (groundSpring)
    pub const MEASUREMENT: &str = "measurement";

    /// Machine learning, surrogates, isomorphic patterns (neuralSpring)
    pub const LEARNING: &str = "learning";
}

/// Environment variable names
pub mod env_vars {
    /// Bind address environment variable
    pub const BIND_ADDRESS: &str = "BIND_ADDRESS";

    /// HTTP port environment variable
    pub const HTTP_PORT: &str = "HTTP_PORT";

    /// WebSocket port environment variable
    pub const WEBSOCKET_PORT: &str = "WEBSOCKET_PORT";

    /// HTTPS port environment variable
    pub const HTTPS_PORT: &str = "HTTPS_PORT";

    /// MCP WebSocket port environment variable
    pub const MCP_WEBSOCKET_PORT: &str = "MCP_WEBSOCKET_PORT";

    /// BearDog endpoint URL environment variable
    pub const BEARDOG_ENDPOINT: &str = "BEARDOG_ENDPOINT";

    /// BearDog port environment variable
    pub const BEARDOG_PORT: &str = "BEARDOG_PORT";

    /// Songbird endpoint URL environment variable
    pub const SONGBIRD_ENDPOINT: &str = "SONGBIRD_ENDPOINT";

    /// Songbird port environment variable
    pub const SONGBIRD_PORT: &str = "SONGBIRD_PORT";

    /// Connection timeout environment variable
    pub const CONNECTION_TIMEOUT: &str = "CONNECTION_TIMEOUT";

    /// Request timeout environment variable
    pub const REQUEST_TIMEOUT: &str = "REQUEST_TIMEOUT";

    /// Operation timeout environment variable
    pub const OPERATION_TIMEOUT: &str = "OPERATION_TIMEOUT";

    /// Database timeout environment variable
    pub const DATABASE_TIMEOUT: &str = "DATABASE_TIMEOUT";

    /// Heartbeat interval environment variable
    pub const HEARTBEAT_INTERVAL: &str = "HEARTBEAT_INTERVAL";

    /// Maximum connections environment variable
    pub const MAX_CONNECTIONS: &str = "MAX_CONNECTIONS";

    /// Buffer size environment variable
    pub const BUFFER_SIZE: &str = "BUFFER_SIZE";

    /// Service mesh maximum services environment variable
    pub const SERVICE_MESH_MAX_SERVICES: &str = "SERVICE_MESH_MAX_SERVICES";

    /// Maximum message size environment variable
    pub const MAX_MESSAGE_SIZE: &str = "MAX_MESSAGE_SIZE";
}

/// Re-export commonly used constants at module level
pub use endpoints::DEFAULT_LOCALHOST;
pub use limits::{DEFAULT_BUFFER_SIZE, DEFAULT_MAX_CONNECTIONS};
pub use network::{DEFAULT_HTTPS_PORT, DEFAULT_HTTP_PORT};
pub use timeouts::{DEFAULT_CONNECTION_TIMEOUT, DEFAULT_REQUEST_TIMEOUT};
pub use version::*;

// Re-export capability constants for easy access
pub use capabilities::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_version_constants() {
        // Version constants are compile-time validated
        // VERSION is a const &str, checked at compile time
        assert_eq!(version::TYPES_VERSION, version::VERSION);
        assert_eq!(version::API_VERSION, "biomeOS/v1");
        assert_eq!(version::MCP_PROTOCOL_VERSION, "1.0");
    }

    #[test]
    fn test_endpoint_constants() {
        assert_eq!(endpoints::DEFAULT_LOCALHOST, "127.0.0.1");
        assert_eq!(endpoints::PRODUCTION_BIND_ADDRESS, "0.0.0.0");
        assert_eq!(endpoints::HEALTH_ENDPOINT, "/health");
        assert_eq!(endpoints::METRICS_ENDPOINT, "/metrics");
        assert_eq!(endpoints::DISCOVERY_ENDPOINT, "/discovery");
    }

    #[test]
    fn test_timeout_constants() {
        assert_eq!(timeouts::DEFAULT_CONNECTION_TIMEOUT.as_secs(), 30);
        assert_eq!(timeouts::DEFAULT_REQUEST_TIMEOUT.as_secs(), 30);
        assert_eq!(timeouts::DEFAULT_OPERATION_TIMEOUT.as_secs(), 60);
        assert_eq!(timeouts::DEFAULT_SESSION_TIMEOUT.as_secs(), 3600);
        assert_eq!(timeouts::DEFAULT_RETRY_DELAY.as_millis(), 1000);
    }

    #[test]
    fn test_limit_constants() {
        assert_eq!(limits::DEFAULT_MAX_CONNECTIONS, 1000);
        assert_eq!(limits::DEFAULT_BUFFER_SIZE, 8192);
        assert_eq!(limits::DEFAULT_MAX_MESSAGE_SIZE, 1024 * 1024);
        assert_eq!(limits::DEFAULT_RATE_LIMIT_PER_MINUTE, 100);
    }

    #[test]
    fn test_network_constants() {
        assert_eq!(network::DEFAULT_HTTP_PORT, 8080);
        assert_eq!(network::DEFAULT_HTTPS_PORT, 8443);
        assert_eq!(network::DEFAULT_WS_PORT, 8081);
        assert_eq!(network::DEFAULT_MCP_PORT, 3000);
        assert_eq!(network::DEFAULT_BEARDOG_PORT, 9000);
        assert_eq!(network::DEFAULT_SONGBIRD_PORT, 3000);
        assert_eq!(network::DEFAULT_BROADCAST_DISCOVERY_PORT, 9199);
        assert_eq!(network::DEFAULT_DEV_PORT, 5000);
        assert_eq!(network::DEFAULT_USER_AGENT, "biomeOS/1.0");
        assert_eq!(network::DEFAULT_CONTENT_TYPE, "application/json");
    }

    #[test]
    fn test_security_constants() {
        assert_eq!(security::DEFAULT_AUTH_TIMEOUT.as_secs(), 300);
        assert_eq!(security::DEFAULT_TOKEN_EXPIRY.as_secs(), 3600);
        assert_eq!(security::DEFAULT_LOCKOUT_DURATION.as_secs(), 1800);
    }

    #[test]
    fn test_capability_constants() {
        assert_eq!(capabilities::COMPUTE, "compute");
        assert_eq!(capabilities::STORAGE, "storage");
        assert_eq!(capabilities::SECURITY, "security");
        assert_eq!(capabilities::AI, "ai");
        assert_eq!(capabilities::DISCOVERY, "discovery");
        assert_eq!(capabilities::ORCHESTRATION, "orchestration");
    }

    #[test]
    fn test_files_plugin_dir() {
        let dir = files::default_plugin_dir("squirrel");
        assert_eq!(dir, ".squirrel/plugins");

        let dir2 = files::default_plugin_dir("beardog");
        assert_eq!(dir2, ".beardog/plugins");
    }

    #[test]
    fn test_files_constants() {
        assert_eq!(files::DEFAULT_CONFIG_FILE, "biome.yaml");
        assert_eq!(files::DEFAULT_RULES_DIR, ".rules");
        assert_eq!(files::DEFAULT_HISTORY_FILE, "command_history.json");
    }

    #[test]
    fn test_event_constants() {
        assert_eq!(events::PLUGIN_INITIALIZED, "plugin.initialized");
        assert_eq!(events::SYSTEM_READY, "system.ready");
        assert_eq!(events::COMMAND_EXECUTED, "command.executed");
    }

    #[test]
    fn test_env_var_constants() {
        assert_eq!(env_vars::BIND_ADDRESS, "BIND_ADDRESS");
        assert_eq!(env_vars::HTTP_PORT, "HTTP_PORT");
        assert_eq!(env_vars::MAX_CONNECTIONS, "MAX_CONNECTIONS");
    }

    #[test]
    fn test_network_accessors() {
        let _ = network::http_port();
        let _ = network::https_port();
        let _ = network::websocket_port();
        let _ = network::mcp_port();
        let _ = network::discovery_port();
        assert_eq!(network::DEFAULT_HTTP_PORT, 8080);
        assert_eq!(network::DEFAULT_DISCOVERY_PORT, 8001);
        assert_eq!(network::LINK_LOCAL_RANGE, "169.254.0.0/16");
        assert_eq!(network::PRIVATE_CLASS_A, "10.0.0.0/8");
        assert_eq!(network::PRIVATE_CLASS_B, "172.16.0.0/12");
        assert_eq!(network::PRIVATE_CLASS_C, "192.168.0.0/16");
        assert_eq!(network::DEFAULT_MCP_SUBPROTOCOL, "mcp");
    }

    #[test]
    fn test_endpoints_bind_address() {
        let addr = endpoints::bind_address();
        assert!(!addr.is_empty());
        assert!(addr.contains('.') || addr.contains(':'));
    }

    #[test]
    fn test_endpoints_production_bind_address() {
        let addr = endpoints::production_bind_address();
        assert!(!addr.is_empty());
    }

    #[test]
    fn test_files_current_primal_plugin_dir() {
        let dir = files::current_primal_plugin_dir();
        assert!(dir.contains("plugins"));
    }

    #[test]
    fn test_limits_constants() {
        assert_eq!(limits::DEFAULT_SERVICE_MESH_MAX_SERVICES, 100);
        assert_eq!(limits::DEFAULT_MEMORY_LIMIT_MB, 1024);
        assert_eq!(limits::DEFAULT_CPU_LIMIT_MILLICORES, 1000);
        assert_eq!(limits::DEFAULT_DISK_LIMIT_GB, 10);
    }

    #[test]
    fn test_timeouts_more_constants() {
        assert_eq!(timeouts::DEFAULT_HEALTH_CHECK_TIMEOUT.as_secs(), 10);
        assert_eq!(timeouts::DEFAULT_HEALTH_CHECK_INTERVAL.as_secs(), 30);
        assert_eq!(timeouts::DEFAULT_CACHE_TTL.as_secs(), 300);
        assert_eq!(timeouts::DEFAULT_HEARTBEAT_INTERVAL.as_secs(), 30);
    }

    #[test]
    fn test_events_all_constants() {
        assert_eq!(events::PLUGIN_STOPPED, "plugin.stopped");
        assert_eq!(events::PLUGIN_ERROR, "plugin.error");
        assert_eq!(events::COMMAND_FAILED, "command.failed");
        assert_eq!(events::SYSTEM_SHUTDOWN, "system.shutdown");
        assert_eq!(events::CUSTOM_EVENT, "custom.event");
    }

    #[test]
    fn test_files_base64_alphabet() {
        assert_eq!(files::BASE64_ALPHABET.len(), 64);
        assert!(files::SIZE_UNITS.contains(&"MB"));
        assert!(files::SIZE_UNITS.contains(&"GB"));
    }

    #[test]
    fn test_capabilities_all() {
        assert_eq!(capabilities::VISUALIZATION, "visualization");
        assert_eq!(capabilities::NETWORKING, "networking");
        assert_eq!(capabilities::MONITORING, "monitoring");
        assert_eq!(capabilities::DATA_PROCESSING, "data-processing");
    }
}
