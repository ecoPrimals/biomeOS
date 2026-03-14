// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! BiomeOS Primal SDK
//!
//! Complete SDK for developing autonomous primals that follow deep debt principles:
//! - Self-knowledge only
//! - Runtime discovery of other primals
//! - Capability-based communication
//! - Graceful degradation
//!
#![warn(missing_docs)]
#![forbid(unsafe_code)]
//!
//! # Quick Start
//!
//! ```rust,no_run
//! use biomeos_primal_sdk::prelude::*;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Discover a primal by capability
//! let security = PrimalDiscovery::find_by_capability(
//!     PrimalCapability::security()
//! ).await?;
//!
//! // Connect and communicate
//! let client = PrimalClient::new(security);
//! let response = client.request("method", serde_json::json!({})).await?;
//! # Ok(())
//! # }
//! ```

// biomeOS unified types - direct re-exports
pub use biomeos_types::{
    BiomeError,

    // Error handling and results
    BiomeResult,
    CapabilityMetadata,

    ConfigValidationResult,

    // Health system
    Health,
    HealthReport,
    NetworkIoMetrics,
    PrimalCapability,
    PrimalConfiguration,

    PrimalServiceMetadata,
    // Core primal types
    PrimalType,
    ResourceMetrics,
    // Resource monitoring
    ResourceRequirements,
    ServiceStatus,
    // Core service interface
    UniversalPrimalService,
    UniversalServiceRegistration,
    UniversalServiceRequest,
    UniversalServiceResponse,
};

/// Extended types for primal-specific functionality (requests, responses, priorities)
pub mod types;

// Re-export extended types for convenience
pub use types::{PrimalRequest, PrimalResponse, RequestPriority};

/// Runtime primal discovery patterns
pub mod discovery;

/// Inter-primal communication helpers
pub mod communication;

/// tarpc binary transport helpers for protocol escalation
pub mod tarpc_transport;

/// Convenient prelude for common SDK imports
pub mod prelude {
    pub use crate::communication::{PrimalClient, SecureTunnel};
    pub use crate::discovery::{DiscoveredPrimal, DiscoveryQuery, PrimalDiscovery};
    pub use crate::types::{PrimalRequest, PrimalResponse, RequestPriority};
    pub use crate::{BiomeResult, Health, PrimalCapability, PrimalType, UniversalPrimalService};
}
