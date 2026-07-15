// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::{test_context_with_env, test_node_with_config};
use super::super::{crypto_derive_seed, health_check, lineage_verify};
use biomeos_test_utils::MockJsonRpcServer;
use crate::nucleation::SocketNucleation;
use serde_json::json;
use std::collections::HashMap;

#[tokio::test]
async fn test_crypto_derive_seed_via_security_socket() {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("sec.sock");
    let _server =
        MockJsonRpcServer::spawn_echo_success(&sock, json!({ "seed": "from-mock", "ok": true }))
            .await;

    let node = test_node_with_config("cd1", {
        let mut c = HashMap::new();
        c.insert("source".to_string(), json!("unit"));
        c
    });
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "fam-x".to_string());
    env.insert(
        "SECURITY_SOCKET".to_string(),
        sock.to_string_lossy().to_string(),
    );
    let ctx = test_context_with_env(env);

    let result = crypto_derive_seed(&node, &ctx).await.unwrap();
    assert_eq!(result["seed"], "from-mock");
}

#[tokio::test]
async fn test_crypto_derive_seed_rpc_error_from_primal() {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("sec2.sock");
    let _server = MockJsonRpcServer::spawn_echo_error(&sock, -32000, "crypto failed").await;

    let node = test_node_with_config("cd2", HashMap::new());
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "fam".to_string());
    env.insert(
        "SECURITY_SOCKET".to_string(),
        sock.to_string_lossy().to_string(),
    );
    let ctx = test_context_with_env(env);

    let err = crypto_derive_seed(&node, &ctx).await.unwrap_err();
    assert!(err.to_string().contains("Crypto derive failed"));
}

#[tokio::test]
async fn test_health_check_healthy_true() {
    let fam = "hc-healthy-fam";
    let mut nuc = SocketNucleation::default();
    let sock = nuc.assign_socket("beardog", fam);
    let _ = std::fs::remove_file(&sock);
    let _server = MockJsonRpcServer::spawn(&sock, |_| {
        r#"{"jsonrpc":"2.0","id":1,"result":{"healthy":true,"detail":"ok"}}"#.to_string()
    })
    .await;

    let node = test_node_with_config("hc2", {
        let mut c = HashMap::new();
        c.insert("primal_name".to_string(), json!("beardog"));
        c.insert("timeout_secs".to_string(), json!(5));
        c
    });
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), fam.to_string());
    let ctx = test_context_with_env(env);

    let r = health_check(&node, &ctx).await.unwrap();
    assert_eq!(r["healthy"], true);
}

#[tokio::test]
async fn test_health_check_reports_unhealthy() {
    let fam = "hc-unhealthy-fam";
    let mut nuc = SocketNucleation::default();
    let sock = nuc.assign_socket("beardog", fam);
    let _ = std::fs::remove_file(&sock);
    let _server = MockJsonRpcServer::spawn(&sock, |_| {
        r#"{"jsonrpc":"2.0","id":1,"result":{"healthy":false}}"#.to_string()
    })
    .await;

    let node = test_node_with_config("hc3", {
        let mut c = HashMap::new();
        c.insert("primal_name".to_string(), json!("beardog"));
        c.insert("timeout_secs".to_string(), json!(5));
        c
    });
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), fam.to_string());
    let ctx = test_context_with_env(env);

    let r = health_check(&node, &ctx).await.unwrap();
    assert_eq!(r["healthy"], false);
}

#[tokio::test]
async fn test_health_check_invalid_json_response_errors() {
    let fam = "hc-badjson-fam";
    let mut nuc = SocketNucleation::default();
    let sock = nuc.assign_socket("beardog", fam);
    let _ = std::fs::remove_file(&sock);
    let _server = MockJsonRpcServer::spawn(&sock, |_| "not-json\n".to_string()).await;

    let node = test_node_with_config("hc4", {
        let mut c = HashMap::new();
        c.insert("primal_name".to_string(), json!("beardog"));
        c
    });
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), fam.to_string());
    let ctx = test_context_with_env(env);

    let err = health_check(&node, &ctx).await.unwrap_err();
    assert!(
        err.to_string().contains("JSON") || err.to_string().contains("Invalid"),
        "unexpected: {err}"
    );
}

#[tokio::test]
async fn test_lineage_verify_with_security_provider_result() {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("lin.sock");
    let _server =
        MockJsonRpcServer::spawn_echo_success(&sock, json!({ "verified": true, "lineage": "ok" }))
            .await;

    let node = test_node_with_config("lv1", {
        let mut c = HashMap::new();
        c.insert("primal_name".to_string(), json!("beardog"));
        c
    });
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "fam".to_string());
    env.insert(
        "SECURITY_SOCKET".to_string(),
        sock.to_string_lossy().to_string(),
    );
    let ctx = test_context_with_env(env);

    let r = lineage_verify(&node, &ctx).await.unwrap();
    assert_eq!(r["verified"], true);
    assert_eq!(r["lineage"], "ok");
}

#[tokio::test]
async fn test_lineage_verify_missing_result_branch() {
    let dir = tempfile::tempdir().unwrap();
    let sock = dir.path().join("lin2.sock");
    let _server = MockJsonRpcServer::spawn(&sock, |_| {
        r#"{"jsonrpc":"2.0","id":1,"error":{"code":-1,"message":"nope"}}"#.to_string()
    })
    .await;

    let node = test_node_with_config("lv2", {
        let mut c = HashMap::new();
        c.insert("primal_name".to_string(), json!("beardog"));
        c
    });
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "fam".to_string());
    env.insert(
        "SECURITY_SOCKET".to_string(),
        sock.to_string_lossy().to_string(),
    );
    let ctx = test_context_with_env(env);

    let r = lineage_verify(&node, &ctx).await.unwrap();
    assert_eq!(r["verified"], false);
}
