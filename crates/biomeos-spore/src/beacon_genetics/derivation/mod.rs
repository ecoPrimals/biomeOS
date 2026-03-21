// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Device Lineage Derivation
//!
//! Implements the genetic model for device-specific seed derivation.
//! DERIVE, Don't Clone - each device gets a unique seed from the shared family root.

mod lineage_deriver;
mod types;
mod utils;

#[cfg(test)]
mod tests;

pub use lineage_deriver::LineageDeriver;
pub use types::{DerivationParams, DeviceLineage, EnrollmentResult};
pub use utils::generate_device_entropy;
