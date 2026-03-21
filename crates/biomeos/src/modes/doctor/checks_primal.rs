// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Doctor mode primal checks - primal discovery, plasmidBin

use anyhow::Result;
use std::path::{Path, PathBuf};

use super::types::{HealthCheck, HealthStatus};

pub(crate) async fn check_primal_discovery() -> Result<HealthCheck> {
    let family_id = biomeos_core::family_discovery::get_family_id();
    let runtime_dir = biomeos_types::paths::SystemPaths::new_lazy()
        .runtime_dir()
        .to_path_buf();
    check_primal_discovery_with(&runtime_dir, &family_id).await
}

pub(crate) async fn check_primal_discovery_with(
    runtime_dir: &Path,
    family_id: &str,
) -> Result<HealthCheck> {
    let mut check = HealthCheck {
        name: "Primal Discovery".to_string(),
        status: HealthStatus::Healthy,
        details: Vec::new(),
    };

    let health_checker =
        biomeos_atomic_deploy::health_check::HealthChecker::new(runtime_dir.to_path_buf());

    let primals = biomeos_types::primal_names::CORE_PRIMALS;

    check
        .details
        .push(format!("Socket dir: {}", runtime_dir.display()));
    check.details.push(format!("Family ID: {family_id}"));

    let mut found_count = 0;
    for primal_name in primals {
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
    let base = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    check_plasmid_bin_at(&base).await
}

pub(crate) async fn check_plasmid_bin_at(base_dir: &Path) -> Result<HealthCheck> {
    let mut check = HealthCheck {
        name: "PlasmidBin".to_string(),
        status: HealthStatus::Healthy,
        details: Vec::new(),
    };

    let plasmid_dir = base_dir.join("plasmidBin").join("primals");

    if plasmid_dir.exists() && plasmid_dir.is_dir() {
        let binaries: Vec<_> = std::fs::read_dir(&plasmid_dir)?
            .filter_map(std::result::Result::ok)
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
    #![allow(clippy::unwrap_used, clippy::expect_used)]

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
        assert!(
            check
                .details
                .iter()
                .any(|d| d.contains("not found") || d.contains("Directory"))
        );
    }

    #[tokio::test]
    async fn test_check_primal_discovery_structure() {
        let check = check_primal_discovery().await.unwrap();
        assert_eq!(check.name, "Primal Discovery");
        assert!(check.details.iter().any(|d| d.starts_with("Socket dir:")));
        assert!(check.details.iter().any(|d| d.starts_with("Family ID:")));
        assert!(
            check
                .details
                .iter()
                .any(|d| d.contains("primals discovered"))
        );
        // Status may be Healthy or Warning depending on running primals
        assert!(matches!(
            check.status,
            HealthStatus::Healthy | HealthStatus::Warning
        ));
    }

    #[tokio::test]
    async fn test_check_plasmid_bin_with_binaries() {
        let temp = tempfile::tempdir().unwrap();
        let plasmid_dir = temp.path().join("plasmidBin").join("primals");
        std::fs::create_dir_all(&plasmid_dir).unwrap();
        std::fs::write(plasmid_dir.join("beardog"), "fake-binary").unwrap();
        let check = check_plasmid_bin_at(temp.path()).await.unwrap();
        assert_eq!(check.status, HealthStatus::Healthy);
        assert!(check.details.iter().any(|d| d.contains("Binaries: 1")));
    }

    #[tokio::test]
    async fn test_check_plasmid_bin_path_is_file_not_dir() {
        let temp = tempfile::tempdir().unwrap();
        let plasmid_parent = temp.path().join("plasmidBin");
        std::fs::create_dir_all(&plasmid_parent).unwrap();
        std::fs::write(plasmid_parent.join("primals"), "not-a-dir").unwrap();
        let check = check_plasmid_bin_at(temp.path()).await.unwrap();
        assert_eq!(check.status, HealthStatus::Warning);
        assert!(
            check
                .details
                .iter()
                .any(|d| d.contains("not found") || d.contains("Directory"))
        );
    }

    #[tokio::test]
    async fn test_check_plasmid_bin_empty_directory() {
        let temp = tempfile::tempdir().unwrap();
        let plasmid_dir = temp.path().join("plasmidBin").join("primals");
        std::fs::create_dir_all(&plasmid_dir).unwrap();
        let check = check_plasmid_bin_at(temp.path()).await.unwrap();
        assert_eq!(check.status, HealthStatus::Healthy);
        assert!(check.details.iter().any(|d| d.contains("Binaries: 0")));
        assert!(check.details.iter().any(|d| d.contains("Ready")));
    }

    #[tokio::test]
    async fn test_check_plasmid_bin_multiple_binaries_and_size() {
        let temp = tempfile::tempdir().unwrap();
        let plasmid_dir = temp.path().join("plasmidBin").join("primals");
        std::fs::create_dir_all(&plasmid_dir).unwrap();
        let content = vec![0u8; 2 * 1024 * 1024];
        std::fs::write(plasmid_dir.join("beardog"), &content).unwrap();
        std::fs::write(plasmid_dir.join("songbird"), b"small").unwrap();
        std::fs::create_dir(plasmid_dir.join("subdir")).unwrap();
        let check = check_plasmid_bin_at(temp.path()).await.unwrap();
        assert_eq!(check.status, HealthStatus::Healthy);
        assert!(check.details.iter().any(|d| d.contains("Binaries: 2")));
        assert!(check.details.iter().any(|d| d.contains("Total size:")));
    }

    #[tokio::test]
    async fn test_check_primal_discovery_with_healthy_sockets() {
        let temp = tempfile::tempdir().unwrap();
        let runtime_dir = temp.path().join("xdg-runtime").join("biomeos");
        std::fs::create_dir_all(&runtime_dir).unwrap();

        let primals = biomeos_types::primal_names::CORE_PRIMALS;
        for primal_name in primals.iter().take(4) {
            let socket_path = runtime_dir.join(format!("{primal_name}-test.sock"));
            let _listener = std::os::unix::net::UnixListener::bind(&socket_path).expect("bind");
        }

        let check = check_primal_discovery_with(&runtime_dir, "test")
            .await
            .unwrap();

        assert_eq!(check.name, "Primal Discovery");
        assert!(check.details.iter().any(|d| d.contains("Healthy")));
        assert!(
            check
                .details
                .iter()
                .any(|d| d.contains("primals discovered"))
        );
        assert_eq!(check.status, HealthStatus::Healthy);
    }

    #[tokio::test]
    async fn test_check_primal_discovery_warning_when_few_found() {
        let temp = tempfile::tempdir().unwrap();
        let runtime_dir = temp.path().join("xdg-runtime").join("biomeos");
        std::fs::create_dir_all(&runtime_dir).unwrap();

        let socket_path = runtime_dir.join("beardog-test.sock");
        let _listener = std::os::unix::net::UnixListener::bind(&socket_path).expect("bind");

        let check = check_primal_discovery_with(&runtime_dir, "test")
            .await
            .unwrap();

        assert_eq!(check.status, HealthStatus::Warning);
        assert!(
            check
                .details
                .iter()
                .any(|d| d.contains("1/5") || d.contains("2/5"))
        );
    }

    #[tokio::test]
    async fn test_check_primal_discovery_details_contain_all_primals() {
        let check = check_primal_discovery().await.unwrap();
        let primals = biomeos_types::primal_names::CORE_PRIMALS;
        for primal_name in primals {
            assert!(
                check.details.iter().any(|d| d.starts_with(primal_name)),
                "expected detail for primal {primal_name}"
            );
        }
    }

    #[tokio::test]
    async fn test_check_primal_discovery_total_format() {
        let check = check_primal_discovery().await.unwrap();
        let total_detail = check
            .details
            .iter()
            .find(|d| d.contains("primals discovered"))
            .expect("total line");
        assert!(total_detail.contains("/5"));
    }
}
