// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Device lineage derivation types

use serde::{Deserialize, Serialize};

/// Device lineage seed (derived, unique per device)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceLineage {
    /// Device identifier
    pub device_id: String,
    /// Node identifier (human-friendly name)
    pub node_id: String,
    /// Family ID this device belongs to
    pub family_id: String,
    /// Generation (1 = direct child of genesis)
    pub generation: u32,
    /// Derived seed (base64, 32 bytes)
    pub derived_seed: String,
    /// Timestamp of derivation
    pub derived_at: u64,
    /// Derivation method used
    pub derivation_method: String,
    /// Optional lineage certificate (when available)
    pub lineage_certificate: Option<String>,
}

/// Result of device enrollment
#[derive(Debug, Clone)]
pub struct EnrollmentResult {
    /// The derived device lineage
    pub lineage: DeviceLineage,
    /// Path where lineage seed was saved
    pub seed_path: std::path::PathBuf,
}

/// Derivation request parameters
#[derive(Debug, Clone, Serialize)]
pub struct DerivationParams {
    /// The root/family seed (base64)
    pub family_seed: String,
    /// Device ID for derivation
    pub device_id: String,
    /// Node ID for derivation
    pub node_id: String,
    /// Additional entropy (base64, optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub device_entropy: Option<String>,
    /// Purpose/context string
    pub purpose: String,
}
