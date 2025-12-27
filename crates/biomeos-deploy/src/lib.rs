//! BiomeOS Deployment Orchestration
//!
//! Pure Rust deployment orchestration for BiomeOS federations.
//! Replaces bash scripts with type-safe, async Rust implementation.

pub mod error;
pub mod network;
pub mod qemu;
pub mod topology;
pub mod federation;
pub mod health;
pub mod verify;

pub use error::{DeployError, Result};
pub use federation::{Federation, FederationConfig};
pub use health::{HealthCheck, HealthStatus, VmHealth};
pub use network::{NetworkBridge, BridgeConfig};
pub use qemu::{QemuInstance, QemuConfig};
pub use topology::{Topology, VmTopology};
pub use verify::{VmVerifier, VerifyConfig, VerifyResult};

/// Re-export commonly used types
pub mod prelude {
    pub use crate::error::{DeployError, Result};
    pub use crate::federation::{Federation, FederationConfig};
    pub use crate::health::{HealthCheck, HealthStatus};
    pub use crate::network::NetworkBridge;
    pub use crate::qemu::QemuInstance;
    pub use crate::topology::Topology;
}

