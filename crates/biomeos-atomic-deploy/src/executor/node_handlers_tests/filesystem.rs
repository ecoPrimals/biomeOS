// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::{test_context, test_context_with_env, test_node_with_config};
use super::super::filesystem_check_exists;
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_filesystem_check_exists_present() {
    let temp_dir = tempfile::tempdir().unwrap();
    let test_file = temp_dir.path().join("test_seed.bin");
    std::fs::write(&test_file, b"seed data").unwrap();

    let node = test_node_with_config("fs1", {
        let mut c = HashMap::new();
        c.insert(
            "path".to_string(),
            json!(test_file.to_string_lossy().to_string()),
        );
        c
    });
    let ctx = test_context();

    let result = filesystem_check_exists(&node, &ctx).await.unwrap();
    assert_eq!(result["exists"], true);
    assert_eq!(result["path"], test_file.to_string_lossy().to_string());
}

#[tokio::test]
async fn test_filesystem_check_exists_missing() {
    let node = test_node_with_config("fs2", {
        let mut c = HashMap::new();
        c.insert("path".to_string(), json!("/nonexistent/path/seed.bin"));
        c
    });
    let ctx = test_context();

    let result = filesystem_check_exists(&node, &ctx).await.unwrap();
    assert_eq!(result["exists"], false);
}

#[tokio::test]
async fn test_filesystem_check_exists_with_env_var() {
    let temp_dir = tempfile::tempdir().unwrap();
    let test_file = temp_dir.path().join("family.seed");
    std::fs::write(&test_file, b"seed").unwrap();

    let node = test_node_with_config("fs3", {
        let mut c = HashMap::new();
        c.insert("path".to_string(), json!("${SEED_DIR}/family.seed"));
        c
    });
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test".to_string());
    env.insert(
        "SEED_DIR".to_string(),
        temp_dir.path().to_string_lossy().to_string(),
    );
    let ctx = test_context_with_env(env);

    let result = filesystem_check_exists(&node, &ctx).await.unwrap();
    assert_eq!(result["exists"], true);
}

#[tokio::test]
async fn test_filesystem_check_exists_missing_path_config() {
    let node = test_node_with_config("fs4", HashMap::new());
    let ctx = test_context();

    let result = filesystem_check_exists(&node, &ctx).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("path"));
}
