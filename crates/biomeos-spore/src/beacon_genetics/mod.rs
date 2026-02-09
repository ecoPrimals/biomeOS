//! Beacon Genetics - Mitochondrial DNA Model
//!
//! Implements the two-seed architecture for Dark Forest discovery.
//!
//! ## Architectural Principle: Primals Have Self-Knowledge Only
//!
//! **biomeOS orchestrates. Primals execute primitives.**
//!
//! This module handles ecosystem-level concepts:
//! - ✅ "Meetings" (social graph of beacon exchanges)
//! - ✅ "Address book" (collection of met beacon seeds)
//! - ✅ "Beacon vs Lineage" (discovery vs permissions)
//! - ✅ Orchestration of multi-step workflows
//!
//! Primals provide primitives via capability.call:
//! - BearDog: `beacon.encrypt`, `beacon.decrypt`, `beacon.generate`
//! - Nestgate: `storage.write`, `storage.read`
//! - Songbird: `network.send`, `network.receive`
//!
//! ## Key Concepts
//!
//! - **Own Seed**: Your beacon seed (encrypts YOUR broadcasts)
//! - **Met Seeds**: Seeds from meetings (decrypt THEIR broadcasts)
//! - **Address Book**: Collection of met seeds + metadata
//! - **Lineage Hint**: Loose tie to lineage (for sync only)
//!
//! ## capability.call Flow
//!
//! ```text
//! BeaconGeneticsManager (ecosystem knowledge)
//!     │
//!     │ capability.call("beacon.encrypt", params)
//!     ▼
//! CapabilityTranslationRegistry (semantic → actual)
//!     │
//!     │ Translates to: method="beacon.encrypt", socket="/run/user/.../beardog.sock"
//!     ▼
//! BearDog (self-knowledge: crypto primitives)
//! ```
//!
//! ## Module Structure (Smart Refactor)
//!
//! - `types` - Core data types (BeaconId, MeetingRecord, etc.)
//! - `capability` - CapabilityCaller trait and NeuralAPI implementation
//! - `manager` - BeaconGeneticsManager orchestration
//!
//! AGPL-3.0-only License

mod capability;
mod derivation;
mod manager;
mod types;

// Re-export public types
pub use capability::{CapabilityCaller, DirectBeardogCaller, NeuralApiCapabilityCaller};
pub use derivation::{
    generate_device_entropy, DerivationParams, DeviceLineage, EnrollmentResult, LineageDeriver,
};
pub use manager::BeaconGeneticsManager;
pub use types::{
    current_timestamp, BeaconGeneticsManifest, BeaconId, ClusterMembership, ClusterRole,
    MeetingRecord, MeetingRelationship, MeetingVisibility, SyncResult, Timestamp,
};
