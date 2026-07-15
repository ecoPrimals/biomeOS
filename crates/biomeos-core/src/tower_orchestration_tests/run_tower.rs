// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::mock_env;
use super::super::run_tower;
use std::collections::HashMap;

#[tokio::test]
async fn run_tower_returns_ok_when_no_primals_configured() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("tower.toml");
    let env: HashMap<String, String> = HashMap::new();
    let result = run_tower(&config_path, None, false, &mock_env(&env)).await;
    assert!(result.is_ok(), "early exit with no primals should succeed");
}

#[tokio::test]
async fn run_tower_with_existing_empty_config_no_primals() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("tower.toml");
    std::fs::write(&config_path, "[tower]\nname = \"test-tower\"\n").unwrap();
    let env: HashMap<String, String> = HashMap::new();
    let result = run_tower(&config_path, None, false, &mock_env(&env)).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn run_tower_with_empty_scan_dir_no_primals() {
    let dir = tempfile::tempdir().unwrap();
    let config_path = dir.path().join("tower.toml");
    let scan = tempfile::tempdir().unwrap();
    let env: HashMap<String, String> = HashMap::new();
    let result = run_tower(
        &config_path,
        Some(scan.path().to_path_buf()),
        false,
        &mock_env(&env),
    )
    .await;
    assert!(result.is_ok());
}
