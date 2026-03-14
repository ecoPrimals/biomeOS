// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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
            return Self::from_env_string(&mode_str);
        }

        // 2. Check if running from removable media
        if let Ok(media_path) = Self::detect_removable_media() {
            let host_os = Self::detect_host_os()?;
            let persistence = std::env::var("BIOMEOS_PERSISTENCE")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false);

            return Ok(DeploymentMode::ColdSpore {
                media_path,
                persistence,
                host_os,
            });
        }

        // 3. Check if installed to root filesystem
        if let Ok((root, boot, version)) = Self::detect_root_installation() {
            return Ok(DeploymentMode::LiveSpore {
                root_partition: root,
                boot_partition: boot,
                installed_version: version,
            });
        }

        // 4. Default to Sibling Spore
        let host_os = Self::detect_host_os()?;
        let install_dir = Self::get_install_dir()?;
        let isolation = Self::detect_isolation_level();

        Ok(DeploymentMode::SiblingSpore {
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
    pub fn socket_prefix(&self) -> PathBuf {
        match self {
            DeploymentMode::ColdSpore { media_path, .. } => media_path.join("runtime"),
            DeploymentMode::LiveSpore { .. } => {
                // EVOLVED: Use XDG_RUNTIME_DIR instead of hardcoded path
                // This respects the system's runtime directory configuration
                if let Ok(xdg_runtime) = std::env::var("XDG_RUNTIME_DIR") {
                    PathBuf::from(xdg_runtime).join("biomeos")
                } else {
                    // Fallback: construct XDG-compliant path using UID
                    let uid = Self::get_uid();
                    PathBuf::from(format!("/run/user/{uid}/biomeos"))
                }
            }
            DeploymentMode::SiblingSpore { install_dir, .. } => {
                // User-space runtime directory
                install_dir.join("runtime")
            }
        }
    }

    /// Get a human-readable description of the deployment mode
    pub fn description(&self) -> String {
        match self {
            DeploymentMode::ColdSpore {
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
            DeploymentMode::LiveSpore {
                installed_version, ..
            } => {
                format!("Live Spore (v{installed_version})")
            }
            DeploymentMode::SiblingSpore { host_os, .. } => {
                format!("Sibling Spore (on {})", host_os.name())
            }
        }
    }

    // Private helper methods

    fn from_env_string(s: &str) -> Result<Self> {
        match s.to_lowercase().as_str() {
            "cold" | "coldspore" | "cold_spore" => {
                let media_path = std::env::var("BIOMEOS_MEDIA_PATH")
                    .map(PathBuf::from)
                    .unwrap_or_else(|_| PathBuf::from("/media/biomeos"));
                let persistence = std::env::var("BIOMEOS_PERSISTENCE")
                    .map(|v| v == "true" || v == "1")
                    .unwrap_or(false);
                let host_os = Self::detect_host_os()?;

                Ok(DeploymentMode::ColdSpore {
                    media_path,
                    persistence,
                    host_os,
                })
            }
            "live" | "livespore" | "live_spore" => {
                let root = PathBuf::from("/");
                let boot = PathBuf::from("/boot");
                let version = std::env::var("BIOMEOS_VERSION")
                    .unwrap_or_else(|_| env!("CARGO_PKG_VERSION").to_string());

                Ok(DeploymentMode::LiveSpore {
                    root_partition: root,
                    boot_partition: boot,
                    installed_version: version,
                })
            }
            "sibling" | "siblingspore" | "sibling_spore" => {
                let host_os = Self::detect_host_os()?;
                let install_dir = Self::get_install_dir()?;
                let isolation = Self::detect_isolation_level();

                Ok(DeploymentMode::SiblingSpore {
                    host_os,
                    install_dir,
                    isolation,
                })
            }
            _ => anyhow::bail!("Invalid deployment mode: {s}"),
        }
    }

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

    fn is_removable_mount(device: &str, mount_point: &Path) -> bool {
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
                    } else {
                        return Ok(HostOS::Linux { distro: name });
                    }
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

    fn detect_isolation_level() -> IsolationLevel {
        // Check environment variable
        if let Ok(level) = std::env::var("BIOMEOS_ISOLATION") {
            match level.to_lowercase().as_str() {
                "sandboxed" | "sandbox" => return IsolationLevel::Sandboxed,
                "shared" => return IsolationLevel::Shared,
                "full" => return IsolationLevel::Full,
                _ => {}
            }
        }

        // Default to Shared
        IsolationLevel::Shared
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
    pub fn name(&self) -> String {
        match self {
            HostOS::Linux { distro } => format!("Linux ({distro})"),
            HostOS::MacOS { version } => format!("macOS {version}"),
            HostOS::Windows { version } => format!("Windows {version}"),
            HostOS::BSD { variant } => variant.clone(),
            HostOS::Unknown => "Unknown OS".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_deployment_mode_from_env_cold() {
        std::env::set_var("BIOMEOS_DEPLOYMENT_MODE", "cold");
        std::env::set_var("BIOMEOS_MEDIA_PATH", "/media/usb0");

        let mode = DeploymentMode::from_env_string("cold").unwrap();

        match mode {
            DeploymentMode::ColdSpore { media_path, .. } => {
                assert_eq!(media_path, PathBuf::from("/media/usb0"));
            }
            _ => panic!("Expected ColdSpore"),
        }
    }

    #[test]
    fn test_deployment_mode_from_env_live() {
        let mode = DeploymentMode::from_env_string("live").unwrap();

        match mode {
            DeploymentMode::LiveSpore { root_partition, .. } => {
                assert_eq!(root_partition, PathBuf::from("/"));
            }
            _ => panic!("Expected LiveSpore"),
        }
    }

    #[test]
    fn test_deployment_mode_from_env_sibling() {
        let mode = DeploymentMode::from_env_string("sibling").unwrap();

        match mode {
            DeploymentMode::SiblingSpore { .. } => {
                // Success
            }
            _ => panic!("Expected SiblingSpore"),
        }
    }

    #[test]
    fn test_socket_prefix_cold() {
        let mode = DeploymentMode::ColdSpore {
            media_path: PathBuf::from("/media/usb0"),
            persistence: false,
            host_os: HostOS::Unknown,
        };

        assert_eq!(mode.socket_prefix(), PathBuf::from("/media/usb0/runtime"));
    }

    #[test]
    fn test_socket_prefix_sibling() {
        let mode = DeploymentMode::SiblingSpore {
            host_os: HostOS::Unknown,
            install_dir: PathBuf::from("/home/user/.local/share/biomeos"),
            isolation: IsolationLevel::Shared,
        };

        assert_eq!(
            mode.socket_prefix(),
            PathBuf::from("/home/user/.local/share/biomeos/runtime")
        );
    }

    #[test]
    fn test_description() {
        let mode = DeploymentMode::ColdSpore {
            media_path: PathBuf::from("/media/usb0"),
            persistence: true,
            host_os: HostOS::Linux {
                distro: "Ubuntu".to_string(),
            },
        };

        let desc = mode.description();
        assert!(desc.contains("Cold Spore"));
        assert!(desc.contains("persistent"));
    }

    #[test]
    fn test_host_os_name() {
        let os = HostOS::Linux {
            distro: "Ubuntu 22.04".to_string(),
        };

        assert_eq!(os.name(), "Linux (Ubuntu 22.04)");
    }

    #[test]
    fn test_from_env_string_invalid() {
        let result = DeploymentMode::from_env_string("invalid_mode");
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert!(err.to_string().contains("Invalid deployment mode"));
    }

    #[test]
    fn test_from_env_string_variants() {
        // cold_spore, livespore, sibling_spore (underscore variants)
        let cold = DeploymentMode::from_env_string("cold_spore").unwrap();
        assert!(matches!(cold, DeploymentMode::ColdSpore { .. }));

        let live = DeploymentMode::from_env_string("live_spore").unwrap();
        assert!(matches!(live, DeploymentMode::LiveSpore { .. }));

        let sibling = DeploymentMode::from_env_string("sibling_spore").unwrap();
        assert!(matches!(sibling, DeploymentMode::SiblingSpore { .. }));
    }

    #[test]
    fn test_from_env_string_cold_persistence() {
        std::env::set_var("BIOMEOS_DEPLOYMENT_MODE", "cold");
        std::env::set_var("BIOMEOS_MEDIA_PATH", "/media/usb1");
        std::env::set_var("BIOMEOS_PERSISTENCE", "1");

        let mode = DeploymentMode::from_env_string("cold").unwrap();
        match mode {
            DeploymentMode::ColdSpore { persistence, .. } => assert!(persistence),
            _ => panic!("Expected ColdSpore"),
        }
    }

    #[test]
    fn test_socket_prefix_livespore_with_xdg() {
        std::env::set_var("XDG_RUNTIME_DIR", "/run/user/1000");
        let mode = DeploymentMode::LiveSpore {
            root_partition: PathBuf::from("/"),
            boot_partition: PathBuf::from("/boot"),
            installed_version: "1.0.0".to_string(),
        };
        let prefix = mode.socket_prefix();
        assert_eq!(prefix, PathBuf::from("/run/user/1000/biomeos"));
        std::env::remove_var("XDG_RUNTIME_DIR");
    }

    #[test]
    fn test_description_cold_ephemeral() {
        let mode = DeploymentMode::ColdSpore {
            media_path: PathBuf::from("/media/usb0"),
            persistence: false,
            host_os: HostOS::Unknown,
        };
        let desc = mode.description();
        assert!(desc.contains("Cold Spore"));
        assert!(desc.contains("ephemeral"));
    }

    #[test]
    fn test_description_livespore() {
        let mode = DeploymentMode::LiveSpore {
            root_partition: PathBuf::from("/"),
            boot_partition: PathBuf::from("/boot"),
            installed_version: "2.0.0".to_string(),
        };
        let desc = mode.description();
        assert!(desc.contains("Live Spore"));
        assert!(desc.contains("2.0.0"));
    }

    #[test]
    fn test_description_siblingspore() {
        let mode = DeploymentMode::SiblingSpore {
            host_os: HostOS::Windows {
                version: "11".to_string(),
            },
            install_dir: PathBuf::from("/opt/biomeos"),
            isolation: IsolationLevel::Full,
        };
        let desc = mode.description();
        assert!(desc.contains("Sibling Spore"));
        assert!(desc.contains("Windows"));
    }

    #[test]
    fn test_host_os_name_all_variants() {
        assert_eq!(
            HostOS::MacOS {
                version: "14.0".to_string()
            }
            .name(),
            "macOS 14.0"
        );
        assert_eq!(
            HostOS::Windows {
                version: "11".to_string()
            }
            .name(),
            "Windows 11"
        );
        assert_eq!(
            HostOS::BSD {
                variant: "FreeBSD".to_string()
            }
            .name(),
            "FreeBSD"
        );
        assert_eq!(HostOS::Unknown.name(), "Unknown OS");
    }

    #[test]
    fn test_deployment_mode_serialization() {
        let mode = DeploymentMode::SiblingSpore {
            host_os: HostOS::Unknown,
            install_dir: PathBuf::from("/home/test/.local/share/biomeos"),
            isolation: IsolationLevel::Sandboxed,
        };
        let json = serde_json::to_string(&mode).expect("serialize");
        let restored: DeploymentMode = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(mode, restored);
    }

    #[test]
    fn test_isolation_level_serialization() {
        let levels = [
            IsolationLevel::Sandboxed,
            IsolationLevel::Shared,
            IsolationLevel::Full,
        ];
        for level in levels {
            let json = serde_json::to_string(&level).expect("serialize");
            let restored: IsolationLevel = serde_json::from_str(&json).expect("deserialize");
            assert_eq!(level, restored);
        }
    }
}
