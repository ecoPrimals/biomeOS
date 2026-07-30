// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::*;
use runtime::{
    bootstrap_capability_hint, clear_runtime_capability_cache_for_tests,
    store_runtime_capability_hint,
};

#[test]
fn test_discovery_query_capability() {
    let cap = PrimalCapability::encryption();
    let query = DiscoveryQuery::capability(cap.clone());
    assert_eq!(query.capability, Some(cap));
    assert!(query.healthy_only);
}

#[test]
fn test_discovery_query_primal() {
    let query = DiscoveryQuery::primal("beardog");
    assert_eq!(query.name, Some("beardog".to_string()));
    assert!(query.capability.is_none());
    assert!(!query.healthy_only);
}

#[test]
fn test_discovery_query_default() {
    let query = DiscoveryQuery::default();
    assert!(query.name.is_none());
    assert!(query.capability.is_none());
    assert!(!query.healthy_only);
    assert!(query.limit.is_none());
}

#[test]
fn test_capability_providers_encryption() {
    let providers = providers_for_capability(&PrimalCapability::encryption());
    assert!(providers.contains(&"beardog"));
}

#[test]
fn test_capability_providers_security() {
    let providers = providers_for_capability(&PrimalCapability::new("security", "x", "1.0"));
    assert!(providers.contains(&"beardog"));
}

#[test]
fn test_capability_providers_networking() {
    let providers = providers_for_capability(&PrimalCapability::networking());
    assert!(providers.contains(&"songbird"));
}

#[test]
fn test_capability_providers_compute() {
    let providers = providers_for_capability(&PrimalCapability::compute());
    assert!(providers.contains(&"toadstool"));
}

#[test]
fn test_capability_providers_storage() {
    let providers = providers_for_capability(&PrimalCapability::storage());
    assert!(providers.contains(&"nestgate"));
}

#[test]
fn test_capability_providers_ai() {
    let providers = providers_for_capability(&PrimalCapability::ai());
    assert!(providers.contains(&"squirrel"));
}

#[test]
fn test_capability_providers_science() {
    let providers = providers_for_capability(&PrimalCapability::science());
    assert!(providers.contains(&"wetspring"));
    assert!(providers.contains(&"neuralspring"));
}

#[test]
fn test_capability_providers_unknown_empty() {
    let providers = providers_for_capability(&PrimalCapability::new("unknown", "x", "1.0"));
    assert!(providers.is_empty());
}

#[test]
fn test_capability_from_name_beardog() {
    assert_eq!(bootstrap_capability_hint("beardog").category, "encryption");
}

#[test]
fn test_capability_from_name_songbird() {
    assert_eq!(bootstrap_capability_hint("songbird").category, "networking");
}

#[test]
fn test_capability_from_name_toadstool() {
    assert_eq!(bootstrap_capability_hint("toadstool").category, "compute");
}

#[test]
fn test_capability_from_name_nestgate() {
    assert_eq!(bootstrap_capability_hint("nestgate").category, "storage");
}

#[test]
fn test_capability_from_name_squirrel() {
    assert_eq!(bootstrap_capability_hint("squirrel").category, "ai");
}

#[test]
fn test_capability_from_name_wetspring() {
    assert_eq!(bootstrap_capability_hint("wetspring").category, "science");
}

#[test]
fn test_capability_from_name_neuralspring() {
    assert_eq!(
        bootstrap_capability_hint("neuralspring").category,
        "science"
    );
}

#[test]
fn test_capability_from_name_unknown_custom() {
    let cap = bootstrap_capability_hint("unknownprimal");
    assert_eq!(cap.category, "custom");
    assert_eq!(cap.name, "unknownprimal");
}

#[test]
fn test_capability_from_name_case_insensitive() {
    assert_eq!(bootstrap_capability_hint("BEARDOG").category, "encryption");
}

#[test]
fn test_runtime_cache_overrides_static_hint() {
    clear_runtime_capability_cache_for_tests();
    let learned = PrimalCapability::new("orchestration", "planning", "1.0");
    store_runtime_capability_hint("beardog", learned.clone());
    assert_eq!(bootstrap_capability_hint("beardog"), learned);
    clear_runtime_capability_cache_for_tests();
}

#[tokio::test]
async fn test_discover_uses_runtime_cache_from_capability_probe() {
    clear_runtime_capability_cache_for_tests();
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("beardog-fam.sock");
    let listener = tokio::net::UnixListener::bind(&sock).expect("bind");
    let server = tokio::spawn(async move {
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        let response = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {"capabilities": ["orchestration.planning"]}
        });
        let mut line = serde_json::to_string(&response).expect("json");
        line.push('\n');
        for _ in 0..2 {
            if let Ok((stream, _)) = listener.accept().await {
                let mut reader = BufReader::new(stream);
                let mut request_line = String::new();
                let _ = reader.read_line(&mut request_line).await;
                let _ = reader.get_mut().write_all(line.as_bytes()).await;
            }
        }
    });
    let discovery = PrimalDiscovery::new("fam");
    let p = discovery
        .discover_primal_in("beardog", tmp.path())
        .await
        .expect("primal");
    assert_eq!(p.capability.category, "orchestration");
    assert_eq!(p.capability.name, "planning");
    server.await.expect("server");
    clear_runtime_capability_cache_for_tests();
}

#[test]
fn test_primal_discovery_new() {
    let discovery = PrimalDiscovery::new("my_family");
    // Resolve uses env - just verify construction
    let _ = discovery;
}

#[tokio::test]
async fn test_discover_by_capability_returns_vec() {
    let discovery = PrimalDiscovery::new("test-family");
    // No primals running in test env; should return empty or discovered names
    let result = discovery.discover_by_capability("encryption").await;
    assert!(result.is_ok());
    let names = result.unwrap();
    // Names is Vec<String> - may be empty if no beardog socket
    assert!(names.is_empty() || names.contains(&"beardog".to_string()));
}

#[test]
fn test_discovered_primal_serialization() {
    let primal = DiscoveredPrimal {
        name: "beardog".to_string(),
        socket_path: PathBuf::from("/run/user/1000/biomeos/beardog-default.sock"),
        capability: PrimalCapability::encryption(),
        discovered_via: DiscoveryMethod::XdgRuntime,
        is_healthy: true,
    };
    let json = serde_json::to_string(&primal).unwrap();
    assert!(json.contains("beardog"));
}

#[test]
fn test_discovery_method_enum_roundtrip() {
    for m in [
        DiscoveryMethod::Environment("K".to_string()),
        DiscoveryMethod::XdgRuntime,
        DiscoveryMethod::RunUser,
        DiscoveryMethod::AndroidData,
        DiscoveryMethod::TmpFallback,
        DiscoveryMethod::NeuralApi,
    ] {
        let json = serde_json::to_string(&m).unwrap();
        let _: DiscoveryMethod = serde_json::from_str(&json).unwrap();
    }
}

#[tokio::test]
async fn test_find_by_capability_errors_when_empty_socket_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let err = PrimalDiscovery::find_by_capability_in(PrimalCapability::encryption(), tmp.path())
        .await
        .expect_err("should fail");
    assert!(err.to_string().contains("No primal"));
}

#[tokio::test]
async fn test_discover_primal_resolves_under_biomeos_socket_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("songbird-fam.sock");
    let listener = std::os::unix::net::UnixListener::bind(&sock).expect("bind");
    let discovery = PrimalDiscovery::new("fam");
    let p = discovery
        .discover_primal_in("songbird", tmp.path())
        .await
        .expect("primal");
    assert_eq!(p.name, "songbird");
    assert_eq!(p.socket_path, sock);
    drop(listener);
}

#[tokio::test]
async fn test_discover_alt_socket_name_without_family_suffix() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("toadstool.sock");
    let listener = std::os::unix::net::UnixListener::bind(&sock).expect("bind");
    let discovery = PrimalDiscovery::new("fam");
    let p = discovery
        .discover_primal_in("toadstool", tmp.path())
        .await
        .expect("primal");
    assert_eq!(p.socket_path, sock);
    drop(listener);
}

#[tokio::test]
async fn test_discover_respects_limit() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("beardog-x.sock");
    let listener = std::os::unix::net::UnixListener::bind(&sock).expect("bind");
    let discovery = PrimalDiscovery::new("x");
    let mut q = DiscoveryQuery::capability(PrimalCapability::encryption());
    q.limit = Some(0);
    let v = discovery
        .discover_in(&q, tmp.path())
        .await
        .expect("discover");
    assert!(v.is_empty());
    drop(listener);
}

#[tokio::test]
async fn test_discover_by_capability_taxonomy_empty_dir() {
    let tmp = tempfile::tempdir().unwrap();
    let discovery = PrimalDiscovery::new("x");
    let names = discovery
        .discover_by_capability_in("encryption", tmp.path())
        .await
        .expect("ok");
    assert!(names.is_empty());
}

#[test]
fn test_providers_for_capability_registry_alias() {
    let p = providers_for_capability(&PrimalCapability::new("registry", "r", "1"));
    assert!(!p.is_empty());
}

#[test]
fn test_providers_for_capability_http_alias() {
    let p = providers_for_capability(&PrimalCapability::new("http", "h", "1"));
    assert!(!p.is_empty());
}

#[test]
fn test_providers_for_capability_crypto_alias() {
    let p = providers_for_capability(&PrimalCapability::new("crypto", "c", "1"));
    assert!(!p.is_empty());
}

#[test]
fn test_providers_for_capability_networking_alias() {
    let p = providers_for_capability(&PrimalCapability::new("networking", "n", "1"));
    assert!(!p.is_empty());
}

#[tokio::test]
async fn test_discover_query_by_name_only() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("beardog-fam.sock");
    let _l = std::os::unix::net::UnixListener::bind(&sock).expect("bind");
    let discovery = PrimalDiscovery::new("fam");
    let mut q = DiscoveryQuery::primal("beardog");
    q.limit = Some(5);
    let v = discovery
        .discover_in(&q, tmp.path())
        .await
        .expect("discover");
    assert_eq!(v.len(), 1);
    assert_eq!(v[0].name, "beardog");
}

#[tokio::test]
async fn test_discover_healthy_only_skips_dead_socket_file() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("beardog-fam.sock");
    std::fs::write(&sock, b"not-a-socket").expect("w");
    let discovery = PrimalDiscovery::new("fam");
    let q = DiscoveryQuery::capability(PrimalCapability::encryption());
    let v = discovery
        .discover_in(&q, tmp.path())
        .await
        .expect("discover");
    assert!(v.is_empty());
}

#[tokio::test]
async fn test_discover_capability_unhealthy_included_when_not_healthy_only() {
    let tmp = tempfile::tempdir().unwrap();
    let sock = tmp.path().join("beardog-fam.sock");
    std::fs::write(&sock, b"stale").expect("w");
    let discovery = PrimalDiscovery::new("fam");
    let mut q = DiscoveryQuery::capability(PrimalCapability::encryption());
    q.healthy_only = false;
    let v = discovery
        .discover_in(&q, tmp.path())
        .await
        .expect("discover");
    assert!(v.iter().any(|p| !p.is_healthy));
}

#[tokio::test]
async fn test_discovered_via_run_user_style_path() {
    let tmp = tempfile::tempdir().unwrap();
    let run = tmp.path().join("run/user/1000/biomeos");
    std::fs::create_dir_all(&run).expect("d");
    let sock = run.join("songbird-x.sock");
    let _l = std::os::unix::net::UnixListener::bind(&sock).expect("bind");
    let discovery = PrimalDiscovery::new("x");
    let p = discovery
        .discover_primal_in("songbird", &run)
        .await
        .expect("p");
    assert!(matches!(p.discovered_via, DiscoveryMethod::XdgRuntime));
}
