// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Common types for the Universal BiomeOS Manager

use bytes::Bytes;
use serde::{Deserialize, Serialize};

pub use super::discovery::{DiscoveryResult, ProbeResult};
pub use biomeos_types::health::ComponentHealth;

/// Genetic beardog key for partnership access
#[derive(Debug, Clone)]
pub struct GeneticAccessKey {
    pub key_data: Bytes,
    pub access_level: AccessLevel,
    pub expires_at: chrono::DateTime<chrono::Utc>,
}

/// Genetic access key access levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccessLevel {
    Individual,
    SmallBusiness,
    Enterprise,
    MegaCorp,
}
