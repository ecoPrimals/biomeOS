//! # biomeOS Spore System
//!
//! USB spore creation and management for biomeOS towers.
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
//! use biomeos_spore::{Spore, SporeConfig};
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create new spore on USB
//!     let config = SporeConfig {
//!         label: "biomeOS1".to_string(),
//!         node_id: "tower1".to_string(),
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

pub mod error;
pub mod seed;
pub mod spore;
pub mod usb;
pub mod verify;

pub use error::{SporeError, SporeResult};
pub use seed::FamilySeed;
pub use spore::{Spore, SporeConfig};
pub use verify::{SporeVerification, VerificationResult};

