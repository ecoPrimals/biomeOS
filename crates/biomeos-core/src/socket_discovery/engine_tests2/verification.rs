// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::engine::SocketDiscovery;

#[tokio::test]
async fn test_verify_unix_socket_connects_to_bound_listener() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let path = temp_dir.path().join("verify-live.sock");
    let _listener = tokio::net::UnixListener::bind(&path).expect("bind unix listener");
    let discovery = SocketDiscovery::new("test");
    assert!(
        discovery.verify_unix_socket(&path).await,
        "listener should accept verify_unix_socket probe"
    );
}

#[tokio::test]
async fn test_verify_tcp_connection_refused_fast() {
    let discovery = SocketDiscovery::new("test");
    assert!(!discovery.verify_tcp_connection("127.0.0.1", 59998).await);
}

#[tokio::test]
async fn test_verify_tcp_connection_accepts_open_port() {
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind");
    let port = listener.local_addr().expect("addr").port();
    tokio::spawn(async move {
        let _ = listener.accept().await;
    });
    let discovery = SocketDiscovery::new("tcp-verify");
    assert!(
        discovery.verify_tcp_connection("127.0.0.1", port).await,
        "expected successful TCP probe"
    );
}

#[tokio::test]
async fn test_verify_unix_socket_accepts_bound_listener() {
    let temp = tempfile::TempDir::new().unwrap();
    let path = temp.path().join("verify.sock");
    let _listener = tokio::net::UnixListener::bind(&path).expect("bind unix");
    let discovery = SocketDiscovery::new("fam");
    assert!(
        discovery.verify_unix_socket(&path).await,
        "listener socket should accept connections"
    );
}

#[tokio::test]
async fn verify_tcp_connection_fails_on_unused_port() {
    let d = SocketDiscovery::new("f");
    assert!(
        !d.verify_tcp_connection("127.0.0.1", 1).await,
        "port 1 is not a typical listening service in tests"
    );
}
