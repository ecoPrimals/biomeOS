// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::make_ionic_token;
use super::super::{CallerContext, ConnectionOrigin, EnforcementMode, MethodGate};

#[test]
fn public_method_always_passes() {
    let gate = MethodGate::new(EnforcementMode::Enforced);
    let caller = CallerContext::loopback();
    assert!(gate.check("health.check", &caller).is_ok());
    assert!(gate.check("identity.get", &caller).is_ok());
    assert!(gate.check("auth.check", &caller).is_ok());
}

#[test]
fn protected_method_passes_in_permissive_mode() {
    let gate = MethodGate::new(EnforcementMode::Permissive);
    let caller = CallerContext::loopback();
    assert!(gate.check("graph.execute", &caller).is_ok());
}

#[test]
fn protected_method_rejected_in_enforced_mode_without_token() {
    let gate = MethodGate::new(EnforcementMode::Enforced);
    let caller = CallerContext::loopback();
    let err = gate.check("capability.call", &caller).unwrap_err();
    assert_eq!(err.code, -32_001);
}

#[test]
fn local_trusted_method_passes_in_enforced_mode_from_loopback() {
    let gate = MethodGate::new(EnforcementMode::Enforced);
    let caller = CallerContext::loopback();
    assert!(gate.check("composition.deploy", &caller).is_ok());
    assert!(gate.check("graph.execute", &caller).is_ok());
}

#[test]
fn local_trusted_method_passes_from_unix() {
    let gate = MethodGate::new(EnforcementMode::Enforced);
    let caller = CallerContext::unix();
    assert!(gate.check("composition.deploy", &caller).is_ok());
}

#[test]
fn local_trusted_method_rejected_from_remote_without_token() {
    let gate = MethodGate::new(EnforcementMode::Enforced);
    let caller = CallerContext {
        bearer_token: None,
        claims: None,
        peer: None,
        origin: ConnectionOrigin::Remote,
    };
    let err = gate.check("composition.deploy", &caller).unwrap_err();
    assert_eq!(err.code, -32_001);
}

#[test]
fn token_with_matching_scope_passes_enforced() {
    let gate = MethodGate::new(EnforcementMode::Enforced);
    let token = make_ionic_token(&serde_json::json!({
        "scope": ["graph.*"],
        "exp": 9_999_999_999_u64
    }));
    let caller = CallerContext::loopback().with_bearer_token(token);
    assert!(gate.check("graph.execute", &caller).is_ok());
}

#[test]
fn token_with_wrong_scope_rejected_enforced() {
    let gate = MethodGate::new(EnforcementMode::Enforced);
    let token = make_ionic_token(&serde_json::json!({
        "scope": ["compute.*"],
        "exp": 9_999_999_999_u64
    }));
    let caller = CallerContext::loopback().with_bearer_token(token);
    let err = gate.check("capability.call", &caller).unwrap_err();
    assert_eq!(err.code, -32_001);
}

#[test]
fn token_with_wrong_scope_allowed_permissive() {
    let gate = MethodGate::new(EnforcementMode::Permissive);
    let token = make_ionic_token(&serde_json::json!({
        "scope": ["compute.*"],
        "exp": 9_999_999_999_u64
    }));
    let caller = CallerContext::loopback().with_bearer_token(token);
    assert!(gate.check("graph.execute", &caller).is_ok());
}

#[test]
fn expired_token_rejected_enforced() {
    let gate = MethodGate::new(EnforcementMode::Enforced);
    let token = make_ionic_token(&serde_json::json!({
        "scope": ["*"],
        "exp": 1
    }));
    let caller = CallerContext::loopback().with_bearer_token(token);
    let err = gate.check("capability.call", &caller).unwrap_err();
    assert_eq!(err.code, -32_001);
}

#[test]
fn method_allowlist_enforced() {
    let gate = MethodGate::new(EnforcementMode::Enforced);
    let token = make_ionic_token(&serde_json::json!({
        "scope": ["*"],
        "exp": 9_999_999_999_u64,
        "resources": { "method_allowlist": ["capability.call"] }
    }));
    let caller = CallerContext::loopback().with_bearer_token(token);
    assert!(gate.check("capability.call", &caller).is_ok());
    let err = gate.check("capability.resolve", &caller).unwrap_err();
    assert_eq!(err.code, -32_001);
}

#[test]
fn opaque_token_passes_enforced() {
    let gate = MethodGate::new(EnforcementMode::Enforced);
    let caller = CallerContext::loopback().with_bearer_token("opaque-token".to_owned());
    assert!(gate.check("graph.execute", &caller).is_ok());
}

#[test]
fn gate_error_includes_method_in_data() {
    let gate = MethodGate::new(EnforcementMode::Enforced);
    let caller = CallerContext::loopback();
    let err = gate.check("capability.call", &caller).unwrap_err();
    let method_in_data = err
        .data
        .as_ref()
        .and_then(|d| d.get("method"))
        .and_then(serde_json::Value::as_str);
    assert_eq!(method_in_data, Some("capability.call"));
}
