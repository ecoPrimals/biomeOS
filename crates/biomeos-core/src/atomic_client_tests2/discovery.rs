// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::TransportEndpoint;
use crate::atomic_client::{
    AtomicClient, DiscoverByCapabilityOpts, DiscoverOpts, discover_primal_endpoint,
    discover_primal_endpoint_with_opts,
};
use std::collections::HashMap;

#[tokio::test]
async fn test_discover_primal_endpoint_failure() {
    let result = discover_primal_endpoint("nonexistent_primal_xyz_123").await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not found") || err.contains("Primal"));
}

#[tokio::test]
async fn test_atomic_client_discover_failure() {
    let result = AtomicClient::discover("nonexistent_primal_xyz_456").await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("not found") || err.contains("Primal"));
}

#[tokio::test]
async fn test_atomic_client_discover_by_capability_failure() {
    let result = AtomicClient::discover_by_capability("nonexistent.capability.xyz.123").await;
    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(err.contains("No primal found") || err.contains("capability"));
}

#[tokio::test]
async fn test_discover_with_opts_includes_family_id_in_error() {
    let result = AtomicClient::discover_with_opts(
        "totally_missing_primal_zz",
        DiscoverOpts {
            family_id: Some("custom-family-xyz"),
            ..Default::default()
        },
    )
    .await;
    assert!(result.is_err());
    let msg = result.unwrap_err().to_string();
    assert!(
        msg.contains("custom-family-xyz"),
        "error should mention family id: {msg}"
    );
}

#[tokio::test]
async fn test_discover_primal_endpoint_with_opts_family_id() {
    let result = discover_primal_endpoint_with_opts(
        "missing_endpoint_primal_ab",
        DiscoverOpts {
            family_id: Some("fam-endpoint-test"),
            ..Default::default()
        },
    )
    .await;
    assert!(result.is_err());
    assert!(
        result
            .unwrap_err()
            .to_string()
            .contains("fam-endpoint-test")
    );
}

#[tokio::test]
async fn test_discover_by_capability_strict_skips_taxonomy() {
    let result = AtomicClient::discover_by_capability_with_opts(
        "nonexistent.strict.cap.123",
        DiscoverByCapabilityOpts {
            strict_discovery: Some(true),
            ..Default::default()
        },
    )
    .await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_discover_primal_uses_node_family_id_env() {
    let err = AtomicClient::discover_with_opts(
        "totally_missing_primal_xyz_999",
        DiscoverOpts {
            family_id: Some("from-node-env"),
            ..Default::default()
        },
    )
    .await;
    assert!(err.is_err());
    let msg = err.unwrap_err().to_string();
    assert!(
        msg.contains("from-node-env") || msg.contains("not found"),
        "{msg}"
    );
}

#[tokio::test]
async fn test_discover_by_capability_strict_taxonomy_path() {
    let err = AtomicClient::discover_by_capability("capability.that.does.not.exist.ever").await;
    assert!(err.is_err());
}

#[tokio::test]
async fn test_discover_by_capability_strict_env_disables_taxonomy_bootstrap() {
    let err = AtomicClient::discover_by_capability_with_opts(
        "nonexistent.strict.cap",
        DiscoverByCapabilityOpts {
            strict_discovery: Some(true),
            ..Default::default()
        },
    )
    .await;
    assert!(err.is_err());
}

#[tokio::test]
#[expect(clippy::unwrap_used, reason = "test asserts successful discovery")]
async fn test_atomic_client_discover_via_tcp_env_succeeds() {
    let mut m = HashMap::new();
    m.insert("DISCOVERUT_TCP".to_string(), "127.0.0.1:59996".to_string());
    let client = AtomicClient::discover_with_opts(
        "discoverut",
        DiscoverOpts {
            env_overrides: Some(&m),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert!(
        matches!(client.endpoint(), TransportEndpoint::TcpSocket { .. }),
        "expected TCP from env, got {:?}",
        client.endpoint()
    );
    assert!(!client.endpoint().is_native(), "TCP should be Tier 2");
}

#[tokio::test]
#[expect(clippy::unwrap_used, reason = "test asserts successful discovery")]
async fn test_discover_primal_endpoint_via_tcp_env_succeeds() {
    let mut m = HashMap::new();
    m.insert("DISCOVERPE_TCP".to_string(), "127.0.0.1:59995".to_string());
    let ep = discover_primal_endpoint_with_opts(
        "discoverpe",
        DiscoverOpts {
            env_overrides: Some(&m),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert!(matches!(ep, TransportEndpoint::TcpSocket { .. }));
}
