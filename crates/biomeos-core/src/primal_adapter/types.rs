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
    Direct,
    SubcommandServe,
    SubcommandService,
    SubcommandStart,
    SubcommandRun,
    Systemd,
    Docker,
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
        #[allow(dead_code)]
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

    /// Check if primal is healthy (HTTP-based, DEPRECATED)
    #[cfg(feature = "http-transport")]
    pub async fn check_health(&self) -> Result<bool> {
        if let Some(health_config) = &self.capabilities.health_check {
            let port = match &self.state {
                PrimalState::Running { port, .. } => *port,
                PrimalState::Unhealthy { port, .. } => *port,
                _ => return Ok(false),
            };

            let url = health_config.url_pattern.replace("PORT", &port.to_string());

            let client = reqwest::Client::new();
            let response =
                tokio::time::timeout(health_config.timeout, client.get(&url).send()).await;

            match response {
                Ok(Ok(resp)) => Ok(resp.status().as_u16() == health_config.expected_status),
                _ => Ok(false),
            }
        } else {
            // No health check configured, assume healthy if running
            Ok(matches!(self.state, PrimalState::Running { .. }))
        }
    }

    /// Check if primal is healthy (Pure Rust stub for non-HTTP builds)
    #[cfg(not(feature = "http-transport"))]
    pub async fn check_health(&self) -> Result<bool> {
        // Without HTTP, assume healthy if running
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

        // Redirect output to /tmp
        let log_path = format!("/tmp/{}.log", self.name);
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
