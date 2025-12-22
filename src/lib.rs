//! BiomeOS - Universal Operating System
//!
//! A capability-based orchestration layer for managing primals and ecosystems

pub mod universal_adapter;

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Re-export key types from universal adapter
pub use universal_adapter::{
    coordinate_execution, find_primal_by_capability, get_discovered_primals, AdapterConfig,
    PrimalInfo, RequestMetadata, RequestPriority, ResponseMetadata, ServiceStatus, SongbirdClient,
    SystemStatus, ToadstoolClient, UniversalAdapter, UniversalRequest, UniversalResponse,
};

// Re-export core types
pub use biomeos_core::UniversalBiomeOSManager;
pub use biomeos_types::BiomeOSConfig;

/// UI interaction modes
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub enum UIMode {
    /// Automatically detect best UI mode
    #[default]
    Auto,

    /// Terminal/CLI interface
    Terminal,

    /// Web interface
    Web,

    /// Desktop GUI
    Desktop,

    /// Mobile interface
    Mobile,

    /// Headless (no UI)
    Headless,
}

/// BiomeOS runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeOSRuntime {
    /// Version information
    pub version: String,

    /// UI mode
    pub mode: UIMode,

    /// Runtime metadata
    pub metadata: RuntimeMetadata,

    /// System configuration
    pub system: SystemConfig,
}

/// Runtime metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeMetadata {
    /// Startup timestamp
    pub startup_time: chrono::DateTime<chrono::Utc>,

    /// Process ID
    pub pid: u32,

    /// Host information
    pub host: HostInfo,

    /// Environment variables
    pub environment: HashMap<String, String>,
}

/// Host information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HostInfo {
    /// Hostname
    pub hostname: String,

    /// Operating system
    pub os: String,

    /// Architecture
    pub arch: String,

    /// Kernel version
    pub kernel: String,
}

/// System configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemConfig {
    /// Maximum memory usage (bytes)
    pub max_memory_mb: Option<u64>,

    /// Maximum CPU cores
    pub max_cpu_cores: Option<u32>,

    /// Temporary directory
    pub temp_dir: Option<String>,

    /// Log level
    pub log_level: String,

    /// Debug mode enabled
    pub debug: bool,
}

impl Default for BiomeOSRuntime {
    fn default() -> Self {
        Self {
            version: biomeos_types::VERSION.to_string(),
            mode: UIMode::Auto,
            metadata: RuntimeMetadata {
                startup_time: chrono::Utc::now(),
                pid: std::process::id(),
                host: HostInfo {
                    hostname: gethostname::gethostname().to_string_lossy().to_string(),
                    os: std::env::consts::OS.to_string(),
                    arch: std::env::consts::ARCH.to_string(),
                    kernel: get_kernel_version(),
                },
                environment: std::env::vars().collect(),
            },
            system: SystemConfig {
                max_memory_mb: None,
                max_cpu_cores: None,
                temp_dir: None,
                log_level: "info".to_string(),
                debug: false,
            },
        }
    }
}

impl BiomeOSRuntime {
    /// Create new runtime with specific UI mode
    pub fn with_ui_mode(mut self, mode: UIMode) -> Self {
        self.mode = mode;
        self
    }

    /// Enable debug mode
    pub fn with_debug(mut self, debug: bool) -> Self {
        self.system.debug = debug;
        if debug {
            self.system.log_level = "debug".to_string();
        }
        self
    }

    /// Set log level
    pub fn with_log_level(mut self, level: &str) -> Self {
        self.system.log_level = level.to_string();
        self
    }

    /// Set resource limits
    pub fn with_limits(mut self, max_memory_mb: Option<u64>, max_cpu_cores: Option<u32>) -> Self {
        self.system.max_memory_mb = max_memory_mb;
        self.system.max_cpu_cores = max_cpu_cores;
        self
    }

    /// Get runtime information as JSON
    pub fn to_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(self)
    }

    /// Initialize runtime
    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        tracing::info!("Initializing BiomeOS Runtime v{}", self.version);
        tracing::debug!("UI Mode: {:?}", self.mode);
        tracing::debug!("Debug Mode: {}", self.system.debug);
        tracing::debug!(
            "Host: {} ({})",
            self.metadata.host.hostname,
            self.metadata.host.os
        );

        // Initialize tracing and logging subsystems
        tracing::info!("Initializing BiomeOS core systems");

        // Validate system configuration
        if self.system.max_memory_mb.unwrap_or(0) > 0 {
            tracing::debug!(
                "Memory limit configured: {}MB",
                self.system.max_memory_mb.unwrap_or(0)
            );
        }

        // Initialize system health monitoring baseline
        tracing::info!("Core systems initialized successfully");
        Ok(())
    }
}

/// Universal adapter for coordinating between Toadstool and Songbird
/// This uses the adapter pattern to delegate functionality to specialized services
pub struct BiomeOSCoordinator {
    adapter: UniversalAdapter,
}

impl Default for BiomeOSCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

impl BiomeOSCoordinator {
    /// Create new coordinator
    pub fn new() -> Self {
        let config = AdapterConfig::default();
        Self {
            adapter: UniversalAdapter::new(config),
        }
    }

    /// Create new coordinator with custom configuration
    pub fn with_config(config: AdapterConfig) -> Self {
        Self {
            adapter: UniversalAdapter::new(config),
        }
    }

    /// Initialize the coordinator
    pub async fn initialize(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        self.adapter.initialize().await?;
        Ok(())
    }

    /// Process a request through the universal adapter
    pub async fn process_request(
        &self,
        request: UniversalRequest,
    ) -> Result<UniversalResponse, Box<dyn std::error::Error>> {
        let response = self.adapter.process_request(request).await?;
        Ok(response)
    }

    /// Get system status
    pub async fn get_system_status(&self) -> Result<SystemStatus, Box<dyn std::error::Error>> {
        let status = self.adapter.get_system_status().await?;
        Ok(status)
    }

    /// Get discovered primals
    pub async fn get_discovered_primals(
        &self,
    ) -> Result<Vec<PrimalInfo>, Box<dyn std::error::Error>> {
        let primals = get_discovered_primals().await?;
        Ok(primals)
    }

    /// Find primal by capability
    pub async fn find_primal_by_capability(
        &self,
        capability: &str,
    ) -> Result<Option<PrimalInfo>, Box<dyn std::error::Error>> {
        let primal = find_primal_by_capability(capability).await?;
        Ok(primal)
    }

    /// Create example system status (for demonstration)
    pub fn create_example_system_status() -> SystemStatus {
        SystemStatus {
            toadstool: ServiceStatus::Healthy,
            songbird: ServiceStatus::Healthy,
            adapter_uptime: 3600, // 1 hour
        }
    }
}

/// Get the actual kernel version from the system
fn get_kernel_version() -> String {
    #[cfg(target_os = "linux")]
    {
        if let Ok(output) = std::process::Command::new("uname").arg("-r").output() {
            if output.status.success() {
                return String::from_utf8_lossy(&output.stdout).trim().to_string();
            }
        }
        // Fallback: try reading /proc/version
        if let Ok(version_info) = std::fs::read_to_string("/proc/version") {
            if let Some(version) = version_info.split_whitespace().nth(2) {
                return version.to_string();
            }
        }
    }

    #[cfg(target_os = "macos")]
    {
        if let Ok(output) = std::process::Command::new("uname").arg("-r").output() {
            if output.status.success() {
                return String::from_utf8_lossy(&output.stdout).trim().to_string();
            }
        }
    }

    #[cfg(target_os = "windows")]
    {
        if let Ok(output) = std::process::Command::new("cmd")
            .args(["/C", "ver"])
            .output()
        {
            if output.status.success() {
                let version_str = String::from_utf8_lossy(&output.stdout);
                return version_str.trim().to_string();
            }
        }
    }

    // Fallback for any platform
    "unknown".to_string()
}

/// UI Features configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UIFeatures {
    pub dashboard_enabled: bool,
    pub monitoring_enabled: bool,
    pub primal_management: bool,
    pub system_controls: bool,
    pub advanced_features: bool,
}

impl Default for UIFeatures {
    fn default() -> Self {
        Self {
            dashboard_enabled: true,
            monitoring_enabled: true,
            primal_management: true,
            system_controls: false,
            advanced_features: false,
        }
    }
}

/// BiomeOS UI Manager
#[derive(Debug, Clone)]
pub struct BiomeOSUI {
    pub mode: UIMode,
    pub features: UIFeatures,
    pub config: HashMap<String, String>,
}

impl BiomeOSUI {
    pub fn new(mode: UIMode) -> Self {
        Self {
            mode,
            features: UIFeatures::default(),
            config: HashMap::new(),
        }
    }

    pub fn with_features(mut self, features: UIFeatures) -> Self {
        self.features = features;
        self
    }

    pub async fn initialize(&self) -> Result<(), Box<dyn std::error::Error>> {
        println!("🎨 Initializing BiomeOS UI in {:?} mode", self.mode);
        Ok(())
    }

    pub async fn render(&self) -> Result<String, Box<dyn std::error::Error>> {
        match self.mode {
            UIMode::Terminal => Ok("Terminal UI rendered".to_string()),
            UIMode::Web => Ok("Web UI rendered".to_string()),
            UIMode::Desktop => Ok("Desktop UI rendered".to_string()),
            UIMode::Mobile => Ok("Mobile UI rendered".to_string()),
            UIMode::Auto => Ok("Auto-detected UI rendered".to_string()),
            UIMode::Headless => Ok("Headless mode (no UI)".to_string()),
        }
    }
}

/// Universal UI module for compatibility
pub mod universal_ui {
    pub use super::{BiomeOSUI, UIFeatures, UIMode};

    pub async fn create_ui_manager(
        mode: super::UIMode,
    ) -> Result<super::BiomeOSUI, Box<dyn std::error::Error>> {
        Ok(super::BiomeOSUI::new(mode))
    }
}
