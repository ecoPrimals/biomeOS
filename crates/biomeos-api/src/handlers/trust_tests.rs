// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::*;

// ========== TrustEvaluationRequest Tests ==========

#[test]
fn test_trust_evaluation_request_deserialize() {
    let json = r#"{
        "peer_id": "peer-123",
        "peer_tags": ["trusted", "verified"]
    }"#;
    let req: TrustEvaluationRequest = serde_json::from_str(json).expect("deserialize");
    assert_eq!(req.peer_id, "peer-123");
    assert_eq!(req.peer_tags.len(), 2);
    assert_eq!(req.peer_tags[0], "trusted");
    assert_eq!(req.peer_tags[1], "verified");
}

#[test]
fn test_trust_evaluation_request_serialize() {
    let req = TrustEvaluationRequest {
        peer_id: "peer-456".to_string(),
        peer_tags: vec!["family".to_string()],
    };
    let json = serde_json::to_string(&req).expect("serialize");
    assert!(json.contains("peer-456"));
    assert!(json.contains("family"));
}

#[test]
fn test_trust_evaluation_request_empty_tags() {
    let json = r#"{"peer_id": "lonely-peer", "peer_tags": []}"#;
    let req: TrustEvaluationRequest = serde_json::from_str(json).expect("deserialize");
    assert_eq!(req.peer_id, "lonely-peer");
    assert!(req.peer_tags.is_empty());
}

#[test]
fn test_trust_evaluation_request_roundtrip() {
    let req = TrustEvaluationRequest {
        peer_id: "test-peer".to_string(),
        peer_tags: vec!["tag1".to_string(), "tag2".to_string()],
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let back: TrustEvaluationRequest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.peer_id, req.peer_id);
    assert_eq!(back.peer_tags, req.peer_tags);
}

// ========== TrustEvaluationResponse Tests ==========

#[test]
fn test_trust_evaluation_response_serialize() {
    let resp = TrustEvaluationResponse {
        decision: "allow".to_string(),
        confidence: 0.95,
        reason: "known_peer".to_string(),
        trust_level: "high".to_string(),
        metadata: serde_json::json!({"provider": "security"}),
    };
    let json = serde_json::to_string(&resp).expect("serialize");
    assert!(json.contains("allow"));
    assert!(json.contains("0.95"));
    assert!(json.contains("high"));
}

#[test]
fn test_trust_evaluation_response_deserialize() {
    let json = r#"{
        "decision": "deny",
        "confidence": 0.1,
        "reason": "unknown_peer",
        "trust_level": "none",
        "metadata": {}
    }"#;
    let resp: TrustEvaluationResponse = serde_json::from_str(json).expect("deserialize");
    assert_eq!(resp.decision, "deny");
    assert!((resp.confidence - 0.1).abs() < f32::EPSILON);
    assert_eq!(resp.trust_level, "none");
}

#[test]
fn test_trust_evaluation_response_roundtrip() {
    let resp = TrustEvaluationResponse {
        decision: "evaluate".to_string(),
        confidence: 0.5,
        reason: "partial_match".to_string(),
        trust_level: "medium".to_string(),
        metadata: serde_json::json!({"score": 42}),
    };
    let json = serde_json::to_string(&resp).expect("serialize");
    let back: TrustEvaluationResponse = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.decision, "evaluate");
    assert!((back.confidence - 0.5).abs() < f32::EPSILON);
}

// ========== IdentityResponse Tests ==========

#[test]
fn test_identity_response_serialize() {
    let resp = IdentityResponse {
        encryption_tag: "security:family:1894e909e454:node1".to_string(),
        capabilities: vec!["btsp".to_string(), "birdsong".to_string()],
        family_id: "1894e909e454".to_string(),
        identity_attestations: Some(serde_json::json!({"role": "tower"})),
    };
    let json = serde_json::to_string(&resp).expect("serialize");
    assert!(json.contains("security:family:1894e909e454:node1"));
    assert!(json.contains("btsp"));
    assert!(json.contains("1894e909e454"));
}

#[test]
fn test_identity_response_without_attestations() {
    let resp = IdentityResponse {
        encryption_tag: "tag".to_string(),
        capabilities: vec![],
        family_id: "fam".to_string(),
        identity_attestations: None,
    };
    let json = serde_json::to_string(&resp).expect("serialize");
    let back: IdentityResponse = serde_json::from_str(&json).expect("deserialize");
    assert!(back.identity_attestations.is_none());
    assert!(back.capabilities.is_empty());
}

#[test]
fn test_identity_response_roundtrip() {
    let resp = IdentityResponse {
        encryption_tag: "security:test:tag".to_string(),
        capabilities: vec!["cap1".to_string(), "cap2".to_string(), "cap3".to_string()],
        family_id: "test-family".to_string(),
        identity_attestations: Some(serde_json::json!({"node": "tower", "level": 5})),
    };
    let json = serde_json::to_string(&resp).expect("serialize");
    let back: IdentityResponse = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.encryption_tag, resp.encryption_tag);
    assert_eq!(back.capabilities.len(), 3);
    assert_eq!(back.family_id, "test-family");
}

// ========== Debug Formatting ==========

#[test]
fn test_trust_evaluation_request_debug() {
    let req = TrustEvaluationRequest {
        peer_id: "debug-peer".to_string(),
        peer_tags: vec!["test".to_string()],
    };
    let debug = format!("{req:?}");
    assert!(debug.contains("debug-peer"));
}

#[test]
fn test_trust_evaluation_response_debug() {
    let resp = TrustEvaluationResponse {
        decision: "allow".to_string(),
        confidence: 1.0,
        reason: "test".to_string(),
        trust_level: "high".to_string(),
        metadata: serde_json::json!(null),
    };
    let debug = format!("{resp:?}");
    assert!(debug.contains("allow"));
    assert!(debug.contains("high"));
}

// ========== Error handling and edge cases ==========

#[test]
fn test_trust_evaluation_request_invalid_json_fails() {
    let json = r#"{"peer_id": "ok", "peer_tags": "not-an-array"}"#;
    let result: Result<TrustEvaluationRequest, _> = serde_json::from_str(json);
    assert!(
        result.is_err(),
        "invalid peer_tags type should fail deserialization"
    );
}

#[test]
fn test_trust_evaluation_request_missing_peer_id_fails() {
    let json = r#"{"peer_tags": []}"#;
    let result: Result<TrustEvaluationRequest, _> = serde_json::from_str(json);
    assert!(
        result.is_err(),
        "missing peer_id should fail deserialization"
    );
}

#[test]
fn test_trust_evaluation_request_empty_peer_id() {
    let json = r#"{"peer_id": "", "peer_tags": []}"#;
    let req: TrustEvaluationRequest =
        serde_json::from_str(json).expect("empty string is valid");
    assert_eq!(req.peer_id, "");
    assert!(req.peer_tags.is_empty());
}

#[test]
fn test_trust_evaluation_response_invalid_decision_type() {
    let json = r#"{"decision": 123, "confidence": 0.5, "reason": "x", "trust_level": "low", "metadata": {}}"#;
    let result: Result<TrustEvaluationResponse, _> = serde_json::from_str(json);
    assert!(result.is_err(), "decision must be string");
}

#[test]
fn test_trust_evaluation_response_confidence_bounds() {
    let resp = TrustEvaluationResponse {
        decision: "allow".to_string(),
        confidence: 0.0,
        reason: "min".to_string(),
        trust_level: "none".to_string(),
        metadata: serde_json::json!({}),
    };
    let json = serde_json::to_string(&resp).expect("serialize");
    let back: TrustEvaluationResponse = serde_json::from_str(&json).expect("deserialize");
    assert!((back.confidence - 0.0).abs() < f32::EPSILON);

    let resp2 = TrustEvaluationResponse {
        decision: "allow".to_string(),
        confidence: 1.0,
        reason: "max".to_string(),
        trust_level: "high".to_string(),
        metadata: serde_json::json!({}),
    };
    let json2 = serde_json::to_string(&resp2).expect("serialize");
    let back2: TrustEvaluationResponse = serde_json::from_str(&json2).expect("deserialize");
    assert!((back2.confidence - 1.0).abs() < f32::EPSILON);
}

#[test]
fn test_identity_response_empty_encryption_tag() {
    let resp = IdentityResponse {
        encryption_tag: String::new(),
        capabilities: vec![],
        family_id: "fam".to_string(),
        identity_attestations: None,
    };
    let json = serde_json::to_string(&resp).expect("serialize");
    let back: IdentityResponse = serde_json::from_str(&json).expect("deserialize");
    assert!(back.encryption_tag.is_empty());
    assert_eq!(back.family_id, "fam");
}

#[test]
fn test_identity_response_deserialize_with_capabilities_array() {
    let json = r#"{"encryption_tag": "tag", "capabilities": ["btsp"], "family_id": "fam"}"#;
    let resp: IdentityResponse = serde_json::from_str(json).expect("valid json");
    assert_eq!(resp.encryption_tag, "tag");
    assert_eq!(resp.capabilities.len(), 1);
    assert_eq!(resp.capabilities[0], "btsp");
}

#[test]
fn test_trust_evaluation_response_evaluate_decision() {
    let resp = TrustEvaluationResponse {
        decision: "evaluate".to_string(),
        confidence: 0.3,
        reason: "needs_more_data".to_string(),
        trust_level: "low".to_string(),
        metadata: serde_json::json!({"pending": true}),
    };
    let json = serde_json::to_string(&resp).expect("serialize");
    let back: TrustEvaluationResponse = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.decision, "evaluate");
    assert_eq!(back.trust_level, "low");
}

#[test]
fn test_identity_response_debug() {
    let resp = IdentityResponse {
        encryption_tag: "tag".to_string(),
        capabilities: vec![],
        family_id: "fam".to_string(),
        identity_attestations: None,
    };
    let debug = format!("{resp:?}");
    assert!(debug.contains("tag"));
    assert!(debug.contains("fam"));
}

#[test]
fn test_trust_evaluation_request_many_tags() {
    let req = TrustEvaluationRequest {
        peer_id: "p1".to_string(),
        peer_tags: vec!["a".to_string(), "b".to_string(), "c".to_string()],
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let back: TrustEvaluationRequest = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(back.peer_id, "p1");
    assert_eq!(back.peer_tags.len(), 3);
}

#[tokio::test]
async fn test_evaluate_trust_provider_unavailable_returns_internal_error() {
    use crate::AppState;
    use std::sync::Arc;

    let state = Arc::new(AppState::builder().build_with_defaults().expect("state"));
    let req = TrustEvaluationRequest {
        peer_id: "peer-x".to_string(),
        peer_tags: vec!["t".to_string()],
    };
    let result = evaluate_trust(axum::extract::State(state), axum::Json(req)).await;
    assert!(
        result.is_err(),
        "expected error when security provider unavailable"
    );
}

#[tokio::test]
async fn test_get_identity_provider_unavailable_returns_internal_error() {
    use crate::AppState;
    use std::sync::Arc;

    let state = Arc::new(AppState::builder().build_with_defaults().expect("state"));
    let result = get_identity(axum::extract::State(state)).await;
    assert!(result.is_err());
}
