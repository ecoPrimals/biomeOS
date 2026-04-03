// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::*;

// ── NatType tests ──────────────────────────────────────────────────

#[test]
fn test_nat_type_from_detection() {
    assert_eq!(NatType::from_detection("symmetric"), NatType::Symmetric);
    assert_eq!(NatType::from_detection("Symmetric"), NatType::Symmetric);
    assert_eq!(NatType::from_detection("full_cone"), NatType::FullCone);
    assert_eq!(NatType::from_detection("full-cone"), NatType::FullCone);
    assert_eq!(NatType::from_detection("none"), NatType::None);
    assert_eq!(NatType::from_detection("public"), NatType::None);
    assert_eq!(
        NatType::from_detection("address_restricted"),
        NatType::AddressRestricted
    );
    assert_eq!(
        NatType::from_detection("port_restricted"),
        NatType::PortRestricted
    );
    assert_eq!(NatType::from_detection("garbage"), NatType::Unknown);
}

#[test]
fn test_nat_type_properties() {
    assert!(NatType::Symmetric.is_symmetric());
    assert!(!NatType::FullCone.is_symmetric());
    assert!(!NatType::None.is_symmetric());

    assert!(NatType::None.supports_direct_punch());
    assert!(NatType::FullCone.supports_direct_punch());
    assert!(NatType::PortRestricted.supports_direct_punch());
    assert!(!NatType::Symmetric.supports_direct_punch());
    assert!(!NatType::Unknown.supports_direct_punch());
}

// ── PortPattern tests ──────────────────────────────────────────────

#[test]
fn test_port_pattern_sequential_from_json() {
    let json = serde_json::json!({
        "type": "sequential",
        "step": 1,
        "last_port": 41204,
        "predicted_next": 41205,
        "confidence": 0.85
    });

    let pattern = PortPattern::from_json(&json);
    assert!(pattern.is_predictable());

    if let PortPattern::Sequential {
        step,
        last_port,
        predicted_next,
        confidence,
    } = pattern
    {
        assert_eq!(step, 1);
        assert_eq!(last_port, 41204);
        assert_eq!(predicted_next, 41205);
        assert!((confidence - 0.85).abs() < f64::EPSILON);
    } else {
        panic!("Expected Sequential pattern");
    }
}

#[test]
fn test_port_pattern_random_from_json() {
    let json = serde_json::json!({
        "type": "random",
        "observed": [41200, 52300, 10500, 33000]
    });

    let pattern = PortPattern::from_json(&json);
    assert!(!pattern.is_predictable());

    if let PortPattern::Random { observed } = pattern {
        assert_eq!(observed.len(), 4);
        assert_eq!(observed[0], 41200);
    } else {
        panic!("Expected Random pattern");
    }
}

#[test]
fn test_port_pattern_unknown_from_json() {
    let json = serde_json::json!({});
    let pattern = PortPattern::from_json(&json);
    assert!(!pattern.is_predictable());
    assert!(matches!(pattern, PortPattern::Unknown));
}

#[test]
fn test_port_pattern_low_confidence_not_predictable() {
    let json = serde_json::json!({
        "type": "sequential",
        "step": 3,
        "last_port": 50000,
        "predicted_next": 50003,
        "confidence": 0.3
    });

    let pattern = PortPattern::from_json(&json);
    assert!(!pattern.is_predictable());
}

// ── ConnectionTier tests ───────────────────────────────────────────

#[test]
fn test_connection_tier_display() {
    assert_eq!(ConnectionTier::LanDirect.to_string(), "LAN Direct");
    assert_eq!(ConnectionTier::DirectPunch.to_string(), "Direct Punch");
    assert_eq!(
        ConnectionTier::CoordinatedPunch.to_string(),
        "Coordinated Punch"
    );
    assert_eq!(ConnectionTier::PureRelay.to_string(), "Pure Relay");
}

#[test]
fn test_connection_result_serialization() {
    let result = ConnectionResult {
        tier: ConnectionTier::CoordinatedPunch,
        endpoint: "relay-session-abc123".to_string(),
        elapsed_ms: 450,
        tiers_attempted: vec![ConnectionTier::LanDirect, ConnectionTier::CoordinatedPunch],
    };

    let json = serde_json::to_string(&result).expect("serialize");
    assert!(json.contains("CoordinatedPunch"));
    assert!(json.contains("relay-session-abc123"));
    assert!(json.contains("450"));

    let deserialized: ConnectionResult = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.tier, ConnectionTier::CoordinatedPunch);
    assert_eq!(deserialized.tiers_attempted.len(), 2);
}

// ── PeerConnectionInfo tests ───────────────────────────────────────

#[test]
fn test_peer_connection_info_serialization() {
    let info = PeerConnectionInfo {
        stun_results: Some(StunResults {
            public_addr: "1.2.3.4:41200".to_string(),
            nat_type: "symmetric".to_string(),
        }),
        relay_endpoint: Some("192.0.2.1:3479".to_string()),
        stun_server: Some("192.0.2.1:3478".to_string()),
    };

    let json = serde_json::to_string(&info).expect("serialize");
    assert!(json.contains("1.2.3.4:41200"));
    assert!(json.contains("symmetric"));

    let deserialized: PeerConnectionInfo = serde_json::from_str(&json).expect("deserialize");
    assert!(deserialized.stun_results.is_some());
}

#[test]
fn test_peer_connection_info_minimal() {
    let info = PeerConnectionInfo {
        stun_results: None,
        relay_endpoint: None,
        stun_server: None,
    };

    let json = serde_json::to_string(&info).expect("serialize");
    assert_eq!(json, "{}");
}

// ── NatType serde tests ────────────────────────────────────────────

#[test]
fn test_nat_type_serialization_roundtrip() {
    for nat in &[
        NatType::None,
        NatType::FullCone,
        NatType::AddressRestricted,
        NatType::PortRestricted,
        NatType::Symmetric,
        NatType::Unknown,
    ] {
        let json = serde_json::to_string(nat).expect("serialize");
        let deserialized: NatType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized, *nat);
    }
}

#[test]
fn test_nat_type_from_detection_open() {
    assert_eq!(NatType::from_detection("open"), NatType::None);
}

#[test]
fn test_nat_type_from_detection_fullcone() {
    assert_eq!(NatType::from_detection("fullcone"), NatType::FullCone);
}

#[test]
fn test_port_pattern_from_json_sequential_defaults() {
    let json = serde_json::json!({"type": "sequential"});
    let pattern = PortPattern::from_json(&json);
    if let PortPattern::Sequential {
        step,
        last_port,
        predicted_next,
        confidence,
    } = pattern
    {
        assert_eq!(step, 1);
        assert_eq!(last_port, 0);
        assert_eq!(predicted_next, 0);
        assert!((confidence - 0.0).abs() < f64::EPSILON);
    } else {
        panic!("Expected Sequential with defaults");
    }
}

#[test]
fn test_port_pattern_from_json_random_empty() {
    let json = serde_json::json!({"type": "random"});
    let pattern = PortPattern::from_json(&json);
    assert!(matches!(pattern, PortPattern::Random { observed } if observed.is_empty()));
}

#[test]
fn test_stun_results_serialization() {
    let results = StunResults {
        public_addr: "1.2.3.4:41200".to_string(),
        nat_type: "symmetric".to_string(),
    };
    let json = serde_json::to_string(&results).expect("serialize");
    assert!(json.contains("1.2.3.4"));
    assert!(json.contains("symmetric"));

    let deserialized: StunResults = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(deserialized.public_addr, results.public_addr);
    assert_eq!(deserialized.nat_type, results.nat_type);
}

#[test]
fn test_connection_tier_copy() {
    let tier = ConnectionTier::LanDirect;
    let copied = tier;
    assert_eq!(tier, copied);
}

#[test]
fn test_nat_type_from_detection_mixed_case() {
    assert_eq!(NatType::from_detection("SYMMETRIC"), NatType::Symmetric);
    assert_eq!(NatType::from_detection("Full_Cone"), NatType::FullCone);
}

#[test]
fn nat_type_address_restricted_supports_punch() {
    assert!(NatType::AddressRestricted.supports_direct_punch());
    assert!(!NatType::Unknown.supports_direct_punch());
}

#[test]
fn port_pattern_random_non_numeric_observed_entries_skipped() {
    let json = serde_json::json!({
        "type": "random",
        "observed": ["not-a-number", 41200, null]
    });
    let pattern = PortPattern::from_json(&json);
    match pattern {
        PortPattern::Random { observed } => assert_eq!(observed, vec![41200u16]),
        _ => panic!("expected Random"),
    }
}

#[test]
fn test_port_pattern_sequential_exact_confidence_threshold() {
    let json = serde_json::json!({
        "type": "sequential",
        "step": 2,
        "last_port": 50000,
        "predicted_next": 50002,
        "confidence": 0.6
    });
    let pattern = PortPattern::from_json(&json);
    assert!(pattern.is_predictable());
}

#[test]
fn test_port_pattern_sequential_just_below_threshold() {
    let json = serde_json::json!({
        "type": "sequential",
        "step": 1,
        "last_port": 40000,
        "predicted_next": 40001,
        "confidence": 0.59
    });
    let pattern = PortPattern::from_json(&json);
    assert!(!pattern.is_predictable());
}

#[test]
fn test_port_pattern_unknown_type() {
    let json = serde_json::json!({"type": "custom_unknown"});
    let pattern = PortPattern::from_json(&json);
    assert!(matches!(pattern, PortPattern::Unknown));
    assert!(!pattern.is_predictable());
}

#[test]
fn test_connection_tier_partial_eq() {
    assert_eq!(ConnectionTier::LanDirect, ConnectionTier::LanDirect);
    assert_ne!(ConnectionTier::LanDirect, ConnectionTier::PureRelay);
}

#[test]
fn test_stun_results_deserialization() {
    let json = r#"{"public_addr":"10.0.0.1:41200","nat_type":"full_cone"}"#;
    let result: StunResults = serde_json::from_str(json).expect("deserialize");
    assert_eq!(result.public_addr, "10.0.0.1:41200");
    assert_eq!(result.nat_type, "full_cone");
}

#[test]
fn test_peer_connection_info_empty_stun() {
    let info = PeerConnectionInfo {
        stun_results: Some(StunResults {
            public_addr: String::new(),
            nat_type: "unknown".to_string(),
        }),
        relay_endpoint: None,
        stun_server: None,
    };
    let json = serde_json::to_string(&info).expect("serialize");
    assert!(json.contains("public_addr"));
}

/// connect_to_peer with non-existent neural-api socket — tests error path
#[tokio::test]
async fn test_connect_to_peer_socket_not_found() {
    let result = connect_to_peer("peer-123", "/nonexistent/path/neural-api-12345.sock", None).await;

    assert!(result.is_err());
    let err = result.unwrap_err().to_string();
    assert!(
        err.contains("Failed")
            || err.contains("connect")
            || err.contains("No such file")
            || err.contains("Connection refused"),
        "Expected connection error, got: {err}"
    );
}

/// connect_to_peer with peer_connection_info (uses stun_results for peer NAT)
#[tokio::test]
async fn test_connect_to_peer_with_connection_info() {
    let info = PeerConnectionInfo {
        stun_results: Some(StunResults {
            public_addr: "1.2.3.4:41200".to_string(),
            nat_type: "symmetric".to_string(),
        }),
        relay_endpoint: None,
        stun_server: None,
    };

    let result = connect_to_peer("peer-456", "/nonexistent/neural-api.sock", Some(&info)).await;

    assert!(result.is_err());
}

#[test]
fn test_connection_result_roundtrip() {
    let result = ConnectionResult {
        tier: ConnectionTier::LanDirect,
        endpoint: "/tmp/sock".to_string(),
        elapsed_ms: 10,
        tiers_attempted: vec![ConnectionTier::LanDirect],
    };
    let json = serde_json::to_string(&result).expect("serialize");
    let parsed: ConnectionResult = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.tier, result.tier);
    assert_eq!(parsed.endpoint, result.endpoint);
}

#[test]
fn test_connection_tier_serialization_roundtrip() {
    for tier in &[
        ConnectionTier::LanDirect,
        ConnectionTier::DirectPunch,
        ConnectionTier::CoordinatedPunch,
        ConnectionTier::PureRelay,
    ] {
        let json = serde_json::to_string(tier).expect("serialize");
        let parsed: ConnectionTier = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed, *tier);
    }
}

/// Unix-only: exercises `connect_to_peer` against a local JSON-RPC mock.
#[cfg(unix)]
mod connect_mock_unix {
    use super::*;
    use biomeos_types::JsonRpcResponse;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;

    fn spawn_sequential_neural_mock(
        socket_path: &std::path::Path,
        results: Vec<serde_json::Value>,
    ) {
        let listener = UnixListener::bind(socket_path).expect("bind neural mock");
        let results = Arc::new(results);
        let idx = Arc::new(AtomicUsize::new(0));
        tokio::spawn(async move {
            loop {
                let (stream, _) = listener.accept().await.expect("accept");
                let results = Arc::clone(&results);
                let idx = Arc::clone(&idx);
                tokio::spawn(async move {
                    let mut reader = BufReader::new(stream);
                    let mut line = String::new();
                    if reader.read_line(&mut line).await.is_err() {
                        return;
                    }
                    let Ok(req) = serde_json::from_str::<serde_json::Value>(line.trim()) else {
                        return;
                    };
                    let id = req.get("id").cloned().unwrap_or(serde_json::Value::Null);
                    let n = idx.fetch_add(1, Ordering::SeqCst);
                    let payload = results
                        .get(n)
                        .cloned()
                        .unwrap_or_else(|| serde_json::json!({}));
                    let response = JsonRpcResponse::success(id, payload);
                    let mut stream = reader.into_inner();
                    let body = serde_json::to_string(&response).expect("serialize");
                    let _ = stream.write_all(format!("{body}\n").as_bytes()).await;
                });
            }
        });
    }

    #[tokio::test]
    async fn connect_to_peer_tier1_lan_direct() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("neural.sock");
        let peer_id = "node-lan-1";
        spawn_sequential_neural_mock(
            &sock,
            vec![serde_json::json!({
                "peers": [{
                    "node_id": peer_id,
                    "endpoint": "unix:///tmp/mesh.sock"
                }]
            })],
        );

        let res = connect_to_peer(peer_id, sock.to_str().unwrap(), None)
            .await
            .expect("tier1");
        assert_eq!(res.tier, ConnectionTier::LanDirect);
        assert!(res.endpoint.contains("mesh") || res.endpoint.contains("unix"));
        assert!(res.tiers_attempted.contains(&ConnectionTier::LanDirect));
    }

    #[tokio::test]
    async fn connect_to_peer_tier2_direct_punch() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("neural.sock");
        let peer_id = "node-punch";
        spawn_sequential_neural_mock(
            &sock,
            vec![
                serde_json::json!({ "peers": [] }),
                serde_json::json!({ "nat_type": "full_cone" }),
                serde_json::json!({
                    "success": true,
                    "endpoint": "udp://punched:1234"
                }),
            ],
        );

        let info = PeerConnectionInfo {
            stun_results: Some(StunResults {
                public_addr: "1.1.1.1:1".to_string(),
                nat_type: "full_cone".to_string(),
            }),
            relay_endpoint: None,
            stun_server: None,
        };

        let res = connect_to_peer(peer_id, sock.to_str().unwrap(), Some(&info))
            .await
            .expect("tier2");
        assert_eq!(res.tier, ConnectionTier::DirectPunch);
        assert!(res.tiers_attempted.contains(&ConnectionTier::DirectPunch));
    }

    #[tokio::test]
    async fn connect_to_peer_tier4_pure_relay_after_failed_punch() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("neural.sock");
        let peer_id = "node-relay";
        spawn_sequential_neural_mock(
            &sock,
            vec![
                serde_json::json!({ "peers": [] }),
                serde_json::json!({ "nat_type": "full_cone" }),
                serde_json::json!({ "success": false }),
                serde_json::json!({ "session_id": "relay-final-session" }),
            ],
        );

        let info = PeerConnectionInfo {
            stun_results: Some(StunResults {
                public_addr: "1.1.1.1:1".to_string(),
                nat_type: "full_cone".to_string(),
            }),
            relay_endpoint: None,
            stun_server: None,
        };

        let res = connect_to_peer(peer_id, sock.to_str().unwrap(), Some(&info))
            .await
            .expect("relay fallback");
        assert_eq!(res.tier, ConnectionTier::PureRelay);
        assert_eq!(res.endpoint, "relay-final-session");
    }

    #[tokio::test]
    async fn connect_to_peer_tier3_symmetric_pure_relay() {
        let dir = tempfile::tempdir().expect("tempdir");
        let sock = dir.path().join("neural.sock");
        let peer_id = "node-sym";
        spawn_sequential_neural_mock(
            &sock,
            vec![
                serde_json::json!({ "peers": [] }),
                serde_json::json!({ "nat_type": "symmetric" }),
                serde_json::json!({ "session_id": "relay-sym" }),
                serde_json::json!({}),
            ],
        );

        let info = PeerConnectionInfo {
            stun_results: Some(StunResults {
                public_addr: "1.1.1.1:1".to_string(),
                nat_type: "symmetric".to_string(),
            }),
            relay_endpoint: None,
            stun_server: None,
        };

        let res = connect_to_peer(peer_id, sock.to_str().unwrap(), Some(&info))
            .await
            .expect("symmetric relay");
        assert_eq!(res.tier, ConnectionTier::PureRelay);
        assert_eq!(res.endpoint, "relay-sym");
    }
}
