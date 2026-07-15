// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::make_ionic_token;
use super::super::{CallerContext, EnforcementMode, MethodGate};

#[test]
fn auth_check_returns_primalspring_contract_fields() {
    let gate = MethodGate::new(EnforcementMode::Enforced);
    let token = make_ionic_token(&serde_json::json!({
        "sub": "test-user",
        "scope": ["graph.*", "compute.*"],
        "exp": 9_999_999_999_u64,
    }));
    let caller = CallerContext::loopback().with_bearer_token(token);
    let result = gate.handle_auth_check(&caller);
    assert_eq!(result["authenticated"], true);
    assert_eq!(result["verified"], true);
    assert_eq!(result["enforcement"], "enforced");
    assert_eq!(result["subject"], "test-user");
    assert!(result["scopes"].is_array());
    assert_eq!(result["scopes"].as_array().unwrap().len(), 2);
    assert!(result["expires_in"].as_u64().unwrap() > 0);
    assert_eq!(result["expired"], false);
}

#[test]
fn auth_check_unauthenticated_has_contract_fields() {
    let gate = MethodGate::new(EnforcementMode::Permissive);
    let caller = CallerContext::loopback();
    let result = gate.handle_auth_check(&caller);
    assert_eq!(result["authenticated"], false);
    assert_eq!(result["verified"], false);
    assert_eq!(result["enforcement"], "permissive");
}
