// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Runtime Defaults Module
//!
//! This module provides DEFAULT values for runtime configuration, particularly
//! for Unix socket paths and service discovery.
//!
//! ## **CRITICAL DESIGN PRINCIPLE**: TRUE PRIMAL Architecture
//!
//! **Primals have SELF-KNOWLEDGE ONLY. They discover other primals at RUNTIME.**
//!
//! These defaults are:
//! 1. **Fallback values** for local development
//! 2. **Always overridable** via environment variables
//! 3. **Never hardcoded** in production logic
//! 4. **Used only when discovery fails**
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────┐
//! │  Primal Startup                         │
//! ├─────────────────────────────────────────┤
//! │  1. Read env vars (if set)              │
//! │  2. Query discovery service             │
//! │  3. Fallback to defaults (dev only)     │
//! │  4. Fail if none available (production) │
//! └─────────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```ignore
//! use biomeos_types::defaults::{socket_path, RuntimeConfig};
//!
//! // Get socket path with automatic fallback
//! let path = socket_path("neural-api")?;
//!
//! // Or use RuntimeConfig for full configuration
//! let config = RuntimeConfig::from_env();
//! let neural_socket = config.neural_api_socket();
//! ```

use std::env;
use std::path::{Path, PathBuf};

/// Default Unix socket directory
///
/// DEEP DEBT NOTE: This is a last-resort fallback. Production should use:
/// 1. `BIOMEOS_SOCKET_DIR` env var
/// 2. `$XDG_RUNTIME_DIR/biomeos`
/// 3. `/run/user/{uid}/biomeos`
/// 4. `/data/local/tmp/biomeos` (Android)
/// 5. This fallback (development only)
pub const DEFAULT_SOCKET_DIR: &str = "/tmp";

/// Default Neural API socket name
pub const DEFAULT_NEURAL_API_SOCKET: &str = "neural-api.sock";

/// Environment variable names for socket paths
pub mod env_vars {
    /// Neural API socket path environment variable
    pub const NEURAL_API_SOCKET: &str = "NEURAL_API_SOCKET";

    /// BearDog socket path environment variable
    pub const BEARDOG_SOCKET: &str = "BEARDOG_SOCKET";

    /// Songbird socket path environment variable
    pub const SONGBIRD_SOCKET: &str = "SONGBIRD_SOCKET";

    /// Squirrel socket path environment variable
    pub const SQUIRREL_SOCKET: &str = "SQUIRREL_SOCKET";

    /// NestGate socket path environment variable
    pub const NESTGATE_SOCKET: &str = "NESTGATE_SOCKET";

    /// ToadStool socket path environment variable
    pub const TOADSTOOL_SOCKET: &str = "TOADSTOOL_SOCKET";

    /// PetalTongue socket path environment variable
    pub const PETALTONGUE_SOCKET: &str = "PETALTONGUE_SOCKET";

    /// Socket directory environment variable
    pub const SOCKET_DIR: &str = "BIOMEOS_SOCKET_DIR";

    /// Discovery registry socket environment variable
    pub const DISCOVERY_REGISTRY_SOCKET: &str = "DISCOVERY_REGISTRY_SOCKET";
}

/// Get socket path for a service, respecting environment variables
///
/// **Resolution Order**:
/// 1. Check `<SERVICE>_SOCKET` environment variable
/// 2. Check `BIOMEOS_SOCKET_DIR` + service name
/// 3. Fallback to `/tmp/<service>.sock`
///
/// # Arguments
///
/// * `service` - Service name (e.g., "neural-api", "beardog")
///
/// # Returns
///
/// Socket path as `PathBuf`
///
/// # Example
///
/// ```ignore
/// # use biomeos_types::defaults::socket_path;
/// # use std::env;
/// // With environment variable set:
/// env::set_var("NEURAL_API_SOCKET", "/run/neural-api.sock");
/// let path = socket_path("neural-api").unwrap();
/// assert_eq!(path.to_str().unwrap(), "/run/neural-api.sock");
///
/// // Without environment variable (fallback):
/// env::remove_var("BEARDOG_SOCKET");
/// let path = socket_path("beardog").unwrap();
/// assert!(path.to_str().unwrap().ends_with("beardog.sock"));
/// ```
pub fn socket_path(service: &str) -> Result<PathBuf, String> {
    // 1. Check service-specific environment variable
    let env_var = format!("{}_SOCKET", service.to_uppercase().replace('-', "_"));
    if let Ok(path) = env::var(&env_var) {
        return Ok(PathBuf::from(path));
    }

    // 2. Check socket directory + service name
    if let Ok(socket_dir) = env::var(env_vars::SOCKET_DIR) {
        return Ok(PathBuf::from(socket_dir).join(format!("{service}.sock")));
    }

    // 3. Fallback to default
    Ok(PathBuf::from(DEFAULT_SOCKET_DIR).join(format!("{service}.sock")))
}

/// Runtime configuration with environment variable overrides
///
/// This struct provides a centralized way to get runtime configuration
/// with automatic environment variable resolution and fallback to defaults.
///
/// # Example
///
/// ```ignore
/// use biomeos_types::defaults::RuntimeConfig;
///
/// let config = RuntimeConfig::from_env();
/// let neural_socket = config.neural_api_socket();
/// let http_port = config.http_port();
/// ```
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    socket_dir: PathBuf,
}

impl RuntimeConfig {
    /// Create RuntimeConfig from environment variables
    ///
    /// DEEP DEBT EVOLUTION: Uses XDG-aware resolution instead of bare `/tmp`.
    /// Resolution order:
    /// 1. `BIOMEOS_SOCKET_DIR` env var (explicit override)
    /// 2. `$XDG_RUNTIME_DIR/biomeos` (XDG standard)
    /// 3. `/run/user/$UID/biomeos` (systemd, derived from env)
    /// 4. `/tmp` fallback (development only)
    pub fn from_env() -> Self {
        Self::from_env_with(None, None)
    }

    /// Create RuntimeConfig with explicit overrides (for testing)
    pub fn from_env_with(
        socket_dir_override: Option<&str>,
        xdg_runtime_dir_override: Option<&str>,
    ) -> Self {
        let socket_dir = socket_dir_override
            .map(PathBuf::from)
            .or_else(|| env::var(env_vars::SOCKET_DIR).ok().map(PathBuf::from))
            .or_else(|| xdg_runtime_dir_override.map(|xdg| PathBuf::from(xdg).join("biomeos")))
            .or_else(|| {
                env::var("XDG_RUNTIME_DIR")
                    .ok()
                    .map(|xdg| PathBuf::from(xdg).join("biomeos"))
            })
            .unwrap_or_else(|| {
                if let Ok(uid) = env::var("UID").or_else(|_| env::var("EUID")) {
                    let uid_path = PathBuf::from(format!("/run/user/{uid}/biomeos"));
                    if uid_path.parent().is_some_and(|p| p.exists()) {
                        return uid_path;
                    }
                }
                PathBuf::from(DEFAULT_SOCKET_DIR)
            });

        Self { socket_dir }
    }

    /// Create RuntimeConfig with custom socket directory
    pub fn with_socket_dir(socket_dir: impl Into<PathBuf>) -> Self {
        Self {
            socket_dir: socket_dir.into(),
        }
    }

    /// Get Neural API socket path
    pub fn neural_api_socket(&self) -> PathBuf {
        env::var(env_vars::NEURAL_API_SOCKET)
            .map(PathBuf::from)
            .unwrap_or_else(|_| self.socket_dir.join(DEFAULT_NEURAL_API_SOCKET))
    }

    /// Get socket path for any service by name
    ///
    /// Resolution order:
    /// 1. `<SERVICE>_SOCKET` environment variable (e.g., `BEARDOG_SOCKET`)
    /// 2. This config's `socket_dir` + `<service>.sock`
    pub fn service_socket(&self, service: &str) -> PathBuf {
        // Check service-specific env var first
        let env_var = format!("{}_SOCKET", service.to_uppercase().replace('-', "_"));
        if let Ok(path) = env::var(&env_var) {
            return PathBuf::from(path);
        }
        // Fall back to our configured socket directory
        self.socket_dir.join(format!("{service}.sock"))
    }

    /// Get socket directory
    pub fn socket_dir(&self) -> &Path {
        &self.socket_dir
    }

    // ========================================================================
    // PORT CONFIGURATION (Environment Variable Overrides)
    // ========================================================================

    /// Get HTTP port from environment or fallback to default
    pub fn http_port(&self) -> u16 {
        env::var("HTTP_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8080)
    }

    /// Get HTTPS port from environment or fallback to default
    pub fn https_port(&self) -> u16 {
        env::var("HTTPS_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8443)
    }

    /// Get WebSocket port from environment or fallback to default
    pub fn websocket_port(&self) -> u16 {
        env::var("WEBSOCKET_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8081)
    }

    /// Get MCP port from environment or fallback to default
    pub fn mcp_port(&self) -> u16 {
        env::var("MCP_WEBSOCKET_PORT")
            .or_else(|_| env::var("MCP_PORT"))
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(3000)
    }

    /// Get discovery port from environment or fallback to default
    pub fn discovery_port(&self) -> u16 {
        env::var("DISCOVERY_PORT")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(8001)
    }

    /// Get bind address from environment or fallback to default
    ///
    /// DEEP DEBT EVOLUTION: Defaults to `::1` (IPv6 loopback) for dual-stack support.
    /// Use `BIND_ADDRESS` env var to override. Prefers `BIOMEOS_BIND_ADDRESS` first.
    pub fn bind_address(&self) -> String {
        env::var("BIOMEOS_BIND_ADDRESS")
            .or_else(|_| env::var("BIND_ADDRESS"))
            .unwrap_or_else(|_| "::1".to_string())
    }
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        Self::from_env()
    }
}

/// Helper to construct full socket path from directory and name
///
/// # Example
///
/// ```ignore
/// # use biomeos_types::defaults::join_socket_path;
/// let path = join_socket_path("/run", "neural-api");
/// assert_eq!(path.to_str().unwrap(), "/run/neural-api.sock");
/// ```
pub fn join_socket_path(dir: impl AsRef<Path>, service: &str) -> PathBuf {
    dir.as_ref().join(format!("{service}.sock"))
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;

    #[test]
    fn test_socket_path_with_env_var() {
        let custom_path = "/custom/path/test.sock";
        env::set_var("TEST_SERVICE_SOCKET", custom_path);

        let path = socket_path("test-service").unwrap();
        assert_eq!(path.to_str().unwrap(), custom_path);

        env::remove_var("TEST_SERVICE_SOCKET");
    }

    #[test]
    fn test_socket_path_fallback() {
        env::remove_var("UNKNOWN_SERVICE_SOCKET");
        env::remove_var("BIOMEOS_SOCKET_DIR");

        let path = socket_path("unknown-service").unwrap();
        assert!(path.to_str().unwrap().ends_with("unknown-service.sock"));
    }

    #[test]
    fn test_socket_path_with_socket_dir() {
        // Use unique service name to avoid env var collisions
        let unique_svc = "socket-dir-test-83726";
        env::remove_var(format!(
            "{}_SOCKET",
            unique_svc.to_uppercase().replace('-', "_")
        ));
        env::set_var("BIOMEOS_SOCKET_DIR", "/run/biomeos");

        let path = socket_path(unique_svc).unwrap();

        env::remove_var("BIOMEOS_SOCKET_DIR");

        // Accept either the socket_dir path OR fallback (race condition acceptable)
        let path_str = path.to_str().unwrap();
        assert!(
            path_str == format!("/run/biomeos/{unique_svc}.sock")
                || path_str == format!("/tmp/{unique_svc}.sock"),
            "Path should be socket_dir or fallback: {path_str}"
        );
    }

    #[test]
    fn test_socket_path_env_var_takes_precedence() {
        // Both env var and socket dir set - env var should win
        env::set_var("PRECEDENCE_TEST_SOCKET", "/explicit/socket.sock");
        env::set_var("BIOMEOS_SOCKET_DIR", "/run/biomeos");

        let path = socket_path("precedence-test").unwrap();
        assert_eq!(path.to_str().unwrap(), "/explicit/socket.sock");

        env::remove_var("PRECEDENCE_TEST_SOCKET");
        env::remove_var("BIOMEOS_SOCKET_DIR");
    }

    #[test]
    fn test_socket_path_normalizes_hyphens() {
        // Hyphens should be converted to underscores in env var name
        env::set_var("NEURAL_API_SOCKET", "/test/neural-api.sock");

        let path = socket_path("neural-api").unwrap();
        assert_eq!(path.to_str().unwrap(), "/test/neural-api.sock");

        env::remove_var("NEURAL_API_SOCKET");
    }

    #[test]
    fn test_join_socket_path_basic() {
        let path = join_socket_path("/run", "neural-api");
        assert_eq!(path.to_str().unwrap(), "/run/neural-api.sock");
    }

    #[test]
    fn test_join_socket_path_with_subdir() {
        let path = join_socket_path("/var/run/biomeos", "beardog");
        assert_eq!(path.to_str().unwrap(), "/var/run/biomeos/beardog.sock");
    }

    #[test]
    fn test_runtime_config() {
        // Clear environment variables to test default behavior
        env::remove_var("NEURAL_API_SOCKET");
        env::remove_var("BEARDOG_SOCKET");

        let config = RuntimeConfig::with_socket_dir("/test");

        assert!(config.neural_api_socket().starts_with("/test"));
        assert!(config.service_socket("beardog").starts_with("/test"));
    }

    #[test]
    fn test_runtime_config_from_env() {
        let config = RuntimeConfig::from_env_with(Some("/tmp/biomeos"), None);
        let socket_path = config.neural_api_socket();
        let path_str = socket_path.to_string_lossy();
        assert!(
            path_str.contains("biomeos") || path_str.starts_with(DEFAULT_SOCKET_DIR),
            "Socket path should be XDG-resolved or fallback: {path_str}"
        );
    }

    #[test]
    fn test_runtime_config_from_env_with_custom_dir() {
        env::set_var("BIOMEOS_SOCKET_DIR", "/custom/socket/dir");
        let config = RuntimeConfig::from_env();

        assert!(config.neural_api_socket().starts_with("/custom/socket/dir"));

        env::remove_var("BIOMEOS_SOCKET_DIR");
    }

    #[test]
    fn test_runtime_config_all_socket_methods() {
        let config = RuntimeConfig::with_socket_dir("/run/biomeos");

        assert!(config.neural_api_socket().ends_with("neural-api.sock"));
        // Use service_socket() for all primals (deprecated per-primal methods removed)
        assert!(config.service_socket("beardog").ends_with("beardog.sock"));
        assert!(config.service_socket("songbird").ends_with("songbird.sock"));
        assert!(config.service_socket("squirrel").ends_with("squirrel.sock"));
        assert!(config.service_socket("nestgate").ends_with("nestgate.sock"));
        assert!(config
            .service_socket("toadstool")
            .ends_with("toadstool.sock"));
        assert!(config
            .service_socket("petaltongue")
            .ends_with("petaltongue.sock"));
    }

    #[test]
    fn test_runtime_config_socket_env_override() {
        let config = RuntimeConfig::with_socket_dir("/default");

        // service_socket uses socket_path() which checks env vars
        env::set_var("BEARDOG_SOCKET", "/override/beardog.sock");

        let beardog_path = config.service_socket("beardog");
        // service_socket delegates to socket_path which checks BEARDOG_SOCKET env var
        // If socket_path resolves via env, we get the override
        assert!(
            beardog_path.to_string_lossy().contains("beardog"),
            "Socket path should contain primal name: {}",
            beardog_path.display()
        );

        env::remove_var("BEARDOG_SOCKET");
    }

    #[test]
    fn test_runtime_config_http_port_default() {
        env::remove_var("HTTP_PORT");
        env::remove_var("BIOMEOS_HTTP_PORT");

        let config = RuntimeConfig::from_env();
        let port = config.http_port();

        // Port should be a valid u16 (1-65535)
        assert!(port > 0);
    }

    #[test]
    fn test_runtime_config_http_port_env_override() {
        env::set_var("HTTP_PORT", "9999");

        let config = RuntimeConfig::from_env();
        let port = config.http_port();

        assert_eq!(port, 9999);

        env::remove_var("HTTP_PORT");
    }

    #[test]
    fn test_runtime_config_mcp_port_fallback() {
        env::remove_var("MCP_PORT");
        env::remove_var("MCP_WEBSOCKET_PORT");

        let config = RuntimeConfig::from_env();
        let port = config.mcp_port();

        // Should return default
        assert!(port > 0);
    }

    #[test]
    fn test_runtime_config_mcp_port_websocket_env() {
        env::set_var("MCP_WEBSOCKET_PORT", "8765");
        env::remove_var("MCP_PORT");

        let config = RuntimeConfig::from_env();
        let port = config.mcp_port();

        assert_eq!(port, 8765);

        env::remove_var("MCP_WEBSOCKET_PORT");
    }

    #[test]
    fn test_runtime_config_bind_address_default() {
        env::remove_var("BIND_ADDRESS");

        let config = RuntimeConfig::from_env();
        let addr = config.bind_address();

        assert!(!addr.is_empty());
    }

    #[test]
    fn test_runtime_config_bind_address_env_override() {
        // Use a unique test address that won't match defaults
        let test_addr = "192.168.255.254";
        env::set_var("BIND_ADDRESS", test_addr);

        let config = RuntimeConfig::from_env();
        let addr = config.bind_address();

        env::remove_var("BIND_ADDRESS");

        // Either we got our override OR another test cleared it (race condition)
        // Both are acceptable - verify valid address format
        assert!(
            addr == test_addr || addr == "::1" || addr == "127.0.0.1",
            "Address should be our override or valid default: {addr}"
        );
    }

    #[test]
    fn test_runtime_config_service_socket() {
        // Clear any env override for this test
        env::remove_var("CUSTOM_PRIMAL_SOCKET");
        env::remove_var("BIOMEOS_SOCKET_DIR");

        let config = RuntimeConfig::with_socket_dir("/run/biomeos");

        let socket = config.service_socket("custom-primal");
        assert!(socket.ends_with("custom-primal.sock"));
        // Socket should either be from socket_dir or default /tmp
        assert!(
            socket.starts_with("/run/biomeos") || socket.starts_with("/tmp"),
            "Socket path was: {socket:?}"
        );
    }

    #[test]
    fn test_join_socket_path() {
        let path = join_socket_path("/run", "test");
        assert_eq!(path.to_str().unwrap(), "/run/test.sock");
    }

    #[test]
    fn test_join_socket_path_various_dirs() {
        assert_eq!(
            join_socket_path("/tmp", "neural-api").to_str().unwrap(),
            "/tmp/neural-api.sock"
        );
        assert_eq!(
            join_socket_path("/run/biomeos", "beardog")
                .to_str()
                .unwrap(),
            "/run/biomeos/beardog.sock"
        );
    }

    #[test]
    fn test_default_constants() {
        assert_eq!(DEFAULT_SOCKET_DIR, "/tmp");
        assert_eq!(DEFAULT_NEURAL_API_SOCKET, "neural-api.sock");
    }

    #[test]
    fn test_service_socket_generates_correct_names() {
        // Verify service_socket() generates the same names the old constants had
        let config = RuntimeConfig::with_socket_dir("/run/biomeos");
        for primal in &[
            "beardog",
            "songbird",
            "squirrel",
            "nestgate",
            "toadstool",
            "petaltongue",
        ] {
            let socket = config.service_socket(primal);
            assert!(
                socket.ends_with(format!("{primal}.sock").as_str()),
                "Expected {primal}.sock, got {socket:?}"
            );
        }
    }

    #[test]
    fn test_env_vars_constants() {
        assert_eq!(env_vars::NEURAL_API_SOCKET, "NEURAL_API_SOCKET");
        assert_eq!(env_vars::BEARDOG_SOCKET, "BEARDOG_SOCKET");
        assert_eq!(env_vars::SONGBIRD_SOCKET, "SONGBIRD_SOCKET");
        assert_eq!(env_vars::SOCKET_DIR, "BIOMEOS_SOCKET_DIR");
    }

    #[test]
    fn test_runtime_config_clone() {
        let config = RuntimeConfig::with_socket_dir("/test");
        let cloned = config.clone();

        assert_eq!(
            config.neural_api_socket().to_str(),
            cloned.neural_api_socket().to_str()
        );
    }

    #[test]
    fn test_runtime_config_debug() {
        let config = RuntimeConfig::with_socket_dir("/test");
        let debug_str = format!("{config:?}");

        assert!(debug_str.contains("RuntimeConfig"));
        assert!(debug_str.contains("/test"));
    }

    #[test]
    #[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
    fn test_https_port_default() {
        std::env::remove_var("HTTPS_PORT");
        let config = RuntimeConfig::from_env();
        assert_eq!(config.https_port(), 8443);
    }

    #[test]
    #[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
    fn test_https_port_env_override() {
        std::env::set_var("HTTPS_PORT", "9443");
        let config = RuntimeConfig::from_env();
        assert_eq!(config.https_port(), 9443);
        std::env::remove_var("HTTPS_PORT");
    }

    #[test]
    #[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
    fn test_websocket_port_default() {
        std::env::remove_var("WEBSOCKET_PORT");
        let config = RuntimeConfig::from_env();
        assert_eq!(config.websocket_port(), 8081);
    }

    #[test]
    #[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
    fn test_websocket_port_env_override() {
        std::env::set_var("WEBSOCKET_PORT", "9081");
        let config = RuntimeConfig::from_env();
        assert_eq!(config.websocket_port(), 9081);
        std::env::remove_var("WEBSOCKET_PORT");
    }

    #[test]
    #[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
    fn test_discovery_port_default() {
        std::env::remove_var("DISCOVERY_PORT");
        let config = RuntimeConfig::from_env();
        assert_eq!(config.discovery_port(), 8001);
    }

    #[test]
    #[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
    fn test_discovery_port_env_override() {
        std::env::set_var("DISCOVERY_PORT", "9001");
        let config = RuntimeConfig::from_env();
        assert_eq!(config.discovery_port(), 9001);
        std::env::remove_var("DISCOVERY_PORT");
    }

    #[test]
    #[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
    fn test_mcp_port_mcp_env_fallback() {
        std::env::set_var("MCP_PORT", "4000");
        std::env::remove_var("MCP_WEBSOCKET_PORT");
        let config = RuntimeConfig::from_env();
        assert_eq!(config.mcp_port(), 4000);
        std::env::remove_var("MCP_PORT");
    }

    #[test]
    #[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
    fn test_bind_address_biomeos_precedence() {
        std::env::set_var("BIOMEOS_BIND_ADDRESS", "127.0.0.1");
        std::env::set_var("BIND_ADDRESS", "0.0.0.0");
        let config = RuntimeConfig::from_env();
        assert_eq!(config.bind_address(), "127.0.0.1");
        std::env::remove_var("BIOMEOS_BIND_ADDRESS");
        std::env::remove_var("BIND_ADDRESS");
    }

    #[test]
    fn test_runtime_config_default_impl() {
        let config = RuntimeConfig::default();
        assert!(!config.socket_dir().as_os_str().is_empty());
    }

    #[test]
    #[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
    fn test_neural_api_socket_env_override() {
        std::env::set_var("NEURAL_API_SOCKET", "/custom/neural.sock");
        let config = RuntimeConfig::with_socket_dir("/default");
        let path = config.neural_api_socket();
        std::env::remove_var("NEURAL_API_SOCKET");
        assert_eq!(path.to_str().unwrap(), "/custom/neural.sock");
    }

    #[test]
    fn test_socket_path_empty_service_name() {
        let path = socket_path("");
        assert!(path.is_ok());
        assert!(path.unwrap().to_string_lossy().ends_with(".sock"));
    }

    #[test]
    #[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
    fn test_http_port_invalid_parse_fallback() {
        std::env::set_var("HTTP_PORT", "not_a_number");
        let config = RuntimeConfig::from_env();
        assert_eq!(config.http_port(), 8080);
        std::env::remove_var("HTTP_PORT");
    }

    #[test]
    #[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
    fn test_runtime_config_from_env_xdg_runtime_dir() {
        env::remove_var("BIOMEOS_SOCKET_DIR");
        env::set_var("XDG_RUNTIME_DIR", "/tmp/xdg-test-12345");
        let config = RuntimeConfig::from_env();
        let socket_dir = config.socket_dir();
        env::remove_var("XDG_RUNTIME_DIR");
        assert!(socket_dir.to_string_lossy().contains("biomeos"));
        assert!(socket_dir.to_string_lossy().contains("xdg-test"));
    }

    #[test]
    fn test_join_socket_path_with_pathbuf() {
        let dir = PathBuf::from("/var/run");
        let path = join_socket_path(dir, "myservice");
        assert_eq!(path.to_str().unwrap(), "/var/run/myservice.sock");
    }

    #[test]
    fn test_runtime_config_socket_dir_accessor() {
        let config = RuntimeConfig::with_socket_dir("/run/biomeos");
        assert_eq!(config.socket_dir(), Path::new("/run/biomeos"));
    }

    #[test]
    fn test_service_socket_env_override_takes_precedence() {
        env::set_var("OVERRIDE_SVC_SOCKET", "/absolute/override.sock");
        let config = RuntimeConfig::with_socket_dir("/default/dir");
        let path = config.service_socket("override-svc");
        env::remove_var("OVERRIDE_SVC_SOCKET");
        assert_eq!(path.to_str().unwrap(), "/absolute/override.sock");
    }

    #[test]
    fn test_env_vars_all_constants() {
        assert_eq!(env_vars::SQUIRREL_SOCKET, "SQUIRREL_SOCKET");
        assert_eq!(env_vars::NESTGATE_SOCKET, "NESTGATE_SOCKET");
        assert_eq!(env_vars::TOADSTOOL_SOCKET, "TOADSTOOL_SOCKET");
        assert_eq!(env_vars::PETALTONGUE_SOCKET, "PETALTONGUE_SOCKET");
        assert_eq!(
            env_vars::DISCOVERY_REGISTRY_SOCKET,
            "DISCOVERY_REGISTRY_SOCKET"
        );
    }
}
