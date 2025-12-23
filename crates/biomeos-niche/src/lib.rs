//! # BiomeOS Niche System
//!
//! Niches are **biomes** - complete environments where primals and chimeras
//! operate together. The BYOB (Build Your Own Biome) system creates niches.
//!
//! ## Conceptual Model
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────────┐
//! │                    NICHE: Gaming-Tournament                     │
//! │                       (The Biome)                               │
//! │  ┌────────────┐ ┌────────────┐ ┌─────────────────────────────┐ │
//! │  │  BearDog   │ │  NestGate  │ │    CHIMERA: Gaming-Mesh     │ │
//! │  │  (Primal)  │ │  (Primal)  │ │  ┌─────────┬──────────────┐ │ │
//! │  │ Anti-cheat │ │  Replays   │ │  │Songbird │   ToadStool  │ │ │
//! │  └────────────┘ └────────────┘ │  │  Array  │     GPU      │ │ │
//! │                                │  └─────────┴──────────────┘ │ │
//! │                                └─────────────────────────────┘ │
//! └─────────────────────────────────────────────────────────────────┘
//! ```

#![deny(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod definition;
pub mod organism;
pub mod interaction;
pub mod deployment;
pub mod error;

// Re-exports
pub use definition::{NicheDefinition, NicheMetadata, NicheCustomization};
pub use organism::{Organism, OrganismType, ChimeraOrganism, PrimalOrganism};
pub use interaction::Interaction;
pub use deployment::{NicheDeployment, DeploymentStatus};
pub use error::{NicheError, NicheResult};

