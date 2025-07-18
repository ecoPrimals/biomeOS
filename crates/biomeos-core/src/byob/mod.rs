//! BYOB (Bring Your Own Biome) functionality
//!
//! This module enables teams to deploy independently while leveraging shared Primal infrastructure.
//! Teams maintain sovereignty while benefiting from network effects.
//!
//! Enhanced with comprehensive health monitoring and Toadstool orchestration integration.

pub mod types;
pub mod manager;
pub mod health;
pub mod defaults;

// Re-export commonly used types and structures
pub use types::*;

// Re-export the main manager implementation
pub use types::ByobDeploymentManager;

// Re-export key types for external use
pub use types::{
    TeamWorkspace, ResourceQuota, ResourceUsage, TeamHealthConfig,
    DeploymentInstance, DeploymentStatus, IsolationConfig,
    SimpleBiomeManifest, SimpleBiomeMetadata, SimpleBiomeService, SimpleBiomeResources,
    ToadstoolIntegrationConfig, ToadstoolFeatures, AlertThresholds,
    AutoScalingConfig, ServiceMeshConfig, LoadBalancerConfig
};

// Re-export health types
pub use health::{TeamHealthReport, ResourceUtilizationReport}; 