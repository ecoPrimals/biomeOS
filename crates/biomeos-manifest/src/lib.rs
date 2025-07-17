// biomeOS Manifest Library
// Core types and structures for biome manifest files

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

// Error types
pub type BiomeResult<T> = Result<T, BiomeError>;

#[derive(Debug, Clone)]
pub enum BiomeError {
    ConfigError(String),
    ValidationError(String),
    SourceError(String),
}
// No sub-modules - all types are defined in this file
impl std::fmt::Display for BiomeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            BiomeError::ConfigError(msg) => write!(f, "Config error: {}", msg),
            BiomeError::ValidationError(msg) => write!(f, "Validation error: {}", msg),
            BiomeError::SourceError(msg) => write!(f, "Source error: {}", msg),
        }
    }
}

impl std::error::Error for BiomeError {}

/// Primary biome manifest structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeManifest {
    /// API version for compatibility
    #[serde(default = "default_api_version")]
    pub api_version: String,

    /// Resource kind (always "Biome")
    #[serde(default = "default_kind")]
    pub kind: String,

    /// Manifest metadata
    pub metadata: ManifestMetadata,

    /// Source management configuration
    #[serde(default)]
    pub sources: SourceConfig,

    /// Primal component specifications
    #[serde(default)]
    pub primals: HashMap<String, PrimalSpec>,

    /// Service definitions
    #[serde(default)]
    pub services: HashMap<String, ServiceSpec>,

    /// MYCORRHIZA sovereignty configuration
    #[serde(default)]
    pub mycorrhiza: MycorrhizaConfig,

    /// Volume definitions
    #[serde(default)]
    pub volumes: HashMap<String, VolumeSpec>,

    /// Network definitions
    #[serde(default)]
    pub networks: HashMap<String, NetworkSpec>,

    /// Global networking configuration
    pub networking: Option<NetworkingSpec>,

    /// Global security configuration  
    pub security: Option<SecuritySpec>,

    /// Resource allocation and limits
    pub resources: Option<ResourceSpec>,

    /// BYOB: Scheduling configurations for dynamic biomes
    #[serde(default)]
    pub schedules: HashMap<String, ScheduleConfig>,

    /// BYOB: Environment-specific configurations
    #[serde(default)]
    pub environments: HashMap<String, EnvironmentConfig>,

    /// BYOB: Dependency management
    pub dependencies: Option<DependencyConfig>,

    /// Custom extensions for future features
    pub extensions: Option<HashMap<String, serde_json::Value>>,

    /// Recursive biome references for nested deployments
    pub biomes: Option<HashMap<String, BiomeReference>>,

    /// Nested biome definitions
    pub nested_biomes: Option<HashMap<String, BiomeManifest>>,

    /// Topology configuration for complex deployments
    pub topology: Option<TopologyConfig>,

    /// Iterative deployment configurations
    pub iterative: Option<HashMap<String, IterativeDeployment>>,

    /// Template composition for complex biomes
    pub templates: Option<HashMap<String, BiomeManifest>>,

    /// Recursive monitoring configuration
    pub monitoring: Option<RecursiveMonitoring>,

    /// Recursive scaling configuration
    pub scaling: Option<RecursiveScaling>,
}

/// Manifest metadata structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ManifestMetadata {
    /// Human-readable name for this biome
    pub name: String,

    /// Semantic version
    pub version: String,

    /// Description of this biome's purpose
    pub description: Option<String>,

    /// Tags for categorization and discovery
    pub tags: Option<Vec<String>>,

    /// Author/maintainer information
    pub author: Option<String>,

    /// Creation timestamp
    pub created: Option<DateTime<Utc>>,

    /// Biome specialization type
    pub specialization: Option<BiomeSpecialization>,

    /// Source repository information
    pub repository: Option<String>,

    /// License information
    pub license: Option<String>,

    /// BYOB: Creator username for attribution
    pub created_by: Option<String>,

    /// BYOB: Parent biome ID if this is a fork
    pub forked_from: Option<String>,

    /// BYOB: Sharing configuration
    pub sharing: Option<SharingConfig>,

    /// BYOB: Niche classifications
    pub niches: Option<NicheClassification>,

    /// BYOB: Template metadata if this biome serves as a template
    pub template: Option<TemplateMetadata>,

    /// Custom metadata extensions
    pub custom: Option<HashMap<String, serde_json::Value>>,
}

/// Primal specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalSpec {
    /// Whether this primal is enabled
    #[serde(default = "default_true")]
    pub enabled: bool,

    /// Type of this primal
    pub primal_type: PrimalType,

    /// Priority for startup ordering (lower = earlier)
    #[serde(default = "default_priority")]
    pub priority: u32,

    /// Version requirement
    pub version: Option<String>,

    /// Source specification for this primal
    pub source: Option<SourceSpec>,

    /// Dependencies on other primals
    #[serde(default)]
    pub depends_on: Vec<String>,

    /// Startup timeout
    pub startup_timeout: Option<String>,

    /// Primal-specific configuration
    pub config: Option<serde_json::Value>,

    /// Networking configuration for this primal
    pub networking: Option<PrimalNetworking>,

    /// Resource requirements for this primal
    pub resources: Option<PrimalResources>,

    /// Custom primal extensions
    pub extensions: Option<HashMap<String, serde_json::Value>>,
}

/// Service specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSpec {
    /// Runtime type for this service
    pub runtime: RuntimeType,

    /// Source specification
    pub source: Option<SourceSpec>,

    /// Responsible primal
    pub primal: String,

    /// Container image (for container runtime)
    pub image: Option<String>,

    /// Dependencies on other services
    #[serde(default)]
    pub depends_on: Vec<String>,

    /// Port mappings
    #[serde(default)]
    pub ports: Vec<String>,

    /// Volume mappings
    #[serde(default)]
    pub volumes: Vec<String>,

    /// Environment variables
    #[serde(default)]
    pub environment: HashMap<String, String>,

    /// Service configuration
    pub config: Option<serde_json::Value>,
}

/// Source configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SourceConfig {
    /// Global source repositories
    #[serde(default)]
    pub repositories: HashMap<String, RepositorySpec>,

    /// Authentication configurations
    #[serde(default)]
    pub auth: HashMap<String, AuthSpec>,

    /// Build configurations
    #[serde(default)]
    pub build: BuildConfig,

    /// Distribution configurations
    #[serde(default)]
    pub distribution: DistributionConfig,
}

/// MYCORRHIZA configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct MycorrhizaConfig {
    /// Energy flow state
    #[serde(default)]
    pub energy_flow: EnergyFlowState,

    /// Personal AI configuration
    #[serde(default)]
    pub personal_ai: PersonalAiConfig,

    /// Trusted externals configuration
    #[serde(default)]
    pub trusted_externals: TrustedExternalsConfig,

    /// Commercial access configuration
    #[serde(default)]
    pub commercial_access: CommercialAccessConfig,

    /// Enforcement configuration
    #[serde(default)]
    pub enforcement: EnforcementConfig,
}

/// Primal types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum PrimalType {
    BearDog,
    Songbird,
    NestGate,
    Toadstool,
    Squirrel,
    Custom(String),
}

/// Runtime types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeType {
    Container,
    Wasm,
    Native,
    GPU,
    Agent,
}

/// Source types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum SourceType {
    Git,
    Local,
    Http,
    Container,
    Custom(String),
}

/// Energy flow states
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum EnergyFlowState {
    Closed,
    PrivateOpen,
    CommercialOpen,
}

/// Biome specialization types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "kebab-case")]
pub enum BiomeSpecialization {
    AiResearch,
    DataScience,
    WebDevelopment,
    Gaming,
    Scientific,
    Enterprise,
    Edge,
    GamingServer,
    GamingClient,
    GamingDevelopment,
    Biocomputation,
    DataAnalysis,
    MachineLearning,
    QuantumComputing,
    DevOps,
    Frontend,
    Backend,
    Mobile,
    DataCenter,
    EdgeComputing,
    IoTGateway,
    NetworkingLab,
    ContentCreation,
    DigitalArt,
    Music,
    Streaming,
    Ecommerce,
    Finance,
    Legal,
    Healthcare,
    LearningEnvironment,
    ResearchLab,
    Teaching,
    PersonalProductivity,
    HomeAutomation,
    MediaCenter,
    CommunityHub,
    Collaboration,
    SocialNetwork,
    DualPurpose,
    DynamicShift,
    Custom(String),
}

/// Threat response types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum ThreatResponse {
    Block,
    Warn,
    Preserve,
    BlockAndPreserve,
}

/// Default functions
pub fn default_api_version() -> String {
    "v1".to_string()
}

pub fn default_kind() -> String {
    "Biome".to_string()
}

pub fn default_true() -> bool {
    true
}

pub fn default_priority() -> u32 {
    100
}

impl Default for EnergyFlowState {
    fn default() -> Self {
        EnergyFlowState::Closed
    }
}

impl Default for ThreatResponse {
    fn default() -> Self {
        ThreatResponse::Block
    }
}

// Additional configuration structures
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct PersonalAiConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
    #[serde(default)]
    pub local_models: Vec<String>,
    #[serde(default)]
    pub api_keys: Vec<ApiKeyConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TrustedExternalsConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub grants: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CommercialAccessConfig {
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub licensed_providers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct EnforcementConfig {
    #[serde(default = "default_true")]
    pub deep_packet_inspection: bool,
    #[serde(default = "default_true")]
    pub api_signature_detection: bool,
    #[serde(default = "default_true")]
    pub behavioral_analysis: bool,
    #[serde(default)]
    pub threat_response: ThreatResponse,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiKeyConfig {
    pub provider: String,
    pub key_ref: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BuildConfig {
    pub parallel_jobs: Option<u32>,
    pub timeout: Option<String>,
    pub cache_size: Option<String>,
    pub targets: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DistributionConfig {
    pub channels: Option<Vec<String>>,
    pub signing_key: Option<String>,
    pub registry: Option<String>,
}

// Repository and source types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RepositorySpec {
    pub repo_type: RepositoryType,
    pub url: String,
    pub default_ref: Option<String>,
    pub auth: Option<String>,
    pub cache: Option<CacheSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum RepositoryType {
    Git,
    Mercurial,
    Svn,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceSpec {
    pub source_type: SourceType,
    pub location: String,
    pub version: Option<String>,
    pub auth: Option<String>,
    pub build_command: Option<String>,
    #[serde(default)]
    pub watch: bool,
    pub checksum: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthSpec {
    pub auth_type: AuthType,
    pub config: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AuthType {
    SshKey,
    Token,
    Certificate,
    OAuth,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheSpec {
    pub enabled: bool,
    pub ttl: Option<String>,
    pub size_limit: Option<String>,
}

// Volume and network types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeSpec {
    pub volume_type: VolumeType,
    pub source: Option<String>,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum VolumeType {
    EmptyDir,
    HostPath,
    ConfigMap,
    Secret,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkSpec {
    pub driver: String,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkingSpec {
    pub mode: Option<String>,
    pub dns: Option<DnsSpec>,
    pub discovery: Option<DiscoverySpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecuritySpec {
    pub auth: Option<AuthSpec>,
    pub tls: Option<TlsSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceSpec {
    pub cpu: Option<CpuSpec>,
    pub memory: Option<MemorySpec>,
    pub storage: Option<StorageSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DnsSpec {
    pub servers: Option<Vec<String>>,
    pub search_domains: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoverySpec {
    pub method: String,
    pub config: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsSpec {
    pub enabled: bool,
    pub certificates: Option<serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuSpec {
    pub max_cores: Option<f64>,
    pub shares: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySpec {
    pub max_mb: Option<u64>,
    pub swap_mb: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSpec {
    pub max_mb: Option<u64>,
    pub storage_type: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalNetworking {
    pub ports: Option<Vec<u16>>,
    pub host: Option<String>,
    pub discovery: Option<DiscoverySpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalResources {
    pub cpu: Option<CpuSpec>,
    pub memory: Option<MemorySpec>,
    pub storage: Option<StorageSpec>,
}

// Complex deployment structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduleConfig {
    pub active: String,
    pub timezone: Option<String>,
    pub config_overrides: Option<serde_json::Value>,
    #[serde(default)]
    pub primal_overrides: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub service_overrides: HashMap<String, serde_json::Value>,
    #[serde(default = "default_true")]
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentConfig {
    pub description: Option<String>,
    pub extends: Option<String>,
    #[serde(default)]
    pub primals: HashMap<String, PrimalSpec>,
    #[serde(default)]
    pub services: HashMap<String, ServiceSpec>,
    #[serde(default)]
    pub environment: HashMap<String, String>,
    pub resources: Option<ResourceSpec>,
    pub security: Option<SecuritySpec>,
    pub mycorrhiza: Option<MycorrhizaConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencyConfig {
    #[serde(default)]
    pub requires: Vec<DependencySpec>,
    #[serde(default)]
    pub suggests: Vec<DependencySpec>,
    #[serde(default)]
    pub conflicts: Vec<DependencySpec>,
    #[serde(default)]
    pub features: HashMap<String, FeatureSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DependencySpec {
    pub name: String,
    pub version: Option<String>,
    pub source: Option<SourceSpec>,
    #[serde(default)]
    pub optional: bool,
    pub reason: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeatureSpec {
    pub description: String,
    #[serde(default)]
    pub dependencies: Vec<String>,
    #[serde(default)]
    pub services: Vec<String>,
    pub config: Option<serde_json::Value>,
    #[serde(default)]
    pub default_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiomeReference {
    pub topology: TopologyPattern,
    pub instances: u32,
    pub regions: Option<Vec<String>>,
    pub template: String,
    pub depends_on: Option<Vec<String>>,
    pub placement_strategy: Option<String>,
    pub hosts: Option<Vec<BiomeReference>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TopologyPattern {
    Ring,
    Mesh,
    Cluster,
    Hierarchy,
    Singleton,
    #[serde(rename = "custom")]
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologyConfig {
    pub topology_type: String,
    pub layers: Option<Vec<DeploymentLayer>>,
    pub orchestration_ring: Option<BiomeReference>,
    pub physics_layer: Option<BiomeReference>,
    pub compute_layers: Option<Vec<BiomeReference>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentLayer {
    pub name: String,
    pub instances: u32,
    pub template: String,
    pub parent_layer: Option<String>,
    pub dependencies: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterativeDeployment {
    pub pattern: TopologyPattern,
    pub instances: u32,
    pub configuration: HashMap<String, serde_json::Value>,
    pub iteration: IterationConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IterationConfig {
    pub variables: HashMap<String, String>,
    pub dependencies: Option<Vec<String>>,
    pub constraints: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecursiveMonitoring {
    pub recursive: bool,
    pub aggregation: String,
    pub metrics: Vec<LayerMetrics>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerMetrics {
    pub layer: String,
    pub collect: Vec<String>,
    pub thresholds: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecursiveScaling {
    pub triggers: Vec<ScalingTrigger>,
    pub constraints: Option<ScalingConstraints>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingTrigger {
    pub metric: String,
    pub threshold: String,
    pub action: ScalingAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingAction {
    pub scale_up: Option<ScalingTarget>,
    pub scale_down: Option<ScalingTarget>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingTarget {
    pub component: String,
    pub instances: Option<String>,
    pub resources: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScalingConstraints {
    pub max_instances: Option<u32>,
    pub min_instances: Option<u32>,
    pub max_resources: Option<HashMap<String, String>>,
    pub cooldown_period: Option<String>,
}

// Sharing and template types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharingConfig {
    #[serde(default)]
    pub public: bool,
    pub license: Option<String>,
    pub repository: Option<String>,
    pub registry: Option<String>,
    #[serde(default)]
    pub fork_permissions: ForkPermissions,
    #[serde(default)]
    pub attribution_required: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ForkPermissions {
    Public,
    Authenticated,
    Restricted(Vec<String>),
    None,
}

impl Default for ForkPermissions {
    fn default() -> Self {
        ForkPermissions::Public
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NicheClassification {
    pub primary: String,
    #[serde(default)]
    pub secondary: Vec<String>,
    #[serde(default)]
    pub custom: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateMetadata {
    #[serde(default)]
    pub is_template: bool,
    pub category: Option<String>,
    pub difficulty: Option<TemplateDifficulty>,
    #[serde(default)]
    pub parameters: Vec<TemplateParameter>,
    #[serde(default)]
    pub examples: Vec<TemplateExample>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TemplateDifficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateParameter {
    pub name: String,
    pub description: String,
    pub param_type: ParameterType,
    #[serde(default)]
    pub required: bool,
    pub default: Option<serde_json::Value>,
    pub validation: Option<ParameterValidation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Array,
    Object,
    Choice(Vec<String>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParameterValidation {
    pub min: Option<serde_json::Value>,
    pub max: Option<serde_json::Value>,
    pub pattern: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TemplateExample {
    pub name: String,
    pub description: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Validation results structure
#[derive(Debug, Clone)]
pub struct ValidationResults {
    /// Validation errors
    pub errors: Vec<String>,
    /// Validation warnings
    pub warnings: Vec<String>,
}

impl ValidationResults {
    pub fn new() -> Self {
        Self {
            errors: Vec::new(),
            warnings: Vec::new(),
        }
    }

    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn add_error(&mut self, error: String) {
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }
}

impl Default for ValidationResults {
    fn default() -> Self {
        Self::new()
    }
}
