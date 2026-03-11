//! Core types for the Primal Adapter Pattern

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use std::time::Duration;

/// Adapter that knows how to interact with a specific primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalAdapter {
    /// Name of the primal (e.g., "squirrel", "songbird")
    pub name: String,

    /// Path to the primal binary
    pub binary: PathBuf,

    /// Discovered interface pattern
    pub interface: PrimalInterface,

    /// What this primal can do
    pub capabilities: PrimalCapabilities,

    /// Current lifecycle state (not serialized)
    #[serde(skip)]
    pub state: PrimalState,

    /// When this adapter was discovered/cached
    pub discovered_at: chrono::DateTime<chrono::Utc>,

    /// Primal version (if available)
    pub version: Option<String>,
}

/// How to communicate with a primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrimalInterface {
    /// Direct execution (no subcommands) - like Squirrel
    Direct {
        /// Additional args to pass
        args: Vec<String>,
    },

    /// Subcommand-based - like "service", "serve", etc
    Subcommand {
        /// Command to start (e.g., "serve", "service", "start")
        start_cmd: String,
        /// Command to stop (if supported)
        stop_cmd: Option<String>,
    },

    /// Systemd service
    Service {
        /// Service name
        service_name: String,
    },

    /// Docker container
    Docker {
        /// Image name
        image: String,
        /// Container name
        container: String,
    },

    /// HTTP API-based lifecycle
    Api {
        /// Base endpoint
        endpoint: String,
        /// Start endpoint
        start_path: String,
        /// Stop endpoint (if supported)
        stop_path: Option<String>,
    },

    /// Unknown - still learning
    Unknown {
        /// Patterns we've tried
        attempted_patterns: Vec<InterfacePattern>,
    },
}

/// Interface patterns we try to discover
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InterfacePattern {
    /// Direct execution (no subcommands)
    Direct,
    /// `serve` subcommand pattern
    SubcommandServe,
    /// `service` subcommand pattern
    SubcommandService,
    /// `start` subcommand pattern
    SubcommandStart,
    /// `run` subcommand pattern
    SubcommandRun,
    /// Systemd service management
    Systemd,
    /// Docker container management
    Docker,
    /// HTTP API-based lifecycle control
    ApiLifecycle,
}

/// What a primal can do
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalCapabilities {
    /// Lifecycle capabilities
    pub lifecycle: LifecycleCapabilities,

    /// Health check endpoint (if available)
    pub health_check: Option<HealthCheckConfig>,

    /// Port configuration method
    pub port_config: PortConfigMethod,

    /// Can respond to version query
    pub has_version_cmd: bool,

    /// Can respond to help quickly
    pub has_fast_help: bool,
}

/// Lifecycle capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifecycleCapabilities {
    /// Can we start this primal?
    pub can_start: bool,

    /// Can we request stop?
    pub can_stop: bool,

    /// Can we request restart?
    pub can_restart: bool,

    /// Supports graceful shutdown (SIGTERM)
    pub graceful_shutdown: bool,

    /// Can refuse our requests (sovereignty!)
    pub can_refuse: bool,
}

/// How to configure the port
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortConfigMethod {
    /// Environment variable
    EnvVar(String),

    /// CLI flag
    CliFlag(String),

    /// Config file
    ConfigFile { path: String, format: String },

    /// Multiple methods (in priority order)
    Multiple(Vec<PortConfigMethod>),

    /// Unknown
    Unknown,
}

/// Health check configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckConfig {
    /// Health check URL pattern (PORT replaced at runtime)
    pub url_pattern: String,

    /// Expected status code
    pub expected_status: u16,

    /// Timeout
    pub timeout: Duration,
}

/// Current lifecycle state
#[derive(Debug, Clone, Default)]
pub enum PrimalState {
    /// Not started yet
    #[default]
    NotStarted,

    /// Starting up
    Starting {
        /// When start was initiated
        started_at: chrono::DateTime<chrono::Utc>,
    },

    /// Running and healthy
    Running {
        /// Process handle (if we started it)
        #[allow(dead_code)] // Used for process lifecycle management and diagnostics
        pid: Option<u32>,
        /// Port it's running on
        port: u16,
    },

    /// Running but unhealthy
    Unhealthy {
        /// Port it's on
        port: u16,
        /// What's wrong
        reason: String,
    },

    /// Stopping
    Stopping,

    /// Stopped
    Stopped,

    /// Unknown state
    Unknown,
}

impl PrimalAdapter {
    /// Create a new adapter with default capabilities
    pub fn new(name: String, binary: PathBuf) -> Self {
        Self {
            name,
            binary,
            interface: PrimalInterface::Unknown {
                attempted_patterns: vec![],
            },
            capabilities: PrimalCapabilities::default(),
            state: PrimalState::NotStarted,
            discovered_at: chrono::Utc::now(),
            version: None,
        }
    }

    /// Start the primal on the specified port
    pub async fn start(&mut self, port: u16) -> Result<()> {
        self.state = PrimalState::Starting {
            started_at: chrono::Utc::now(),
        };

        // Build command based on interface
        let mut cmd = self.build_start_command(port)?;

        // Spawn the process
        let child = cmd.spawn()?;

        // Update state
        self.state = PrimalState::Running {
            pid: Some(child.id()),
            port,
        };

        // Don't wait for child (it runs in background)
        std::mem::forget(child);

        Ok(())
    }

    /// Check if primal is healthy
    ///
    /// Uses running state as health indicator (socket-based health checks
    /// are done via AtomicClient in the deployment layer).
    pub async fn check_health(&self) -> Result<bool> {
        // Socket-based health checks are done via AtomicClient
        // Here we just check if the primal is in a running state
        Ok(matches!(self.state, PrimalState::Running { .. }))
    }

    /// Build the start command
    fn build_start_command(&self, port: u16) -> Result<std::process::Command> {
        let mut cmd = std::process::Command::new(&self.binary);

        // Add interface-specific args
        match &self.interface {
            PrimalInterface::Direct { args } => {
                cmd.args(args);
            }
            PrimalInterface::Subcommand { start_cmd, .. } => {
                cmd.arg(start_cmd);
            }
            _ => {
                // Other types handled elsewhere
            }
        }

        // Add port configuration
        match &self.capabilities.port_config {
            PortConfigMethod::EnvVar(name) => {
                cmd.env(name, port.to_string());
            }
            PortConfigMethod::CliFlag(flag) => {
                cmd.arg(flag).arg(port.to_string());
            }
            PortConfigMethod::Multiple(methods) => {
                // Try first method
                if let Some(method) = methods.first() {
                    match method {
                        PortConfigMethod::EnvVar(name) => {
                            cmd.env(name, port.to_string());
                        }
                        PortConfigMethod::CliFlag(flag) => {
                            cmd.arg(flag).arg(port.to_string());
                        }
                        _ => {}
                    }
                }
            }
            PortConfigMethod::Unknown => {
                // Try common patterns
                cmd.env("PORT", port.to_string());
            }
            _ => {}
        }

        // Redirect output to XDG state directory
        let log_path = biomeos_types::SystemPaths::new_lazy().log_file(&self.name);
        if let Some(parent) = log_path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let log_file = std::fs::File::create(&log_path)?;
        cmd.stdout(log_file.try_clone()?);
        cmd.stderr(log_file);

        Ok(cmd)
    }
}

impl Default for PrimalCapabilities {
    fn default() -> Self {
        Self {
            lifecycle: LifecycleCapabilities::default(),
            health_check: None,
            port_config: PortConfigMethod::Unknown,
            has_version_cmd: false,
            has_fast_help: false,
        }
    }
}

impl Default for LifecycleCapabilities {
    fn default() -> Self {
        Self {
            can_start: false,
            can_stop: false,
            can_restart: false,
            graceful_shutdown: false,
            can_refuse: true, // Always true - sovereignty!
        }
    }
}

impl PrimalInterface {
    /// Check if this is a known interface
    pub fn is_known(&self) -> bool {
        !matches!(self, PrimalInterface::Unknown { .. })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primal_adapter_new() {
        let adapter =
            PrimalAdapter::new("squirrel".to_string(), PathBuf::from("/usr/bin/squirrel"));
        assert_eq!(adapter.name, "squirrel");
        assert_eq!(adapter.binary, PathBuf::from("/usr/bin/squirrel"));
        assert!(!adapter.interface.is_known());
        assert!(matches!(adapter.state, PrimalState::NotStarted));
        assert!(adapter.version.is_none());
    }

    #[test]
    fn test_primal_capabilities_default() {
        let caps = PrimalCapabilities::default();
        assert!(!caps.lifecycle.can_start);
        assert!(!caps.lifecycle.can_stop);
        assert!(caps.lifecycle.can_refuse);
        assert!(caps.health_check.is_none());
        assert!(matches!(caps.port_config, PortConfigMethod::Unknown));
        assert!(!caps.has_version_cmd);
        assert!(!caps.has_fast_help);
    }

    #[test]
    fn test_lifecycle_capabilities_default() {
        let lc = LifecycleCapabilities::default();
        assert!(!lc.can_start);
        assert!(!lc.can_stop);
        assert!(!lc.can_restart);
        assert!(!lc.graceful_shutdown);
        assert!(lc.can_refuse);
    }

    #[test]
    fn test_primal_interface_is_known() {
        assert!(PrimalInterface::Direct { args: vec![] }.is_known());
        assert!(PrimalInterface::Subcommand {
            start_cmd: "serve".to_string(),
            stop_cmd: None,
        }
        .is_known());
        assert!(PrimalInterface::Service {
            service_name: "biomeos".to_string(),
        }
        .is_known());
        assert!(PrimalInterface::Docker {
            image: "img".to_string(),
            container: "cnt".to_string(),
        }
        .is_known());
        assert!(PrimalInterface::Api {
            endpoint: "http://localhost".to_string(),
            start_path: "/start".to_string(),
            stop_path: None,
        }
        .is_known());
        assert!(!PrimalInterface::Unknown {
            attempted_patterns: vec![],
        }
        .is_known());
    }

    #[test]
    fn test_primal_interface_serialization() {
        let iface = PrimalInterface::Subcommand {
            start_cmd: "serve".to_string(),
            stop_cmd: Some("stop".to_string()),
        };
        let json = serde_json::to_string(&iface).expect("serialize");
        let restored: PrimalInterface = serde_json::from_str(&json).expect("deserialize");
        match (&iface, &restored) {
            (
                PrimalInterface::Subcommand {
                    start_cmd: s1,
                    stop_cmd: o1,
                },
                PrimalInterface::Subcommand {
                    start_cmd: s2,
                    stop_cmd: o2,
                },
            ) => {
                assert_eq!(s1, s2);
                assert_eq!(o1, o2);
            }
            _ => panic!("mismatch"),
        }
    }

    #[test]
    fn test_port_config_method_serialization() {
        let methods = [
            PortConfigMethod::EnvVar("PORT".to_string()),
            PortConfigMethod::CliFlag("--port".to_string()),
            PortConfigMethod::ConfigFile {
                path: "/etc/config".to_string(),
                format: "yaml".to_string(),
            },
            PortConfigMethod::Multiple(vec![
                PortConfigMethod::EnvVar("PORT".to_string()),
                PortConfigMethod::CliFlag("-p".to_string()),
            ]),
            PortConfigMethod::Unknown,
        ];
        for method in methods {
            let json = serde_json::to_value(&method).expect("serialize");
            let restored: PortConfigMethod = serde_json::from_value(json).expect("deserialize");
            assert_eq!(
                serde_json::to_value(&method).unwrap(),
                serde_json::to_value(&restored).unwrap()
            );
        }
    }

    #[test]
    fn test_health_check_config_serialization() {
        let config = HealthCheckConfig {
            url_pattern: "http://localhost:{PORT}/health".to_string(),
            expected_status: 200,
            timeout: Duration::from_secs(5),
        };
        let json = serde_json::to_value(&config).expect("serialize");
        let restored: HealthCheckConfig = serde_json::from_value(json).expect("deserialize");
        assert_eq!(config.url_pattern, restored.url_pattern);
        assert_eq!(config.expected_status, restored.expected_status);
    }

    #[test]
    fn test_primal_state_default() {
        assert!(matches!(PrimalState::default(), PrimalState::NotStarted));
    }
}
