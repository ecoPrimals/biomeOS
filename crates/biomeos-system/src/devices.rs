//! # biomeOS Device Manager
//!
//! Manages hardware devices and device drivers for biomeOS.
//! Works with Toadstool to provide device abstraction for Primals.

use biomeos_core::BiomeResult;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tokio::sync::RwLock;

/// Device manager for biomeOS
pub struct DeviceManager {
    /// Configuration
    pub config: DeviceConfig,
    /// Detected devices
    pub devices: RwLock<HashMap<String, Device>>,
    /// Device drivers
    pub drivers: RwLock<HashMap<String, DeviceDriver>>,
}

/// Device configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceConfig {
    /// Device detection enabled
    pub enable_detection: bool,
    /// Device auto-configuration
    pub auto_configure: bool,
    /// Device driver directory
    pub driver_dir: PathBuf,
    /// Device configuration directory
    pub config_dir: PathBuf,
    /// Device blacklist
    pub blacklist: Vec<String>,
    /// Device whitelist (empty = all allowed)
    pub whitelist: Vec<String>,
}

/// Hardware device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    /// Device ID
    pub id: String,
    /// Device name
    pub name: String,
    /// Device type
    pub device_type: DeviceType,
    /// Device vendor
    pub vendor: Option<String>,
    /// Device model
    pub model: Option<String>,
    /// Device serial number
    pub serial: Option<String>,
    /// Device version/revision
    pub version: Option<String>,
    /// Device driver
    pub driver: Option<String>,
    /// Device status
    pub status: DeviceStatus,
    /// Device capabilities
    pub capabilities: Vec<DeviceCapability>,
    /// Device properties
    pub properties: HashMap<String, String>,
    /// Device path in filesystem
    pub path: Option<PathBuf>,
}

/// Device type
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceType {
    /// CPU
    Cpu,
    /// Memory
    Memory,
    /// Storage device
    Storage,
    /// Network interface
    Network,
    /// GPU
    Gpu,
    /// Audio device
    Audio,
    /// Input device
    Input,
    /// Display device
    Display,
    /// USB device
    Usb,
    /// PCI device
    Pci,
    /// Bluetooth device
    Bluetooth,
    /// Sensor
    Sensor,
    /// Camera
    Camera,
    /// Printer
    Printer,
    /// Unknown device
    Unknown,
}

/// Device status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceStatus {
    /// Device state
    pub state: DeviceState,
    /// Device health
    pub health: DeviceHealth,
    /// Device temperature (if available)
    pub temperature: Option<f64>,
    /// Device power state
    pub power_state: PowerState,
    /// Device usage statistics
    pub usage: DeviceUsage,
    /// Last error
    pub last_error: Option<String>,
}

/// Device state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceState {
    /// Device is available
    Available,
    /// Device is in use
    InUse,
    /// Device is disabled
    Disabled,
    /// Device has failed
    Failed,
    /// Device is not present
    NotPresent,
    /// Device state is unknown
    Unknown,
}

/// Device health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceHealth {
    /// Device is healthy
    Healthy,
    /// Device has warnings
    Warning,
    /// Device is unhealthy
    Unhealthy,
    /// Device health is unknown
    Unknown,
}

/// Power state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PowerState {
    /// Device is on
    On,
    /// Device is off
    Off,
    /// Device is in sleep mode
    Sleep,
    /// Device is in hibernate mode
    Hibernate,
    /// Power state is unknown
    Unknown,
}

/// Device usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceUsage {
    /// Usage percentage
    pub usage_percent: f64,
    /// Total bytes processed
    pub bytes_processed: u64,
    /// Operations per second
    pub operations_per_second: f64,
    /// Error count
    pub error_count: u64,
    /// Uptime in seconds
    pub uptime_seconds: u64,
}

/// Device capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceCapability {
    /// Read capability
    Read,
    /// Write capability
    Write,
    /// Execute capability
    Execute,
    /// Network capability
    Network,
    /// Graphics capability
    Graphics,
    /// Audio capability
    Audio,
    /// Video capability
    Video,
    /// Compute capability
    Compute,
    /// Storage capability
    Storage,
    /// Custom capability
    Custom(String),
}

/// Device driver
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceDriver {
    /// Driver name
    pub name: String,
    /// Driver version
    pub version: String,
    /// Driver description
    pub description: String,
    /// Driver path
    pub path: PathBuf,
    /// Supported devices
    pub supported_devices: Vec<String>,
    /// Driver status
    pub status: DriverStatus,
    /// Driver configuration
    pub config: HashMap<String, String>,
}

/// Driver status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DriverStatus {
    /// Driver is loaded
    Loaded,
    /// Driver is not loaded
    NotLoaded,
    /// Driver failed to load
    Failed,
    /// Driver is loading
    Loading,
    /// Driver is unloading
    Unloading,
}

impl DeviceManager {
    /// Create new device manager
    pub fn new(config: DeviceConfig) -> Self {
        Self {
            config,
            devices: RwLock::new(HashMap::new()),
            drivers: RwLock::new(HashMap::new()),
        }
    }

    /// Initialize device manager
    pub async fn initialize(&self) -> BiomeResult<()> {
        tracing::info!("Initializing device manager");

        // Create directories
        if let Err(e) = tokio::fs::create_dir_all(&self.config.driver_dir).await {
            tracing::warn!("Failed to create driver directory: {}", e);
        }
        if let Err(e) = tokio::fs::create_dir_all(&self.config.config_dir).await {
            tracing::warn!("Failed to create config directory: {}", e);
        }

        // Load drivers
        self.load_drivers().await?;

        // Detect devices if enabled
        if self.config.enable_detection {
            self.detect_devices().await?;
        }

        tracing::info!("Device manager initialized");
        Ok(())
    }

    /// Start device manager
    pub async fn start(&self) -> BiomeResult<()> {
        tracing::info!("Starting device manager");

        // Configure detected devices
        if self.config.auto_configure {
            self.configure_devices().await?;
        }

        tracing::info!("Device manager started");
        Ok(())
    }

    /// Load device drivers
    async fn load_drivers(&self) -> BiomeResult<()> {
        tracing::info!("Loading device drivers");

        // TODO: Implement driver loading
        // For now, add some default drivers
        let mut drivers = self.drivers.write().await;

        drivers.insert(
            "generic_storage".to_string(),
            DeviceDriver {
                name: "generic_storage".to_string(),
                version: "1.0.0".to_string(),
                description: "Generic storage device driver".to_string(),
                path: PathBuf::from("/lib/drivers/generic_storage.so"),
                supported_devices: vec!["storage".to_string()],
                status: DriverStatus::Loaded,
                config: HashMap::new(),
            },
        );

        drivers.insert(
            "generic_network".to_string(),
            DeviceDriver {
                name: "generic_network".to_string(),
                version: "1.0.0".to_string(),
                description: "Generic network device driver".to_string(),
                path: PathBuf::from("/lib/drivers/generic_network.so"),
                supported_devices: vec!["network".to_string()],
                status: DriverStatus::Loaded,
                config: HashMap::new(),
            },
        );

        tracing::info!("Device drivers loaded");
        Ok(())
    }

    /// Detect hardware devices
    async fn detect_devices(&self) -> BiomeResult<()> {
        tracing::info!("Detecting hardware devices");

        let mut devices = self.devices.write().await;

        // CPU detection
        let cpu_info = self.detect_cpu().await?;
        devices.insert("cpu0".to_string(), cpu_info);

        // Memory detection
        let memory_info = self.detect_memory().await?;
        devices.insert("memory0".to_string(), memory_info);

        // Storage detection
        let storage_devices = self.detect_storage().await?;
        for (id, device) in storage_devices {
            devices.insert(id, device);
        }

        // Network detection
        let network_devices = self.detect_network().await?;
        for (id, device) in network_devices {
            devices.insert(id, device);
        }

        tracing::info!(
            "Hardware detection complete, found {} devices",
            devices.len()
        );
        Ok(())
    }

    /// Detect CPU
    async fn detect_cpu(&self) -> BiomeResult<Device> {
        // Read CPU information from /proc/cpuinfo
        let mut vendor = "Unknown".to_string();
        let mut model = "Unknown CPU".to_string();
        let mut cores = 1u32;
        let mut properties = HashMap::new();

        if let Ok(cpuinfo) = tokio::fs::read_to_string("/proc/cpuinfo").await {
            for line in cpuinfo.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    let key = key.trim();
                    let value = value.trim();
                    
                    match key {
                        "vendor_id" => vendor = value.to_string(),
                        "model name" => model = value.to_string(),
                        "cpu cores" => {
                            if let Ok(c) = value.parse::<u32>() {
                                cores = c;
                            }
                        }
                        "cpu MHz" => {
                            properties.insert("frequency_mhz".to_string(), value.to_string());
                        }
                        "cache size" => {
                            properties.insert("cache_size".to_string(), value.to_string());
                        }
                        "flags" => {
                            properties.insert("features".to_string(), value.to_string());
                        }
                        _ => {}
                    }
                }
            }
        }

        properties.insert("cores".to_string(), cores.to_string());

        // Check CPU temperature if available
        let mut temperature = None;
        if let Ok(temp_str) = tokio::fs::read_to_string("/sys/class/thermal/thermal_zone0/temp").await {
            if let Ok(temp_millicelsius) = temp_str.trim().parse::<u64>() {
                temperature = Some(temp_millicelsius as f64 / 1000.0);
            }
        }

        Ok(Device {
            id: "cpu0".to_string(),
            name: "CPU".to_string(),
            device_type: DeviceType::Cpu,
            vendor: Some(vendor),
            model: Some(model),
            serial: None,
            version: None,
            driver: None,
            status: DeviceStatus {
                state: DeviceState::Available,
                health: DeviceHealth::Healthy,
                temperature,
                power_state: PowerState::On,
                usage: DeviceUsage {
                    usage_percent: 0.0,
                    bytes_processed: 0,
                    operations_per_second: 0.0,
                    error_count: 0,
                    uptime_seconds: 0,
                },
                last_error: None,
            },
            capabilities: vec![DeviceCapability::Compute],
            properties,
            path: Some(PathBuf::from("/proc/cpuinfo")),
        })
    }

    /// Detect memory
    async fn detect_memory(&self) -> BiomeResult<Device> {
        let mut total_memory = 0u64;
        let mut available_memory = 0u64;
        let mut properties = HashMap::new();

        // Read memory information from /proc/meminfo
        if let Ok(meminfo) = tokio::fs::read_to_string("/proc/meminfo").await {
            for line in meminfo.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    let key = key.trim();
                    let value = value.trim();
                    
                    // Parse memory sizes (usually in kB)
                    if let Some(size_str) = value.split_whitespace().next() {
                        if let Ok(size_kb) = size_str.parse::<u64>() {
                            let size_bytes = size_kb * 1024;
                            
                            match key {
                                "MemTotal" => {
                                    total_memory = size_bytes;
                                    properties.insert("total_bytes".to_string(), size_bytes.to_string());
                                }
                                "MemAvailable" => {
                                    available_memory = size_bytes;
                                    properties.insert("available_bytes".to_string(), size_bytes.to_string());
                                }
                                "SwapTotal" => {
                                    properties.insert("swap_total_bytes".to_string(), size_bytes.to_string());
                                }
                                "SwapFree" => {
                                    properties.insert("swap_free_bytes".to_string(), size_bytes.to_string());
                                }
                                _ => {}
                            }
                        }
                    }
                }
            }
        }

        // Calculate memory usage percentage
        let usage_percent = if total_memory > 0 {
            ((total_memory - available_memory) as f64 / total_memory as f64) * 100.0
        } else {
            0.0
        };

        let model = if total_memory > 0 {
            format!("System RAM ({} MB)", total_memory / 1024 / 1024)
        } else {
            "System RAM".to_string()
        };

        Ok(Device {
            id: "memory0".to_string(),
            name: "System Memory".to_string(),
            device_type: DeviceType::Memory,
            vendor: Some("System".to_string()),
            model: Some(model),
            serial: None,
            version: None,
            driver: None,
            status: DeviceStatus {
                state: DeviceState::Available,
                health: if usage_percent > 90.0 {
                    DeviceHealth::Warning
                } else {
                    DeviceHealth::Healthy
                },
                temperature: None,
                power_state: PowerState::On,
                usage: DeviceUsage {
                    usage_percent,
                    bytes_processed: 0,
                    operations_per_second: 0.0,
                    error_count: 0,
                    uptime_seconds: 0,
                },
                last_error: None,
            },
            capabilities: vec![DeviceCapability::Read, DeviceCapability::Write],
            properties,
            path: Some(PathBuf::from("/proc/meminfo")),
        })
    }

    /// Detect storage devices
    async fn detect_storage(&self) -> BiomeResult<HashMap<String, Device>> {
        let mut devices = HashMap::new();

        // Read from /proc/partitions to get block devices
        if let Ok(partitions) = tokio::fs::read_to_string("/proc/partitions").await {
            let mut device_id = 0;
            
            for line in partitions.lines().skip(2) { // Skip header lines
                let parts: Vec<&str> = line.split_whitespace().collect();
                if parts.len() >= 4 {
                    let _major = parts[0];
                    let _minor = parts[1];
                    let blocks = parts[2];
                    let name = parts[3];
                    
                    // Skip partition entries (look for base devices)
                    if name.chars().last().map_or(false, |c| c.is_ascii_digit()) {
                        continue;
                    }
                    
                    // Parse size in blocks (512 bytes each)
                    let size_bytes = blocks.parse::<u64>().unwrap_or(0) * 512;
                    
                    let mut properties = HashMap::new();
                    properties.insert("size_bytes".to_string(), size_bytes.to_string());
                    properties.insert("size_mb".to_string(), (size_bytes / 1024 / 1024).to_string());
                    
                    // Try to read additional info from /sys/block/
                    let sys_path = format!("/sys/block/{}", name);
                    if let Ok(model) = tokio::fs::read_to_string(format!("{}/device/model", sys_path)).await {
                        properties.insert("model".to_string(), model.trim().to_string());
                    }
                    if let Ok(vendor) = tokio::fs::read_to_string(format!("{}/device/vendor", sys_path)).await {
                        properties.insert("vendor".to_string(), vendor.trim().to_string());
                    }
                    if let Ok(serial) = tokio::fs::read_to_string(format!("{}/device/serial", sys_path)).await {
                        properties.insert("serial".to_string(), serial.trim().to_string());
                    }
                    
                    // Determine device type
                    let device_type = if name.starts_with("nvme") {
                        DeviceType::Storage
                    } else if name.starts_with("sd") {
                        DeviceType::Storage
                    } else if name.starts_with("hd") {
                        DeviceType::Storage
                    } else if name.starts_with("sr") {
                        DeviceType::Storage // CD/DVD
                    } else {
                        DeviceType::Storage
                    };
                    
                    let device = Device {
                        id: format!("storage{}", device_id),
                        name: format!("Storage Device {}", name),
                        device_type,
                        vendor: properties.get("vendor").cloned(),
                        model: properties.get("model").cloned(),
                        serial: properties.get("serial").cloned(),
                        version: None,
                        driver: Some("generic_storage".to_string()),
                        status: DeviceStatus {
                            state: DeviceState::Available,
                            health: DeviceHealth::Healthy,
                            temperature: None,
                            power_state: PowerState::On,
                            usage: DeviceUsage {
                                usage_percent: 0.0,
                                bytes_processed: 0,
                                operations_per_second: 0.0,
                                error_count: 0,
                                uptime_seconds: 0,
                            },
                            last_error: None,
                        },
                        capabilities: vec![
                            DeviceCapability::Read,
                            DeviceCapability::Write,
                            DeviceCapability::Storage,
                        ],
                        properties,
                        path: Some(PathBuf::from(format!("/dev/{}", name))),
                    };
                    
                    devices.insert(format!("storage{}", device_id), device);
                    device_id += 1;
                }
            }
        }

        // If no devices found, add a generic one
        if devices.is_empty() {
            devices.insert(
                "storage0".to_string(),
                Device {
                    id: "storage0".to_string(),
                    name: "System Storage".to_string(),
                    device_type: DeviceType::Storage,
                    vendor: Some("Generic".to_string()),
                    model: Some("Generic Storage".to_string()),
                    serial: None,
                    version: None,
                    driver: Some("generic_storage".to_string()),
                    status: DeviceStatus {
                        state: DeviceState::Available,
                        health: DeviceHealth::Healthy,
                        temperature: None,
                        power_state: PowerState::On,
                        usage: DeviceUsage {
                            usage_percent: 0.0,
                            bytes_processed: 0,
                            operations_per_second: 0.0,
                            error_count: 0,
                            uptime_seconds: 0,
                        },
                        last_error: None,
                    },
                    capabilities: vec![
                        DeviceCapability::Read,
                        DeviceCapability::Write,
                        DeviceCapability::Storage,
                    ],
                    properties: HashMap::new(),
                    path: Some(PathBuf::from("/dev/sda")),
                },
            );
        }

        Ok(devices)
    }

    /// Detect network devices
    async fn detect_network(&self) -> BiomeResult<HashMap<String, Device>> {
        let mut devices = HashMap::new();

        // Read network interfaces from /proc/net/dev
        if let Ok(net_dev) = tokio::fs::read_to_string("/proc/net/dev").await {
            let mut device_id = 0;
            
            for line in net_dev.lines().skip(2) { // Skip header lines
                if let Some((iface_name, stats)) = line.split_once(':') {
                    let iface_name = iface_name.trim();
                    
                    // Skip loopback interface
                    if iface_name == "lo" {
                        continue;
                    }
                    
                    let stats: Vec<&str> = stats.split_whitespace().collect();
                    let mut properties = HashMap::new();
                    
                    if stats.len() >= 16 {
                        // RX stats
                        properties.insert("rx_bytes".to_string(), stats[0].to_string());
                        properties.insert("rx_packets".to_string(), stats[1].to_string());
                        properties.insert("rx_errors".to_string(), stats[2].to_string());
                        properties.insert("rx_dropped".to_string(), stats[3].to_string());
                        
                        // TX stats
                        properties.insert("tx_bytes".to_string(), stats[8].to_string());
                        properties.insert("tx_packets".to_string(), stats[9].to_string());
                        properties.insert("tx_errors".to_string(), stats[10].to_string());
                        properties.insert("tx_dropped".to_string(), stats[11].to_string());
                    }
                    
                    // Try to read additional info from /sys/class/net/
                    let sys_path = format!("/sys/class/net/{}", iface_name);
                    
                    // Check if interface is up
                    let mut state = DeviceState::Available;
                    if let Ok(operstate) = tokio::fs::read_to_string(format!("{}/operstate", sys_path)).await {
                        match operstate.trim() {
                            "up" => state = DeviceState::Available,
                            "down" => state = DeviceState::Disabled,
                            _ => state = DeviceState::Unknown,
                        }
                    }
                    
                    // Get MAC address
                    if let Ok(address) = tokio::fs::read_to_string(format!("{}/address", sys_path)).await {
                        properties.insert("mac_address".to_string(), address.trim().to_string());
                    }
                    
                    // Get MTU
                    if let Ok(mtu) = tokio::fs::read_to_string(format!("{}/mtu", sys_path)).await {
                        properties.insert("mtu".to_string(), mtu.trim().to_string());
                    }
                    
                    // Get speed if available
                    if let Ok(speed) = tokio::fs::read_to_string(format!("{}/speed", sys_path)).await {
                        properties.insert("speed_mbps".to_string(), speed.trim().to_string());
                    }
                    
                    // Determine device type
                    let device_type = if iface_name.starts_with("eth") || iface_name.starts_with("en") {
                        DeviceType::Network
                    } else if iface_name.starts_with("wlan") || iface_name.starts_with("wl") {
                        DeviceType::Network
                    } else {
                        DeviceType::Network
                    };
                    
                    let device = Device {
                        id: format!("network{}", device_id),
                        name: format!("Network Interface {}", iface_name),
                        device_type,
                        vendor: Some("Generic".to_string()),
                        model: Some("Network Interface".to_string()),
                        serial: None,
                        version: None,
                        driver: Some("generic_network".to_string()),
                        status: DeviceStatus {
                            state,
                            health: DeviceHealth::Healthy,
                            temperature: None,
                            power_state: PowerState::On,
                            usage: DeviceUsage {
                                usage_percent: 0.0,
                                bytes_processed: stats.get(0).and_then(|s| s.parse().ok()).unwrap_or(0),
                                operations_per_second: 0.0,
                                error_count: stats.get(2).and_then(|s| s.parse().ok()).unwrap_or(0),
                                uptime_seconds: 0,
                            },
                            last_error: None,
                        },
                        capabilities: vec![DeviceCapability::Network],
                        properties,
                        path: Some(PathBuf::from(format!("/sys/class/net/{}", iface_name))),
                    };
                    
                    devices.insert(format!("network{}", device_id), device);
                    device_id += 1;
                }
            }
        }

        // If no devices found, add a generic one
        if devices.is_empty() {
            devices.insert(
                "network0".to_string(),
                Device {
                    id: "network0".to_string(),
                    name: "Network Interface".to_string(),
                    device_type: DeviceType::Network,
                    vendor: Some("Generic".to_string()),
                    model: Some("Generic Network".to_string()),
                    serial: None,
                    version: None,
                    driver: Some("generic_network".to_string()),
                    status: DeviceStatus {
                        state: DeviceState::Available,
                        health: DeviceHealth::Healthy,
                        temperature: None,
                        power_state: PowerState::On,
                        usage: DeviceUsage {
                            usage_percent: 0.0,
                            bytes_processed: 0,
                            operations_per_second: 0.0,
                            error_count: 0,
                            uptime_seconds: 0,
                        },
                        last_error: None,
                    },
                    capabilities: vec![DeviceCapability::Network],
                    properties: HashMap::new(),
                    path: Some(PathBuf::from("/dev/eth0")),
                },
            );
        }

        Ok(devices)
    }

    /// Configure devices
    async fn configure_devices(&self) -> BiomeResult<()> {
        tracing::info!("Configuring devices");

        let device_ids: Vec<String> = {
            let devices = self.devices.read().await;
            devices.keys().cloned().collect()
        };

        for device_id in device_ids {
            if let Err(e) = self.configure_device(&device_id).await {
                tracing::warn!("Failed to configure device {}: {}", device_id, e);
            }
        }

        tracing::info!("Device configuration complete");
        Ok(())
    }

    /// Configure a specific device
    async fn configure_device(&self, device_id: &str) -> BiomeResult<()> {
        let device = {
            let devices = self.devices.read().await;
            devices.get(device_id).cloned()
        };

        if let Some(mut device) = device {
            tracing::debug!("Configuring device: {}", device_id);

            // Apply device-specific configuration
            match device.device_type {
                DeviceType::Storage => {
                    // Configure storage device
                    self.configure_storage_device(&mut device).await?;
                }
                DeviceType::Network => {
                    // Configure network device
                    self.configure_network_device(&mut device).await?;
                }
                DeviceType::Cpu => {
                    // Configure CPU device
                    self.configure_cpu_device(&mut device).await?;
                }
                DeviceType::Memory => {
                    // Configure memory device
                    self.configure_memory_device(&mut device).await?;
                }
                DeviceType::Gpu => {
                    // Configure GPU device
                    self.configure_gpu_device(&mut device).await?;
                }
                _ => {
                    // Generic device configuration
                    self.configure_generic_device(&mut device).await?;
                }
            }

            // Update device in the collection
            let mut devices = self.devices.write().await;
            devices.insert(device_id.to_string(), device);
        }

        Ok(())
    }

    /// Configure storage device
    async fn configure_storage_device(&self, device: &mut Device) -> BiomeResult<()> {
        tracing::debug!("Configuring storage device: {}", device.id);

        // Set optimal I/O scheduler if available
        if let Some(path) = &device.path {
            let device_name = path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown");
            
            let scheduler_path = format!("/sys/block/{}/queue/scheduler", device_name);
            if tokio::fs::metadata(&scheduler_path).await.is_ok() {
                // Try to set mq-deadline scheduler for better performance
                if let Err(e) = tokio::fs::write(&scheduler_path, "mq-deadline").await {
                    tracing::debug!("Could not set I/O scheduler for {}: {}", device_name, e);
                }
            }
        }

        // Update device status
        device.status.state = DeviceState::Available;
        device.properties.insert("configured".to_string(), "true".to_string());

        Ok(())
    }

    /// Configure network device
    async fn configure_network_device(&self, device: &mut Device) -> BiomeResult<()> {
        tracing::debug!("Configuring network device: {}", device.id);

        // Check if device is up and configure if needed
        if let Some(path) = &device.path {
            let iface_name = path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown");
            
            // Check current state
            let operstate_path = format!("/sys/class/net/{}/operstate", iface_name);
            if let Ok(state) = tokio::fs::read_to_string(&operstate_path).await {
                match state.trim() {
                    "up" => {
                        device.status.state = DeviceState::Available;
                        device.status.health = DeviceHealth::Healthy;
                    }
                    "down" => {
                        device.status.state = DeviceState::Disabled;
                        device.status.health = DeviceHealth::Warning;
                    }
                    _ => {
                        device.status.state = DeviceState::Unknown;
                        device.status.health = DeviceHealth::Unknown;
                    }
                }
            }

            // Update network statistics
            let stats_path = format!("/sys/class/net/{}/statistics", iface_name);
            if let Ok(rx_bytes) = tokio::fs::read_to_string(format!("{}/rx_bytes", stats_path)).await {
                if let Ok(bytes) = rx_bytes.trim().parse::<u64>() {
                    device.status.usage.bytes_processed = bytes;
                }
            }
        }

        device.properties.insert("configured".to_string(), "true".to_string());
        Ok(())
    }

    /// Configure CPU device
    async fn configure_cpu_device(&self, device: &mut Device) -> BiomeResult<()> {
        tracing::debug!("Configuring CPU device: {}", device.id);

        // Set CPU governor for optimal performance
        let governor_path = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor";
        if tokio::fs::metadata(governor_path).await.is_ok() {
            // Try to set performance governor
            if let Err(e) = tokio::fs::write(governor_path, "performance").await {
                tracing::debug!("Could not set CPU governor: {}", e);
            } else {
                device.properties.insert("governor".to_string(), "performance".to_string());
            }
        }

        // Update CPU usage if available
        if let Ok(loadavg) = tokio::fs::read_to_string("/proc/loadavg").await {
            let load: Vec<&str> = loadavg.split_whitespace().collect();
            if let Some(load_1min) = load.get(0) {
                if let Ok(load_val) = load_1min.parse::<f64>() {
                    device.status.usage.usage_percent = (load_val * 100.0).min(100.0);
                }
            }
        }

        device.properties.insert("configured".to_string(), "true".to_string());
        Ok(())
    }

    /// Configure memory device
    async fn configure_memory_device(&self, device: &mut Device) -> BiomeResult<()> {
        tracing::debug!("Configuring memory device: {}", device.id);

        // Update memory usage information
        if let Ok(meminfo) = tokio::fs::read_to_string("/proc/meminfo").await {
            let mut total_memory = 0u64;
            let mut available_memory = 0u64;

            for line in meminfo.lines() {
                if let Some((key, value)) = line.split_once(':') {
                    let key = key.trim();
                    let value = value.trim();
                    
                    if let Some(size_str) = value.split_whitespace().next() {
                        if let Ok(size_kb) = size_str.parse::<u64>() {
                            let size_bytes = size_kb * 1024;
                            
                            match key {
                                "MemTotal" => total_memory = size_bytes,
                                "MemAvailable" => available_memory = size_bytes,
                                _ => {}
                            }
                        }
                    }
                }
            }

            // Update usage percentage
            if total_memory > 0 {
                let usage_percent = ((total_memory - available_memory) as f64 / total_memory as f64) * 100.0;
                device.status.usage.usage_percent = usage_percent;
                
                // Update health based on usage
                device.status.health = match usage_percent {
                    p if p > 95.0 => DeviceHealth::Unhealthy,
                    p if p > 85.0 => DeviceHealth::Warning,
                    _ => DeviceHealth::Healthy,
                };
            }
        }

        device.properties.insert("configured".to_string(), "true".to_string());
        Ok(())
    }

    /// Configure GPU device
    async fn configure_gpu_device(&self, device: &mut Device) -> BiomeResult<()> {
        tracing::debug!("Configuring GPU device: {}", device.id);

        // Check for GPU-specific configuration
        // This is a placeholder as GPU configuration depends on specific hardware
        device.status.state = DeviceState::Available;
        device.status.health = DeviceHealth::Healthy;
        device.properties.insert("configured".to_string(), "true".to_string());

        Ok(())
    }

    /// Configure generic device
    async fn configure_generic_device(&self, device: &mut Device) -> BiomeResult<()> {
        tracing::debug!("Configuring generic device: {}", device.id);

        // Basic device configuration
        device.status.state = DeviceState::Available;
        device.status.health = DeviceHealth::Healthy;
        device.properties.insert("configured".to_string(), "true".to_string());

        Ok(())
    }

    /// Get device by ID
    pub async fn get_device(&self, device_id: &str) -> Option<Device> {
        let devices = self.devices.read().await;
        devices.get(device_id).cloned()
    }

    /// Get all devices
    pub async fn get_all_devices(&self) -> HashMap<String, Device> {
        self.devices.read().await.clone()
    }

    /// Get devices by type
    pub async fn get_devices_by_type(&self, device_type: DeviceType) -> Vec<Device> {
        let devices = self.devices.read().await;
        devices
            .values()
            .filter(|d| {
                std::mem::discriminant(&d.device_type) == std::mem::discriminant(&device_type)
            })
            .cloned()
            .collect()
    }

    /// Shutdown device manager
    pub async fn shutdown(&self) -> BiomeResult<()> {
        tracing::info!("Shutting down device manager");

        // Get all device IDs
        let device_ids: Vec<String> = {
            let devices = self.devices.read().await;
            devices.keys().cloned().collect()
        };

        // Shutdown devices in reverse order (opposite of startup)
        for device_id in device_ids.iter().rev() {
            if let Err(e) = self.shutdown_device(device_id).await {
                tracing::warn!("Failed to shutdown device {}: {}", device_id, e);
            }
        }

        // Clear device collections
        {
            let mut devices = self.devices.write().await;
            devices.clear();
        }
        {
            let mut drivers = self.drivers.write().await;
            drivers.clear();
        }

        tracing::info!("Device manager shutdown complete");
        Ok(())
    }

    /// Shutdown a specific device
    async fn shutdown_device(&self, device_id: &str) -> BiomeResult<()> {
        let device = {
            let devices = self.devices.read().await;
            devices.get(device_id).cloned()
        };

        if let Some(mut device) = device {
            tracing::debug!("Shutting down device: {}", device_id);

            // Apply device-specific shutdown procedures
            match device.device_type {
                DeviceType::Storage => {
                    self.shutdown_storage_device(&mut device).await?;
                }
                DeviceType::Network => {
                    self.shutdown_network_device(&mut device).await?;
                }
                DeviceType::Cpu => {
                    self.shutdown_cpu_device(&mut device).await?;
                }
                DeviceType::Memory => {
                    self.shutdown_memory_device(&mut device).await?;
                }
                DeviceType::Gpu => {
                    self.shutdown_gpu_device(&mut device).await?;
                }
                _ => {
                    self.shutdown_generic_device(&mut device).await?;
                }
            }

            // Update device status
            device.status.state = DeviceState::NotPresent;
            device.status.power_state = PowerState::Off;

            // Update device in the collection
            let mut devices = self.devices.write().await;
            devices.insert(device_id.to_string(), device);
        }

        Ok(())
    }

    /// Shutdown storage device
    async fn shutdown_storage_device(&self, device: &mut Device) -> BiomeResult<()> {
        tracing::debug!("Shutting down storage device: {}", device.id);

        // Sync any pending writes
        if let Some(path) = &device.path {
            let device_name = path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown");
            
            // Force sync of pending writes
            if let Err(e) = tokio::process::Command::new("sync")
                .arg(path)
                .status()
                .await
            {
                tracing::warn!("Failed to sync device {}: {}", device_name, e);
            }
        }

        device.status.state = DeviceState::Disabled;
        device.status.power_state = PowerState::Off;
        device.properties.insert("shutdown".to_string(), "true".to_string());

        Ok(())
    }

    /// Shutdown network device
    async fn shutdown_network_device(&self, device: &mut Device) -> BiomeResult<()> {
        tracing::debug!("Shutting down network device: {}", device.id);

        // Close active connections and bring interface down
        if let Some(path) = &device.path {
            let iface_name = path.file_name()
                .and_then(|name| name.to_str())
                .unwrap_or("unknown");
            
            // Try to bring interface down
            if let Err(e) = tokio::process::Command::new("ip")
                .args(["link", "set", iface_name, "down"])
                .status()
                .await
            {
                tracing::warn!("Failed to bring down interface {}: {}", iface_name, e);
            }
        }

        device.status.state = DeviceState::Disabled;
        device.status.power_state = PowerState::Off;
        device.properties.insert("shutdown".to_string(), "true".to_string());

        Ok(())
    }

    /// Shutdown CPU device
    async fn shutdown_cpu_device(&self, device: &mut Device) -> BiomeResult<()> {
        tracing::debug!("Shutting down CPU device: {}", device.id);

        // Set CPU governor to powersave mode
        let governor_path = "/sys/devices/system/cpu/cpu0/cpufreq/scaling_governor";
        if tokio::fs::metadata(governor_path).await.is_ok() {
            if let Err(e) = tokio::fs::write(governor_path, "powersave").await {
                tracing::debug!("Could not set CPU governor to powersave: {}", e);
            }
        }

        device.status.state = DeviceState::Disabled;
        device.status.power_state = PowerState::Sleep;
        device.properties.insert("shutdown".to_string(), "true".to_string());

        Ok(())
    }

    /// Shutdown memory device
    async fn shutdown_memory_device(&self, device: &mut Device) -> BiomeResult<()> {
        tracing::debug!("Shutting down memory device: {}", device.id);

        // Memory cannot be "shutdown" in the traditional sense
        // Just mark it as disabled
        device.status.state = DeviceState::Disabled;
        device.status.power_state = PowerState::Sleep;
        device.properties.insert("shutdown".to_string(), "true".to_string());

        Ok(())
    }

    /// Shutdown GPU device
    async fn shutdown_gpu_device(&self, device: &mut Device) -> BiomeResult<()> {
        tracing::debug!("Shutting down GPU device: {}", device.id);

        // GPU-specific shutdown procedures would go here
        // This is a placeholder as GPU shutdown depends on specific hardware
        device.status.state = DeviceState::Disabled;
        device.status.power_state = PowerState::Off;
        device.properties.insert("shutdown".to_string(), "true".to_string());

        Ok(())
    }

    /// Shutdown generic device
    async fn shutdown_generic_device(&self, device: &mut Device) -> BiomeResult<()> {
        tracing::debug!("Shutting down generic device: {}", device.id);

        // Generic device shutdown
        device.status.state = DeviceState::Disabled;
        device.status.power_state = PowerState::Off;
        device.properties.insert("shutdown".to_string(), "true".to_string());

        Ok(())
    }
}

impl Default for DeviceConfig {
    fn default() -> Self {
        Self {
            enable_detection: true,
            auto_configure: true,
            driver_dir: PathBuf::from("/lib/drivers"),
            config_dir: dirs::config_dir()
                .unwrap_or_else(|| PathBuf::from("/etc"))
                .join("biomeos/devices"),
            blacklist: Vec::new(),
            whitelist: Vec::new(),
        }
    }
}
