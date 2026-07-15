// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;
use std::path::PathBuf;

#[tokio::test]
async fn test_handle_health_graph_missing_niche() {
    let result = handle_health(
        None, false, false, 10, true, // use_graph
        None, // niche_path - required for graph health
    )
    .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("niche"));
}

#[tokio::test]
async fn test_handle_health_graph_deprecated() {
    let result = handle_health(
        None,
        false,
        false,
        10,
        true,
        Some(PathBuf::from("/tmp/test-niche")),
    )
    .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("deprecated"));
}

#[tokio::test]
async fn test_handle_health_legacy() {
    let result = handle_health(None, false, false, 10, false, None).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_handle_health_with_service() {
    let result = handle_health(
        Some("test-service".to_string()),
        false,
        false,
        10,
        false,
        None,
    )
    .await;
    assert!(result.is_ok());
}
