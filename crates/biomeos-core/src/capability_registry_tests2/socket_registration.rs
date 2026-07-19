// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::capability_registry::*;
use super::common::make_registry;
use crate::Capability;
use biomeos_types::PrimalId;

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
