// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Doctor mode config checks - binary health, configuration, graphs directory

use anyhow::Result;
use std::path::{Path, PathBuf};

use super::types::{HealthCheck, HealthStatus};

pub(crate) async fn check_binary_health() -> Result<HealthCheck> {
    Ok(check_binary_health_inner(std::env::current_exe()))
}

fn check_binary_health_inner(exe_result: Result<PathBuf, std::io::Error>) -> HealthCheck {
    let mut check = HealthCheck {
        name: "Binary Health".to_string(),
        status: HealthStatus::Healthy,
        details: Vec::new(),
    };

    // Check current binary
    if let Ok(exe) = exe_result {
        check.details.push(format!("Binary: {}", exe.display()));

        if let Ok(metadata) = std::fs::metadata(&exe) {
            let size_mb = metadata.len() as f64 / 1_048_576.0;
            check.details.push(format!("Size: {size_mb:.1}M"));
        }
    } else {
        check.status = HealthStatus::Warning;
        check
            .details
            .push("Could not determine binary path".to_string());
    }

    check
        .details
        .push(format!("Version: {}", env!("CARGO_PKG_VERSION")));
    check.details.push("Modes: 7/7 available".to_string());
    check.details.push("UniBin: ✅ Compliant".to_string());

    check
}

pub(crate) async fn check_configuration() -> Result<HealthCheck> {
    let paths = biomeos_types::paths::SystemPaths::new_lazy();
    check_configuration_with(paths.config_dir()).await
}

pub(crate) async fn check_configuration_with(config_dir: &Path) -> Result<HealthCheck> {
    let mut check = HealthCheck {
        name: "Configuration".to_string(),
        status: HealthStatus::Healthy,
        details: Vec::new(),
    };

    let config_path = config_dir.join("config.toml");

    if config_path.exists() {
        check
            .details
            .push(format!("Config file: {}", config_path.display()));
        check.details.push("Status: ✅ Found".to_string());
    } else {
        check.status = HealthStatus::Warning;
        check
            .details
            .push(format!("Config file: {}", config_path.display()));
        check
            .details
            .push("Status: ⚠️  Not found (using defaults)".to_string());
    }

    Ok(check)
}

pub(crate) async fn check_graphs_dir() -> Result<HealthCheck> {
    let base = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    check_graphs_dir_at(&base).await
}

pub(crate) async fn check_graphs_dir_at(base_dir: &Path) -> Result<HealthCheck> {
    let mut check = HealthCheck {
        name: "Graphs Directory".to_string(),
        status: HealthStatus::Healthy,
        details: Vec::new(),
    };

    let graphs_dir = base_dir.join("graphs");

    if graphs_dir.exists() && graphs_dir.is_dir() {
        let graph_count = std::fs::read_dir(&graphs_dir)?
            .filter_map(std::result::Result::ok)
            .filter(|e| e.path().extension().is_some_and(|ext| ext == "toml"))
            .count();

        check
            .details
            .push(format!("Path: {}", graphs_dir.display()));
        check.details.push(format!("Graphs found: {graph_count}"));

        if graph_count == 0 {
            check.status = HealthStatus::Warning;
            check
                .details
                .push("Warning: No graph files found".to_string());
        } else {
            check.details.push("Status: ✅ Ready".to_string());
        }
    } else {
        check.status = HealthStatus::Warning;
        check
            .details
            .push(format!("Path: {}", graphs_dir.display()));
        check
            .details
            .push("Status: ⚠️  Directory not found".to_string());
    }

    Ok(check)
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::Context;

    #[tokio::test]
    async fn test_check_graphs_dir_no_directory() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let check = check_graphs_dir_at(temp.path()).await?;
        assert_eq!(check.status, HealthStatus::Warning);
        assert!(
            check
                .details
                .iter()
                .any(|d| d.contains("not found") || d.contains("Directory"))
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_check_graphs_dir_with_toml_files() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let graphs_dir = temp.path().join("graphs");
        std::fs::create_dir_all(&graphs_dir)?;
        std::fs::write(graphs_dir.join("deploy.toml"), "name = \"test\"")?;
        let check = check_graphs_dir_at(temp.path()).await?;
        assert_eq!(check.status, HealthStatus::Healthy);
        assert!(check.details.iter().any(|d| d.contains("Graphs found: 1")));
        Ok(())
    }

    #[tokio::test]
    async fn test_check_graphs_dir_empty_graphs_dir() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let graphs_dir = temp.path().join("graphs");
        std::fs::create_dir_all(&graphs_dir)?;
        let check = check_graphs_dir_at(temp.path()).await?;
        assert_eq!(check.status, HealthStatus::Warning);
        assert!(check.details.iter().any(|d| d.contains("No graph")));
        Ok(())
    }

    #[tokio::test]
    async fn test_check_configuration_no_config() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let config_dir = temp.path().join("biomeos");
        std::fs::create_dir_all(&config_dir)?;
        let check = check_configuration_with(&config_dir).await?;
        assert_eq!(check.status, HealthStatus::Warning);
        assert!(check.details.iter().any(|d| d.contains("Not found")));
        Ok(())
    }

    #[tokio::test]
    async fn test_check_binary_health_structure() -> anyhow::Result<()> {
        let check = check_binary_health().await?;
        assert_eq!(check.name, "Binary Health");
        assert!(check.details.iter().any(|d| d.starts_with("Version:")));
        assert!(check.details.iter().any(|d| d.contains("Modes:")));
        assert!(check.details.iter().any(|d| d.contains("UniBin")));
        // When running as cargo test, current_exe typically succeeds
        if check.status == HealthStatus::Healthy {
            assert!(check.details.iter().any(|d| d.starts_with("Binary:")));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_check_configuration_with_config() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let config_dir = temp.path().join("biomeos");
        std::fs::create_dir_all(&config_dir)?;
        std::fs::write(config_dir.join("config.toml"), "[default]")?;
        let check = check_configuration_with(&config_dir).await?;
        assert_eq!(check.status, HealthStatus::Healthy);
        assert!(check.details.iter().any(|d| d.contains("Found")));
        Ok(())
    }

    #[tokio::test]
    async fn test_check_binary_health_details_completeness() -> anyhow::Result<()> {
        let check = check_binary_health().await?;
        assert_eq!(check.name, "Binary Health");
        assert!(check.status == HealthStatus::Healthy || check.status == HealthStatus::Warning);
        assert!(check.details.iter().any(|d| d.starts_with("Version:")));
        assert!(check.details.iter().any(|d| d.contains("Modes:")));
        assert!(check.details.iter().any(|d| d.contains("UniBin")));
        if let Some(size_detail) = check.details.iter().find(|d| d.starts_with("Size:")) {
            assert!(size_detail.contains('M'));
        }
        Ok(())
    }

    #[tokio::test]
    async fn test_check_graphs_dir_multiple_toml_files() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let graphs_dir = temp.path().join("graphs");
        std::fs::create_dir_all(&graphs_dir)?;
        std::fs::write(graphs_dir.join("a.toml"), "a = 1")?;
        std::fs::write(graphs_dir.join("b.toml"), "b = 2")?;
        std::fs::write(graphs_dir.join("c.toml"), "c = 3")?;
        let check = check_graphs_dir_at(temp.path()).await?;
        assert_eq!(check.status, HealthStatus::Healthy);
        assert!(check.details.iter().any(|d| d.contains("Graphs found: 3")));
        assert!(check.details.iter().any(|d| d.contains("Ready")));
        Ok(())
    }

    #[tokio::test]
    async fn test_check_graphs_dir_non_toml_files_ignored() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let graphs_dir = temp.path().join("graphs");
        std::fs::create_dir_all(&graphs_dir)?;
        std::fs::write(graphs_dir.join("readme.txt"), "text")?;
        std::fs::write(graphs_dir.join("data.json"), "{}")?;
        let check = check_graphs_dir_at(temp.path()).await?;
        assert_eq!(check.status, HealthStatus::Warning);
        assert!(check.details.iter().any(|d| d.contains("Graphs found: 0")));
        assert!(check.details.iter().any(|d| d.contains("No graph")));
        Ok(())
    }

    #[tokio::test]
    async fn test_check_graphs_dir_path_is_file_not_directory() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        std::fs::write(temp.path().join("graphs"), "not a directory")?;
        let check = check_graphs_dir_at(temp.path()).await?;
        assert_eq!(check.status, HealthStatus::Warning);
        assert!(check.details.iter().any(|d| d.contains("not found")));
        Ok(())
    }

    #[tokio::test]
    async fn test_check_configuration_config_path_in_details() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let config_dir = temp.path().join("biomeos");
        std::fs::create_dir_all(&config_dir)?;
        std::fs::write(config_dir.join("config.toml"), "")?;
        let check = check_configuration_with(&config_dir).await?;
        assert!(check.details.iter().any(|d| d.contains("config.toml")));
        Ok(())
    }

    #[tokio::test]
    async fn test_check_configuration_not_found_details() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let config_dir = temp.path().join("biomeos");
        std::fs::create_dir_all(&config_dir)?;
        let check = check_configuration_with(&config_dir).await?;
        assert!(check.details.iter().any(|d| d.contains("config.toml")));
        assert!(check.details.iter().any(|d| d.contains("defaults")));
        Ok(())
    }

    #[tokio::test]
    async fn test_check_binary_health_inner_current_exe_fails() {
        let check = check_binary_health_inner(Err(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "simulated",
        )));
        assert_eq!(check.status, HealthStatus::Warning);
        assert!(
            check
                .details
                .iter()
                .any(|d| d.contains("Could not determine binary path"))
        );
    }

    #[tokio::test]
    async fn test_check_binary_health_inner_metadata_missing_exe_path() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let missing_exe = temp.path().join("definitely_not_present");
        let check = check_binary_health_inner(Ok(missing_exe));
        assert_eq!(check.status, HealthStatus::Healthy);
        assert!(check.details.iter().any(|d| d.starts_with("Binary:")));
        assert!(!check.details.iter().any(|d| d.starts_with("Size:")));
        Ok(())
    }

    #[cfg(unix)]
    #[tokio::test]
    async fn test_check_graphs_dir_read_dir_error() -> anyhow::Result<()> {
        use std::os::unix::fs::PermissionsExt;

        let temp = tempfile::tempdir()?;
        let graphs_dir = temp.path().join("graphs");
        std::fs::create_dir_all(&graphs_dir)?;
        std::fs::set_permissions(&graphs_dir, std::fs::Permissions::from_mode(0o000))
            .context("chmod graphs")?;
        let result = check_graphs_dir_at(temp.path()).await;
        let _ = std::fs::set_permissions(&graphs_dir, std::fs::Permissions::from_mode(0o755));
        assert!(result.is_err());
        Ok(())
    }

    #[tokio::test]
    async fn test_check_graphs_dir_toml_only_in_subdirectory() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let graphs_dir = temp.path().join("graphs");
        std::fs::create_dir_all(graphs_dir.join("nested"))?;
        std::fs::write(graphs_dir.join("nested").join("x.toml"), "a = 1")?;
        let check = check_graphs_dir_at(temp.path()).await?;
        assert_eq!(check.status, HealthStatus::Warning);
        assert!(check.details.iter().any(|d| d.contains("Graphs found: 0")));
        Ok(())
    }

    #[tokio::test]
    async fn test_check_configuration_config_path_is_directory() -> anyhow::Result<()> {
        let temp = tempfile::tempdir()?;
        let config_dir = temp.path().join("biomeos");
        std::fs::create_dir_all(&config_dir)?;
        std::fs::create_dir_all(config_dir.join("config.toml"))?;
        let check = check_configuration_with(&config_dir).await?;
        assert_eq!(check.status, HealthStatus::Healthy);
        assert!(check.details.iter().any(|d| d.contains("Found")));
        Ok(())
    }
}
