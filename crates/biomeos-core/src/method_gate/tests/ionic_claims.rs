// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::make_ionic_token;
use super::super::IonicTokenClaims;

#[test]
fn parse_ionic_token_extracts_claims() {
    let token = make_ionic_token(&serde_json::json!({
        "iss": "did:key:z6MkTest",
        "sub": "user1",
        "scope": ["compute.*", "storage.*"],
        "iat": 1000,
        "exp": 9_999_999_999_u64,
        "jti": "tok-1"
    }));
    let claims = IonicTokenClaims::parse(&token).unwrap();
    assert_eq!(claims.iss, "did:key:z6MkTest");
    assert_eq!(claims.sub, "user1");
    assert_eq!(claims.scope.len(), 2);
    assert!(!claims.is_expired());
}

#[test]
fn parse_non_ionic_returns_none() {
    assert!(IonicTokenClaims::parse("opaque-token-string").is_none());
    assert!(IonicTokenClaims::parse("only.two").is_none());
}

#[test]
fn parse_with_resource_envelope() {
    let token = make_ionic_token(&serde_json::json!({
        "scope": ["*"],
        "resources": {
            "mem": 1_073_741_824u64,
            "cpu": 2.5,
            "method_allowlist": ["compute.submit", "compute.status"]
        }
    }));
    let claims = IonicTokenClaims::parse(&token).unwrap();
    let env = claims.resources.as_ref().unwrap();
    assert_eq!(env.mem, Some(1_073_741_824));
    assert_eq!(env.cpu, Some(2.5));
    assert_eq!(env.method_allowlist.as_ref().unwrap().len(), 2);
}

#[test]
fn expired_token_detected() {
    let token = make_ionic_token(&serde_json::json!({
        "scope": ["*"],
        "exp": 1
    }));
    let claims = IonicTokenClaims::parse(&token).unwrap();
    assert!(claims.is_expired());
}

#[test]
fn resource_allowed_checks_mem() {
    let token = make_ionic_token(&serde_json::json!({
        "scope": ["*"],
        "resources": { "mem": 1000 }
    }));
    let claims = IonicTokenClaims::parse(&token).unwrap();
    assert!(claims.resource_allowed(Some(500), None));
    assert!(claims.resource_allowed(Some(1000), None));
    assert!(!claims.resource_allowed(Some(1001), None));
}

#[test]
fn resource_allowed_checks_cpu() {
    let token = make_ionic_token(&serde_json::json!({
        "scope": ["*"],
        "resources": { "cpu": 4.0 }
    }));
    let claims = IonicTokenClaims::parse(&token).unwrap();
    assert!(claims.resource_allowed(None, Some(3.5)));
    assert!(!claims.resource_allowed(None, Some(4.5)));
}

#[test]
fn resource_allowed_no_envelope_allows_all() {
    let token = make_ionic_token(&serde_json::json!({ "scope": ["*"] }));
    let claims = IonicTokenClaims::parse(&token).unwrap();
    assert!(claims.resource_allowed(Some(u64::MAX), Some(f64::MAX)));
}

#[test]
fn method_allowlist_check() {
    let token = make_ionic_token(&serde_json::json!({
        "scope": ["*"],
        "resources": { "method_allowlist": ["compute.submit"] }
    }));
    let claims = IonicTokenClaims::parse(&token).unwrap();
    assert!(claims.method_in_allowlist("compute.submit"));
    assert!(!claims.method_in_allowlist("compute.status"));
}

#[test]
fn method_allowlist_absent_allows_all() {
    let token = make_ionic_token(&serde_json::json!({ "scope": ["*"] }));
    let claims = IonicTokenClaims::parse(&token).unwrap();
    assert!(claims.method_in_allowlist("anything"));
}
