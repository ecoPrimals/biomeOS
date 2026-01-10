// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! Shared types for spore operations

use serde::{Deserialize, Serialize};
use crate::spore_types::SporeType;

/// Configuration for spore creation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SporeConfig {
    /// Human-readable label for this spore
    pub label: String,
    
    /// Node ID for this tower (e.g., "tower1")
    pub node_id: String,

    /// Type of spore (Cold = storage, Live = deployable)
    #[serde(default)]
    pub spore_type: SporeType,
}


