//! # biomeOS Spore System
//!
//! USB spore creation and management for biomeOS towers.
//!
#![warn(missing_docs)]
#![deny(unsafe_code)]
//!
//! ## Architectural Principle: Composability
//!
//! **biomeOS orchestrates. BearDog secures.**
//!
//! This module handles:
//! - ✅ File I/O (`.family.seed` management)
//! - ✅ Directory structure creation
//! - ✅ Configuration generation (`tower.toml`)
//! - ✅ Binary deployment
//! - ✅ USB device orchestration
//!
//! This module does NOT handle:
//! - ❌ Cryptography (BearDog's responsibility)
//! - ❌ Key derivation (BearDog's responsibility)
//! - ❌ Family ID extraction (BearDog's responsibility)
//! - ❌ Genetic lineage verification (BearDog's responsibility)
//!
//! ## Usage
//!
//! ```rust,no_run
//! use biomeos_spore::{Spore, SporeConfig, SporeType};
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create new spore on USB
//!     let config = SporeConfig {
//!         label: "biomeOS1".to_string(),
//!         node_id: "tower1".to_string(),
//!         spore_type: SporeType::Live,
//!         family_id: "1894e909e454".to_string(),
//!     };
//!     
//!     let spore = Spore::create(
//!         PathBuf::from("/media/usb"),
//!         config,
//!     ).await?;
//!     
//!     println!("Spore created at: {}", spore.root_path().display());
//!     Ok(())
//! }
//! ```

pub mod beacon_genetics;
pub mod dark_forest;
pub mod error;
pub mod incubation;
pub mod logs;
pub mod manifest;
pub mod neural_spore;
pub mod refresh;
pub mod seed;
pub mod spore;
pub mod spore_log_tracker;
pub mod spore_types;
pub mod usb;
pub mod verification;
pub mod verify;

// Test support - available in dev/test builds
#[doc(hidden)]
pub mod test_support;

pub use beacon_genetics::{
    BeaconGeneticsManager, BeaconGeneticsManifest, BeaconId, ClusterMembership, ClusterRole,
    MeetingRecord, MeetingRelationship, MeetingVisibility, SyncResult,
};
pub use dark_forest::{BeaconPlaintext, DarkForestBeacon, DiscoveredPeer, EncryptedBeacon};
pub use error::{SporeError, SporeResult};
pub use neural_spore::{DeploymentMetrics, NeuralSpore, PhaseMetrics, RollbackState};
pub use seed::FamilySeed;
pub use spore::{Spore, SporeConfig};
pub use spore_types::SporeType;
pub use verify::{SporeVerification, VerificationResult};

// Re-export for integration tests
#[doc(hidden)]
pub use test_support::setup_test_binaries;
