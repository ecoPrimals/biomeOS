// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Discovery tests - extracted to keep discovery/mod.rs under 1000 lines

#![allow(clippy::unwrap_used, clippy::expect_used)]

use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::OnceLock;

use super::*;

/// Serialize tests that mutate `XDG_RUNTIME_DIR` or Songbird-related env vars (parallel runs race).
/// Tokio mutex so async tests can hold the guard across `.await` without blocking the executor.
static DISCOVERY_ENV_MUTEX: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();

fn discovery_env_lock() -> &'static tokio::sync::Mutex<()> {
    DISCOVERY_ENV_MUTEX.get_or_init(|| tokio::sync::Mutex::new(()))
}

#[test]
fn test_endpoint_parsing_unix() {
    let ep = PrimalDiscovery::parse_endpoint("unix:///tmp/test.sock");
    match ep {
        Some(PrimalEndpoint::UnixSocket { path }) => {
            assert_eq!(path, PathBuf::from("/tmp/test.sock"));
        }
        other => panic!("expected UnixSocket, got {other:?}"),
    }
}

#[test]
fn test_endpoint_parsing_udp() {
    let ep = PrimalDiscovery::parse_endpoint("udp://127.0.0.1:8080");
    match ep {
        Some(PrimalEndpoint::Udp { addr }) => {
            assert_eq!(addr.port(), 8080);
            assert_eq!(addr.ip().to_string(), "127.0.0.1");
        }
        other => panic!("expected Udp, got {other:?}"),
    }
}

#[test]
fn test_endpoint_parsing_http() {
    let ep = PrimalDiscovery::parse_endpoint("http://localhost:3000");
    match ep {
        Some(PrimalEndpoint::Http { url }) => {
            assert_eq!(url, "http://localhost:3000");
        }
        other => panic!("expected Http, got {other:?}"),
    }
}

#[test]
fn test_endpoint_parsing_https() {
    let ep = PrimalDiscovery::parse_endpoint("https://example.com/api");
    match ep {
        Some(PrimalEndpoint::Http { url }) => {
            assert_eq!(url, "https://example.com/api");
        }
        other => panic!("expected Http, got {other:?}"),
    }
}

#[test]
fn test_endpoint_parsing_invalid() {
    assert!(PrimalDiscovery::parse_endpoint("ftp://host").is_none());
    assert!(PrimalDiscovery::parse_endpoint("random-string").is_none());
    assert!(PrimalDiscovery::parse_endpoint("").is_none());
}

#[test]
fn test_endpoint_parsing_invalid_udp_addr() {
    assert!(PrimalDiscovery::parse_endpoint("udp://not-an-addr").is_none());
}

#[test]
fn test_primal_endpoint_serde_unix() {
    let ep = PrimalEndpoint::UnixSocket {
        path: PathBuf::from("/tmp/test.sock"),
    };
    let json = serde_json::to_string(&ep).expect("serialize");
    assert!(json.contains("unix_socket"));
    let restored: PrimalEndpoint = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(restored, PrimalEndpoint::UnixSocket { .. }));
}

#[test]
fn test_primal_endpoint_serde_http() {
    let ep = PrimalEndpoint::Http {
        url: "http://example.com".into(),
    };
    let json = serde_json::to_string(&ep).expect("serialize");
    assert!(json.contains("http"));
    let restored: PrimalEndpoint = serde_json::from_str(&json).expect("deserialize");
    match restored {
        PrimalEndpoint::Http { url } => assert_eq!(url, "http://example.com"),
        other => panic!("expected Http, got {other:?}"),
    }
}

#[test]
fn test_primal_endpoint_serde_udp() {
    let addr: SocketAddr = "127.0.0.1:9000".parse().expect("valid addr");
    let ep = PrimalEndpoint::Udp { addr };
    let json = serde_json::to_string(&ep).expect("serialize");
    assert!(json.contains("udp"));
    let restored: PrimalEndpoint = serde_json::from_str(&json).expect("deserialize");
    match restored {
        PrimalEndpoint::Udp { addr: a } => assert_eq!(a.port(), 9000),
        other => panic!("expected Udp, got {other:?}"),
    }
}

#[test]
fn test_primal_endpoint_debug() {
    let ep = PrimalEndpoint::UnixSocket {
        path: PathBuf::from("/a"),
    };
    assert!(format!("{ep:?}").contains("UnixSocket"));
}

#[test]
fn test_primal_endpoint_clone() {
    let ep = PrimalEndpoint::Http {
        url: "http://x".into(),
    };
    let cloned = ep;
    assert!(matches!(cloned, PrimalEndpoint::Http { .. }));
}

#[test]
fn test_discovered_primal_serde_roundtrip() {
    let dp = DiscoveredPrimal {
        name: "beardog".into(),
        primal_type: "security".into(),
        capabilities: CapabilitySet::from_vec(vec![Capability::Storage]),
        endpoints: vec![PrimalEndpoint::UnixSocket {
            path: PathBuf::from("/tmp/beardog.sock"),
        }],
        metadata: HashMap::from([("key".into(), "val".into())]),
    };
    let json = serde_json::to_string(&dp).expect("serialize");
    let restored: DiscoveredPrimal = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(restored.name, "beardog");
    assert_eq!(restored.primal_type, "security");
    assert!(restored.capabilities.has(&Capability::Storage));
    assert_eq!(restored.endpoints.len(), 1);
    assert_eq!(restored.metadata["key"], "val");
}

#[test]
fn test_discovered_primal_clone() {
    let dp = DiscoveredPrimal {
        name: "x".into(),
        primal_type: "y".into(),
        capabilities: CapabilitySet::new(),
        endpoints: vec![],
        metadata: HashMap::new(),
    };
    let cloned = dp;
    assert_eq!(cloned.name, "x");
}

#[test]
fn test_discovered_primal_debug() {
    let dp = DiscoveredPrimal {
        name: "test".into(),
        primal_type: "t".into(),
        capabilities: CapabilitySet::new(),
        endpoints: vec![],
        metadata: HashMap::new(),
    };
    let dbg = format!("{dp:?}");
    assert!(dbg.contains("test"));
    assert!(dbg.contains("DiscoveredPrimal"));
}

#[test]
fn test_primal_discovery_new() {
    let pd = PrimalDiscovery::new();
    assert!(pd.all().is_empty());
}

#[test]
fn test_primal_discovery_default() {
    let pd = PrimalDiscovery::default();
    assert!(pd.all().is_empty());
}

#[test]
fn test_primal_discovery_get_none() {
    let pd = PrimalDiscovery::new();
    assert!(pd.get("unknown").is_none());
}

#[test]
fn test_primal_discovery_with_registered() {
    let mut pd = PrimalDiscovery::new();
    pd.discovered_primals.insert(
        "beardog".into(),
        DiscoveredPrimal {
            name: "beardog".into(),
            primal_type: "security".into(),
            capabilities: CapabilitySet::from_vec(vec![Capability::Storage]),
            endpoints: vec![],
            metadata: HashMap::new(),
        },
    );

    assert!(pd.get("beardog").is_some());
    assert_eq!(pd.get("beardog").expect("should exist").name, "beardog");
    assert_eq!(pd.all().len(), 1);
}

#[test]
fn test_primal_discovery_with_capability() {
    let mut pd = PrimalDiscovery::new();
    pd.discovered_primals.insert(
        "store".into(),
        DiscoveredPrimal {
            name: "store".into(),
            primal_type: "storage".into(),
            capabilities: CapabilitySet::from_vec(vec![Capability::Storage]),
            endpoints: vec![],
            metadata: HashMap::new(),
        },
    );
    pd.discovered_primals.insert(
        "compute".into(),
        DiscoveredPrimal {
            name: "compute".into(),
            primal_type: "compute".into(),
            capabilities: CapabilitySet::from_vec(vec![Capability::Compute]),
            endpoints: vec![],
            metadata: HashMap::new(),
        },
    );

    assert_eq!(pd.with_capability(&Capability::Storage).len(), 1);
    assert_eq!(pd.with_capability(&Capability::Compute).len(), 1);
    assert_eq!(pd.with_capability(&Capability::Voice).len(), 0);
}

#[test]
fn test_register_songbird_peer_full() {
    let mut pd = PrimalDiscovery::new();
    let peer = serde_json::json!({
        "node_id": "beardog:fam1:direct:abc123",
        "family_id": "fam1",
        "capabilities": ["security", "crypto"],
        "endpoints": {
            "unix_socket": "/tmp/beardog.sock",
            "udp": "192.168.1.10:9000"
        }
    });
    pd.register_songbird_peer(&peer);

    assert_eq!(pd.discovered_primals.len(), 1);
    let dp = pd.get("beardog").expect("should exist");
    assert_eq!(dp.primal_type, "remote");
    assert_eq!(dp.endpoints.len(), 2);
    assert_eq!(dp.metadata["family_id"], "fam1");
    assert_eq!(dp.metadata["discovered_via"], "songbird_udp");
}

#[test]
fn test_register_songbird_peer_minimal() {
    let mut pd = PrimalDiscovery::new();
    let peer = serde_json::json!({
        "node_id": "songbird"
    });
    pd.register_songbird_peer(&peer);

    let dp = pd.get("songbird").expect("should exist");
    assert_eq!(dp.name, "songbird");
    assert!(dp.endpoints.is_empty());
    assert_eq!(dp.metadata["family_id"], "unknown");
}

#[test]
fn test_register_songbird_peer_no_node_id() {
    let mut pd = PrimalDiscovery::new();
    let peer = serde_json::json!({"family_id": "x"});
    pd.register_songbird_peer(&peer);
    assert!(pd.discovered_primals.is_empty());
}

#[test]
fn test_register_songbird_peer_with_unix_only() {
    let mut pd = PrimalDiscovery::new();
    let peer = serde_json::json!({
        "node_id": "nestgate:fam:direct:hash",
        "endpoints": {
            "unix_socket": "/run/biomeos/nestgate.sock"
        }
    });
    pd.register_songbird_peer(&peer);

    let dp = pd.get("nestgate").expect("should exist");
    assert_eq!(dp.endpoints.len(), 1);
    assert!(matches!(
        &dp.endpoints[0],
        PrimalEndpoint::UnixSocket { .. }
    ));
}

#[test]
fn test_register_songbird_peer_with_udp_only() {
    let mut pd = PrimalDiscovery::new();
    let peer = serde_json::json!({
        "node_id": "svc",
        "endpoints": {
            "udp": "10.0.0.1:5000"
        }
    });
    pd.register_songbird_peer(&peer);

    let dp = pd.get("svc").expect("should exist");
    assert_eq!(dp.endpoints.len(), 1);
    assert!(matches!(&dp.endpoints[0], PrimalEndpoint::Udp { .. }));
}

#[test]
fn test_register_songbird_peer_invalid_udp() {
    let mut pd = PrimalDiscovery::new();
    let peer = serde_json::json!({
        "node_id": "svc",
        "endpoints": {
            "udp": "not-valid-addr"
        }
    });
    pd.register_songbird_peer(&peer);

    let dp = pd.get("svc").expect("should exist");
    assert!(
        dp.endpoints.is_empty(),
        "invalid UDP addr should be skipped"
    );
}

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
fn test_discover_songbird_socket_not_found() {
    let result = PrimalDiscovery::discover_songbird_socket();
    if let Err(e) = result {
        let err_msg = format!("{e}");
        assert!(err_msg.contains("not found") || err_msg.contains("Songbird"));
    }
}

#[test]
fn test_discover_songbird_socket_from_env_vars() {
    tokio::runtime::Runtime::new()
        .expect("runtime")
        .block_on(async {
            let _lock = discovery_env_lock().lock().await;
            {
                let _sb = biomeos_test_utils::TestEnvGuard::remove("SONGBIRD_SOCKET");
                let _ds = biomeos_test_utils::TestEnvGuard::set(
                    "DISCOVERY_SOCKET",
                    "/tmp/discovery-via-env.sock",
                );
                assert_eq!(
                    PrimalDiscovery::discover_songbird_socket().unwrap(),
                    "/tmp/discovery-via-env.sock"
                );
            }
            {
                let _sb =
                    biomeos_test_utils::TestEnvGuard::set("SONGBIRD_SOCKET", "/tmp/test-sb.sock");
                assert_eq!(
                    PrimalDiscovery::discover_songbird_socket().unwrap(),
                    "/tmp/test-sb.sock"
                );
            }
        });
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
    let _lock = discovery_env_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let xdg = dir.path().join("xdg-run");
    // SystemPaths::get_runtime_dir() uses $XDG_RUNTIME_DIR/biomeos
    let runtime = xdg.join(biomeos_types::primal_names::BIOMEOS);
    std::fs::create_dir_all(&runtime).expect("mkdir");
    let _rt =
        biomeos_test_utils::TestEnvGuard::set("XDG_RUNTIME_DIR", xdg.to_string_lossy().as_ref());
    let _e = biomeos_test_utils::TestEnvGuard::set(
        "PRIMAL_CLI_COVERAGE_TEST_ENDPOINT",
        "unix:///tmp/cli-coverage-primal.sock",
    );
    let mut pd = PrimalDiscovery::new();
    let list = pd.discover().await.expect("discover");
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
    let _lock = discovery_env_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let xdg = dir.path().join("runtime");
    let runtime = xdg.join(biomeos_types::primal_names::BIOMEOS);
    std::fs::create_dir_all(&runtime).expect("mkdir");
    let _xdg =
        biomeos_test_utils::TestEnvGuard::set("XDG_RUNTIME_DIR", xdg.to_string_lossy().as_ref());

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
    pd.discover().await.expect("discover");

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
    let _lock = discovery_env_lock().lock().await;
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("songbird-err.sock");
    let listener = tokio::net::UnixListener::bind(&sock_path).expect("bind");

    let line = r#"{"jsonrpc":"2.0","id":1,"error":{"message":"no peers"}}"#.to_string() + "\n";
    tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        let (read_half, mut write_half) = stream.into_split();
        let mut reader = BufReader::new(read_half);
        let mut req = String::new();
        reader.read_line(&mut req).await.expect("read");
        write_half.write_all(line.as_bytes()).await.expect("write");
    });

    let _sock_guard = biomeos_test_utils::TestEnvGuard::set(
        "SONGBIRD_SOCKET",
        sock_path.to_string_lossy().as_ref(),
    );

    let mut pd = PrimalDiscovery::new();
    let err = pd
        .discover_via_songbird()
        .await
        .expect_err("songbird should return error");
    let msg = format!("{err}");
    assert!(
        msg.contains("Songbird discovery failed") || msg.contains("no peers"),
        "got: {msg}"
    );
}
