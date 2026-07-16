#![expect(clippy::unwrap_used, reason = "test assertions")]
#![expect(clippy::expect_used, reason = "test assertions")]

use std::sync::Arc;

use super::super::*;
use super::spawn_neural_api_loopback_mock;

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
