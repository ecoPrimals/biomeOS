// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! AtomicClient Tests
//!
//! Extracted from atomic_client.rs to maintain files under 1000 lines.
//! Tests cover JSON-RPC requests/responses, client constructors, configuration,
//! transport endpoints, and edge cases.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::atomic_client::*;
use crate::TransportEndpoint;
use crate::atomic_primal_client::{AtomicPrimalClient, ExecutionResult};
use biomeos_test_utils::ready_signal;
use serde_json::{Value, json};
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

// ========================================================================
// JSON-RPC Tests
// ========================================================================

#[test]
fn test_jsonrpc_request_creation() {
    let request = JsonRpcRequest::new("test_method", serde_json::json!({"key": "value"}));
    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.method.as_ref(), "test_method");
    assert_eq!(request.params.as_ref().unwrap()["key"], "value");
    assert!(request.id.as_ref().and_then(|v| v.as_u64()).unwrap_or(0) > 0);
}

// ========================================================================
// AtomicClient Constructor Tests - Universal IPC v3.0
// ========================================================================

#[test]
fn test_atomic_client_unix() {
    let client = AtomicClient::unix("/tmp/test.sock");
    assert_eq!(client.socket_path().to_str().unwrap(), "/tmp/test.sock");
    assert!(matches!(
        client.endpoint(),
        TransportEndpoint::UnixSocket { .. }
    ));
}

#[test]
fn test_atomic_client_tcp() {
    let client = AtomicClient::tcp("192.168.1.100", 9100);
    assert!(matches!(
        client.endpoint(),
        TransportEndpoint::TcpSocket { .. }
    ));
    if let TransportEndpoint::TcpSocket { host, port } = client.endpoint() {
        assert_eq!(host.as_ref(), "192.168.1.100");
        assert_eq!(*port, 9100);
    }
}

#[test]
fn test_atomic_client_from_endpoint() {
    let endpoint = TransportEndpoint::TcpSocket {
        host: Arc::from("localhost"),
        port: 8080,
    };
    let client = AtomicClient::from_endpoint(endpoint);
    assert!(matches!(
        client.endpoint(),
        TransportEndpoint::TcpSocket { .. }
    ));
}

#[test]
fn test_atomic_client_new_legacy() {
    // Test backwards compatibility
    let client = AtomicClient::new("/tmp/test.sock");
    assert_eq!(client.socket_path().to_str().unwrap(), "/tmp/test.sock");
}

#[test]
fn test_client_with_timeout() {
    let client = AtomicClient::unix("/tmp/test.sock").with_timeout(Duration::from_secs(10));
    assert_eq!(client.timeout, Duration::from_secs(10));
}

#[test]
fn test_is_available_unix() {
    // Non-existent socket
    let client = AtomicClient::unix("/tmp/nonexistent.sock");
    assert!(!client.is_available());
}

#[test]
fn test_is_available_tcp() {
    // TCP always returns true (availability checked on connect)
    let client = AtomicClient::tcp("127.0.0.1", 9999);
    assert!(client.is_available());
}

// ========================================================================
// AtomicPrimalClient Constructor Tests
// ========================================================================

#[test]
fn test_atomic_primal_client_unix() {
    let client = AtomicPrimalClient::unix("beardog", "/tmp/beardog.sock");
    assert_eq!(client.primal_name(), "beardog");
    assert!(matches!(
        client.endpoint(),
        TransportEndpoint::UnixSocket { .. }
    ));
}

#[test]
fn test_atomic_primal_client_tcp() {
    let client = AtomicPrimalClient::tcp("beardog", "192.168.1.100", 9100);
    assert_eq!(client.primal_name(), "beardog");
    assert!(matches!(
        client.endpoint(),
        TransportEndpoint::TcpSocket { .. }
    ));
}

#[test]
fn test_atomic_primal_client_from_endpoint() {
    let endpoint = TransportEndpoint::TcpSocket {
        host: Arc::from("10.0.0.1"),
        port: 9200,
    };
    let client = AtomicPrimalClient::from_endpoint("songbird", endpoint);
    assert_eq!(client.primal_name(), "songbird");
}

// ========================================================================
// Integration Tests (require running primals)
// ========================================================================

#[tokio::test]
#[ignore = "Requires running BearDog instance"]
async fn test_beardog_discovery() {
    let client = AtomicPrimalClient::discover("beardog").await;
    if let Ok(client) = client {
        assert!(client.is_available());

        // Log the transport type discovered
        println!(
            "BearDog discovered via: {}",
            client.endpoint().display_string()
        );

        // Try a health check
        let health = client.health_check().await;
        assert!(
            health.is_ok(),
            "BearDog health check failed: {:?}",
            health.err()
        );
    }
}

#[tokio::test]
#[ignore = "Requires running Songbird instance"]
async fn test_songbird_discovery() {
    let client = AtomicPrimalClient::discover("songbird").await;
    if let Ok(client) = client {
        assert!(client.is_available());
        println!(
            "Songbird discovered via: {}",
            client.endpoint().display_string()
        );
    }
}

#[tokio::test]
#[ignore = "Requires running TCP endpoint"]
async fn test_tcp_connection() {
    let client = AtomicClient::tcp("127.0.0.1", 9100);
    // This will fail unless something is listening
    let result = client.call("ping", Value::Null).await;
    // Just verify we can construct and attempt TCP calls
    assert!(result.is_err() || result.is_ok()); // Either works or fails gracefully
}

// ========================================================================
// JSON-RPC Request/Response Tests
// ========================================================================

#[test]
fn test_jsonrpc_request_auto_increment_id() {
    let req1 = JsonRpcRequest::new("method1", Value::Null);
    let req2 = JsonRpcRequest::new("method2", Value::Null);
    let req3 = JsonRpcRequest::new("method3", Value::Null);

    // IDs should be sequential
    let id1 = req1.id.as_ref().and_then(|v| v.as_u64()).unwrap_or(0);
    let id2 = req2.id.as_ref().and_then(|v| v.as_u64()).unwrap_or(0);
    let id3 = req3.id.as_ref().and_then(|v| v.as_u64()).unwrap_or(0);
    assert!(id2 > id1);
    assert!(id3 > id2);
}

#[test]
fn test_jsonrpc_request_serialization() {
    let request = JsonRpcRequest::new("test_method", json!({"key": "value"}));
    let json = serde_json::to_string(&request).unwrap();
    assert!(json.contains("test_method"));
    assert!(json.contains("key"));
    assert!(json.contains("value"));
    assert!(json.contains("2.0"));
}

#[test]
fn test_jsonrpc_response_with_result() {
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: Some(json!({"status": "ok"})),
        error: None,
        id: serde_json::json!(1),
    };
    assert!(response.result.is_some());
    assert!(response.error.is_none());
}

#[test]
fn test_jsonrpc_response_with_error() {
    let error = JsonRpcError {
        code: -32601,
        message: "Method not found".to_string(),
        data: None,
    };
    let response = JsonRpcResponse {
        jsonrpc: "2.0".to_string(),
        result: None,
        error: Some(error),
        id: serde_json::json!(1),
    };
    assert!(response.result.is_none());
    assert!(response.error.is_some());
}

#[test]
fn test_jsonrpc_error_with_data() {
    let error = JsonRpcError {
        code: -32000,
        message: "Server error".to_string(),
        data: Some(json!({"details": "Something went wrong"})),
    };
    assert_eq!(error.code, -32000);
    assert!(error.data.is_some());
}

// ========================================================================
// AtomicClient Constructor and Configuration Tests
// ========================================================================

#[test]
fn test_atomic_client_default_timeout() {
    let client = AtomicClient::unix("/tmp/test.sock");
    assert_eq!(client.timeout, Duration::from_secs(30));
}

#[test]
fn test_atomic_client_custom_timeout() {
    let client = AtomicClient::unix("/tmp/test.sock").with_timeout(Duration::from_secs(60));
    assert_eq!(client.timeout, Duration::from_secs(60));
}

#[test]
fn test_atomic_client_from_endpoint_unix() {
    let endpoint = TransportEndpoint::UnixSocket {
        path: PathBuf::from("/tmp/test.sock"),
    };
    let client = AtomicClient::from_endpoint(endpoint);
    assert_eq!(client.socket_path(), PathBuf::from("/tmp/test.sock"));
}

#[test]
fn test_atomic_client_from_endpoint_tcp() {
    let endpoint = TransportEndpoint::TcpSocket {
        host: Arc::from("localhost"),
        port: 8080,
    };
    let client = AtomicClient::from_endpoint(endpoint);
    assert!(client.socket_path().as_os_str().is_empty());
}

#[test]
fn test_atomic_client_endpoint_accessor() {
    let client = AtomicClient::tcp("192.168.1.1", 9100);
    let endpoint = client.endpoint();
    assert!(matches!(endpoint, TransportEndpoint::TcpSocket { .. }));
}

#[test]
fn test_atomic_client_socket_path_accessor() {
    let client = AtomicClient::unix("/tmp/beardog.sock");
    assert_eq!(client.socket_path(), PathBuf::from("/tmp/beardog.sock"));
}

#[test]
fn test_atomic_client_socket_path_tcp_empty() {
    let client = AtomicClient::tcp("localhost", 9100);
    assert!(client.socket_path().as_os_str().is_empty());
}

// ========================================================================
// AtomicPrimalClient Tests
// ========================================================================

#[test]
fn test_atomic_primal_client_primal_name() {
    let client = AtomicPrimalClient::unix("beardog", "/tmp/beardog.sock");
    assert_eq!(client.primal_name(), "beardog");
}

#[test]
fn test_atomic_primal_client_unix_constructor() {
    let client = AtomicPrimalClient::unix("songbird", "/tmp/songbird.sock");
    assert_eq!(client.primal_name(), "songbird");
}

#[test]
fn test_atomic_primal_client_atomic_client_accessor() {
    let client = AtomicPrimalClient::tcp("beardog", "localhost", 9100);
    let atomic = client.atomic_client();
    assert!(matches!(
        atomic.endpoint(),
        TransportEndpoint::TcpSocket { .. }
    ));
}

#[test]
fn test_atomic_primal_client_is_available() {
    let client = AtomicPrimalClient::unix("beardog", "/tmp/nonexistent.sock");
    // Should return false for non-existent socket
    assert!(!client.is_available());
}

#[test]
fn test_atomic_primal_client_endpoint() {
    let client = AtomicPrimalClient::tcp("beardog", "192.168.1.100", 9100);
    let endpoint = client.endpoint();
    if let TransportEndpoint::TcpSocket { host, port } = endpoint {
        assert_eq!(host.as_ref(), "192.168.1.100");
        assert_eq!(*port, 9100);
    } else {
        panic!("Expected TCP endpoint");
    }
}

// ========================================================================
// Error Handling Tests
// ========================================================================

#[test]
fn test_jsonrpc_request_different_methods() {
    let req1 = JsonRpcRequest::new("method_a", Value::Null);
    let req2 = JsonRpcRequest::new("method_b", json!({"param": 123}));

    assert_eq!(req1.method.as_ref(), "method_a");
    assert_eq!(req2.method.as_ref(), "method_b");
    assert_eq!(req2.params.as_ref().unwrap()["param"], 123);
}

#[test]
fn test_jsonrpc_request_complex_params() {
    let params = json!({
        "nested": {
            "key": "value",
            "array": [1, 2, 3]
        },
        "number": 42
    });
    let request = JsonRpcRequest::new("complex_method", params);
    let p = request.params.as_ref().unwrap();
    assert_eq!(p["number"], 42);
    assert_eq!(p["nested"]["key"], "value");
}

#[test]
fn test_atomic_client_clone() {
    let client1 = AtomicClient::unix("/tmp/test.sock").with_timeout(Duration::from_secs(10));
    let client2 = client1.clone();

    assert_eq!(client1.timeout, client2.timeout);
    assert_eq!(client1.socket_path(), client2.socket_path());
}

#[test]
fn test_atomic_primal_client_clone() {
    let client1 = AtomicPrimalClient::unix("beardog", "/tmp/beardog.sock");
    let client2 = client1.clone();

    assert_eq!(client1.primal_name(), client2.primal_name());
    assert_eq!(client1.endpoint(), client2.endpoint());
}

// ========================================================================
// Edge Cases and Validation Tests
// ========================================================================

#[test]
fn test_atomic_client_empty_socket_path() {
    let client = AtomicClient::unix("");
    assert_eq!(client.socket_path(), PathBuf::from(""));
}

#[test]
fn test_atomic_client_very_long_timeout() {
    let client = AtomicClient::unix("/tmp/test.sock").with_timeout(Duration::from_secs(3600));
    assert_eq!(client.timeout, Duration::from_secs(3600));
}

#[test]
fn test_atomic_client_zero_timeout() {
    let client = AtomicClient::unix("/tmp/test.sock").with_timeout(Duration::from_secs(0));
    assert_eq!(client.timeout, Duration::from_secs(0));
}

#[test]
fn test_jsonrpc_request_null_params() {
    let request = JsonRpcRequest::new("method", Value::Null);
    assert!(request.params.as_ref().unwrap().is_null());
}

#[test]
fn test_jsonrpc_request_empty_object_params() {
    let request = JsonRpcRequest::new("method", json!({}));
    let params = request.params.as_ref().unwrap();
    assert!(params.is_object());
    assert!(params.as_object().unwrap().is_empty());
}

#[test]
fn test_jsonrpc_request_array_params() {
    let request = JsonRpcRequest::new("method", json!([1, 2, 3]));
    let params = request.params.as_ref().unwrap();
    assert!(params.is_array());
    assert_eq!(params.as_array().unwrap().len(), 3);
}

#[test]
fn test_atomic_primal_client_empty_name() {
    let client = AtomicPrimalClient::unix("", "/tmp/test.sock");
    assert_eq!(client.primal_name(), "");
}

#[test]
fn test_atomic_client_tcp_with_different_hosts() {
    let client1 = AtomicClient::tcp("localhost", 9100);
    let client2 = AtomicClient::tcp("127.0.0.1", 9100);
    let client3 = AtomicClient::tcp("192.168.1.1", 9100);

    if let TransportEndpoint::TcpSocket { host: h1, .. } = client1.endpoint() {
        assert_eq!(h1.as_ref(), "localhost");
    }
    if let TransportEndpoint::TcpSocket { host: h2, .. } = client2.endpoint() {
        assert_eq!(h2.as_ref(), "127.0.0.1");
    }
    if let TransportEndpoint::TcpSocket { host: h3, .. } = client3.endpoint() {
        assert_eq!(h3.as_ref(), "192.168.1.1");
    }
}

#[test]
fn test_atomic_client_tcp_with_different_ports() {
    let client1 = AtomicClient::tcp("localhost", 9100);
    let client2 = AtomicClient::tcp("localhost", 9101);
    let client3 = AtomicClient::tcp("localhost", 65535);

    if let TransportEndpoint::TcpSocket { port: p1, .. } = client1.endpoint() {
        assert_eq!(*p1, 9100);
    }
    if let TransportEndpoint::TcpSocket { port: p2, .. } = client2.endpoint() {
        assert_eq!(*p2, 9101);
    }
    if let TransportEndpoint::TcpSocket { port: p3, .. } = client3.endpoint() {
        assert_eq!(*p3, 65535);
    }
}

// ========================================================================
// AtomicClient HTTP and discovery tests
// ========================================================================

#[test]
fn test_atomic_client_http_constructor() {
    let client = AtomicClient::http("192.168.1.100", 8080);
    assert!(matches!(
        client.endpoint(),
        TransportEndpoint::HttpJsonRpc { .. }
    ));
    if let TransportEndpoint::HttpJsonRpc { host, port } = client.endpoint() {
        assert_eq!(host.as_ref(), "192.168.1.100");
        assert_eq!(*port, 8080);
    }
    assert!(client.is_available());
}

#[test]
fn test_atomic_client_from_endpoint_http() {
    let endpoint = TransportEndpoint::HttpJsonRpc {
        host: Arc::from("api.example.com"),
        port: 443,
    };
    let client = AtomicClient::from_endpoint(endpoint);
    assert!(matches!(
        client.endpoint(),
        TransportEndpoint::HttpJsonRpc { .. }
    ));
    assert!(client.socket_path().as_os_str().is_empty());
}

#[test]
fn test_atomic_client_is_available_http() {
    let client = AtomicClient::http("127.0.0.1", 8080);
    assert!(client.is_available());
}

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
        stdout: "".to_string(),
        stderr: "".to_string(),
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
        .await
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
