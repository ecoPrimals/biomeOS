// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! PlasmidBin search paths, binary probing, and lifecycle state labels for spring status.

use serde_json::{Value, json};
use std::path::PathBuf;

use crate::lifecycle_manager::LifecycleState;

/// Collect plasmidBin search directories (same search order as primal_spawner).
pub(crate) fn binary_search_dirs() -> Vec<PathBuf> {
    [
        std::env::var("ECOPRIMALS_PLASMID_BIN")
            .ok()
            .map(PathBuf::from),
        std::env::var("BIOMEOS_PLASMID_BIN_DIR")
            .ok()
            .map(PathBuf::from),
        Some(PathBuf::from("./plasmidBin")),
        Some(PathBuf::from("../plasmidBin")),
        Some(PathBuf::from("../../plasmidBin")),
    ]
    .into_iter()
    .flatten()
    .filter(|p| p.exists())
    .collect()
}

/// Probe for a primal binary on disk, returning (found, path_or_null).
pub(crate) fn probe_binary(primal_name: &str, search_dirs: &[PathBuf]) -> (bool, Value) {
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;

    let patterns = [
        format!("{primal_name}_{arch}_{os}_musl/{primal_name}"),
        format!("{primal_name}_{arch}_{os}/{primal_name}"),
        format!("primals/{primal_name}/{primal_name}"),
        format!("primals/{primal_name}"),
        format!("{primal_name}/{primal_name}"),
        primal_name.to_string(),
    ];

    for dir in search_dirs {
        for pat in &patterns {
            let candidate = dir.join(pat);
            if candidate.exists() && candidate.is_file() {
                return (true, json!(candidate.display().to_string()));
            }
        }
    }
    (false, Value::Null)
}

/// Convert lifecycle state to a simple string.
pub(crate) const fn state_to_string(state: &LifecycleState) -> &'static str {
    match state {
        LifecycleState::Germinating => "germinating",
        LifecycleState::Incubating { .. } => "incubating",
        LifecycleState::Active { .. } => "active",
        LifecycleState::Degraded { .. } => "degraded",
        LifecycleState::Apoptosis { .. } => "apoptosis",
        LifecycleState::Dead { .. } => "dead",
    }
}
