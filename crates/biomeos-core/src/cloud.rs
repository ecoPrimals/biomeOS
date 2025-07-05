use std::collections::HashMap;
use std::path::PathBuf;
use serde::{Deserialize, Serialize};
use async_trait::async_trait;
use crate::BiomeResult;

/// Universal Cloud Interface - eliminates cloud provider vendor lock-in
#[async_trait]
pub trait UniversalCloudInterface {
    /// Get cloud provider information
    async fn provider_info(&self) -> BiomeResult<CloudProviderInfo>;
    
    /// Create virtual machine instance
    async fn create_instance(&self, spec: &InstanceSpec) -> BiomeResult<InstanceId>;
    
    /// Start instance
    async fn start_instance(&self, id: &InstanceId) -> BiomeResult<()>;
    
    /// Stop instance
    async fn stop_instance(&self, id: &InstanceId) -> BiomeResult<()>;
    
    /// Terminate instance
    async fn terminate_instance(&self, id: &InstanceId) -> BiomeResult<()>;
    
    /// Get instance status
    async fn instance_status(&self, id: &InstanceId) -> BiomeResult<InstanceStatus>;
    
    /// List all instances
    async fn list_instances(&self) -> BiomeResult<Vec<InstanceInfo>>;
    
    /// Create storage volume
    async fn create_volume(&self, spec: &VolumeSpec) -> BiomeResult<VolumeId>;
    
    /// Attach volume to instance
    async fn attach_volume(&self, volume_id: &VolumeId, instance_id: &InstanceId) -> BiomeResult<()>;
    
    /// Detach volume from instance
    async fn detach_volume(&self, volume_id: &VolumeId) -> BiomeResult<()>;
    
    /// Create network
    async fn create_network(&self, spec: &NetworkSpec) -> BiomeResult<NetworkId>;
    
    /// Create load balancer
    async fn create_load_balancer(&self, spec: &LoadBalancerSpec) -> BiomeResult<LoadBalancerId>;
    
    /// Upload object to storage
    async fn upload_object(&self, bucket: &str, key: &str, data: &[u8]) -> BiomeResult<()>;
    
    /// Download object from storage
    async fn download_object(&self, bucket: &str, key: &str) -> BiomeResult<Vec<u8>>;
    
    /// Execute serverless function
    async fn invoke_function(&self, spec: &FunctionSpec, payload: &[u8]) -> BiomeResult<FunctionResult>;
    
    /// Get billing information
    async fn get_billing_info(&self) -> BiomeResult<BillingInfo>;
    
    /// Export data for sovereignty
    async fn export_all_data(&self) -> BiomeResult<DataExport>;
}

/// Cloud provider information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CloudProviderInfo {
    pub name: String,
    pub provider_type: CloudProviderType,
    pub regions: Vec<String>,
    pub services: Vec<String>,
    pub pricing_model: PricingModel,
    pub sovereignty_compliance: SovereigntyCompliance,
}

/// Cloud provider types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloudProviderType {
    /// Amazon Web Services
    Aws { account_id: String },
    /// Google Cloud Platform
    Gcp { project_id: String },
    /// Microsoft Azure
    Azure { subscription_id: String },
    /// DigitalOcean
    DigitalOcean { team_id: Option<String> },
    /// Linode/Akamai
    Linode { account_id: String },
    /// Vultr
    Vultr { account_id: String },
    /// Hetzner Cloud
    Hetzner { project_id: String },
    /// OVH Cloud
    Ovh { project_id: String },
    /// Scaleway
    Scaleway { organization_id: String },
    /// Oracle Cloud
    Oracle { tenancy_id: String },
    /// IBM Cloud
    Ibm { account_id: String },
    /// Alibaba Cloud
    Alibaba { account_id: String },
    /// Private cloud (OpenStack, CloudStack, etc.)
    Private { name: String, api_endpoint: String },
    /// Edge cloud
    Edge { provider: String, location: String },
    /// Hybrid multi-cloud
    Hybrid { providers: Vec<String> },
    /// Self-hosted (no cloud)
    SelfHosted,
}

/// Pricing models
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PricingModel {
    PayAsYouGo,
    Reserved,
    Spot,
    Preemptible,
    Dedicated,
    Free,
    CustomContract,
}

/// Sovereignty compliance levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SovereigntyCompliance {
    /// Full sovereignty - no vendor lock-in
    FullSovereignty,
    /// Partial sovereignty - some vendor dependencies
    PartialSovereignty { locked_services: Vec<String> },
    /// No sovereignty - full vendor lock-in
    NoSovereignty { lock_in_risks: Vec<String> },
}

/// Universal instance specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceSpec {
    pub name: String,
    pub instance_type: String,
    pub image: CloudImage,
    pub region: String,
    pub network_config: NetworkConfig,
    pub storage_config: StorageConfig,
    pub security_config: SecurityConfig,
    pub metadata: HashMap<String, String>,
    pub user_data: Option<String>,
}

/// Cloud image specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CloudImage {
    /// Standard OS images
    Standard { os: String, version: String },
    /// Custom image
    Custom { image_id: String },
    /// Marketplace image
    Marketplace { name: String, publisher: String },
    /// Container-optimized image
    Container { runtime: String },
    /// biomeOS image
    BiomeOs { version: String, variant: String },
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub vpc_id: Option<String>,
    pub subnet_id: Option<String>,
    pub security_groups: Vec<String>,
    pub public_ip: bool,
    pub elastic_ip: Option<String>,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub root_volume: VolumeSpec,
    pub additional_volumes: Vec<VolumeSpec>,
}

/// Security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    pub key_pair: Option<String>,
    pub iam_role: Option<String>,
    pub encryption_at_rest: bool,
    pub encryption_in_transit: bool,
}

/// Instance and related types
pub type InstanceId = String;
pub type VolumeId = String;
pub type NetworkId = String;
pub type LoadBalancerId = String;

/// Volume specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeSpec {
    pub name: String,
    pub size_gb: u64,
    pub volume_type: VolumeType,
    pub iops: Option<u32>,
    pub encrypted: bool,
    pub snapshot_id: Option<String>,
}

/// Volume types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolumeType {
    Standard,
    Ssd,
    NvmeSsd,
    HighIops,
    Throughput,
    Cold,
    Archive,
}

/// Instance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceStatus {
    pub state: InstanceState,
    pub health: InstanceHealth,
    pub uptime_seconds: Option<u64>,
    pub public_ip: Option<String>,
    pub private_ip: Option<String>,
    pub resource_usage: InstanceResourceUsage,
}

/// Instance states
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstanceState {
    Pending,
    Running,
    Stopping,
    Stopped,
    Terminating,
    Terminated,
    Unknown,
}

/// Instance health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum InstanceHealth {
    Healthy,
    Unhealthy,
    Initializing,
    Unknown,
}

/// Instance resource usage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceResourceUsage {
    pub cpu_percent: f64,
    pub memory_percent: f64,
    pub network_in_mbps: f64,
    pub network_out_mbps: f64,
    pub disk_read_mbps: f64,
    pub disk_write_mbps: f64,
}

/// Instance information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstanceInfo {
    pub id: InstanceId,
    pub name: String,
    pub instance_type: String,
    pub status: InstanceStatus,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub cost_estimate: CostEstimate,
}

/// Network specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSpec {
    pub name: String,
    pub cidr_block: String,
    pub region: String,
    pub subnets: Vec<SubnetSpec>,
    pub internet_gateway: bool,
    pub nat_gateway: bool,
}

/// Subnet specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubnetSpec {
    pub name: String,
    pub cidr_block: String,
    pub availability_zone: String,
    pub public: bool,
}

/// Load balancer specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoadBalancerSpec {
    pub name: String,
    pub load_balancer_type: LoadBalancerType,
    pub scheme: LoadBalancerScheme,
    pub listeners: Vec<ListenerSpec>,
    pub target_groups: Vec<TargetGroupSpec>,
}

/// Load balancer types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancerType {
    Application,
    Network,
    Gateway,
}

/// Load balancer schemes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancerScheme {
    InternetFacing,
    Internal,
}

/// Listener specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListenerSpec {
    pub protocol: String,
    pub port: u16,
    pub ssl_certificate: Option<String>,
    pub default_action: ActionSpec,
}

/// Action specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ActionSpec {
    Forward { target_group: String },
    Redirect { url: String, status_code: u16 },
    FixedResponse { status_code: u16, content: String },
}

/// Target group specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetGroupSpec {
    pub name: String,
    pub protocol: String,
    pub port: u16,
    pub health_check: HealthCheckSpec,
    pub targets: Vec<TargetSpec>,
}

/// Health check specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthCheckSpec {
    pub protocol: String,
    pub path: String,
    pub interval_seconds: u32,
    pub timeout_seconds: u32,
    pub healthy_threshold: u32,
    pub unhealthy_threshold: u32,
}

/// Target specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetSpec {
    pub id: String,
    pub port: Option<u16>,
}

/// Function specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionSpec {
    pub name: String,
    pub runtime: String,
    pub code: FunctionCode,
    pub handler: String,
    pub timeout_seconds: u32,
    pub memory_mb: u32,
    pub environment: HashMap<String, String>,
}

/// Function code
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FunctionCode {
    Inline { source: String },
    ZipFile { path: PathBuf },
    S3Object { bucket: String, key: String },
    ContainerImage { uri: String },
}

/// Function result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionResult {
    pub status_code: u16,
    pub payload: Vec<u8>,
    pub execution_time_ms: u64,
    pub memory_used_mb: u32,
    pub logs: String,
}

/// Billing information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingInfo {
    pub current_month_cost: f64,
    pub projected_month_cost: f64,
    pub currency: String,
    pub cost_breakdown: Vec<CostBreakdown>,
    pub alerts: Vec<BillingAlert>,
}

/// Cost breakdown
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostBreakdown {
    pub service: String,
    pub cost: f64,
    pub usage_unit: String,
    pub usage_amount: f64,
}

/// Cost estimate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CostEstimate {
    pub hourly_cost: f64,
    pub monthly_estimate: f64,
    pub currency: String,
}

/// Billing alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BillingAlert {
    pub alert_type: String,
    pub message: String,
    pub threshold: f64,
    pub current_value: f64,
}

/// Data export for sovereignty
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataExport {
    pub export_id: String,
    pub format: ExportFormat,
    pub size_mb: u64,
    pub download_urls: Vec<String>,
    pub encryption_key: Option<String>,
    pub checksum: String,
}

/// Export formats
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ExportFormat {
    Json,
    Xml,
    Csv,
    Parquet,
    Archive,
    Raw,
}

/// Cloud adapter implementations for specific providers
pub struct AwsAdapter {
    pub credentials: AwsCredentials,
    pub region: String,
}

pub struct GcpAdapter {
    pub credentials: GcpCredentials,
    pub project_id: String,
}

pub struct AzureAdapter {
    pub credentials: AzureCredentials,
    pub subscription_id: String,
}

/// Cloud credentials (encrypted)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AwsCredentials {
    pub access_key_id: String,
    pub secret_access_key: String,
    pub session_token: Option<String>,
    pub role_arn: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GcpCredentials {
    pub service_account_key: String,
    pub project_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AzureCredentials {
    pub client_id: String,
    pub client_secret: String,
    pub tenant_id: String,
}

/// Universal Cloud Manager - manages multiple cloud providers
pub struct UniversalCloudManager {
    pub providers: HashMap<String, Box<dyn UniversalCloudInterface>>,
    pub default_provider: Option<String>,
    pub multi_cloud_config: MultiCloudConfig,
}

/// Multi-cloud configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MultiCloudConfig {
    pub load_balancing_strategy: LoadBalancingStrategy,
    pub cost_optimization: bool,
    pub geo_distribution: bool,
    pub disaster_recovery: DisasterRecoveryConfig,
    pub sovereignty_requirements: SovereigntyRequirements,
}

/// Load balancing strategies across clouds
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoadBalancingStrategy {
    RoundRobin,
    LeastCost,
    LeastLatency,
    GeographicProximity,
    AvailabilityZoneSpread,
    Custom { algorithm: String },
}

/// Disaster recovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DisasterRecoveryConfig {
    pub enabled: bool,
    pub backup_providers: Vec<String>,
    pub recovery_time_objective_minutes: u32,
    pub recovery_point_objective_minutes: u32,
    pub auto_failover: bool,
}

/// Sovereignty requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SovereigntyRequirements {
    pub data_residency: Vec<String>,
    pub encryption_requirements: EncryptionRequirements,
    pub compliance_frameworks: Vec<String>,
    pub vendor_independence: bool,
    pub exit_strategy: ExitStrategy,
}

/// Encryption requirements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptionRequirements {
    pub at_rest: bool,
    pub in_transit: bool,
    pub key_management: KeyManagementStrategy,
    pub customer_managed_keys: bool,
}

/// Key management strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum KeyManagementStrategy {
    CloudProvider,
    CustomerManaged,
    HybridEscrow,
    SovereignKeys,
}

/// Exit strategy for vendor independence
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExitStrategy {
    pub data_portability: bool,
    pub application_portability: bool,
    pub estimated_migration_time_hours: u32,
    pub migration_cost_estimate: f64,
    pub dependencies_to_resolve: Vec<String>,
}

impl UniversalCloudManager {
    /// Create a new universal cloud manager
    pub fn new() -> Self {
        Self {
            providers: HashMap::new(),
            default_provider: None,
            multi_cloud_config: MultiCloudConfig {
                load_balancing_strategy: LoadBalancingStrategy::RoundRobin,
                cost_optimization: true,
                geo_distribution: true,
                disaster_recovery: DisasterRecoveryConfig {
                    enabled: true,
                    backup_providers: vec!["aws".to_string()],
                    recovery_time_objective_minutes: 60,
                    recovery_point_objective_minutes: 60,
                    auto_failover: true,
                },
                sovereignty_requirements: SovereigntyRequirements {
                    data_residency: vec!["US".to_string()],
                    encryption_requirements: EncryptionRequirements {
                        at_rest: true,
                        in_transit: true,
                        key_management: KeyManagementStrategy::CloudProvider,
                        customer_managed_keys: false,
                    },
                    compliance_frameworks: vec!["HIPAA".to_string()],
                    vendor_independence: true,
                    exit_strategy: ExitStrategy {
                        data_portability: true,
                        application_portability: true,
                        estimated_migration_time_hours: 24,
                        migration_cost_estimate: 1000.0,
                        dependencies_to_resolve: vec!["aws".to_string()],
                    },
                },
            },
        }
    }
} 