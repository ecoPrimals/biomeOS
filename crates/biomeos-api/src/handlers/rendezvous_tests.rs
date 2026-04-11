// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

use std::sync::Arc;

use super::*;

#[cfg(unix)]
async fn spawn_neural_api_loopback_mock(
    family_id: &str,
) -> (tempfile::TempDir, std::path::PathBuf) {
    use std::time::Duration;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    use tokio::net::UnixListener;

    let dir = tempfile::tempdir().expect("tempdir");
    let sock = dir.path().join("neural-rendezvous-mock.sock");
    let listener = UnixListener::bind(&sock).expect("bind mock neural");
    let fam = family_id.to_string();
    tokio::spawn(async move {
        loop {
            let Ok((mut stream, _)) = listener.accept().await else {
                continue;
            };
            let mut buf = vec![0u8; 64 * 1024];
            let n = match stream.read(&mut buf).await {
                Ok(n) if n > 0 => n,
                _ => continue,
            };
            let line = String::from_utf8_lossy(&buf[..n]);
            let Ok(v) = serde_json::from_str::<serde_json::Value>(line.trim_end()) else {
                continue;
            };
            let params = v.get("params").cloned().unwrap_or_default();
            let inner_method = params.get("method").and_then(|m| m.as_str()).unwrap_or("");
            let result = if inner_method == "birdsong.decrypt" {
                serde_json::json!({
                    "success": true,
                    "plaintext": "ok",
                    "family_id": fam.as_str(),
                })
            } else if inner_method == "crypto.blake3_hash" {
                serde_json::json!({ "hash": "node-hash-test" })
            } else {
                serde_json::json!({})
            };
            let body = serde_json::json!({
                "jsonrpc": "2.0",
                "id": v.get("id").clone(),
                "result": result,
            });
            let mut out = serde_json::to_string(&body).expect("serialize");
            out.push('\n');
            let _ = stream.write_all(out.as_bytes()).await;
            drop(stream);
        }
    });
    tokio::time::sleep(Duration::from_millis(40)).await;
    (dir, sock)
}

// ========== RendezvousState Tests ==========

#[test]
fn test_rendezvous_state_creation() {
    let state = RendezvousState::new();
    assert!(!state.family_id.is_empty());
}

#[test]
fn test_rendezvous_state_clone() {
    let state = RendezvousState::new();
    let cloned = state.clone();
    assert_eq!(cloned.family_id, state.family_id);
}

#[tokio::test]
async fn test_clean_expired_removes_old_slots() {
    let state = RendezvousState::new();

    // Add an expired slot
    let mut slots = state.slots.write().await;
    slots.insert(
        "lineage1".to_string(),
        vec![RendezvousSlot {
            encrypted_beacon: "test".to_string(),
            node_hash: "node1".to_string(),
            lineage_hash: "lineage1".to_string(),
            created_at: 0,
            expires_at: 1, // Expired long ago
            connection_info: None,
        }],
    );
    drop(slots);

    state.clean_expired().await;

    let slots = state.slots.read().await;
    assert!(slots.is_empty());
}

#[tokio::test]
async fn test_clean_expired_keeps_valid_slots() {
    let state = RendezvousState::new();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut slots = state.slots.write().await;
    slots.insert(
        "lineage1".to_string(),
        vec![RendezvousSlot {
            encrypted_beacon: "valid".to_string(),
            node_hash: "node1".to_string(),
            lineage_hash: "lineage1".to_string(),
            created_at: now,
            expires_at: now + 300, // 5 minutes from now
            connection_info: None,
        }],
    );
    drop(slots);

    state.clean_expired().await;

    let slots = state.slots.read().await;
    assert_eq!(slots.len(), 1);
    assert_eq!(slots["lineage1"][0].encrypted_beacon, "valid");
}

#[tokio::test]
async fn test_clean_expired_mixed_slots() {
    let state = RendezvousState::new();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut slots = state.slots.write().await;
    slots.insert(
        "lineage1".to_string(),
        vec![
            RendezvousSlot {
                encrypted_beacon: "expired".to_string(),
                node_hash: "node1".to_string(),
                lineage_hash: "lineage1".to_string(),
                created_at: 0,
                expires_at: 1, // Expired
                connection_info: None,
            },
            RendezvousSlot {
                encrypted_beacon: "valid".to_string(),
                node_hash: "node2".to_string(),
                lineage_hash: "lineage1".to_string(),
                created_at: now,
                expires_at: now + 300, // Valid
                connection_info: None,
            },
        ],
    );
    drop(slots);

    state.clean_expired().await;

    let slots = state.slots.read().await;
    assert_eq!(slots.len(), 1);
    assert_eq!(slots["lineage1"].len(), 1);
    assert_eq!(slots["lineage1"][0].encrypted_beacon, "valid");
}

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

#[tokio::test]
async fn test_multiple_lineage_groups() {
    let state = RendezvousState::new();

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();

    let mut slots = state.slots.write().await;

    slots.insert(
        "family-a".to_string(),
        vec![RendezvousSlot {
            encrypted_beacon: "beacon-a1".to_string(),
            node_hash: "node-a1".to_string(),
            lineage_hash: "family-a".to_string(),
            created_at: now,
            expires_at: now + 300,
            connection_info: None,
        }],
    );
    slots.insert(
        "family-b".to_string(),
        vec![
            RendezvousSlot {
                encrypted_beacon: "beacon-b1".to_string(),
                node_hash: "node-b1".to_string(),
                lineage_hash: "family-b".to_string(),
                created_at: now,
                expires_at: now + 300,
                connection_info: None,
            },
            RendezvousSlot {
                encrypted_beacon: "beacon-b2".to_string(),
                node_hash: "node-b2".to_string(),
                lineage_hash: "family-b".to_string(),
                created_at: now,
                expires_at: now + 300,
                connection_info: None,
            },
        ],
    );
    drop(slots);

    let slots = state.slots.read().await;
    assert_eq!(slots.len(), 2);
    assert_eq!(slots["family-a"].len(), 1);
    assert_eq!(slots["family-b"].len(), 2);
}

// ========== Request validation and edge cases ==========

#[test]
fn test_rendezvous_post_request_empty_beacon() {
    let json = serde_json::json!({
        "encrypted_beacon": "",
        "dark_forest_token": "some-token"
    });
    let request: RendezvousPostRequest = serde_json::from_value(json).expect("deserialize");
    assert!(request.encrypted_beacon.is_empty());
    assert_eq!(request.dark_forest_token, "some-token");
}

#[test]
fn test_rendezvous_post_request_connection_info_default() {
    let json = r#"{"encrypted_beacon": "x", "dark_forest_token": "t"}"#;
    let request: RendezvousPostRequest = serde_json::from_str(json).expect("deserialize");
    assert!(
        request.connection_info.is_none(),
        "connection_info should default to None"
    );
}

#[test]
fn test_rendezvous_post_request_with_connection_info() {
    let json = serde_json::json!({
        "encrypted_beacon": "beacon",
        "dark_forest_token": "token",
        "connection_info": {
            "stun_results": {"public_addr": "1.2.3.4:41200", "nat_type": "symmetric"},
            "relay_endpoint": "192.0.2.1:3479"
        }
    });
    let request: RendezvousPostRequest = serde_json::from_value(json).expect("deserialize");
    assert!(request.connection_info.is_some());
    let info = request.connection_info.as_ref().unwrap();
    assert!(info.stun_results.is_some());
    assert_eq!(
        info.stun_results.as_ref().unwrap().public_addr,
        "1.2.3.4:41200"
    );
}

#[test]
fn test_rendezvous_check_request_empty_token() {
    let json = serde_json::json!({"dark_forest_token": ""});
    let request: RendezvousCheckRequest = serde_json::from_value(json).expect("deserialize");
    assert!(request.dark_forest_token.is_empty());
}

// ========== Handler integration - invalid token returns 403 ==========

#[tokio::test]
async fn test_post_beacon_invalid_token_returns_forbidden() {
    let state = Arc::new(RendezvousState::new());
    let request = RendezvousPostRequest {
        encrypted_beacon: "fake-beacon-data".to_string(),
        dark_forest_token: "invalid-token-no-socket".to_string(),
        connection_info: None,
    };

    let response = post_beacon(axum::extract::State(state), axum::Json(request)).await;

    let (parts, _body) = response.into_response().into_parts();
    assert_eq!(
        parts.status,
        axum::http::StatusCode::FORBIDDEN,
        "invalid token should return 403 Forbidden (Dark Forest: no socket = not family)"
    );
}

#[tokio::test]
async fn test_check_peer_invalid_token_returns_forbidden() {
    let state = Arc::new(RendezvousState::new());
    let request = RendezvousCheckRequest {
        dark_forest_token: "invalid-token".to_string(),
    };

    let response = check_peer(axum::extract::State(state), axum::Json(request)).await;

    let (parts, _) = response.into_response().into_parts();
    assert_eq!(
        parts.status,
        axum::http::StatusCode::FORBIDDEN,
        "invalid token should return 403 Forbidden"
    );
}

#[test]
fn test_rendezvous_slot_expiry_logic() {
    let slot = RendezvousSlot {
        encrypted_beacon: "x".to_string(),
        node_hash: "n".to_string(),
        lineage_hash: "l".to_string(),
        created_at: 100,
        expires_at: 400,
        connection_info: None,
    };
    assert!(slot.expires_at > slot.created_at);
    assert_eq!(slot.expires_at - slot.created_at, 300);
}

#[cfg(unix)]
#[tokio::test]
async fn test_post_beacon_accepted_neural_mock() {
    use http_body_util::BodyExt;

    let (_dir, sock) = spawn_neural_api_loopback_mock("fam-rdz-1").await;

    let state = Arc::new(RendezvousState::new_for_test(
        "fam-rdz-1",
        Some(sock.to_str().expect("utf8 socket path")),
    ));
    let request = RendezvousPostRequest {
        encrypted_beacon: "beacon-a".to_string(),
        dark_forest_token: "token-a".to_string(),
        connection_info: None,
    };

    let response = post_beacon(axum::extract::State(state), axum::Json(request)).await;
    let (parts, body) = response.into_response().into_parts();
    assert_eq!(parts.status, axum::http::StatusCode::OK);
    let bytes = body.collect().await.expect("body").to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(v["accepted"], true);
    assert!(v.get("slot_id").is_some());
    assert_eq!(v["peers_waiting"], 0);
}

#[cfg(unix)]
#[tokio::test]
async fn test_check_peer_matched_neural_mock() {
    use http_body_util::BodyExt;

    let (_dir, sock) = spawn_neural_api_loopback_mock("fam-rdz-2").await;

    let state = Arc::new(RendezvousState::new_for_test(
        "fam-rdz-2",
        Some(sock.to_str().expect("utf8 socket path")),
    ));
    let post = post_beacon(
        axum::extract::State(state.clone()),
        axum::Json(RendezvousPostRequest {
            encrypted_beacon: "peer-beacon".to_string(),
            dark_forest_token: "tok-1".to_string(),
            connection_info: None,
        }),
    )
    .await;
    assert_eq!(post.into_response().status(), axum::http::StatusCode::OK);

    let check = check_peer(
        axum::extract::State(state),
        axum::Json(RendezvousCheckRequest {
            dark_forest_token: "tok-2".to_string(),
        }),
    )
    .await;
    let (parts, body) = check.into_response().into_parts();
    assert_eq!(parts.status, axum::http::StatusCode::OK);
    let bytes = body.collect().await.expect("body").to_bytes();
    let v: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
    assert_eq!(v["matched"], true);
    assert_eq!(v["peer_beacon"].as_str(), Some("peer-beacon"));
    assert_eq!(v["peers_waiting"], 1);
}
