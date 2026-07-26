use crate::TransportEndpoint;
use crate::atomic_primal_client::AtomicPrimalClient;
use std::sync::Arc;

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
    let client = AtomicPrimalClient::tcp("beardog", "192.0.2.100", 9100);
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
    let client = AtomicPrimalClient::tcp("beardog", "192.0.2.100", 9100);
    let endpoint = client.endpoint();
    if let TransportEndpoint::TcpSocket { host, port } = endpoint {
        assert_eq!(host.as_ref(), "192.0.2.100");
        assert_eq!(*port, 9100);
    } else {
        panic!("Expected TCP endpoint");
    }
}
#[test]
fn test_atomic_primal_client_clone() {
    let client1 = AtomicPrimalClient::unix("beardog", "/tmp/beardog.sock");
    let client2 = client1.clone();

    assert_eq!(client1.primal_name(), client2.primal_name());
    assert_eq!(client1.endpoint(), client2.endpoint());
}
#[test]
fn test_atomic_primal_client_empty_name() {
    let client = AtomicPrimalClient::unix("", "/tmp/test.sock");
    assert_eq!(client.primal_name(), "");
}
