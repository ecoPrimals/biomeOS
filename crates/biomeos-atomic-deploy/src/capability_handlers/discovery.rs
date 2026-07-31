// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Capability resolution and binary discovery
//!
//! Resolves capabilities to primal names via taxonomy and discovers primal binaries
//! in plasmidBin directories.

use anyhow::Result;
use std::path::PathBuf;
use tracing::{debug, info};

use crate::executor::context::ExecutionContext;

/// Resolve capability to primal name using the capability taxonomy
///
/// Uses `biomeos_types::CapabilityTaxonomy` for consistent mapping across the codebase.
/// Set `BIOMEOS_STRICT_DISCOVERY=1` to disable fallback and require Songbird discovery.
pub fn resolve_capability_to_primal(capability: &str) -> Option<&'static str> {
    biomeos_types::CapabilityTaxonomy::resolve_to_primal(capability)
}

/// Get all known primal names from the capability taxonomy
///
/// This provides a canonical list of primals for health checks and deployment.
/// Uses `biomeos_types::CapabilityTaxonomy::known_primals()` for consistency.
pub fn known_primal_names() -> Vec<&'static str> {
    known_primal_names_with(false)
}

/// Like [`known_primal_names`], with explicit strict-discovery mode (no env reads).
#[must_use]
pub fn known_primal_names_with(strict_discovery: bool) -> Vec<&'static str> {
    biomeos_types::CapabilityTaxonomy::known_primals_with(strict_discovery).to_vec()
}

/// Discover binary path for a primal
///
/// Search order:
/// 1. `BIOMEOS_PLASMID_BIN_DIR` from execution context (explicit override)
/// 2. Shared search dirs (plasmidBin + user-space + $PATH) via `binary_search_dirs()`
pub async fn discover_primal_binary(
    primal_name: &str,
    context: &ExecutionContext,
) -> Result<PathBuf> {
    let base_dirs: Vec<PathBuf> = if let Some(explicit) = context
        .env()
        .get(biomeos_types::env_config::vars::PLASMID_BIN_DIR)
    {
        vec![PathBuf::from(explicit)]
    } else {
        crate::handlers::spring_status::binary_search_dirs()
    };

    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    let target = format!("{arch}-{os}");

    debug!("   Discovering {} binary for {}", primal_name, target);

    for base_dir in &base_dirs {
        if !base_dir.exists() {
            continue;
        }

        // Try architecture-specific path first
        let arch_path = base_dir.join(&target).join(primal_name);
        if arch_path.exists() {
            info!("   Found: {}", arch_path.display());
            return Ok(arch_path);
        }

        // Try generic path (bare binary name in directory)
        let generic_path = base_dir.join(primal_name);
        if generic_path.exists() && generic_path.is_file() {
            info!("   Found: {}", generic_path.display());
            return Ok(generic_path);
        }
    }

    anyhow::bail!("Binary not found for: {primal_name}")
}

#[cfg(test)]
#[path = "discovery_tests.rs"]
mod tests;
