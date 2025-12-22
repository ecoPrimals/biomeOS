//! Service Runtime Configurations
//!
//! This module contains runtime-related types including ServiceRuntime,
//! RuntimeType, container configurations, and process management.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Service runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRuntime {
    /// Runtime type
    pub runtime_type: RuntimeType,

    /// Runtime configuration
    pub config: RuntimeConfig,

    /// Environment variables
    pub environment: HashMap<String, String>,

    /// Command line arguments
    pub args: Vec<String>,

    /// Working directory
    pub working_directory: Option<String>,

    /// User context
    pub user: Option<String>,

    /// Process limits
    pub limits: ProcessLimits,
}

/// Runtime types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuntimeType {
    /// Container runtime
    Container {
        /// Container engine
        engine: ContainerEngine,
        /// Image specification
        image: ContainerImage,
    },

    /// Binary/native runtime
    Binary {
        /// Executable path
        executable: String,
        /// Binary type
        binary_type: BinaryType,
    },

    /// WASM runtime
    Wasm {
        /// WASM module
        module: String,
        /// Runtime engine
        engine: WasmEngine,
    },

    /// Function runtime
    Function {
        /// Function handler
        handler: String,
        /// Runtime environment
        runtime: String,
        /// Function code
        code: String,
    },

    /// VM runtime
    VirtualMachine {
        /// VM image
        image: String,
        /// VM configuration
        config: VmConfig,
    },
}

/// Container engines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContainerEngine {
    Docker,
    Podman,
    Containerd,
    Crio,
    Custom(String),
}

/// Container image specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerImage {
    /// Image registry
    pub registry: Option<String>,

    /// Image repository
    pub repository: String,

    /// Image tag
    pub tag: String,

    /// Image digest
    pub digest: Option<String>,

    /// Pull policy
    pub pull_policy: ImagePullPolicy,

    /// Pull secrets
    pub pull_secrets: Vec<String>,
}

/// Image pull policies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ImagePullPolicy {
    Always,
    IfNotPresent,
    Never,
}

/// Binary types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BinaryType {
    /// Regular executable
    Executable,

    /// Shared library
    SharedLibrary,

    /// Static binary
    Static,

    /// Script
    Script {
        /// Script interpreter
        interpreter: String,
    },
}

/// WASM engines
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WasmEngine {
    Wasmtime,
    Wasmer,
    WasmEdge,
    Custom(String),
}

/// Virtual machine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmConfig {
    /// VM type
    pub vm_type: VmType,

    /// CPU configuration
    pub cpu: VmCpuConfig,

    /// Memory configuration
    pub memory: VmMemoryConfig,

    /// Storage configuration
    pub storage: Vec<VmStorageConfig>,

    /// Network configuration
    pub network: Vec<VmNetworkConfig>,
}

/// VM types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VmType {
    Qemu,
    Kvm,
    Xen,
    VirtualBox,
    VMware,
    Custom(String),
}

/// VM CPU configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmCpuConfig {
    /// Number of cores
    pub cores: u32,

    /// CPU model
    pub model: Option<String>,

    /// CPU features
    pub features: Vec<String>,
}

/// VM memory configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmMemoryConfig {
    /// Memory size (MB)
    pub size_mb: u32,

    /// Memory balloon enabled
    pub balloon: bool,

    /// Memory sharing
    pub sharing: VmMemorySharing,
}

/// VM memory sharing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VmMemorySharing {
    None,
    Shared,
    Private,
}

/// VM storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmStorageConfig {
    /// Storage type
    pub storage_type: VmStorageType,

    /// Storage size (GB)
    pub size_gb: u32,

    /// Mount point
    pub mount_point: String,

    /// Read-only
    pub read_only: bool,
}

/// VM storage types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VmStorageType {
    Disk,
    Volume,
    Network,
}

/// VM network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VmNetworkConfig {
    /// Network type
    pub network_type: VmNetworkType,

    /// MAC address
    pub mac_address: Option<String>,

    /// Network bridge
    pub bridge: Option<String>,
}

/// VM network types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VmNetworkType {
    Bridge,
    Nat,
    HostOnly,
    Internal,
}

/// Runtime configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    /// Runtime parameters
    pub parameters: HashMap<String, String>,

    /// Security options
    pub security_options: Vec<String>,

    /// Volume mounts
    pub mounts: Vec<RuntimeMount>,

    /// Device mounts
    pub devices: Vec<RuntimeDevice>,

    /// Runtime capabilities
    pub capabilities: RuntimeCapabilities,
}

/// Runtime mount
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeMount {
    /// Source path
    pub source: String,

    /// Target path
    pub target: String,

    /// Mount type
    pub mount_type: MountType,

    /// Mount options
    pub options: Vec<String>,

    /// Read-only
    pub read_only: bool,
}

/// Mount types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MountType {
    Bind,
    Volume,
    Tmpfs,
    Cache,
}

/// Runtime device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeDevice {
    /// Host device path
    pub host_path: String,

    /// Container device path
    pub container_path: String,

    /// Device permissions
    pub permissions: String,
}

/// Runtime capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeCapabilities {
    /// Capabilities to add
    pub add: Vec<String>,

    /// Capabilities to drop
    pub drop: Vec<String>,
}

/// Process limits
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessLimits {
    /// Maximum file descriptors
    pub max_fds: Option<u32>,

    /// Maximum processes
    pub max_processes: Option<u32>,

    /// Maximum core dump size
    pub max_core_dump_size: Option<u64>,

    /// Maximum stack size
    pub max_stack_size: Option<u64>,

    /// Process nice value
    pub nice: Option<i32>,

    /// OOM score adjustment
    pub oom_score_adj: Option<i32>,
}

/// Default implementation for ServiceRuntime
impl Default for ServiceRuntime {
    fn default() -> Self {
        Self {
            runtime_type: RuntimeType::Binary {
                executable: "service".to_string(),
                binary_type: BinaryType::Executable,
            },
            config: RuntimeConfig {
                parameters: HashMap::new(),
                security_options: vec![],
                mounts: vec![],
                devices: vec![],
                capabilities: RuntimeCapabilities {
                    add: vec![],
                    drop: vec!["ALL".to_string()],
                },
            },
            environment: HashMap::new(),
            args: vec![],
            working_directory: None,
            user: None,
            limits: ProcessLimits {
                max_fds: Some(1024),
                max_processes: Some(100),
                max_core_dump_size: Some(0),
                max_stack_size: None,
                nice: None,
                oom_score_adj: None,
            },
        }
    }
} 