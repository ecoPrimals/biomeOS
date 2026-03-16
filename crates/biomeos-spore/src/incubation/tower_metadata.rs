// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Extraction of metadata from tower.toml configuration.
//!
//! Reads spore ID, family ID, and other tower-specific metadata
//! from the spore's tower.toml file.

use std::path::Path;

use crate::error::SporeResult;

/// Extract spore ID from spore path or config
///
/// Tries to read from tower.toml meta.node_id, falls back to directory name.
pub fn extract_spore_id(spore_path: &Path) -> SporeResult<String> {
    let tower_toml_path = spore_path.join("tower.toml");

    if tower_toml_path.exists() {
        let content = std::fs::read_to_string(&tower_toml_path)?;

        // Parse TOML and extract node_id from meta section
        if let Ok(config) = toml::from_str::<toml::Value>(&content)
            && let Some(meta) = config.get("meta")
            && let Some(node_id) = meta.get("node_id")
            && let Some(id) = node_id.as_str()
        {
            return Ok(id.to_string());
        }
    }

    // Fallback: use directory name or UUID
    Ok(spore_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown")
        .to_string())
}

/// Extract family_id from tower.toml
///
/// Tries tower.family first, then meta.family_id, falls back to "unknown".
pub fn extract_family_id(spore_path: &Path) -> Result<String, anyhow::Error> {
    let tower_toml_path = spore_path.join("tower.toml");
    let content = std::fs::read_to_string(&tower_toml_path)?;

    if let Ok(config) = toml::from_str::<toml::Value>(&content) {
        if let Some(tower) = config.get("tower")
            && let Some(family) = tower.get("family")
            && let Some(family_str) = family.as_str()
        {
            return Ok(family_str.to_string());
        }

        // Fallback: check meta.family_id
        if let Some(meta) = config.get("meta")
            && let Some(family_id) = meta.get("family_id")
            && let Some(id) = family_id.as_str()
        {
            return Ok(id.to_string());
        }
    }

    Ok("unknown".to_string())
}
