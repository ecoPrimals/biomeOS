// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

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

use crate::constants::ports;
use std::collections::HashMap;
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

    /// `BearDog` socket path environment variable
    pub const BEARDOG_SOCKET: &str = "BEARDOG_SOCKET";

    /// Songbird socket path environment variable
    pub const SONGBIRD_SOCKET: &str = "SONGBIRD_SOCKET";

    /// Squirrel socket path environment variable
    pub const SQUIRREL_SOCKET: &str = "SQUIRREL_SOCKET";

    /// `NestGate` socket path environment variable
    pub const NESTGATE_SOCKET: &str = "NESTGATE_SOCKET";

    /// `ToadStool` socket path environment variable
    pub const TOADSTOOL_SOCKET: &str = "TOADSTOOL_SOCKET";

    /// `PetalTongue` socket path environment variable
    pub const PETALTONGUE_SOCKET: &str = "PETALTONGUE_SOCKET";

    /// Socket directory environment variable
    pub const SOCKET_DIR: &str = "BIOMEOS_SOCKET_DIR";

    /// Discovery registry socket environment variable
    pub const DISCOVERY_REGISTRY_SOCKET: &str = "DISCOVERY_REGISTRY_SOCKET";

    /// Derive the socket environment variable name from a primal process name.
    ///
    /// Strips common suffixes (`-server`, `-orchestrator`), uppercases, replaces
    /// hyphens with underscores, and appends `_SOCKET`.
    ///
    /// ```
    /// # use biomeos_types::defaults::env_vars::socket_env_key;
    /// assert_eq!(socket_env_key("beardog-server"), "BEARDOG_SOCKET");
    /// assert_eq!(socket_env_key("songbird-orchestrator"), "SONGBIRD_SOCKET");
    /// assert_eq!(socket_env_key("toadstool"), "TOADSTOOL_SOCKET");
    /// assert_eq!(socket_env_key("nestgate"), "NESTGATE_SOCKET");
    /// assert_eq!(socket_env_key("my-custom-primal"), "MY_CUSTOM_PRIMAL_SOCKET");
    /// ```
    #[must_use] 
    pub fn socket_env_key(primal_name: &str) -> String {
        let base = primal_name
            .strip_suffix("-server")
            .or_else(|| primal_name.strip_suffix("-orchestrator"))
            .unwrap_or(primal_name);
        format!("{}_SOCKET", base.to_uppercase().replace('-', "_"))
    }
}

/// Get socket path with explicit environment map (for testing)
#[expect(
    clippy::implicit_hasher,
    reason = "HashMap with default hasher is idiomatic for env maps"
)]
pub fn socket_path_with(service: &str, env: &HashMap<String, String>) -> Result<PathBuf, String> {
    // 1. Check service-specific environment variable
    let env_var = format!("{}_SOCKET", service.to_uppercase().replace('-', "_"));
    if let Some(path) = env.get(&env_var) {
        return Ok(PathBuf::from(path));
    }

    // 2. Check socket directory + service name
    if let Some(socket_dir) = env.get(env_vars::SOCKET_DIR) {
        return Ok(PathBuf::from(socket_dir).join(format!("{service}.sock")));
    }

    // 3. Fallback to default
    Ok(PathBuf::from(DEFAULT_SOCKET_DIR).join(format!("{service}.sock")))
}

/// Get socket path for a service, respecting environment variables
///
/// **Resolution Order**:
/// 1. Check `<SERVICE>_SOCKET` environment variable
/// 2. Check `BIOMEOS_SOCKET_DIR` + service name
/// 3. Fallback to `/tmp/<service>.sock`
///
/// Delegates to [`socket_path_with`] using the current process environment.
/// For testing without mutating process env, use [`socket_path_with`].
pub fn socket_path(service: &str) -> Result<PathBuf, String> {
    socket_path_with(service, &env::vars().collect())
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
    /// Create `RuntimeConfig` from environment variables
    ///
    /// DEEP DEBT EVOLUTION: Uses XDG-aware resolution instead of bare `/tmp`.
    /// Resolution order:
    /// 1. `BIOMEOS_SOCKET_DIR` env var (explicit override)
    /// 2. `$XDG_RUNTIME_DIR/biomeos` (XDG standard)
    /// 3. `/run/user/$UID/biomeos` (systemd, derived from env)
    /// 4. `/tmp` fallback (development only)
    #[must_use]
    pub fn from_env() -> Self {
        Self::from_env_with(None, None)
    }

    /// Create `RuntimeConfig` with explicit overrides (for testing)
    #[must_use]
    pub fn from_env_with(
        socket_dir_override: Option<&str>,
        xdg_runtime_dir_override: Option<&str>,
    ) -> Self {
        Self::from_env_with_map(
            &env::vars().collect(),
            socket_dir_override,
            xdg_runtime_dir_override,
        )
    }

    /// Create `RuntimeConfig` from explicit environment map (for testing)
    #[must_use]
    pub fn from_env_with_map(
        env: &HashMap<String, String>,
        socket_dir_override: Option<&str>,
        xdg_runtime_dir_override: Option<&str>,
    ) -> Self {
        let socket_dir = socket_dir_override
            .map(PathBuf::from)
            .or_else(|| {
                env.get(env_vars::SOCKET_DIR)
                    .map(|s| PathBuf::from(s.as_str()))
            })
            .or_else(|| xdg_runtime_dir_override.map(|xdg| PathBuf::from(xdg).join("biomeos")))
            .or_else(|| {
                env.get("XDG_RUNTIME_DIR")
                    .map(|xdg| PathBuf::from(xdg.as_str()).join("biomeos"))
            })
            .unwrap_or_else(|| {
                if let Some(uid) = env.get("UID").or_else(|| env.get("EUID")) {
                    let uid_path = PathBuf::from(format!("/run/user/{uid}/biomeos"));
                    if uid_path.parent().is_some_and(Path::exists) {
                        return uid_path;
                    }
                }
                PathBuf::from(DEFAULT_SOCKET_DIR)
            });

        Self { socket_dir }
    }

    /// Create `RuntimeConfig` with custom socket directory
    #[must_use]
    pub fn with_socket_dir(socket_dir: impl Into<PathBuf>) -> Self {
        Self {
            socket_dir: socket_dir.into(),
        }
    }

    /// Get Neural API socket path
    #[must_use]
    pub fn neural_api_socket(&self) -> PathBuf {
        self.neural_api_socket_with(&env::vars().collect())
    }

    /// Get Neural API socket path with explicit environment map (for testing)
    #[must_use]
    pub fn neural_api_socket_with(&self, env: &HashMap<String, String>) -> PathBuf {
        env.get(env_vars::NEURAL_API_SOCKET).map_or_else(
            || self.socket_dir.join(DEFAULT_NEURAL_API_SOCKET),
            PathBuf::from,
        )
    }

    /// Get socket path for any service by name
    ///
    /// Resolution order:
    /// 1. `<SERVICE>_SOCKET` environment variable (e.g., `BEARDOG_SOCKET`)
    /// 2. This config's `socket_dir` + `<service>.sock`
    #[must_use]
    pub fn service_socket(&self, service: &str) -> PathBuf {
        self.service_socket_with(service, &env::vars().collect())
    }

    /// Get socket path for any service with explicit environment map (for testing)
    #[must_use]
    pub fn service_socket_with(&self, service: &str, env: &HashMap<String, String>) -> PathBuf {
        let env_var = format!("{}_SOCKET", service.to_uppercase().replace('-', "_"));
        if let Some(path) = env.get(&env_var) {
            return PathBuf::from(path);
        }
        self.socket_dir.join(format!("{service}.sock"))
    }

    /// Get socket directory
    #[must_use]
    pub fn socket_dir(&self) -> &Path {
        &self.socket_dir
    }

    // ========================================================================
    // PORT CONFIGURATION (Environment Variable Overrides)
    // ========================================================================

    /// Get HTTP port from environment or fallback to default
    #[must_use]
    pub fn http_port(&self) -> u16 {
        Self::http_port_with(&env::vars().collect())
    }

    /// Get HTTP port with explicit environment map (for testing)
    #[must_use]
    pub fn http_port_with(env: &HashMap<String, String>) -> u16 {
        env.get("HTTP_PORT")
            .and_then(|v| v.parse().ok())
            .unwrap_or(ports::HTTP_BRIDGE)
    }

    /// Get HTTPS port from environment or fallback to default
    #[must_use]
    pub fn https_port(&self) -> u16 {
        Self::https_port_with(&env::vars().collect())
    }

    /// Get HTTPS port with explicit environment map (for testing)
    #[must_use]
    pub fn https_port_with(env: &HashMap<String, String>) -> u16 {
        env.get("HTTPS_PORT")
            .and_then(|v| v.parse().ok())
            .unwrap_or(ports::HTTPS_DEFAULT)
    }

    /// Get WebSocket port from environment or fallback to default
    #[must_use]
    pub fn websocket_port(&self) -> u16 {
        Self::websocket_port_with(&env::vars().collect())
    }

    /// Get WebSocket port with explicit environment map (for testing)
    #[must_use]
    pub fn websocket_port_with(env: &HashMap<String, String>) -> u16 {
        env.get("WEBSOCKET_PORT")
            .and_then(|v| v.parse().ok())
            .unwrap_or(ports::WS_DEFAULT)
    }

    /// Get MCP port from environment or fallback to default
    #[must_use]
    pub fn mcp_port(&self) -> u16 {
        Self::mcp_port_with(&env::vars().collect())
    }

    /// Get MCP port with explicit environment map (for testing)
    #[must_use]
    pub fn mcp_port_with(env: &HashMap<String, String>) -> u16 {
        env.get("MCP_WEBSOCKET_PORT")
            .or_else(|| env.get("MCP_PORT"))
            .and_then(|v| v.parse().ok())
            .unwrap_or(ports::API_DEFAULT)
    }

    /// Get discovery port from environment or fallback to default
    #[must_use]
    pub fn discovery_port(&self) -> u16 {
        Self::discovery_port_with(&env::vars().collect())
    }

    /// Get discovery port with explicit environment map (for testing)
    #[must_use]
    pub fn discovery_port_with(env: &HashMap<String, String>) -> u16 {
        env.get("DISCOVERY_PORT")
            .and_then(|v| v.parse().ok())
            .unwrap_or(ports::WEBSOCKET)
    }

    /// Get bind address from environment or fallback to default
    ///
    /// DEEP DEBT EVOLUTION: Defaults to `::1` (IPv6 loopback) for dual-stack support.
    /// Use `BIND_ADDRESS` env var to override. Prefers `BIOMEOS_BIND_ADDRESS` first.
    #[must_use]
    pub fn bind_address(&self) -> String {
        Self::bind_address_with(&env::vars().collect())
    }

    /// Get bind address with explicit environment map (for testing)
    #[must_use]
    pub fn bind_address_with(env: &HashMap<String, String>) -> String {
        env.get("BIOMEOS_BIND_ADDRESS")
            .or_else(|| env.get("BIND_ADDRESS"))
            .cloned()
            .unwrap_or_else(|| "::1".to_string())
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
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
mod tests {
    use std::collections::HashMap;

    use super::*;

    #[test]
    fn test_socket_path_with_env_var() {
        let custom_path = "/custom/path/test.sock";
        let mut env = HashMap::new();
        env.insert("TEST_SERVICE_SOCKET".to_string(), custom_path.to_string());

        let path = socket_path_with("test-service", &env).unwrap();
        assert_eq!(path.to_str().unwrap(), custom_path);
    }

    #[test]
    fn test_socket_path_fallback() {
        let env: HashMap<String, String> = HashMap::new();

        let path = socket_path_with("unknown-service", &env).unwrap();
        assert!(path.to_str().unwrap().ends_with("unknown-service.sock"));
    }

    #[test]
    fn test_socket_path_with_socket_dir() {
        // Use unique service name to avoid env var collisions
        let unique_svc = "socket-dir-test-83726";
        let mut env = HashMap::new();
        env.insert("BIOMEOS_SOCKET_DIR".to_string(), "/run/biomeos".to_string());

        let path = socket_path_with(unique_svc, &env).unwrap();
        let path_str = path.to_str().unwrap();
        assert_eq!(path_str, format!("/run/biomeos/{unique_svc}.sock"));
    }

    #[test]
    fn test_socket_path_env_var_takes_precedence() {
        // Both env var and socket dir set - env var should win
        let mut env = HashMap::new();
        env.insert(
            "PRECEDENCE_TEST_SOCKET".to_string(),
            "/explicit/socket.sock".to_string(),
        );
        env.insert("BIOMEOS_SOCKET_DIR".to_string(), "/run/biomeos".to_string());

        let path = socket_path_with("precedence-test", &env).unwrap();
        assert_eq!(path.to_str().unwrap(), "/explicit/socket.sock");
    }

    #[test]
    fn test_socket_path_normalizes_hyphens() {
        // Hyphens should be converted to underscores in env var name
        let mut env = HashMap::new();
        env.insert(
            "NEURAL_API_SOCKET".to_string(),
            "/test/neural-api.sock".to_string(),
        );

        let path = socket_path_with("neural-api", &env).unwrap();
        assert_eq!(path.to_str().unwrap(), "/test/neural-api.sock");
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
        let env: HashMap<String, String> = HashMap::new();
        let config = RuntimeConfig::with_socket_dir("/test");

        assert!(config.neural_api_socket_with(&env).starts_with("/test"));
        assert!(
            config
                .service_socket_with("beardog", &env)
                .starts_with("/test")
        );
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
        let mut env = HashMap::new();
        env.insert(
            "BIOMEOS_SOCKET_DIR".to_string(),
            "/custom/socket/dir".to_string(),
        );
        let config = RuntimeConfig::from_env_with_map(&env, None, None);

        assert!(
            config
                .neural_api_socket_with(&env)
                .starts_with("/custom/socket/dir")
        );
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
        assert!(
            config
                .service_socket("toadstool")
                .ends_with("toadstool.sock")
        );
        assert!(
            config
                .service_socket("petaltongue")
                .ends_with("petaltongue.sock")
        );
    }

    #[test]
    fn test_runtime_config_socket_env_override() {
        let mut env = HashMap::new();
        env.insert(
            "BEARDOG_SOCKET".to_string(),
            "/override/beardog.sock".to_string(),
        );
        let config = RuntimeConfig::with_socket_dir("/default");

        let beardog_path = config.service_socket_with("beardog", &env);
        assert_eq!(beardog_path.to_str().unwrap(), "/override/beardog.sock");
    }

    #[test]
    fn test_runtime_config_http_port_default() {
        let env: HashMap<String, String> = HashMap::new();
        let port = RuntimeConfig::http_port_with(&env);

        assert_eq!(port, 8080);
    }

    #[test]
    fn test_runtime_config_http_port_env_override() {
        let mut env = HashMap::new();
        env.insert("HTTP_PORT".to_string(), "9999".to_string());
        let port = RuntimeConfig::http_port_with(&env);

        assert_eq!(port, 9999);
    }

    #[test]
    fn test_runtime_config_mcp_port_fallback() {
        let env: HashMap<String, String> = HashMap::new();
        let port = RuntimeConfig::mcp_port_with(&env);

        assert_eq!(port, 3000);
    }

    #[test]
    fn test_runtime_config_mcp_port_websocket_env() {
        let mut env = HashMap::new();
        env.insert("MCP_WEBSOCKET_PORT".to_string(), "8765".to_string());
        let port = RuntimeConfig::mcp_port_with(&env);

        assert_eq!(port, 8765);
    }

    #[test]
    fn test_runtime_config_bind_address_default() {
        let env: HashMap<String, String> = HashMap::new();
        let addr = RuntimeConfig::bind_address_with(&env);

        assert_eq!(addr, "::1");
    }

    #[test]
    fn test_runtime_config_bind_address_env_override() {
        let test_addr = "192.168.255.254";
        let mut env = HashMap::new();
        env.insert("BIND_ADDRESS".to_string(), test_addr.to_string());

        let addr = RuntimeConfig::bind_address_with(&env);
        assert_eq!(addr, test_addr);
    }

    #[test]
    fn test_runtime_config_service_socket() {
        let env: HashMap<String, String> = HashMap::new();
        let config = RuntimeConfig::with_socket_dir("/run/biomeos");

        let socket = config.service_socket_with("custom-primal", &env);
        assert!(socket.ends_with("custom-primal.sock"));
        assert!(socket.starts_with("/run/biomeos"));
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
    fn test_https_port_default() {
        let env: HashMap<String, String> = HashMap::new();
        assert_eq!(RuntimeConfig::https_port_with(&env), 8443);
    }

    #[test]
    fn test_https_port_env_override() {
        let mut env = HashMap::new();
        env.insert("HTTPS_PORT".to_string(), "9443".to_string());
        assert_eq!(RuntimeConfig::https_port_with(&env), 9443);
    }

    #[test]
    fn test_websocket_port_default() {
        let env: HashMap<String, String> = HashMap::new();
        assert_eq!(RuntimeConfig::websocket_port_with(&env), 8081);
    }

    #[test]
    fn test_websocket_port_env_override() {
        let mut env = HashMap::new();
        env.insert("WEBSOCKET_PORT".to_string(), "9081".to_string());
        assert_eq!(RuntimeConfig::websocket_port_with(&env), 9081);
    }

    #[test]
    fn test_discovery_port_default() {
        let env: HashMap<String, String> = HashMap::new();
        assert_eq!(RuntimeConfig::discovery_port_with(&env), 8001);
    }

    #[test]
    fn test_discovery_port_env_override() {
        let mut env = HashMap::new();
        env.insert("DISCOVERY_PORT".to_string(), "9001".to_string());
        assert_eq!(RuntimeConfig::discovery_port_with(&env), 9001);
    }

    #[test]
    fn test_mcp_port_mcp_env_fallback() {
        let mut env = HashMap::new();
        env.insert("MCP_PORT".to_string(), "4000".to_string());
        assert_eq!(RuntimeConfig::mcp_port_with(&env), 4000);
    }

    #[test]
    fn test_bind_address_biomeos_precedence() {
        let mut env = HashMap::new();
        env.insert("BIOMEOS_BIND_ADDRESS".to_string(), "127.0.0.1".to_string());
        env.insert("BIND_ADDRESS".to_string(), "0.0.0.0".to_string());
        assert_eq!(RuntimeConfig::bind_address_with(&env), "127.0.0.1");
    }

    #[test]
    fn test_runtime_config_default_impl() {
        let config = RuntimeConfig::default();
        assert!(!config.socket_dir().as_os_str().is_empty());
    }

    #[test]
    fn test_neural_api_socket_env_override() {
        let mut env = HashMap::new();
        env.insert(
            "NEURAL_API_SOCKET".to_string(),
            "/custom/neural.sock".to_string(),
        );
        let config = RuntimeConfig::with_socket_dir("/default");
        let path = config.neural_api_socket_with(&env);
        assert_eq!(path.to_str().unwrap(), "/custom/neural.sock");
    }

    #[test]
    fn test_socket_path_empty_service_name() {
        let path = socket_path("");
        assert!(path.is_ok());
        assert!(path.unwrap().to_string_lossy().ends_with(".sock"));
    }

    #[test]
    fn test_http_port_invalid_parse_fallback() {
        let mut env = HashMap::new();
        env.insert("HTTP_PORT".to_string(), "not_a_number".to_string());
        assert_eq!(RuntimeConfig::http_port_with(&env), 8080);
    }

    #[test]
    fn test_runtime_config_from_env_xdg_runtime_dir() {
        let mut env = HashMap::new();
        env.insert(
            "XDG_RUNTIME_DIR".to_string(),
            "/tmp/xdg-test-12345".to_string(),
        );
        let config = RuntimeConfig::from_env_with_map(&env, None, None);
        let socket_dir = config.socket_dir();
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
        let mut env = HashMap::new();
        env.insert(
            "OVERRIDE_SVC_SOCKET".to_string(),
            "/absolute/override.sock".to_string(),
        );
        let config = RuntimeConfig::with_socket_dir("/default/dir");
        let path = config.service_socket_with("override-svc", &env);
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
