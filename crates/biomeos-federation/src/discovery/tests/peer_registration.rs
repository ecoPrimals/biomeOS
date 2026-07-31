// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Discovery tests - extracted to keep discovery/mod.rs under 1000 lines

#![expect(clippy::expect_used, reason = "test assertions")]

use super::super::*;

#[test]
fn test_register_songbird_peer_full() {
    let mut pd = PrimalDiscovery::new();
    let peer = serde_json::json!({
        "node_id": "beardog:fam1:direct:abc123",
        "family_id": "fam1",
        "capabilities": ["security", "crypto"],
        "endpoints": {
            "unix_socket": "/tmp/beardog.sock",
            "udp": "192.0.2.10:9000"
        }
    });
    pd.register_discovery_peer(&peer);

    assert_eq!(pd.discovered_primals.len(), 1);
    let dp = pd.get("beardog").expect("should exist");
    assert_eq!(dp.primal_type, "remote");
    assert_eq!(dp.endpoints.len(), 2);
    assert_eq!(dp.metadata["family_id"], "fam1");
    assert_eq!(dp.metadata["discovered_via"], "discovery_udp");
}

#[test]
fn test_register_songbird_peer_minimal() {
    let mut pd = PrimalDiscovery::new();
    let peer = serde_json::json!({
        "node_id": "songbird"
    });
    pd.register_discovery_peer(&peer);

    let dp = pd.get("songbird").expect("should exist");
    assert_eq!(dp.name, "songbird");
    assert!(dp.endpoints.is_empty());
    assert_eq!(dp.metadata["family_id"], "");
}

#[test]
fn test_register_songbird_peer_no_node_id() {
    let mut pd = PrimalDiscovery::new();
    let peer = serde_json::json!({"family_id": "x"});
    pd.register_discovery_peer(&peer);
    assert!(pd.discovered_primals.is_empty());
}

#[test]
fn test_register_songbird_peer_with_unix_only() {
    let mut pd = PrimalDiscovery::new();
    let peer = serde_json::json!({
        "node_id": "nestgate:fam:direct:hash",
        "endpoints": {
            "unix_socket": "/run/membrane/nestgate.sock"
        }
    });
    pd.register_discovery_peer(&peer);

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
    pd.register_discovery_peer(&peer);

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
    pd.register_discovery_peer(&peer);

    let dp = pd.get("svc").expect("should exist");
    assert!(
        dp.endpoints.is_empty(),
        "invalid UDP addr should be skipped"
    );
}
