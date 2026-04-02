// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

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

use crate::primal_names;

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
#[expect(
    clippy::struct_field_names,
    reason = "dir suffix is XDG convention (runtime_dir, data_dir, etc.)"
)]
pub struct SystemPaths {
    /// Runtime directory (Unix sockets, PID files)
    /// Default: $`XDG_RUNTIME_DIR/biomeos` or /tmp/biomeos-$USER/
    runtime_dir: PathBuf,

    /// Data directory (persistent state, databases)
    /// Default: $`XDG_DATA_HOME/biomeos` or ~/.local/share/biomeos
    data_dir: PathBuf,

    /// Config directory (configuration files)
    /// Default: $`XDG_CONFIG_HOME/biomeos` or ~/.config/biomeos
    config_dir: PathBuf,

    /// Cache directory (temporary cached data)
    /// Default: $`XDG_CACHE_HOME/biomeos` or ~/.cache/biomeos
    cache_dir: PathBuf,

    /// State directory (logs, history)
    /// Default: $`XDG_STATE_HOME/biomeos` or ~/.local/state/biomeos
    state_dir: PathBuf,
}

impl SystemPaths {
    /// Create new `SystemPaths` with XDG-compliant defaults
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

    /// Create `SystemPaths` from explicit directory paths (no environment reads).
    ///
    /// Each argument must be the full biomeOS leaf directory (e.g. `$XDG_RUNTIME_DIR/biomeos`),
    /// matching what `new()` would construct from XDG variables.
    pub fn from_overrides(
        runtime_dir: PathBuf,
        data_dir: PathBuf,
        config_dir: PathBuf,
        cache_dir: PathBuf,
        state_dir: PathBuf,
    ) -> Result<Self> {
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

    /// Returns the biomeOS runtime directory under an XDG-style parent (`$XDG_RUNTIME_DIR`),
    /// or the `/tmp/biomeos-$USER`-style fallback when `runtime_parent` is `None`.
    ///
    /// Does not read `XDG_RUNTIME_DIR`; callers pass the parent explicitly or use `None` for the
    /// same fallback as `get_runtime_dir` when that variable is unset.
    #[must_use]
    pub fn runtime_dir_from_xdg_parent(runtime_parent: Option<&Path>) -> PathBuf {
        if let Some(p) = runtime_parent {
            p.join(primal_names::BIOMEOS)
        } else {
            let username = Self::get_username();
            env::temp_dir().join(format!("{}-{}", primal_names::BIOMEOS, username))
        }
    }

    /// Create `SystemPaths` with XDG env overrides (for testing without mutating env)
    pub fn new_with_xdg_overrides(
        xdg_runtime_dir: Option<impl AsRef<Path>>,
        xdg_data_home: Option<impl AsRef<Path>>,
    ) -> Result<Self> {
        let runtime_dir = xdg_runtime_dir.map_or_else(Self::get_runtime_dir, |p| {
            Ok(p.as_ref().to_path_buf().join(primal_names::BIOMEOS))
        })?;
        let data_dir = xdg_data_home.map_or_else(Self::get_data_dir, |p| {
            Ok(p.as_ref().to_path_buf().join(primal_names::BIOMEOS))
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

    /// Create `SystemPaths` with custom base directories
    ///
    /// Use this for testing or custom deployments
    ///
    /// # Examples
    ///
    /// ```
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// use biomeos_types::paths::SystemPaths;
    /// use std::fs;
    ///
    /// let base = std::env::temp_dir().join(format!("biomeos-paths-{}", std::process::id()));
    /// fs::create_dir_all(&base)?;
    /// let paths = SystemPaths::with_base(&base)?;
    /// assert!(paths.primal_socket("beardog").ends_with("beardog.sock"));
    /// # Ok(())
    /// # }
    /// ```
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
    #[must_use]
    pub fn runtime_dir(&self) -> &Path {
        &self.runtime_dir
    }

    /// Get Unix socket path for a primal
    ///
    /// Example: `beardog-main` → `$XDG_RUNTIME_DIR/biomeos/beardog-main.sock`
    #[must_use]
    pub fn primal_socket(&self, primal_id: &str) -> PathBuf {
        self.runtime_dir.join(format!("{primal_id}.sock"))
    }

    /// Get PID file path
    #[must_use]
    pub fn pid_file(&self, service_name: &str) -> PathBuf {
        self.runtime_dir.join(format!("{service_name}.pid"))
    }

    /// Get lock file path
    #[must_use]
    pub fn lock_file(&self, name: &str) -> PathBuf {
        self.runtime_dir.join(format!("{name}.lock"))
    }

    // =============================================================================
    // Data Directory (persistent state, databases)
    // =============================================================================

    /// Get the data directory
    #[must_use]
    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    /// Get database file path
    #[must_use]
    pub fn database(&self, name: &str) -> PathBuf {
        self.data_dir.join(format!("{name}.db"))
    }

    /// Get metrics database path
    #[must_use]
    pub fn metrics_db(&self) -> PathBuf {
        self.data_dir.join("metrics.db")
    }

    /// Get spore storage directory
    #[must_use]
    pub fn spore_dir(&self) -> PathBuf {
        self.data_dir.join("spores")
    }

    /// Get genetic seed file path
    #[must_use]
    pub fn genetic_seed(&self, family_id: &str) -> PathBuf {
        self.data_dir
            .join("seeds")
            .join(format!("{family_id}.seed"))
    }

    // =============================================================================
    // Config Directory (configuration files)
    // =============================================================================

    /// Get the config directory
    #[must_use]
    pub fn config_dir(&self) -> &Path {
        &self.config_dir
    }

    /// Get main biomeOS config file
    #[must_use]
    pub fn main_config(&self) -> PathBuf {
        self.config_dir.join("biomeos.toml")
    }

    /// Get niche manifest directory
    #[must_use]
    pub fn niche_dir(&self) -> PathBuf {
        self.config_dir.join("niches")
    }

    /// Get graph definitions directory
    #[must_use]
    pub fn graph_dir(&self) -> PathBuf {
        self.config_dir.join("graphs")
    }

    // =============================================================================
    // Cache Directory (temporary cached data)
    // =============================================================================

    /// Get the cache directory
    #[must_use]
    pub fn cache_dir(&self) -> &Path {
        &self.cache_dir
    }

    /// Get temporary workspace
    #[must_use]
    pub fn temp_workspace(&self, name: &str) -> PathBuf {
        self.cache_dir.join("workspace").join(name)
    }

    /// Get download cache
    #[must_use]
    pub fn download_cache(&self) -> PathBuf {
        self.cache_dir.join("downloads")
    }

    // =============================================================================
    // State Directory (logs, history)
    // =============================================================================

    /// Get the state directory
    #[must_use]
    pub fn state_dir(&self) -> &Path {
        &self.state_dir
    }

    /// Get log file path
    #[must_use]
    pub fn log_file(&self, service_name: &str) -> PathBuf {
        self.state_dir
            .join("logs")
            .join(format!("{service_name}.log"))
    }

    /// Get fossil record directory
    #[must_use]
    pub fn fossil_record_dir(&self) -> PathBuf {
        self.state_dir.join("fossil-record")
    }

    /// Get audit log path
    #[must_use]
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
    #[expect(
        clippy::unnecessary_wraps,
        reason = "Result required for consistency with other get_*_dir methods"
    )]
    fn get_runtime_dir() -> Result<PathBuf> {
        // 1. Try $XDG_RUNTIME_DIR
        if let Ok(xdg_runtime) = env::var("XDG_RUNTIME_DIR") {
            return Ok(PathBuf::from(xdg_runtime).join(primal_names::BIOMEOS));
        }

        // 2. Fallback to /tmp/biomeos-$USER
        let username = Self::get_username();
        Ok(env::temp_dir().join(format!("{}-{}", primal_names::BIOMEOS, username)))
    }

    /// Get XDG data directory
    fn get_data_dir() -> Result<PathBuf> {
        use etcetera::base_strategy::{BaseStrategy, choose_base_strategy};
        // 1. Try $XDG_DATA_HOME
        if let Ok(xdg_data) = env::var("XDG_DATA_HOME") {
            return Ok(PathBuf::from(xdg_data).join(primal_names::BIOMEOS));
        }

        // 2. Try $HOME/.local/share
        if let Ok(home) = env::var("HOME") {
            return Ok(PathBuf::from(home)
                .join(".local/share")
                .join(primal_names::BIOMEOS));
        }

        // 3. Use etcetera (Pure Rust!) as fallback
        let strategy = choose_base_strategy().map_err(|_| PathError::NoHomeDir)?;
        Ok(strategy.data_dir().join(primal_names::BIOMEOS))
    }

    /// Get XDG config directory
    fn get_config_dir() -> Result<PathBuf> {
        use etcetera::base_strategy::{BaseStrategy, choose_base_strategy};
        // 1. Try $XDG_CONFIG_HOME
        if let Ok(xdg_config) = env::var("XDG_CONFIG_HOME") {
            return Ok(PathBuf::from(xdg_config).join(primal_names::BIOMEOS));
        }

        // 2. Try $HOME/.config
        if let Ok(home) = env::var("HOME") {
            return Ok(PathBuf::from(home)
                .join(".config")
                .join(primal_names::BIOMEOS));
        }

        // 3. Use etcetera (Pure Rust!) as fallback
        let strategy = choose_base_strategy().map_err(|_| PathError::NoHomeDir)?;
        Ok(strategy.config_dir().join(primal_names::BIOMEOS))
    }

    /// Get XDG cache directory
    fn get_cache_dir() -> Result<PathBuf> {
        use etcetera::base_strategy::{BaseStrategy, choose_base_strategy};
        // 1. Try $XDG_CACHE_HOME
        if let Ok(xdg_cache) = env::var("XDG_CACHE_HOME") {
            return Ok(PathBuf::from(xdg_cache).join(primal_names::BIOMEOS));
        }

        // 2. Try $HOME/.cache
        if let Ok(home) = env::var("HOME") {
            return Ok(PathBuf::from(home)
                .join(".cache")
                .join(primal_names::BIOMEOS));
        }

        // 3. Use etcetera (Pure Rust!) as fallback
        let strategy = choose_base_strategy().map_err(|_| PathError::NoHomeDir)?;
        Ok(strategy.cache_dir().join(primal_names::BIOMEOS))
    }

    /// Get XDG state directory
    fn get_state_dir() -> Result<PathBuf> {
        // 1. Try $XDG_STATE_HOME
        if let Ok(xdg_state) = env::var("XDG_STATE_HOME") {
            return Ok(PathBuf::from(xdg_state).join(primal_names::BIOMEOS));
        }

        // 2. Try $HOME/.local/state
        if let Ok(home) = env::var("HOME") {
            return Ok(PathBuf::from(home)
                .join(".local/state")
                .join(primal_names::BIOMEOS));
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

    /// Create `SystemPaths` without creating directories
    ///
    /// This is useful for Default implementation and cases where you want
    /// to compute paths but defer directory creation until actually needed.
    /// Directories will be created lazily when first accessed via methods
    /// like `primal_socket()`.
    ///
    /// Note: This will use fallback paths (e.g., /tmp) if XDG paths cannot
    /// be determined. For stricter path requirements, use `new()` which
    /// returns a `Result`.
    #[must_use]
    pub fn new_lazy() -> Self {
        // Compute paths with fallbacks - these operations cannot fail
        let runtime_dir =
            Self::get_runtime_dir().unwrap_or_else(|_| env::temp_dir().join(primal_names::BIOMEOS));
        let data_dir = Self::get_data_dir()
            .unwrap_or_else(|_| env::temp_dir().join(format!("{}-data", primal_names::BIOMEOS)));
        let config_dir = Self::get_config_dir()
            .unwrap_or_else(|_| env::temp_dir().join(format!("{}-config", primal_names::BIOMEOS)));
        let cache_dir = Self::get_cache_dir()
            .unwrap_or_else(|_| env::temp_dir().join(format!("{}-cache", primal_names::BIOMEOS)));
        let state_dir = Self::get_state_dir()
            .unwrap_or_else(|_| env::temp_dir().join(format!("{}-state", primal_names::BIOMEOS)));

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
    /// Create `SystemPaths` with lazy directory creation
    ///
    /// This implementation uses `new_lazy()` to avoid panicking.
    /// Directories will be created when first accessed. For explicit
    /// directory creation and error handling, use `SystemPaths::new()`.
    fn default() -> Self {
        Self::new_lazy()
    }
}

#[cfg(test)]
#[path = "paths_tests.rs"]
mod tests;
