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

use biomeos_core::TransportEndpoint;
use std::path::PathBuf;
use std::sync::Arc;

use super::super::{AtomicType, NeuralRouter};

fn unix_endpoint(path: &str) -> TransportEndpoint {
    TransportEndpoint::UnixSocket {
        path: PathBuf::from(path),
    }
}

#[tokio::test]
async fn test_discover_capability_unregistered() {
    let router = NeuralRouter::new("test-family");
    let result = router
        .discover_capability("nonexistent_capability_xyz")
        .await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("not registered") || err.contains("Capability") || err.contains("not found"),
        "expected capability error, got: {err}"
    );
}

#[tokio::test]
async fn test_discover_capability_registered() {
    let router = NeuralRouter::new("test-family");
    router
        .register_capability(
            "security",
            "beardog",
            unix_endpoint("/tmp/beardog-test.sock"),
            "test",
        )
        .await
        .expect("register");

    let result = router.discover_capability("security").await;
    assert!(result.is_ok());
    let atomic = result.unwrap();
    assert_eq!(atomic.capability.as_ref(), "security");
    assert_eq!(atomic.primals.len(), 1);
    assert_eq!(atomic.primals[0].name.as_ref(), "beardog");
}

#[tokio::test]
async fn test_find_primal_by_socket_nonexistent() {
    let router = NeuralRouter::new("test-family-xyz");
    let result = router
        .find_primal_by_socket_with_runtime_dir(
            "beardog",
            Some(std::path::Path::new("/nonexistent/path/for/tests")),
        )
        .await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("not found") || err.contains("does not exist"),
        "expected socket not found, got: {err}"
    );
}

#[tokio::test]
async fn test_discover_by_category_empty_registry_security() {
    let router = NeuralRouter::new("empty-reg");
    // Prevent lazy socket rescan from finding real primals running on this host.
    router
        .lazy_rescan_attempted
        .store(true, std::sync::atomic::Ordering::Relaxed);
    let err = router.discover_capability("security").await.unwrap_err();
    assert!(
        err.to_string().contains("No primals") || err.to_string().contains("not registered"),
        "got: {err}"
    );
}

#[tokio::test]
async fn test_discover_capability_unknown_category_string() {
    let router = NeuralRouter::new("x");
    let err = router
        .discover_capability("totally_unknown_capability_xyz")
        .await
        .unwrap_err();
    assert!(
        err.to_string().contains("not registered") || err.to_string().contains("does not map"),
        "got: {err}"
    );
}

#[tokio::test]
async fn test_discover_capability_http_alias_requires_registry() {
    let router = NeuralRouter::new("http-test");
    let err = router.discover_capability("http.get").await.unwrap_err();
    assert!(!err.to_string().is_empty());
}

#[tokio::test]
async fn test_discover_capability_ai_category_empty_registry() {
    let router = NeuralRouter::new("ai-test");
    let err = router
        .discover_capability("ai.text_generation")
        .await
        .unwrap_err();
    assert!(err.to_string().contains("No primals") || err.to_string().contains("not registered"));
}

#[tokio::test]
async fn test_discover_registered_sets_primary_endpoint() {
    let router = NeuralRouter::new("ps");
    let ep = unix_endpoint("/tmp/neural-discovery-unit.sock");
    router
        .register_capability("storage", "nest", ep.clone(), "t")
        .await
        .unwrap();
    let atomic = router.discover_capability("storage").await.unwrap();
    assert_eq!(atomic.primary_endpoint, ep);
    assert_eq!(atomic.primals.len(), 1);
}

#[tokio::test]
async fn test_discover_tower_atomic_via_secure_http_alias() {
    let router = NeuralRouter::new("tower-fam");
    router
        .register_capability(
            "security",
            "beardog",
            unix_endpoint("/tmp/tower-security.sock"),
            "t",
        )
        .await
        .unwrap();
    router
        .register_capability(
            "discovery",
            "songbird",
            unix_endpoint("/tmp/tower-discovery.sock"),
            "t",
        )
        .await
        .unwrap();
    let atomic = router.discover_capability("http.get").await.expect("tower");
    assert_eq!(atomic.capability.as_ref(), "secure_http");
    assert_eq!(atomic.primals.len(), 2);
    assert!(matches!(atomic.atomic_type, Some(AtomicType::Tower)));
}

#[tokio::test]
async fn test_discover_nest_atomic_requires_storage() {
    let router = NeuralRouter::new("nest-fam");
    router
        .register_capability("security", "bd", unix_endpoint("/tmp/nest-bd.sock"), "t")
        .await
        .unwrap();
    router
        .register_capability("discovery", "sb", unix_endpoint("/tmp/nest-sb.sock"), "t")
        .await
        .unwrap();
    let err = router
        .discover_capability("secure_storage")
        .await
        .unwrap_err();
    assert!(
        err.to_string().contains("storage") || err.to_string().contains("not found"),
        "{}",
        err
    );
}

#[tokio::test]
async fn test_discover_capability_category_discovery_unknown_maps_error() {
    let router = NeuralRouter::new("cat-reg");
    router
        .register_capability(
            "discovery.meta",
            "songbird",
            unix_endpoint("/tmp/discovery-meta.sock"),
            "t",
        )
        .await
        .unwrap();
    let atomic = router
        .discover_capability("discovery")
        .await
        .expect("discovery");
    assert!(
        atomic.primals.iter().any(|p| p.name.as_ref() == "songbird"),
        "{atomic:?}"
    );
}

#[tokio::test]
async fn test_discover_capability_http_post_alias() {
    let router = NeuralRouter::new("http-post");
    let err = router.discover_capability("http.post").await.unwrap_err();
    assert!(!err.to_string().is_empty());
}

#[tokio::test]
async fn test_register_capability_tcp_endpoint() {
    let router = NeuralRouter::new("tcp-test");
    let ep = TransportEndpoint::TcpSocket {
        host: Arc::from("192.0.2.100"),
        port: 9001,
    };
    router
        .register_capability("crypto.sign", "beardog", ep.clone(), "cross-gate")
        .await
        .unwrap();
    let providers = router
        .get_capability_providers("crypto.sign")
        .await
        .unwrap();
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].endpoint, ep);
    assert_eq!(providers[0].primal_name.as_ref(), "beardog");
}

#[tokio::test]
async fn test_register_capability_abstract_socket() {
    let router = NeuralRouter::new("abstract-test");
    let ep = TransportEndpoint::AbstractSocket {
        name: Arc::from("biomeos_squirrel_abc123"),
    };
    router
        .register_capability("storage.put", "squirrel", ep.clone(), "primal_announcement")
        .await
        .unwrap();
    let providers = router
        .get_capability_providers("storage.put")
        .await
        .unwrap();
    assert_eq!(providers[0].endpoint, ep);
}

#[tokio::test]
async fn test_register_capability_http_endpoint() {
    let router = NeuralRouter::new("http-ep-test");
    let ep = TransportEndpoint::HttpJsonRpc {
        host: Arc::from("songbird.local"),
        port: 8080,
    };
    router
        .register_capability("discovery.mesh", "songbird", ep.clone(), "beacon")
        .await
        .unwrap();
    let providers = router
        .get_capability_providers("discovery.mesh")
        .await
        .unwrap();
    assert_eq!(providers[0].endpoint, ep);
}

#[tokio::test]
async fn test_prefix_lookup_finds_dag_domain() {
    let router = NeuralRouter::new("prefix-dag");
    let ep = unix_endpoint("/tmp/rhizocrypt-prefix.sock");
    router
        .register_capability("dag.session.create", "rhizocrypt", ep.clone(), "graph")
        .await
        .unwrap();
    router
        .register_capability("dag.event.append", "rhizocrypt", ep.clone(), "graph")
        .await
        .unwrap();

    let result = router.try_prefix_lookup("dag").await;
    assert!(result.is_some(), "prefix lookup should find dag.* methods");
    let atomic = result.unwrap();
    assert_eq!(atomic.primals.len(), 1, "deduplicate by primal name");
    assert_eq!(atomic.primals[0].name.as_ref(), "rhizocrypt");
}

#[tokio::test]
async fn test_prefix_lookup_misses_unrelated() {
    let router = NeuralRouter::new("prefix-miss");
    router
        .register_capability(
            "dag.session.create",
            "rhizocrypt",
            unix_endpoint("/tmp/rc.sock"),
            "graph",
        )
        .await
        .unwrap();

    let result = router.try_prefix_lookup("spine").await;
    assert!(result.is_none(), "spine.* should not match dag.*");
}

#[tokio::test]
async fn test_discover_capability_via_prefix() {
    let router = NeuralRouter::new("discover-prefix");
    let ep = unix_endpoint("/tmp/loamspine-prefix.sock");
    router
        .register_capability("session.commit", "loamspine", ep.clone(), "graph")
        .await
        .unwrap();
    router
        .register_capability("spine.create", "loamspine", ep.clone(), "graph")
        .await
        .unwrap();

    let atomic = router
        .discover_capability("session")
        .await
        .expect("should find loamspine via session.* prefix");
    assert_eq!(atomic.primals[0].name.as_ref(), "loamspine");
}

#[tokio::test]
async fn test_prefix_lookup_deduplicates_providers() {
    let router = NeuralRouter::new("prefix-dedup");
    let ep = unix_endpoint("/tmp/sweetgrass-dedup.sock");
    router
        .register_capability("braid.create", "sweetgrass", ep.clone(), "graph")
        .await
        .unwrap();
    router
        .register_capability("braid.commit", "sweetgrass", ep.clone(), "graph")
        .await
        .unwrap();
    router
        .register_capability("braid.get", "sweetgrass", ep.clone(), "graph")
        .await
        .unwrap();

    let result = router.try_prefix_lookup("braid").await;
    assert!(result.is_some());
    let atomic = result.unwrap();
    assert_eq!(
        atomic.primals.len(),
        1,
        "same primal registered 3x should appear once"
    );
}
