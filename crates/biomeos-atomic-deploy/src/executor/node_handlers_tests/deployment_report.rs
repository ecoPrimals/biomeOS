// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::{test_context, test_node_with_config};
use super::super::deployment_report;
use super::super::super::context::NodeStatus;
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_deployment_report_empty() {
    let node = test_node_with_config("report1", {
        let mut c = HashMap::new();
        c.insert("title".to_string(), json!("Test Report"));
        c
    });
    let ctx = test_context();

    let result = deployment_report(&node, &ctx).await.unwrap();
    assert_eq!(result["title"], "Test Report");
    assert_eq!(result["completed"], 0);
    assert_eq!(result["failed"], 0);
    assert_eq!(result["total"], 0);
    assert_eq!(result["success"], true);
}

#[tokio::test]
async fn test_deployment_report_with_completed_nodes() {
    let node = test_node_with_config("report2", {
        let mut c = HashMap::new();
        c.insert("title".to_string(), json!("NUCLEUS Deployment"));
        c
    });
    let ctx = test_context();

    // Simulate completed nodes
    ctx.set_status("beardog", NodeStatus::Completed(json!({"status": "ok"})))
        .await;
    ctx.set_status("songbird", NodeStatus::Completed(json!({"status": "ok"})))
        .await;

    let result = deployment_report(&node, &ctx).await.unwrap();
    assert_eq!(result["completed"], 2);
    assert_eq!(result["failed"], 0);
    assert_eq!(result["total"], 2);
    assert_eq!(result["success"], true);
}

#[tokio::test]
async fn test_deployment_report_with_failures() {
    let node = test_node_with_config("report3", HashMap::new());
    let ctx = test_context();

    ctx.set_status("beardog", NodeStatus::Completed(json!({"status": "ok"})))
        .await;
    ctx.set_status("songbird", NodeStatus::Failed("Socket timeout".to_string()))
        .await;

    let result = deployment_report(&node, &ctx).await.unwrap();
    assert_eq!(result["title"], "Deployment Report"); // default title
    assert_eq!(result["completed"], 1);
    assert_eq!(result["failed"], 1);
    assert_eq!(result["total"], 2);
    assert_eq!(result["success"], false);
}

#[tokio::test]
async fn test_deployment_report_mixed_statuses() {
    let node = test_node_with_config("report4", HashMap::new());
    let ctx = test_context();

    ctx.set_status("beardog", NodeStatus::Completed(json!({})))
        .await;
    ctx.set_status("songbird", NodeStatus::Running).await;
    ctx.set_status("toadstool", NodeStatus::Failed("OOM".to_string()))
        .await;
    ctx.set_status("nestgate", NodeStatus::Pending).await;
    ctx.set_status("squirrel", NodeStatus::Skipped).await;

    let result = deployment_report(&node, &ctx).await.unwrap();
    assert_eq!(result["completed"], 1);
    assert_eq!(result["failed"], 1);
    assert_eq!(result["total"], 5);
    assert_eq!(result["success"], false);
}
