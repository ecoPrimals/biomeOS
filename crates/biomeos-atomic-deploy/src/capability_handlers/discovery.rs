// SPDX-License-Identifier: AGPL-3.0-only
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
/// 1. `BIOMEOS_PLASMID_BIN_DIR` environment variable
/// 2. ./plasmidBin directory
/// 3. ../plasmidBin directory
/// 4. ../../plasmidBin directory
pub async fn discover_primal_binary(
    primal_name: &str,
    context: &ExecutionContext,
) -> Result<PathBuf> {
    let explicit_dir = context
        .env()
        .get("BIOMEOS_PLASMID_BIN_DIR")
        .cloned()
        .map(PathBuf::from);

    let base_dirs: Vec<Option<PathBuf>> = if explicit_dir.is_some() {
        vec![explicit_dir]
    } else {
        vec![
            Some(PathBuf::from("./plasmidBin")),
            Some(PathBuf::from("../plasmidBin")),
            Some(PathBuf::from("../../plasmidBin")),
        ]
    };

    // Auto-detect architecture
    let arch = std::env::consts::ARCH;
    let os = std::env::consts::OS;
    let target = format!("{arch}-{os}");

    debug!("   Discovering {} binary for {}", primal_name, target);

    for base_dir_opt in base_dirs {
        let Some(base_dir) = base_dir_opt else {
            continue;
        };

        // Try architecture-specific path first
        let arch_path = base_dir.join(&target).join(primal_name);
        if arch_path.exists() {
            info!("   Found: {}", arch_path.display());
            return Ok(arch_path);
        }

        // Try generic path
        let generic_path = base_dir.join(primal_name);
        if generic_path.exists() {
            info!("   Found: {}", generic_path.display());
            return Ok(generic_path);
        }
    }

    anyhow::bail!("Binary not found for: {primal_name}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::executor::context::ExecutionContext;
    use std::collections::HashMap;

    // -------------------------------------------------------------------------
    // resolve_capability_to_primal - Domain mapping logic
    // -------------------------------------------------------------------------

    #[test]
    fn test_resolve_capability_to_primal_encryption() {
        assert_eq!(
            resolve_capability_to_primal("encryption"),
            Some("beardog"),
            "Encryption capability maps to beardog"
        );
    }

    #[test]
    fn test_resolve_capability_to_primal_discovery() {
        assert_eq!(
            resolve_capability_to_primal("discovery"),
            Some("songbird"),
            "Discovery capability maps to songbird"
        );
    }

    #[test]
    fn test_resolve_capability_to_primal_compute() {
        assert_eq!(
            resolve_capability_to_primal("compute"),
            Some("toadstool"),
            "Compute capability maps to toadstool"
        );
    }

    #[test]
    fn test_resolve_capability_to_primal_storage() {
        assert_eq!(
            resolve_capability_to_primal("storage"),
            Some("nestgate"),
            "Storage capability maps to nestgate"
        );
    }

    #[test]
    fn test_resolve_capability_to_primal_ai() {
        assert_eq!(
            resolve_capability_to_primal("ai"),
            Some("squirrel"),
            "AI capability maps to squirrel"
        );
    }

    #[test]
    fn test_resolve_capability_to_primal_case_insensitive() {
        assert_eq!(
            resolve_capability_to_primal("ENCRYPTION"),
            Some("beardog"),
            "Capability resolution should be case-insensitive"
        );
        assert_eq!(
            resolve_capability_to_primal("Discovery"),
            Some("songbird"),
            "Mixed case should resolve"
        );
    }

    #[test]
    fn test_resolve_capability_to_primal_nat_traversal_aliases() {
        assert_eq!(
            resolve_capability_to_primal("mesh"),
            Some("songbird"),
            "mesh maps to songbird"
        );
        assert_eq!(
            resolve_capability_to_primal("punch"),
            Some("songbird"),
            "punch maps to songbird"
        );
        assert_eq!(
            resolve_capability_to_primal("stun"),
            Some("songbird"),
            "stun maps to songbird"
        );
        assert_eq!(
            resolve_capability_to_primal("federation"),
            Some("songbird"),
            "federation maps to songbird"
        );
    }

    #[test]
    fn test_resolve_capability_to_primal_unknown() {
        assert_eq!(
            resolve_capability_to_primal("unknown"),
            None,
            "Unknown capabilities return None"
        );
        assert_eq!(
            resolve_capability_to_primal("nonexistent"),
            None,
            "Nonexistent capability returns None"
        );
    }

    #[test]
    fn test_resolve_capability_to_primal_empty_input() {
        assert_eq!(
            resolve_capability_to_primal(""),
            None,
            "Empty string should return None"
        );
    }

    #[test]
    fn test_resolve_capability_to_primal_whitespace() {
        assert_eq!(
            resolve_capability_to_primal("  encryption  "),
            None,
            "Whitespace-padded should not match (no trim)"
        );
    }

    // -------------------------------------------------------------------------
    // known_primal_names - Capability taxonomy bootstrap
    // -------------------------------------------------------------------------

    #[test]
    fn test_known_primal_names_contains_core_primals() {
        let primals = known_primal_names();

        assert!(primals.contains(&"beardog"), "Should contain beardog");
        assert!(primals.contains(&"songbird"), "Should contain songbird");
        assert!(primals.contains(&"toadstool"), "Should contain toadstool");
        assert!(primals.contains(&"nestgate"), "Should contain nestgate");
        assert!(primals.contains(&"squirrel"), "Should contain squirrel");
    }

    #[test]
    fn test_known_primal_names_returns_vec() {
        let primals = known_primal_names();
        assert_eq!(
            primals.len(),
            7,
            "Should have exactly 7 core primals when not in strict mode"
        );
    }

    #[test]
    fn test_known_primal_names_strict_discovery() {
        let primals = known_primal_names_with(true);

        assert!(
            primals.is_empty(),
            "Strict discovery mode should return empty list"
        );
    }

    #[test]
    fn test_known_primal_names_no_duplicates() {
        let primals = known_primal_names();
        let unique: std::collections::HashSet<_> = primals.iter().collect();
        assert_eq!(
            unique.len(),
            primals.len(),
            "Known primals should have no duplicates"
        );
    }

    // -------------------------------------------------------------------------
    // discover_primal_binary - Binary discovery logic
    // -------------------------------------------------------------------------

    #[tokio::test]
    async fn test_discover_primal_binary_success_via_env() {
        let temp = tempfile::tempdir().expect("temp dir");
        let bin_path = temp.path().join("beardog");
        std::fs::write(&bin_path, "#!/bin/sh\nexit 0").expect("write stub");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bin_path, std::fs::Permissions::from_mode(0o755))
                .expect("chmod");
        }

        let mut env = HashMap::new();
        env.insert(
            "BIOMEOS_PLASMID_BIN_DIR".to_string(),
            temp.path().to_string_lossy().to_string(),
        );
        let ctx = ExecutionContext::new(env);

        let result = discover_primal_binary("beardog", &ctx)
            .await
            .expect("Should find beardog in BIOMEOS_PLASMID_BIN_DIR");

        assert!(result.exists(), "Resolved path should exist");
        assert_eq!(result.file_name().unwrap(), "beardog");
    }

    #[tokio::test]
    async fn test_discover_primal_binary_success_arch_specific() {
        let temp = tempfile::tempdir().expect("temp dir");
        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;
        let target_dir = temp.path().join(format!("{arch}-{os}"));
        std::fs::create_dir_all(&target_dir).expect("create dir");
        let bin_path = target_dir.join("squirrel");
        std::fs::write(&bin_path, "#!/bin/sh\nexit 0").expect("write stub");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bin_path, std::fs::Permissions::from_mode(0o755))
                .expect("chmod");
        }

        let mut env = HashMap::new();
        env.insert(
            "BIOMEOS_PLASMID_BIN_DIR".to_string(),
            temp.path().to_string_lossy().to_string(),
        );
        let ctx = ExecutionContext::new(env);

        let result = discover_primal_binary("squirrel", &ctx)
            .await
            .expect("Should find arch-specific squirrel");

        assert!(result.exists());
        assert_eq!(result.file_name().unwrap(), "squirrel");
    }

    #[tokio::test]
    async fn test_discover_primal_binary_not_found() {
        let temp = tempfile::tempdir().expect("temp dir");
        let mut env = HashMap::new();
        env.insert(
            "BIOMEOS_PLASMID_BIN_DIR".to_string(),
            temp.path().to_string_lossy().to_string(),
        );
        let ctx = ExecutionContext::new(env);

        let result = discover_primal_binary("nonexistent_primal", &ctx).await;

        let err = result.expect_err("Should fail when binary not found");
        assert!(
            err.to_string().contains("Binary not found"),
            "Error should mention binary: {err}"
        );
        assert!(
            err.to_string().contains("nonexistent_primal"),
            "Error should mention primal name"
        );
    }

    #[tokio::test]
    async fn test_discover_primal_binary_empty_dir() {
        let temp = tempfile::tempdir().expect("temp dir");

        let mut env = HashMap::new();
        env.insert(
            "BIOMEOS_PLASMID_BIN_DIR".to_string(),
            temp.path().to_string_lossy().to_string(),
        );
        let ctx = ExecutionContext::new(env);

        let result = discover_primal_binary("beardog", &ctx).await;

        let err = result.expect_err("Empty dir should not find beardog");
        assert!(err.to_string().contains("Binary not found"));
    }

    #[tokio::test]
    async fn test_discover_primal_binary_prefers_env_over_default_paths() {
        let temp = tempfile::tempdir().expect("temp dir");
        let bin_path = temp.path().join("nestgate");
        std::fs::write(&bin_path, "x").expect("write");
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(&bin_path, std::fs::Permissions::from_mode(0o755))
                .expect("chmod");
        }

        let mut env = HashMap::new();
        env.insert(
            "BIOMEOS_PLASMID_BIN_DIR".to_string(),
            temp.path().to_string_lossy().to_string(),
        );
        let ctx = ExecutionContext::new(env);

        let result = discover_primal_binary("nestgate", &ctx)
            .await
            .expect("Should find in env dir");

        assert!(result.starts_with(temp.path()));
    }

    // -------------------------------------------------------------------------
    // Path construction and type serialization
    // -------------------------------------------------------------------------

    #[test]
    fn test_binary_discovery_path_construction() {
        let arch = std::env::consts::ARCH;
        let os = std::env::consts::OS;
        let target = format!("{arch}-{os}");

        assert!(target.contains(arch));
        assert!(target.contains(os));
    }

    #[test]
    fn test_plasmid_bin_paths() {
        let paths = [
            PathBuf::from("./plasmidBin"),
            PathBuf::from("../plasmidBin"),
            PathBuf::from("../../plasmidBin"),
        ];

        for path in paths {
            assert!(path.to_string_lossy().contains("plasmidBin"));
        }
    }
}
