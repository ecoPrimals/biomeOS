// =============================================================================
// Primal Capability Taxonomy - Deep Debt Evolution
// =============================================================================
//
// Defines well-known capabilities that primals can provide.
// This enables capability-based discovery instead of hardcoded primal names.
//
// Deep Debt Principle:
// "Primal code only has self knowledge and discovers other primals in runtime"
//
// BEFORE:
//   if primal_name == "beardog" { ... }  // ❌ Hardcoded
//
// AFTER:
//   if primal.has_capability(PrimalCapability::Encryption) { ... }  // ✅ Capability-based
//
// =============================================================================

use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::fmt;

/// Well-known capabilities that primals can provide
///
/// This taxonomy allows primals to be discovered by what they can do,
/// not by their name. This is fundamental to the primal philosophy:
/// - No hardcoded primal names
/// - Runtime discovery
/// - Composable architectures
/// - Evolvable systems
///
/// # Examples
///
/// ```
/// use biomeos_types::capabilities::PrimalCapability;
///
/// // Discover by capability, not name
/// let security_providers = registry.find_by_capability(PrimalCapability::Encryption);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PrimalCapability {
    // =============================================================================
    // Security & Cryptography
    // =============================================================================
    
    /// Encrypt/decrypt data
    /// Typical provider: BearDog
    Encryption,
    
    /// Cryptographic identity verification
    /// Typical provider: BearDog
    Identity,
    
    /// Trust evaluation (family membership, lineage)
    /// Typical provider: BearDog
    Trust,
    
    /// Key generation and management
    /// Typical provider: BearDog
    KeyManagement,
    
    /// HSM operations (hardware security module)
    /// Typical provider: BearDog
    HardwareSecurity,
    
    /// Secure tunneling (BTSP)
    /// Typical provider: BearDog, Songbird
    SecureTunneling,
    
    // =============================================================================
    // Communication & Networking
    // =============================================================================
    
    /// Discover other primals/nodes (UDP multicast, mDNS, etc.)
    /// Typical provider: Songbird
    Discovery,
    
    /// Peer-to-peer federation
    /// Typical provider: Songbird
    P2PFederation,
    
    /// Network tunneling and routing
    /// Typical provider: Songbird
    Tunneling,
    
    /// Packet routing and forwarding
    /// Typical provider: Songbird
    Routing,
    
    /// BirdSong genetic lineage NAT
    /// Typical provider: Songbird
    GeneticRouting,
    
    /// Announce capabilities to network
    /// Typical provider: Songbird
    CapabilityAnnouncement,
    
    // =============================================================================
    // Compute & Execution
    // =============================================================================
    
    /// Execute workloads (containers, processes, VMs)
    /// Typical provider: Toadstool
    WorkloadExecution,
    
    /// Schedule resources across compute nodes
    /// Typical provider: Toadstool
    ResourceScheduling,
    
    /// Isolate processes/containers
    /// Typical provider: Toadstool
    ProcessIsolation,
    
    /// Fractal compute scaling
    /// Typical provider: Toadstool
    FractalCompute,
    
    /// GPU acceleration
    /// Typical provider: Toadstool
    GpuAcceleration,
    
    // =============================================================================
    // Storage & Data
    // =============================================================================
    
    /// Store and retrieve data
    /// Typical provider: NestGate
    DataStorage,
    
    /// Track data provenance and lineage
    /// Typical provider: NestGate
    Provenance,
    
    /// Compress data adaptively
    /// Typical provider: NestGate
    Compression,
    
    /// Replicate data across nodes
    /// Typical provider: NestGate
    Replication,
    
    /// Deduplication
    /// Typical provider: NestGate
    Deduplication,
    
    /// Content-addressed storage
    /// Typical provider: NestGate
    ContentAddressed,
    
    // =============================================================================
    // User Interface & Rendering
    // =============================================================================
    
    /// Render visual interfaces
    /// Typical provider: petalTongue
    VisualRendering,
    
    /// Handle user input
    /// Typical provider: petalTongue
    InputHandling,
    
    /// Multi-modal interface (visual, audio, haptic)
    /// Typical provider: petalTongue
    MultiModal,
    
    /// Topology visualization
    /// Typical provider: petalTongue
    TopologyVisualization,
    
    /// Real-time updates
    /// Typical provider: petalTongue
    RealtimeUpdates,
    
    // =============================================================================
    // Orchestration & Management
    // =============================================================================
    
    /// Manage primal lifecycle (start, stop, restart)
    /// Typical provider: biomeOS
    LifecycleManagement,
    
    /// Health monitoring and checks
    /// Typical provider: biomeOS
    HealthMonitoring,
    
    /// Configuration management
    /// Typical provider: biomeOS
    ConfigManagement,
    
    /// Metrics collection
    /// Typical provider: biomeOS
    MetricsCollection,
    
    /// Log aggregation
    /// Typical provider: biomeOS
    LogAggregation,
    
    /// Graph-based orchestration
    /// Typical provider: biomeOS
    GraphOrchestration,
    
    // =============================================================================
    // AI & Intelligence
    // =============================================================================
    
    /// AI coordination and routing
    /// Typical provider: Squirrel
    AiCoordination,
    
    /// Multi-provider AI support
    /// Typical provider: Squirrel
    AiMultiProvider,
    
    /// MCP (Model Context Protocol) server
    /// Typical provider: Squirrel
    McpServer,
    
    /// Tool/capability discovery for AI
    /// Typical provider: Squirrel
    AiCapabilityDiscovery,
    
    // =============================================================================
    // Specialized
    // =============================================================================
    
    /// Bluetooth genesis for initial device pairing
    /// Typical provider: Songbird (genesis module)
    BluetoothGenesis,
    
    /// USB spore creation and deployment
    /// Typical provider: biomeOS (spore module)
    SporeDeployment,
    
    /// Genetic lineage management
    /// Typical provider: biomeOS, BearDog
    GeneticLineage,
    
    /// Niche (biome) deployment
    /// Typical provider: biomeOS
    NicheDeployment,
    
    /// Custom capability (use sparingly!)
    /// For capabilities not yet in the taxonomy
    Custom(String),
}

impl PrimalCapability {
    /// Get a human-readable description of this capability
    ///
    /// Returns `Cow<'static, str>` to avoid memory leaks for custom capabilities
    /// while maintaining zero-copy for static descriptions.
    pub fn description(&self) -> Cow<'static, str> {
        match self {
            // Security
            Self::Encryption => Cow::Borrowed("Encrypt and decrypt data"),
            Self::Identity => Cow::Borrowed("Verify cryptographic identities"),
            Self::Trust => Cow::Borrowed("Evaluate trust relationships"),
            Self::KeyManagement => Cow::Borrowed("Generate and manage cryptographic keys"),
            Self::HardwareSecurity => Cow::Borrowed("Hardware security module operations"),
            Self::SecureTunneling => Cow::Borrowed("Create secure encrypted tunnels"),

            // Communication
            Self::Discovery => Cow::Borrowed("Discover other primals and nodes"),
            Self::P2PFederation => Cow::Borrowed("Peer-to-peer federation"),
            Self::Tunneling => Cow::Borrowed("Network tunneling and routing"),
            Self::Routing => Cow::Borrowed("Packet routing and forwarding"),
            Self::GeneticRouting => Cow::Borrowed("BirdSong genetic lineage NAT"),
            Self::CapabilityAnnouncement => Cow::Borrowed("Announce capabilities to network"),

            // Compute
            Self::WorkloadExecution => Cow::Borrowed("Execute workloads"),
            Self::ResourceScheduling => Cow::Borrowed("Schedule compute resources"),
            Self::ProcessIsolation => Cow::Borrowed("Isolate processes and containers"),
            Self::FractalCompute => Cow::Borrowed("Fractal compute scaling"),
            Self::GpuAcceleration => Cow::Borrowed("GPU acceleration"),

            // Storage
            Self::DataStorage => Cow::Borrowed("Store and retrieve data"),
            Self::Provenance => Cow::Borrowed("Track data provenance"),
            Self::Compression => Cow::Borrowed("Adaptive data compression"),
            Self::Replication => Cow::Borrowed("Data replication"),
            Self::Deduplication => Cow::Borrowed("Data deduplication"),
            Self::ContentAddressed => Cow::Borrowed("Content-addressed storage"),

            // UI
            Self::VisualRendering => Cow::Borrowed("Render visual interfaces"),
            Self::InputHandling => Cow::Borrowed("Handle user input"),
            Self::MultiModal => Cow::Borrowed("Multi-modal interface"),
            Self::TopologyVisualization => Cow::Borrowed("Visualize network topology"),
            Self::RealtimeUpdates => Cow::Borrowed("Real-time UI updates"),

            // Orchestration
            Self::LifecycleManagement => Cow::Borrowed("Primal lifecycle management"),
            Self::HealthMonitoring => Cow::Borrowed("Health monitoring"),
            Self::ConfigManagement => Cow::Borrowed("Configuration management"),
            Self::MetricsCollection => Cow::Borrowed("Metrics collection"),
            Self::LogAggregation => Cow::Borrowed("Log aggregation"),
            Self::GraphOrchestration => Cow::Borrowed("Graph-based orchestration"),

            // AI
            Self::AiCoordination => Cow::Borrowed("AI coordination and routing"),
            Self::AiMultiProvider => Cow::Borrowed("Multi-provider AI support"),
            Self::McpServer => Cow::Borrowed("MCP server"),
            Self::AiCapabilityDiscovery => Cow::Borrowed("AI capability discovery"),

            // Specialized
            Self::BluetoothGenesis => Cow::Borrowed("Bluetooth genesis pairing"),
            Self::SporeDeployment => Cow::Borrowed("USB spore deployment"),
            Self::GeneticLineage => Cow::Borrowed("Genetic lineage management"),
            Self::NicheDeployment => Cow::Borrowed("Niche deployment"),

            // Custom capabilities use Owned to avoid memory leaks
            Self::Custom(name) => Cow::Owned(format!("Custom: {}", name)),
        }
    }
    
    /// Get the capability category
    pub fn category(&self) -> CapabilityCategory {
        match self {
            Self::Encryption | Self::Identity | Self::Trust | Self::KeyManagement 
            | Self::HardwareSecurity | Self::SecureTunneling => CapabilityCategory::Security,
            
            Self::Discovery | Self::P2PFederation | Self::Tunneling | Self::Routing 
            | Self::GeneticRouting | Self::CapabilityAnnouncement => CapabilityCategory::Communication,
            
            Self::WorkloadExecution | Self::ResourceScheduling | Self::ProcessIsolation 
            | Self::FractalCompute | Self::GpuAcceleration => CapabilityCategory::Compute,
            
            Self::DataStorage | Self::Provenance | Self::Compression | Self::Replication 
            | Self::Deduplication | Self::ContentAddressed => CapabilityCategory::Storage,
            
            Self::VisualRendering | Self::InputHandling | Self::MultiModal 
            | Self::TopologyVisualization | Self::RealtimeUpdates => CapabilityCategory::UserInterface,
            
            Self::LifecycleManagement | Self::HealthMonitoring | Self::ConfigManagement 
            | Self::MetricsCollection | Self::LogAggregation | Self::GraphOrchestration => CapabilityCategory::Orchestration,
            
            Self::AiCoordination | Self::AiMultiProvider | Self::McpServer 
            | Self::AiCapabilityDiscovery => CapabilityCategory::AI,
            
            Self::BluetoothGenesis | Self::SporeDeployment | Self::GeneticLineage 
            | Self::NicheDeployment | Self::Custom(_) => CapabilityCategory::Specialized,
        }
    }
    
    /// Parse from string (case-insensitive)
    pub fn from_str_flexible(s: &str) -> Option<Self> {
        let s_lower = s.to_lowercase();
        match s_lower.as_str() {
            "encryption" => Some(Self::Encryption),
            "identity" => Some(Self::Identity),
            "trust" => Some(Self::Trust),
            "key_management" | "keymanagement" => Some(Self::KeyManagement),
            "hardware_security" | "hardwaresecurity" | "hsm" => Some(Self::HardwareSecurity),
            "secure_tunneling" | "securetunneling" => Some(Self::SecureTunneling),
            
            "discovery" => Some(Self::Discovery),
            "p2p_federation" | "p2pfederation" | "federation" => Some(Self::P2PFederation),
            "tunneling" => Some(Self::Tunneling),
            "routing" => Some(Self::Routing),
            "genetic_routing" | "geneticrouting" => Some(Self::GeneticRouting),
            "capability_announcement" | "capabilityannouncement" => Some(Self::CapabilityAnnouncement),
            
            "workload_execution" | "workloadexecution" | "execution" => Some(Self::WorkloadExecution),
            "resource_scheduling" | "resourcescheduling" | "scheduling" => Some(Self::ResourceScheduling),
            "process_isolation" | "processisolation" | "isolation" => Some(Self::ProcessIsolation),
            "fractal_compute" | "fractalcompute" => Some(Self::FractalCompute),
            "gpu_acceleration" | "gpuacceleration" | "gpu" => Some(Self::GpuAcceleration),
            
            "data_storage" | "datastorage" | "storage" => Some(Self::DataStorage),
            "provenance" => Some(Self::Provenance),
            "compression" => Some(Self::Compression),
            "replication" => Some(Self::Replication),
            "deduplication" | "dedup" => Some(Self::Deduplication),
            "content_addressed" | "contentaddressed" => Some(Self::ContentAddressed),
            
            "visual_rendering" | "visualrendering" | "rendering" => Some(Self::VisualRendering),
            "input_handling" | "inputhandling" | "input" => Some(Self::InputHandling),
            "multi_modal" | "multimodal" => Some(Self::MultiModal),
            "topology_visualization" | "topologyvisualization" => Some(Self::TopologyVisualization),
            "realtime_updates" | "realtimeupdates" => Some(Self::RealtimeUpdates),
            
            "lifecycle_management" | "lifecyclemanagement" | "lifecycle" => Some(Self::LifecycleManagement),
            "health_monitoring" | "healthmonitoring" | "health" => Some(Self::HealthMonitoring),
            "config_management" | "configmanagement" | "config" => Some(Self::ConfigManagement),
            "metrics_collection" | "metricscollection" | "metrics" => Some(Self::MetricsCollection),
            "log_aggregation" | "logaggregation" | "logs" => Some(Self::LogAggregation),
            "graph_orchestration" | "graphorchestration" => Some(Self::GraphOrchestration),
            
            "ai_coordination" | "aicoordination" => Some(Self::AiCoordination),
            "ai_multi_provider" | "aimultiprovider" => Some(Self::AiMultiProvider),
            "mcp_server" | "mcpserver" | "mcp" => Some(Self::McpServer),
            "ai_capability_discovery" | "aicapabilitydiscovery" => Some(Self::AiCapabilityDiscovery),
            
            "bluetooth_genesis" | "bluetoothgenesis" => Some(Self::BluetoothGenesis),
            "spore_deployment" | "sporedeployment" | "spore" => Some(Self::SporeDeployment),
            "genetic_lineage" | "geneticlineage" | "lineage" => Some(Self::GeneticLineage),
            "niche_deployment" | "nichedeployment" | "niche" => Some(Self::NicheDeployment),
            
            _ => None,
        }
    }
}

impl fmt::Display for PrimalCapability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Custom(name) => write!(f, "custom:{}", name),
            _ => {
                let s = format!("{:?}", self);
                write!(f, "{}", s.to_lowercase())
            }
        }
    }
}

/// Capability category for grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityCategory {
    Security,
    Communication,
    Compute,
    Storage,
    UserInterface,
    Orchestration,
    AI,
    Specialized,
}

impl fmt::Display for CapabilityCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = format!("{:?}", self);
        write!(f, "{}", s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_description() {
        assert_eq!(PrimalCapability::Encryption.description(), "Encrypt and decrypt data");
        assert_eq!(PrimalCapability::Discovery.description(), "Discover other primals and nodes");
    }

    #[test]
    fn test_capability_category() {
        assert_eq!(PrimalCapability::Encryption.category(), CapabilityCategory::Security);
        assert_eq!(PrimalCapability::Discovery.category(), CapabilityCategory::Communication);
        assert_eq!(PrimalCapability::WorkloadExecution.category(), CapabilityCategory::Compute);
    }

    #[test]
    fn test_capability_from_str() {
        assert_eq!(PrimalCapability::from_str_flexible("encryption"), Some(PrimalCapability::Encryption));
        assert_eq!(PrimalCapability::from_str_flexible("DISCOVERY"), Some(PrimalCapability::Discovery));
        assert_eq!(PrimalCapability::from_str_flexible("federation"), Some(PrimalCapability::P2PFederation));
        assert_eq!(PrimalCapability::from_str_flexible("unknown"), None);
    }

    #[test]
    fn test_capability_display() {
        assert_eq!(format!("{}", PrimalCapability::Encryption), "encryption");
        assert_eq!(format!("{}", PrimalCapability::Custom("my-cap".to_string())), "custom:my-cap");
    }

    #[test]
    fn test_capability_serde() {
        let cap = PrimalCapability::Encryption;
        let json = serde_json::to_string(&cap).unwrap();
        let deserialized: PrimalCapability = serde_json::from_str(&json).unwrap();
        assert_eq!(cap, deserialized);
    }
}

