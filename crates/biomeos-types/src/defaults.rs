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
pub const DEFAULT_SOCKET_DIR: &str = "/tmp";

/// Default Neural API socket name
pub const DEFAULT_NEURAL_API_SOCKET: &str = "neural-api.sock";

/// Default BearDog socket name
pub const DEFAULT_BEARDOG_SOCKET: &str = "beardog.sock";

/// Default Songbird socket name
pub const DEFAULT_SONGBIRD_SOCKET: &str = "songbird.sock";

/// Default Squirrel socket name
pub const DEFAULT_SQUIRREL_SOCKET: &str = "squirrel.sock";

/// Default NestGate socket name
pub const DEFAULT_NESTGATE_SOCKET: &str = "nestgate.sock";

/// Default ToadStool socket name
pub const DEFAULT_TOADSTOOL_SOCKET: &str = "toadstool.sock";

/// Default PetalTongue socket name
pub const DEFAULT_PETALTONGUE_SOCKET: &str = "petaltongue.sock";

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
        return Ok(PathBuf::from(socket_dir).join(format!("{}.sock", service)));
    }

    // 3. Fallback to default
    Ok(PathBuf::from(DEFAULT_SOCKET_DIR).join(format!("{}.sock", service)))
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
/// let beardog_socket = config.beardog_socket();
/// ```
#[derive(Debug, Clone)]
pub struct RuntimeConfig {
    socket_dir: PathBuf,
}

impl RuntimeConfig {
    /// Create RuntimeConfig from environment variables
    pub fn from_env() -> Self {
        let socket_dir = env::var(env_vars::SOCKET_DIR)
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_SOCKET_DIR));

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

    /// Get BearDog socket path
    pub fn beardog_socket(&self) -> PathBuf {
        env::var(env_vars::BEARDOG_SOCKET)
            .map(PathBuf::from)
            .unwrap_or_else(|_| self.socket_dir.join(DEFAULT_BEARDOG_SOCKET))
    }

    /// Get Songbird socket path
    pub fn songbird_socket(&self) -> PathBuf {
        env::var(env_vars::SONGBIRD_SOCKET)
            .map(PathBuf::from)
            .unwrap_or_else(|_| self.socket_dir.join(DEFAULT_SONGBIRD_SOCKET))
    }

    /// Get Squirrel socket path
    pub fn squirrel_socket(&self) -> PathBuf {
        env::var(env_vars::SQUIRREL_SOCKET)
            .map(PathBuf::from)
            .unwrap_or_else(|_| self.socket_dir.join(DEFAULT_SQUIRREL_SOCKET))
    }

    /// Get NestGate socket path
    pub fn nestgate_socket(&self) -> PathBuf {
        env::var(env_vars::NESTGATE_SOCKET)
            .map(PathBuf::from)
            .unwrap_or_else(|_| self.socket_dir.join(DEFAULT_NESTGATE_SOCKET))
    }

    /// Get ToadStool socket path
    pub fn toadstool_socket(&self) -> PathBuf {
        env::var(env_vars::TOADSTOOL_SOCKET)
            .map(PathBuf::from)
            .unwrap_or_else(|_| self.socket_dir.join(DEFAULT_TOADSTOOL_SOCKET))
    }

    /// Get PetalTongue socket path
    pub fn petaltongue_socket(&self) -> PathBuf {
        env::var(env_vars::PETALTONGUE_SOCKET)
            .map(PathBuf::from)
            .unwrap_or_else(|_| self.socket_dir.join(DEFAULT_PETALTONGUE_SOCKET))
    }

    /// Get socket path for any service by name
    pub fn service_socket(&self, service: &str) -> PathBuf {
        socket_path(service).unwrap_or_else(|_| self.socket_dir.join(format!("{}.sock", service)))
    }

    /// Get socket directory
    pub fn socket_dir(&self) -> &Path {
        &self.socket_dir
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
    dir.as_ref().join(format!("{}.sock", service))
}

#[cfg(test)]
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
    fn test_runtime_config() {
        // Clear environment variables to test default behavior
        env::remove_var("NEURAL_API_SOCKET");
        env::remove_var("BEARDOG_SOCKET");
        
        let config = RuntimeConfig::with_socket_dir("/test");

        assert!(config.neural_api_socket().starts_with("/test"));
        assert!(config.beardog_socket().starts_with("/test"));
    }

    #[test]
    fn test_join_socket_path() {
        let path = join_socket_path("/run", "test");
        assert_eq!(path.to_str().unwrap(), "/run/test.sock");
    }
}
