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

/// Default endpoint configurations
pub mod endpoints {
    /// Default localhost address
    pub const DEFAULT_LOCALHOST: &str = "127.0.0.1";
    
    /// Production bind address
    pub const PRODUCTION_BIND_ADDRESS: &str = "0.0.0.0";
    
    /// Default toadstool endpoint
    pub const DEFAULT_TOADSTOOL_ENDPOINT: &str = "http://localhost:8080";
    
    /// Default songbird endpoint
    pub const DEFAULT_SONGBIRD_ENDPOINT: &str = "http://localhost:3000";
    
    /// Default nestgate endpoint
    pub const DEFAULT_NESTGATE_ENDPOINT: &str = "http://localhost:8002";
    
    /// Default beardog endpoint
    pub const DEFAULT_BEARDOG_ENDPOINT: &str = "http://localhost:9000";
    
    /// Default squirrel endpoint
    pub const DEFAULT_SQUIRREL_ENDPOINT: &str = "http://localhost:8001";
    
    /// Default discovery endpoint
    pub const DEFAULT_DISCOVERY_ENDPOINT: &str = "http://localhost:8001";
    
    /// Health check endpoint path
    pub const HEALTH_ENDPOINT: &str = "/health";
    
    /// Metrics endpoint path
    pub const METRICS_ENDPOINT: &str = "/metrics";
    
    /// Admin endpoint path
    pub const ADMIN_ENDPOINT: &str = "/admin";
    
    /// WebSocket endpoint path
    pub const WS_ENDPOINT: &str = "/ws";
    
    /// Service discovery endpoint path
    pub const DISCOVERY_ENDPOINT: &str = "/discovery";
    
    /// Service registration endpoint path
    pub const REGISTRATION_ENDPOINT: &str = "/register";
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
pub mod network {
    /// Default HTTP port
    pub const DEFAULT_HTTP_PORT: u16 = 8080;
    
    /// Default HTTPS port
    pub const DEFAULT_HTTPS_PORT: u16 = 8443;
    
    /// Default WebSocket port
    pub const DEFAULT_WS_PORT: u16 = 8081;
    
    /// Default MCP port
    pub const DEFAULT_MCP_PORT: u16 = 3000;
    
    /// Default discovery port
    pub const DEFAULT_DISCOVERY_PORT: u16 = 8001;
    
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
    pub const DEFAULT_THREAT_SCAN_INTERVAL: Duration = Duration::from_secs(3600); // 1 hour
}

/// File and data constants
pub mod files {
    /// Default charset for encoding
    pub const DEFAULT_CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    
    /// Base64 alphabet
    pub const BASE64_ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    
    /// Size units for formatting
    pub const SIZE_UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB", "PB"];
    
    /// Default configuration file name
    pub const DEFAULT_CONFIG_FILE: &str = "biome.yaml";
    
    /// Default rules directory
    pub const DEFAULT_RULES_DIR: &str = ".rules";
    
    /// Default plugins directory
    pub const DEFAULT_PLUGIN_DIR: &str = ".squirrel/plugins";
    
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

/// Primal type constants
pub mod primals {
    /// Squirrel primal type
    pub const SQUIRREL_TYPE: &str = "squirrel";
    
    /// Toadstool primal type
    pub const TOADSTOOL_TYPE: &str = "toadstool";
    
    /// Songbird primal type
    pub const SONGBIRD_TYPE: &str = "songbird";
    
    /// Nestgate primal type
    pub const NESTGATE_TYPE: &str = "nestgate";
    
    /// Beardog primal type
    pub const BEARDOG_TYPE: &str = "beardog";
    
    /// BiomeOS primal type
    pub const BIOMEOS_TYPE: &str = "biomeos";
}

/// Environment variable names
pub mod env_vars {
    /// Bind address environment variable
    pub const BIND_ADDRESS: &str = "BIND_ADDRESS";
    
    /// HTTP port environment variable
    pub const HTTP_PORT: &str = "HTTP_PORT";
    
    /// WebSocket port environment variable
    pub const WEBSOCKET_PORT: &str = "WEBSOCKET_PORT";
    
    /// MCP WebSocket port environment variable
    pub const MCP_WEBSOCKET_PORT: &str = "MCP_WEBSOCKET_PORT";
    
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
pub use version::*;
pub use endpoints::DEFAULT_LOCALHOST;
pub use timeouts::{DEFAULT_CONNECTION_TIMEOUT, DEFAULT_REQUEST_TIMEOUT};
pub use network::{DEFAULT_HTTP_PORT, DEFAULT_HTTPS_PORT};
pub use limits::{DEFAULT_MAX_CONNECTIONS, DEFAULT_BUFFER_SIZE}; 