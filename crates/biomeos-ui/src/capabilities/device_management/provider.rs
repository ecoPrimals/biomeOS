//! Device Management Capability Provider
//!
//! Generic implementation of the `device.management` capability.
//! NO primal-specific code - ANY primal can discover and use this!

use anyhow::Result;
use biomeos_core::atomic_client::AtomicClient;
use biomeos_types::CapabilityTaxonomy;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use super::types::*;

/// Resolve a capability to its runtime provider name via env var or taxonomy default.
fn resolve_provider(env_var: &str, capability: CapabilityTaxonomy) -> String {
    std::env::var(env_var)
        .unwrap_or_else(|_| capability.default_primal().unwrap_or("unknown").to_string())
}

/// Device Management Capability Provider
///
/// Provides the `device.management` capability for biomeOS.
/// Discovered at runtime - no hardcoded primal names!
pub struct DeviceManagementProvider {
    /// Cache for graceful degradation
    pub(crate) cache: Arc<RwLock<ProviderCache>>,
    /// Socket path for RPC server
    pub(crate) socket_path: String,
}

/// Cached data for offline mode
#[derive(Debug, Clone, Default)]
pub(crate) struct ProviderCache {
    pub(crate) devices: Vec<Device>,
    pub(crate) primals: Vec<ManagedPrimal>,
    pub(crate) templates: Vec<NicheTemplate>,
    pub(crate) last_update: Option<std::time::Instant>,
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

        // Advertise capability via registry provider (Songbird or env-configured)
        let registry_provider =
            resolve_provider("BIOMEOS_REGISTRY_PROVIDER", CapabilityTaxonomy::Discovery);
        if let Ok(registry) = AtomicClient::discover(&registry_provider).await {
            match registry
                .call(
                    "registry.register_service",
                    serde_json::json!({
                        "service_name": "device.management",
                        "capabilities": ["device.management"],
                        "endpoint": self.socket_path,
                        "metadata": {
                            "version": env!("CARGO_PKG_VERSION"),
                            "tags": ["ui", "management", "devices"]
                        }
                    }),
                )
                .await
            {
                Ok(_) => info!("✅ Registered device.management capability with registry"),
                Err(e) => warn!(
                    "⚠️ Failed to register with registry provider: {} (continuing anyway)",
                    e
                ),
            }
        } else {
            warn!("⚠️ Songbird not available, capability not advertised");
        }

        // Note: JSON-RPC server is started by the binary (device_management_server.rs)
        // This provider just handles the capability logic

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

        let mut templates = Vec::new();

        // Try to load from storage provider (NestGate or env-configured)
        let storage_provider =
            resolve_provider("BIOMEOS_STORAGE_PROVIDER", CapabilityTaxonomy::DataStorage);
        if let Ok(storage) = AtomicClient::discover(&storage_provider).await {
            match storage
                .call(
                    "storage.list",
                    serde_json::json!({ "key_prefix": "template:" }),
                )
                .await
            {
                Ok(result) => {
                    if let Some(items) = result.as_array() {
                        for item in items {
                            if let Ok(template) =
                                serde_json::from_value::<NicheTemplate>(item.clone())
                            {
                                templates.push(template);
                            }
                        }
                        info!(
                            "📚 Loaded {} templates from storage provider",
                            templates.len()
                        );
                    }
                }
                Err(e) => {
                    debug!(
                        "Storage provider template load failed: {} - using built-in",
                        e
                    );
                }
            }
        }

        // Fall back to built-in templates if none loaded
        if templates.is_empty() {
            templates = self.get_builtin_templates();
            debug!("📚 Using {} built-in templates", templates.len());
        }

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

        // Deploy via biomeOS orchestration capability
        if let Ok(biomeos) = AtomicClient::discover("biomeos").await {
            match biomeos
                .call("orchestration.deploy_niche", config.clone())
                .await
            {
                Ok(result) => {
                    let niche_id = result
                        .get("niche_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    info!("✅ Niche deployed: {}", niche_id);
                    return Ok(niche_id);
                }
                Err(e) => {
                    warn!("❌ Niche deployment failed: {}", e);
                    return Err(anyhow::anyhow!("Niche deployment failed: {}", e));
                }
            }
        }

        // Try orchestration provider as backup
        let orch_provider =
            resolve_provider("BIOMEOS_REGISTRY_PROVIDER", CapabilityTaxonomy::Discovery);
        if let Ok(orchestrator) = AtomicClient::discover(&orch_provider).await {
            match orchestrator
                .call("orchestration.deploy_niche", config)
                .await
            {
                Ok(result) => {
                    let niche_id = result
                        .get("niche_id")
                        .and_then(|v| v.as_str())
                        .unwrap_or("unknown")
                        .to_string();
                    info!("✅ Niche deployed via Songbird: {}", niche_id);
                    return Ok(niche_id);
                }
                Err(e) => {
                    warn!("❌ Songbird niche deployment failed: {}", e);
                }
            }
        }

        Err(anyhow::anyhow!("No orchestration capability available"))
    }

    /// Assign a device to a primal
    ///
    /// Coordinates device assignment via orchestration
    pub async fn assign_device(&self, device_id: String, primal_id: String) -> Result<()> {
        info!("🔗 Assigning device {} to primal {}", device_id, primal_id);

        // Coordinate via registry provider
        let registry_provider =
            resolve_provider("BIOMEOS_REGISTRY_PROVIDER", CapabilityTaxonomy::Discovery);
        if let Ok(registry) = AtomicClient::discover(&registry_provider).await {
            match registry
                .call(
                    "registry.assign_device",
                    serde_json::json!({
                        "device_id": device_id,
                        "primal_id": primal_id
                    }),
                )
                .await
            {
                Ok(_) => {
                    info!("✅ Device {} assigned to {}", device_id, primal_id);

                    // Persist assignment to storage provider
                    let storage_prov = resolve_provider(
                        "BIOMEOS_STORAGE_PROVIDER",
                        CapabilityTaxonomy::DataStorage,
                    );
                    if let Ok(storage) = AtomicClient::discover(&storage_prov).await {
                        let _ = storage
                            .call(
                                "storage.store",
                                serde_json::json!({
                                    "key": format!("assignment:{}:{}", device_id, primal_id),
                                    "value": {
                                        "device_id": device_id,
                                        "primal_id": primal_id,
                                        "assigned_at": chrono::Utc::now().to_rfc3339()
                                    }
                                }),
                            )
                            .await;
                    }

                    return Ok(());
                }
                Err(e) => {
                    warn!("❌ Device assignment failed: {}", e);
                    return Err(anyhow::anyhow!("Device assignment failed: {}", e));
                }
            }
        }

        Err(anyhow::anyhow!(
            "Songbird not available for device assignment"
        ))
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

    /// Discover GPU devices (pure Rust via /proc/driver/nvidia/)
    async fn discover_gpus(&self) -> Result<Vec<Device>> {
        let mut gpus = Vec::new();

        // Read NVIDIA GPU info from /proc/driver/nvidia/gpus/ (pure Rust, no nvidia-smi)
        if let Ok(mut entries) = tokio::fs::read_dir("/proc/driver/nvidia/gpus").await {
            let mut idx = 0;
            while let Ok(Some(entry)) = entries.next_entry().await {
                let gpu_dir = entry.path();
                let info_path = gpu_dir.join("information");

                if let Ok(info) = tokio::fs::read_to_string(&info_path).await {
                    let mut name = format!("NVIDIA GPU {}", idx);
                    let mut memory_total_mb: u64 = 0;

                    for line in info.lines() {
                        if let Some(val) = line.strip_prefix("Model:") {
                            name = val.trim().to_string();
                        }
                        // Memory info may not always be in /proc, but try
                    }

                    // Try to get memory from sysfs
                    let pci_id = entry.file_name().to_string_lossy().to_string();
                    let mem_path = format!("/sys/bus/pci/devices/{}/mem_info_vram_total", pci_id);
                    if let Ok(mem_str) = tokio::fs::read_to_string(&mem_path).await {
                        if let Ok(bytes) = mem_str.trim().parse::<u64>() {
                            memory_total_mb = bytes / (1024 * 1024);
                        }
                    }

                    gpus.push(Device {
                        id: format!("gpu-{}", idx),
                        name,
                        device_type: DeviceType::Gpu,
                        status: DeviceStatus::Available,
                        resource_usage: 0.0,
                        assigned_to: None,
                        metadata: serde_json::json!({
                            "vendor": "nvidia",
                            "pci_id": pci_id,
                            "memory_total_mb": memory_total_mb,
                        }),
                    });
                    idx += 1;
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

            // Get actual CPU usage from /proc/stat
            let cpu_usage = self.get_cpu_usage().await.unwrap_or(0.0);

            cpus.push(Device {
                id: "cpu-0".to_string(),
                name: format!("CPU ({} cores)", cpu_count),
                device_type: DeviceType::Cpu,
                status: if cpu_usage > 0.9 {
                    DeviceStatus::InUse
                } else {
                    DeviceStatus::Available
                },
                resource_usage: cpu_usage,
                assigned_to: None,
                metadata: serde_json::json!({
                    "cores": cpu_count,
                    "usage_percent": (cpu_usage * 100.0) as u32
                }),
            });
        }

        Ok(cpus)
    }

    /// Get current CPU usage from /proc/stat
    async fn get_cpu_usage(&self) -> Result<f64> {
        // Read /proc/stat twice with a small delay to calculate usage
        let stat1 = tokio::fs::read_to_string("/proc/stat").await?;
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        let stat2 = tokio::fs::read_to_string("/proc/stat").await?;

        // Parse first line (aggregate CPU stats)
        let parse_cpu_line = |line: &str| -> Option<(u64, u64)> {
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() < 5 || !parts[0].starts_with("cpu") {
                return None;
            }
            // user + nice + system + idle
            let user: u64 = parts[1].parse().ok()?;
            let nice: u64 = parts[2].parse().ok()?;
            let system: u64 = parts[3].parse().ok()?;
            let idle: u64 = parts[4].parse().ok()?;
            Some((user + nice + system, user + nice + system + idle))
        };

        let (active1, total1) = stat1
            .lines()
            .next()
            .and_then(parse_cpu_line)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse /proc/stat"))?;
        let (active2, total2) = stat2
            .lines()
            .next()
            .and_then(parse_cpu_line)
            .ok_or_else(|| anyhow::anyhow!("Failed to parse /proc/stat"))?;

        let active_diff = active2.saturating_sub(active1);
        let total_diff = total2.saturating_sub(total1);

        if total_diff == 0 {
            return Ok(0.0);
        }

        Ok(active_diff as f64 / total_diff as f64)
    }

    /// Discover storage devices (pure Rust via /proc/mounts + statvfs)
    async fn discover_storage(&self) -> Result<Vec<Device>> {
        let mut storage = Vec::new();

        // Read /proc/mounts for mounted filesystems (pure Rust, no `df` shell-out)
        if let Ok(mounts) = tokio::fs::read_to_string("/proc/mounts").await {
            for (idx, line) in mounts.lines().enumerate() {
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 2 {
                    let source = parts[0];
                    let mount_point = parts[1];

                    // Only show real block devices
                    if source.starts_with("/dev/") {
                        // Use statvfs for space info (pure Rust via std)
                        let (size_str, used_str, usage) = Self::statvfs_info(mount_point)
                            .unwrap_or(("unknown".to_string(), "unknown".to_string(), 0.0));

                        storage.push(Device {
                            id: format!("storage-{}", idx),
                            name: mount_point.to_string(),
                            device_type: DeviceType::Storage,
                            status: DeviceStatus::Available,
                            resource_usage: usage,
                            assigned_to: None,
                            metadata: serde_json::json!({
                                "source": source,
                                "size": size_str,
                                "used": used_str,
                            }),
                        });
                    }
                }
            }
        }

        Ok(storage)
    }

    /// Get filesystem stats via nix::sys::statvfs (pure Rust, no libc)
    fn statvfs_info(path: &str) -> Option<(String, String, f64)> {
        #[cfg(unix)]
        {
            use nix::sys::statvfs::statvfs;
            let stat = statvfs(path).ok()?;

            let block_size = stat.fragment_size() as u64;
            let total = stat.blocks() * block_size;
            let available = stat.blocks_available() * block_size;
            let used = total.saturating_sub(available);
            let usage = if total > 0 {
                used as f64 / total as f64
            } else {
                0.0
            };

            Some((Self::human_size(total), Self::human_size(used), usage))
        }
        #[cfg(not(unix))]
        None
    }

    /// Format bytes as human-readable size
    fn human_size(bytes: u64) -> String {
        const UNITS: &[&str] = &["B", "K", "M", "G", "T"];
        let mut size = bytes as f64;
        for unit in UNITS {
            if size < 1024.0 {
                return format!("{:.1}{}", size, unit);
            }
            size /= 1024.0;
        }
        format!("{:.1}P", size)
    }

    /// Discover network interfaces (pure Rust via /sys/class/net/)
    async fn discover_network(&self) -> Result<Vec<Device>> {
        let mut network = Vec::new();

        // Read /sys/class/net/ for network interfaces (pure Rust, no `ip` shell-out)
        if let Ok(mut entries) = tokio::fs::read_dir("/sys/class/net").await {
            while let Ok(Some(entry)) = entries.next_entry().await {
                let name = entry.file_name().to_string_lossy().to_string();

                // Skip loopback
                if name == "lo" {
                    continue;
                }

                // Read operstate to determine if interface is up
                let operstate_path = format!("/sys/class/net/{}/operstate", name);
                let status = match tokio::fs::read_to_string(&operstate_path).await {
                    Ok(state) if state.trim() == "up" => DeviceStatus::InUse,
                    _ => DeviceStatus::Offline,
                };

                network.push(Device {
                    id: format!("net-{}", name),
                    name,
                    device_type: DeviceType::Network,
                    status,
                    resource_usage: 0.0,
                    assigned_to: None,
                    metadata: serde_json::json!({}),
                });
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
                        let (health, load, status) = self.probe_primal_health(&socket_path).await;

                        // Get capabilities from primal (TRUE PRIMAL: query at runtime!)
                        let capabilities = self.get_primal_capabilities(&socket_path).await;

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
    ///
    /// **TRUE PRIMAL Principle**: Primal code only has self-knowledge.
    /// We query the primal for its identity rather than hardcoding assumptions.
    async fn query_primal_identity(&self, socket_path: &str) -> String {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;

        // Try to connect to the Unix socket
        let stream = match UnixStream::connect(socket_path).await {
            Ok(s) => s,
            Err(e) => {
                warn!("Failed to connect to {}: {}", socket_path, e);
                return "unknown".to_string();
            }
        };

        // Send JSON-RPC request for identity
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "identity.get",
            "params": {},
            "id": 1
        });

        let request_str = match serde_json::to_string(&request) {
            Ok(s) => s + "\n",
            Err(e) => {
                warn!("Failed to serialize identity request: {}", e);
                return "unknown".to_string();
            }
        };
        let (read, mut write) = stream.into_split();

        if let Err(e) = write.write_all(request_str.as_bytes()).await {
            warn!("Failed to send identity query: {}", e);
            return "unknown".to_string();
        }

        // Read response
        let mut reader = BufReader::new(read);
        let mut response_line = String::new();

        match tokio::time::timeout(
            std::time::Duration::from_secs(2),
            reader.read_line(&mut response_line),
        )
        .await
        {
            Ok(Ok(_)) => {
                if let Ok(response) = serde_json::from_str::<serde_json::Value>(&response_line) {
                    if let Some(name) = response["result"]["name"].as_str() {
                        return name.to_string();
                    }
                }
                warn!("Invalid identity response from {}", socket_path);
                "unknown".to_string()
            }
            Ok(Err(e)) => {
                warn!("Failed to read identity response: {}", e);
                "unknown".to_string()
            }
            Err(_) => {
                warn!("Identity query timeout for {}", socket_path);
                "unknown".to_string()
            }
        }
    }

    /// Probe a primal for health metrics
    ///
    /// **TRUE PRIMAL Principle**: Query primal for its own health status.
    /// Each primal knows its own state, we don't assume.
    async fn probe_primal_health(&self, socket_path: &str) -> (f64, f64, PrimalStatus) {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;

        // Try to connect
        let stream = match UnixStream::connect(socket_path).await {
            Ok(s) => s,
            Err(_) => return (0.0, 1.0, PrimalStatus::Offline),
        };

        // Send JSON-RPC health check request
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "health.check",
            "params": {},
            "id": 1
        });

        let request_str = match serde_json::to_string(&request) {
            Ok(s) => s + "\n",
            Err(e) => {
                warn!("Failed to serialize health request: {}", e);
                return (0.0, 1.0, PrimalStatus::Degraded);
            }
        };
        let (read, mut write) = stream.into_split();

        if write.write_all(request_str.as_bytes()).await.is_err() {
            return (0.0, 1.0, PrimalStatus::Degraded);
        }

        // Read response
        let mut reader = BufReader::new(read);
        let mut response_line = String::new();

        match tokio::time::timeout(
            std::time::Duration::from_secs(2),
            reader.read_line(&mut response_line),
        )
        .await
        {
            Ok(Ok(_)) => {
                if let Ok(response) = serde_json::from_str::<serde_json::Value>(&response_line) {
                    // Parse health metrics from response
                    let health = response["result"]["health"].as_f64().unwrap_or(1.0);
                    let load = response["result"]["load"].as_f64().unwrap_or(0.0);
                    let status_str = response["result"]["status"].as_str().unwrap_or("healthy");

                    let status = match status_str {
                        "healthy" | "ok" => PrimalStatus::Healthy,
                        "degraded" | "unhealthy" => PrimalStatus::Degraded,
                        "offline" => PrimalStatus::Offline,
                        _ => PrimalStatus::Healthy,
                    };

                    return (health, load, status);
                }
                // If we got a response but couldn't parse it, assume degraded
                (0.8, 0.2, PrimalStatus::Degraded)
            }
            _ => {
                // Timeout or error means degraded
                (0.0, 1.0, PrimalStatus::Degraded)
            }
        }
    }

    /// Get capabilities for a primal via capability-based discovery
    ///
    /// **EVOLUTION FROM HARDCODING**: This method now queries the primal
    /// for its actual capabilities rather than assuming based on name patterns.
    ///
    /// **TRUE PRIMAL Principle**: Each primal advertises its own capabilities.
    /// We discover them at runtime, not at compile time.
    async fn get_primal_capabilities(&self, socket_path: &str) -> Vec<String> {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;

        // Try to connect to the primal
        let stream = match UnixStream::connect(socket_path).await {
            Ok(s) => s,
            Err(e) => {
                warn!(
                    "Failed to connect to {} for capabilities: {}",
                    socket_path, e
                );
                return vec![];
            }
        };

        // Send JSON-RPC request for capabilities
        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": "capabilities.list",
            "params": {},
            "id": 1
        });

        let request_str = match serde_json::to_string(&request) {
            Ok(s) => s + "\n",
            Err(e) => {
                warn!("Failed to serialize capabilities request: {}", e);
                return vec![];
            }
        };
        let (read, mut write) = stream.into_split();

        if let Err(e) = write.write_all(request_str.as_bytes()).await {
            warn!("Failed to send capabilities query: {}", e);
            return vec![];
        }

        // Read response
        let mut reader = BufReader::new(read);
        let mut response_line = String::new();

        match tokio::time::timeout(
            std::time::Duration::from_secs(2),
            reader.read_line(&mut response_line),
        )
        .await
        {
            Ok(Ok(_)) => {
                if let Ok(response) = serde_json::from_str::<serde_json::Value>(&response_line) {
                    if let Some(caps) = response["result"]["capabilities"].as_array() {
                        return caps
                            .iter()
                            .filter_map(|v| v.as_str().map(String::from))
                            .collect();
                    }
                }
                warn!("Invalid capabilities response from {}", socket_path);
                vec![]
            }
            Ok(Err(e)) => {
                warn!("Failed to read capabilities response: {}", e);
                vec![]
            }
            Err(_) => {
                warn!("Capabilities query timeout for {}", socket_path);
                vec![]
            }
        }
    }

    /// Get built-in niche templates
    ///
    /// Delegates to the templates module for standard templates.
    /// This keeps template definitions separate from provider logic.
    pub(crate) fn get_builtin_templates(&self) -> Vec<NicheTemplate> {
        super::templates::builtin_templates()
    }
}
// Tests are in provider_tests.rs to keep this file under 1000 lines
