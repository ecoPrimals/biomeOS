// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "Tests use unwrap for clarity")]

use super::*;

fn make_ionic_token(payload: &serde_json::Value) -> String {
    use base64::Engine;
    let header = serde_json::json!({"alg":"EdDSA","typ":"ionic","ver":1});
    let h = base64::engine::general_purpose::STANDARD.encode(header.to_string().as_bytes());
    let p = base64::engine::general_purpose::STANDARD.encode(payload.to_string().as_bytes());
    let s = base64::engine::general_purpose::STANDARD.encode(b"fake-sig");
    format!("{h}.{p}.{s}")
}

// ── classify_method ──

#[test]
fn health_methods_are_public() {
    assert_eq!(classify_method("health.check"), MethodAccessLevel::Public);
    assert_eq!(
        classify_method("health.liveness"),
        MethodAccessLevel::Public
    );
}

#[test]
fn identity_is_public() {
    assert_eq!(classify_method("identity.get"), MethodAccessLevel::Public);
}

#[test]
fn capabilities_list_is_public() {
    assert_eq!(
        classify_method("capabilities.list"),
        MethodAccessLevel::Public
    );
    assert_eq!(
        classify_method("capability.list"),
        MethodAccessLevel::Public
    );
}

#[test]
fn auth_introspection_is_public() {
    assert_eq!(classify_method("auth.check"), MethodAccessLevel::Public);
    assert_eq!(classify_method("auth.mode"), MethodAccessLevel::Public);
    assert_eq!(classify_method("auth.peer_info"), MethodAccessLevel::Public);
}

#[test]
fn lifecycle_status_is_public() {
    assert_eq!(
        classify_method("lifecycle.status"),
        MethodAccessLevel::Public
    );
}

#[test]
fn graph_methods_are_local_trusted() {
    assert_eq!(
        classify_method("graph.execute"),
        MethodAccessLevel::LocalTrusted
    );
    assert_eq!(
        classify_method("graph.save"),
        MethodAccessLevel::LocalTrusted
    );
}

#[test]
fn composition_methods_are_local_trusted() {
    assert_eq!(
        classify_method("composition.deploy"),
        MethodAccessLevel::LocalTrusted
    );
    assert_eq!(
        classify_method("composition.status"),
        MethodAccessLevel::LocalTrusted
    );
}

#[test]
fn deploy_methods_are_local_trusted() {
    assert_eq!(
        classify_method("deploy.start"),
        MethodAccessLevel::LocalTrusted
    );
}

#[test]
fn non_orchestration_methods_are_protected() {
    assert_eq!(
        classify_method("capability.call"),
        MethodAccessLevel::Protected
    );
    assert_eq!(
        classify_method("neural_api.weight_health"),
        MethodAccessLevel::Protected
    );
}

#[test]
fn empty_method_is_protected() {
    assert_eq!(classify_method(""), MethodAccessLevel::Protected);
}

// ── scope_covers_method ──

#[test]
fn scope_wildcard_matches_all() {
    let scope = vec!["*".to_owned()];
    assert!(scope_covers_method(&scope, "anything.here"));
    assert!(scope_covers_method(&scope, "graph.execute"));
}

#[test]
fn scope_prefix_matches_domain() {
    let scope = vec!["compute.*".to_owned()];
    assert!(scope_covers_method(&scope, "compute.submit"));
    assert!(scope_covers_method(&scope, "compute.status"));
    assert!(!scope_covers_method(&scope, "storage.get"));
    assert!(!scope_covers_method(&scope, "compute_x.submit"));
}

#[test]
fn scope_exact_matches() {
    let scope = vec!["graph.execute".to_owned()];
    assert!(scope_covers_method(&scope, "graph.execute"));
    assert!(!scope_covers_method(&scope, "graph.save"));
}

#[test]
fn scope_empty_denies_all() {
    assert!(!scope_covers_method(&[], "anything"));
}

#[test]
fn scope_multiple_patterns() {
    let scope = vec!["compute.*".to_owned(), "storage.get".to_owned()];
    assert!(scope_covers_method(&scope, "compute.submit"));
    assert!(scope_covers_method(&scope, "storage.get"));
    assert!(!scope_covers_method(&scope, "storage.put"));
}

// ── IonicTokenClaims ──

#[test]
fn parse_ionic_token_extracts_claims() {
    let token = make_ionic_token(&serde_json::json!({
        "iss": "did:key:z6MkTest",
        "sub": "user1",
        "scope": ["compute.*", "storage.*"],
        "iat": 1000,
        "exp": 9999999999u64,
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

// ── CallerContext ──

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

// ── EnforcementMode ──

#[test]
fn enforcement_mode_as_str() {
    assert_eq!(EnforcementMode::Permissive.as_str(), "permissive");
    assert_eq!(EnforcementMode::Enforced.as_str(), "enforced");
}

// ── MethodGate::check with scope ──

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
        "exp": 9999999999u64
    }));
    let caller = CallerContext::loopback().with_bearer_token(token);
    assert!(gate.check("graph.execute", &caller).is_ok());
}

#[test]
fn token_with_wrong_scope_rejected_enforced() {
    let gate = MethodGate::new(EnforcementMode::Enforced);
    let token = make_ionic_token(&serde_json::json!({
        "scope": ["compute.*"],
        "exp": 9999999999u64
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
        "exp": 9999999999u64
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
        "exp": 9999999999u64,
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

// ── auth introspection ──

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
        "exp": 9999999999u64,
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

// ── JH-2 cpu/timeout_ms enforcement ──

#[test]
fn dispatch_timeout_ms_from_envelope() {
    let token = make_ionic_token(&serde_json::json!({
        "sub": "worker",
        "scope": ["*"],
        "exp": 9999999999u64,
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
        "exp": 9999999999u64,
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
        "exp": 9999999999u64
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
        "exp": 9999999999u64,
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
        "exp": 9999999999u64,
        "resources": {
            "cpu": 2.0,
            "mem": 2147483648u64,
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
        "exp": 9999999999u64,
        "resources": { "cpu": 2.0 }
    }));
    let claims = IonicTokenClaims::parse(&token).unwrap();
    assert!(!claims.resource_allowed(None, Some(4.0)));
    assert!(claims.resource_allowed(None, Some(1.5)));
    assert!(claims.resource_allowed(None, Some(2.0)));
}

// ── auth.check primalSpring contract alignment ──

#[test]
fn auth_check_returns_primalspring_contract_fields() {
    let gate = MethodGate::new(EnforcementMode::Enforced);
    let token = make_ionic_token(&serde_json::json!({
        "sub": "test-user",
        "scope": ["graph.*", "compute.*"],
        "exp": 9999999999u64,
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

// ── TokenVerifier trait ──

#[test]
fn local_claims_verifier_parses_ionic_token() {
    let verifier = LocalClaimsVerifier;
    let token = make_ionic_token(&serde_json::json!({
        "sub": "verifier-test",
        "scope": ["*"],
        "exp": 9999999999u64,
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
        "exp": 9999999999u64,
    }));
    assert!(verifier.verify(&token).is_none());
}

// ── BearDogVerifier (JH-11) ──

#[test]
fn beardog_verifier_sync_falls_back_to_local_parse() {
    let verifier = BearDogVerifier::new(std::path::PathBuf::from("/nonexistent/beardog.sock"));
    let token = make_ionic_token(&serde_json::json!({
        "sub": "federation-test",
        "scope": ["compute.*"],
        "exp": 9999999999u64,
    }));
    let claims = verifier.verify(&token).unwrap();
    assert_eq!(claims.sub, "federation-test");
}

#[test]
fn beardog_verifier_sync_returns_none_for_opaque() {
    let verifier = BearDogVerifier::new(std::path::PathBuf::from("/nonexistent/beardog.sock"));
    assert!(verifier.verify("opaque-token-xyz").is_none());
}

#[tokio::test]
async fn beardog_verifier_async_degrades_gracefully_when_unreachable() {
    let verifier = BearDogVerifier::new(std::path::PathBuf::from("/nonexistent/beardog.sock"));
    let token = make_ionic_token(&serde_json::json!({
        "sub": "async-fallback",
        "scope": ["*"],
        "exp": 9999999999u64,
    }));
    let claims = verifier.verify_async(&token).await;
    assert!(claims.is_some(), "should degrade to local parse");
    assert_eq!(claims.unwrap().sub, "async-fallback");
}

#[test]
fn beardog_verifier_from_env_does_not_panic() {
    let _ = BearDogVerifier::from_env();
}

#[test]
fn beardog_verifier_clone() {
    let v = BearDogVerifier::new(std::path::PathBuf::from("/tmp/bd.sock"));
    let v2 = v.clone();
    assert_eq!(v.socket_path(), v2.socket_path());
}
