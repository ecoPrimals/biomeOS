use super::super::*;

// ========== Request/Response Serialization Tests ==========

#[test]
fn test_rendezvous_post_request_deserialize() {
    let json = serde_json::json!({
        "encrypted_beacon": "base64data",
        "dark_forest_token": "tokendata"
    });
    let request: RendezvousPostRequest = serde_json::from_value(json).expect("deserialize");
    assert_eq!(request.encrypted_beacon, "base64data");
    assert_eq!(request.dark_forest_token, "tokendata");
}

#[test]
fn test_rendezvous_post_response_serialize() {
    let response = RendezvousPostResponse {
        accepted: true,
        slot_id: Some("slot-abc".to_string()),
        peer_beacon: None,
        peer_connection_info: None,
        peers_waiting: 2,
    };

    let json = serde_json::to_string(&response).expect("serialize");
    assert!(json.contains("\"accepted\":true"));
    assert!(json.contains("\"slot_id\":\"slot-abc\""));
    assert!(!json.contains("peer_beacon")); // skip_serializing_if = None
    assert!(!json.contains("peer_connection_info")); // skip_serializing_if = None
    assert!(json.contains("\"peers_waiting\":2"));
}

#[test]
fn test_rendezvous_post_response_with_peer() {
    let response = RendezvousPostResponse {
        accepted: true,
        slot_id: Some("slot-123".to_string()),
        peer_beacon: Some("encrypted_peer_data".to_string()),
        peer_connection_info: None,
        peers_waiting: 0,
    };

    let json = serde_json::to_string(&response).expect("serialize");
    assert!(json.contains("encrypted_peer_data"));
}

#[test]
fn test_rendezvous_post_response_with_connection_info() {
    use biomeos_core::connection_strategy::{PeerConnectionInfo, StunResults};

    let response = RendezvousPostResponse {
        accepted: true,
        slot_id: Some("slot-456".to_string()),
        peer_beacon: Some("peer_data".to_string()),
        peer_connection_info: Some(PeerConnectionInfo {
            stun_results: Some(StunResults {
                public_addr: "1.2.3.4:41200".to_string(),
                nat_type: "symmetric".to_string(),
            }),
            relay_endpoint: Some("192.0.2.10:3479".to_string()),
            stun_server: None,
        }),
        peers_waiting: 1,
    };

    let json = serde_json::to_string(&response).expect("serialize");
    assert!(json.contains("peer_connection_info"));
    assert!(json.contains("1.2.3.4:41200"));
    assert!(json.contains("symmetric"));
    assert!(json.contains("192.0.2.10:3479"));
}

#[test]
fn test_rendezvous_check_request_deserialize() {
    let json = serde_json::json!({
        "dark_forest_token": "check-token"
    });
    let request: RendezvousCheckRequest = serde_json::from_value(json).expect("deserialize");
    assert_eq!(request.dark_forest_token, "check-token");
}

#[test]
fn test_rendezvous_check_response_no_match() {
    let response = RendezvousCheckResponse {
        matched: false,
        peer_beacon: None,
        peer_connection_info: None,
        peers_waiting: 0,
    };

    let json = serde_json::to_string(&response).expect("serialize");
    assert!(json.contains("\"matched\":false"));
    assert!(!json.contains("peer_beacon")); // skip_serializing_if = None
    assert!(!json.contains("peer_connection_info")); // skip_serializing_if = None
}

#[test]
fn test_rendezvous_check_response_with_match() {
    let response = RendezvousCheckResponse {
        matched: true,
        peer_beacon: Some("matched_beacon_data".to_string()),
        peer_connection_info: None,
        peers_waiting: 3,
    };

    let json = serde_json::to_string(&response).expect("serialize");
    assert!(json.contains("\"matched\":true"));
    assert!(json.contains("matched_beacon_data"));
    assert!(json.contains("\"peers_waiting\":3"));
}

// ========== RendezvousSlot Tests ==========

#[test]
fn test_rendezvous_slot_clone() {
    let slot = RendezvousSlot {
        encrypted_beacon: "beacon".to_string(),
        node_hash: "hash".to_string(),
        lineage_hash: "lineage".to_string(),
        created_at: 1000,
        expires_at: 1300,
        connection_info: None,
    };

    assert_eq!(slot.encrypted_beacon, "beacon");
    assert_eq!(slot.node_hash, "hash");
    assert_eq!(slot.lineage_hash, "lineage");
    assert_eq!(slot.created_at, 1000);
    assert_eq!(slot.expires_at, 1300);
    assert!(slot.connection_info.is_none());
}

#[test]
fn test_rendezvous_slot_serialization() {
    let slot = RendezvousSlot {
        encrypted_beacon: "enc_data".to_string(),
        node_hash: "nh".to_string(),
        lineage_hash: "lh".to_string(),
        created_at: 100,
        expires_at: 400,
        connection_info: None,
    };

    let json = serde_json::to_string(&slot).expect("serialize");
    assert!(json.contains("enc_data"));
    assert!(json.contains("\"created_at\":100"));
    assert!(json.contains("\"expires_at\":400"));
    assert!(!json.contains("connection_info")); // skip_serializing_if = None
}
