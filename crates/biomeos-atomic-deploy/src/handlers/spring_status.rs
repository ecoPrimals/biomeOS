// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! PlasmidBin search paths, binary probing, and lifecycle state labels for spring status.

use serde_json::{Value, json};
use std::path::PathBuf;

use crate::lifecycle_manager::LifecycleState;

/// Collect binary search directories: plasmidBin + user-space + $PATH.
///
/// Search order mirrors `primal_spawner::discover_primal_binary_impl`:
/// 1. Explicit env overrides (`ECOPRIMALS_PLASMID_BIN`, `BIOMEOS_PLASMID_BIN_DIR`)
/// 2. Relative plasmidBin paths (depot-style)
/// 3. User-space paths (`~/.local/bin`, `~/.cargo/bin`)
/// 4. `$PATH` entries (covers system-wide installs and source builds)
pub(crate) fn binary_search_dirs() -> Vec<PathBuf> {
    let mut dirs: Vec<PathBuf> = [
        std::env::var(biomeos_types::env_config::vars::PLASMID_BIN)
            .ok()
            .map(PathBuf::from),
        std::env::var(biomeos_types::env_config::vars::PLASMID_BIN_DIR)
            .ok()
            .map(PathBuf::from),
        Some(PathBuf::from("./plasmidBin")),
        Some(PathBuf::from("../plasmidBin")),
        Some(PathBuf::from("../../plasmidBin")),
    ]
    .into_iter()
    .flatten()
    .collect();

    // User-space paths for non-depot deployments (westGate, strandGate, steamGate)
    if let Ok(home) = std::env::var("HOME") {
        let local_bin = PathBuf::from(&home).join(".local/bin");
        if local_bin.exists() {
            dirs.push(local_bin);
        }
        let cargo_bin = PathBuf::from(&home).join(".cargo/bin");
        if cargo_bin.exists() {
            dirs.push(cargo_bin);
        }
    }

    // $PATH entries as final fallback (covers arbitrary install locations)
    if let Ok(path_var) = std::env::var("PATH") {
        for entry in std::env::split_paths(&path_var) {
            if entry.exists() && !dirs.contains(&entry) {
                dirs.push(entry);
            }
        }
    }

    dirs.retain(|p| p.exists());
    dirs
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
