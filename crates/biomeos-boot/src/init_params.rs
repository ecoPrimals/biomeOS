//! Boot Parameter Parsing for BiomeOS Init
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
    pub fn description(&self) -> &str {
        match self {
            Self::Standard { .. } => "Standard (load biome.yaml)",
            Self::Discovery => "Discovery (scan network)",
            Self::Install { .. } => "Installation",
            Self::Network { .. } => "Network Boot",
            Self::Recovery => "Recovery Mode",
        }
    }

    /// Checks if this is an interactive mode requiring user input
    pub fn is_interactive(&self) -> bool {
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
            .find_map(|p| {
                p.strip_prefix("biomeos.install=")
                    .map(|t| PathBuf::from(t))
            });
        BootMode::Install { target }
    } else if params.iter().any(|p| p.starts_with("biomeos.network")) {
        let server = params
            .iter()
            .find_map(|p| p.strip_prefix("biomeos.network=").map(String::from));
        BootMode::Network { server }
    } else if params.iter().any(|p| p.starts_with("biomeos.recovery") || *p == "recovery") {
        BootMode::Recovery
    } else {
        let config = params
            .iter()
            .find_map(|p| {
                p.strip_prefix("biomeos.config=")
                    .map(|c| PathBuf::from(c))
            });
        BootMode::Standard { config }
    };

    let extra_params = params
        .iter()
        .filter(|p| !p.starts_with("biomeos."))
        .map(|s| s.to_string())
        .collect();

    info!("Boot mode: {}", mode.description());

    Ok(BootParams { mode, extra_params })
}

#[cfg(test)]
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
        let params = parse_cmdline("biomeos.network=192.168.1.1:8080").unwrap();
        match params.mode {
            BootMode::Network { server: Some(s) } => {
                assert_eq!(s, "192.168.1.1:8080");
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
        assert_eq!(BootMode::Discovery.description(), "Discovery (scan network)");
    }

    #[test]
    fn test_is_interactive() {
        assert!(BootMode::Install { target: None }.is_interactive());
        assert!(BootMode::Recovery.is_interactive());
        assert!(!BootMode::Standard { config: None }.is_interactive());
        assert!(!BootMode::Discovery.is_interactive());
    }
}

