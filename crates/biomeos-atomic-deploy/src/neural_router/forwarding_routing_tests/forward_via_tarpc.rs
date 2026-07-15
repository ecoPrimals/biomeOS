// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::create_router;
use tempfile::TempDir;

#[tokio::test]
async fn test_forward_via_tarpc_socket_not_found() {
    let router = create_router("test");
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("nonexistent.sock");
    let result = router
        .forward_via_tarpc(&socket_path, "health.check", &serde_json::json!({}))
        .await;
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("tarpc socket not found") || msg.contains("not found"));
}

#[tokio::test]
async fn test_forward_via_tarpc_discovery_method_requires_tarpc_server() {
    let router = create_router("test");
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("primal.sock");
    let tarpc_path = temp.path().join("primal.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_path);

    let result = router
        .forward_via_tarpc(
            &socket_path,
            "discovery.unknown_method",
            &serde_json::json!({}),
        )
        .await;
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("discovery") || msg.contains("connect") || msg.contains("tarpc"),
        "unexpected error: {msg}"
    );
}

#[tokio::test]
async fn test_forward_via_tarpc_security_method_requires_tarpc_server() {
    let router = create_router("test");
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("primal.sock");
    let tarpc_path = temp.path().join("primal.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_path);

    let result = router
        .forward_via_tarpc(
            &socket_path,
            "security.unknown_method",
            &serde_json::json!({}),
        )
        .await;
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("security") || msg.contains("connect") || msg.contains("tarpc"),
        "unexpected error: {msg}"
    );
}

#[tokio::test]
async fn test_forward_via_tarpc_no_tarpc_mapping() {
    let router = create_router("test");
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("primal.sock");
    let tarpc_path = temp.path().join("primal.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_path);

    let result = router
        .forward_via_tarpc(&socket_path, "custom.unknown", &serde_json::json!({}))
        .await;
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(msg.contains("no tarpc mapping"));
}

#[tokio::test]
async fn test_forward_via_tarpc_discovery_unknown_method_after_socket_exists() {
    let router = create_router("test");
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("disc.sock");
    let tarpc_path = temp.path().join("disc.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_path);

    let result = router
        .forward_via_tarpc(
            &socket_path,
            "discovery.not_a_real_method",
            &serde_json::json!({}),
        )
        .await;

    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("unknown discovery method") || msg.contains("connect"),
        "unexpected: {msg}"
    );
}

#[tokio::test]
async fn test_forward_via_tarpc_discovery_register_invalid_body() {
    let router = create_router("test");
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("p.sock");
    let tarpc_path = temp.path().join("p.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_path);

    let result = router
        .forward_via_tarpc(
            &socket_path,
            "discovery.register",
            &serde_json::json!({"not": "ServiceRegistration"}),
        )
        .await;
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("serde") || msg.contains("connect") || msg.contains("register"),
        "unexpected: {msg}"
    );
}

#[tokio::test]
async fn test_forward_via_tarpc_security_sign_missing_data() {
    let router = create_router("test");
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("sec.sock");
    let tarpc_path = temp.path().join("sec.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_path);

    let result = router
        .forward_via_tarpc(&socket_path, "security.sign", &serde_json::json!({}))
        .await;
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("missing param: data") || msg.contains("connect"),
        "unexpected: {msg}"
    );
}

#[tokio::test]
async fn test_forward_via_tarpc_security_unknown_method() {
    let router = create_router("test");
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("sec2.sock");
    let tarpc_path = temp.path().join("sec2.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_path);

    let result = router
        .forward_via_tarpc(&socket_path, "security.unknown_xyz", &serde_json::json!({}))
        .await;
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("unknown security") || msg.contains("connect"),
        "unexpected: {msg}"
    );
}

#[tokio::test]
async fn test_forward_via_tarpc_health_metrics_alias() {
    let router = create_router("test");
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("hm.sock");
    let tarpc_path = temp.path().join("hm.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_path);

    let result = router
        .forward_via_tarpc(&socket_path, "health_metrics", &serde_json::json!({}))
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_forward_via_tarpc_health_version_alias() {
    let router = create_router("test");
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("hv.sock");
    let tarpc_path = temp.path().join("hv.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_path);

    let result = router
        .forward_via_tarpc(&socket_path, "version", &serde_json::json!({}))
        .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_forward_via_tarpc_discovery_discover_all() {
    let router = create_router("test");
    let temp = TempDir::new().expect("temp dir");
    let socket_path = temp.path().join("da.sock");
    let tarpc_path = temp.path().join("da.tarpc.sock");
    let _ = std::fs::File::create(&tarpc_path);

    let result = router
        .forward_via_tarpc(
            &socket_path,
            "discovery_discover_all",
            &serde_json::json!({}),
        )
        .await;
    assert!(result.is_err());
}
