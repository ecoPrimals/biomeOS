//! # NUCLEUS - Secure Primal Discovery Protocol
//!
//! **Core Principle**: Delegate to primals, don't reimplement!
//!
//! NUCLEUS provides secure, authenticated, capability-based discovery by coordinating
//! existing primal capabilities:
//! - **`BearDog`**: Cryptographic identity, trust verification
//! - **Songbird**: Physical discovery, communication routing
//! - **biomeOS**: Orchestration, capability validation
//!
//! ## Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                         NUCLEUS                              │
//! │                  (Secure Discovery Protocol)                 │
//! ├─────────────────────────────────────────────────────────────┤
//! │ Layer 1: Physical Discovery       → Songbird (UDP multicast)│
//! │ Layer 2: Identity Verification    → BearDog (Ed25519 sig)   │
//! │ Layer 3: Capability Verification  → Direct socket query     │
//! │ Layer 4: Trust Evaluation         → BearDog (lineage)       │
//! │ Layer 5: Registration & Tracking  → Local registry          │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Principles (Deep Debt Solutions)
//!
//! 1. **No Hardcoding**: Discover primals at runtime by capability
//! 2. **No Reimplementation**: Use primal APIs, don't duplicate crypto/comms
//! 3. **Fast AND Safe**: Zero unsafe code, async/await throughout
//! 4. **Capability-Based**: Select primals by what they can do, not by name
//! 5. **Mocks Isolated**: Test utilities only in `#[cfg(test)]`
//!
//! ## Usage
//!
//! ```ignore
//! use biomeos_nucleus::{NucleusClient, DiscoveryRequest};
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Discover primals by capability (no hardcoding)
//! let client = NucleusClient::new().await?;
//! let primals = client.discover(DiscoveryRequest {
//!     capability: "encryption".to_string(),
//!     family: Some("nat0".to_string()),
//!     timeout: None,
//! }).await?;
//!
//! // Use discovered primal
//! for primal in primals {
//!     println!("Found: {} at {}", primal.name, primal.endpoint.address);
//! }
//! # Ok(())
//! # }
//! ```

#![deny(unsafe_code)] // Deep debt principle: Fast AND safe
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]

pub mod capability;
pub mod client;
pub mod discovery;
pub mod error;
pub mod identity;
pub mod registry;
pub mod trust;

// Re-export core types
pub use capability::{CapabilityInfo, CapabilityLayer, CapabilityVerification};
pub use client::{NucleusClient, NucleusClientBuilder};
pub use discovery::{DiscoveryLayer, DiscoveryRequest, PhysicalDiscovery};
pub use error::{Error, Result};
pub use identity::{IdentityLayer, IdentityProof, IdentityVerification};
pub use registry::{PrimalInfo, RegisteredPrimal, Registry};
pub use trust::{TrustEvaluation, TrustLayer, TrustLevel};

/// Primal endpoint information (discovered via Songbird)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Endpoint {
    /// Endpoint type (unix_socket, udp, tcp)
    pub endpoint_type: EndpointType,
    /// Endpoint address/path
    pub address: String,
}

/// Type of primal endpoint
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EndpointType {
    /// Unix domain socket (primary for local IPC)
    UnixSocket,
    /// UDP socket (for discovery and P2P)
    Udp,
    /// TCP socket (fallback)
    Tcp,
}

/// Verified primal (passed all NUCLEUS layers)
#[derive(Debug, Clone)]
pub struct VerifiedPrimal {
    /// Primal name (discovered, not hardcoded)
    pub name: String,
    /// Node ID
    pub node_id: String,
    /// Family ID
    pub family_id: String,
    /// Verified capabilities
    pub capabilities: Vec<String>,
    /// Primary endpoint
    pub endpoint: Endpoint,
    /// Trust level (evaluated by BearDog)
    pub trust_level: TrustLevel,
    /// Version
    pub version: String,
}
