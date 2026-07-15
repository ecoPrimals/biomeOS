// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

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
