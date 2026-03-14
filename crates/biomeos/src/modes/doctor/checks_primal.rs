// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Doctor mode primal checks - primal discovery, plasmidBin

use anyhow::Result;
use std::path::Path;

use super::types::{HealthCheck, HealthStatus};

pub(crate) async fn check_primal_discovery() -> Result<HealthCheck> {
    let mut check = HealthCheck {
        name: "Primal Discovery".to_string(),
        status: HealthStatus::Healthy,
        details: Vec::new(),
    };

    // Get family_id from environment or use default
    // Uses capability-based discovery pattern (no hardcoded paths)
    let family_id = biomeos_core::family_discovery::get_family_id();

    // Use XDG-compliant SystemPaths for socket directory
    let paths = biomeos_types::paths::SystemPaths::new_lazy();
    let runtime_dir = paths.runtime_dir();

    let health_checker =
        biomeos_atomic_deploy::health_check::HealthChecker::new(runtime_dir.to_path_buf());

    let primals = biomeos_types::primal_names::CORE_PRIMALS;

    check
        .details
        .push(format!("Socket dir: {}", runtime_dir.display()));
    check.details.push(format!("Family ID: {family_id}"));

    let mut found_count = 0;
    for primal_name in primals {
        // Use family-suffixed socket naming convention
        let socket_path = runtime_dir.join(format!("{primal_name}-{family_id}.sock"));

        match health_checker.check_primal(&socket_path).await {
            Ok(status) if status.is_healthy => {
                found_count += 1;
                check.details.push(format!(
                    "{}: ✅ Healthy ({})",
                    primal_name,
                    socket_path.display()
                ));
            }
            Ok(status) => {
                let msg = status.message.unwrap_or_else(|| "Not found".to_string());
                check.details.push(format!("{primal_name}: ❌ {msg}"));
            }
            Err(e) => {
                check.details.push(format!("{primal_name}: ❌ Error: {e}"));
            }
        }
    }

    check
        .details
        .push(format!("Total: {found_count}/5 primals discovered"));

    if found_count < 3 {
        check.status = HealthStatus::Warning;
    }

    Ok(check)
}

pub(crate) async fn check_plasmid_bin() -> Result<HealthCheck> {
    let mut check = HealthCheck {
        name: "PlasmidBin".to_string(),
        status: HealthStatus::Healthy,
        details: Vec::new(),
    };

    let plasmid_dir = Path::new("plasmidBin/primals");

    if plasmid_dir.exists() && plasmid_dir.is_dir() {
        let binaries: Vec<_> = std::fs::read_dir(plasmid_dir)?
            .filter_map(|e| e.ok())
            .filter(|e| e.path().is_file())
            .collect();

        let total_size: u64 = binaries
            .iter()
            .filter_map(|e| e.metadata().ok())
            .map(|m| m.len())
            .sum();

        let size_mb = total_size as f64 / 1_048_576.0;

        check
            .details
            .push(format!("Path: {}", plasmid_dir.display()));
        check.details.push(format!("Binaries: {}", binaries.len()));
        check.details.push(format!("Total size: {size_mb:.1}M"));
        check.details.push("Status: ✅ Ready".to_string());
    } else {
        check.status = HealthStatus::Warning;
        check
            .details
            .push(format!("Path: {}", plasmid_dir.display()));
        check
            .details
            .push("Status: ⚠️  Directory not found".to_string());
    }

    Ok(check)
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[tokio::test]
    #[ignore = "cwd-changing test is thread-unsafe; run with --test-threads=1"]
    async fn test_check_plasmid_bin_no_directory() {
        let temp = tempfile::tempdir().unwrap();
        let old_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();
        let check = check_plasmid_bin().await.unwrap();
        std::env::set_current_dir(&old_cwd).unwrap();
        assert_eq!(check.status, HealthStatus::Warning);
        assert!(check
            .details
            .iter()
            .any(|d| d.contains("not found") || d.contains("Directory")));
    }

    #[tokio::test]
    #[ignore = "cwd-changing test is thread-unsafe; run with --test-threads=1"]
    async fn test_check_plasmid_bin_with_binaries() {
        let temp = tempfile::tempdir().unwrap();
        let plasmid_dir = temp.path().join("plasmidBin").join("primals");
        std::fs::create_dir_all(&plasmid_dir).unwrap();
        std::fs::write(plasmid_dir.join("beardog"), "fake-binary").unwrap();
        let old_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();
        let check = check_plasmid_bin().await.unwrap();
        std::env::set_current_dir(&old_cwd).unwrap();
        assert_eq!(check.status, HealthStatus::Healthy);
        assert!(check.details.iter().any(|d| d.contains("Binaries: 1")));
    }
}
