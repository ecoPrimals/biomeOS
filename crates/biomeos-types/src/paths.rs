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

#[derive(Error, Debug)]
pub enum PathError {
    #[error("Failed to create directory: {path}")]
    CreateDirFailed {
        path: String,
        source: std::io::Error,
    },

    #[error("Failed to determine home directory")]
    NoHomeDir,

    #[error("Invalid path: {0}")]
    InvalidPath(String),
}

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
        self.runtime_dir.join(format!("{}.sock", primal_id))
    }

    /// Get PID file path
    pub fn pid_file(&self, service_name: &str) -> PathBuf {
        self.runtime_dir.join(format!("{}.pid", service_name))
    }

    /// Get lock file path
    pub fn lock_file(&self, name: &str) -> PathBuf {
        self.runtime_dir.join(format!("{}.lock", name))
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
        self.data_dir.join(format!("{}.db", name))
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
            .join(format!("{}.seed", family_id))
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
            .join(format!("{}.log", service_name))
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
    fn get_runtime_dir() -> Result<PathBuf> {
        // 1. Try $XDG_RUNTIME_DIR
        if let Ok(xdg_runtime) = env::var("XDG_RUNTIME_DIR") {
            return Ok(PathBuf::from(xdg_runtime).join("biomeos"));
        }

        // 2. Fallback to /tmp/biomeos-$USER
        let username = Self::get_username();
        Ok(env::temp_dir().join(format!("biomeos-{}", username)))
    }

    /// Get XDG data directory
    fn get_data_dir() -> Result<PathBuf> {
        // 1. Try $XDG_DATA_HOME
        if let Ok(xdg_data) = env::var("XDG_DATA_HOME") {
            return Ok(PathBuf::from(xdg_data).join("biomeos"));
        }

        // 2. Try $HOME/.local/share
        if let Ok(home) = env::var("HOME") {
            return Ok(PathBuf::from(home).join(".local/share/biomeos"));
        }

        // 3. Use etcetera (Pure Rust!) as fallback
        use etcetera::base_strategy::{choose_base_strategy, BaseStrategy};
        let strategy = choose_base_strategy().map_err(|_| PathError::NoHomeDir)?;
        Ok(strategy.data_dir().join("biomeos"))
    }

    /// Get XDG config directory
    fn get_config_dir() -> Result<PathBuf> {
        // 1. Try $XDG_CONFIG_HOME
        if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
            return Ok(PathBuf::from(xdg_config).join("biomeos"));
        }

        // 2. Try $HOME/.config
        if let Ok(home) = env::var("HOME") {
            return Ok(PathBuf::from(home).join(".config/biomeos"));
        }

        // 3. Use etcetera (Pure Rust!) as fallback
        use etcetera::base_strategy::{choose_base_strategy, BaseStrategy};
        let strategy = choose_base_strategy().map_err(|_| PathError::NoHomeDir)?;
        Ok(strategy.config_dir().join("biomeos"))
    }

    /// Get XDG cache directory
    fn get_cache_dir() -> Result<PathBuf> {
        // 1. Try $XDG_CACHE_HOME
        if let Ok(xdg_cache) = env::var("XDG_CACHE_HOME") {
            return Ok(PathBuf::from(xdg_cache).join("biomeos"));
        }

        // 2. Try $HOME/.cache
        if let Ok(home) = env::var("HOME") {
            return Ok(PathBuf::from(home).join(".cache/biomeos"));
        }

        // 3. Use etcetera (Pure Rust!) as fallback
        use etcetera::base_strategy::{choose_base_strategy, BaseStrategy};
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
}
