// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Core Error Types
//!
//! This module contains the main `BiomeError` enum and core error types
//! that form the foundation of the unified error handling system.

mod biome_error;
mod ipc;
mod validation;

#[cfg(test)]
mod tests;

pub use biome_error::BiomeError;
pub use ipc::IpcError;
pub use validation::ValidationError;
