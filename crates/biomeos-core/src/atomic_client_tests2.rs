// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Continuation of AtomicClient tests (split from `atomic_client_tests.rs`).
#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use crate::TransportEndpoint;
use crate::atomic_client::{
    AtomicClient, DiscoverByCapabilityOpts, DiscoverOpts, discover_primal_endpoint,
    discover_primal_endpoint_with_opts,
};
use crate::atomic_primal_client::{AtomicPrimalClient, ExecutionResult};
use biomeos_test_utils::ready_signal;
use serde_json::{Value, json};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

#[tokio::test]
async fn test_discover_primal_endpoint_failure() {
    let result = discover_primal_endpoint("nonexistent_primal_xyz_123").await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not found") || err.contains("Primal"));
}

#[tokio::test]
async fn test_atomic_client_discover_failure() {
    let result = AtomicClient::discover("nonexistent_primal_xyz_456").await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not found") || err.contains("Primal"));
}

#[tokio::test]
async fn test_atomic_client_discover_by_capability_failure() {
    let result = AtomicClient::discover_by_capability("nonexistent.capability.xyz.123").await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("No primal found") || err.contains("capability"));
}

#[tokio::test]
async fn test_discover_with_opts_includes_family_id_in_error() {
    let result = AtomicClient::discover_with_opts(
        "totally_missing_primal_zz",
        DiscoverOpts {
            family_id: Some("custom-family-xyz"),
            ..Default::default()
        },
    )
    .await;
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("custom-family-xyz"),
        "error should mention family id: {msg}"
    );
}

#[tokio::test]
async fn test_discover_primal_endpoint_with_opts_family_id() {
    let result = discover_primal_endpoint_with_opts(
        "missing_endpoint_primal_ab",
        DiscoverOpts {
            family_id: Some("fam-endpoint-test"),
            ..Default::default()
        },
    )
    .await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("fam-endpoint-test")
    );
}

#[tokio::test]
async fn test_discover_by_capability_strict_skips_taxonomy() {
    let result = AtomicClient::discover_by_capability_with_opts(
        "nonexistent.strict.cap.123",
        DiscoverByCapabilityOpts {
            strict_discovery: Some(true),
            ..Default::default()
        },
    )
    .await;
    assert!(result.is_err());
}

#[test]
fn test_discover_opts_clone() {
    let mut m = HashMap::new();
    m.insert("K".to_string(), "v".to_string());
    let a = DiscoverOpts {
        family_id: Some("f"),
        env_overrides: Some(&m),
        tcp_tier2_override: Some("h:1"),
    };
    let b = a.clone();
    assert_eq!(a.family_id, b.family_id);
}

#[test]
fn test_discover_by_capability_opts_clone() {
    let a = DiscoverByCapabilityOpts {
        family_id: Some("g"),
        strict_discovery: Some(false),
    };
    assert_eq!(a.clone().strict_discovery, Some(false));
}

#[test]
fn test_atomic_client_from_endpoint_http_preserves_host_port() {
    let ep = TransportEndpoint::HttpJsonRpc {
        host: Arc::from("gw.example"),
        port: 8443,
    };
    let c = AtomicClient::from_endpoint(ep);
    assert!(matches!(
        c.endpoint(),
        TransportEndpoint::HttpJsonRpc { .. }
    ));
    assert!(c.socket_path().as_os_str().is_empty());
}

#[cfg(target_os = "linux")]
#[test]
fn test_atomic_client_abstract_socket_constructor() {
    let c = AtomicClient::abstract_socket("test-abs-name");
    assert!(matches!(
        c.endpoint(),
        TransportEndpoint::AbstractSocket { .. }
    ));
    assert!(c.is_available());
}

#[test]
fn test_atomic_primal_client_http_constructor() {
    let client = AtomicPrimalClient::tcp("beardog", "192.168.1.100", 9100);
    assert_eq!(client.primal_name(), "beardog");
    assert!(client.endpoint().display_string().contains("192.168.1.100"));
}

#[test]
fn test_execution_result_construction() {
    let result = ExecutionResult {
        stdout: "output".to_string(),
        stderr: "errors".to_string(),
        exit_code: Some(0),
    };
    assert_eq!(result.stdout, "output");
    assert_eq!(result.stderr, "errors");
    assert_eq!(result.exit_code, Some(0));
}

#[test]
fn test_execution_result_without_exit_code() {
    let result = ExecutionResult {
        stdout: String::new(),
        stderr: String::new(),
        exit_code: None,
    };
    assert!(result.exit_code.is_none());
}

// ========================================================================
// AtomicClient call error paths (connection refused, socket not found)
// ========================================================================

#[tokio::test]
async fn test_atomic_client_call_connection_refused() {
    let client = AtomicClient::unix("/nonexistent/socket/path/12345.sock")
        .with_timeout(Duration::from_millis(100));

    let result = client.call("ping", Value::Null).await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("Failed")
            || err.contains("connect")
            || err.contains("No such file")
            || err.contains("Connection refused"),
        "Expected connection error, got: {err}"
    );
}

#[tokio::test]
async fn test_atomic_client_try_call_connection_refused() {
    use biomeos_types::IpcError;

    let client = AtomicClient::unix("/nonexistent/socket/path/67890.sock")
        .with_timeout(Duration::from_millis(100));

    let result = client.try_call("ping", Value::Null).await;
    assert!(result.is_err());
    let ipc_err = result.unwrap_err();
    assert!(
        matches!(ipc_err, IpcError::ConnectionFailed { .. }) || ipc_err.is_timeout(),
        "Expected ConnectionFailed or Timeout, got: {ipc_err:?}"
    );
}

#[tokio::test]
async fn test_atomic_client_tcp_connection_refused() {
    let client = AtomicClient::tcp("127.0.0.1", 59999).with_timeout(Duration::from_millis(100));

    let result = client.call("ping", Value::Null).await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_atomic_client_call_timeout() {
    let client =
        AtomicClient::unix("/nonexistent/socket.sock").with_timeout(Duration::from_millis(1));

    let result = client.call("ping", Value::Null).await;
    assert!(result.is_err());
}

#[test]
fn test_execution_result_serialization_roundtrip() {
    let result = ExecutionResult {
        stdout: "out".to_string(),
        stderr: "err".to_string(),
        exit_code: Some(1),
    };
    let json = serde_json::to_string(&result).expect("serialize");
    let parsed: ExecutionResult = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.stdout, result.stdout);
    assert_eq!(parsed.stderr, result.stderr);
    assert_eq!(parsed.exit_code, result.exit_code);
}

// ========================================================================
// AtomicPrimalClient error path tests
// ========================================================================

#[tokio::test]
async fn test_atomic_primal_client_discover_failure() {
    let result = AtomicPrimalClient::discover("nonexistent_primal_xyz_789").await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not found") || err.contains("Primal"));
}

#[tokio::test]
async fn test_atomic_primal_client_health_check_connection_refused() {
    let client =
        AtomicPrimalClient::unix("test-primal", "/nonexistent/socket/health_check_test.sock");
    let result = client.health_check().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_atomic_primal_client_execute_command_connection_refused() {
    let client = AtomicPrimalClient::unix("test-primal", "/nonexistent/socket/execute_test.sock");
    let result = client.execute_command("echo hello").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_atomic_primal_client_get_identity_connection_refused() {
    let client = AtomicPrimalClient::unix("test-primal", "/nonexistent/socket/identity_test.sock");
    let result = client.get_identity().await;
    assert!(result.is_err());
}

// ========================================================================
// Additional AtomicClient coverage - try_call error paths, call_stream
// ========================================================================

#[tokio::test]
async fn test_try_call_missing_result() {
    use biomeos_types::IpcError;

    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("missing_result.sock");

    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 1024];
            let n = stream.read(&mut buf).await.unwrap_or(0);
            if n > 0 {
                let response = serde_json::json!({
                    "jsonrpc": "2.0",
                    "result": null,
                    "id": 1
                });
                let _ = stream
                    .write_all(
                        format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes(),
                    )
                    .await;
            }
        }
    });

    ready_rx.wait().await.unwrap();

    let client = AtomicClient::unix(&socket_path).with_timeout(Duration::from_secs(2));
    let result = client.try_call("test", Value::Null).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, IpcError::MissingResult { .. }));
}

#[tokio::test]
async fn test_try_call_jsonrpc_error() {
    use biomeos_types::IpcError;

    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("jsonrpc_error.sock");

    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 1024];
            let _ = stream.read(&mut buf).await;
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": -32601, "message": "Method not found"},
                "id": 1
            });
            let _ = stream
                .write_all(format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes())
                .await;
        }
    });

    ready_rx.wait().await.unwrap();

    let client = AtomicClient::unix(&socket_path).with_timeout(Duration::from_secs(2));
    let result = client.try_call("nonexistent", Value::Null).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(matches!(err, IpcError::JsonRpcError { code: -32601, .. }));
}

#[tokio::test]
async fn test_call_stream_connection_refused() {
    let client = AtomicClient::unix("/nonexistent/socket/stream_test.sock")
        .with_timeout(Duration::from_millis(100));

    let mut rx = client
        .call_stream("stream_method", Value::Null)
        .expect("call_stream returns receiver");

    let item = rx.recv().await;
    assert!(item.is_some());
    let item = item.unwrap();
    assert!(
        matches!(item, biomeos_graph::StreamItem::Error { .. })
            || matches!(item, biomeos_graph::StreamItem::End)
    );
}

#[tokio::test]
async fn test_call_success_with_result() {
    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("success.sock");

    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 2048];
            let _ = stream.read(&mut buf).await;
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "result": {"status": "ok", "value": 42},
                "id": 1
            });
            let _ = stream
                .write_all(format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes())
                .await;
        }
    });

    ready_rx.wait().await.unwrap();

    let client = AtomicClient::unix(&socket_path).with_timeout(Duration::from_secs(2));
    let result = client.call("test", Value::Null).await;
    assert!(result.is_ok());
    let value = result.unwrap();
    assert_eq!(value["status"], "ok");
    assert_eq!(value["value"], 42);
}

#[test]
fn test_atomic_client_http_display() {
    let client = AtomicClient::http("api.example.com", 443);
    let endpoint = client.endpoint();
    assert!(
        endpoint.display_string().contains("api.example.com")
            || endpoint.display_string().contains("443")
    );
}

#[cfg(target_os = "linux")]
#[test]
fn test_atomic_client_abstract_socket_linux() {
    let client = AtomicClient::abstract_socket("test-abstract");
    assert!(matches!(
        client.endpoint(),
        TransportEndpoint::AbstractSocket { .. }
    ));
    assert!(client.is_available());
}

/// Server accepts the connection but never sends a line — `try_call` must return `IpcError::Timeout`.
#[tokio::test]
async fn test_try_call_timeout_while_reading_response() {
    use biomeos_types::IpcError;

    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("hang.sock");

    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 256];
            let _ = stream.read(&mut buf).await;
            std::future::pending::<()>().await;
        }
    });

    ready_rx.wait().await.expect("ready");

    let client = AtomicClient::unix(&socket_path).with_timeout(Duration::from_millis(150));
    let result = client.try_call("test", Value::Null).await;
    assert!(result.is_err());
    let err = result.unwrap_err();
    assert!(
        matches!(err, IpcError::Timeout { .. }),
        "expected timeout, got {err:?}"
    );
}

// ========================================================================
// HTTP JSON-RPC, streaming, env discovery
// ========================================================================

#[tokio::test]
async fn test_call_http_jsonrpc_success() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let port = listener.local_addr().expect("addr").port();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 16384];
            let _ = stream.read(&mut buf).await;
            let body = r#"{"jsonrpc":"2.0","result":{"http_ok":true},"id":1}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes()).await;
        }
    });
    tokio::task::yield_now().await;
    let client = AtomicClient::http("127.0.0.1", port);
    let result = client.call("ping", json!({})).await.expect("http call");
    assert_eq!(result["http_ok"], true);
}

#[tokio::test]
async fn test_call_http_malformed_no_separator_fails() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let port = listener.local_addr().expect("addr").port();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 4096];
            let _ = stream.read(&mut buf).await;
            let _ = stream
                .write_all(b"HTTP/1.1 200 OK\r\noops no body sep")
                .await;
        }
    });
    tokio::task::yield_now().await;
    let client = AtomicClient::http("127.0.0.1", port).with_timeout(Duration::from_secs(2));
    let err = client
        .call("m", json!({}))
        .await
        .expect_err("malformed http");
    let s = err.to_string();
    assert!(
        s.contains("Malformed") || s.contains("separator") || s.contains("body"),
        "{s}"
    );
}

#[tokio::test]
async fn test_call_stream_http_yields_transport_error() {
    use biomeos_graph::StreamItem;
    let client = AtomicClient::http("127.0.0.1", 59997).with_timeout(Duration::from_millis(200));
    let mut rx = client.call_stream("stream", json!({})).expect("receiver");
    let first = rx.recv().await.expect("event");
    assert!(
        matches!(first, StreamItem::Error { .. }),
        "expected StreamItem::Error, got {first:?}"
    );
}

#[tokio::test]
async fn test_call_stream_unix_jsonrpc_single_line_wrapped() {
    use biomeos_graph::StreamItem;
    let temp = tempfile::tempdir().expect("temp");
    let socket_path = temp.path().join("stream_wrap.sock");
    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 4096];
            let _ = stream.read(&mut buf).await;
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "result": {"wrapped": 7},
                "id": 1
            });
            let _ = stream
                .write_all(format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes())
                .await;
        }
    });
    ready_rx.wait().await.expect("ready");
    let client = AtomicClient::unix(&socket_path).with_timeout(Duration::from_secs(2));
    let mut rx = client.call_stream("m", json!({})).expect("stream");
    let mut saw_data = false;
    while let Some(item) = rx.recv().await {
        if matches!(item, StreamItem::Data(_)) {
            saw_data = true;
        }
        if matches!(item, StreamItem::End) {
            break;
        }
    }
    assert!(saw_data);
}

#[tokio::test]
async fn test_discover_primal_uses_node_family_id_env() {
    let err = AtomicClient::discover_with_opts(
        "totally_missing_primal_xyz_999",
        DiscoverOpts {
            family_id: Some("from-node-env"),
            ..Default::default()
        },
    )
    .await;
    assert!(err.is_err());
    let msg = err.unwrap_err().to_string();
    assert!(
        msg.contains("from-node-env") || msg.contains("not found"),
        "{msg}"
    );
}

#[tokio::test]
async fn test_discover_by_capability_strict_taxonomy_path() {
    let err = AtomicClient::discover_by_capability("capability.that.does.not.exist.ever").await;
    assert!(err.is_err());
}

#[test]
fn test_transport_endpoint_debug_clone_roundtrip() {
    let e1 = TransportEndpoint::TcpSocket {
        host: Arc::from("h"),
        port: 1,
    };
    let e2 = e1.clone();
    assert_eq!(format!("{e1:?}"), format!("{e2:?}"));
}

#[tokio::test]
async fn test_try_call_jsonrpc_error_includes_code() {
    use biomeos_types::IpcError;
    let temp = tempfile::tempdir().expect("temp");
    let socket_path = temp.path().join("code.sock");
    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 1024];
            let _ = stream.read(&mut buf).await;
            let response = serde_json::json!({
                "jsonrpc": "2.0",
                "error": {"code": -32000, "message": "app err"},
                "id": 1
            });
            let _ = stream
                .write_all(format!("{}\n", serde_json::to_string(&response).unwrap()).as_bytes())
                .await;
        }
    });
    ready_rx.wait().await.expect("ready");
    let client = AtomicClient::unix(&socket_path).with_timeout(Duration::from_secs(2));
    let err = client
        .try_call("x", json!({}))
        .await
        .expect_err("jsonrpc err");
    match err {
        IpcError::JsonRpcError { code, .. } => assert_eq!(code, -32000),
        other => panic!("unexpected {other:?}"),
    }
}

#[tokio::test]
async fn test_call_stream_unix_raw_non_json_line_becomes_string_data() {
    use biomeos_graph::StreamItem;
    let temp = tempfile::tempdir().expect("temp");
    let socket_path = temp.path().join("stream_raw.sock");
    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 4096];
            let _ = stream.read(&mut buf).await;
            let _ = stream.write_all(b"plain-text-not-json\n").await;
            drop(stream);
        }
    });
    ready_rx.wait().await.expect("ready");
    let client = AtomicClient::unix(&socket_path).with_timeout(Duration::from_secs(2));
    let mut rx = client.call_stream("m", json!({})).expect("stream");
    let mut saw_plain = false;
    while let Some(item) = rx.recv().await {
        if let StreamItem::Data(v) = &item {
            if v.as_str() == Some("plain-text-not-json") {
                saw_plain = true;
            }
        }
        if matches!(item, StreamItem::End) {
            break;
        }
    }
    assert!(saw_plain);
}

#[tokio::test]
async fn test_discover_by_capability_strict_env_disables_taxonomy_bootstrap() {
    let err = AtomicClient::discover_by_capability_with_opts(
        "nonexistent.strict.cap",
        DiscoverByCapabilityOpts {
            strict_discovery: Some(true),
            ..Default::default()
        },
    )
    .await;
    assert!(err.is_err());
}

#[test]
fn test_is_available_abstract_linux_only() {
    #[cfg(target_os = "linux")]
    {
        let c = AtomicClient::abstract_socket("abs-name-test");
        assert!(c.is_available());
    }
}

// ========================================================================
// Discovery success via env (TCP), try_call serialization, HTTP edge cases
// ========================================================================

#[tokio::test]
#[expect(clippy::unwrap_used, reason = "test asserts successful discovery")]
async fn test_atomic_client_discover_via_tcp_env_succeeds() {
    let mut m = HashMap::new();
    m.insert("DISCOVERUT_TCP".to_string(), "127.0.0.1:59996".to_string());
    let client = AtomicClient::discover_with_opts(
        "discoverut",
        DiscoverOpts {
            env_overrides: Some(&m),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert!(
        matches!(client.endpoint(), TransportEndpoint::TcpSocket { .. }),
        "expected TCP from env, got {:?}",
        client.endpoint()
    );
    assert!(!client.endpoint().is_native(), "TCP should be Tier 2");
}

#[tokio::test]
#[expect(clippy::unwrap_used, reason = "test asserts successful discovery")]
async fn test_discover_primal_endpoint_via_tcp_env_succeeds() {
    let mut m = HashMap::new();
    m.insert("DISCOVERPE_TCP".to_string(), "127.0.0.1:59995".to_string());
    let ep = discover_primal_endpoint_with_opts(
        "discoverpe",
        DiscoverOpts {
            env_overrides: Some(&m),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert!(matches!(ep, TransportEndpoint::TcpSocket { .. }));
}

#[tokio::test]
async fn test_try_call_jsonrpc_response_invalid_json_serialization() {
    use biomeos_types::IpcError;

    let temp = tempfile::tempdir().expect("temp dir");
    let socket_path = temp.path().join("bad_json.sock");

    let (mut ready_tx, ready_rx) = ready_signal();
    let listener = tokio::net::UnixListener::bind(&socket_path).expect("bind");
    ready_tx.signal();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 1024];
            let _ = stream.read(&mut buf).await;
            let _ = stream.write_all(b"not json at all\n").await;
        }
    });

    ready_rx.wait().await.expect("ready");

    let client = AtomicClient::unix(&socket_path).with_timeout(Duration::from_secs(2));
    let err = client
        .try_call("x", json!({}))
        .await
        .expect_err("invalid JSON line");
    assert!(
        matches!(err, IpcError::Serialization(_)),
        "expected Serialization, got {err:?}"
    );
}

#[tokio::test]
async fn test_call_http_jsonrpc_body_after_lf_only_separator() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let port = listener.local_addr().expect("addr").port();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 16384];
            let _ = stream.read(&mut buf).await;
            let body = r#"{"jsonrpc":"2.0","result":{"lf_sep":true},"id":1}"#;
            let response = format!("HTTP/1.1 200 OK\n\n{}", body);
            let _ = stream.write_all(response.as_bytes()).await;
        }
    });
    tokio::task::yield_now().await;
    let client = AtomicClient::http("127.0.0.1", port);
    let result = client.call("ping", json!({})).await.expect("http call");
    assert_eq!(result["lf_sep"], true);
}

#[tokio::test]
async fn test_call_http_jsonrpc_invalid_body_json_fails() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let port = listener.local_addr().expect("addr").port();
    tokio::spawn(async move {
        if let Ok((mut stream, _)) = listener.accept().await {
            let mut buf = vec![0u8; 4096];
            let _ = stream.read(&mut buf).await;
            let body = "not-json";
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                body.len(),
                body
            );
            let _ = stream.write_all(response.as_bytes()).await;
        }
    });
    tokio::task::yield_now().await;
    let client = AtomicClient::http("127.0.0.1", port).with_timeout(Duration::from_secs(2));
    let err = client
        .call("m", json!({}))
        .await
        .expect_err("bad json body");
    let s = err.to_string();
    assert!(
        s.contains("serialization") || s.contains("parse") || s.contains("JSON"),
        "{s}"
    );
}
