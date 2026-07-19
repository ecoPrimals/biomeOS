// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::{
    CallerContext, EnforcementMode, IonicTokenClaims, MethodGate, ResourceEnvelope,
};
use super::common::make_ionic_token;

#[test]
fn dispatch_timeout_ms_from_envelope() {
    let token = make_ionic_token(&serde_json::json!({
        "sub": "worker",
        "scope": ["*"],
        "exp": 9_999_999_999_u64,
        "resources": { "timeout_ms": 5000 }
    }));
    let claims = IonicTokenClaims::parse(&token).unwrap();
    assert_eq!(claims.dispatch_timeout_ms(), Some(5000));
}

#[test]
fn dispatch_timeout_ms_none_when_absent() {
    let token = make_ionic_token(&serde_json::json!({
        "sub": "worker",
        "scope": ["*"],
        "exp": 9_999_999_999_u64,
        "resources": { "mem": 4096 }
    }));
    let claims = IonicTokenClaims::parse(&token).unwrap();
    assert_eq!(claims.dispatch_timeout_ms(), None);
}

#[test]
fn dispatch_timeout_ms_none_without_envelope() {
    let token = make_ionic_token(&serde_json::json!({
        "sub": "worker",
        "scope": ["*"],
        "exp": 9_999_999_999_u64
    }));
    let claims = IonicTokenClaims::parse(&token).unwrap();
    assert_eq!(claims.dispatch_timeout_ms(), None);
}

#[test]
fn resource_envelope_to_forwarding_value() {
    let env = ResourceEnvelope {
        mem: Some(1024),
        cpu: Some(2.0),
        timeout_ms: Some(10_000),
        method_allowlist: None,
    };
    let val = env.to_forwarding_value();
    assert_eq!(val["mem"], 1024);
    assert_eq!(val["cpu"], 2.0);
    assert_eq!(val["timeout_ms"], 10_000);
}

#[test]
fn resource_envelope_forwarding_value_null_fields() {
    let env = ResourceEnvelope::default();
    let val = env.to_forwarding_value();
    assert!(val["mem"].is_null());
    assert!(val["cpu"].is_null());
    assert!(val["timeout_ms"].is_null());
}

#[test]
fn auth_check_includes_resource_envelope_details() {
    let gate = MethodGate::new(EnforcementMode::Enforced);
    let token = make_ionic_token(&serde_json::json!({
        "sub": "researcher",
        "scope": ["compute.*"],
        "exp": 9_999_999_999_u64,
        "resources": {
            "mem": 4096,
            "cpu": 2.5,
            "timeout_ms": 30000
        }
    }));
    let caller = CallerContext::loopback().with_bearer_token(token);
    let result = gate.handle_auth_check(&caller);
    assert_eq!(result["has_resource_envelope"], true);
    let env = &result["resource_envelope"];
    assert_eq!(env["mem"], 4096);
    assert_eq!(env["cpu"], 2.5);
    assert_eq!(env["timeout_ms"], 30000);
}

#[test]
fn cpu_field_in_resource_envelope_parses() {
    let token = make_ionic_token(&serde_json::json!({
        "sub": "jupyter-user",
        "scope": ["compute.*"],
        "exp": 9_999_999_999_u64,
        "resources": {
            "cpu": 2.0,
            "mem": 2_147_483_648_u64,
            "timeout_ms": 60000
        }
    }));
    let claims = IonicTokenClaims::parse(&token).unwrap();
    let env = claims.resources.unwrap();
    assert_eq!(env.cpu, Some(2.0));
    assert_eq!(env.mem, Some(2_147_483_648));
    assert_eq!(env.timeout_ms, Some(60_000));
}

#[test]
fn resource_allowed_cpu_over_limit_rejected() {
    let token = make_ionic_token(&serde_json::json!({
        "sub": "user",
        "scope": ["*"],
        "exp": 9_999_999_999_u64,
        "resources": { "cpu": 2.0 }
    }));
    let claims = IonicTokenClaims::parse(&token).unwrap();
    assert!(!claims.resource_allowed(None, Some(4.0)));
    assert!(claims.resource_allowed(None, Some(1.5)));
    assert!(claims.resource_allowed(None, Some(2.0)));
}
