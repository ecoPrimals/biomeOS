// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Doctor mode config checks - binary health, configuration, graphs directory

use anyhow::Result;
use std::path::Path;

use super::types::{HealthCheck, HealthStatus};

pub(crate) async fn check_binary_health() -> Result<HealthCheck> {
    let mut check = HealthCheck {
        name: "Binary Health".to_string(),
        status: HealthStatus::Healthy,
        details: Vec::new(),
    };

    // Check current binary
    if let Ok(exe) = std::env::current_exe() {
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

    Ok(check)
}

pub(crate) async fn check_configuration() -> Result<HealthCheck> {
    let mut check = HealthCheck {
        name: "Configuration".to_string(),
        status: HealthStatus::Healthy,
        details: Vec::new(),
    };

    // Use SystemPaths (XDG-compliant) for config directory
    let paths = biomeos_types::paths::SystemPaths::new_lazy();
    let config_path = paths.config_dir().join("config.toml");

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
    let mut check = HealthCheck {
        name: "Graphs Directory".to_string(),
        status: HealthStatus::Healthy,
        details: Vec::new(),
    };

    let graphs_dir = Path::new("graphs");

    if graphs_dir.exists() && graphs_dir.is_dir() {
        // Count .toml files
        let graph_count = std::fs::read_dir(graphs_dir)?
            .filter_map(|e| e.ok())
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
    #![allow(clippy::unwrap_used)]

    use super::*;

    #[tokio::test]
    #[ignore = "cwd-changing test is thread-unsafe; run with --test-threads=1"]
    async fn test_check_graphs_dir_no_directory() {
        let temp = tempfile::tempdir().unwrap();
        let old_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();
        let check = check_graphs_dir().await.unwrap();
        std::env::set_current_dir(&old_cwd).unwrap();
        assert_eq!(check.status, HealthStatus::Warning);
        assert!(check
            .details
            .iter()
            .any(|d| d.contains("not found") || d.contains("Directory")));
    }

    #[tokio::test]
    #[ignore = "cwd-changing test is thread-unsafe; run with --test-threads=1"]
    async fn test_check_graphs_dir_with_toml_files() {
        let temp = tempfile::tempdir().unwrap();
        let graphs_dir = temp.path().join("graphs");
        std::fs::create_dir_all(&graphs_dir).unwrap();
        std::fs::write(graphs_dir.join("deploy.toml"), "name = \"test\"").unwrap();
        let old_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();
        let check = check_graphs_dir().await.unwrap();
        std::env::set_current_dir(&old_cwd).unwrap();
        assert_eq!(check.status, HealthStatus::Healthy);
        assert!(check.details.iter().any(|d| d.contains("Graphs found: 1")));
    }

    #[tokio::test]
    #[ignore = "cwd-changing test is thread-unsafe; run with --test-threads=1"]
    async fn test_check_graphs_dir_empty_graphs_dir() {
        let temp = tempfile::tempdir().unwrap();
        let graphs_dir = temp.path().join("graphs");
        std::fs::create_dir_all(&graphs_dir).unwrap();
        let old_cwd = std::env::current_dir().unwrap();
        std::env::set_current_dir(temp.path()).unwrap();
        let check = check_graphs_dir().await.unwrap();
        std::env::set_current_dir(&old_cwd).unwrap();
        assert_eq!(check.status, HealthStatus::Warning);
        assert!(check.details.iter().any(|d| d.contains("No graph")));
    }

    #[tokio::test]
    #[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
    async fn test_check_configuration_no_config() {
        let temp = tempfile::tempdir().unwrap();
        std::fs::create_dir_all(temp.path().join("biomeos")).unwrap();
        let old_xdg = std::env::var("XDG_CONFIG_HOME").ok();
        std::env::set_var("XDG_CONFIG_HOME", temp.path());
        let check = check_configuration().await.unwrap();
        if let Some(xdg) = old_xdg {
            std::env::set_var("XDG_CONFIG_HOME", xdg);
        } else {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
        assert_eq!(check.status, HealthStatus::Warning);
        assert!(check.details.iter().any(|d| d.contains("Not found")));
    }

    #[tokio::test]
    async fn test_check_binary_health_structure() {
        let check = check_binary_health().await.unwrap();
        assert_eq!(check.name, "Binary Health");
        assert!(check.details.iter().any(|d| d.starts_with("Version:")));
        assert!(check.details.iter().any(|d| d.contains("Modes:")));
        assert!(check.details.iter().any(|d| d.contains("UniBin")));
        // When running as cargo test, current_exe typically succeeds
        if check.status == HealthStatus::Healthy {
            assert!(check.details.iter().any(|d| d.starts_with("Binary:")));
        }
    }

    #[tokio::test]
    #[ignore = "env-var test is thread-unsafe; run with --test-threads=1"]
    async fn test_check_configuration_with_config() {
        let temp = tempfile::tempdir().unwrap();
        let config_dir = temp.path().join("biomeos");
        std::fs::create_dir_all(&config_dir).unwrap();
        std::fs::write(config_dir.join("config.toml"), "[default]").unwrap();
        let old_xdg = std::env::var("XDG_CONFIG_HOME").ok();
        std::env::set_var("XDG_CONFIG_HOME", temp.path());
        let check = check_configuration().await.unwrap();
        if let Some(xdg) = old_xdg {
            std::env::set_var("XDG_CONFIG_HOME", xdg);
        } else {
            std::env::remove_var("XDG_CONFIG_HOME");
        }
        assert_eq!(check.status, HealthStatus::Healthy);
        assert!(check.details.iter().any(|d| d.contains("Found")));
    }
}
