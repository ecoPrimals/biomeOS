// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::{LocalClaimsVerifier, NoopVerifier, SecurityVerifier, TokenVerifier};
use super::common::make_ionic_token;

#[test]
fn local_claims_verifier_parses_ionic_token() {
    let verifier = LocalClaimsVerifier;
    let token = make_ionic_token(&serde_json::json!({
        "sub": "verifier-test",
        "scope": ["*"],
        "exp": 9_999_999_999_u64,
    }));
    let claims = verifier.verify(&token).unwrap();
    assert_eq!(claims.sub, "verifier-test");
}

#[test]
fn local_claims_verifier_returns_none_for_opaque() {
    let verifier = LocalClaimsVerifier;
    assert!(verifier.verify("opaque-token").is_none());
}

#[test]
fn noop_verifier_always_returns_none() {
    let verifier = NoopVerifier;
    let token = make_ionic_token(&serde_json::json!({
        "sub": "test",
        "scope": ["*"],
        "exp": 9_999_999_999_u64,
    }));
    assert!(verifier.verify(&token).is_none());
}

#[test]
fn security_verifier_sync_falls_back_to_local_parse() {
    let verifier = SecurityVerifier::new(std::path::PathBuf::from("/nonexistent/beardog.sock"));
    let token = make_ionic_token(&serde_json::json!({
        "sub": "federation-test",
        "scope": ["compute.*"],
        "exp": 9_999_999_999_u64,
    }));
    let claims = verifier.verify(&token).unwrap();
    assert_eq!(claims.sub, "federation-test");
}

#[test]
fn security_verifier_sync_returns_none_for_opaque() {
    let verifier = SecurityVerifier::new(std::path::PathBuf::from("/nonexistent/beardog.sock"));
    assert!(verifier.verify("opaque-token-xyz").is_none());
}

#[tokio::test]
async fn security_verifier_async_degrades_gracefully_when_unreachable() {
    let verifier = SecurityVerifier::new(std::path::PathBuf::from("/nonexistent/beardog.sock"));
    let token = make_ionic_token(&serde_json::json!({
        "sub": "async-fallback",
        "scope": ["*"],
        "exp": 9_999_999_999_u64,
    }));
    let claims = verifier.verify_async(&token).await;
    assert!(claims.is_some(), "should degrade to local parse");
    assert_eq!(claims.unwrap().sub, "async-fallback");
}

#[test]
fn security_verifier_from_env_does_not_panic() {
    let _ = SecurityVerifier::from_env();
}

#[test]
fn security_verifier_clone() {
    let v = SecurityVerifier::new(std::path::PathBuf::from("/tmp/bd.sock"));
    let v2 = v.clone();
    assert_eq!(v.socket_path(), v2.socket_path());
}
