// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Unified Constants Module
//!
//! This module centralizes all constants that were previously scattered across
//! the biomeOS ecosystem, providing a single source of truth for:
//! - Default endpoints and URLs
//! - Timeout durations
//! - Size limits and thresholds
//! - Version information
//! - Network configurations

#![forbid(unsafe_code)]

use std::time::Duration;

pub mod env_vars;
pub mod network;

pub mod capabilities;
pub mod capability;

/// Version and build information
pub mod version {
    /// Build timestamp, injected at compile time by `build.rs`.
    pub const BUILD_TIMESTAMP: &str = env!("BIOMEOS_BUILD_TIMESTAMP");

    /// Unified version constant - single source of truth
    pub const VERSION: &str = env!("CARGO_PKG_VERSION");

    /// Types version (alias for VERSION)
    pub const TYPES_VERSION: &str = VERSION;

    /// Build information for the entire `BiomeOS` ecosystem
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

    /// Default localhost address (IPv4, for binding) - fallback only
    pub const DEFAULT_LOCALHOST: &str = "127.0.0.1";

    /// Default localhost address (IPv6, for binding) - fallback only
    pub const DEFAULT_LOCALHOST_V6: &str = "::1";

    /// Production bind address (for accepting connections) - fallback only
    pub const PRODUCTION_BIND_ADDRESS: &str = "0.0.0.0";

    /// Ephemeral UDP bind address — binds any available port on all IPv4 interfaces.
    pub const EPHEMERAL_UDP_BIND: &str = "0.0.0.0:0";

    /// Get bind address from an optional value (same semantics as env `BIND_ADDRESS`).
    #[must_use]
    pub fn bind_address_from(val: Option<&str>) -> String {
        val.map_or_else(|| DEFAULT_LOCALHOST.to_string(), String::from)
    }

    /// Get bind address from environment or fallback to default
    ///
    /// Checks `BIND_ADDRESS` environment variable first.
    /// Falls back to `DEFAULT_LOCALHOST` for development.
    #[must_use]
    pub fn bind_address() -> String {
        bind_address_from(env::var(env_vars::BIND_ADDRESS).ok().as_deref())
    }

    /// Production bind address from an optional value.
    #[must_use]
    pub fn production_bind_address_from(val: Option<&str>) -> String {
        val.map_or_else(|| PRODUCTION_BIND_ADDRESS.to_string(), String::from)
    }

    /// Get production bind address from environment or fallback
    ///
    /// Checks `BIND_ADDRESS` environment variable first.
    /// Falls back to `PRODUCTION_BIND_ADDRESS` for production.
    #[must_use]
    pub fn production_bind_address() -> String {
        production_bind_address_from(env::var(env_vars::BIND_ADDRESS).ok().as_deref())
    }

    // REMOVED: FALLBACK_*_ENDPOINT constants
    //
    // These hardcoded endpoints violated BiomeOS's architecture principle:
    // "Primals do NOT have hardcoded knowledge of other primals"
    //
    // Instead, use:
    // 1. Capability-based env vars (e.g., DISCOVERY_ENDPOINT, SECURITY_ENDPOINT)
    // 2. Capability-based discovery via the discovery provider
    // 3. mDNS automatic discovery
    //
    // For local development, set environment variables:
    //   export DISCOVERY_ENDPOINT="unix:///run/biomeos/<family>/discovery.sock"
    //   export SECURITY_ENDPOINT="unix:///run/biomeos/<family>/security.sock"

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

    /// Default StatsD / DogStatsD UDP endpoint (local relay).
    ///
    /// Port must stay in sync with [`super::ports::STATSD`].
    pub const DEFAULT_STATSD_UDP_ENDPOINT: &str = "udp://localhost:8125";

    /// Default Zipkin HTTP collector URL (local development).
    ///
    /// Port must stay in sync with [`super::ports::ZIPKIN_HTTP`].
    pub const DEFAULT_ZIPKIN_HTTP_ENDPOINT: &str = "http://localhost:9411";

    /// Default Songbird-style HTTP registry base URL (local development).
    ///
    /// Port must stay in sync with [`super::ports::REGISTRY_HTTP`].
    pub const DEFAULT_REGISTRY_HTTP_URL: &str = "http://localhost:9999/registry";

    /// TCP bind address for all interfaces at `port` (`0.0.0.0:port`).
    #[must_use]
    pub const fn production_tcp_bind_addr(port: u16) -> std::net::SocketAddr {
        std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::UNSPECIFIED), port)
    }

    /// Default TCP bind address — localhost only.
    ///
    /// Primals bind `127.0.0.1` by default so TCP listeners are not exposed
    /// on LAN. Use `--bind 0.0.0.0` to opt in to all-interfaces binding.
    #[must_use]
    pub const fn default_tcp_bind_addr(port: u16) -> std::net::SocketAddr {
        std::net::SocketAddr::new(std::net::IpAddr::V4(std::net::Ipv4Addr::LOCALHOST), port)
    }

    /// TCP bind address with an optional host override.
    ///
    /// If `bind_host` is `Some`, parses it as an IP address and binds to
    /// `host:port`. If `None`, falls back to `127.0.0.1:port` (localhost).
    /// Accepts `"127.0.0.1"`, `"::1"`, `"0.0.0.0"`, or any valid IP.
    /// Pass `"0.0.0.0"` explicitly to bind all interfaces.
    #[must_use]
    pub fn tcp_bind_addr_with_host(bind_host: Option<&str>, port: u16) -> std::net::SocketAddr {
        match bind_host {
            Some(host) => {
                if let Ok(addr) = host.parse::<std::net::SocketAddr>() {
                    addr
                } else if let Ok(ip) = host.parse::<std::net::IpAddr>() {
                    std::net::SocketAddr::new(ip, port)
                } else {
                    default_tcp_bind_addr(port)
                }
            }
            None => default_tcp_bind_addr(port),
        }
    }
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

    /// Default IPC probe timeout (local socket round-trip, should be fast)
    pub const DEFAULT_IPC_TIMEOUT: Duration = Duration::from_secs(2);

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
    pub const DEFAULT_RETRY_DELAY: Duration = Duration::from_secs(1);

    /// Default heartbeat interval
    pub const DEFAULT_HEARTBEAT_INTERVAL: Duration = Duration::from_secs(30);

    /// Default ping interval
    pub const DEFAULT_PING_INTERVAL: Duration = Duration::from_secs(30);

    /// Default pong timeout
    pub const DEFAULT_PONG_TIMEOUT: Duration = Duration::from_secs(10);

    // Millisecond constants for APIs that expect u64 ms

    /// Default timeout for IPC/capability discovery operations (milliseconds).
    pub const DEFAULT_DISCOVERY_TIMEOUT_MS: u64 = 5000;

    /// Default timeout for connection establishment (milliseconds).
    pub const DEFAULT_CONNECTION_TIMEOUT_MS: u64 = 5000;

    /// Short timeout for fast operations like health checks (milliseconds).
    pub const SHORT_TIMEOUT_MS: u64 = 3000;

    /// Capability probe connect/read timeout (milliseconds).
    ///
    /// Intentionally short: probes are best-effort and must not block startup
    /// or lazy discovery paths.
    pub const PROBE_TIMEOUT_MS: u64 = 500;

    /// Capability probe timeout as a `Duration`.
    pub const PROBE_TIMEOUT: Duration = Duration::from_millis(PROBE_TIMEOUT_MS);
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

/// Well-named port constants for production code
///
/// These are the standard/default ports used across biomeOS.
/// Use these constants instead of magic numbers.
pub mod ports {
    /// STUN standard port (RFC 5389)
    pub const STUN: u16 = 3478;

    /// STUN-over-TLS port (RFC 5389)
    pub const STUN_TLS: u16 = 3479;

    /// mDNS standard port (RFC 6762)
    pub const MDNS: u16 = 5353;

    /// SSDP/UPnP standard port
    pub const SSDP: u16 = 1900;

    /// Default biomeOS API port
    pub const API_DEFAULT: u16 = 3000;

    /// Default HTTP bridge port
    pub const HTTP_BRIDGE: u16 = 8080;

    /// Default HTTPS port
    pub const HTTPS_DEFAULT: u16 = 8443;

    /// Default WebSocket port (alternative to WEBSOCKET for some services)
    pub const WS_DEFAULT: u16 = 8081;

    /// Default Neural API port
    pub const NEURAL_API: u16 = 9000;

    /// Default WebSocket port
    pub const WEBSOCKET: u16 = 8001;

    /// Default metrics port
    pub const METRICS: u16 = 8002;

    /// TURN/relay default port (RFC 5766)
    pub const RELAY: u16 = 3490;

    /// Default TCP port scan start for socket discovery
    pub const TCP_PORT_SCAN_START: u16 = 9100;

    /// Base TCP port for child-primal spawn allocation (TCP-only mode).
    ///
    /// `ExecutionContext::next_tcp_port()` starts here and increments.
    /// Discovery probes `TCP_SPAWN_BASE..TCP_SPAWN_BASE + TCP_SPAWN_SCAN_RANGE`.
    pub const TCP_SPAWN_BASE: u16 = 9900;

    /// Number of ports to scan during TCP-only auto-discovery.
    pub const TCP_SPAWN_SCAN_RANGE: u16 = 20;

    /// StatsD / DogStatsD standard UDP port
    pub const STATSD: u16 = 8125;

    /// Default Zipkin HTTP collector port
    pub const ZIPKIN_HTTP: u16 = 9411;

    /// Default local HTTP service registry (Songbird-style) port
    pub const REGISTRY_HTTP: u16 = 9999;

    /// Default port for test environments (avoids colliding with dev/production ports)
    pub const TEST_DEFAULT: u16 = 8083;
}

/// Security and authentication constants
pub mod security {
    use super::Duration;

    /// Default authentication timeout
    pub const DEFAULT_AUTH_TIMEOUT: Duration = Duration::from_secs(300); // 5 minutes

    /// Default token expiry
    pub const DEFAULT_TOKEN_EXPIRY: Duration = Duration::from_secs(3600); // 1 hour

    /// Default key rotation interval
    pub const DEFAULT_KEY_ROTATION_INTERVAL: Duration = Duration::from_secs(86_400); // 24 hours

    /// Default lockout duration
    pub const DEFAULT_LOCKOUT_DURATION: Duration = Duration::from_secs(1800); // 30 minutes

    /// Default key cache TTL
    pub const DEFAULT_KEY_CACHE_TTL: Duration = Duration::from_secs(300); // 5 minutes

    /// Default audit retention period
    pub const DEFAULT_AUDIT_RETENTION: Duration = Duration::from_secs(2_592_000); // 30 days

    /// Default compliance check interval
    pub const DEFAULT_COMPLIANCE_CHECK_INTERVAL: Duration = Duration::from_secs(86_400); // 24 hours

    /// Default threat scan interval
    pub const DEFAULT_THREAT_SCAN_INTERVAL: Duration = Duration::from_secs(3600);
    // 1 hour
}

/// Filesystem path constants for the 5-tier socket discovery standard.
///
/// These define the standardized base paths used when XDG runtime directories
/// are unavailable (tier 4 of the `PRIMAL_IPC_PROTOCOL.md` discovery chain).
pub mod runtime_paths {
    /// Linux runtime directory prefix (tier 3 in socket discovery).
    ///
    /// Used when `$XDG_RUNTIME_DIR` is not set but the system provides
    /// `/run/user/{uid}` directories. Append `/biomeos/` for socket paths.
    pub const LINUX_RUNTIME_DIR_PREFIX: &str = "/run/user";

    /// Android runtime directory for biomeOS sockets and data (tier 4).
    pub const ANDROID_RUNTIME_BASE: &str = "/data/local/tmp/biomeos";

    /// Base path for the `/tmp` fallback tier in socket discovery.
    ///
    /// Used as tier 5 when `$XDG_RUNTIME_DIR` is not set. Family-scoped
    /// variants append `-{family_id}` (e.g., `/tmp/biomeos-nat0`).
    pub const FALLBACK_RUNTIME_BASE: &str = "/tmp/biomeos";

    /// Socket directory name under the runtime base.
    pub const SOCKET_SUBDIR: &str = "sockets";

    /// biomeOS subdirectory name under `$XDG_RUNTIME_DIR`.
    pub const BIOMEOS_SUBDIR: &str = "biomeos";

    /// Build the family-scoped `/tmp` fallback path.
    ///
    /// Returns `/tmp/biomeos` when `family_id` is empty,
    /// or `/tmp/biomeos-{family_id}` otherwise.
    #[must_use]
    pub fn fallback_runtime_dir(family_id: &str) -> std::path::PathBuf {
        if family_id.is_empty() {
            std::path::PathBuf::from(FALLBACK_RUNTIME_BASE)
        } else {
            std::path::PathBuf::from(format!("{FALLBACK_RUNTIME_BASE}-{family_id}"))
        }
    }
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
    #[must_use]
    pub fn default_plugin_dir(primal_name: &str) -> String {
        format!(".{primal_name}/plugins")
    }

    /// Get default plugins directory for current primal (from environment)
    ///
    /// Reads the `PRIMAL_NAME` environment variable. Falls back to "unknown" if not set.
    ///
    /// # Example
    /// ```
    /// # use biomeos_types::constants::files;
    /// // Assumes PRIMAL_NAME is set in environment
    /// let dir = files::current_primal_plugin_dir();
    /// ```
    #[must_use]
    pub fn current_primal_plugin_dir() -> String {
        let primal_name = std::env::var(crate::env_config::vars::PRIMAL_NAME).unwrap_or_else(|_| "unknown".to_string());
        default_plugin_dir(&primal_name)
    }

    /// Default history file
    pub const DEFAULT_HISTORY_FILE: &str = "command_history.json";

    /// Default journal file
    pub const DEFAULT_JOURNAL_FILE: &str = "command_journal.json";

    /// Default Neural API metrics database filename (redb)
    pub const DEFAULT_NEURAL_METRICS_DB: &str = "neural_api_metrics.redb";
}

/// Runtime Unix socket basename rules for discovery scans
///
/// **`WateringHole`**: These name *infrastructure* IPC endpoints (orchestration, not primals).
/// Primal atomic sockets use `{instance}-{family_id}.sock` and are discovered by suffix match,
/// not by enumerating primal names.
pub mod runtime_ipc {
    /// Neural API control-plane socket: `{NEURAL_API_BASENAME_PREFIX}{family_id}.sock`
    pub const NEURAL_API_BASENAME_PREFIX: &str = "neural-api-";
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

/// Re-export commonly used constants at module level
pub use endpoints::DEFAULT_LOCALHOST;
pub use limits::{DEFAULT_BUFFER_SIZE, DEFAULT_MAX_CONNECTIONS};
pub use network::{DEFAULT_HTTP_PORT, DEFAULT_HTTPS_PORT};
pub use timeouts::{DEFAULT_CONNECTION_TIMEOUT, DEFAULT_REQUEST_TIMEOUT};
pub use version::*;

// Re-export capability constants for easy access
pub use capabilities::*;

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
