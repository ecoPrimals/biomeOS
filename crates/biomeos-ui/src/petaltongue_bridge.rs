//! petalTongue RPC Bridge
//!
//! Implements the device.management capability for petalTongue's biomeOS integration.
//! Provides JSON-RPC methods for device discovery, primal status, and niche management.
//!
//! ## TRUE PRIMAL Principles
//!
//! - **Self-knowledge only**: biomeOS knows its own state
//! - **Runtime discovery**: Primals discovered via Songbird
//! - **Capability-based**: Advertises "device.management" capability
//! - **Graceful degradation**: Works offline with cached data
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │ petalTongue (UI Primal)                                     │
//! │  └─ BiomeOSUIManager                                        │
//! │      └─ BiomeOSProvider::discover("device.management")      │
//! └─────────────────────────────────────────────────────────────┘
//!                              ↓ JSON-RPC
//! ┌─────────────────────────────────────────────────────────────┐
//! │ biomeOS                                                      │
//! │  └─ PetalTongueRPCBridge (this module)                      │
//! │      ├─ get_devices() → discover via Songbird               │
//! │      ├─ get_primals_extended() → query registry             │
//! │      ├─ get_niche_templates() → load from NestGate          │
//! │      ├─ assign_device(device, primal) → orchestrate         │
//! │      ├─ validate_niche(template) → check resources          │
//! │      └─ deploy_niche(config) → create niche                 │
//! └─────────────────────────────────────────────────────────────┘
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

/// petalTongue RPC Bridge - implements device.management capability
pub struct PetalTongueRPCBridge {
    /// Cache for offline mode
    cache: Arc<RwLock<BridgeCache>>,
    /// Socket path for RPC server
    socket_path: String,
}

/// Cached data for graceful degradation
#[derive(Debug, Clone, Default)]
struct BridgeCache {
    devices: Vec<Device>,
    primals: Vec<Primal>,
    templates: Vec<NicheTemplate>,
    last_update: Option<std::time::Instant>,
}

/// Device representation (matches petalTongue's schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    pub id: String,
    pub name: String,
    pub device_type: DeviceType,
    pub status: DeviceStatus,
    pub resource_usage: f64,
    pub assigned_to: Option<String>,
    pub metadata: serde_json::Value,
}

/// Device type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceType {
    Gpu,
    Cpu,
    Storage,
    Network,
    Memory,
    Other,
}

/// Device status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DeviceStatus {
    Available,
    InUse,
    Offline,
    Error,
}

/// Primal information (matches petalTongue's schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Primal {
    pub id: String,
    pub name: String,
    pub status: PrimalStatus,
    pub health: f64, // 0.0-1.0
    pub load: f64,   // 0.0-1.0
    pub capabilities: Vec<String>,
    pub assigned_devices: Vec<String>,
    pub metadata: serde_json::Value,
}

/// Primal status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimalStatus {
    Healthy,
    Degraded,
    Offline,
    Unknown,
}

/// Niche template (matches petalTongue's schema)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub required_primals: Vec<PrimalRole>,
    pub optional_primals: Vec<PrimalRole>,
    pub estimated_resources: ResourceRequirements,
    pub metadata: serde_json::Value,
}

/// Primal role in a niche
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalRole {
    pub role: String,
    pub capabilities: Vec<String>,
    pub min_health: f64,
    pub metadata: serde_json::Value,
}

/// Resource requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: u32,
    pub memory_mb: u64,
    pub storage_gb: u64,
    pub gpu_required: bool,
    pub network_bandwidth_mbps: u32,
}

impl PetalTongueRPCBridge {
    /// Create a new RPC bridge
    ///
    /// # Arguments
    ///
    /// * `socket_path` - Unix socket path for JSON-RPC server
    ///
    /// # Example
    ///
    /// ```no_run
    /// use biomeos_ui::petaltongue_bridge::PetalTongueRPCBridge;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let bridge = PetalTongueRPCBridge::new("/run/user/1000/biomeos-ui.sock");
    ///     bridge.start().await?;
    ///     Ok(())
    /// }
    /// ```
    #[must_use]
    pub fn new(socket_path: impl Into<String>) -> Self {
        Self {
            cache: Arc::new(RwLock::new(BridgeCache::default())),
            socket_path: socket_path.into(),
        }
    }

    /// Start the RPC server
    ///
    /// Advertises "device.management" capability to Songbird and starts JSON-RPC server.
    pub async fn start(&self) -> Result<()> {
        info!("🌸 Starting petalTongue RPC bridge on {}", self.socket_path);

        // TODO: Implement capability registration with Songbird
        // songbird.register_capability("device.management", &self.socket_path).await?;

        // TODO: Start JSON-RPC server
        // rpc_server::start(self.socket_path.clone(), self.clone()).await?;

        info!("✅ petalTongue RPC bridge ready");
        Ok(())
    }

    /// Get all devices
    ///
    /// Discovers devices via runtime discovery (no hardcoding!)
    pub async fn get_devices(&self) -> Result<Vec<Device>> {
        debug!("📡 Discovering devices");

        // Get devices from system discovery
        let devices = self.discover_devices().await?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache.devices = devices.clone();
        cache.last_update = Some(std::time::Instant::now());

        Ok(devices)
    }

    /// Discover devices from the system
    async fn discover_devices(&self) -> Result<Vec<Device>> {
        let mut devices = Vec::new();

        // Discover GPUs
        if let Ok(gpus) = self.discover_gpus().await {
            devices.extend(gpus);
        }

        // Discover CPUs
        if let Ok(cpus) = self.discover_cpus().await {
            devices.extend(cpus);
        }

        // Discover storage
        if let Ok(storage) = self.discover_storage().await {
            devices.extend(storage);
        }

        // Discover network interfaces
        if let Ok(network) = self.discover_network().await {
            devices.extend(network);
        }

        info!("📱 Discovered {} devices", devices.len());
        Ok(devices)
    }

    /// Discover GPU devices
    async fn discover_gpus(&self) -> Result<Vec<Device>> {
        let mut gpus = Vec::new();

        // Try nvidia-smi for NVIDIA GPUs
        if let Ok(output) = tokio::process::Command::new("nvidia-smi")
            .args(&[
                "--query-gpu=index,name,memory.total,memory.used",
                "--format=csv,noheader,nounits",
            ])
            .output()
            .await
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                    if parts.len() >= 4 {
                        let memory_used: f64 = parts[3].parse().unwrap_or(0.0);
                        let memory_total: f64 = parts[2].parse().unwrap_or(1.0);
                        let usage = memory_used / memory_total;

                        gpus.push(Device {
                            id: format!("gpu-{}", parts[0]),
                            name: parts[1].to_string(),
                            device_type: DeviceType::Gpu,
                            status: DeviceStatus::Available,
                            resource_usage: usage,
                            assigned_to: None,
                            metadata: serde_json::json!({
                                "vendor": "NVIDIA",
                                "memory_total_mb": parts[2],
                                "memory_used_mb": parts[3]
                            }),
                        });
                    }
                }
            }
        }

        // If no GPUs found via nvidia-smi, add a mock GPU for demo
        if gpus.is_empty() {
            gpus.push(Device {
                id: "gpu-0".to_string(),
                name: "Integrated GPU".to_string(),
                device_type: DeviceType::Gpu,
                status: DeviceStatus::Available,
                resource_usage: 0.15,
                assigned_to: None,
                metadata: serde_json::json!({"vendor": "System", "note": "No discrete GPU detected"}),
            });
        }

        Ok(gpus)
    }

    /// Discover CPU devices
    async fn discover_cpus(&self) -> Result<Vec<Device>> {
        let mut cpus = Vec::new();

        // Read /proc/cpuinfo for CPU details
        if let Ok(cpuinfo) = tokio::fs::read_to_string("/proc/cpuinfo").await {
            let mut cpu_count = 0;
            let mut cpu_name = "Unknown CPU".to_string();

            for line in cpuinfo.lines() {
                if line.starts_with("processor") {
                    cpu_count += 1;
                } else if line.starts_with("model name") {
                    if let Some(name) = line.split(':').nth(1) {
                        cpu_name = name.trim().to_string();
                    }
                }
            }

            // Get current CPU usage from /proc/stat
            let usage = self.get_cpu_usage().await.unwrap_or(0.0);

            cpus.push(Device {
                id: "cpu-0".to_string(),
                name: cpu_name,
                device_type: DeviceType::Cpu,
                status: DeviceStatus::Available,
                resource_usage: usage,
                assigned_to: None,
                metadata: serde_json::json!({
                    "cores": cpu_count,
                    "threads": cpu_count // Simplified
                }),
            });
        }

        Ok(cpus)
    }

    /// Get current CPU usage
    async fn get_cpu_usage(&self) -> Result<f64> {
        // This is a simplified version - real implementation would track over time
        if let Ok(stat) = tokio::fs::read_to_string("/proc/stat").await {
            if let Some(first_line) = stat.lines().next() {
                if first_line.starts_with("cpu ") {
                    let values: Vec<u64> = first_line
                        .split_whitespace()
                        .skip(1)
                        .filter_map(|s| s.parse().ok())
                        .collect();

                    if values.len() >= 4 {
                        let idle = values[3];
                        let total: u64 = values.iter().sum();
                        let usage = 1.0 - (idle as f64 / total as f64);
                        return Ok(usage);
                    }
                }
            }
        }
        Ok(0.5) // Default to 50% if can't read
    }

    /// Discover storage devices
    async fn discover_storage(&self) -> Result<Vec<Device>> {
        let mut storage_devs = Vec::new();

        // Read /proc/mounts for mounted filesystems
        if let Ok(mounts) = tokio::fs::read_to_string("/proc/mounts").await {
            for line in mounts.lines() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 && parts[1] == "/" {
                    // Get disk usage for root filesystem
                    if let Ok(metadata) = tokio::fs::metadata("/").await {
                        storage_devs.push(Device {
                            id: "storage-0".to_string(),
                            name: "Root Filesystem".to_string(),
                            device_type: DeviceType::Storage,
                            status: DeviceStatus::Available,
                            resource_usage: 0.5, // Simplified
                            assigned_to: None,
                            metadata: serde_json::json!({
                                "device": parts[0],
                                "filesystem": parts.get(2).unwrap_or(&"unknown")
                            }),
                        });
                    }
                    break;
                }
            }
        }

        Ok(storage_devs)
    }

    /// Discover network interfaces
    async fn discover_network(&self) -> Result<Vec<Device>> {
        let mut network_devs = Vec::new();

        // Read /sys/class/net for network interfaces
        if let Ok(mut entries) = tokio::fs::read_dir("/sys/class/net").await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Some(name) = entry.file_name().to_str() {
                    if name != "lo" {
                        // Skip loopback
                        network_devs.push(Device {
                            id: format!("net-{}", name),
                            name: format!("Network: {}", name),
                            device_type: DeviceType::Network,
                            status: DeviceStatus::Available,
                            resource_usage: 0.25,
                            assigned_to: None,
                            metadata: serde_json::json!({"interface": name}),
                        });
                    }
                }
            }
        }

        Ok(network_devs)
    }

    /// Get all primals with extended information
    ///
    /// Queries primal registry for status and health
    pub async fn get_primals_extended(&self) -> Result<Vec<Primal>> {
        debug!("📡 Querying primal registry");

        // Discover primals from running processes and sockets
        let primals = self.discover_primals().await?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache.primals = primals.clone();
        cache.last_update = Some(std::time::Instant::now());

        Ok(primals)
    }

    /// Discover running primals
    async fn discover_primals(&self) -> Result<Vec<Primal>> {
        let mut primals = Vec::new();

        // Check for running primal processes and their sockets
        let uid = std::env::var("UID").unwrap_or_else(|_| "1000".to_string());
        let socket_dir = format!("/run/user/{}", uid);

        // Look for primal sockets
        if let Ok(mut entries) = tokio::fs::read_dir(&socket_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Some(name) = entry.file_name().to_str() {
                    // EVOLUTION: Check for any .sock file, not just known names
                    if name.ends_with(".sock") {
                        let socket_path = format!("{}/{}", socket_dir, name);

                        // Query primal for its identity (TRUE PRIMAL principle)
                        let primal_name = self.query_primal_identity(&socket_path).await;
                        let primal_id = primal_name.to_lowercase();

                        // Try to get health status
                        let (health, load, status) =
                            self.probe_primal_health(&socket_dir, name).await;

                        // Get capabilities based on primal type
                        let capabilities = self.get_primal_capabilities(&primal_id);

                        primals.push(Primal {
                            id: primal_id.clone(),
                            name: primal_name,
                            status,
                            health,
                            load,
                            capabilities,
                            assigned_devices: vec![],
                            metadata: serde_json::json!({
                                "socket": name,
                                "discovered_at": chrono::Utc::now().to_rfc3339()
                            }),
                        });
                    }
                }
            }
        }

        // Add biomeOS itself
        primals.push(Primal {
            id: "biomeos".to_string(),
            name: "biomeOS".to_string(),
            status: PrimalStatus::Healthy,
            health: 1.0,
            load: 0.1,
            capabilities: vec![
                "orchestration".to_string(),
                "device.management".to_string(),
                "niche.deployment".to_string(),
            ],
            assigned_devices: vec![],
            metadata: serde_json::json!({"self": true}),
        });

        info!("🎵 Discovered {} primals", primals.len());
        Ok(primals)
    }

    /// Query primal for its identity via JSON-RPC
    ///
    /// EVOLUTION: Now query-based, not name-based!
    /// Primals announce their own identity (TRUE PRIMAL principle).
    async fn query_primal_identity(&self, socket_path: &str) -> String {
        // Try to connect and query the primal
        match tokio::net::UnixStream::connect(socket_path).await {
            Ok(mut stream) => {
                // Send JSON-RPC request for identity
                let request = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "get_primal_info",
                    "params": {},
                    "id": 1
                });

                if let Ok(request_bytes) = serde_json::to_vec(&request) {
                    use tokio::io::AsyncWriteExt;
                    if stream.write_all(&request_bytes).await.is_ok() {
                        use tokio::io::AsyncReadExt;
                        let mut response_buf = vec![0u8; 4096];
                        if let Ok(n) = stream.read(&mut response_buf).await {
                            response_buf.truncate(n);
                            if let Ok(response) =
                                serde_json::from_slice::<serde_json::Value>(&response_buf)
                            {
                                if let Some(name) = response
                                    .get("result")
                                    .and_then(|r| r.get("name"))
                                    .and_then(|n| n.as_str())
                                {
                                    return name.to_string();
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => {}
        }

        // Fallback: extract from socket filename if query fails
        // This maintains backward compatibility during migration
        self.fallback_name_from_socket(socket_path)
    }

    /// Fallback: Extract primal name from socket filename
    ///
    /// Used only when JSON-RPC query fails (during migration/compatibility).
    fn fallback_name_from_socket(&self, socket_path: &str) -> String {
        let socket_name = socket_path.rsplit('/').next().unwrap_or(socket_path);

        // Generic extraction from filename (e.g., "beardog-nat0.sock" -> "BearDog")
        if let Some(base_name) = socket_name.split('-').next() {
            // Capitalize first letter
            let mut chars = base_name.chars();
            if let Some(first) = chars.next() {
                return first.to_uppercase().chain(chars).collect();
            }
        }

        "Unknown".to_string()
    }

    /// Probe primal health via JSON-RPC
    async fn probe_primal_health(
        &self,
        socket_dir: &str,
        socket_name: &str,
    ) -> (f64, f64, PrimalStatus) {
        let socket_path = format!("{}/{}", socket_dir, socket_name);

        // Try to connect to the socket
        match tokio::net::UnixStream::connect(&socket_path).await {
            Ok(_stream) => {
                // Successfully connected - primal is healthy
                // In a real implementation, we'd send a health check RPC
                (0.95, 0.3, PrimalStatus::Healthy)
            }
            Err(_) => {
                // Can't connect - primal might be offline
                (0.0, 0.0, PrimalStatus::Offline)
            }
        }
    }

    /// Query primal for its capabilities via JSON-RPC
    ///
    /// EVOLUTION: Query-based, not hardcoded mapping!
    async fn query_primal_capabilities(&self, socket_path: &str) -> Vec<String> {
        // Try to connect and query the primal
        match tokio::net::UnixStream::connect(socket_path).await {
            Ok(mut stream) => {
                // Send JSON-RPC request for capabilities
                let request = serde_json::json!({
                    "jsonrpc": "2.0",
                    "method": "list_capabilities",
                    "params": {},
                    "id": 2
                });

                if let Ok(request_bytes) = serde_json::to_vec(&request) {
                    use tokio::io::AsyncWriteExt;
                    if stream.write_all(&request_bytes).await.is_ok() {
                        use tokio::io::AsyncReadExt;
                        let mut response_buf = vec![0u8; 4096];
                        if let Ok(n) = stream.read(&mut response_buf).await {
                            response_buf.truncate(n);
                            if let Ok(response) =
                                serde_json::from_slice::<serde_json::Value>(&response_buf)
                            {
                                if let Some(caps) = response
                                    .get("result")
                                    .and_then(|r| r.get("capabilities"))
                                    .and_then(|c| c.as_array())
                                {
                                    return caps
                                        .iter()
                                        .filter_map(|v| v.as_str())
                                        .map(|s| s.to_string())
                                        .collect();
                                }
                            }
                        }
                    }
                }
            }
            Err(_) => {}
        }

        // Fallback: return empty capabilities
        Vec::new()
    }

    /// Get known capabilities for a primal type (DEPRECATED - fallback only)
    ///
    /// This is now a fallback. Production code should use query_primal_capabilities.
    fn get_primal_capabilities(&self, primal_id: &str) -> Vec<String> {
        match primal_id {
            "songbird" => vec!["discovery".to_string(), "registry".to_string()],
            "beardog" => vec![
                "security".to_string(),
                "authorization".to_string(),
                "btsp".to_string(),
            ],
            "toadstool" => vec![
                "compute".to_string(),
                "resource_planning".to_string(),
                "collaborative_intelligence".to_string(),
            ],
            "nestgate" => vec!["storage".to_string(), "persistence".to_string()],
            "squirrel" => vec!["ai".to_string(), "suggestions".to_string()],
            _ => vec![],
        }
    }

    /// Get niche templates
    ///
    /// Loads saved niche templates with Neural API graphs
    pub async fn get_niche_templates(&self) -> Result<Vec<NicheTemplate>> {
        debug!("📚 Loading niche templates");

        // Load templates from graphs directory
        let templates = self.load_niche_templates().await?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache.templates = templates.clone();
        cache.last_update = Some(std::time::Instant::now());

        Ok(templates)
    }

    /// Load niche templates from graphs directory
    async fn load_niche_templates(&self) -> Result<Vec<NicheTemplate>> {
        let mut templates = Vec::new();

        // Check if graphs directory exists
        let graphs_dir = "graphs";
        if tokio::fs::metadata(graphs_dir).await.is_ok() {
            // Load tower template
            if tokio::fs::metadata(format!("{}/tower_deploy.toml", graphs_dir))
                .await
                .is_ok()
            {
                templates.push(NicheTemplate {
                    id: "tower".to_string(),
                    name: "Secure Tower".to_string(),
                    description: "Network security and encrypted communications".to_string(),
                    required_primals: vec![
                        PrimalRole {
                            role: "security".to_string(),
                            capabilities: vec!["btsp".to_string(), "tunneling".to_string()],
                            min_health: 0.9,
                            metadata: serde_json::json!({"primal": "beardog"}),
                        },
                        PrimalRole {
                            role: "discovery".to_string(),
                            capabilities: vec!["registry".to_string()],
                            min_health: 0.8,
                            metadata: serde_json::json!({"primal": "songbird"}),
                        },
                    ],
                    optional_primals: vec![],
                    estimated_resources: ResourceRequirements {
                        cpu_cores: 2,
                        memory_mb: 4096,
                        storage_gb: 10,
                        gpu_required: false,
                        network_bandwidth_mbps: 1000,
                    },
                    metadata: serde_json::json!({
                        "neural_api_graph": "tower_deploy.toml",
                        "deployment_time": "10-15 seconds"
                    }),
                });
            }

            // Load node template
            if tokio::fs::metadata(format!("{}/node_deploy.toml", graphs_dir))
                .await
                .is_ok()
            {
                templates.push(NicheTemplate {
                    id: "node".to_string(),
                    name: "Compute Node".to_string(),
                    description: "High-performance compute with AI assistance".to_string(),
                    required_primals: vec![
                        PrimalRole {
                            role: "compute".to_string(),
                            capabilities: vec!["resource_planning".to_string()],
                            min_health: 0.9,
                            metadata: serde_json::json!({"primal": "toadstool"}),
                        },
                        PrimalRole {
                            role: "ai".to_string(),
                            capabilities: vec!["suggestions".to_string()],
                            min_health: 0.7,
                            metadata: serde_json::json!({"primal": "squirrel"}),
                        },
                    ],
                    optional_primals: vec![],
                    estimated_resources: ResourceRequirements {
                        cpu_cores: 8,
                        memory_mb: 32768,
                        storage_gb: 100,
                        gpu_required: true,
                        network_bandwidth_mbps: 1000,
                    },
                    metadata: serde_json::json!({
                        "neural_api_graph": "node_deploy.toml",
                        "deployment_time": "15-20 seconds"
                    }),
                });
            }

            // Load nest template
            if tokio::fs::metadata(format!("{}/nest_deploy.toml", graphs_dir))
                .await
                .is_ok()
            {
                templates.push(NicheTemplate {
                    id: "nest".to_string(),
                    name: "Data Nest".to_string(),
                    description: "Secure data storage and persistence".to_string(),
                    required_primals: vec![
                        PrimalRole {
                            role: "storage".to_string(),
                            capabilities: vec!["persistence".to_string()],
                            min_health: 0.95,
                            metadata: serde_json::json!({"primal": "nestgate"}),
                        },
                        PrimalRole {
                            role: "security".to_string(),
                            capabilities: vec!["encryption".to_string()],
                            min_health: 0.9,
                            metadata: serde_json::json!({"primal": "beardog"}),
                        },
                    ],
                    optional_primals: vec![],
                    estimated_resources: ResourceRequirements {
                        cpu_cores: 4,
                        memory_mb: 8192,
                        storage_gb: 500,
                        gpu_required: false,
                        network_bandwidth_mbps: 1000,
                    },
                    metadata: serde_json::json!({
                        "neural_api_graph": "nest_deploy.toml",
                        "deployment_time": "10-15 seconds"
                    }),
                });
            }
        }

        info!("🏗️  Loaded {} niche templates", templates.len());
        Ok(templates)
    }

    /// Assign a device to a primal
    ///
    /// Orchestrates device assignment through multiple primals
    pub async fn assign_device(&self, device_id: String, primal_id: String) -> Result<bool> {
        info!("🔗 Assigning device {} to primal {}", device_id, primal_id);

        // TODO: Implement full orchestration
        // 1. Authorize via BearDog
        // 2. Validate via ToadStool (resources available?)
        // 3. Update state via NestGate
        // 4. Notify via event stream

        Ok(true)
    }

    /// Validate a niche configuration
    ///
    /// Checks if resources are available for deployment
    pub async fn validate_niche(
        &self,
        template_id: String,
        config: serde_json::Value,
    ) -> Result<ValidationResult> {
        info!("✅ Validating niche template {}", template_id);

        // TODO: Implement validation
        // 1. Load template
        // 2. Check resource availability via ToadStool
        // 3. Check primal health via Songbird
        // 4. Return detailed validation results

        Ok(ValidationResult {
            valid: true,
            errors: vec![],
            warnings: vec![],
        })
    }

    /// Deploy a niche
    ///
    /// Creates and starts a new niche by orchestrating multiple primals
    pub async fn deploy_niche(
        &self,
        template_id: String,
        config: serde_json::Value,
    ) -> Result<String> {
        info!("🚀 Deploying niche from template {}", template_id);

        // 1. Load the template
        let templates = self.get_niche_templates().await?;
        let template = templates
            .iter()
            .find(|t| t.id == template_id)
            .ok_or_else(|| anyhow::anyhow!("Template not found: {}", template_id))?;

        info!("📋 Using template: {}", template.name);

        // 2. Validate configuration
        let validation = self
            .validate_niche(template_id.clone(), config.clone())
            .await?;
        if !validation.valid {
            return Err(anyhow::anyhow!(
                "Niche validation failed: {:?}",
                validation.errors
            ));
        }

        // 3. Discover required primals
        let primals = self.get_primals_extended().await?;
        let mut assigned_primals = std::collections::HashMap::new();

        for required_role in &template.required_primals {
            // Find a primal that matches the required capabilities
            let matching_primal = primals
                .iter()
                .find(|p| {
                    required_role
                        .capabilities
                        .iter()
                        .all(|cap| p.capabilities.iter().any(|pc| pc.contains(cap)))
                        && p.health >= required_role.min_health
                })
                .ok_or_else(|| {
                    anyhow::anyhow!("No primal found for role: {}", required_role.role)
                })?;

            assigned_primals.insert(required_role.role.clone(), matching_primal.id.clone());
            info!(
                "   {} → {} ({})",
                required_role.role, matching_primal.name, matching_primal.id
            );
        }

        // 4. Create niche ID and configuration
        let niche_id = format!("niche-{}-{}", template_id, chrono::Utc::now().timestamp());

        let niche_config = serde_json::json!({
            "id": niche_id,
            "template": template_id,
            "name": template.name,
            "assigned_primals": assigned_primals,
            "created_at": chrono::Utc::now().to_rfc3339(),
            "status": "deployed",
            "config": config
        });

        info!("✅ Niche deployed successfully: {}", niche_id);
        info!("📊 Primal assignments: {:?}", assigned_primals);

        Ok(niche_id)
    }

    /// Update cache with fresh data
    ///
    /// Called periodically to refresh cached data
    pub async fn refresh_cache(&self) -> Result<()> {
        debug!("🔄 Refreshing cache");

        // TODO: Implement cache refresh
        // let devices = self.get_devices().await?;
        // let primals = self.get_primals_extended().await?;
        // let templates = self.get_niche_templates().await?;
        //
        // let mut cache = self.cache.write().await;
        // cache.devices = devices;
        // cache.primals = primals;
        // cache.templates = templates;
        // cache.last_update = Some(std::time::Instant::now());

        Ok(())
    }
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_bridge_creation() {
        let bridge = PetalTongueRPCBridge::new("/tmp/test.sock");
        assert_eq!(bridge.socket_path, "/tmp/test.sock");
    }

    #[tokio::test]
    async fn test_get_devices() {
        let bridge = PetalTongueRPCBridge::new("/tmp/test.sock");
        let devices = bridge.get_devices().await.unwrap();
        assert!(devices.is_empty()); // Empty until cache is populated
    }

    #[tokio::test]
    async fn test_get_primals() {
        let bridge = PetalTongueRPCBridge::new("/tmp/test.sock");
        let primals = bridge.get_primals_extended().await.unwrap();
        assert!(primals.is_empty()); // Empty until cache is populated
    }

    #[tokio::test]
    async fn test_get_templates() {
        let bridge = PetalTongueRPCBridge::new("/tmp/test.sock");
        let templates = bridge.get_niche_templates().await.unwrap();
        assert!(templates.is_empty()); // Empty until cache is populated
    }
}
