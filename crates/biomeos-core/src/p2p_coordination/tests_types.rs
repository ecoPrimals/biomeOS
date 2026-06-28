// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;
use std::time::SystemTime;

// ========================================================================
// Type serialization coverage for types.rs structs
// ========================================================================

#[test]
fn overall_health_json_roundtrip() {
    let health = OverallHealth {
        tunnel_id: "t1".to_string(),
        security_health: TunnelHealth {
            encryption_status: HealthStatus::Healthy,
            forward_secrecy: true,
            last_key_rotation: None,
            status: HealthStatus::Healthy,
        },
        transport_health: TransportHealth {
            connection_status: HealthStatus::Healthy,
            latency_ms: Some(10),
            packet_loss: Some(0.1),
            status: HealthStatus::Healthy,
        },
        status: HealthStatus::Healthy,
    };
    let json = serde_json::to_string(&health).unwrap();
    let back: OverallHealth = serde_json::from_str(&json).unwrap();
    assert_eq!(back.tunnel_id, "t1");
    assert_eq!(back.status, HealthStatus::Healthy);
}

#[test]
fn tunnel_info_json_roundtrip() {
    let info = TunnelInfo {
        tunnel_id: "t2".to_string(),
        status: TunnelStatus::Establishing,
        endpoints: vec![TransportEndpoint {
            node_id: "n1".to_string(),
            address: "10.0.0.1".to_string(),
            port: 9000,
            protocol: "tcp".to_string(),
            secure: true,
        }],
        established_at: SystemTime::now(),
    };
    let json = serde_json::to_string(&info).unwrap();
    let back: TunnelInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(back.tunnel_id, "t2");
    assert_eq!(back.status, TunnelStatus::Establishing);
    assert_eq!(back.endpoints.len(), 1);
}

#[test]
fn relay_info_json_roundtrip() {
    let info = RelayInfo {
        relay_node: "relay".to_string(),
        requester: "req".to_string(),
        target: "tgt".to_string(),
        status: RelayStatus::Active,
    };
    let json = serde_json::to_string(&info).unwrap();
    let back: RelayInfo = serde_json::from_str(&json).unwrap();
    assert_eq!(back.relay_node, "relay");
    assert_eq!(back.status, RelayStatus::Active);
}

#[test]
fn transport_endpoint_json_roundtrip() {
    let ep = TransportEndpoint {
        node_id: "node-1".to_string(),
        address: "192.168.1.1".to_string(),
        port: 8080,
        protocol: "quic".to_string(),
        secure: false,
    };
    let json = serde_json::to_string(&ep).unwrap();
    let back: TransportEndpoint = serde_json::from_str(&json).unwrap();
    assert_eq!(back.node_id, "node-1");
    assert_eq!(back.port, 8080);
    assert_eq!(back.protocol, "quic");
    assert!(!back.secure);
}

#[test]
fn encrypted_discovery_config_json_roundtrip() {
    let config = EncryptedDiscoveryConfig {
        encryption_key: bytes::Bytes::from_static(b"key"),
        lineage_filter: LineageProof {
            lineage_id: "test".to_string(),
            depth: 0,
            proof: bytes::Bytes::new(),
            timestamp: SystemTime::now(),
        },
        mode: DiscoveryMode::Encrypted,
    };
    let json = serde_json::to_string(&config).unwrap();
    let back: EncryptedDiscoveryConfig = serde_json::from_str(&json).unwrap();
    assert_eq!(back.mode, DiscoveryMode::Encrypted);
}

#[test]
fn broadcast_keys_json_roundtrip() {
    let keys = BroadcastKeys {
        broadcast_key: bytes::Bytes::from_static(b"bkey"),
        lineage_proof: LineageProof {
            lineage_id: "fam".to_string(),
            depth: 2,
            proof: bytes::Bytes::new(),
            timestamp: SystemTime::now(),
        },
        generated_at: SystemTime::now(),
    };
    let json = serde_json::to_string(&keys).unwrap();
    let back: BroadcastKeys = serde_json::from_str(&json).unwrap();
    assert_eq!(back.lineage_proof.lineage_id, "fam");
    assert_eq!(back.lineage_proof.depth, 2);
}

#[test]
fn lineage_info_json_roundtrip() {
    let info = LineageInfo {
        is_ancestor: true,
        depth: 5,
        proof: LineageProof {
            lineage_id: "l".to_string(),
            depth: 0,
            proof: bytes::Bytes::new(),
            timestamp: SystemTime::now(),
        },
    };
    let json = serde_json::to_string(&info).unwrap();
    let back: LineageInfo = serde_json::from_str(&json).unwrap();
    assert!(back.is_ancestor);
    assert_eq!(back.depth, 5);
}

#[test]
fn relay_offer_json_roundtrip() {
    let offer = RelayOffer {
        relay_node: "r".to_string(),
        relay_endpoint: TransportEndpoint {
            node_id: "r".to_string(),
            address: "10.0.0.1".to_string(),
            port: 9000,
            protocol: "tcp".to_string(),
            secure: true,
        },
        expires_at: SystemTime::now() + std::time::Duration::from_secs(60),
        lineage_verified: true,
    };
    let json = serde_json::to_string(&offer).unwrap();
    let back: RelayOffer = serde_json::from_str(&json).unwrap();
    assert_eq!(back.relay_node, "r");
    assert!(back.lineage_verified);
}

#[test]
fn relay_connection_json_roundtrip() {
    let conn = RelayConnection {
        connection_id: "c1".to_string(),
        relay_node: "r".to_string(),
        established_at: SystemTime::now(),
        status: RelayStatus::Establishing,
    };
    let json = serde_json::to_string(&conn).unwrap();
    let back: RelayConnection = serde_json::from_str(&json).unwrap();
    assert_eq!(back.connection_id, "c1");
    assert_eq!(back.status, RelayStatus::Establishing);
}

#[test]
fn tunnel_request_json_roundtrip() {
    let req = TunnelRequest {
        id: "tr1".to_string(),
        endpoint_a: TransportEndpoint {
            node_id: "a".to_string(),
            address: "1.2.3.4".to_string(),
            port: 100,
            protocol: "udp".to_string(),
            secure: false,
        },
        endpoint_b: TransportEndpoint {
            node_id: "b".to_string(),
            address: "5.6.7.8".to_string(),
            port: 200,
            protocol: "tcp".to_string(),
            secure: true,
        },
        encryption_key: bytes::Bytes::from_static(b"enc"),
        created_at: SystemTime::now(),
    };
    let json = serde_json::to_string(&req).unwrap();
    let back: TunnelRequest = serde_json::from_str(&json).unwrap();
    assert_eq!(back.id, "tr1");
    assert_eq!(back.endpoint_a.port, 100);
    assert_eq!(back.endpoint_b.protocol, "tcp");
}

#[test]
fn broadcast_test_json_roundtrip() {
    let bt = BroadcastTest {
        encrypted: true,
        timestamp: SystemTime::now(),
        success: false,
    };
    let json = serde_json::to_string(&bt).unwrap();
    let back: BroadcastTest = serde_json::from_str(&json).unwrap();
    assert!(back.encrypted);
    assert!(!back.success);
}

#[test]
fn lineage_proof_json_roundtrip() {
    let proof = LineageProof {
        lineage_id: "fam-1".to_string(),
        depth: 3,
        proof: bytes::Bytes::from_static(b"proof-data"),
        timestamp: SystemTime::now(),
    };
    let json = serde_json::to_string(&proof).unwrap();
    let back: LineageProof = serde_json::from_str(&json).unwrap();
    assert_eq!(back.lineage_id, "fam-1");
    assert_eq!(back.depth, 3);
}
