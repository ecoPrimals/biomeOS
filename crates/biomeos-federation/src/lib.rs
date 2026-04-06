// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! biomeOS Federation - Hierarchical trust and sub-federation management
//!
//! This module provides:
//! - Family-level federation (genetic lineage baseline)
//! - Sub-federations (capability-based granular access)
//! - Runtime primal discovery (no hardcoding)
//! - Security provider integration for all cryptographic operations
//! - NUCLEUS secure discovery protocol (5-layer verification)

#![warn(missing_docs)]
#![forbid(unsafe_code)]

pub mod capability;
pub mod discovery;
pub mod nucleus;
pub mod security_client;
pub mod subfederation;
pub mod unix_socket_client;

pub use capability::{Capability, CapabilitySet};
pub use discovery::{DiscoveredPrimal, PrimalDiscovery};
pub use nucleus::{
    IdentityProof, SecureNucleusDiscovery, SelectionCriteria, TrustLevel, VerifiedPrimal,
};
pub use subfederation::{IsolationLevel, SubFederation, SubFederationManager};

// Re-export PrimalEndpoint for tests
pub use discovery::PrimalEndpoint;
pub use security_client::SecurityProviderClient;
pub use unix_socket_client::{JsonRpcRequest, JsonRpcResponse, UnixSocketClient};

use thiserror::Error;

/// Errors produced by federation operations
#[derive(Error, Debug)]
pub enum FederationError {
    /// Underlying I/O error
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// TOML deserialization error
    #[error("Serialization error: {0}")]
    Serialization(#[from] toml::de::Error),

    /// TOML serialization error
    #[error("Deserialization error: {0}")]
    Deserialization(#[from] toml::ser::Error),

    /// A requested node was not found in the federation
    #[error("Node {0} not found in federation")]
    NodeNotFound(String),

    /// A requested sub-federation was not found
    #[error("Sub-federation {0} not found")]
    SubFederationNotFound(String),

    /// An operation required a capability not allowed in the sub-federation
    #[error("Capability {0} not allowed in sub-federation {1}")]
    CapabilityDenied(String, String),

    /// Genetic lineage verification failed
    #[error("Genetic lineage verification failed: {0}")]
    LineageVerificationFailed(String),

    /// Error communicating with the security provider
    #[error("Security provider error: {0}")]
    SecurityProviderError(String),

    /// Primal discovery error
    #[error("Discovery error: {0}")]
    DiscoveryError(String),

    /// Catch-all error
    #[error("Generic error: {0}")]
    Generic(String),
}

/// Convenience result alias for federation operations
pub type FederationResult<T> = Result<T, FederationError>;
