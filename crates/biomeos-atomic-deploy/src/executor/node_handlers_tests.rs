// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::super::context::NodeStatus;
use super::*;
use crate::neural_graph::GraphNode;
use crate::nucleation::SocketNucleation;
use biomeos_test_utils::MockJsonRpcServer;
use serde_json::json;

// ========================================================================
// substitute_env tests
// ========================================================================

#[test]
fn test_substitute_env() {
    let mut env = HashMap::new();
    env.insert("FOO".to_string(), "bar".to_string());
    env.insert("FAMILY_ID".to_string(), "1894e909e454".to_string());

    assert_eq!(substitute_env("${FOO}", &env), "bar");
    assert_eq!(substitute_env("$FOO", &env), "bar");
    assert_eq!(
        substitute_env("prefix-${FAMILY_ID}-suffix", &env),
        "prefix-1894e909e454-suffix"
    );
}

#[test]
fn test_substitute_env_missing() {
    let env = HashMap::new();
    assert_eq!(substitute_env("${MISSING}", &env), "${MISSING}");
}

#[test]
fn test_substitute_env_multiple_vars() {
    let mut env = HashMap::new();
    env.insert("A".to_string(), "alpha".to_string());
    env.insert("B".to_string(), "beta".to_string());
    env.insert("C".to_string(), "gamma".to_string());

    assert_eq!(substitute_env("${A}/${B}/${C}", &env), "alpha/beta/gamma");
}

#[test]
fn test_substitute_env_xdg_runtime_dir() {
    let mut env = HashMap::new();
    env.insert("XDG_RUNTIME_DIR".to_string(), "/run/user/1000".to_string());
    env.insert("FAMILY_ID".to_string(), "cf7e8729".to_string());

    assert_eq!(
        substitute_env("${XDG_RUNTIME_DIR}/biomeos/beardog-${FAMILY_ID}.sock", &env),
        "/run/user/1000/biomeos/beardog-cf7e8729.sock"
    );
}

#[test]
fn test_substitute_env_empty_value() {
    let mut env = HashMap::new();
    env.insert("EMPTY".to_string(), String::new());

    assert_eq!(
        substitute_env("prefix-${EMPTY}-suffix", &env),
        "prefix--suffix"
    );
}

#[test]
fn test_substitute_env_no_vars_in_string() {
    let mut env = HashMap::new();
    env.insert("FOO".to_string(), "bar".to_string());

    assert_eq!(
        substitute_env("no variables here", &env),
        "no variables here"
    );
}

#[test]
fn test_substitute_env_dollar_sign_syntax() {
    let mut env = HashMap::new();
    env.insert("PORT".to_string(), "8080".to_string());

    // $PORT syntax (without braces)
    assert_eq!(substitute_env("localhost:$PORT", &env), "localhost:8080");
}

#[test]
fn test_substitute_env_repeated_var() {
    let mut env = HashMap::new();
    env.insert("HOST".to_string(), "gate2".to_string());

    assert_eq!(substitute_env("${HOST}:${HOST}", &env), "gate2:gate2");
}

// ========================================================================
// Helper: create test GraphNode with config
// ========================================================================

fn test_node_with_config(id: &str, config: HashMap<String, serde_json::Value>) -> GraphNode {
    GraphNode {
        id: id.to_string(),
        config,
        ..Default::default()
    }
}

fn test_context() -> ExecutionContext {
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test-family".to_string());
    ExecutionContext::new(env)
}

fn test_context_with_env(env: HashMap<String, String>) -> ExecutionContext {
    ExecutionContext::new(env)
}

// ========================================================================
// log_info / log_warn / log_error tests
// ========================================================================

#[tokio::test]
async fn test_log_info() {
    let node = test_node_with_config("log1", {
        let mut c = HashMap::new();
        c.insert("message".to_string(), json!("Hello from biomeOS"));
        c
    });
    let ctx = test_context();

    let result = log_info(&node, &ctx).await.unwrap();
    assert_eq!(result["level"], "info");
    assert_eq!(result["message"], "Hello from biomeOS");
}

#[tokio::test]
async fn test_log_info_with_env_substitution() {
    let node = test_node_with_config("log2", {
        let mut c = HashMap::new();
        c.insert("message".to_string(), json!("Family: ${FAMILY_ID}"));
        c
    });
    let ctx = test_context();

    let result = log_info(&node, &ctx).await.unwrap();
    assert_eq!(result["message"], "Family: test-family");
}

#[tokio::test]
async fn test_log_info_no_message() {
    let node = test_node_with_config("log3", HashMap::new());
    let ctx = test_context();

    let result = log_info(&node, &ctx).await.unwrap();
    assert_eq!(result["message"], "(no message)");
}

#[tokio::test]
async fn test_log_warn() {
    let node = test_node_with_config("warn1", {
        let mut c = HashMap::new();
        c.insert("message".to_string(), json!("Something concerning"));
        c
    });
    let ctx = test_context();

    let result = log_warn(&node, &ctx).await.unwrap();
    assert_eq!(result["level"], "warn");
    assert_eq!(result["message"], "Something concerning");
}

#[tokio::test]
async fn test_log_error() {
    let node = test_node_with_config("err1", {
        let mut c = HashMap::new();
        c.insert("message".to_string(), json!("Critical failure"));
        c
    });
    let ctx = test_context();

    let result = log_error(&node, &ctx).await.unwrap();
    assert_eq!(result["level"], "error");
    assert_eq!(result["message"], "Critical failure");
}

// ========================================================================
// deployment_report tests
// ========================================================================

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

// ========================================================================
// filesystem_check_exists tests
// ========================================================================

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

// ========================================================================
// discover_capability_provider tests
// ========================================================================

#[tokio::test]
async fn test_discover_capability_provider_via_socket_env() {
    let temp_dir = tempfile::tempdir().unwrap();
    let sock_path = temp_dir.path().join("beardog.sock");
    // Create a regular file (not a real socket, but metadata check uses exists)
    std::fs::write(&sock_path, b"").unwrap();

    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test".to_string());
    env.insert(
        "SECURITY_SOCKET".to_string(),
        sock_path.to_string_lossy().to_string(),
    );
    let ctx = test_context_with_env(env);

    let result = discover_capability_provider(&ctx, "security").await;
    assert_eq!(result, Some(sock_path.to_string_lossy().to_string()));
}

#[tokio::test]
async fn test_discover_capability_provider_via_endpoint_env() {
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test".to_string());
    env.insert(
        "SECURITY_ENDPOINT".to_string(),
        "http://localhost:8080".to_string(),
    );
    let ctx = test_context_with_env(env);

    let result = discover_capability_provider(&ctx, "security").await;
    assert_eq!(result, Some("http://localhost:8080".to_string()));
}

#[tokio::test]
async fn test_discover_capability_provider_none_found() {
    let ctx = test_context();

    let result = discover_capability_provider(&ctx, "unknown_capability").await;
    assert!(result.is_none());
}

#[tokio::test]
async fn test_discover_capability_provider_socket_not_exists() {
    let mut env = HashMap::new();
    env.insert("FAMILY_ID".to_string(), "test".to_string());
    env.insert(
        "SECURITY_SOCKET".to_string(),
        "/nonexistent/beardog.sock".to_string(),
    );
    let ctx = test_context_with_env(env);

    // Socket file doesn't exist, should fall through to endpoint check
    let result = discover_capability_provider(&ctx, "security").await;
    assert!(result.is_none());
}

// ========================================================================
// crypto_derive_seed tests (fallback path — no security provider)
// ========================================================================

#[tokio::test]
async fn test_crypto_derive_seed_fallback() {
    let node = test_node_with_config("derive1", {
        let mut c = HashMap::new();
        c.insert("source".to_string(), json!("tower"));
        c
    });
    let ctx = test_context();

    // No security socket configured → should use deterministic fallback
    let result = crypto_derive_seed(&node, &ctx).await.unwrap();
    assert_eq!(result["method"], "deterministic_fallback");
    assert_eq!(result["derived_from"], "tower");
    assert!(result["seed"].as_str().unwrap().contains("test-family"));
}

#[tokio::test]
async fn test_crypto_derive_seed_default_source() {
    let node = test_node_with_config("derive2", HashMap::new());
    let ctx = test_context();

    let result = crypto_derive_seed(&node, &ctx).await.unwrap();
    assert_eq!(result["derived_from"], "family"); // default
}

// ========================================================================
// lineage_verify tests (fallback path — no security provider)
// ========================================================================

#[tokio::test]
async fn test_lineage_verify_no_provider() {
    let node = test_node_with_config("verify1", {
        let mut c = HashMap::new();
        c.insert("primal_name".to_string(), json!("beardog"));
        c
    });
    let ctx = test_context();

    let result = lineage_verify(&node, &ctx).await.unwrap();
    assert_eq!(result["verified"], true);
    assert_eq!(result["method"], "assumed_valid_no_provider");
}

#[tokio::test]
async fn test_lineage_verify_missing_primal_name() {
    let node = test_node_with_config("verify2", HashMap::new());
    let ctx = test_context();

    let result = lineage_verify(&node, &ctx).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("primal_name"));
}

// ========================================================================
// health_check tests (error paths)
// ========================================================================

#[tokio::test]
async fn test_health_check_missing_primal_name() {
    let node = test_node_with_config("hc1", HashMap::new());
    let ctx = test_context();

    let result = health_check(&node, &ctx).await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("primal_name"));
}

// ========================================================================
// register_capabilities tests
// ========================================================================

fn test_node_with_capabilities(
    id: &str,
    config: HashMap<String, serde_json::Value>,
    capabilities: Vec<String>,
) -> GraphNode {
    GraphNode {
        id: id.to_string(),
        config,
        capabilities,
        ..Default::default()
    }
}

#[tokio::test]
async fn test_register_capabilities_with_caps() {
    let node = test_node_with_capabilities(
        "reg1",
        {
            let mut c = HashMap::new();
            c.insert("primal_name".to_string(), json!("beardog"));
            c
        },
        vec!["crypto.encrypt".to_string(), "crypto.decrypt".to_string()],
    );
    let ctx = test_context();

    let result = register_capabilities(&node, &ctx).await.unwrap();
    assert_eq!(result["primal"], "beardog");
    assert_eq!(result["count"], 2);
    let registered = result["registered"].as_array().unwrap();
    assert_eq!(registered.len(), 2);
    assert!(
        registered
            .iter()
            .any(|v| v.as_str() == Some("crypto.encrypt"))
    );
    assert!(
        registered
            .iter()
            .any(|v| v.as_str() == Some("crypto.decrypt"))
    );
}

#[tokio::test]
async fn test_register_capabilities_empty_caps() {
    let node = test_node_with_capabilities(
        "reg2",
        {
            let mut c = HashMap::new();
            c.insert("primal_name".to_string(), json!("songbird"));
            c
        },
        vec![],
    );
    let ctx = test_context();

    let result = register_capabilities(&node, &ctx).await.unwrap();
    assert_eq!(result["primal"], "songbird");
    assert_eq!(result["count"], 0);
    assert!(result["registered"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn test_register_capabilities_default_primal_name() {
    let node = test_node_with_capabilities("reg3", HashMap::new(), vec!["mesh".to_string()]);
    let ctx = test_context();

    let result = register_capabilities(&node, &ctx).await.unwrap();
    assert_eq!(result["primal"], "unknown");
    assert_eq!(result["count"], 1);
}

// ========================================================================
// crypto_derive_seed / health_check / lineage_verify — mock primal socket
// ========================================================================

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
