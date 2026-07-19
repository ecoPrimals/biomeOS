// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::{CallerContext, ConnectionOrigin, EnforcementMode, MethodGate, PeerCredentials};
use super::common::make_ionic_token;

#[test]
fn auth_check_unauthenticated() {
    let gate = MethodGate::new(EnforcementMode::Permissive);
    let caller = CallerContext::loopback();
    let result = gate.handle_auth_check(&caller);
    assert_eq!(result["authenticated"], false);
    assert_eq!(result["mode"], "permissive");
}

#[test]
fn auth_check_with_ionic_token() {
    let gate = MethodGate::new(EnforcementMode::Enforced);
    let token = make_ionic_token(&serde_json::json!({
        "sub": "researcher",
        "scope": ["compute.*"],
        "exp": 9_999_999_999_u64,
        "resources": { "mem": 4096 }
    }));
    let caller = CallerContext::loopback().with_bearer_token(token);
    let result = gate.handle_auth_check(&caller);
    assert_eq!(result["authenticated"], true);
    assert_eq!(result["subject"], "researcher");
    assert_eq!(result["has_resource_envelope"], true);
}

#[test]
fn auth_mode_returns_current_mode() {
    let gate = MethodGate::new(EnforcementMode::Enforced);
    let result = gate.handle_auth_mode();
    assert_eq!(result["mode"], "enforced");
}

#[test]
fn auth_peer_info_loopback() {
    let gate = MethodGate::new(EnforcementMode::Permissive);
    let caller = CallerContext::loopback();
    let result = gate.handle_auth_peer_info(&caller);
    assert_eq!(result["origin"], "Loopback");
    assert_eq!(result["has_token"], false);
}

#[test]
fn auth_peer_info_with_credentials() {
    let gate = MethodGate::new(EnforcementMode::Permissive);
    let caller = CallerContext {
        bearer_token: Some("tok".to_owned()),
        claims: None,
        peer: Some(PeerCredentials {
            pid: Some(1234),
            uid: 1000,
        }),
        origin: ConnectionOrigin::Unix,
    };
    let result = gate.handle_auth_peer_info(&caller);
    assert_eq!(result["origin"], "Unix");
    assert_eq!(result["has_token"], true);
    assert_eq!(result["peer_uid"], 1000);
    assert_eq!(result["peer_pid"], 1234);
}
