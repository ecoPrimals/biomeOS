// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! # `biomeOS` Chimera System
//!
//! Chimeras are **mixed-boundary primal amalgams** - custom organisms that fuse
//! components from multiple standard primals into a single deployable unit.
//!
//! ## Conceptual Model
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                    CHIMERA: P2P-Secure                      │
//! │  ┌─────────────────────┐  ┌─────────────────────────────┐  │
//! │  │  BearDog Components │  │   Songbird Components       │  │
//! │  │  ├── btsp           │  │   ├── birdsong              │  │
//! │  │  ├── genetic_crypto │  │   ├── discovery             │  │
//! │  │  └── identity       │  │   └── mesh                  │  │
//! │  └─────────────────────┘  └─────────────────────────────┘  │
//! │                      ↓ FUSION ↓                             │
//! │            Unified API + Shared State                       │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Example
//!
//! ```rust,ignore
//! use biomeos_chimera::{ChimeraDefinition, ChimeraRegistry};
//!
//! // Load chimera definitions from YAML
//! let registry = ChimeraRegistry::from_directory("chimeras/definitions")?;
//!
//! // Get a specific chimera
//! let p2p_secure = registry.get("p2p-secure")?;
//!
//! // Build the chimera binary
//! let builder = ChimeraBuilder::new(p2p_secure);
//! builder.build_to("bin/chimeras/p2p-secure")?;
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod builder;
pub mod component;
pub mod definition;
pub mod error;
pub mod fusion;
pub mod registry;

// Re-exports
pub use builder::ChimeraBuilder;
pub use component::{Component, ComponentModule, PrimalSource};
pub use definition::{ChimeraDefinition, ChimeraMetadata};
pub use error::{ChimeraError, ChimeraResult};
pub use fusion::{Fusion, FusionBinding, FusionEndpoint};
pub use registry::ChimeraRegistry;
