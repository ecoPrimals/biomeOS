//! # Platypus - A Fused Chimera
//!
//! Platypus demonstrates **deep genetic mixing** - not orchestration of
//! separate processes, but a genuinely new organism with fused genetics.
//!
//! ## The Platypus Pattern
//!
//! Nature creates weird niches. The platypus isn't a duck + beaver running
//! side by side - it's a new species with mixed genetic material that
//! creates novel capabilities.
//!
//! Similarly, Platypus isn't BearDog + Songbird orchestrated together.
//! It's a new primal that deeply integrates:
//!
//! - **Genetic cryptography** (from BearDog lineage)
//! - **Mesh networking** (from Songbird lineage)
//!
//! The result: capabilities impossible with orchestration alone.
//!
//! ## Novel Capabilities
//!
//! - **Genetic mesh discovery**: Peers discovered through mesh, but trust
//!   established through cryptographic lineage verification
//! - **Evolving encrypted state**: Mesh topology that mutates over time,
//!   with each mutation cryptographically signed
//! - **Aquatic operation**: Designed for fluid, changing network conditions

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod crypto;
pub mod mesh;
pub mod fusion;

pub use crypto::{GeneticKeys, Identity};
pub use mesh::{MeshNode, Peer};
pub use fusion::{Platypus, PlatypusConfig};

/// Result type for Platypus operations
pub type Result<T> = std::result::Result<T, PlatypusError>;

/// Errors specific to Platypus operations
#[derive(Debug, thiserror::Error)]
pub enum PlatypusError {
    /// Cryptographic operation failed
    #[error("Crypto error: {0}")]
    Crypto(String),
    
    /// Mesh operation failed
    #[error("Mesh error: {0}")]
    Mesh(String),
    
    /// Lineage verification failed
    #[error("Lineage verification failed for peer: {peer_id}")]
    LineageVerification {
        /// The peer that failed verification
        peer_id: String,
    },
    
    /// Configuration error
    #[error("Configuration error: {0}")]
    Config(String),
}

