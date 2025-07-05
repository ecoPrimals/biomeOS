//! # biomeOS System Services Manager
//!
//! Manages OS-level system services for biomeOS.
//! These are distinct from Primal services and handle core system functionality.

use biomeos_core::BiomeResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::process::Stdio;
use tokio::process::Command;
use tokio::sync::RwLock;

/// System services manager
pub struct SystemServicesManager {
    /// Configuration
    pub config: SystemServicesConfig,
    /// Running services
    pub services: RwLock<HashMap<String, SystemServiceInstance>>,
}

/// System services configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemServicesConfig {
    /// Services to start
    pub services: HashMap<String, SystemServiceConfig>,
    /// Service startup timeout
    pub startup_timeout_seconds: u64,
    /// Service restart policy
    pub restart_policy: RestartPolicy,
    /// Service log directory
    pub log_dir: PathBuf,
}

/// System service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemServiceConfig {
    /// Service name
    pub name: String,
    /// Service description
    pub description: String,
    /// Service executable path
    pub executable: PathBuf,
    /// Service arguments
    pub args: Vec<String>,
    /// Service environment variables
    pub environment: HashMap<String, String>,
    /// Service working directory
    pub working_dir: Option<PathBuf>,
    /// Service user (if different from system user)
    pub user: Option<String>,
    /// Service group (if different from system group)
    pub group: Option<String>,
    /// Service dependencies
    pub dependencies: Vec<String>,
    /// Service is critical (system fails if this fails)
    pub critical: bool,
    /// Service auto-start
    pub auto_start: bool,
    /// Service restart policy
    pub restart_policy: RestartPolicy,
}

/// Service restart policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RestartPolicy {
    /// Never restart
    Never,
    /// Restart on failure
    OnFailure,
    /// Always restart
    Always,
    /// Restart unless stopped
    UnlessStopped,
}

/// System service instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemServiceInstance {
    /// Service configuration
    pub config: SystemServiceConfig,
    /// Service status
    pub status: SystemServiceStatus,
    /// Service process ID
    pub pid: Option<u32>,
    /// Service start time
    pub start_time: Option<chrono::DateTime<chrono::Utc>>,
    /// Service uptime
    pub uptime: Option<std::time::Duration>,
    /// Service restart count
    pub restart_count: u32,
    /// Service last error
    pub last_error: Option<String>,
}

/// System service status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemServiceStatus {
    /// Service state
    pub state: ServiceState,
    /// Service health
    pub health: ServiceHealth,
    /// Service logs
    pub logs: Vec<ServiceLogEntry>,
}

/// Service state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceState {
    /// Service is stopped
    Stopped,
    /// Service is starting
    Starting,
    /// Service is running
    Running,
    /// Service is stopping
    Stopping,
    /// Service has failed
    Failed { reason: String },
    /// Service is in unknown state
    Unknown,
}

/// Service health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServiceHealth {
    /// Service is healthy
    Healthy,
    /// Service is unhealthy
    Unhealthy,
    /// Service health is unknown
    Unknown,
}

/// Service log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceLogEntry {
    /// Log timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
    /// Log level
    pub level: LogLevel,
    /// Log message
    pub message: String,
}

/// Log level
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LogLevel {
    /// Debug
    Debug,
    /// Info
    Info,
    /// Warning
    Warning,
    /// Error
    Error,
}

impl SystemServicesManager {
    /// Create new system services manager
    pub fn new(config: SystemServicesConfig) -> Self {
        Self {
            config,
            services: RwLock::new(HashMap::new()),
        }
    }

    /// Initialize system services manager
    pub async fn initialize(&self) -> BiomeResult<()> {
        tracing::info!("Initializing system services manager");

        // Create log directory if it doesn't exist
        if let Err(e) = tokio::fs::create_dir_all(&self.config.log_dir).await {
            tracing::warn!("Failed to create log directory: {}", e);
        }

        // Initialize service instances
        {
            let mut services = self.services.write().await;
            for (name, config) in &self.config.services {
                let instance = SystemServiceInstance {
                    config: config.clone(),
                    status: SystemServiceStatus {
                        state: ServiceState::Stopped,
                        health: ServiceHealth::Unknown,
                        logs: Vec::new(),
                    },
                    pid: None,
                    start_time: None,
                    uptime: None,
                    restart_count: 0,
                    last_error: None,
                };
                services.insert(name.clone(), instance);
            }
        }

        tracing::info!("System services manager initialized");
        Ok(())
    }

    /// Start system services
    pub async fn start(&self) -> BiomeResult<()> {
        tracing::info!("Starting system services");

        // Start auto-start services in dependency order
        let start_order = self.calculate_start_order().await?;
        
        for service_name in start_order {
            let should_start = {
                let services = self.services.read().await;
                services.get(&service_name)
                    .map(|s| s.config.auto_start)
                    .unwrap_or(false)
            };

            if should_start {
                if let Err(e) = self.start_service(&service_name).await {
                    tracing::error!("Failed to start service {}: {}", service_name, e);
                    
                    // Check if service is critical
                    let is_critical = {
                        let services = self.services.read().await;
                        services.get(&service_name)
                            .map(|s| s.config.critical)
                            .unwrap_or(false)
                    };

                    if is_critical {
                        return Err(biomeos_core::BiomeError::Generic {
                            message: format!("Critical service failed to start: {}", service_name),
                        });
                    }
                }
            }
        }

        tracing::info!("System services started");
        Ok(())
    }

    /// Start a specific service
    pub async fn start_service(&self, service_name: &str) -> BiomeResult<()> {
        tracing::info!("Starting service: {}", service_name);

        let config = {
            let services = self.services.read().await;
            services.get(service_name)
                .ok_or_else(|| biomeos_core::BiomeError::Generic {
                    message: format!("Service not found: {}", service_name),
                })?
                .config.clone()
        };

        // Check dependencies
        for dep in &config.dependencies {
            if !self.is_service_running(dep).await? {
                return Err(biomeos_core::BiomeError::Generic {
                    message: format!("Service dependency not running: {}", dep),
                });
            }
        }

        // Update service state to starting
        {
            let mut services = self.services.write().await;
            if let Some(service) = services.get_mut(service_name) {
                service.status.state = ServiceState::Starting;
            }
        }

        // Start the service process
        let mut cmd = Command::new(&config.executable);
        cmd.args(&config.args)
            .envs(&config.environment)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());

        if let Some(working_dir) = &config.working_dir {
            cmd.current_dir(working_dir);
        }

        let child = cmd.spawn()
            .map_err(|e| biomeos_core::BiomeError::Generic {
                message: format!("Failed to start service {}: {}", service_name, e),
            })?;

        let pid = child.id();
        let start_time = chrono::Utc::now();

        // Update service state to running
        {
            let mut services = self.services.write().await;
            if let Some(service) = services.get_mut(service_name) {
                service.status.state = ServiceState::Running;
                service.status.health = ServiceHealth::Healthy;
                service.pid = pid;
                service.start_time = Some(start_time);
                service.uptime = Some(std::time::Duration::from_secs(0));
            }
        }

        tracing::info!("Service started: {} (PID: {:?})", service_name, pid);
        Ok(())
    }

    /// Stop a specific service
    pub async fn stop_service(&self, service_name: &str) -> BiomeResult<()> {
        tracing::info!("Stopping service: {}", service_name);

        let pid = {
            let services = self.services.read().await;
            services.get(service_name)
                .and_then(|s| s.pid)
                .ok_or_else(|| biomeos_core::BiomeError::Generic {
                    message: format!("Service not running: {}", service_name),
                })?
        };

        // Update service state to stopping
        {
            let mut services = self.services.write().await;
            if let Some(service) = services.get_mut(service_name) {
                service.status.state = ServiceState::Stopping;
            }
        }

        // Send SIGTERM to the process
        unsafe {
            libc::kill(pid as i32, libc::SIGTERM);
        }

        // Wait for process to stop (with timeout)
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        // Update service state to stopped
        {
            let mut services = self.services.write().await;
            if let Some(service) = services.get_mut(service_name) {
                service.status.state = ServiceState::Stopped;
                service.status.health = ServiceHealth::Unknown;
                service.pid = None;
                service.start_time = None;
                service.uptime = None;
            }
        }

        tracing::info!("Service stopped: {}", service_name);
        Ok(())
    }

    /// Check if a service is running
    async fn is_service_running(&self, service_name: &str) -> BiomeResult<bool> {
        let services = self.services.read().await;
        Ok(services.get(service_name)
            .map(|s| matches!(s.status.state, ServiceState::Running))
            .unwrap_or(false))
    }

    /// Calculate service start order based on dependencies
    async fn calculate_start_order(&self) -> BiomeResult<Vec<String>> {
        let services = self.services.read().await;
        let mut order = Vec::new();
        let mut visited = std::collections::HashSet::new();

        // Simple topological sort
        for service_name in services.keys() {
            if !visited.contains(service_name) {
                self.visit_service(service_name, &services, &mut visited, &mut order)?;
            }
        }

        Ok(order)
    }

    /// Visit service for dependency resolution
    fn visit_service(
        &self,
        service_name: &str,
        services: &HashMap<String, SystemServiceInstance>,
        visited: &mut std::collections::HashSet<String>,
        order: &mut Vec<String>,
    ) -> BiomeResult<()> {
        if visited.contains(service_name) {
            return Ok(());
        }

        visited.insert(service_name.to_string());

        if let Some(service) = services.get(service_name) {
            // Visit dependencies first
            for dep in &service.config.dependencies {
                self.visit_service(dep, services, visited, order)?;
            }
        }

        order.push(service_name.to_string());
        Ok(())
    }

    /// Shutdown all services
    pub async fn shutdown(&self) -> BiomeResult<()> {
        tracing::info!("Shutting down system services");

        // Stop all running services
        let service_names: Vec<String> = {
            let services = self.services.read().await;
            services.keys().cloned().collect()
        };

        for service_name in service_names {
            if self.is_service_running(&service_name).await? {
                if let Err(e) = self.stop_service(&service_name).await {
                    tracing::error!("Failed to stop service {}: {}", service_name, e);
                }
            }
        }

        tracing::info!("System services shutdown complete");
        Ok(())
    }

    /// Get service status
    pub async fn get_service_status(&self, service_name: &str) -> Option<SystemServiceStatus> {
        let services = self.services.read().await;
        services.get(service_name).map(|s| s.status.clone())
    }

    /// Get all services status
    pub async fn get_all_services_status(&self) -> HashMap<String, SystemServiceStatus> {
        let services = self.services.read().await;
        services.iter()
            .map(|(name, instance)| (name.clone(), instance.status.clone()))
            .collect()
    }
}

impl Default for SystemServicesConfig {
    fn default() -> Self {
        let log_dir = dirs::data_dir()
            .unwrap_or_else(|| std::path::PathBuf::from("/var/lib"))
            .join("biomeos/logs/services");

        let mut services = HashMap::new();
        
        // Add default system services
        services.insert("syslog".to_string(), SystemServiceConfig {
            name: "syslog".to_string(),
            description: "System logging service".to_string(),
            executable: std::path::PathBuf::from("/usr/sbin/rsyslog"),
            args: vec!["-n".to_string()],
            environment: HashMap::new(),
            working_dir: None,
            user: None,
            group: None,
            dependencies: vec![],
            critical: true,
            auto_start: true,
            restart_policy: RestartPolicy::Always,
        });

        Self {
            services,
            startup_timeout_seconds: 60,
            restart_policy: RestartPolicy::OnFailure,
            log_dir,
        }
    }
} 