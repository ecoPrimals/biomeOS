use super::super::atomic_client::*;
use crate::TransportEndpoint;
use crate::atomic_primal_client::AtomicPrimalClient;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

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
    let client = AtomicClient::tcp("192.0.2.100", 9100);
    assert!(matches!(
        client.endpoint(),
        TransportEndpoint::TcpSocket { .. }
    ));
    if let TransportEndpoint::TcpSocket { host, port } = client.endpoint() {
        assert_eq!(host.as_ref(), "192.0.2.100");
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
    let client = AtomicClient::tcp("192.0.2.1", 9100);
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
fn test_atomic_client_tcp_with_different_hosts() {
    let client1 = AtomicClient::tcp("localhost", 9100);
    let client2 = AtomicClient::tcp("127.0.0.1", 9100);
    let client3 = AtomicClient::tcp("192.0.2.1", 9100);

    if let TransportEndpoint::TcpSocket { host: h1, .. } = client1.endpoint() {
        assert_eq!(h1.as_ref(), "localhost");
    }
    if let TransportEndpoint::TcpSocket { host: h2, .. } = client2.endpoint() {
        assert_eq!(h2.as_ref(), "127.0.0.1");
    }
    if let TransportEndpoint::TcpSocket { host: h3, .. } = client3.endpoint() {
        assert_eq!(h3.as_ref(), "192.0.2.1");
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
    let client = AtomicClient::http("192.0.2.100", 8080);
    assert!(matches!(
        client.endpoint(),
        TransportEndpoint::HttpJsonRpc { .. }
    ));
    if let TransportEndpoint::HttpJsonRpc { host, port } = client.endpoint() {
        assert_eq!(host.as_ref(), "192.0.2.100");
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
