// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::{CallerContext, ConnectionOrigin};
use super::common::make_ionic_token;

#[test]
fn loopback_context_has_no_peer() {
    let ctx = CallerContext::loopback();
    assert!(ctx.peer.is_none());
    assert!(ctx.bearer_token.is_none());
    assert!(ctx.claims.is_none());
    assert_eq!(ctx.origin, ConnectionOrigin::Loopback);
}

#[test]
fn with_bearer_token_parses_ionic_claims() {
    let token = make_ionic_token(&serde_json::json!({
        "sub": "user1",
        "scope": ["graph.*"]
    }));
    let ctx = CallerContext::loopback().with_bearer_token(token);
    assert!(ctx.claims.is_some());
    assert_eq!(ctx.claims.as_ref().unwrap().sub, "user1");
}

#[test]
fn with_opaque_token_has_no_claims() {
    let ctx = CallerContext::loopback().with_bearer_token("opaque-tok".to_owned());
    assert!(ctx.bearer_token.is_some());
    assert!(ctx.claims.is_none());
}

#[test]
fn enforcement_mode_as_str() {
    use super::super::EnforcementMode;
    assert_eq!(EnforcementMode::Permissive.as_str(), "permissive");
    assert_eq!(EnforcementMode::Enforced.as_str(), "enforced");
}
