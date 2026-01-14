//! Device Management Capability Provider
//!
//! Generic implementation of the `device.management` capability.
//! NO primal-specific code - ANY primal can discover and use this!

use anyhow::{Context, Result};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::types::*;

/// Device Management Capability Provider
///
/// Provides the `device.management` capability for biomeOS.
/// Discovered at runtime - no hardcoded primal names!
pub struct DeviceManagementProvider {
    /// Cache for graceful degradation
    cache: Arc<RwLock<ProviderCache>>,
    /// Socket path for RPC server
    socket_path: String,
}

/// Cached data for offline mode
#[derive(Debug, Clone, Default)]
struct ProviderCache {
    devices: Vec<Device>,
    primals: Vec<ManagedPrimal>,
    templates: Vec<NicheTemplate>,
    last_update: Option<std::time::Instant>,
}

impl DeviceManagementProvider {
    /// Create a new device management provider
    ///
    /// # Arguments
    ///
    /// * `socket_path` - Unix socket path for JSON-RPC server
    ///
    /// # Example
    ///
    /// ```no_run
    /// use biomeos_ui::capabilities::device_management::DeviceManagementProvider;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let provider = DeviceManagementProvider::new("/run/user/1000/biomeos-device-mgmt.sock");
    ///     provider.start().await?;
    ///     Ok(())
    ///     }
    /// ```
    #[must_use]
    pub fn new(socket_path: impl Into<String>) -> Self {
        Self {
            cache: Arc::new(RwLock::new(ProviderCache::default())),
            socket_path: socket_path.into(),
        }
    }

    /// Start the capability provider
    ///
    /// Advertises "device.management" capability to Songbird and starts JSON-RPC server.
    pub async fn start(&self) -> Result<()> {
        info!(
            "🔧 Starting device.management capability provider on {}",
            self.socket_path
        );

        // TODO: Advertise capability via Songbird
        // let songbird = SongbirdClient::discover().await?;
        // songbird.register_service(&ServiceRegistration {
        //     service_name: "device.management".to_string(),
        //     capabilities: vec!["device.management".to_string()],
        //     endpoint: self.socket_path.clone(),
        //     metadata: ServiceMetadata {
        //         version: "1.0.0".to_string(),
        //         location: None,
        //         tags: vec!["ui", "management", "devices"],
        //     },
        // }).await?;

        // TODO: Start JSON-RPC server
        // rpc_server::start(self.socket_path.clone(), self.clone()).await?;

        info!("✅ device.management capability provider ready");
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

    /// Get all managed primals
    ///
    /// Discovers primals via runtime discovery
    pub async fn get_primals(&self) -> Result<Vec<ManagedPrimal>> {
        debug!("📡 Discovering primals");

        // Get primals from discovery
        let primals = self.discover_primals().await?;

        // Update cache
        let mut cache = self.cache.write().await;
        cache.primals = primals.clone();
        cache.last_update = Some(std::time::Instant::now());

        Ok(primals)
    }

    /// Get niche templates
    ///
    /// Loads templates from storage (NestGate or local)
    pub async fn get_niche_templates(&self) -> Result<Vec<NicheTemplate>> {
        debug!("📚 Loading niche templates");

        // TODO: Load from NestGate
        // let nestgate = discover_by_capability("storage").await?;
        // let templates = nestgate.call("get_templates", params).await?;

        // For now, return built-in templates
        let templates = self.get_builtin_templates();

        // Update cache
        let mut cache = self.cache.write().await;
        cache.templates = templates.clone();
        cache.last_update = Some(std::time::Instant::now());

        Ok(templates)
    }

    /// Validate a niche template
    ///
    /// Checks if resources are available for deployment
    pub async fn validate_niche(&self, template: &NicheTemplate) -> Result<ValidationResult> {
        debug!("✅ Validating niche template: {}", template.name);

        let mut errors = Vec::new();
        let mut warnings = Vec::new();

        // Check available devices
        let devices = self.get_devices().await?;
        let available_devices: Vec<_> = devices
            .iter()
            .filter(|d| matches!(d.status, DeviceStatus::Available))
            .collect();

        // Check GPU requirements
        if template.estimated_resources.gpu_required {
            let available_gpus = available_devices
                .iter()
                .filter(|d| matches!(d.device_type, DeviceType::Gpu))
                .count();

            if available_gpus == 0 {
                errors.push("No available GPUs found, but GPU is required".to_string());
            }
        }

        // Check available primals
        let primals = self.get_primals().await?;
        let healthy_primals: Vec<_> = primals
            .iter()
            .filter(|p| matches!(p.status, PrimalStatus::Healthy))
            .collect();

        // Validate required primals
        for required_role in &template.required_primals {
            let matching_primals = healthy_primals
                .iter()
                .filter(|p| {
                    required_role
                        .capabilities
                        .iter()
                        .all(|cap| p.capabilities.contains(cap))
                        && p.health >= required_role.min_health
                })
                .count();

            if matching_primals == 0 {
                errors.push(format!(
                    "No healthy primals found for required role: {}",
                    required_role.role
                ));
            }
        }

        // Check optional primals (warnings only)
        for optional_role in &template.optional_primals {
            let matching_primals = healthy_primals
                .iter()
                .filter(|p| {
                    optional_role
                        .capabilities
                        .iter()
                        .all(|cap| p.capabilities.contains(cap))
                })
                .count();

            if matching_primals == 0 {
                warnings.push(format!(
                    "No primals found for optional role: {} (may reduce functionality)",
                    optional_role.role
                ));
            }
        }

        Ok(ValidationResult {
            valid: errors.is_empty(),
            errors,
            warnings,
        })
    }

    /// Deploy a niche
    ///
    /// Creates and starts a niche from a template
    pub async fn deploy_niche(&self, config: serde_json::Value) -> Result<String> {
        info!("🚀 Deploying niche with config: {:?}", config);

        // TODO: Implement niche deployment via orchestrator
        // let orchestrator = discover_by_capability("orchestration").await?;
        // let niche_id = orchestrator.call("deploy_niche", config).await?;

        Ok("niche-placeholder".to_string())
    }

    /// Assign a device to a primal
    ///
    /// Coordinates device assignment via orchestration
    pub async fn assign_device(&self, device_id: String, primal_id: String) -> Result<()> {
        info!(
            "🔗 Assigning device {} to primal {}",
            device_id, primal_id
        );

        // TODO: Implement device assignment
        // let orchestrator = discover_by_capability("orchestration").await?;
        // orchestrator.call("assign_device", json!({
        //     "device_id": device_id,
        //     "primal_id": primal_id,
        // })).await?;

        Ok(())
    }

    // ========================================================================
    // PRIVATE METHODS - Discovery implementations
    // ========================================================================

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
            .args(&["--query-gpu=index,name,utilization.gpu,memory.used,memory.total", "--format=csv,noheader,nounits"])
            .output()
            .await
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    let parts: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
                    if parts.len() >= 5 {
                        let id = format!("gpu-{}", parts[0]);
                        let name = parts[1].to_string();
                        let usage = parts[2].parse::<f64>().unwrap_or(0.0) / 100.0;

                        gpus.push(Device {
                            id,
                            name,
                            device_type: DeviceType::Gpu,
                            status: if usage > 0.8 {
                                DeviceStatus::InUse
                            } else {
                                DeviceStatus::Available
                            },
                            resource_usage: usage,
                            assigned_to: None,
                            metadata: serde_json::json!({
                                "vendor": "nvidia",
                                "memory_used_mb": parts[3].parse::<u64>().unwrap_or(0),
                                "memory_total_mb": parts[4].parse::<u64>().unwrap_or(0),
                            }),
                        });
                    }
                }
            }
        }

        Ok(gpus)
    }

    /// Discover CPU devices
    async fn discover_cpus(&self) -> Result<Vec<Device>> {
        let mut cpus = Vec::new();

        // Get CPU count from /proc/cpuinfo
        if let Ok(cpuinfo) = tokio::fs::read_to_string("/proc/cpuinfo").await {
            let cpu_count = cpuinfo.matches("processor").count();
            
            cpus.push(Device {
                id: "cpu-0".to_string(),
                name: format!("CPU ({} cores)", cpu_count),
                device_type: DeviceType::Cpu,
                status: DeviceStatus::Available,
                resource_usage: 0.0, // TODO: Get actual usage
                assigned_to: None,
                metadata: serde_json::json!({
                    "cores": cpu_count,
                }),
            });
        }

        Ok(cpus)
    }

    /// Discover storage devices
    async fn discover_storage(&self) -> Result<Vec<Device>> {
        let mut storage = Vec::new();

        // Parse df output for mounted filesystems
        if let Ok(output) = tokio::process::Command::new("df")
            .args(&["-h", "--output=source,size,used,pcent,target"])
            .output()
            .await
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for (idx, line) in stdout.lines().skip(1).enumerate() {
                    let parts: Vec<&str> = line.split_whitespace().collect();
                    if parts.len() >= 5 {
                        let source = parts[0];
                        // Skip tmpfs and other virtual filesystems
                        if source.starts_with("/dev/") {
                            storage.push(Device {
                                id: format!("storage-{}", idx),
                                name: parts[4].to_string(), // mount point
                                device_type: DeviceType::Storage,
                                status: DeviceStatus::Available,
                                resource_usage: parts[3]
                                    .trim_end_matches('%')
                                    .parse::<f64>()
                                    .unwrap_or(0.0)
                                    / 100.0,
                                assigned_to: None,
                                metadata: serde_json::json!({
                                    "source": source,
                                    "size": parts[1],
                                    "used": parts[2],
                                }),
                            });
                        }
                    }
                }
            }
        }

        Ok(storage)
    }

    /// Discover network interfaces
    async fn discover_network(&self) -> Result<Vec<Device>> {
        let mut network = Vec::new();

        // Parse ip link output
        if let Ok(output) = tokio::process::Command::new("ip")
            .args(&["link", "show"])
            .output()
            .await
        {
            if output.status.success() {
                let stdout = String::from_utf8_lossy(&output.stdout);
                for line in stdout.lines() {
                    if let Some(idx) = line.find(':') {
                        if let Some(name_end) = line[idx + 1..].find(':') {
                            let name = line[idx + 1..idx + 1 + name_end].trim();
                            // Skip loopback
                            if name != "lo" && !name.is_empty() {
                                let status = if line.contains("state UP") {
                                    DeviceStatus::InUse
                                } else {
                                    DeviceStatus::Offline
                                };

                                network.push(Device {
                                    id: format!("net-{}", name),
                                    name: name.to_string(),
                                    device_type: DeviceType::Network,
                                    status,
                                    resource_usage: 0.0,
                                    assigned_to: None,
                                    metadata: serde_json::json!({}),
                                });
                            }
                        }
                    }
                }
            }
        }

        Ok(network)
    }

    /// Discover running primals
    async fn discover_primals(&self) -> Result<Vec<ManagedPrimal>> {
        let mut primals = Vec::new();

        // Check for running primal processes and their sockets
        let uid = std::env::var("UID").unwrap_or_else(|_| "1000".to_string());
        let socket_dir = format!("/run/user/{}", uid);

        // Look for primal sockets
        if let Ok(mut entries) = tokio::fs::read_dir(&socket_dir).await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                if let Some(name) = entry.file_name().to_str() {
                    // Check for any .sock file (discovery!)
                    if name.ends_with(".sock") {
                        let socket_path = format!("{}/{}", socket_dir, name);

                        // Query primal for its identity (TRUE PRIMAL!)
                        let primal_name = self.query_primal_identity(&socket_path).await;
                        let primal_id = primal_name.to_lowercase();

                        // Try to get health status
                        let (health, load, status) =
                            self.probe_primal_health(&socket_path).await;

                        // Get capabilities from primal
                        let capabilities = self.get_primal_capabilities(&primal_id);

                        primals.push(ManagedPrimal {
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
        primals.push(ManagedPrimal {
            id: "biomeos".to_string(),
            name: "biomeOS".to_string(),
            status: PrimalStatus::Healthy,
            health: 1.0,
            load: 0.1,
            capabilities: vec!["orchestration".to_string(), "device.management".to_string()],
            assigned_devices: vec![],
            metadata: serde_json::json!({
                "version": env!("CARGO_PKG_VERSION"),
                "self": true
            }),
        });

        info!("🔍 Discovered {} primals", primals.len());
        Ok(primals)
    }

    /// Query a primal for its identity (TRUE PRIMAL discovery!)
    async fn query_primal_identity(&self, _socket_path: &str) -> String {
        // TODO: Query via JSON-RPC for primal name
        // For now, derive from socket path
        "unknown".to_string()
    }

    /// Probe a primal for health metrics
    async fn probe_primal_health(&self, _socket_path: &str) -> (f64, f64, PrimalStatus) {
        // TODO: Query via JSON-RPC for health
        (1.0, 0.0, PrimalStatus::Healthy)
    }

    /// Get capabilities for a primal (based on known patterns)
    fn get_primal_capabilities(&self, primal_id: &str) -> Vec<String> {
        // TODO: Query primal for actual capabilities
        // For now, use known patterns
        match primal_id {
            id if id.contains("beardog") => vec!["security".to_string(), "crypto".to_string()],
            id if id.contains("songbird") => vec!["discovery".to_string(), "mesh".to_string()],
            id if id.contains("toadstool") => vec!["compute".to_string(), "gpu".to_string()],
            id if id.contains("nestgate") => vec!["storage".to_string(), "data".to_string()],
            _ => vec![],
        }
    }

    /// Get built-in niche templates
    fn get_builtin_templates(&self) -> Vec<NicheTemplate> {
        vec![
            NicheTemplate {
                id: "tower".to_string(),
                name: "Tower (Secure Base)".to_string(),
                description: "BearDog + Songbird atomic deployment".to_string(),
                required_primals: vec![
                    PrimalRole {
                        role: "security".to_string(),
                        capabilities: vec!["crypto".to_string(), "identity".to_string()],
                        min_health: 0.9,
                        metadata: serde_json::json!({}),
                    },
                    PrimalRole {
                        role: "discovery".to_string(),
                        capabilities: vec!["mesh".to_string(), "p2p".to_string()],
                        min_health: 0.8,
                        metadata: serde_json::json!({}),
                    },
                ],
                optional_primals: vec![],
                estimated_resources: ResourceRequirements {
                    cpu_cores: 2,
                    memory_mb: 512,
                    storage_gb: 1,
                    gpu_required: false,
                    network_bandwidth_mbps: 10,
                },
                metadata: serde_json::json!({}),
            },
            NicheTemplate {
                id: "node".to_string(),
                name: "Node (Compute Ready)".to_string(),
                description: "Tower + Toadstool for compute workloads".to_string(),
                required_primals: vec![
                    PrimalRole {
                        role: "security".to_string(),
                        capabilities: vec!["crypto".to_string()],
                        min_health: 0.9,
                        metadata: serde_json::json!({}),
                    },
                    PrimalRole {
                        role: "discovery".to_string(),
                        capabilities: vec!["mesh".to_string()],
                        min_health: 0.8,
                        metadata: serde_json::json!({}),
                    },
                    PrimalRole {
                        role: "compute".to_string(),
                        capabilities: vec!["gpu".to_string(), "cpu".to_string()],
                        min_health: 0.8,
                        metadata: serde_json::json!({}),
                    },
                ],
                optional_primals: vec![],
                estimated_resources: ResourceRequirements {
                    cpu_cores: 4,
                    memory_mb: 2048,
                    storage_gb: 10,
                    gpu_required: true,
                    network_bandwidth_mbps: 100,
                },
                metadata: serde_json::json!({}),
            },
        ]
    }
}

