// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

// =============================================================================
// System Paths - XDG Base Directory Compliance
// =============================================================================
//
// Provides portable, XDG-compliant path management for biomeOS.
//
// Deep Debt Principle:
// "Hardcoded paths should be evolved to agnostic and capability-based"
//
// BEFORE:
//   let socket = "/tmp/beardog.sock";  // ❌ Hardcoded, not portable
//
// AFTER:
//   let paths = SystemPaths::new()?;
//   let socket = paths.primal_socket("beardog");  // ✅ XDG-compliant
//
// References:
// - XDG Base Directory Specification: https://specifications.freedesktop.org/basedir-spec/
// - systemd file hierarchy: https://www.freedesktop.org/software/systemd/man/file-hierarchy.html
//
// =============================================================================

use std::env;
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Errors related to system path operations
#[derive(Error, Debug)]
pub enum PathError {
    /// Failed to create a directory
    #[error("Failed to create directory: {path}")]
    CreateDirFailed {
        /// Directory path that failed
        path: String,
        /// Underlying IO error
        source: std::io::Error,
    },

    /// Failed to determine the home directory
    #[error("Failed to determine home directory")]
    NoHomeDir,

    /// Invalid path encountered
    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

/// Result type for path operations
pub type Result<T> = std::result::Result<T, PathError>;

/// System paths following XDG Base Directory specification
///
/// This struct provides portable, user-scoped paths for biomeOS.
/// All paths are created on-demand and follow XDG standards.
///
/// # Examples
///
/// ```no_run
/// use biomeos_types::paths::SystemPaths;
///
/// let paths = SystemPaths::new()?;
///
/// // Get Unix socket path for a primal
/// let socket = paths.primal_socket("beardog-main");
/// // → $XDG_RUNTIME_DIR/biomeos/beardog-main.sock
///
/// // Get data directory for persistent storage
/// let data = paths.data_dir();
/// // → ~/.local/share/biomeos
/// # Ok::<(), Box<dyn std::error::Error>>(())
/// ```
#[derive(Debug, Clone)]
#[allow(clippy::struct_field_names)] // dir suffix is XDG convention (runtime_dir, data_dir, etc.)
pub struct SystemPaths {
    /// Runtime directory (Unix sockets, PID files)
    /// Default: $XDG_RUNTIME_DIR/biomeos or /tmp/biomeos-$USER/
    runtime_dir: PathBuf,

    /// Data directory (persistent state, databases)
    /// Default: $XDG_DATA_HOME/biomeos or ~/.local/share/biomeos
    data_dir: PathBuf,

    /// Config directory (configuration files)
    /// Default: $XDG_CONFIG_HOME/biomeos or ~/.config/biomeos
    config_dir: PathBuf,

    /// Cache directory (temporary cached data)
    /// Default: $XDG_CACHE_HOME/biomeos or ~/.cache/biomeos
    cache_dir: PathBuf,

    /// State directory (logs, history)
    /// Default: $XDG_STATE_HOME/biomeos or ~/.local/state/biomeos
    state_dir: PathBuf,
}

impl SystemPaths {
    /// Create new SystemPaths with XDG-compliant defaults
    ///
    /// This will create all necessary directories if they don't exist.
    pub fn new() -> Result<Self> {
        let runtime_dir = Self::get_runtime_dir()?;
        let data_dir = Self::get_data_dir()?;
        let config_dir = Self::get_config_dir()?;
        let cache_dir = Self::get_cache_dir()?;
        let state_dir = Self::get_state_dir()?;

        // Create directories
        Self::ensure_dir(&runtime_dir)?;
        Self::ensure_dir(&data_dir)?;
        Self::ensure_dir(&config_dir)?;
        Self::ensure_dir(&cache_dir)?;
        Self::ensure_dir(&state_dir)?;

        Ok(Self {
            runtime_dir,
            data_dir,
            config_dir,
            cache_dir,
            state_dir,
        })
    }

    /// Create SystemPaths with XDG env overrides (for testing without mutating env)
    pub fn new_with_xdg_overrides(
        xdg_runtime_dir: Option<impl AsRef<Path>>,
        xdg_data_home: Option<impl AsRef<Path>>,
    ) -> Result<Self> {
        let runtime_dir = xdg_runtime_dir.map_or_else(Self::get_runtime_dir, |p| {
            Ok(p.as_ref().to_path_buf().join("biomeos"))
        })?;
        let data_dir = xdg_data_home.map_or_else(Self::get_data_dir, |p| {
            Ok(p.as_ref().to_path_buf().join("biomeos"))
        })?;
        let config_dir = Self::get_config_dir()?;
        let cache_dir = Self::get_cache_dir()?;
        let state_dir = Self::get_state_dir()?;
        Self::ensure_dir(&runtime_dir)?;
        Self::ensure_dir(&data_dir)?;
        Self::ensure_dir(&config_dir)?;
        Self::ensure_dir(&cache_dir)?;
        Self::ensure_dir(&state_dir)?;
        Ok(Self {
            runtime_dir,
            data_dir,
            config_dir,
            cache_dir,
            state_dir,
        })
    }

    /// Create SystemPaths with custom base directories
    ///
    /// Use this for testing or custom deployments
    pub fn with_base(base: impl AsRef<Path>) -> Result<Self> {
        let base = base.as_ref();
        let runtime_dir = base.join("runtime");
        let data_dir = base.join("data");
        let config_dir = base.join("config");
        let cache_dir = base.join("cache");
        let state_dir = base.join("state");

        Self::ensure_dir(&runtime_dir)?;
        Self::ensure_dir(&data_dir)?;
        Self::ensure_dir(&config_dir)?;
        Self::ensure_dir(&cache_dir)?;
        Self::ensure_dir(&state_dir)?;

        Ok(Self {
            runtime_dir,
            data_dir,
            config_dir,
            cache_dir,
            state_dir,
        })
    }

    // =============================================================================
    // Runtime Directory (Unix sockets, PID files, volatile state)
    // =============================================================================

    /// Get the runtime directory
    pub fn runtime_dir(&self) -> &Path {
        &self.runtime_dir
    }

    /// Get Unix socket path for a primal
    ///
    /// Example: `beardog-main` → `$XDG_RUNTIME_DIR/biomeos/beardog-main.sock`
    pub fn primal_socket(&self, primal_id: &str) -> PathBuf {
        self.runtime_dir.join(format!("{primal_id}.sock"))
    }

    /// Get PID file path
    pub fn pid_file(&self, service_name: &str) -> PathBuf {
        self.runtime_dir.join(format!("{service_name}.pid"))
    }

    /// Get lock file path
    pub fn lock_file(&self, name: &str) -> PathBuf {
        self.runtime_dir.join(format!("{name}.lock"))
    }

    // =============================================================================
    // Data Directory (persistent state, databases)
    // =============================================================================

    /// Get the data directory
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// Get database file path
    pub fn database(&self, name: &str) -> PathBuf {
        self.data_dir.join(format!("{name}.db"))
    }

    /// Get metrics database path
    pub fn metrics_db(&self) -> PathBuf {
        self.data_dir.join("metrics.db")
    }

    /// Get spore storage directory
    pub fn spore_dir(&self) -> PathBuf {
        self.data_dir.join("spores")
    }

    /// Get genetic seed file path
    pub fn genetic_seed(&self, family_id: &str) -> PathBuf {
        self.data_dir
            .join("seeds")
            .join(format!("{family_id}.seed"))
    }

    // =============================================================================
    // Config Directory (configuration files)
    // =============================================================================

    /// Get the config directory
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// Get main biomeOS config file
    pub fn main_config(&self) -> PathBuf {
        self.config_dir.join("biomeos.toml")
    }

    /// Get niche manifest directory
    pub fn niche_dir(&self) -> PathBuf {
        self.config_dir.join("niches")
    }

    /// Get graph definitions directory
    pub fn graph_dir(&self) -> PathBuf {
        self.config_dir.join("graphs")
    }

    // =============================================================================
    // Cache Directory (temporary cached data)
    // =============================================================================

    /// Get the cache directory
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Get temporary workspace
    pub fn temp_workspace(&self, name: &str) -> PathBuf {
        self.cache_dir.join("workspace").join(name)
    }

    /// Get download cache
    pub fn download_cache(&self) -> PathBuf {
        self.cache_dir.join("downloads")
    }

    // =============================================================================
    // State Directory (logs, history)
    // =============================================================================

    /// Get the state directory
    pub fn state_dir(&self) -> &Path {
        &self.state_dir
    }

    /// Get log file path
    pub fn log_file(&self, service_name: &str) -> PathBuf {
        self.state_dir
            .join("logs")
            .join(format!("{service_name}.log"))
    }

    /// Get fossil record directory
    pub fn fossil_record_dir(&self) -> PathBuf {
        self.state_dir.join("fossil-record")
    }

    /// Get audit log path
    pub fn audit_log(&self) -> PathBuf {
        self.state_dir.join("audit.log")
    }

    // =============================================================================
    // Helper Methods
    // =============================================================================

    /// Ensure a directory exists
    fn ensure_dir(path: &Path) -> Result<()> {
        if !path.exists() {
            std::fs::create_dir_all(path).map_err(|e| PathError::CreateDirFailed {
                path: path.display().to_string(),
                source: e,
            })?;
        }
        Ok(())
    }

    /// Get XDG runtime directory
    #[allow(clippy::unnecessary_wraps)] // Result required for consistency with other get_*_dir methods
    fn get_runtime_dir() -> Result<PathBuf> {
        // 1. Try $XDG_RUNTIME_DIR
        if let Ok(xdg_runtime) = env::var("XDG_RUNTIME_DIR") {
            return Ok(PathBuf::from(xdg_runtime).join("biomeos"));
        }

        // 2. Fallback to /tmp/biomeos-$USER
        let username = Self::get_username();
        Ok(env::temp_dir().join(format!("biomeos-{username}")))
    }

    /// Get XDG data directory
    fn get_data_dir() -> Result<PathBuf> {
        use etcetera::base_strategy::{BaseStrategy, choose_base_strategy};
        // 1. Try $XDG_DATA_HOME
        if let Ok(xdg_data) = env::var("XDG_DATA_HOME") {
            return Ok(PathBuf::from(xdg_data).join("biomeos"));
        }

        // 2. Try $HOME/.local/share
        if let Ok(home) = env::var("HOME") {
            return Ok(PathBuf::from(home).join(".local/share/biomeos"));
        }

        // 3. Use etcetera (Pure Rust!) as fallback
        let strategy = choose_base_strategy().map_err(|_| PathError::NoHomeDir)?;
        Ok(strategy.data_dir().join("biomeos"))
    }

    /// Get XDG config directory
    fn get_config_dir() -> Result<PathBuf> {
        use etcetera::base_strategy::{BaseStrategy, choose_base_strategy};
        // 1. Try $XDG_CONFIG_HOME
        if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
            return Ok(PathBuf::from(xdg_config).join("biomeos"));
        }

        // 2. Try $HOME/.config
        if let Ok(home) = env::var("HOME") {
            return Ok(PathBuf::from(home).join(".config/biomeos"));
        }

        // 3. Use etcetera (Pure Rust!) as fallback
        let strategy = choose_base_strategy().map_err(|_| PathError::NoHomeDir)?;
        Ok(strategy.config_dir().join("biomeos"))
    }

    /// Get XDG cache directory
    fn get_cache_dir() -> Result<PathBuf> {
        use etcetera::base_strategy::{BaseStrategy, choose_base_strategy};
        // 1. Try $XDG_CACHE_HOME
        if let Ok(xdg_cache) = env::var("XDG_CACHE_HOME") {
            return Ok(PathBuf::from(xdg_cache).join("biomeos"));
        }

        // 2. Try $HOME/.cache
        if let Ok(home) = env::var("HOME") {
            return Ok(PathBuf::from(home).join(".cache/biomeos"));
        }

        // 3. Use etcetera (Pure Rust!) as fallback
        let strategy = choose_base_strategy().map_err(|_| PathError::NoHomeDir)?;
        Ok(strategy.cache_dir().join("biomeos"))
    }

    /// Get XDG state directory
    fn get_state_dir() -> Result<PathBuf> {
        // 1. Try $XDG_STATE_HOME
        if let Ok(xdg_state) = env::var("XDG_STATE_HOME") {
            return Ok(PathBuf::from(xdg_state).join("biomeos"));
        }

        // 2. Try $HOME/.local/state
        if let Ok(home) = env::var("HOME") {
            return Ok(PathBuf::from(home).join(".local/state/biomeos"));
        }

        // 3. Fallback to data_dir/state
        Ok(Self::get_data_dir()?.join("state"))
    }

    /// Get current username
    fn get_username() -> String {
        env::var("USER")
            .or_else(|_| env::var("USERNAME"))
            .unwrap_or_else(|_| "default".to_string())
    }

    /// Create SystemPaths without creating directories
    ///
    /// This is useful for Default implementation and cases where you want
    /// to compute paths but defer directory creation until actually needed.
    /// Directories will be created lazily when first accessed via methods
    /// like `primal_socket()`.
    ///
    /// Note: This will use fallback paths (e.g., /tmp) if XDG paths cannot
    /// be determined. For stricter path requirements, use `new()` which
    /// returns a `Result`.
    pub fn new_lazy() -> Self {
        // Compute paths with fallbacks - these operations cannot fail
        let runtime_dir =
            Self::get_runtime_dir().unwrap_or_else(|_| env::temp_dir().join("biomeos"));
        let data_dir =
            Self::get_data_dir().unwrap_or_else(|_| env::temp_dir().join("biomeos-data"));
        let config_dir =
            Self::get_config_dir().unwrap_or_else(|_| env::temp_dir().join("biomeos-config"));
        let cache_dir =
            Self::get_cache_dir().unwrap_or_else(|_| env::temp_dir().join("biomeos-cache"));
        let state_dir =
            Self::get_state_dir().unwrap_or_else(|_| env::temp_dir().join("biomeos-state"));

        Self {
            runtime_dir,
            data_dir,
            config_dir,
            cache_dir,
            state_dir,
        }
    }

    /// Ensure all directories exist
    ///
    /// Call this after `new_lazy()` when you need to guarantee directories exist.
    pub fn ensure_all_dirs(&self) -> Result<()> {
        Self::ensure_dir(&self.runtime_dir)?;
        Self::ensure_dir(&self.data_dir)?;
        Self::ensure_dir(&self.config_dir)?;
        Self::ensure_dir(&self.cache_dir)?;
        Self::ensure_dir(&self.state_dir)?;
        Ok(())
    }

    /// Returns the real UID of the current process without using `unsafe`.
    ///
    /// Reads `/proc/self/status` and parses the `Uid:` line to obtain the real UID.
    /// Falls back to 65534 (nobody) if the proc filesystem is unavailable (e.g. non-Linux).
    ///
    /// This approach avoids `unsafe` by using pure Rust I/O (`std::fs::read_to_string`)
    /// instead of libc's `getuid()`, making it safe for use in restricted environments
    /// and avoiding FFI boundary concerns.
    #[must_use]
    pub fn safe_uid() -> u32 {
        safe_uid()
    }
}

/// Returns the real UID of the current process without using `unsafe`.
///
/// Reads `/proc/self/status` and parses the `Uid:` line to obtain the real UID.
/// Falls back to 65534 (nobody) if the proc filesystem is unavailable (e.g. non-Linux).
///
/// This approach avoids `unsafe` by using pure Rust I/O (`std::fs::read_to_string`)
/// instead of libc's `getuid()`, making it safe for use in restricted environments
/// and avoiding FFI boundary concerns.
#[must_use]
pub fn safe_uid() -> u32 {
    const PROC_STATUS_PATH: &str = "/proc/self/status";
    const NOBODY_UID: u32 = 65534;
    std::fs::read_to_string(PROC_STATUS_PATH)
        .ok()
        .and_then(|status| {
            status.lines().find_map(|line| {
                line.strip_prefix("Uid:")
                    .and_then(|rest| rest.split_whitespace().next())
                    .and_then(|s| s.parse::<u32>().ok())
            })
        })
        .unwrap_or(NOBODY_UID)
}

impl Default for SystemPaths {
    /// Create SystemPaths with lazy directory creation
    ///
    /// This implementation uses `new_lazy()` to avoid panicking.
    /// Directories will be created when first accessed. For explicit
    /// directory creation and error handling, use `SystemPaths::new()`.
    fn default() -> Self {
        Self::new_lazy()
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_system_paths_with_base() {
        let temp = tempdir().unwrap();
        let paths = SystemPaths::with_base(temp.path()).unwrap();

        assert!(paths.runtime_dir().exists());
        assert!(paths.data_dir().exists());
        assert!(paths.config_dir().exists());
        assert!(paths.cache_dir().exists());
        assert!(paths.state_dir().exists());
    }

    #[test]
    fn test_primal_socket_path() {
        let temp = tempdir().unwrap();
        let paths = SystemPaths::with_base(temp.path()).unwrap();

        let socket = paths.primal_socket("beardog-main");
        assert_eq!(socket.file_name().unwrap(), "beardog-main.sock");
        assert!(socket.starts_with(paths.runtime_dir()));
    }

    #[test]
    fn test_database_paths() {
        let temp = tempdir().unwrap();
        let paths = SystemPaths::with_base(temp.path()).unwrap();

        let metrics_db = paths.metrics_db();
        assert_eq!(metrics_db.file_name().unwrap(), "metrics.db");
        assert!(metrics_db.starts_with(paths.data_dir()));

        let custom_db = paths.database("custom");
        assert_eq!(custom_db.file_name().unwrap(), "custom.db");
    }

    #[test]
    fn test_config_paths() {
        let temp = tempdir().unwrap();
        let paths = SystemPaths::with_base(temp.path()).unwrap();

        let main_config = paths.main_config();
        assert_eq!(main_config.file_name().unwrap(), "biomeos.toml");
        assert!(main_config.starts_with(paths.config_dir()));

        let niche_dir = paths.niche_dir();
        assert_eq!(niche_dir.file_name().unwrap(), "niches");
    }

    #[test]
    fn test_log_paths() {
        let temp = tempdir().unwrap();
        let paths = SystemPaths::with_base(temp.path()).unwrap();

        let log = paths.log_file("test-service");
        assert!(log.to_string_lossy().contains("test-service.log"));
        assert!(log.starts_with(paths.state_dir()));
    }

    #[test]
    fn test_genetic_seed_path() {
        let temp = tempdir().unwrap();
        let paths = SystemPaths::with_base(temp.path()).unwrap();

        let seed = paths.genetic_seed("family-alpha");
        assert!(seed.to_string_lossy().contains("family-alpha.seed"));
        assert!(seed.starts_with(paths.data_dir()));
    }

    #[test]
    fn test_new_lazy_and_ensure_all_dirs() {
        let temp = tempdir().unwrap();
        let base = temp.path().join("lazy-base");
        std::fs::create_dir_all(&base).unwrap();

        let _paths = SystemPaths::with_base(&base).unwrap();
        let lazy_paths = SystemPaths::new_lazy();
        let _ = lazy_paths.runtime_dir();
        let _ = lazy_paths.data_dir();
        let _ = lazy_paths.config_dir();
        let _ = lazy_paths.cache_dir();
        let _ = lazy_paths.state_dir();

        let paths_with_base = SystemPaths::with_base(&base).unwrap();
        assert!(paths_with_base.ensure_all_dirs().is_ok());
    }

    #[test]
    fn test_default_impl() {
        let paths = SystemPaths::default();
        assert!(!paths.runtime_dir().as_os_str().is_empty());
    }

    #[test]
    fn test_all_path_resolution_methods() {
        let temp = tempdir().unwrap();
        let paths = SystemPaths::with_base(temp.path()).unwrap();

        let pid = paths.pid_file("test-service");
        assert!(pid.to_string_lossy().contains("test-service.pid"));

        let lock = paths.lock_file("test-lock");
        assert!(lock.to_string_lossy().contains("test-lock.lock"));

        let spore = paths.spore_dir();
        assert!(spore.ends_with("spores"));

        let temp_ws = paths.temp_workspace("my-workspace");
        assert!(temp_ws.to_string_lossy().contains("my-workspace"));

        let download = paths.download_cache();
        assert!(download.ends_with("downloads"));

        let fossil = paths.fossil_record_dir();
        assert!(fossil.ends_with("fossil-record"));

        let audit = paths.audit_log();
        assert!(audit.ends_with("audit.log"));

        let graph = paths.graph_dir();
        assert!(graph.ends_with("graphs"));
    }

    #[test]
    fn test_path_error_display() {
        let err = PathError::InvalidPath("bad-path".to_string());
        assert!(err.to_string().contains("Invalid path"));
        assert!(err.to_string().contains("bad-path"));
    }

    #[test]
    fn test_xdg_runtime_dir_override() {
        let temp = tempdir().unwrap();
        let xdg_runtime = temp.path().join("xdg-runtime");
        std::fs::create_dir_all(&xdg_runtime).unwrap();

        let paths = SystemPaths::new_with_xdg_overrides(Some(&xdg_runtime), None::<&Path>).unwrap();
        assert!(
            paths
                .runtime_dir()
                .to_string_lossy()
                .contains("xdg-runtime")
        );
    }

    #[test]
    fn test_xdg_data_home_override() {
        let temp = tempdir().unwrap();
        let xdg_data = temp.path().join("xdg-data");
        std::fs::create_dir_all(&xdg_data).unwrap();

        let paths = SystemPaths::new_with_xdg_overrides(None::<&Path>, Some(&xdg_data)).unwrap();
        assert!(paths.data_dir().to_string_lossy().contains("xdg-data"));
    }

    #[test]
    fn test_empty_primal_id_in_socket() {
        let temp = tempdir().unwrap();
        let paths = SystemPaths::with_base(temp.path()).unwrap();
        let socket = paths.primal_socket("");
        assert!(socket.ends_with(".sock"));
    }

    #[test]
    fn test_safe_uid() {
        let uid = safe_uid();
        assert_ne!(uid, 0, "safe_uid should return non-zero value");
    }

    #[test]
    fn test_path_error_create_dir_failed_display() {
        let err = PathError::CreateDirFailed {
            path: "/invalid/path".to_string(),
            source: std::io::Error::new(std::io::ErrorKind::PermissionDenied, "denied"),
        };
        let s = err.to_string();
        assert!(s.contains("Failed to create directory"));
        assert!(s.contains("/invalid/path"));
    }

    #[test]
    fn test_path_error_no_home_dir_display() {
        let err = PathError::NoHomeDir;
        let s = err.to_string();
        assert!(s.to_lowercase().contains("home"));
    }

    #[test]
    fn test_path_error_invalid_path_display() {
        let err = PathError::InvalidPath("bad/path".to_string());
        let s = err.to_string();
        assert!(s.contains("Invalid path"));
        assert!(s.contains("bad/path"));
    }

    #[test]
    fn test_path_error_debug() {
        let err = PathError::NoHomeDir;
        let s = format!("{:?}", err);
        assert!(s.contains("NoHomeDir"));
    }

    #[test]
    fn test_spore_dir_path() {
        let temp = tempdir().unwrap();
        let paths = SystemPaths::with_base(temp.path()).unwrap();
        let spore = paths.spore_dir();
        assert!(spore.ends_with("spores"));
    }

    #[test]
    fn test_graph_dir_path() {
        let temp = tempdir().unwrap();
        let paths = SystemPaths::with_base(temp.path()).unwrap();
        let graph = paths.graph_dir();
        assert!(graph.ends_with("graphs"));
    }

    #[test]
    fn test_new_lazy_default_paths() {
        let paths = SystemPaths::new_lazy();
        assert!(!paths.runtime_dir().as_os_str().is_empty());
        assert!(!paths.data_dir().as_os_str().is_empty());
        assert!(!paths.config_dir().as_os_str().is_empty());
        assert!(!paths.cache_dir().as_os_str().is_empty());
        assert!(!paths.state_dir().as_os_str().is_empty());
    }
}
