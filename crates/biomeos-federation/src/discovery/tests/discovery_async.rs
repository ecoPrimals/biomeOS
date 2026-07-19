// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Discovery tests - extracted to keep discovery/mod.rs under 1000 lines

#![expect(clippy::expect_used, reason = "test assertions")]

use std::net::SocketAddr;
use std::path::PathBuf;

use super::super::*;

#[test]
fn test_primal_info_debug_and_clone() {
    let info = PrimalInfo {
        name: "test".into(),
        primal_type: "storage".into(),
        capabilities: CapabilitySet::new(),
    };
    let cloned = info.clone();
    assert_eq!(cloned.name, "test");
    let dbg = format!("{info:?}");
    assert!(dbg.contains("PrimalInfo"));
}

#[test]
fn test_discover_discovery_provider_not_found() {
    let result = PrimalDiscovery::discover_discovery_provider();
    if let Err(e) = result {
        let err_msg = format!("{e}");
        assert!(err_msg.contains("not found") || err_msg.contains("Discovery"));
    }
}

#[test]
fn test_discover_discovery_provider_from_env_vars() {
    let via_discovery_provider =
        PrimalDiscovery::discover_discovery_provider_with_env(&|key| match key {
            "DISCOVERY_PROVIDER_SOCKET" => Some("/tmp/discovery-via-env.sock".to_string()),
            _ => None,
        })
        .expect("tier-1 discovery provider socket");
    assert_eq!(via_discovery_provider, "/tmp/discovery-via-env.sock");

    let via_legacy_socket =
        PrimalDiscovery::discover_discovery_provider_with_env(&|key| match key {
            "SONGBIRD_SOCKET" => Some("/tmp/test-sb.sock".to_string()),
            _ => None,
        })
        .expect("tier-2 legacy socket env");
    assert_eq!(via_legacy_socket, "/tmp/test-sb.sock");
}

#[test]
fn test_parse_endpoint_empty() {
    assert!(PrimalDiscovery::parse_endpoint("").is_none());
}

#[test]
fn test_parse_endpoint_udp_ipv6() {
    let ep = PrimalDiscovery::parse_endpoint("udp://[::1]:8080");
    assert!(ep.is_some());
}

#[tokio::test]
async fn test_discover_includes_primal_from_env_endpoint() {
    let dir = tempfile::tempdir().expect("tempdir");
    let runtime = dir.path().join(biomeos_types::primal_names::BIOMEOS);
    std::fs::create_dir_all(&runtime).expect("mkdir");
    let mut pd = PrimalDiscovery::new();
    pd.discover_unix_sockets_in(&runtime)
        .await
        .expect("discover unix sockets");
    pd.discover_from_primal_endpoint_pairs(&[(
        "PRIMAL_CLI_COVERAGE_TEST_ENDPOINT",
        "unix:///tmp/cli-coverage-primal.sock",
    )])
    .expect("discover from endpoint pairs");
    let list = pd.all();
    let names: Vec<_> = list.iter().map(|p| p.name.as_str()).collect();
    assert!(
        names.contains(&"cli_coverage_test"),
        "expected env-derived primal name, got {names:?}"
    );
    let p = pd.get("cli_coverage_test").expect("inserted");
    assert_eq!(
        p.metadata.get("discovered_via").map(String::as_str),
        Some("environment")
    );
    assert!(matches!(
        &p.endpoints[0],
        PrimalEndpoint::UnixSocket { path } if path == &PathBuf::from("/tmp/cli-coverage-primal.sock")
    ));
}

#[tokio::test]
async fn test_discover_unix_socket_mock_primal_jsonrpc() {
    let dir = tempfile::tempdir().expect("tempdir");
    let runtime = dir.path().join("sockets");
    std::fs::create_dir_all(&runtime).expect("mkdir");

    let sock_path = runtime.join("mockprimal.sock");
    let listener = tokio::net::UnixListener::bind(&sock_path).expect("bind mock primal");

    let response = serde_json::json!({
        "jsonrpc": "2.0",
        "id": null,
        "result": {
            "name": "mockprimal",
            "primal_type": "test",
            "capabilities": ["storage", "compute"]
        }
    });
    let response_line = serde_json::to_string(&response).expect("serialize") + "\n";

    let mock_handle = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let (mut read_half, mut write_half) = stream.into_split();
        let mut buf = vec![0u8; 8192];
        let _n = read_half.read(&mut buf).await.expect("read req");
        write_half
            .write_all(response_line.as_bytes())
            .await
            .expect("write");
        write_half.flush().await.expect("flush response");
        write_half.shutdown().await.expect("shutdown write half");
    });

    let mut pd = PrimalDiscovery::new();
    pd.discover_unix_sockets_in(&runtime)
        .await
        .expect("discover");

    mock_handle.await.expect("mock server completed");

    let p = pd.get("mockprimal").expect("mock primal registered");
    assert_eq!(p.primal_type, "test");
    assert!(p.capabilities.has(&Capability::Custom("storage".into())));
    assert_eq!(
        p.metadata.get("discovered_via").map(String::as_str),
        Some("unix_socket")
    );
}

#[tokio::test]
async fn test_discover_songbird_jsonrpc_error_path() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("songbird-err.sock");
    let listener = tokio::net::UnixListener::bind(&sock_path).expect("bind");

    let line = r#"{"jsonrpc":"2.0","id":1,"error":{"code":-32000,"message":"no peers"}}"#
        .to_string()
        + "\n";
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        let (read_half, mut write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);
        let mut req = String::new();
        reader.read_line(&mut req).await.expect("read");
        write_half.write_all(line.as_bytes()).await.expect("write");
    });

    let mut pd = PrimalDiscovery::new();
    let err = pd
        .discover_via_discovery_socket_path(sock_path.to_string_lossy().as_ref())
        .await
        .expect_err("songbird should return error");
    let msg = format!("{err}");
    assert!(
        msg.contains("list_peers failed") || msg.contains("no peers"),
        "got: {msg}"
    );
}
