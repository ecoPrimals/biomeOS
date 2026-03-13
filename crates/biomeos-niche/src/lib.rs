// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! # `BiomeOS` Niche System
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

#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![warn(clippy::all)]
#![warn(clippy::pedantic)]
#![allow(clippy::module_name_repetitions)]

pub mod definition;
pub mod deployment;
pub mod error;
pub mod interaction;
pub mod organism;

// Re-exports
pub use definition::{NicheCustomization, NicheDefinition, NicheMetadata};
pub use deployment::{DeploymentStatus, NicheDeployment};
pub use error::{NicheError, NicheResult};
pub use interaction::Interaction;
pub use organism::{ChimeraOrganism, Organism, OrganismType, PrimalOrganism};
