// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

use super::category::CapabilityCategory;
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

    /// Stereoscopic 3D rendering (VR/AR)
    /// Typical provider: petalTongue (behind VisualOutputCapability::ThreeD)
    StereoRendering,

    /// Motion capture and 6DoF tracking (head, hand, tool)
    /// Typical provider: petalTongue (MotionCaptureAdapter)
    MotionTracking,

    /// Haptic feedback output (force feedback, vibration)
    /// Typical provider: petalTongue
    HapticFeedback,

    // =============================================================================
    // Medical / Surgical Domain
    // =============================================================================
    /// Biosignal processing (ECG, PPG, EDA)
    /// Typical provider: healthSpring
    BiosignalProcessing,

    /// Pharmacokinetics / Pharmacodynamics modeling
    /// Typical provider: healthSpring
    PharmacokineticModeling,

    /// Surgical tool simulation and tracking
    /// Typical provider: healthSpring + petalTongue
    SurgicalToolSimulation,

    /// Tissue physics and deformation modeling
    /// Typical provider: healthSpring + barraCuda
    TissuePhysics,

    /// Anatomy model rendering and interaction
    /// Typical provider: healthSpring + petalTongue
    AnatomyModeling,

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
    #[must_use]
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
            Self::StereoRendering => Cow::Borrowed("Stereoscopic 3D rendering for VR/AR"),
            Self::MotionTracking => Cow::Borrowed("Motion capture and 6DoF tracking"),
            Self::HapticFeedback => Cow::Borrowed("Haptic feedback output"),

            // Medical / Surgical
            Self::BiosignalProcessing => Cow::Borrowed("Biosignal processing (ECG, PPG, EDA)"),
            Self::PharmacokineticModeling => {
                Cow::Borrowed("Pharmacokinetic/pharmacodynamic modeling")
            }
            Self::SurgicalToolSimulation => Cow::Borrowed("Surgical tool simulation and tracking"),
            Self::TissuePhysics => Cow::Borrowed("Tissue physics and deformation modeling"),
            Self::AnatomyModeling => Cow::Borrowed("Anatomy model rendering and interaction"),

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
            Self::Custom(name) => Cow::Owned(format!("Custom: {name}")),
        }
    }

    /// Get the capability category
    #[must_use]
    pub const fn category(&self) -> CapabilityCategory {
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
            | Self::RealtimeUpdates
            | Self::StereoRendering
            | Self::MotionTracking
            | Self::HapticFeedback => CapabilityCategory::UserInterface,

            Self::BiosignalProcessing
            | Self::PharmacokineticModeling
            | Self::SurgicalToolSimulation
            | Self::TissuePhysics
            | Self::AnatomyModeling
            | Self::BluetoothGenesis
            | Self::SporeDeployment
            | Self::GeneticLineage
            | Self::NicheDeployment
            | Self::Custom(_) => CapabilityCategory::Specialized,

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
        }
    }

    /// Parse from string (case-insensitive)
    #[must_use]
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

            "discovery" | "network" | "http_bridge" => Some(Self::Discovery),
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

            "visual_rendering" | "visualrendering" | "rendering" | "visualization" | "ui" => {
                Some(Self::VisualRendering)
            }
            "input_handling" | "inputhandling" | "input" => Some(Self::InputHandling),
            "multi_modal" | "multimodal" => Some(Self::MultiModal),
            "topology_visualization" | "topologyvisualization" => Some(Self::TopologyVisualization),
            "realtime_updates" | "realtimeupdates" => Some(Self::RealtimeUpdates),
            "stereo_rendering" | "stereorendering" | "stereo" | "vr_rendering" => {
                Some(Self::StereoRendering)
            }
            "motion_tracking" | "motiontracking" | "mocap" | "tracking" => {
                Some(Self::MotionTracking)
            }
            "haptic_feedback" | "hapticfeedback" | "haptic" | "haptics" => {
                Some(Self::HapticFeedback)
            }
            "biosignal_processing" | "biosignalprocessing" | "biosignal" => {
                Some(Self::BiosignalProcessing)
            }
            "pharmacokinetic_modeling"
            | "pharmacokineticmodeling"
            | "pharmacokinetics"
            | "pkpd" => Some(Self::PharmacokineticModeling),
            "surgical_tool_simulation" | "surgicaltoolsimulation" | "surgical" => {
                Some(Self::SurgicalToolSimulation)
            }
            "tissue_physics" | "tissuephysics" | "tissue" => Some(Self::TissuePhysics),
            "anatomy_modeling" | "anatomymodeling" | "anatomy" => Some(Self::AnatomyModeling),

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
            "genetic_lineage" | "geneticlineage" | "lineage" | "genetic" => {
                Some(Self::GeneticLineage)
            }
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
    #[must_use]
    pub fn default_primal(&self) -> Option<&'static str> {
        self.default_primal_with(std::env::var("BIOMEOS_STRICT_DISCOVERY").is_ok())
    }

    /// Same as `default_primal()` but with explicit strict flag (no env var).
    #[must_use]
    pub const fn default_primal_with(&self, strict: bool) -> Option<&'static str> {
        if strict {
            return None;
        }

        match self {
            // Security capabilities → BearDog (including genetic lineage: BearDog
            // owns HKDF-SHA256 derivation, lineage proofs, and sibling verification)
            Self::Encryption
            | Self::Identity
            | Self::Trust
            | Self::KeyManagement
            | Self::HardwareSecurity
            | Self::SecureTunneling
            | Self::GeneticLineage => Some(crate::primal_names::BEARDOG),

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
            | Self::RelayServer => Some(crate::primal_names::SONGBIRD),

            // Compute → Toadstool
            Self::WorkloadExecution
            | Self::ResourceScheduling
            | Self::ProcessIsolation
            | Self::FractalCompute
            | Self::GpuAcceleration => Some(crate::primal_names::TOADSTOOL),

            // Storage → NestGate
            Self::DataStorage
            | Self::Provenance
            | Self::Compression
            | Self::Replication
            | Self::Deduplication
            | Self::ContentAddressed => Some(crate::primal_names::NESTGATE),

            // AI → Squirrel
            Self::AiCoordination
            | Self::AiMultiProvider
            | Self::McpServer
            | Self::AiCapabilityDiscovery => Some(crate::primal_names::SQUIRREL),

            // UI → petalTongue, Medical/Surgical → healthSpring (discovered at runtime)
            // Custom capabilities have no default
            Self::VisualRendering
            | Self::InputHandling
            | Self::MultiModal
            | Self::TopologyVisualization
            | Self::RealtimeUpdates
            | Self::StereoRendering
            | Self::MotionTracking
            | Self::HapticFeedback
            | Self::BiosignalProcessing
            | Self::PharmacokineticModeling
            | Self::SurgicalToolSimulation
            | Self::TissuePhysics
            | Self::AnatomyModeling
            | Self::Custom(_) => None,

            // Orchestration → biomeOS (self)
            Self::LifecycleManagement
            | Self::HealthMonitoring
            | Self::ConfigManagement
            | Self::MetricsCollection
            | Self::LogAggregation
            | Self::GraphOrchestration
            | Self::SporeDeployment
            | Self::NicheDeployment => Some("biomeos"),
        }
    }

    /// Resolve a capability string to a primal name using the taxonomy
    ///
    /// This is a convenience function for migrating from hardcoded capability→primal
    /// mappings to taxonomy-based resolution.
    #[must_use]
    pub fn resolve_to_primal(capability: &str) -> Option<&'static str> {
        Self::from_str_flexible(capability).and_then(|cap| cap.default_primal())
    }

    /// Return one representative variant for a given category.
    ///
    /// Used by [`helpers::capabilities_for_primal`] to resolve category → default
    /// primal without hardcoding primal names outside the taxonomy.
    #[must_use]
    pub const fn representative_for_category(
        category: super::category::CapabilityCategory,
    ) -> Option<Self> {
        match category {
            super::category::CapabilityCategory::Security => Some(Self::Encryption),
            super::category::CapabilityCategory::Communication => Some(Self::Discovery),
            super::category::CapabilityCategory::Compute => Some(Self::WorkloadExecution),
            super::category::CapabilityCategory::Storage => Some(Self::DataStorage),
            super::category::CapabilityCategory::AI => Some(Self::AiCoordination),
            super::category::CapabilityCategory::Orchestration => Some(Self::LifecycleManagement),
            super::category::CapabilityCategory::UserInterface => Some(Self::VisualRendering),
            super::category::CapabilityCategory::Specialized => Some(Self::GeneticLineage),
        }
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
    #[must_use]
    pub fn known_primals() -> &'static [&'static str] {
        Self::known_primals_with(std::env::var("BIOMEOS_STRICT_DISCOVERY").is_ok())
    }

    /// Returns known primal names; empty when `strict` is `true`.
    #[must_use]
    pub const fn known_primals_with(strict: bool) -> &'static [&'static str] {
        if strict {
            return &[];
        }
        crate::primal_names::CORE_PRIMALS
    }
}

impl fmt::Display for CapabilityTaxonomy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Self::Custom(name) = self {
            write!(f, "custom:{name}")
        } else {
            let s = format!("{self:?}");
            write!(f, "{}", s.to_lowercase())
        }
    }
}

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
#[path = "definition_tests.rs"]
mod tests;
