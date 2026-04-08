// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! `BiomeOS` Bootable Media Creator
//!
//! Pure Rust implementation of bootable USB/ISO creation.
//! Clean architecture with modern idiomatic patterns.

mod builder;
mod copy;
mod grub;
mod iso;
mod types;

pub use types::BootTarget;

use std::path::PathBuf;

/// Bootable media builder with clean separation of concerns.
pub struct BootableMediaBuilder {
    pub(in crate::bootable) project_root: PathBuf,
    pub(in crate::bootable) work_dir: PathBuf,
    pub(in crate::bootable) output_dir: PathBuf,
}
