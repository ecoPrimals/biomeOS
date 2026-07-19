// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::capability_registry::*;
use super::common::make_registry;
use crate::Capability;
use biomeos_types::PrimalId;

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
