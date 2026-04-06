// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Boot Parameter Parsing for `BiomeOS` Init
//!
//! Parses kernel command-line parameters to determine boot mode.

use crate::init_error::{BootError, Result};
use std::path::PathBuf;
use tracing::info;

/// Boot mode determined from kernel parameters
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BootMode {
    /// Standard boot (load biome.yaml)
    Standard {
        /// Optional path to configuration file
        config: Option<PathBuf>,
    },
    /// Discovery mode (scan network for other nodes)
    Discovery,
    /// Installation mode
    Install {
        /// Target device for installation
        target: Option<PathBuf>,
    },
    /// Network boot mode
    Network {
        /// Optional server address
        server: Option<String>,
    },
    /// Recovery/emergency mode
    Recovery,
}

impl BootMode {
    /// Returns a human-readable description
    #[must_use]
    pub const fn description(&self) -> &str {
        match self {
            Self::Standard { .. } => "Standard (load biome.yaml)",
            Self::Discovery => "Discovery (scan network)",
            Self::Install { .. } => "Installation",
            Self::Network { .. } => "Network Boot",
            Self::Recovery => "Recovery Mode",
        }
    }

    /// Checks if this is an interactive mode requiring user input
    #[must_use]
    pub const fn is_interactive(&self) -> bool {
        matches!(self, Self::Install { .. } | Self::Recovery)
    }
}

/// Boot parameters parsed from kernel command line
#[derive(Debug, Clone)]
pub struct BootParams {
    /// Boot mode
    pub mode: BootMode,
    /// Additional kernel parameters
    pub extra_params: Vec<String>,
}

/// Parses boot parameters from `/proc/cmdline`
///
/// # Errors
///
/// Returns an error if `/proc/cmdline` cannot be read.
pub async fn parse() -> Result<BootParams> {
    let cmdline = tokio::fs::read_to_string("/proc/cmdline")
        .await
        .map_err(BootError::CmdlineRead)?;

    parse_cmdline(&cmdline)
}

/// Parses a command line string into boot parameters
///
/// # Arguments
///
/// * `cmdline` - The kernel command line string
///
/// # Errors
///
/// Returns an error if the command line contains invalid parameters.
pub fn parse_cmdline(cmdline: &str) -> Result<BootParams> {
    let params: Vec<&str> = cmdline.split_whitespace().collect();

    let mode = if params.iter().any(|p| p.starts_with("biomeos.discovery")) {
        BootMode::Discovery
    } else if params.iter().any(|p| p.starts_with("biomeos.install")) {
        let target = params
            .iter()
            .find_map(|p| p.strip_prefix("biomeos.install=").map(PathBuf::from));
        BootMode::Install { target }
    } else if params.iter().any(|p| p.starts_with("biomeos.network")) {
        let server = params
            .iter()
            .find_map(|p| p.strip_prefix("biomeos.network=").map(String::from));
        BootMode::Network { server }
    } else if params
        .iter()
        .any(|p| p.starts_with("biomeos.recovery") || *p == "recovery")
    {
        BootMode::Recovery
    } else {
        let config = params
            .iter()
            .find_map(|p| p.strip_prefix("biomeos.config=").map(PathBuf::from));
        BootMode::Standard { config }
    };

    let extra_params = params
        .iter()
        .filter(|p| !p.starts_with("biomeos."))
        .map(std::string::ToString::to_string)
        .collect();

    info!("Boot mode: {}", mode.description());

    Ok(BootParams { mode, extra_params })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_standard_mode() {
        let params = parse_cmdline("root=/dev/sda rw quiet").unwrap();
        assert!(matches!(params.mode, BootMode::Standard { .. }));
    }

    #[test]
    fn test_parse_discovery_mode() {
        let params = parse_cmdline("root=/dev/sda biomeos.discovery").unwrap();
        assert_eq!(params.mode, BootMode::Discovery);
    }

    #[test]
    fn test_parse_install_mode() {
        let params = parse_cmdline("biomeos.install=/dev/sdb").unwrap();
        match params.mode {
            BootMode::Install { target: Some(t) } => {
                assert_eq!(t, PathBuf::from("/dev/sdb"));
            }
            _ => panic!("Expected Install mode with target"),
        }
    }

    #[test]
    fn test_parse_network_mode() {
        use biomeos_types::constants::ports;
        let expected = format!("192.0.2.1:{}", ports::HTTP_BRIDGE);
        let params = parse_cmdline(&format!("biomeos.network={expected}")).unwrap();
        match params.mode {
            BootMode::Network { server: Some(s) } => {
                assert_eq!(s, expected);
            }
            _ => panic!("Expected Network mode with server"),
        }
    }

    #[test]
    fn test_parse_recovery_mode() {
        let params = parse_cmdline("recovery").unwrap();
        assert_eq!(params.mode, BootMode::Recovery);
    }

    #[test]
    fn test_mode_description() {
        assert_eq!(
            BootMode::Standard { config: None }.description(),
            "Standard (load biome.yaml)"
        );
        assert_eq!(
            BootMode::Discovery.description(),
            "Discovery (scan network)"
        );
    }

    #[test]
    fn test_is_interactive() {
        assert!(BootMode::Install { target: None }.is_interactive());
        assert!(BootMode::Recovery.is_interactive());
        assert!(!BootMode::Standard { config: None }.is_interactive());
        assert!(!BootMode::Discovery.is_interactive());
    }

    #[test]
    fn test_parse_standard_with_config() {
        let params = parse_cmdline("root=/dev/sda biomeos.config=/etc/biomeos/biome.yaml").unwrap();
        match &params.mode {
            BootMode::Standard { config: Some(c) } => {
                assert_eq!(c, &PathBuf::from("/etc/biomeos/biome.yaml"));
            }
            _ => panic!("Expected Standard with config"),
        }
    }

    #[test]
    fn test_parse_biomeos_recovery() {
        let params = parse_cmdline("root=/dev/sda biomeos.recovery").unwrap();
        assert_eq!(params.mode, BootMode::Recovery);
    }

    #[test]
    fn test_parse_empty_cmdline() {
        let params = parse_cmdline("").unwrap();
        assert!(matches!(params.mode, BootMode::Standard { .. }));
        assert!(params.extra_params.is_empty());
    }

    #[test]
    fn test_parse_extra_params_filtered() {
        let params = parse_cmdline("root=/dev/sda rw quiet biomeos.discovery").unwrap();
        assert_eq!(params.mode, BootMode::Discovery);
        assert!(
            params.extra_params.iter().any(|p| p == "root=/dev/sda"),
            "extra_params should contain non-biomeos params"
        );
        assert!(
            !params
                .extra_params
                .iter()
                .any(|p| p.starts_with("biomeos.")),
            "extra_params should not contain biomeos params"
        );
    }

    #[test]
    fn test_mode_description_network() {
        assert_eq!(
            BootMode::Network { server: None }.description(),
            "Network Boot"
        );
    }

    #[test]
    fn test_mode_description_install() {
        assert_eq!(
            BootMode::Install { target: None }.description(),
            "Installation"
        );
    }

    #[test]
    fn test_parse_install_without_target() {
        let params = parse_cmdline("biomeos.install").unwrap();
        match params.mode {
            BootMode::Install { target: None } => {}
            _ => panic!("Expected Install mode without target"),
        }
    }

    #[test]
    fn test_parse_network_without_server() {
        let params = parse_cmdline("biomeos.network").unwrap();
        match params.mode {
            BootMode::Network { server: None } => {}
            _ => panic!("Expected Network mode without server"),
        }
    }

    #[test]
    fn test_boot_params_extra_params() {
        let params = parse_cmdline("root=/dev/sda rw quiet").unwrap();
        assert_eq!(params.extra_params.len(), 3);
    }
}
