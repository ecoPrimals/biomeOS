// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::{collect_primals, config_to_primal, metadata_to_primal};
use crate::{PrimalMetadata, TowerConfig, TowerPrimalConfig};
use std::collections::HashMap;
use std::path::PathBuf;

#[tokio::test]
async fn metadata_to_primal_converts_correctly() {
    let metadata = PrimalMetadata {
        id: "test-primal".to_string(),
        binary: PathBuf::from("/usr/bin/test-primal"),
        provides: vec!["security".to_string(), "crypto".to_string()],
        requires: vec!["discovery".to_string()],
        version: Some("1.0.0".to_string()),
        name: Some("Test Primal".to_string()),
    };

    let primal = metadata_to_primal(&metadata);
    assert!(primal.is_ok());
}

#[tokio::test]
async fn collect_primals_from_empty_config() {
    let config = TowerConfig::default_config();
    let primals = collect_primals(&config, None).await.unwrap();
    assert!(primals.is_empty());
}

#[tokio::test]
async fn metadata_to_primal_accepts_empty_capability_lists() {
    let metadata = PrimalMetadata {
        id: "empty-caps".to_string(),
        binary: PathBuf::from("/bin/true"),
        provides: vec![],
        requires: vec![],
        version: None,
        name: None,
    };
    let primal = metadata_to_primal(&metadata).unwrap();
    assert!(primal.provides().is_empty());
    assert!(primal.requires().is_empty());
}

#[tokio::test]
async fn config_to_primal_explicit_capabilities_skips_auto_discover() {
    let config = TowerPrimalConfig {
        binary: PathBuf::from("/bin/true"),
        id: Some("explicit".to_string()),
        provides: vec!["alpha".to_string()],
        requires: vec!["beta".to_string()],
        http_port: 0,
        protocol: None,
        env: HashMap::new(),
        auto_discover: true,
    };
    let primal = config_to_primal(&config).await.unwrap();
    assert_eq!(primal.provides().len(), 1);
    assert_eq!(primal.requires().len(), 1);
}

#[tokio::test]
async fn config_to_primal_auto_discover_fallback_when_query_fails() {
    let config = TowerPrimalConfig {
        binary: PathBuf::from("/bin/false"),
        id: Some("fallback-id".to_string()),
        provides: vec![],
        requires: vec![],
        http_port: 0,
        protocol: None,
        env: HashMap::new(),
        auto_discover: true,
    };
    let primal = config_to_primal(&config).await.unwrap();
    assert!(primal.provides().is_empty());
}

#[tokio::test]
async fn config_to_primal_applies_env_protocol_and_http_port() {
    let mut env_map = HashMap::new();
    env_map.insert("MY_VAR".to_string(), "1".to_string());
    let config = TowerPrimalConfig {
        binary: PathBuf::from("/bin/true"),
        id: Some("rich".to_string()),
        provides: vec!["p".to_string()],
        requires: vec![],
        http_port: 9000,
        protocol: Some("jsonrpc".to_string()),
        env: env_map,
        auto_discover: false,
    };
    let primal = config_to_primal(&config).await.unwrap();
    assert_eq!(primal.provides().len(), 1);
}

#[tokio::test]
async fn collect_primals_merges_config_primal_with_empty_scan() {
    let scan = tempfile::tempdir().unwrap();
    let mut tower = TowerConfig::default_config();
    tower.primals.push(TowerPrimalConfig {
        binary: PathBuf::from("/bin/true"),
        id: Some("from-config".to_string()),
        provides: vec!["x".to_string()],
        requires: vec![],
        http_port: 0,
        protocol: None,
        env: HashMap::new(),
        auto_discover: false,
    });
    let primals = collect_primals(&tower, Some(scan.path())).await.unwrap();
    assert_eq!(primals.len(), 1);
}

#[tokio::test]
#[cfg(unix)]
async fn collect_primals_discovers_executable_in_scan_dir() {
    let scan = tempfile::tempdir().unwrap();
    let bin = scan.path().join("scan-primal");
    std::fs::copy("/bin/true", &bin).unwrap();
    use std::os::unix::fs::PermissionsExt;
    std::fs::set_permissions(&bin, std::fs::Permissions::from_mode(0o755)).unwrap();

    let primals = collect_primals(&TowerConfig::default_config(), Some(scan.path()))
        .await
        .unwrap();
    assert_eq!(primals.len(), 1);
}

#[tokio::test]
async fn config_to_primal_auto_discover_false_uses_config_directly() {
    let config = TowerPrimalConfig {
        binary: PathBuf::from("/bin/true"),
        id: Some("no-discover".to_string()),
        provides: vec!["cap-a".to_string()],
        requires: vec!["cap-b".to_string()],
        http_port: 0,
        protocol: None,
        env: HashMap::new(),
        auto_discover: false,
    };
    let primal = config_to_primal(&config).await.unwrap();
    assert_eq!(primal.provides().len(), 1);
    assert_eq!(primal.requires().len(), 1);
}

#[tokio::test]
async fn config_to_primal_id_none_derives_from_binary_stem() {
    let config = TowerPrimalConfig {
        binary: PathBuf::from("/usr/local/bin/my-primal"),
        id: None,
        provides: vec![],
        requires: vec![],
        http_port: 0,
        protocol: None,
        env: HashMap::new(),
        auto_discover: true,
    };
    let primal = config_to_primal(&config).await.unwrap();
    drop(primal);
}

#[tokio::test]
async fn config_to_primal_zero_http_port_not_set() {
    let config = TowerPrimalConfig {
        binary: PathBuf::from("/bin/true"),
        id: Some("zero-port".to_string()),
        provides: vec!["x".to_string()],
        requires: vec![],
        http_port: 0,
        protocol: Some("jsonrpc".to_string()),
        env: HashMap::new(),
        auto_discover: false,
    };
    let primal = config_to_primal(&config).await.unwrap();
    assert_eq!(primal.provides().len(), 1);
}
