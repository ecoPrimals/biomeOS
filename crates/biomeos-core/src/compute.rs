use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use crate::BiomeResult;

/// Universal Compute Interface - eliminates GPU/accelerator vendor lock-in
#[async_trait]
pub trait UniversalComputeInterface {
    /// Get compute provider information
    async fn provider_info(&self) -> BiomeResult<ComputeProviderInfo>;
    
    /// List available compute devices
    async fn list_devices(&self) -> BiomeResult<Vec<ComputeDevice>>;
    
    /// Get device information
    async fn device_info(&self, device_id: &str) -> BiomeResult<ComputeDevice>;
    
    /// Allocate compute resources
    async fn allocate_resources(&self, spec: &ComputeResourceSpec) -> BiomeResult<ComputeAllocation>;
    
    /// Release compute resources
    async fn release_resources(&self, allocation_id: &str) -> BiomeResult<()>;
    
    /// Execute compute kernel/program
    async fn execute_kernel(&self, spec: &KernelSpec) -> BiomeResult<KernelResult>;
    
    /// Copy data to device
    async fn copy_to_device(&self, allocation_id: &str, data: &[u8]) -> BiomeResult<DeviceMemoryId>;
    
    /// Copy data from device
    async fn copy_from_device(&self, memory_id: &DeviceMemoryId) -> BiomeResult<Vec<u8>>;
    
    /// Synchronize device operations
    async fn synchronize(&self, allocation_id: &str) -> BiomeResult<()>;
    
    /// Get device utilization metrics
    async fn get_utilization(&self, device_id: &str) -> BiomeResult<ComputeUtilization>;
    
    /// Profile kernel execution
    async fn profile_kernel(&self, spec: &KernelSpec) -> BiomeResult<KernelProfile>;
}

/// Compute provider information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeProviderInfo {
    pub name: String,
    pub provider_type: ComputeProviderType,
    pub version: String,
    pub capabilities: Vec<ComputeCapability>,
    pub sovereignty_compliance: ComputeSovereignty,
}

/// Compute provider types - vendor agnostic
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputeProviderType {
    /// NVIDIA GPUs
    Nvidia { driver_version: String, cuda_version: String },
    /// AMD GPUs
    Amd { driver_version: String, rocm_version: String },
    /// Intel GPUs/CPUs
    Intel { driver_version: String, oneapi_version: String },
    /// Apple Silicon
    Apple { metal_version: String, mlcompute_version: String },
    /// Qualcomm Adreno
    Qualcomm { adreno_version: String, hexagon_version: String },
    /// ARM Mali
    Arm { mali_version: String, compute_library_version: String },
    /// Google TPU
    Google { tpu_version: String, jax_version: String },
    /// Amazon Inferentia
    Amazon { inferentia_version: String, neuron_version: String },
    /// Cerebras Wafer Scale Engine
    Cerebras { wse_version: String, sdk_version: String },
    /// Graphcore IPU
    Graphcore { ipu_version: String, poplar_version: String },
    /// SambaNova DataFlow
    SambaNova { dataflow_version: String, snsdk_version: String },
    /// Neuromorphic chips
    Neuromorphic { chip_type: String, sdk_version: String },
    /// Quantum computers
    Quantum { provider: String, api_version: String },
    /// FPGA
    Fpga { vendor: String, toolchain_version: String },
    /// CPU-only fallback
    Cpu { architecture: String, instruction_sets: Vec<String> },
    /// Custom/future compute providers
    Custom { vendor: String, api_version: String, capabilities: Vec<String> },
}

/// Compute capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputeCapability {
    /// Parallel processing
    Parallel { max_threads: u32, max_blocks: u32 },
    /// Tensor operations
    Tensor { precision: Vec<TensorPrecision>, max_dimensions: u32 },
    /// Machine learning
    MachineLearning { frameworks: Vec<String>, model_formats: Vec<String> },
    /// Image/video processing
    MediaProcessing { codecs: Vec<String>, max_resolution: String },
    /// Cryptographic operations
    Cryptography { algorithms: Vec<String>, key_sizes: Vec<u32> },
    /// Scientific computing
    Scientific { libraries: Vec<String>, precision: Vec<String> },
    /// Ray tracing
    RayTracing { rt_cores: u32, max_rays_per_second: u64 },
    /// Quantum operations
    Quantum { qubits: u32, gate_fidelity: f64, coherence_time_ms: f64 },
    /// Custom capability
    Custom { name: String, parameters: HashMap<String, String> },
}

/// Tensor precision support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TensorPrecision {
    Float16,
    Float32,
    Float64,
    Int8,
    Int16,
    Int32,
    Int64,
    BFloat16,
    Complex64,
    Complex128,
    Custom { name: String, bits: u32 },
}

/// Compute sovereignty compliance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputeSovereignty {
    /// Fully sovereign - no vendor dependencies
    FullSovereignty,
    /// Partial sovereignty - some vendor SDKs required
    PartialSovereignty { dependencies: Vec<String> },
    /// Vendor locked - requires proprietary drivers/software
    VendorLocked { lock_ins: Vec<String> },
}

/// Compute device information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeDevice {
    pub id: String,
    pub name: String,
    pub device_type: ComputeDeviceType,
    pub memory_mb: u64,
    pub compute_units: u32,
    pub max_clock_mhz: u32,
    pub capabilities: Vec<ComputeCapability>,
    pub power_consumption_watts: Option<u32>,
    pub thermal_design_power_watts: Option<u32>,
    pub availability: DeviceAvailability,
}

/// Compute device types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputeDeviceType {
    Gpu,
    Cpu,
    Tpu,
    Ipu,
    Dpu,
    Npu,
    Qpu,
    Fpga,
    Asic,
    Neuromorphic,
    Custom { device_type: String },
}

/// Device availability status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeviceAvailability {
    Available,
    Busy { estimated_free_time_ms: u64 },
    Unavailable { reason: String },
    MaintenanceMode,
    PowerSaving,
}

/// Compute resource specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeResourceSpec {
    pub device_ids: Vec<String>,
    pub memory_mb: u64,
    pub compute_units: Option<u32>,
    pub max_duration_ms: Option<u64>,
    pub priority: ComputePriority,
    pub isolation_level: IsolationLevel,
    pub power_budget_watts: Option<u32>,
}

/// Compute priority levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ComputePriority {
    Low,
    Normal,
    High,
    Critical,
    Preemptible,
}

/// Isolation levels for security
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IsolationLevel {
    None,
    Process,
    Container,
    VirtualMachine,
    SecureEnclave,
}

/// Compute allocation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeAllocation {
    pub allocation_id: String,
    pub allocated_devices: Vec<String>,
    pub allocated_memory_mb: u64,
    pub allocation_time: chrono::DateTime<chrono::Utc>,
    pub expiry_time: Option<chrono::DateTime<chrono::Utc>>,
    pub cost_estimate: Option<ComputeCost>,
}

/// Compute cost information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeCost {
    pub currency: String,
    pub cost_per_hour: f64,
    pub estimated_total_cost: f64,
    pub billing_model: BillingModel,
}

/// Billing models for compute resources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BillingModel {
    PayPerUse,
    Reserved,
    Spot,
    Free,
    Credits,
}

/// Kernel specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelSpec {
    pub name: String,
    pub kernel_type: KernelType,
    pub code: KernelCode,
    pub entry_point: String,
    pub input_data: Vec<KernelInput>,
    pub output_spec: Vec<KernelOutput>,
    pub work_size: WorkSize,
    pub optimization_level: OptimizationLevel,
}

/// Kernel types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KernelType {
    /// CUDA kernel
    Cuda,
    /// OpenCL kernel
    OpenCl,
    /// ROCm/HIP kernel
    Hip,
    /// SYCL kernel
    Sycl,
    /// Metal compute kernel
    Metal,
    /// DirectCompute kernel
    DirectCompute,
    /// Vulkan compute shader
    Vulkan,
    /// WGSL (WebGPU)
    Wgsl,
    /// TensorFlow kernel
    TensorFlow,
    /// PyTorch kernel
    PyTorch,
    /// JAX kernel
    Jax,
    /// Custom kernel
    Custom { language: String, runtime: String },
}

/// Kernel code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KernelCode {
    /// Source code
    Source { code: String, includes: Vec<String> },
    /// Compiled binary
    Binary { data: Vec<u8>, format: String },
    /// Precompiled library
    Library { path: String, symbol: String },
    /// Graph definition
    Graph { nodes: Vec<GraphNode>, edges: Vec<GraphEdge> },
}

/// Graph node for dataflow kernels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub operation: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Graph edge for dataflow kernels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub from_node: String,
    pub to_node: String,
    pub data_type: String,
    pub shape: Vec<u32>,
}

/// Kernel input specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelInput {
    pub name: String,
    pub data_type: DataType,
    pub shape: Vec<u32>,
    pub memory_id: Option<DeviceMemoryId>,
    pub data: Option<Vec<u8>>,
}

/// Kernel output specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelOutput {
    pub name: String,
    pub data_type: DataType,
    pub shape: Vec<u32>,
}

/// Data types for kernel I/O
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataType {
    Float16,
    Float32,
    Float64,
    Int8,
    Int16,
    Int32,
    Int64,
    UInt8,
    UInt16,
    UInt32,
    UInt64,
    Bool,
    Complex64,
    Complex128,
    String,
    Tensor { element_type: Box<DataType>, shape: Vec<u32> },
    Custom { name: String, size_bytes: u32 },
}

/// Work size specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WorkSize {
    /// 1D work size
    Linear { global: u32, local: Option<u32> },
    /// 2D work size
    Grid2D { global_x: u32, global_y: u32, local_x: Option<u32>, local_y: Option<u32> },
    /// 3D work size
    Grid3D { global_x: u32, global_y: u32, global_z: u32, local_x: Option<u32>, local_y: Option<u32>, local_z: Option<u32> },
    /// Automatic sizing
    Auto { hint: String },
}

/// Optimization levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OptimizationLevel {
    Debug,
    O1,
    O2,
    O3,
    Size,
    Fast,
    Custom { flags: Vec<String> },
}

/// Device memory ID
pub type DeviceMemoryId = String;

/// Kernel execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelResult {
    pub execution_id: String,
    pub outputs: Vec<KernelOutput>,
    pub execution_time_ms: u64,
    pub memory_used_mb: u64,
    pub energy_consumed_joules: Option<f64>,
    pub performance_metrics: PerformanceMetrics,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub flops_per_second: Option<f64>,
    pub memory_bandwidth_gbps: Option<f64>,
    pub cache_hit_rate: Option<f64>,
    pub occupancy_percent: Option<f64>,
    pub thermal_throttling: bool,
    pub power_efficiency_gflops_per_watt: Option<f64>,
}

/// Kernel profiling information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelProfile {
    pub kernel_name: String,
    pub total_time_ms: u64,
    pub compute_time_ms: u64,
    pub memory_transfer_time_ms: u64,
    pub optimization_suggestions: Vec<String>,
    pub bottlenecks: Vec<PerformanceBottleneck>,
    pub resource_utilization: ResourceUtilization,
}

/// Performance bottlenecks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceBottleneck {
    MemoryBandwidth,
    ComputeUnits,
    CacheEfficiency,
    Synchronization,
    PowerThrottling,
    CustomBottleneck { name: String, description: String },
}

/// Resource utilization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    pub compute_utilization_percent: f64,
    pub memory_utilization_percent: f64,
    pub cache_utilization_percent: f64,
    pub power_utilization_percent: f64,
}

/// Compute utilization metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeUtilization {
    pub device_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub gpu_utilization_percent: f64,
    pub memory_utilization_percent: f64,
    pub temperature_celsius: f64,
    pub power_consumption_watts: f64,
    pub fan_speed_percent: f64,
    pub clock_speeds: ClockSpeeds,
    pub running_processes: Vec<ComputeProcess>,
}

/// Clock speeds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClockSpeeds {
    pub core_clock_mhz: u32,
    pub memory_clock_mhz: u32,
    pub shader_clock_mhz: Option<u32>,
}

/// Running compute process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeProcess {
    pub process_id: u32,
    pub name: String,
    pub memory_used_mb: u64,
    pub gpu_utilization_percent: f64,
}

/// Universal Compute Manager - manages multiple compute providers
pub struct UniversalComputeManager {
    pub providers: HashMap<String, Box<dyn UniversalComputeInterface>>,
    pub default_provider: Option<String>,
    pub provider_preference: Vec<String>,
    pub sovereignty_requirements: ComputeSovereigntyRequirements,
}

/// Compute sovereignty requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeSovereigntyRequirements {
    pub require_sovereign_providers: bool,
    pub allow_vendor_locked_providers: bool,
    pub crypto_lock_external_apis: bool,
    pub prefer_open_source_drivers: bool,
    pub require_local_execution: bool,
}

impl UniversalComputeManager {
    /// Create new compute manager with sovereignty-first defaults
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            default_provider: None,
            provider_preference: vec![
                "cpu".to_string(),          // Always available fallback
                "intel".to_string(),        // Open source friendly
                "amd".to_string(),          // ROCm is open source
                "apple".to_string(),        // Good for Apple Silicon
                "nvidia".to_string(),       // Widely used but proprietary
            ],
            sovereignty_requirements: ComputeSovereigntyRequirements {
                require_sovereign_providers: false,
                allow_vendor_locked_providers: true,
                crypto_lock_external_apis: true,
                prefer_open_source_drivers: true,
                require_local_execution: false,
            },
        }
    }
    
    /// Add compute provider
    pub fn add_provider(&mut self, name: String, provider: Box<dyn UniversalComputeInterface>) {
        self.providers.insert(name, provider);
    }
    
    /// Get best available compute provider based on sovereignty requirements
    pub async fn get_best_provider(&self) -> Option<&Box<dyn UniversalComputeInterface>> {
        for provider_name in &self.provider_preference {
            if let Some(provider) = self.providers.get(provider_name) {
                // Check sovereignty compliance
                if let Ok(info) = provider.provider_info().await {
                    if self.meets_sovereignty_requirements(&info.sovereignty_compliance) {
                        return Some(provider);
                    }
                }
            }
        }
        None
    }
    
    fn meets_sovereignty_requirements(&self, compliance: &ComputeSovereignty) -> bool {
        match compliance {
            ComputeSovereignty::FullSovereignty => true,
            ComputeSovereignty::PartialSovereignty { .. } => !self.sovereignty_requirements.require_sovereign_providers,
            ComputeSovereignty::VendorLocked { .. } => self.sovereignty_requirements.allow_vendor_locked_providers,
        }
    }
} 