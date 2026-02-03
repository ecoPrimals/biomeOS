//! biomeOS Federation - Hierarchical trust and sub-federation management
//!
//! This module provides:
//! - Family-level federation (genetic lineage baseline)
//! - Sub-federations (capability-based granular access)
//! - Runtime primal discovery (no hardcoding)
//! - BearDog integration for all cryptographic operations
//! - NUCLEUS secure discovery protocol (5-layer verification)

pub mod beardog_client;
pub mod capability;
pub mod discovery;
pub mod nucleus;
pub mod subfederation;
pub mod unix_socket_client;

pub use capability::{Capability, CapabilitySet};
pub use discovery::{DiscoveredPrimal, PrimalDiscovery};
pub use nucleus::{
    IdentityProof, SecureNucleusDiscovery, SelectionCriteria, TrustLevel, VerifiedPrimal,
};
pub use subfederation::{IsolationLevel, SubFederation, SubFederationManager};

// Re-export PrimalEndpoint for tests
pub use beardog_client::BearDogClient;
pub use discovery::PrimalEndpoint;
pub use unix_socket_client::{JsonRpcRequest, JsonRpcResponse, UnixSocketClient};

use thiserror::Error;

#[derive(Error, Debug)]
pub enum FederationError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] toml::de::Error),

    #[error("Deserialization error: {0}")]
    Deserialization(#[from] toml::ser::Error),

    #[error("Node {0} not found in federation")]
    NodeNotFound(String),

    #[error("Sub-federation {0} not found")]
    SubFederationNotFound(String),

    #[error("Capability {0} not allowed in sub-federation {1}")]
    CapabilityDenied(String, String),

    #[error("Genetic lineage verification failed: {0}")]
    LineageVerificationFailed(String),

    #[error("BearDog error: {0}")]
    BearDogError(String),

    #[error("Discovery error: {0}")]
    DiscoveryError(String),

    #[error("Generic error: {0}")]
    Generic(String),
}

pub type FederationResult<T> = Result<T, FederationError>;
