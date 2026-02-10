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
/// ```ignore
/// use biomeos_types::CapabilityTaxonomy;
///
/// // Discover by capability, not name
/// let security_providers = registry.find_by_capability(CapabilityTaxonomy::Encryption);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityTaxonomy {
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

    /// Distributed beacon mesh relay
    /// Typical provider: Songbird
    /// Provides: mesh.status, mesh.find_path, mesh.announce, mesh.peers
    MeshRelay,

    /// UDP hole punching for symmetric NAT traversal
    /// Typical provider: Songbird
    /// Provides: punch.request, punch.status
    HolePunch,

    /// STUN protocol for NAT type detection
    /// Typical provider: Songbird
    /// Provides: stun.discover, stun.detect_nat_type
    StunClient,

    /// Sovereign onion service (lightweight .onion address generation)
    /// Typical provider: Songbird (with BearDog crypto)
    /// Provides: onion.create_service, onion.get_address, onion.connect
    OnionService,

    /// Relay server for TURN-like fallback
    /// Typical provider: Songbird
    /// Provides: relay.serve, relay.allocate
    RelayServer,

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

impl CapabilityTaxonomy {
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
            // NAT traversal and mesh
            Self::MeshRelay => Cow::Borrowed("Distributed beacon mesh relay for NAT traversal"),
            Self::HolePunch => Cow::Borrowed("UDP hole punching for symmetric NAT"),
            Self::StunClient => Cow::Borrowed("STUN protocol for NAT type detection"),
            Self::OnionService => Cow::Borrowed("Sovereign onion service for .onion addresses"),
            Self::RelayServer => Cow::Borrowed("TURN-like relay server for fallback connectivity"),

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
            Self::Encryption
            | Self::Identity
            | Self::Trust
            | Self::KeyManagement
            | Self::HardwareSecurity
            | Self::SecureTunneling => CapabilityCategory::Security,

            Self::Discovery
            | Self::P2PFederation
            | Self::Tunneling
            | Self::Routing
            | Self::GeneticRouting
            | Self::CapabilityAnnouncement
            | Self::MeshRelay
            | Self::HolePunch
            | Self::StunClient
            | Self::OnionService
            | Self::RelayServer => CapabilityCategory::Communication,

            Self::WorkloadExecution
            | Self::ResourceScheduling
            | Self::ProcessIsolation
            | Self::FractalCompute
            | Self::GpuAcceleration => CapabilityCategory::Compute,

            Self::DataStorage
            | Self::Provenance
            | Self::Compression
            | Self::Replication
            | Self::Deduplication
            | Self::ContentAddressed => CapabilityCategory::Storage,

            Self::VisualRendering
            | Self::InputHandling
            | Self::MultiModal
            | Self::TopologyVisualization
            | Self::RealtimeUpdates => CapabilityCategory::UserInterface,

            Self::LifecycleManagement
            | Self::HealthMonitoring
            | Self::ConfigManagement
            | Self::MetricsCollection
            | Self::LogAggregation
            | Self::GraphOrchestration => CapabilityCategory::Orchestration,

            Self::AiCoordination
            | Self::AiMultiProvider
            | Self::McpServer
            | Self::AiCapabilityDiscovery => CapabilityCategory::AI,

            Self::BluetoothGenesis
            | Self::SporeDeployment
            | Self::GeneticLineage
            | Self::NicheDeployment
            | Self::Custom(_) => CapabilityCategory::Specialized,
        }
    }

    /// Parse from string (case-insensitive)
    pub fn from_str_flexible(s: &str) -> Option<Self> {
        let s_lower = s.to_lowercase();
        match s_lower.as_str() {
            // Security capabilities - "security" is an alias for encryption
            "encryption" | "security" | "crypto" => Some(Self::Encryption),
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
            "capability_announcement" | "capabilityannouncement" => {
                Some(Self::CapabilityAnnouncement)
            }
            // NAT traversal and mesh capabilities
            "mesh_relay" | "meshrelay" | "mesh" => Some(Self::MeshRelay),
            "hole_punch" | "holepunch" | "punch" => Some(Self::HolePunch),
            "stun_client" | "stunclient" | "stun" => Some(Self::StunClient),
            "onion_service" | "onionservice" | "onion" => Some(Self::OnionService),
            "relay_server" | "relayserver" | "relay" => Some(Self::RelayServer),

            // Compute capabilities - "compute" is an alias for workload execution
            "workload_execution" | "workloadexecution" | "execution" | "compute" => {
                Some(Self::WorkloadExecution)
            }
            "resource_scheduling" | "resourcescheduling" | "scheduling" => {
                Some(Self::ResourceScheduling)
            }
            "process_isolation" | "processisolation" | "isolation" => Some(Self::ProcessIsolation),
            "fractal_compute" | "fractalcompute" => Some(Self::FractalCompute),
            "gpu_acceleration" | "gpuacceleration" | "gpu" => Some(Self::GpuAcceleration),

            // Storage capabilities
            "data_storage" | "datastorage" | "storage" => Some(Self::DataStorage),
            "provenance" => Some(Self::Provenance),
            "compression" => Some(Self::Compression),
            "replication" => Some(Self::Replication),
            "deduplication" | "dedup" => Some(Self::Deduplication),
            "content_addressed" | "contentaddressed" => Some(Self::ContentAddressed),

            "visual_rendering" | "visualrendering" | "rendering" | "visualization" => {
                Some(Self::VisualRendering)
            }
            "input_handling" | "inputhandling" | "input" => Some(Self::InputHandling),
            "multi_modal" | "multimodal" => Some(Self::MultiModal),
            "topology_visualization" | "topologyvisualization" => Some(Self::TopologyVisualization),
            "realtime_updates" | "realtimeupdates" => Some(Self::RealtimeUpdates),

            "lifecycle_management" | "lifecyclemanagement" | "lifecycle" => {
                Some(Self::LifecycleManagement)
            }
            "health_monitoring" | "healthmonitoring" | "health" => Some(Self::HealthMonitoring),
            "config_management" | "configmanagement" | "config" => Some(Self::ConfigManagement),
            "metrics_collection" | "metricscollection" | "metrics" => Some(Self::MetricsCollection),
            "log_aggregation" | "logaggregation" | "logs" => Some(Self::LogAggregation),
            "graph_orchestration" | "graphorchestration" => Some(Self::GraphOrchestration),

            // AI capabilities - "ai" is an alias for coordination
            "ai_coordination" | "aicoordination" | "ai" => Some(Self::AiCoordination),
            "ai_multi_provider" | "aimultiprovider" => Some(Self::AiMultiProvider),
            "mcp_server" | "mcpserver" | "mcp" => Some(Self::McpServer),
            "ai_capability_discovery" | "aicapabilitydiscovery" => {
                Some(Self::AiCapabilityDiscovery)
            }

            "bluetooth_genesis" | "bluetoothgenesis" => Some(Self::BluetoothGenesis),
            "spore_deployment" | "sporedeployment" | "spore" => Some(Self::SporeDeployment),
            "genetic_lineage" | "geneticlineage" | "lineage" => Some(Self::GeneticLineage),
            "niche_deployment" | "nichedeployment" | "niche" => Some(Self::NicheDeployment),

            _ => None,
        }
    }

    /// Get the default primal name that typically provides this capability.
    ///
    /// **MIGRATION NOTE**: This is a fallback for bootstrapping. The target architecture
    /// uses Songbird for runtime capability discovery. This mapping will be deprecated
    /// once all primals register their capabilities with Songbird on startup.
    ///
    /// Set `BIOMEOS_STRICT_DISCOVERY=1` to disable this fallback and require
    /// all capabilities to be discovered via Songbird.
    pub fn default_primal(&self) -> Option<&'static str> {
        // Check if strict discovery is enabled (no fallback)
        if std::env::var("BIOMEOS_STRICT_DISCOVERY").is_ok() {
            return None;
        }

        match self {
            // Security capabilities → BearDog
            Self::Encryption
            | Self::Identity
            | Self::Trust
            | Self::KeyManagement
            | Self::HardwareSecurity
            | Self::SecureTunneling => Some("beardog"),

            // Discovery & Communication → Songbird
            Self::Discovery
            | Self::P2PFederation
            | Self::Tunneling
            | Self::Routing
            | Self::GeneticRouting
            | Self::CapabilityAnnouncement
            | Self::BluetoothGenesis
            | Self::MeshRelay
            | Self::HolePunch
            | Self::StunClient
            | Self::OnionService
            | Self::RelayServer => Some("songbird"),

            // Compute → Toadstool
            Self::WorkloadExecution
            | Self::ResourceScheduling
            | Self::ProcessIsolation
            | Self::FractalCompute
            | Self::GpuAcceleration => Some("toadstool"),

            // Storage → NestGate
            Self::DataStorage
            | Self::Provenance
            | Self::Compression
            | Self::Replication
            | Self::Deduplication
            | Self::ContentAddressed => Some("nestgate"),

            // AI → Squirrel
            Self::AiCoordination
            | Self::AiMultiProvider
            | Self::McpServer
            | Self::AiCapabilityDiscovery => Some("squirrel"),

            // UI → petalTongue (integrated when available via Songbird discovery)
            Self::VisualRendering
            | Self::InputHandling
            | Self::MultiModal
            | Self::TopologyVisualization
            | Self::RealtimeUpdates => None, // petalTongue discovered at runtime

            // Orchestration → biomeOS (self)
            Self::LifecycleManagement
            | Self::HealthMonitoring
            | Self::ConfigManagement
            | Self::MetricsCollection
            | Self::LogAggregation
            | Self::GraphOrchestration
            | Self::SporeDeployment
            | Self::GeneticLineage
            | Self::NicheDeployment => Some("biomeos"),

            // Custom capabilities have no default
            Self::Custom(_) => None,
        }
    }

    /// Resolve a capability string to a primal name using the taxonomy
    ///
    /// This is a convenience function for migrating from hardcoded capability→primal
    /// mappings to taxonomy-based resolution.
    pub fn resolve_to_primal(capability: &str) -> Option<&'static str> {
        Self::from_str_flexible(capability).and_then(|cap| cap.default_primal())
    }

    /// Get known primal names from the capability taxonomy.
    ///
    /// **DEEP DEBT NOTE**: This is a bootstrap-time hint, NOT the source of truth.
    /// In sovereign mode, primals self-register via capability.call() at runtime.
    /// This list is ONLY used during initial bootstrap before capability discovery
    /// is available. Once Songbird is running, use `discovery.query` instead.
    ///
    /// Set `BIOMEOS_STRICT_DISCOVERY=1` to disable this fallback entirely.
    ///
    /// # Returns
    ///
    /// Bootstrap-time primal name hints (empty if strict discovery is enabled)
    pub fn known_primals() -> &'static [&'static str] {
        // In strict discovery mode, return nothing — primals discover at runtime
        if std::env::var("BIOMEOS_STRICT_DISCOVERY").is_ok() {
            return &[];
        }
        // Bootstrap hints only — NOT the canonical list
        // Primals self-register their capabilities at startup
        &["beardog", "songbird", "toadstool", "nestgate", "squirrel"]
    }
}

/// Get capability category names for a primal based on the taxonomy
///
/// This is the reverse of `CapabilityTaxonomy::default_primal()`.
/// Returns the high-level capability categories that a primal provides,
/// derived from the taxonomy rather than hardcoded per-callsite.
///
/// **DEEP DEBT NOTE**: This is a bootstrap-time hint. In production,
/// primals should self-report capabilities via `discover_capabilities`.
pub fn capabilities_for_primal(primal_name: &str) -> Vec<String> {
    match primal_name {
        "beardog" => vec!["crypto".to_string(), "security".to_string()],
        "songbird" => vec!["discovery".to_string(), "network".to_string()],
        "toadstool" => vec!["compute".to_string()],
        "nestgate" => vec!["storage".to_string()],
        "squirrel" => vec!["ai".to_string()],
        "biomeos" => vec!["orchestration".to_string()],
        // Unknown primals contribute their name as a capability domain
        // (runtime discovery will replace this with actual capabilities)
        other => vec![other.to_string()],
    }
}

impl fmt::Display for CapabilityTaxonomy {
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
    /// Security and cryptography capabilities
    Security,
    /// Communication and networking capabilities
    Communication,
    /// Compute and execution capabilities
    Compute,
    /// Storage and data capabilities
    Storage,
    /// User interface and rendering capabilities
    UserInterface,
    /// Orchestration and management capabilities
    Orchestration,
    /// AI and intelligence capabilities
    AI,
    /// Specialized and custom capabilities
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
        assert_eq!(
            CapabilityTaxonomy::Encryption.description(),
            "Encrypt and decrypt data"
        );
        assert_eq!(
            CapabilityTaxonomy::Discovery.description(),
            "Discover other primals and nodes"
        );
    }

    #[test]
    fn test_capability_category() {
        assert_eq!(
            CapabilityTaxonomy::Encryption.category(),
            CapabilityCategory::Security
        );
        assert_eq!(
            CapabilityTaxonomy::Discovery.category(),
            CapabilityCategory::Communication
        );
        assert_eq!(
            CapabilityTaxonomy::WorkloadExecution.category(),
            CapabilityCategory::Compute
        );
    }

    #[test]
    fn test_capability_from_str() {
        assert_eq!(
            CapabilityTaxonomy::from_str_flexible("encryption"),
            Some(CapabilityTaxonomy::Encryption)
        );
        assert_eq!(
            CapabilityTaxonomy::from_str_flexible("DISCOVERY"),
            Some(CapabilityTaxonomy::Discovery)
        );
        assert_eq!(
            CapabilityTaxonomy::from_str_flexible("federation"),
            Some(CapabilityTaxonomy::P2PFederation)
        );
        assert_eq!(CapabilityTaxonomy::from_str_flexible("unknown"), None);
    }

    #[test]
    fn test_capability_display() {
        assert_eq!(format!("{}", CapabilityTaxonomy::Encryption), "encryption");
        assert_eq!(
            format!("{}", CapabilityTaxonomy::Custom("my-cap".to_string())),
            "custom:my-cap"
        );
    }

    #[test]
    fn test_capability_serde() {
        let cap = CapabilityTaxonomy::Encryption;
        let json = serde_json::to_string(&cap).unwrap();
        let deserialized: CapabilityTaxonomy = serde_json::from_str(&json).unwrap();
        assert_eq!(cap, deserialized);
    }

    #[test]
    fn test_known_primals() {
        let primals = CapabilityTaxonomy::known_primals();

        // Should return the canonical list of primals
        assert!(!primals.is_empty(), "Should have known primals");
        assert_eq!(primals.len(), 5, "Should have exactly 5 primals");

        // Verify expected primals are present
        assert!(primals.contains(&"beardog"), "Should include beardog");
        assert!(primals.contains(&"songbird"), "Should include songbird");
        assert!(primals.contains(&"toadstool"), "Should include toadstool");
        assert!(primals.contains(&"nestgate"), "Should include nestgate");
        assert!(primals.contains(&"squirrel"), "Should include squirrel");
    }

    #[test]
    fn test_resolve_to_primal() {
        // Test direct capability resolution
        assert_eq!(
            CapabilityTaxonomy::resolve_to_primal("encryption"),
            Some("beardog")
        );
        assert_eq!(
            CapabilityTaxonomy::resolve_to_primal("discovery"),
            Some("songbird")
        );
        assert_eq!(
            CapabilityTaxonomy::resolve_to_primal("storage"),
            Some("nestgate")
        );
        assert_eq!(
            CapabilityTaxonomy::resolve_to_primal("compute"),
            Some("toadstool")
        );
        assert_eq!(
            CapabilityTaxonomy::resolve_to_primal("ai"),
            Some("squirrel")
        );

        // Unknown capabilities should return None
        assert_eq!(CapabilityTaxonomy::resolve_to_primal("unknown"), None);
    }

    #[test]
    fn test_default_primal_security() {
        // Security capabilities should map to beardog
        assert_eq!(
            CapabilityTaxonomy::Encryption.default_primal(),
            Some("beardog")
        );
        assert_eq!(
            CapabilityTaxonomy::Identity.default_primal(),
            Some("beardog")
        );
        assert_eq!(CapabilityTaxonomy::Trust.default_primal(), Some("beardog"));
        assert_eq!(
            CapabilityTaxonomy::KeyManagement.default_primal(),
            Some("beardog")
        );
    }

    #[test]
    fn test_default_primal_communication() {
        // Communication capabilities should map to songbird
        assert_eq!(
            CapabilityTaxonomy::Discovery.default_primal(),
            Some("songbird")
        );
        assert_eq!(
            CapabilityTaxonomy::P2PFederation.default_primal(),
            Some("songbird")
        );
        assert_eq!(
            CapabilityTaxonomy::Tunneling.default_primal(),
            Some("songbird")
        );
    }

    #[test]
    fn test_default_primal_compute() {
        // Compute capabilities should map to toadstool
        assert_eq!(
            CapabilityTaxonomy::WorkloadExecution.default_primal(),
            Some("toadstool")
        );
        assert_eq!(
            CapabilityTaxonomy::ResourceScheduling.default_primal(),
            Some("toadstool")
        );
    }

    #[test]
    fn test_default_primal_storage() {
        // Storage capabilities should map to nestgate
        assert_eq!(
            CapabilityTaxonomy::DataStorage.default_primal(),
            Some("nestgate")
        );
        assert_eq!(
            CapabilityTaxonomy::ContentAddressed.default_primal(),
            Some("nestgate")
        );
    }

    #[test]
    fn test_default_primal_ai() {
        // AI capabilities should map to squirrel
        assert_eq!(
            CapabilityTaxonomy::AiCoordination.default_primal(),
            Some("squirrel")
        );
        assert_eq!(
            CapabilityTaxonomy::McpServer.default_primal(),
            Some("squirrel")
        );
    }

    #[test]
    fn test_custom_capability_no_default() {
        // Custom capabilities should not have a default primal
        assert_eq!(
            CapabilityTaxonomy::Custom("custom-cap".to_string()).default_primal(),
            None
        );
    }

    #[test]
    fn test_nat_traversal_capabilities() {
        // NAT traversal capabilities should map to songbird
        assert_eq!(
            CapabilityTaxonomy::MeshRelay.default_primal(),
            Some("songbird")
        );
        assert_eq!(
            CapabilityTaxonomy::HolePunch.default_primal(),
            Some("songbird")
        );
        assert_eq!(
            CapabilityTaxonomy::StunClient.default_primal(),
            Some("songbird")
        );
        assert_eq!(
            CapabilityTaxonomy::OnionService.default_primal(),
            Some("songbird")
        );
        assert_eq!(
            CapabilityTaxonomy::RelayServer.default_primal(),
            Some("songbird")
        );

        // All should be in Communication category
        assert_eq!(
            CapabilityTaxonomy::MeshRelay.category(),
            CapabilityCategory::Communication
        );
        assert_eq!(
            CapabilityTaxonomy::HolePunch.category(),
            CapabilityCategory::Communication
        );
        assert_eq!(
            CapabilityTaxonomy::StunClient.category(),
            CapabilityCategory::Communication
        );
        assert_eq!(
            CapabilityTaxonomy::OnionService.category(),
            CapabilityCategory::Communication
        );
        assert_eq!(
            CapabilityTaxonomy::RelayServer.category(),
            CapabilityCategory::Communication
        );
    }

    #[test]
    fn test_nat_traversal_from_str() {
        // Test string parsing for NAT traversal capabilities
        assert_eq!(
            CapabilityTaxonomy::from_str_flexible("mesh"),
            Some(CapabilityTaxonomy::MeshRelay)
        );
        assert_eq!(
            CapabilityTaxonomy::from_str_flexible("mesh_relay"),
            Some(CapabilityTaxonomy::MeshRelay)
        );
        assert_eq!(
            CapabilityTaxonomy::from_str_flexible("punch"),
            Some(CapabilityTaxonomy::HolePunch)
        );
        assert_eq!(
            CapabilityTaxonomy::from_str_flexible("hole_punch"),
            Some(CapabilityTaxonomy::HolePunch)
        );
        assert_eq!(
            CapabilityTaxonomy::from_str_flexible("stun"),
            Some(CapabilityTaxonomy::StunClient)
        );
        assert_eq!(
            CapabilityTaxonomy::from_str_flexible("onion"),
            Some(CapabilityTaxonomy::OnionService)
        );
        assert_eq!(
            CapabilityTaxonomy::from_str_flexible("relay"),
            Some(CapabilityTaxonomy::RelayServer)
        );
    }

    #[test]
    fn test_resolve_nat_traversal_to_primal() {
        // Test resolve_to_primal for NAT traversal capabilities
        assert_eq!(
            CapabilityTaxonomy::resolve_to_primal("mesh"),
            Some("songbird")
        );
        assert_eq!(
            CapabilityTaxonomy::resolve_to_primal("punch"),
            Some("songbird")
        );
        assert_eq!(
            CapabilityTaxonomy::resolve_to_primal("stun"),
            Some("songbird")
        );
        assert_eq!(
            CapabilityTaxonomy::resolve_to_primal("onion"),
            Some("songbird")
        );
        assert_eq!(
            CapabilityTaxonomy::resolve_to_primal("relay"),
            Some("songbird")
        );
    }
}
