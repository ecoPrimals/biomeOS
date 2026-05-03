// SPDX-License-Identifier: AGPL-3.0-or-later
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
/// Last-resort fallback. Production should use:
/// 1. `BIOMEOS_SOCKET_DIR` env var
/// 2. `$XDG_RUNTIME_DIR/biomeos`
/// 3. `/run/user/{uid}/biomeos`
/// 4. `/data/local/tmp/biomeos` (Android)
/// 5. This fallback (development only)
pub const DEFAULT_SOCKET_DIR: &str = "/tmp";

/// Fallback family ID when no env var or seed file is available.
///
/// Used instead of scattering the literal `"default"` across call sites.
/// Overridden by `FAMILY_ID` / `BIOMEOS_FAMILY_ID` or `.family.seed`.
pub const DEFAULT_FAMILY_ID: &str = "default";

/// Default Neural API socket name
pub const DEFAULT_NEURAL_API_SOCKET: &str = "neural-api.sock";

/// Environment variable names for socket paths
pub mod env_vars {
    /// Neural API socket path environment variable
    pub const NEURAL_API_SOCKET: &str = "NEURAL_API_SOCKET";

    /// Security provider (BearDog) socket path environment variable
    pub const BEARDOG_SOCKET: &str = "BEARDOG_SOCKET";

    /// Discovery provider (Songbird) socket path environment variable
    pub const SONGBIRD_SOCKET: &str = "SONGBIRD_SOCKET";

    /// AI bridge (Squirrel) socket path environment variable
    pub const SQUIRREL_SOCKET: &str = "SQUIRREL_SOCKET";

    /// Storage provider (NestGate) socket path environment variable
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
    /// Uses XDG-aware resolution instead of bare `/tmp/`.
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
                    let uid_path = PathBuf::from(format!(
                        "{}/{uid}/{}",
                        crate::constants::runtime_paths::LINUX_RUNTIME_DIR_PREFIX,
                        crate::constants::runtime_paths::BIOMEOS_SUBDIR,
                    ));
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
    /// Defaults to `::1` (IPv6 loopback) for dual-stack support.
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
            .unwrap_or_else(|| crate::constants::endpoints::DEFAULT_LOCALHOST_V6.to_string())
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
