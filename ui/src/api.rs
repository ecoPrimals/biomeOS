//! biomeOS API Integration Layer
//! 
//! This module provides the API abstraction layer for the biomeOS UI to communicate
//! with the core biomeOS system and ecosystem primals. Follows API-driven architecture.

use std::collections::HashMap;
use std::sync::Arc;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use biomeos_core::*;

use crate::state::*;

/// Main API client for biomeOS core integration
pub struct BiomeOSApi {
    /// Core biomeOS manager
    core: Arc<Mutex<Option<UniversalBiomeManager>>>,
    
    /// API endpoints for different services
    endpoints: HashMap<String, String>,
    
    /// HTTP client for external API calls
    client: reqwest::Client,
    
    /// Connection status
    connected: Arc<Mutex<bool>>,
}

impl BiomeOSApi {
    pub fn new() -> Self {
        let mut endpoints = HashMap::new();
        endpoints.insert("core".to_string(), "http://localhost:8080".to_string());
        endpoints.insert("metrics".to_string(), "http://localhost:8081".to_string());
        
        Self {
            core: Arc::new(Mutex::new(None)),
            endpoints,
            client: reqwest::Client::new(),
            connected: Arc::new(Mutex::new(false)),
        }
    }

    /// Initialize connection to biomeOS core
    pub async fn initialize(&self) -> Result<()> {
        let config = biomeos_core::BiomeOSConfig::default();
        let manager = UniversalBiomeManager::new(config);
        
        {
            let mut core = self.core.lock().await;
            *core = Some(manager);
        }
        
        {
            let mut connected = self.connected.lock().await;
            *connected = true;
        }
        
        Ok(())
    }

    /// Check if API is connected
    pub async fn is_connected(&self) -> bool {
        *self.connected.lock().await
    }

    /// Get system status
    pub async fn get_system_status(&self) -> Result<SystemStatusResponse> {
        // For now, return mock data
        // In real implementation, this would call biomeOS core APIs
        Ok(SystemStatusResponse {
            status: "online".to_string(),
            uptime: std::time::Duration::from_secs(3600),
            version: env!("CARGO_PKG_VERSION").to_string(),
            platform: self.get_platform_info().await?,
        })
    }

    /// Get platform information
    pub async fn get_platform_info(&self) -> Result<PlatformInfoResponse> {
        Ok(PlatformInfoResponse {
            os_type: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            cores: num_cpus::get() as u32,
            memory_gb: 8, // Mock data
            container_runtime: self.detect_container_runtime().await,
        })
    }

    /// Start installation process
    pub async fn start_installation(&self, mode: InstallationMode) -> Result<InstallationResponse> {
        // Initialize biomeOS if not already done
        if !self.is_connected().await {
            self.initialize().await?;
        }

        // Mock installation process
        Ok(InstallationResponse {
            installation_id: uuid::Uuid::new_v4().to_string(),
            status: "started".to_string(),
            estimated_duration: std::time::Duration::from_secs(300),
        })
    }

    /// Get installation progress
    pub async fn get_installation_progress(&self, installation_id: &str) -> Result<InstallationProgress> {
        // Mock progress data
        Ok(InstallationProgress {
            installation_id: installation_id.to_string(),
            current_step: "Platform Detection".to_string(),
            progress: 0.25,
            status: "in_progress".to_string(),
            ai_guidance: "Detecting your system capabilities...".to_string(),
            errors: Vec::new(),
        })
    }

    /// Discover available primals
    pub async fn discover_primals(&self) -> Result<Vec<PrimalDiscoveryResult>> {
        // Mock primal discovery
        Ok(vec![
            PrimalDiscoveryResult {
                id: "toadstool".to_string(),
                name: "Toadstool Universal Compute".to_string(),
                version: "0.1.0".to_string(),
                description: "Universal compute substrate".to_string(),
                capabilities: vec![
                    "container_runtime".to_string(),
                    "process_isolation".to_string(),
                    "resource_management".to_string(),
                ],
                status: "available".to_string(),
                api_endpoints: vec!["http://localhost:8082".to_string()],
            },
            PrimalDiscoveryResult {
                id: "songbird".to_string(),
                name: "Songbird Service Mesh".to_string(),
                version: "0.1.0".to_string(),
                description: "Service discovery and mesh networking".to_string(),
                capabilities: vec![
                    "service_discovery".to_string(),
                    "load_balancing".to_string(),
                    "traffic_routing".to_string(),
                ],
                status: "available".to_string(),
                api_endpoints: vec!["http://localhost:8083".to_string()],
            },
        ])
    }

    /// Get primal status
    pub async fn get_primal_status(&self, primal_id: &str) -> Result<PrimalStatusResponse> {
        Ok(PrimalStatusResponse {
            id: primal_id.to_string(),
            status: "running".to_string(),
            health: "healthy".to_string(),
            last_heartbeat: chrono::Utc::now(),
            metrics: PrimalMetricsResponse {
                cpu_usage: 0.15,
                memory_usage: 0.25,
                network_io: (1024, 2048),
                api_requests: 150,
                error_count: 0,
            },
        })
    }

    /// Get sovereignty assessment
    pub async fn get_sovereignty_assessment(&self) -> Result<SovereigntyAssessment> {
        // Mock sovereignty data
        Ok(SovereigntyAssessment {
            compliance_score: 3,
            sovereignty_level: "sovereign".to_string(),
            crypto_locks: CryptoLockAssessment {
                enabled: true,
                active_locks: 5,
                bypassed_locks: 0,
                bypass_reasons: Vec::new(),
            },
            genetic_keys: GeneticKeyAssessment {
                enabled: true,
                active_keys: 1,
                access_level: "individual".to_string(),
                cost_multiplier: 1.0,
            },
            ai_cat_door: AiCatDoorAssessment {
                enabled: true,
                cost_protection: 20.0,
                requests_used: 15,
                requests_limit: 100,
            },
            dependencies: vec![
                DependencyAssessmentResponse {
                    name: "Operating System".to_string(),
                    dependency_type: "platform".to_string(),
                    sovereignty_impact: "none".to_string(),
                    alternatives: vec!["Any OS".to_string()],
                    bypass_available: true,
                },
            ],
            threats: Vec::new(),
        })
    }

    /// Get real-time metrics
    pub async fn get_metrics(&self) -> Result<MetricsResponse> {
        use std::fs;
        use std::io::Read;
        
        // Get real CPU usage from /proc/stat
        let cpu_usage = Self::get_cpu_usage().await.unwrap_or(0.0);
        
        // Get real memory usage from /proc/meminfo
        let memory_usage = Self::get_memory_usage().await.unwrap_or(0.0);
        
        // Get real disk usage
        let disk_usage = Self::get_disk_usage().await.unwrap_or(0.0);
        
        // Get real network I/O from /proc/net/dev
        let network_io = Self::get_network_io().await.unwrap_or((0, 0));
        
        // Get real uptime from /proc/uptime
        let uptime = Self::get_uptime().await.unwrap_or(std::time::Duration::from_secs(0));
        
        Ok(MetricsResponse {
            uptime,
            cpu_usage,
            memory_usage,
            disk_usage,
            network_io,
            api_requests_per_second: 0.0, // TODO: Implement real API request tracking
            active_connections: Self::get_active_connections().await.unwrap_or(0),
            timestamp: chrono::Utc::now(),
        })
    }
    
    async fn get_cpu_usage() -> std::io::Result<f32> {
        use std::fs;
        use std::thread::sleep;
        use std::time::Duration;
        
        // Read /proc/stat twice with a small delay to calculate CPU usage
        let stat1 = fs::read_to_string("/proc/stat")?;
        sleep(Duration::from_millis(100));
        let stat2 = fs::read_to_string("/proc/stat")?;
        
        let parse_cpu_line = |line: &str| -> Option<(u64, u64)> {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 && parts[0] == "cpu" {
                let user: u64 = parts[1].parse().ok()?;
                let nice: u64 = parts[2].parse().ok()?;
                let system: u64 = parts[3].parse().ok()?;
                let idle: u64 = parts[4].parse().ok()?;
                let iowait: u64 = parts.get(5)?.parse().ok().unwrap_or(0);
                let irq: u64 = parts.get(6)?.parse().ok().unwrap_or(0);
                let softirq: u64 = parts.get(7)?.parse().ok().unwrap_or(0);
                
                let total_idle = idle + iowait;
                let total_non_idle = user + nice + system + irq + softirq;
                let total = total_idle + total_non_idle;
                
                Some((total, total_idle))
            } else {
                None
            }
        };
        
        let (total1, idle1) = stat1.lines().next()
            .and_then(parse_cpu_line)
            .unwrap_or((0, 0));
        let (total2, idle2) = stat2.lines().next()
            .and_then(parse_cpu_line)
            .unwrap_or((0, 0));
        
        let total_diff = total2.saturating_sub(total1);
        let idle_diff = idle2.saturating_sub(idle1);
        
        if total_diff == 0 {
            return Ok(0.0);
        }
        
        let cpu_usage = 100.0 * (1.0 - (idle_diff as f32 / total_diff as f32));
        Ok(cpu_usage.clamp(0.0, 100.0))
    }
    
    async fn get_memory_usage() -> std::io::Result<f32> {
        use std::fs;
        
        let meminfo = fs::read_to_string("/proc/meminfo")?;
        let mut total_kb = 0u64;
        let mut available_kb = 0u64;
        
        for line in meminfo.lines() {
            if line.starts_with("MemTotal:") {
                total_kb = line.split_whitespace().nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            } else if line.starts_with("MemAvailable:") {
                available_kb = line.split_whitespace().nth(1)
                    .and_then(|s| s.parse().ok())
                    .unwrap_or(0);
            }
        }
        
        if total_kb == 0 {
            return Ok(0.0);
        }
        
        let used_kb = total_kb.saturating_sub(available_kb);
        let usage_percent = (used_kb as f32 / total_kb as f32) * 100.0;
        Ok(usage_percent.clamp(0.0, 100.0))
    }
    
    async fn get_disk_usage() -> std::io::Result<f32> {
        use std::process::Command;
        
        // Use df command to get disk usage for root filesystem
        let output = Command::new("df")
            .args(&["-h", "/"])
            .output()?;
        
        if !output.status.success() {
            return Ok(0.0);
        }
        
        let output_str = String::from_utf8_lossy(&output.stdout);
        for line in output_str.lines().skip(1) { // Skip header
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 5 {
                if let Some(usage_str) = parts.get(4) {
                    if let Some(percent_str) = usage_str.strip_suffix('%') {
                        if let Ok(percent) = percent_str.parse::<f32>() {
                            return Ok(percent.clamp(0.0, 100.0));
                        }
                    }
                }
            }
        }
        
        Ok(0.0)
    }
    
    async fn get_network_io() -> std::io::Result<(u64, u64)> {
        use std::fs;
        
        let net_dev = fs::read_to_string("/proc/net/dev")?;
        let mut total_rx = 0u64;
        let mut total_tx = 0u64;
        
        for line in net_dev.lines().skip(2) { // Skip header lines
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 10 {
                // Skip loopback interface
                if let Some(interface) = parts.get(0) {
                    if interface.starts_with("lo:") {
                        continue;
                    }
                }
                
                if let (Ok(rx), Ok(tx)) = (parts[1].parse::<u64>(), parts[9].parse::<u64>()) {
                    total_rx += rx;
                    total_tx += tx;
                }
            }
        }
        
        Ok((total_rx, total_tx))
    }
    
    async fn get_uptime() -> std::io::Result<std::time::Duration> {
        use std::fs;
        
        let uptime_str = fs::read_to_string("/proc/uptime")?;
        let uptime_seconds = uptime_str
            .split_whitespace()
            .next()
            .and_then(|s| s.parse::<f64>().ok())
            .unwrap_or(0.0);
        
        Ok(std::time::Duration::from_secs_f64(uptime_seconds))
    }
    
    async fn get_active_connections() -> std::io::Result<u32> {
        use std::fs;
        
        // Count TCP connections in ESTABLISHED state
        let tcp = fs::read_to_string("/proc/net/tcp")?;
        let tcp6 = fs::read_to_string("/proc/net/tcp6").unwrap_or_default();
        
        let count_established = |content: &str| -> u32 {
            content.lines()
                .skip(1) // Skip header
                .filter(|line| {
                    line.split_whitespace()
                        .nth(3) // State column
                        .map(|state| state == "01") // ESTABLISHED = 01
                        .unwrap_or(false)
                })
                .count() as u32
        };
        
        Ok(count_established(&tcp) + count_established(&tcp6))
    }

    /// Trigger primal installation
    pub async fn install_primal(&self, primal_id: &str, config: PrimalInstallConfig) -> Result<InstallationResponse> {
        Ok(InstallationResponse {
            installation_id: uuid::Uuid::new_v4().to_string(),
            status: "started".to_string(),
            estimated_duration: std::time::Duration::from_secs(180),
        })
    }

    /// Configure AI cat door
    pub async fn configure_ai_cat_door(&self, config: AiCatDoorConfig) -> Result<()> {
        // Implementation would configure AI cat door settings
        Ok(())
    }

    /// Update genetic key settings
    pub async fn update_genetic_keys(&self, config: GeneticKeyConfig) -> Result<()> {
        // Implementation would update genetic key configuration
        Ok(())
    }

    /// Detect container runtime
    async fn detect_container_runtime(&self) -> Option<String> {
        // Try to detect Docker
        if let Ok(output) = tokio::process::Command::new("docker")
            .arg("--version")
            .output()
            .await 
        {
            if output.status.success() {
                return Some("docker".to_string());
            }
        }

        // Try to detect Podman
        if let Ok(output) = tokio::process::Command::new("podman")
            .arg("--version")
            .output()
            .await 
        {
            if output.status.success() {
                return Some("podman".to_string());
            }
        }

        None
    }

    /// Initialize biomeOS with configuration
    pub async fn initialize_biome(&self, mode: InstallationMode) -> Result<InitializationResponse> {
        // Create a basic BiomeOSConfig for initialization
        let config = biomeos_core::BiomeOSConfig::default();
        let manager = UniversalBiomeManager::new(config);
        
        {
            let mut core = self.core.lock().await;
            *core = Some(manager);
        }
        
        {
            let mut connected = self.connected.lock().await;
            *connected = true;
        }
        
        Ok(InitializationResponse {
            status: "initialized".to_string(),
            message: "BiomeOS initialized successfully".to_string(),
        })
    }
}

// API Response Types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemStatusResponse {
    pub status: String,
    pub uptime: std::time::Duration,
    pub version: String,
    pub platform: PlatformInfoResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlatformInfoResponse {
    pub os_type: String,
    pub architecture: String,
    pub cores: u32,
    pub memory_gb: u32,
    pub container_runtime: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationResponse {
    pub installation_id: String,
    pub status: String,
    pub estimated_duration: std::time::Duration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstallationProgress {
    pub installation_id: String,
    pub current_step: String,
    pub progress: f32,
    pub status: String,
    pub ai_guidance: String,
    pub errors: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalDiscoveryResult {
    pub id: String,
    pub name: String,
    pub version: String,
    pub description: String,
    pub capabilities: Vec<String>,
    pub status: String,
    pub api_endpoints: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalStatusResponse {
    pub id: String,
    pub status: String,
    pub health: String,
    pub last_heartbeat: chrono::DateTime<chrono::Utc>,
    pub metrics: PrimalMetricsResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalMetricsResponse {
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub network_io: (u64, u64),
    pub api_requests: u64,
    pub error_count: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereigntyAssessment {
    pub compliance_score: u8,
    pub sovereignty_level: String,
    pub crypto_locks: CryptoLockAssessment,
    pub genetic_keys: GeneticKeyAssessment,
    pub ai_cat_door: AiCatDoorAssessment,
    pub dependencies: Vec<DependencyAssessmentResponse>,
    pub threats: Vec<ThreatAssessmentResponse>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CryptoLockAssessment {
    pub enabled: bool,
    pub active_locks: u32,
    pub bypassed_locks: u32,
    pub bypass_reasons: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneticKeyAssessment {
    pub enabled: bool,
    pub active_keys: u32,
    pub access_level: String,
    pub cost_multiplier: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiCatDoorAssessment {
    pub enabled: bool,
    pub cost_protection: f32,
    pub requests_used: u32,
    pub requests_limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyAssessmentResponse {
    pub name: String,
    pub dependency_type: String,
    pub sovereignty_impact: String,
    pub alternatives: Vec<String>,
    pub bypass_available: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatAssessmentResponse {
    pub id: String,
    pub threat_type: String,
    pub severity: String,
    pub description: String,
    pub mitigation: Option<String>,
    pub detected_at: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricsResponse {
    pub uptime: std::time::Duration,
    pub cpu_usage: f32,
    pub memory_usage: f32,
    pub disk_usage: f32,
    pub network_io: (u64, u64),
    pub api_requests_per_second: f32,
    pub active_connections: u32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

// Configuration Types

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInstallConfig {
    pub auto_start: bool,
    pub resource_limits: Option<ResourceLimits>,
    pub custom_settings: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceLimits {
    pub cpu_limit: Option<f32>,
    pub memory_limit: Option<u64>,
    pub disk_limit: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiCatDoorConfig {
    pub enabled: bool,
    pub cost_protection: f32,
    pub request_limit: u32,
    pub providers: Vec<AiProviderConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiProviderConfig {
    pub name: String,
    pub enabled: bool,
    pub api_key: Option<String>,
    pub endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneticKeyConfig {
    pub enabled: bool,
    pub access_level: String,
    pub partnership_endpoint: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializationResponse {
    pub status: String,
    pub message: String,
} 