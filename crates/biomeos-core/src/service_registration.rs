//! Service Registration for biomeOS
//!
//! This module contains the types and functionality for service registration
//! in the biomeOS ecosystem, including the "Songbird Pattern" for service discovery.

use crate::PrimalType;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Service registration for the "Songbird Pattern"
/// This struct defines the "birth announcement" that each Primal sends to Songbird
#[derive(Debug, Clone, Serialize)]
pub struct ServiceRegistration {
    /// The unique name of the service, from the manifest.
    pub service_name: String,

    /// The Primal type that owns this service
    pub primal_type: PrimalType,

    /// The runtime type, for songbird's awareness.
    pub runtime: RuntimeType,

    /// The internal IP address and port where the service's API is listening.
    /// This is on the private `primal_net`.
    pub internal_address: String, // e.g., "10.42.0.5:8000"

    /// The public-facing paths that should be routed to this service.
    /// Example: `"/api/v1/storage"`
    pub public_routes: Vec<String>,

    /// A URL for songbird to poll for health checks.
    pub health_check_url: String,

    /// Any additional metadata the service wants to provide.
    #[serde(default)]
    pub metadata: HashMap<String, String>,

    /// Biome ID this service belongs to
    pub biome_id: String,

    /// Service capabilities
    pub capabilities: ServiceCapabilities,
}

/// Service capabilities
#[derive(Debug, Clone, Serialize)]
pub struct ServiceCapabilities {
    /// Core capabilities provided by this service
    pub core: Vec<String>,
    /// Extended features available
    pub extended: Vec<String>,
    /// Integration points with other Primals
    pub integrations: Vec<String>,
}

/// Runtime types supported by biomeOS
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RuntimeType {
    Wasm,
    Container,
    Native,
    Gpu,
    Agent, // For AI agents via Squirrel
}

/// MYCORRHIZA compliance status
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ComplianceStatus {
    FullyCompliant,
    PartiallyCompliant,
    NonCompliant,
    Unknown,
}

/// Extended resource requirements for biomeOS
#[derive(Debug, Clone)]
pub struct BiomeResourceRequirements {
    pub cpu_cores: Option<f64>,
    pub memory_mb: Option<u64>,
    pub storage_mb: Option<u64>,
    pub network_bandwidth_mbps: Option<u64>,
    pub gpu_count: Option<u32>,
}

/// Primal lifecycle events
#[derive(Debug, Clone)]
pub enum PrimalEvent {
    Starting,
    Started,
    Healthy,
    Degraded,
    Stopped,
    Failed(String),
}

/// Event listener for Primal lifecycle events
#[async_trait]
pub trait PrimalEventListener: Send + Sync {
    async fn on_primal_event(&self, primal_type: PrimalType, event: PrimalEvent);
}
