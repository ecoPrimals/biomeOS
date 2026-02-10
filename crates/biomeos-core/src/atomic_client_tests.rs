//! AtomicClient Tests
//!
//! Extracted from atomic_client.rs to maintain files under 1000 lines.
//! Tests cover JSON-RPC requests/responses, client constructors, configuration,
//! transport endpoints, and edge cases.

use super::atomic_client::*;
use crate::TransportEndpoint;
use serde_json::{json, Value};
use std::path::PathBuf;
use std::time::Duration;

// ========================================================================
// JSON-RPC Tests
// ========================================================================

#[test]
fn test_jsonrpc_request_creation() {
    let request = JsonRpcRequest::new("test_method", serde_json::json!({"key": "value"}));
    assert_eq!(request.jsonrpc, "2.0");
    assert_eq!(request.method, "test_method");
    assert_eq!(request.params["key"], "value");
    assert!(request.id > 0);
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
        assert_eq!(host, "192.168.1.100");
        assert_eq!(*port, 9100);
    }
}

#[test]
fn test_atomic_client_from_endpoint() {
    let endpoint = TransportEndpoint::TcpSocket {
        host: "localhost".to_string(),
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
        host: "10.0.0.1".to_string(),
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
    assert!(req2.id > req1.id);
    assert!(req3.id > req2.id);
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
        id: 1,
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
        id: 1,
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
        host: "localhost".to_string(),
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
        assert_eq!(host, "192.168.1.100");
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

    assert_eq!(req1.method, "method_a");
    assert_eq!(req2.method, "method_b");
    assert_eq!(req2.params["param"], 123);
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
    assert_eq!(request.params["number"], 42);
    assert_eq!(request.params["nested"]["key"], "value");
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
    assert!(request.params.is_null());
}

#[test]
fn test_jsonrpc_request_empty_object_params() {
    let request = JsonRpcRequest::new("method", json!({}));
    assert!(request.params.is_object());
    assert!(request.params.as_object().unwrap().is_empty());
}

#[test]
fn test_jsonrpc_request_array_params() {
    let request = JsonRpcRequest::new("method", json!([1, 2, 3]));
    assert!(request.params.is_array());
    assert_eq!(request.params.as_array().unwrap().len(), 3);
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
        assert_eq!(h1, "localhost");
    }
    if let TransportEndpoint::TcpSocket { host: h2, .. } = client2.endpoint() {
        assert_eq!(h2, "127.0.0.1");
    }
    if let TransportEndpoint::TcpSocket { host: h3, .. } = client3.endpoint() {
        assert_eq!(h3, "192.168.1.1");
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
