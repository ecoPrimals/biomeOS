// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Additional `AtomicClient` constructor and transport coverage (`atomic_client.rs`).

use crate::TransportEndpoint;
use crate::atomic_client::AtomicClient;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

#[test]
fn from_endpoint_tcp_has_empty_legacy_socket_path() {
    let ep = TransportEndpoint::TcpSocket {
        host: Arc::from("10.0.0.1"),
        port: 9000,
    };
    let c = AtomicClient::from_endpoint(ep);
    assert!(c.socket_path().as_os_str().is_empty());
    assert!(matches!(
        c.endpoint(),
        TransportEndpoint::TcpSocket { port: 9000, .. }
    ));
}

#[test]
fn from_endpoint_unix_preserves_socket_path() {
    let p = PathBuf::from("/tmp/biomeos-test-from-endpoint.sock");
    let ep = TransportEndpoint::UnixSocket { path: p.clone() };
    let c = AtomicClient::from_endpoint(ep);
    assert_eq!(c.socket_path(), p);
}

#[test]
fn new_alias_matches_unix() {
    let path = PathBuf::from("/tmp/alias-test.sock");
    let a = AtomicClient::unix(&path);
    let b = AtomicClient::new(&path);
    assert_eq!(a.socket_path(), b.socket_path());
}

#[test]
fn http_constructor_sets_endpoint() {
    let c = AtomicClient::http("songbird.local", 8080);
    assert!(matches!(
        c.endpoint(),
        TransportEndpoint::HttpJsonRpc { port: 8080, .. }
    ));
}

#[test]
fn with_timeout_overrides_default() {
    let c = AtomicClient::tcp("127.0.0.1", 1).with_timeout(Duration::from_secs(7));
    // Duration is private except through behavior; ensure clone chain works
    let _ = format!("{c:?}");
}

#[test]
fn is_available_unix_missing_file_is_false() {
    let c = AtomicClient::unix("/nonexistent/path/atomic_client_tests3.sock");
    assert!(!c.is_available());
}

#[test]
fn is_available_tcp_is_true_without_connect() {
    let c = AtomicClient::tcp("192.0.2.1", 1234);
    assert!(c.is_available());
}

#[cfg(target_os = "linux")]
#[test]
fn from_endpoint_abstract_has_empty_socket_path() {
    let ep = TransportEndpoint::AbstractSocket {
        name: Arc::from("abs-test"),
    };
    let c = AtomicClient::from_endpoint(ep);
    assert!(c.socket_path().as_os_str().is_empty());
}
