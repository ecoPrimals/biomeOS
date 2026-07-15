// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::Plasmodium;
use super::super::PlasmodiumEnvOverrides;

#[tokio::test]
async fn test_query_collective_no_peers() {
    let p = Plasmodium::new();
    let result = p.query_collective().await;
    assert!(result.is_ok());
    let state = result.unwrap();
    assert!(!state.gates.is_empty(), "at least local gate");
    assert!(!state.family_id.is_empty());
    assert!(!state.snapshot_at.is_empty());
}

#[tokio::test]
async fn test_query_collective_merges_plasmodium_peers_env() {
    let p = Plasmodium::new_with_env_overrides(&PlasmodiumEnvOverrides {
        plasmodium_peers: Some("peer-a@127.0.0.1:59997,peer-b@host-only".to_string()),
        ..Default::default()
    });
    let result = p.query_collective().await;
    assert!(result.is_ok());
    let state = result.unwrap();
    assert!(
        state
            .gates
            .iter()
            .any(|g| g.gate_id == "peer-a" || g.gate_id == "peer-b"),
        "expected env-listed peers to appear in collective state: {:?}",
        state.gates.iter().map(|g| &g.gate_id).collect::<Vec<_>>()
    );
}

#[tokio::test]
async fn test_plasmodium_peers_bare_hostname_branch() {
    let p = Plasmodium::new_with_env_overrides(&PlasmodiumEnvOverrides {
        plasmodium_peers: Some("bare-hostname-only-token-unique-8821".to_string()),
        ..Default::default()
    });
    let state = p.query_collective().await.expect("collective");
    assert!(
        state
            .gates
            .iter()
            .any(|g| g.gate_id == "bare-hostname-only-token-unique-8821"),
        "bare entry should use same token for id and address"
    );
}

#[tokio::test]
async fn test_discover_peers_dedupes_duplicate_node_ids_in_env() {
    let p = Plasmodium::new_with_env_overrides(&PlasmodiumEnvOverrides {
        plasmodium_peers: Some("dup@127.0.0.1:1,dup@127.0.0.1:2".to_string()),
        ..Default::default()
    });
    let peers = p.discover_peers().await;
    assert_eq!(peers.iter().filter(|x| x.node_id == "dup").count(), 1);
    assert_eq!(
        peers.iter().find(|x| x.node_id == "dup").unwrap().address,
        "127.0.0.1:1"
    );
}

#[tokio::test]
async fn test_discover_peers_splitn_preserves_at_in_address() {
    let p = Plasmodium::new_with_env_overrides(&PlasmodiumEnvOverrides {
        plasmodium_peers: Some("node@ssh:user@remote.host".to_string()),
        ..Default::default()
    });
    let peers = p.discover_peers().await;
    let peer = peers.iter().find(|x| x.node_id == "node").expect("peer");
    assert_eq!(peer.address, "ssh:user@remote.host");
}

#[tokio::test]
async fn test_plasmodium_peers_skips_empty_segments() {
    let p = Plasmodium::new_with_env_overrides(&PlasmodiumEnvOverrides {
        plasmodium_peers: Some(" ,  dup@127.0.0.1:1 , dup@127.0.0.1:2 ".to_string()),
        ..Default::default()
    });
    let state = p.query_collective().await.expect("collective");
    assert!(
        state.gates.iter().filter(|g| g.gate_id == "dup").count() <= 1,
        "duplicate node ids from env should be deduped"
    );
}

#[tokio::test]
async fn test_query_collective_family_id_from_family_id_env() {
    let p = Plasmodium::new_with_env_overrides(&PlasmodiumEnvOverrides {
        family_id: Some("plasmo-env-family-42".to_string()),
        ..Default::default()
    });
    let state = p.query_collective().await.expect("collective");
    assert_eq!(state.family_id, "plasmo-env-family-42");
}

#[tokio::test]
async fn test_query_collective_node_family_id_fallback_env() {
    let p = Plasmodium::new_with_env_overrides(&PlasmodiumEnvOverrides {
        node_family_id: Some("node-fam-99".to_string()),
        ..Default::default()
    });
    let state = p.query_collective().await.expect("collective");
    assert_eq!(state.family_id, "node-fam-99");
}

#[tokio::test]
async fn test_query_collective_gate_id_from_gate_id_env() {
    let p = Plasmodium::new_with_env_overrides(&PlasmodiumEnvOverrides {
        gate_id: Some("gate-env-unique-771".to_string()),
        ..Default::default()
    });
    let state = p.query_collective().await.expect("collective");
    let local = state.gates.iter().find(|g| g.is_local).expect("local gate");
    assert_eq!(local.gate_id, "gate-env-unique-771");
}
