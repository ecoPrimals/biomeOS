// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::engine::SocketDiscovery;
use super::super::transport::TransportEndpoint;
use std::collections::HashMap;

#[tokio::test]
async fn test_discover_endpoint_via_env_generic_endpoint_unix() {
    let temp_dir = tempfile::TempDir::new().unwrap();
    let socket_path = temp_dir.path().join("via-endpoint.sock");
    std::fs::File::create(&socket_path).unwrap();

    let env_overrides: HashMap<String, String> = [(
        "MY_PRIMAL_ENDPOINT".to_string(),
        socket_path.to_string_lossy().to_string(),
    )]
    .into();
    let discovery = SocketDiscovery::new("test");
    let result = discovery.discover_endpoint_via_env_with("my-primal", Some(&env_overrides));

    assert!(result.is_some());
    if let Some(TransportEndpoint::UnixSocket { path }) = result {
        assert_eq!(path, socket_path);
    } else {
        panic!("expected Unix endpoint from MY_PRIMAL_ENDPOINT");
    }
}

#[tokio::test]
async fn test_discover_endpoint_via_env_tcp_prefix_fallback() {
    let env_overrides: HashMap<String, String> =
        [("BEARDOG_TCP".to_string(), "127.0.0.1:19100".to_string())].into();
    let discovery = SocketDiscovery::new("test");
    let result = discovery.discover_endpoint_via_env_with("beardog", Some(&env_overrides));

    assert!(result.is_some());
    if let Some(TransportEndpoint::TcpSocket { host, port }) = result {
        assert_eq!(host.as_ref(), "127.0.0.1");
        assert_eq!(port, 19100);
    }
}

#[tokio::test]
async fn test_discover_endpoint_via_env_unix_missing_file_skipped() {
    let env_overrides: HashMap<String, String> = [(
        "FOO_SOCKET".to_string(),
        "/nonexistent/path/to/missing.sock".to_string(),
    )]
    .into();
    let discovery = SocketDiscovery::new("test");
    let result = discovery.discover_endpoint_via_env_with("foo", Some(&env_overrides));
    assert!(result.is_none());
}

#[tokio::test]
async fn test_discover_endpoint_via_env_tcp_non_matching_parse_returns_none() {
    let env_overrides: HashMap<String, String> =
        [("FOO_TCP".to_string(), "not-a-tcp-endpoint".to_string())].into();
    let discovery = SocketDiscovery::new("test");
    let r = discovery.discover_endpoint_via_env_with("foo", Some(&env_overrides));
    assert!(r.is_none());
}

#[tokio::test]
async fn test_discover_endpoint_via_env_bar_live_unix_listener() {
    let temp = tempfile::TempDir::new().unwrap();
    let sock = temp.path().join("ep.sock");
    let _listener = tokio::net::UnixListener::bind(&sock).expect("bind");

    let env_overrides: HashMap<String, String> = [(
        "BAR_ENDPOINT".to_string(),
        sock.to_string_lossy().into_owned(),
    )]
    .into();
    let discovery = SocketDiscovery::new("test");
    let r = discovery.discover_endpoint_via_env_with("bar", Some(&env_overrides));
    assert!(r.is_some());
}

#[tokio::test]
async fn discover_endpoint_via_env_tcp_override_parses() {
    let env_overrides: HashMap<String, String> =
        [("FOO_TCP".to_string(), "127.0.0.1:65534".to_string())].into();
    let discovery = SocketDiscovery::new("test");
    let r = discovery.discover_endpoint_via_env_with("foo", Some(&env_overrides));
    assert!(r.is_some());
}
