// SPDX-License-Identifier: AGPL-3.0-only
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! USB Spore Management
//!
//! A spore is a self-contained biomeOS deployment on a USB device. It contains
//! everything needed to boot a tower: genetic seeds, primal binaries, configuration.
//!
//! # Architecture
//!
//! This module is organized by domain:
//!
//! - **[`core`]** - Core Spore struct, lifecycle management (create, load, clone)
//! - **[`filesystem`]** - Directory structure and binary operations
//! - **[`config`]** - Configuration file generation (tower.toml)
//! - **[`genetics`]** - Genetic seed generation and lineage
//! - **[`deployment`]** - Deployment script generation
//! - **[`documentation`]** - README and manifest creation
//! - **[`types`]** - Shared types (SporeConfig, moved from parent)
//!
//! # Biology-Inspired Design
//!
//! Spores follow real biological principles:
//! - **Cold Spores**: Genetic preservation (storage/archival)
//! - **Live Spores**: Self-contained, bootable, ready to germinate
//! - **Siblings**: NOT perfect clones, unique genetic variation
//!
//! # Quick Start
//!
//! ```no_run
//! use biomeos_spore::spore::{Spore, SporeConfig};
//! use biomeos_spore::spore_types::SporeType;
//! use std::path::PathBuf;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Create a new spore
//!     let config = SporeConfig {
//!         label: "biomeOS1".to_string(),
//!         node_id: "tower1".to_string(),
//!         spore_type: SporeType::Live,
//!         family_id: "1894e909e454".to_string(),
//!         plasmid_bin_dir: None,
//!     };
//!     
//!     let spore = Spore::create(
//!         PathBuf::from("/media/usb"),
//!         config,
//!     ).await?;
//!     
//!     // Clone a sibling spore
//!     let sibling = spore.clone_sibling(
//!         PathBuf::from("/media/usb2"),
//!         "tower2",
//!     ).await?;
//!     
//!     Ok(())
//! }
//! ```

// Module declarations
pub mod config;
pub mod core;
pub mod deployment;
pub mod documentation;
pub mod filesystem;
pub mod genetics;
pub mod types;

// Re-export main types
pub use core::Spore;
pub use types::SporeConfig;
