// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::*;

// Tests use capability keywords, not primal names.
// Primals are discovered by what they CAN DO, not what they're CALLED.

#[test]
fn test_infer_capabilities_security_keywords() {
    let caps = infer_capabilities_from_name("crypto-provider");
    assert!(caps.contains(&"security".to_string()));
    assert!(caps.contains(&"crypto.encrypt".to_string()));

    // "key" is a security keyword
    let caps2 = infer_capabilities_from_name("key-manager");
    assert!(caps2.contains(&"security".to_string()));
}

#[test]
fn test_infer_capabilities_discovery_keywords() {
    let caps = infer_capabilities_from_name("mesh-relay");
    assert!(caps.contains(&"discovery".to_string()));

    let caps2 = infer_capabilities_from_name("beacon-service");
    assert!(caps2.contains(&"discovery".to_string()));
}

#[test]
fn test_infer_capabilities_storage_keywords() {
    let caps = infer_capabilities_from_name("data-store");
    assert!(caps.contains(&"storage".to_string()));
}

#[test]
fn test_infer_capabilities_compute_keywords() {
    let caps = infer_capabilities_from_name("shell-gate");
    assert!(caps.contains(&"shell".to_string()));
}

#[test]
fn test_infer_capabilities_ai_keywords() {
    let caps = infer_capabilities_from_name("ai-assistant");
    assert!(caps.contains(&"ai".to_string()));
}

#[test]
fn test_infer_type_by_capability() {
    // Types are inferred from capability keywords, not primal names
    assert_eq!(infer_type_from_name("crypto-vault"), "security");
    assert_eq!(infer_type_from_name("mesh-beacon"), "discovery");
    // Use "shell-runner" instead of "compute-sandbox" to avoid substring collisions
    assert_eq!(infer_type_from_name("shell-runner"), "shell");
    assert_eq!(infer_type_from_name("data-store"), "storage");
    assert_eq!(infer_type_from_name("ai-model"), "ai");
    assert_eq!(infer_type_from_name("Unknown"), "primal");
}

#[test]
fn test_socket_dir_override_and_default() {
    let dir = get_socket_dir_from(Some("/tmp/biomeos-test-override"));
    assert_eq!(dir, "/tmp/biomeos-test-override");

    let dir = get_socket_dir_from(None);
    let expected = biomeos_types::SystemPaths::new_lazy()
        .runtime_dir()
        .to_string_lossy()
        .to_string();
    assert_eq!(dir, expected);
}

#[test]
fn test_live_primal_info_serialize() {
    let info = LivePrimalInfo {
        id: "test-local".to_string(),
        name: "Test".to_string(),
        primal_type: "primal".to_string(),
        version: "1.0.0".to_string(),
        health: "healthy".to_string(),
        capabilities: vec!["test".to_string()],
        endpoint: "/tmp/test.sock".to_string(),
        family_id: None,
    };

    let json = serde_json::to_string(&info).unwrap();
    assert!(json.contains("test-local"));
}

#[test]
fn test_live_primal_info_roundtrip() {
    let info = LivePrimalInfo {
        id: "roundtrip-id".to_string(),
        name: "RoundtripPrimal".to_string(),
        primal_type: "security".to_string(),
        version: "2.0.0".to_string(),
        health: "healthy".to_string(),
        capabilities: vec!["crypto.encrypt".to_string(), "crypto.sign".to_string()],
        endpoint: "/run/user/1000/biomeos/test.sock".to_string(),
        family_id: Some("family-abc".to_string()),
    };

    let json = serde_json::to_string(&info).expect("serialize");
    let restored: LivePrimalInfo = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(info.id, restored.id);
    assert_eq!(info.name, restored.name);
    assert_eq!(info.primal_type, restored.primal_type);
    assert_eq!(info.family_id, restored.family_id);
}

#[test]
fn test_identity_attestation_roundtrip() {
    let attestation = IdentityAttestation {
        provider_capability: "crypto.verify".to_string(),
        format: "jwt".to_string(),
        data: serde_json::json!({"sub": "primal-1", "aud": "biomeos"}),
    };

    let json = serde_json::to_string(&attestation).expect("serialize");
    let restored: IdentityAttestation = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(
        attestation.provider_capability,
        restored.provider_capability
    );
    assert_eq!(attestation.format, restored.format);
}

#[test]
fn test_get_socket_dir_returns_valid_path() {
    let dir = get_socket_dir();
    assert!(!dir.is_empty(), "socket dir should not be empty");
    assert!(
        dir.contains("biomeos")
            || dir.starts_with("/tmp")
            || dir.starts_with("/run")
            || dir.starts_with("/nonexistent")
            || std::path::Path::new(&dir).is_absolute(),
        "socket dir should be an absolute path, got: {dir}"
    );
}

#[test]
fn test_infer_capabilities_unknown_name() {
    let caps = infer_capabilities_from_name("xyz-unknown-service");
    assert_eq!(caps, vec!["primal".to_string()]);
}

#[test]
fn test_infer_capabilities_first_match_wins() {
    // "security" and "discovery" - first domain in CAPABILITY_DOMAINS wins
    let caps = infer_capabilities_from_name("security-discovery-hybrid");
    assert!(caps.contains(&"security".to_string()));
}

#[test]
fn test_infer_type_unknown() {
    assert_eq!(infer_type_from_name("random-service-xyz"), "primal");
}

#[tokio::test]
async fn test_discover_all_primals_empty_dir() {
    // With no sockets, should return empty without panicking
    let primals = discover_all_primals_in("/nonexistent/path/for/tests").await;
    assert!(primals.is_empty());
}

#[tokio::test]
async fn test_discover_by_capability_returns() {
    let primals = discover_by_capability_in("crypto.encrypt", "/nonexistent/path").await;
    assert!(primals.is_empty());
}

#[tokio::test]
async fn test_discover_by_type_returns() {
    let primals = discover_by_type_in("security", "/nonexistent/path").await;
    assert!(primals.is_empty());
}

#[tokio::test]
async fn test_discover_primal_connection_refused() {
    let result = discover_primal("/nonexistent/socket/path.sock").await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("Failed to connect")
            || err.to_string().contains("socket")
            || err.to_string().contains("Connection refused")
            || err.to_string().contains("No such file"),
        "expected connection error, got: {err}"
    );
}

#[tokio::test]
async fn test_discover_all_primals_nonexistent_dir() {
    let primals = discover_all_primals_in("/nonexistent/path/that/does/not/exist").await;
    assert!(primals.is_empty());
}

#[test]
fn test_discover_by_capability_filter_substring() {
    let info = LivePrimalInfo {
        id: "test-1".to_string(),
        name: "Test".to_string(),
        primal_type: "security".to_string(),
        version: "1.0".to_string(),
        health: "healthy".to_string(),
        capabilities: vec!["crypto.encrypt".to_string(), "crypto.sign".to_string()],
        endpoint: "/tmp/test.sock".to_string(),
        family_id: None,
    };
    assert!(info.capabilities.iter().any(|c| c == "crypto.encrypt"));
    assert!(
        info.capabilities
            .iter()
            .any(|c| c.starts_with(&format!("{}.", "crypto")))
    );
}

#[test]
fn test_live_primal_info_deserialize() {
    let json = r#"{"id":"x","name":"N","primal_type":"t","version":"1","health":"ok","capabilities":["c1"],"endpoint":"/tmp/x.sock"}"#;
    let info: LivePrimalInfo = serde_json::from_str(json).expect("deserialize");
    assert_eq!(info.id, "x");
    assert_eq!(info.name, "N");
    assert_eq!(info.capabilities.len(), 1);
}

#[test]
fn test_identity_attestation_serialize() {
    let attestation = IdentityAttestation {
        provider_capability: "crypto.verify".to_string(),
        format: "jwt".to_string(),
        data: serde_json::json!({"sub": "primal"}),
    };
    let json = serde_json::to_string(&attestation).expect("serialize");
    assert!(json.contains("crypto.verify"));
    assert!(json.contains("jwt"));
}

#[tokio::test]
async fn test_discover_all_primals_socket_dir_exists_no_socks() {
    let temp = tempfile::tempdir().expect("tempdir");
    let p = temp.path().to_string_lossy();
    let primals = discover_all_primals_in(p.as_ref()).await;
    assert!(
        primals.is_empty(),
        "expected empty discovery in isolated temp dir, found {} primals",
        primals.len()
    );
}

/// Covers `send_rpc_request` success path and `discover_primal` field mapping.
#[cfg(unix)]
#[tokio::test]
async fn test_discover_primal_health_check_success_unix() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;

    let temp = tempfile::tempdir().expect("tempdir");
    let sock_path = temp.path().join("live-disc.sock");
    let sock_str = sock_path.to_str().expect("utf8 path").to_string();

    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    let path_for_server = sock_path.clone();
    tokio::spawn(async move {
        let listener = UnixListener::bind(&path_for_server).expect("bind");
        ready_tx.send(()).ok();
        let (mut stream, _) = listener.accept().await.expect("accept");
        let mut buf = vec![0u8; 8192];
        let _ = stream.read(&mut buf).await;
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {
                "name": "unix-test-primal",
                "version": "3.0.0",
                "status": "ok",
                "capabilities": ["discovery", "http.get"],
                "family_id": "fam-x"
            },
            "id": 1
        });
        let s = serde_json::to_string(&body).expect("json");
        stream.write_all(s.as_bytes()).await.expect("write");
        stream.flush().await.ok();
    });

    ready_rx.await.expect("listener ready");
    let info = discover_primal(&sock_str).await.expect("discover primal");
    assert_eq!(info.name, "unix-test-primal");
    assert_eq!(info.version, "3.0.0");
    assert_eq!(info.health, "ok");
    assert_eq!(info.primal_type, "discovery");
    assert_eq!(info.family_id.as_deref(), Some("fam-x"));
    assert!(info.id.contains("unix-test-primal"));
}

/// `primal_name` fallback when `name` is absent.
#[cfg(unix)]
#[tokio::test]
async fn test_discover_primal_uses_primal_name_field_unix() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;

    let temp = tempfile::tempdir().expect("tempdir");
    let sock_path = temp.path().join("primal-name.sock");
    let sock_str = sock_path.to_str().expect("utf8").to_string();

    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    let path_for_server = sock_path.clone();
    tokio::spawn(async move {
        let listener = UnixListener::bind(&path_for_server).expect("bind");
        ready_tx.send(()).ok();
        let (mut stream, _) = listener.accept().await.expect("accept");
        let mut buf = vec![0u8; 8192];
        let _ = stream.read(&mut buf).await;
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "result": {
                "primal_name": "only-primal-name",
                "version": "0.0.1"
            },
            "id": 1
        });
        let s = serde_json::to_string(&body).expect("json");
        stream.write_all(s.as_bytes()).await.expect("write");
        stream.flush().await.ok();
    });

    ready_rx.await.expect("listener ready");
    let info = discover_primal(&sock_str).await.expect("discover");
    assert_eq!(info.name, "only-primal-name");
    assert!(info.capabilities.contains(&"primal".to_string()));
}

#[cfg(unix)]
#[tokio::test]
async fn test_discover_primal_jsonrpc_error_unix() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;

    let temp = tempfile::tempdir().expect("tempdir");
    let sock_path = temp.path().join("rpc-err.sock");
    let sock_str = sock_path.to_str().expect("utf8").to_string();

    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    let path_for_server = sock_path.clone();
    tokio::spawn(async move {
        let listener = UnixListener::bind(&path_for_server).expect("bind");
        ready_tx.send(()).ok();
        let (mut stream, _) = listener.accept().await.expect("accept");
        let mut buf = vec![0u8; 8192];
        let _ = stream.read(&mut buf).await;
        let body = serde_json::json!({
            "jsonrpc": "2.0",
            "error": {"code": -32000, "message": "method unavailable"},
            "id": 1
        });
        let s = serde_json::to_string(&body).expect("json");
        stream.write_all(s.as_bytes()).await.expect("write");
        stream.flush().await.ok();
    });

    ready_rx.await.expect("listener ready");
    let err = discover_primal(&sock_str).await.expect_err("rpc error");
    let msg = err.to_string();
    assert!(
        msg.contains("RPC error") && msg.contains("method unavailable"),
        "{msg}"
    );
}

#[cfg(unix)]
#[tokio::test]
async fn test_discover_primal_malformed_json_response_unix() {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;

    let temp = tempfile::tempdir().expect("tempdir");
    let sock_path = temp.path().join("bad-json.sock");
    let sock_str = sock_path.to_str().expect("utf8").to_string();

    let (ready_tx, ready_rx) = tokio::sync::oneshot::channel();
    let path_for_server = sock_path.clone();
    tokio::spawn(async move {
        let listener = UnixListener::bind(&path_for_server).expect("bind");
        ready_tx.send(()).ok();
        let (mut stream, _) = listener.accept().await.expect("accept");
        let mut buf = vec![0u8; 8192];
        let _ = stream.read(&mut buf).await;
        stream.write_all(b"NOT JSON").await.expect("write");
        stream.flush().await.ok();
    });

    ready_rx.await.expect("listener ready");
    assert!(discover_primal(&sock_str).await.is_err());
}
