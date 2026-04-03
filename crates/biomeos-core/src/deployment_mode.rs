// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Deployment Mode Detection
//!
//! Detects whether biomeOS is running as:
//! - **Cold Spore**: From removable media (USB/SD) without installation
//! - **Live Spore**: Installed to bare metal (full system)
//! - **Sibling Spore**: Running on top of existing OS
//!
//! This enables adaptive socket paths, resource allocation, and primal coordination
//! based on the deployment context.
//!
//! # Philosophy
//!
//! - **Self-aware**: biomeOS knows its deployment context
//! - **Adaptive**: Socket paths and behavior adapt to the environment
//! - **Pure Rust**: No unsafe code, no external dependencies for detection
//! - **Graceful**: Works in all environments, degrades gracefully

#![deny(unsafe_code)] // Fast AND safe: Zero unsafe code, pure Rust system detection

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Deployment mode of the biomeOS instance
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeploymentMode {
    /// Running from removable media (USB/SD card) without installation
    ColdSpore {
        /// Path to the removable media mount point
        media_path: PathBuf,
        /// Whether to use persistent storage on the USB
        persistence: bool,
        /// Host operating system (if detectable)
        host_os: HostOS,
    },

    /// Installed to bare metal (full system installation)
    LiveSpore {
        /// Root partition path
        root_partition: PathBuf,
        /// Boot partition path
        boot_partition: PathBuf,
        /// Installed version
        installed_version: String,
    },

    /// Running on top of existing OS (Linux/Mac/Windows)
    SiblingSpore {
        /// Host operating system
        host_os: HostOS,
        /// Installation directory
        install_dir: PathBuf,
        /// Isolation level
        isolation: IsolationLevel,
    },
}

/// Host operating system
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum HostOS {
    /// Linux distribution
    Linux {
        /// Distribution name (e.g. "ubuntu", "fedora")
        distro: String,
    },
    /// macOS
    MacOS {
        /// macOS version string
        version: String,
    },
    /// Windows
    Windows {
        /// Windows version string
        version: String,
    },
    /// BSD variant
    BSD {
        /// BSD variant name (e.g. "freebsd", "openbsd")
        variant: String,
    },
    /// Unknown/Other
    Unknown,
}

/// Optional overrides for [`DeploymentMode::from_env_string_with_params`].
///
/// When a field is `None`, defaults match the behavior of unset environment variables
/// (see each field).
#[derive(Debug, Clone, Default)]
pub struct DeploymentFromEnvParams {
    /// `BIOMEOS_MEDIA_PATH` — when `None`, Cold Spore defaults to `/media/biomeos`.
    pub media_path: Option<String>,
    /// `BIOMEOS_PERSISTENCE` — when `None`, defaults to `false`.
    pub persistence: Option<bool>,
    /// `BIOMEOS_VERSION` — when `None`, Live Spore uses `CARGO_PKG_VERSION`.
    pub installed_version: Option<String>,
    /// `BIOMEOS_ISOLATION` — when `None`, [`DeploymentMode::isolation_level_from_env`] treats it
    /// as unset and defaults to [`IsolationLevel::Shared`].
    pub isolation: Option<String>,
}

impl DeploymentFromEnvParams {
    fn from_current_env() -> Self {
        Self {
            media_path: std::env::var("BIOMEOS_MEDIA_PATH").ok(),
            persistence: std::env::var("BIOMEOS_PERSISTENCE")
                .ok()
                .map(|v| v == "true" || v == "1"),
            installed_version: std::env::var("BIOMEOS_VERSION").ok(),
            isolation: std::env::var("BIOMEOS_ISOLATION").ok(),
        }
    }
}

/// Isolation level for Sibling Spore
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum IsolationLevel {
    /// Sandboxed (no host access)
    Sandboxed,
    /// Shared (limited host access)
    Shared,
    /// Full (full host integration)
    Full,
}

impl DeploymentMode {
    /// Detect the current deployment mode
    ///
    /// # Detection Strategy
    ///
    /// 1. Check `BIOMEOS_DEPLOYMENT_MODE` environment variable (explicit override)
    /// 2. Check if running from removable media (Cold Spore)
    /// 3. Check if installed to root filesystem (Live Spore)
    /// 4. Default to Sibling Spore
    ///
    /// # Examples
    ///
    /// ```no_run
    /// use biomeos_core::deployment_mode::DeploymentMode;
    ///
    /// let mode = DeploymentMode::detect().unwrap();
    /// println!("Running in mode: {:?}", mode);
    /// ```
    pub fn detect() -> Result<Self> {
        // 1. Check for explicit override
        if let Ok(mode_str) = std::env::var("BIOMEOS_DEPLOYMENT_MODE") {
            return Self::from_env_string_with_params(
                &mode_str,
                DeploymentFromEnvParams::from_current_env(),
            );
        }

        // 2. Check if running from removable media
        if let Ok(media_path) = Self::detect_removable_media() {
            let host_os = Self::detect_host_os()?;
            let persistence = std::env::var("BIOMEOS_PERSISTENCE")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false);

            return Ok(Self::ColdSpore {
                media_path,
                persistence,
                host_os,
            });
        }

        // 3. Check if installed to root filesystem
        if let Ok((root, boot, version)) = Self::detect_root_installation() {
            return Ok(Self::LiveSpore {
                root_partition: root,
                boot_partition: boot,
                installed_version: version,
            });
        }

        // 4. Default to Sibling Spore
        let host_os = Self::detect_host_os()?;
        let install_dir = Self::get_install_dir()?;
        let isolation =
            Self::isolation_level_from_env(std::env::var("BIOMEOS_ISOLATION").ok().as_deref());

        Ok(Self::SiblingSpore {
            host_os,
            install_dir,
            isolation,
        })
    }

    /// Get the socket path prefix for this deployment mode
    ///
    /// Returns the base directory where primal Unix sockets should be created.
    ///
    /// # Examples
    ///
    /// ```no_run
    /// # use biomeos_core::deployment_mode::DeploymentMode;
    /// let mode = DeploymentMode::detect().unwrap();
    /// let socket_prefix = mode.socket_prefix();
    /// println!("Sockets will be created in: {}", socket_prefix.display());
    /// ```
    #[must_use]
    pub fn socket_prefix(&self) -> PathBuf {
        self.socket_prefix_with_runtime(std::env::var("XDG_RUNTIME_DIR").ok().as_deref(), None)
    }

    /// Like [`Self::socket_prefix`], but uses the given runtime directory and UID override
    /// instead of reading `XDG_RUNTIME_DIR` / `UID` from the environment.
    ///
    /// For Live Spore, when `xdg_runtime_dir` is `None`, the path is `/run/user/{uid}/biomeos`
    /// where `uid` is `uid_for_run_user_path` if set, otherwise `get_uid()`.
    #[must_use]
    pub fn socket_prefix_with_runtime(
        &self,
        xdg_runtime_dir: Option<&str>,
        uid_for_run_user_path: Option<u32>,
    ) -> PathBuf {
        match self {
            Self::ColdSpore { media_path, .. } => media_path.join("runtime"),
            Self::LiveSpore { .. } => {
                Self::live_socket_prefix(xdg_runtime_dir, uid_for_run_user_path)
            }
            Self::SiblingSpore { install_dir, .. } => install_dir.join("runtime"),
        }
    }

    fn live_socket_prefix(
        xdg_runtime_dir: Option<&str>,
        uid_for_run_user_path: Option<u32>,
    ) -> PathBuf {
        if let Some(xdg) = xdg_runtime_dir {
            PathBuf::from(xdg).join("biomeos")
        } else {
            let uid = uid_for_run_user_path.unwrap_or_else(Self::get_uid);
            PathBuf::from(format!("/run/user/{uid}/biomeos"))
        }
    }

    /// Get a human-readable description of the deployment mode
    #[must_use]
    pub fn description(&self) -> String {
        match self {
            Self::ColdSpore {
                media_path,
                persistence,
                ..
            } => {
                if *persistence {
                    format!("Cold Spore (USB: {}, persistent)", media_path.display())
                } else {
                    format!("Cold Spore (USB: {}, ephemeral)", media_path.display())
                }
            }
            Self::LiveSpore {
                installed_version, ..
            } => {
                format!("Live Spore (v{installed_version})")
            }
            Self::SiblingSpore { host_os, .. } => {
                format!("Sibling Spore (on {})", host_os.name())
            }
        }
    }

    /// Build a deployment mode from a mode string and explicit env-equivalent values.
    ///
    /// This is the parameterized form of parsing `BIOMEOS_DEPLOYMENT_MODE`; see
    /// [`DeploymentFromEnvParams`] for defaults when fields are unset.
    pub fn from_env_string_with_params(s: &str, params: DeploymentFromEnvParams) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "cold" | "coldspore" | "cold_spore" => {
                let media_path = params
                    .media_path
                    .as_ref()
                    .map_or_else(|| PathBuf::from("/media/biomeos"), PathBuf::from);
                let persistence = params.persistence.unwrap_or(false);
                let host_os = Self::detect_host_os()?;

                Ok(Self::ColdSpore {
                    media_path,
                    persistence,
                    host_os,
                })
            }
            "live" | "livespore" | "live_spore" => {
                let root = PathBuf::from("/");
                let boot = PathBuf::from("/boot");
                let version = params
                    .installed_version
                    .unwrap_or_else(|| env!("CARGO_PKG_VERSION").to_string());

                Ok(Self::LiveSpore {
                    root_partition: root,
                    boot_partition: boot,
                    installed_version: version,
                })
            }
            "sibling" | "siblingspore" | "sibling_spore" => {
                let host_os = Self::detect_host_os()?;
                let install_dir = Self::get_install_dir()?;
                let isolation = Self::isolation_level_from_env(params.isolation.as_deref());

                Ok(Self::SiblingSpore {
                    host_os,
                    install_dir,
                    isolation,
                })
            }
            _ => anyhow::bail!("Invalid deployment mode: {s}"),
        }
    }

    /// Resolve isolation level from the `BIOMEOS_ISOLATION`-style value (or `None` for unset).
    #[must_use]
    pub fn isolation_level_from_env(isolation: Option<&str>) -> IsolationLevel {
        if let Some(level) = isolation {
            match level.to_lowercase().as_str() {
                "sandboxed" | "sandbox" => return IsolationLevel::Sandboxed,
                "shared" => return IsolationLevel::Shared,
                "full" => return IsolationLevel::Full,
                _ => {}
            }
        }
        IsolationLevel::Shared
    }

    // Private helper methods

    fn detect_removable_media() -> Result<PathBuf> {
        // Strategy:
        // 1. Check if current executable is on removable media
        // 2. Look for biomeOS marker file on USB devices

        let exe_path = std::env::current_exe().context("Failed to get current executable path")?;

        // Check if executable is on a mount point that looks like removable media
        if let Ok(mounts) = std::fs::read_to_string("/proc/mounts") {
            for line in mounts.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() < 2 {
                    continue;
                }

                let mount_point = Path::new(parts[1]);

                // Check if exe is under this mount point
                if exe_path.starts_with(mount_point) {
                    // Check if this looks like removable media
                    if Self::is_removable_mount(parts[0], mount_point) {
                        return Ok(mount_point.to_path_buf());
                    }
                }
            }
        }

        anyhow::bail!("Not running from removable media")
    }

    pub(crate) fn is_removable_mount(device: &str, mount_point: &Path) -> bool {
        // Check for typical USB/removable media patterns
        if device.contains("/dev/sd") || device.contains("/dev/mmcblk") {
            // Additional check: look for biomeOS marker
            let marker = mount_point.join(".biomeos-spore");
            if marker.exists() {
                return true;
            }
        }

        // Check mount point path patterns
        let mount_str = mount_point.to_string_lossy();
        mount_str.contains("/media/")
            || mount_str.contains("/mnt/")
            || mount_str.contains("/run/media/")
    }

    fn detect_root_installation() -> Result<(PathBuf, PathBuf, String)> {
        // Check if biomeOS is installed to root filesystem
        let marker = Path::new("/etc/biomeos/version");

        if marker.exists() {
            let version = std::fs::read_to_string(marker)
                .context("Failed to read version file")?
                .trim()
                .to_string();

            Ok((PathBuf::from("/"), PathBuf::from("/boot"), version))
        } else {
            anyhow::bail!("Not installed to root filesystem")
        }
    }

    fn detect_host_os() -> Result<HostOS> {
        // Read /etc/os-release for Linux distributions
        if let Ok(os_release) = std::fs::read_to_string("/etc/os-release") {
            for line in os_release.lines() {
                if line.starts_with("NAME=") {
                    let name = line
                        .trim_start_matches("NAME=")
                        .trim_matches('"')
                        .to_string();

                    if name.to_lowercase().contains("mac") || name.to_lowercase().contains("darwin")
                    {
                        return Ok(HostOS::MacOS {
                            version: Self::get_os_version(),
                        });
                    } else if name.to_lowercase().contains("bsd") {
                        return Ok(HostOS::BSD { variant: name });
                    }
                    return Ok(HostOS::Linux { distro: name });
                }
            }
        }

        // Check for macOS
        if Path::new("/System/Library/CoreServices/SystemVersion.plist").exists() {
            return Ok(HostOS::MacOS {
                version: Self::get_os_version(),
            });
        }

        // Check for Windows (if running under WSL)
        if std::env::var("WSL_DISTRO_NAME").is_ok() {
            return Ok(HostOS::Windows {
                version: "WSL".to_string(),
            });
        }

        Ok(HostOS::Unknown)
    }

    fn get_os_version() -> String {
        std::env::var("OS_VERSION").unwrap_or_else(|_| "unknown".to_string())
    }

    fn get_install_dir() -> Result<PathBuf> {
        // 1. Check BIOMEOS_INSTALL_DIR (explicit override)
        if let Ok(dir) = std::env::var("BIOMEOS_INSTALL_DIR") {
            return Ok(PathBuf::from(dir));
        }

        // 2. Use XDG_DATA_HOME if available (XDG-compliant)
        if let Ok(xdg_data) = std::env::var("XDG_DATA_HOME") {
            return Ok(PathBuf::from(xdg_data).join("biomeos"));
        }

        // 3. Default to HOME/.local/share (XDG default)
        if let Ok(home) = std::env::var("HOME") {
            return Ok(PathBuf::from(home).join(".local/share/biomeos"));
        }

        // 4. EVOLVED: Use current directory as last resort (writable, platform-agnostic)
        // This works on all platforms including Android, Windows, etc.
        std::env::current_dir()
            .map(|p| p.join(".biomeos"))
            .context("Failed to determine install directory - no HOME or XDG paths available")
    }

    /// Get the current user ID.
    ///
    /// # Safety
    ///
    /// This function is safe - it reads from environment variables and `/proc/self/loginuid`
    /// which are standard Linux interfaces. All I/O operations use safe Rust APIs.
    /// The fallback value (1000) is a safe default for typical first user on Linux systems.
    ///
    /// # Panics
    ///
    /// This function never panics - it always returns a valid u32 value.
    fn get_uid() -> u32 {
        std::env::var("UID")
            .ok()
            .and_then(|s| s.parse().ok())
            .unwrap_or_else(|| {
                // Linux-specific: read /proc/self/loginuid
                // SAFETY: /proc/self/loginuid is a standard Linux interface that always
                // returns a valid integer string. If reading fails, we use a safe default.
                std::fs::read_to_string("/proc/self/loginuid")
                    .ok()
                    .and_then(|s| s.trim().parse().ok())
                    .unwrap_or(1000) // Safe default (typical first user)
            })
    }
}

impl HostOS {
    /// Get a short name for the host OS
    #[must_use]
    pub fn name(&self) -> String {
        match self {
            Self::Linux { distro } => format!("Linux ({distro})"),
            Self::MacOS { version } => format!("macOS {version}"),
            Self::Windows { version } => format!("Windows {version}"),
            Self::BSD { variant } => variant.clone(),
            Self::Unknown => "Unknown OS".to_string(),
        }
    }
}
