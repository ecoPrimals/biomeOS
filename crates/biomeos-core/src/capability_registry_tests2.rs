// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Continuation of CapabilityRegistry tests (split from `capability_registry_tests.rs`).
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::capability_registry::*;
use crate::Capability;
use biomeos_types::PrimalId;

#[test]
fn test_registry_request_register_serialization() {
    let req = RegistryRequest::Register {
        id: "beardog-localhost".to_string(),
        request_id: "req-1".to_string(),
        params: RegisterParams {
            provides: vec![Capability::Security],
            requires: vec![],
            socket_path: Some("/tmp/beardog.sock".to_string()),
            http_endpoint: None,
            metadata: None,
        },
    };
    let json = serde_json::to_string(&req).expect("serialize");
    assert!(json.contains("register"));
    assert!(json.contains("beardog-localhost"));
    let restored: RegistryRequest = serde_json::from_str(&json).expect("deserialize");
    match restored {
        RegistryRequest::Register { id, .. } => assert_eq!(id, "beardog-localhost"),
        _ => panic!("Expected Register variant"),
    }
}

#[test]
fn test_registry_request_unregister_serialization() {
    let req = RegistryRequest::Unregister {
        request_id: "req-2".to_string(),
        primal_id: "songbird-localhost".to_string(),
    };
    let json = serde_json::to_string(&req).expect("serialize");
    assert!(json.contains("unregister"));
    let restored: RegistryRequest = serde_json::from_str(&json).expect("deserialize");
    match restored {
        RegistryRequest::Unregister { primal_id, .. } => {
            assert_eq!(primal_id, "songbird-localhost");
        }
        _ => panic!("Expected Unregister variant"),
    }
}

#[test]
fn test_registry_request_heartbeat_serialization() {
    let req = RegistryRequest::Heartbeat {
        request_id: "req-3".to_string(),
        primal_id: "beardog-localhost".to_string(),
    };
    let json = serde_json::to_string(&req).expect("serialize");
    assert!(json.contains("heartbeat"));
    let restored: RegistryRequest = serde_json::from_str(&json).expect("deserialize");
    match restored {
        RegistryRequest::Heartbeat { primal_id, .. } => assert_eq!(primal_id, "beardog-localhost"),
        _ => panic!("Expected Heartbeat variant"),
    }
}

#[test]
fn test_registry_request_list_primals_serialization() {
    let req = RegistryRequest::ListPrimals {
        request_id: "req-4".to_string(),
    };
    let json = serde_json::to_string(&req).expect("serialize");
    assert!(json.contains("list_primals"));
    let restored: RegistryRequest = serde_json::from_str(&json).expect("deserialize");
    match restored {
        RegistryRequest::ListPrimals { request_id } => assert_eq!(request_id, "req-4"),
        _ => panic!("Expected ListPrimals variant"),
    }
}

#[test]
fn test_registry_response_error_status() {
    let resp = RegistryResponse {
        request_id: "req-1".to_string(),
        status: ResponseStatus::Error,
        data: None,
        error: Some("Invalid primal ID".to_string()),
    };
    let json = serde_json::to_value(&resp).expect("serialize");
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"], "Invalid primal ID");
}

#[test]
fn test_registry_response_not_found_status() {
    let resp = RegistryResponse {
        request_id: "req-1".to_string(),
        status: ResponseStatus::NotFound,
        data: None,
        error: Some("No provider found".to_string()),
    };
    let json = serde_json::to_value(&resp).expect("serialize");
    assert_eq!(json["status"], "not_found");
}

// ── Socket server integration tests ────────────────────────────────────────
//
// All tests use `CapabilityRegistry::with_socket_path` to inject an explicit
// temp-dir socket, avoiding env-var races with `XDG_RUNTIME_DIR`.

fn make_registry(name: &str) -> (tempfile::TempDir, std::path::PathBuf, CapabilityRegistry) {
    let temp = tempfile::tempdir().expect("temp dir");
    let runtime_dir = temp.path().join("biomeos");
    std::fs::create_dir_all(&runtime_dir).expect("create runtime dir");
    let socket_path = runtime_dir.join(format!("biomeos-registry-{name}.sock"));
    let registry = CapabilityRegistry::with_socket_path(name.to_string(), socket_path.clone());
    (temp, socket_path, registry)
}

#[tokio::test]
async fn test_registry_serve_and_register_via_socket() {
    use biomeos_test_utils::ready_signal;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let (_temp, socket_path, registry) = make_registry("socket-test");
    let registry_clone = registry.clone();
    let (ready_tx, ready_rx) = ready_signal();

    let serve_handle = tokio::spawn(async move {
        if let Err(e) = registry_clone.serve_with_ready(ready_tx).await {
            panic!("serve_with_ready failed: {e}");
        }
    });

    tokio::time::timeout(std::time::Duration::from_secs(5), ready_rx.wait())
        .await
        .expect("server startup timed out")
        .expect("server should signal readiness");

    let mut stream = tokio::net::UnixStream::connect(&socket_path)
        .await
        .expect("connect to registry socket");

    let register_req = RegistryRequest::Register {
        id: "beardog-socket-test".to_string(),
        request_id: "req-1".to_string(),
        params: RegisterParams {
            provides: vec![Capability::Security],
            requires: vec![],
            socket_path: Some("/tmp/beardog.sock".to_string()),
            http_endpoint: None,
            metadata: None,
        },
    };
    let req_json = serde_json::to_string(&register_req).expect("serialize");
    stream.write_all(req_json.as_bytes()).await.expect("write");
    stream.write_all(b"\n").await.expect("write newline");
    stream.flush().await.expect("flush");

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await.expect("read response");
    let response: RegistryResponse = serde_json::from_str(&line).expect("parse response");
    assert_eq!(response.request_id, "req-1");
    assert!(matches!(response.status, ResponseStatus::Success));

    serve_handle.abort();
}

#[tokio::test]
async fn test_registry_serve_parse_error_continues() {
    use biomeos_test_utils::ready_signal;
    use tokio::io::AsyncWriteExt;

    let (_temp, socket_path, registry) = make_registry("parse-test");
    let registry_clone = registry.clone();
    let (ready_tx, ready_rx) = ready_signal();

    let _serve_handle = tokio::spawn(async move {
        if let Err(e) = registry_clone.serve_with_ready(ready_tx).await {
            panic!("serve_with_ready failed: {e}");
        }
    });

    tokio::time::timeout(std::time::Duration::from_secs(5), ready_rx.wait())
        .await
        .expect("server startup timed out")
        .expect("server should signal readiness");

    if socket_path.exists() {
        let mut stream = tokio::net::UnixStream::connect(&socket_path)
            .await
            .expect("connect");
        stream.write_all(b"not valid json\n").await.expect("write");
        stream.flush().await.expect("flush");
    }
}

#[tokio::test]
async fn test_registry_serve_get_provider_via_socket() {
    use biomeos_test_utils::ready_signal;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let (_temp, socket_path, registry) = make_registry("get-provider-test");

    let primal_id = PrimalId::new("beardog-get-test").unwrap();
    registry
        .register(
            primal_id,
            RegisterParams {
                provides: vec![Capability::Security],
                requires: vec![],
                socket_path: Some("/tmp/beardog.sock".to_string()),
                http_endpoint: None,
                metadata: None,
            },
        )
        .await
        .unwrap();

    let registry_clone = registry.clone();
    let (ready_tx, ready_rx) = ready_signal();
    let _serve_handle = tokio::spawn(async move {
        if let Err(e) = registry_clone.serve_with_ready(ready_tx).await {
            panic!("serve_with_ready failed: {e}");
        }
    });

    tokio::time::timeout(std::time::Duration::from_secs(5), ready_rx.wait())
        .await
        .expect("server startup timed out")
        .expect("server should signal readiness");

    if socket_path.exists() {
        let mut stream = tokio::net::UnixStream::connect(&socket_path)
            .await
            .expect("connect");

        let get_req = RegistryRequest::GetProvider {
            request_id: "req-get".to_string(),
            capability: Capability::Security,
        };
        let req_json = serde_json::to_string(&get_req).expect("serialize");
        stream.write_all(req_json.as_bytes()).await.expect("write");
        stream.write_all(b"\n").await.expect("write");
        stream.flush().await.expect("flush");

        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.expect("read");
        let response: RegistryResponse = serde_json::from_str(&line).expect("parse");
        assert_eq!(response.request_id, "req-get");
        assert!(matches!(response.status, ResponseStatus::Success));
        assert!(response.data.is_some());
    }
}

#[tokio::test]
async fn test_registry_socket_register_invalid_primal_id() {
    use biomeos_test_utils::ready_signal;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let (_temp, socket_path, registry) = make_registry("invalid-id-socket");
    let registry_clone = registry.clone();
    let (ready_tx, ready_rx) = ready_signal();

    let serve_handle = tokio::spawn(async move {
        if let Err(e) = registry_clone.serve_with_ready(ready_tx).await {
            panic!("serve_with_ready failed: {e}");
        }
    });

    tokio::time::timeout(std::time::Duration::from_secs(5), ready_rx.wait())
        .await
        .expect("server startup timed out")
        .expect("server should signal readiness");

    let mut stream = tokio::net::UnixStream::connect(&socket_path)
        .await
        .expect("connect to registry socket");

    let register_req = RegistryRequest::Register {
        id: "has spaces invalid".to_string(),
        request_id: "req-bad-id".to_string(),
        params: RegisterParams {
            provides: vec![Capability::Security],
            requires: vec![],
            socket_path: Some("/tmp/x.sock".to_string()),
            http_endpoint: None,
            metadata: None,
        },
    };
    let req_json = serde_json::to_string(&register_req).expect("serialize");
    stream.write_all(req_json.as_bytes()).await.expect("write");
    stream.write_all(b"\n").await.expect("write newline");
    stream.flush().await.expect("flush");

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await.expect("read response");
    let response: RegistryResponse = serde_json::from_str(&line).expect("parse response");
    assert_eq!(response.request_id, "req-bad-id");
    assert!(matches!(response.status, ResponseStatus::Error));
    assert!(response.error.is_some());

    serve_handle.abort();
}

#[tokio::test]
async fn test_registry_socket_heartbeat_invalid_primal_id() {
    use biomeos_test_utils::ready_signal;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let (_temp, socket_path, registry) = make_registry("hb-invalid-socket");
    let registry_clone = registry.clone();
    let (ready_tx, ready_rx) = ready_signal();

    let serve_handle = tokio::spawn(async move {
        if let Err(e) = registry_clone.serve_with_ready(ready_tx).await {
            panic!("serve_with_ready failed: {e}");
        }
    });

    tokio::time::timeout(std::time::Duration::from_secs(5), ready_rx.wait())
        .await
        .expect("server startup timed out")
        .expect("server should signal readiness");

    let mut stream = tokio::net::UnixStream::connect(&socket_path)
        .await
        .expect("connect");

    let hb_req = RegistryRequest::Heartbeat {
        request_id: "req-hb-bad".to_string(),
        primal_id: "not a valid id!".to_string(),
    };
    let req_json = serde_json::to_string(&hb_req).expect("serialize");
    stream.write_all(req_json.as_bytes()).await.expect("write");
    stream.write_all(b"\n").await.expect("newline");
    stream.flush().await.expect("flush");

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await.expect("read");
    let response: RegistryResponse = serde_json::from_str(&line).expect("parse");
    assert_eq!(response.request_id, "req-hb-bad");
    assert!(matches!(response.status, ResponseStatus::Error));

    serve_handle.abort();
}

#[tokio::test]
async fn test_registry_socket_list_primals_via_socket() {
    use biomeos_test_utils::ready_signal;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let (_temp, socket_path, registry) = make_registry("list-socket");
    let pid = PrimalId::new("list-a").unwrap();
    registry
        .register(
            pid,
            RegisterParams {
                provides: vec![Capability::Compute],
                requires: vec![],
                socket_path: Some("/tmp/a.sock".to_string()),
                http_endpoint: None,
                metadata: None,
            },
        )
        .await
        .unwrap();

    let registry_clone = registry.clone();
    let (ready_tx, ready_rx) = ready_signal();
    let serve_handle = tokio::spawn(async move {
        if let Err(e) = registry_clone.serve_with_ready(ready_tx).await {
            panic!("serve_with_ready failed: {e}");
        }
    });

    tokio::time::timeout(std::time::Duration::from_secs(5), ready_rx.wait())
        .await
        .expect("server startup timed out")
        .expect("ready");

    let mut stream = tokio::net::UnixStream::connect(&socket_path)
        .await
        .expect("connect");

    let list_req = RegistryRequest::ListPrimals {
        request_id: "req-list".to_string(),
    };
    let req_json = serde_json::to_string(&list_req).expect("serialize");
    stream.write_all(req_json.as_bytes()).await.expect("write");
    stream.write_all(b"\n").await.expect("nl");
    stream.flush().await.expect("flush");

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await.expect("read");
    let response: RegistryResponse = serde_json::from_str(&line).expect("parse");
    assert_eq!(response.request_id, "req-list");
    assert!(matches!(response.status, ResponseStatus::Success));
    let data = response.data.expect("array");
    assert!(data.is_array());

    serve_handle.abort();
}

#[tokio::test]
async fn test_registry_socket_unregister_via_socket() {
    use biomeos_test_utils::ready_signal;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let (_temp, socket_path, registry) = make_registry("unreg-socket");
    let pid = PrimalId::new("unreg-primal").unwrap();
    registry
        .register(
            pid.clone(),
            RegisterParams {
                provides: vec![Capability::Storage],
                requires: vec![],
                socket_path: Some("/tmp/u.sock".to_string()),
                http_endpoint: None,
                metadata: None,
            },
        )
        .await
        .unwrap();

    let registry_clone = registry.clone();
    let (ready_tx, ready_rx) = ready_signal();
    let serve_handle = tokio::spawn(async move {
        if let Err(e) = registry_clone.serve_with_ready(ready_tx).await {
            panic!("serve_with_ready failed: {e}");
        }
    });

    tokio::time::timeout(std::time::Duration::from_secs(5), ready_rx.wait())
        .await
        .expect("server startup timed out")
        .expect("ready");

    let mut stream = tokio::net::UnixStream::connect(&socket_path)
        .await
        .expect("connect");

    let unreg = RegistryRequest::Unregister {
        request_id: "req-unreg".to_string(),
        primal_id: "unreg-primal".to_string(),
    };
    let req_json = serde_json::to_string(&unreg).expect("serialize");
    stream.write_all(req_json.as_bytes()).await.expect("write");
    stream.write_all(b"\n").await.expect("nl");
    stream.flush().await.expect("flush");

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await.expect("read");
    let response: RegistryResponse = serde_json::from_str(&line).expect("parse");
    assert_eq!(response.request_id, "req-unreg");
    assert!(matches!(response.status, ResponseStatus::Success));

    assert!(
        registry
            .get_provider(&Capability::Storage)
            .await
            .unwrap()
            .is_none()
    );

    serve_handle.abort();
}

#[tokio::test]
async fn test_registry_socket_get_provider_not_found() {
    use biomeos_test_utils::ready_signal;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let (_temp, socket_path, registry) = make_registry("nf-socket");
    let registry_clone = registry.clone();
    let (ready_tx, ready_rx) = ready_signal();
    let serve_handle = tokio::spawn(async move {
        if let Err(e) = registry_clone.serve_with_ready(ready_tx).await {
            panic!("serve_with_ready failed: {e}");
        }
    });

    tokio::time::timeout(std::time::Duration::from_secs(5), ready_rx.wait())
        .await
        .expect("server startup timed out")
        .expect("ready");

    let mut stream = tokio::net::UnixStream::connect(&socket_path)
        .await
        .expect("connect");

    let get_req = RegistryRequest::GetProvider {
        request_id: "req-nf".to_string(),
        capability: Capability::Security,
    };
    stream
        .write_all(format!("{}\n", serde_json::to_string(&get_req).unwrap()).as_bytes())
        .await
        .expect("write");
    stream.flush().await.expect("flush");

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await.expect("read");
    let response: RegistryResponse = serde_json::from_str(&line).expect("parse");
    assert_eq!(response.request_id, "req-nf");
    assert!(matches!(response.status, ResponseStatus::NotFound));

    serve_handle.abort();
}

#[tokio::test]
async fn test_registry_socket_heartbeat_unknown_primal() {
    use biomeos_test_utils::ready_signal;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let (_temp, socket_path, registry) = make_registry("hb-miss");
    let registry_clone = registry.clone();
    let (ready_tx, ready_rx) = ready_signal();
    let serve_handle = tokio::spawn(async move {
        if let Err(e) = registry_clone.serve_with_ready(ready_tx).await {
            panic!("serve_with_ready failed: {e}");
        }
    });

    tokio::time::timeout(std::time::Duration::from_secs(5), ready_rx.wait())
        .await
        .expect("server startup timed out")
        .expect("ready");

    let mut stream = tokio::net::UnixStream::connect(&socket_path)
        .await
        .expect("connect");

    let hb_req = RegistryRequest::Heartbeat {
        request_id: "req-hb-miss".to_string(),
        primal_id: "beardog-localhost".to_string(),
    };
    stream
        .write_all(format!("{}\n", serde_json::to_string(&hb_req).unwrap()).as_bytes())
        .await
        .expect("write");
    stream.flush().await.expect("flush");

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await.expect("read");
    let response: RegistryResponse = serde_json::from_str(&line).expect("parse");
    assert_eq!(response.request_id, "req-hb-miss");
    assert!(matches!(response.status, ResponseStatus::Error));

    serve_handle.abort();
}

#[tokio::test]
async fn test_registry_socket_malformed_line_then_valid_request() {
    use biomeos_test_utils::ready_signal;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};

    let (_temp, socket_path, registry) = make_registry("malformed-line");
    let registry_clone = registry.clone();
    let (ready_tx, ready_rx) = ready_signal();
    let serve_handle = tokio::spawn(async move {
        if let Err(e) = registry_clone.serve_with_ready(ready_tx).await {
            panic!("serve_with_ready failed: {e}");
        }
    });

    tokio::time::timeout(std::time::Duration::from_secs(5), ready_rx.wait())
        .await
        .expect("server startup timed out")
        .expect("ready");

    let mut stream = tokio::net::UnixStream::connect(&socket_path)
        .await
        .expect("connect");

    stream
        .write_all(b"not valid json for registry\n")
        .await
        .expect("write garbage");
    let list_req = RegistryRequest::ListPrimals {
        request_id: "req-after-garbage".to_string(),
    };
    stream
        .write_all(format!("{}\n", serde_json::to_string(&list_req).unwrap()).as_bytes())
        .await
        .expect("write valid");
    stream.flush().await.expect("flush");

    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await.expect("read");
    let response: RegistryResponse = serde_json::from_str(&line).expect("parse");
    assert_eq!(response.request_id, "req-after-garbage");
    assert!(matches!(response.status, ResponseStatus::Success));

    serve_handle.abort();
}
