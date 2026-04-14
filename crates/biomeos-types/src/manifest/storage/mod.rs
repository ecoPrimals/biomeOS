// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Storage Specifications
//!
//! This module contains storage-related manifest types including `VolumeSpec`,
//! `SecretSpec`, `ConfigSpec`, and storage configuration.

pub mod config;
pub mod secret;
pub mod volume;

pub use config::*;
pub use secret::*;
pub use volume::*;

#[cfg(test)]
#[path = "../storage_tests.rs"]
mod tests;
