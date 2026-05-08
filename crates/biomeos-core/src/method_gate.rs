// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Pre-dispatch capability gate for JSON-RPC methods (JH-0).
//!
//! Implements the ecosystem-standard `MethodGate` pattern defined in
//! `primalSpring/wateringHole/METHOD_GATE_STANDARD.md`. Every incoming
//! RPC call passes through [`MethodGate::check`] *before* the dispatch
//! table, classifying methods as [`MethodAccessLevel::Public`] or
//! [`MethodAccessLevel::Protected`].
//!
//! Two enforcement modes control behavior:
//! - **Permissive** (default): protected methods without a token are
//!   logged but allowed, preserving backward compatibility.
//! - **Enforced**: protected methods without a valid token are rejected
//!   with `PERMISSION_DENIED` (-32001).
//!
//! The gate reads its mode from the `BIOMEOS_AUTH_MODE` environment
//! variable (or falls back to `Permissive`).

use biomeos_types::JsonRpcError;

/// Access level for a JSON-RPC method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodAccessLevel {
    /// Health probes, identity, capability advertisement — always allowed.
    Public,
    /// Requires a valid capability token when enforcement is active.
    Protected,
}

/// Methods that are always public (prefix match).
const PUBLIC_METHOD_PREFIXES: &[&str] = &["health."];

/// Methods that are always public (exact match).
const PUBLIC_METHODS: &[&str] = &[
    "identity.get",
    "capabilities.list",
    "capability.list",
    "lifecycle.status",
    "auth.check",
    "auth.mode",
    "auth.peer_info",
];

/// Classify a method string into its access level.
#[must_use]
pub fn classify_method(method: &str) -> MethodAccessLevel {
    if PUBLIC_METHODS.contains(&method) {
        return MethodAccessLevel::Public;
    }
    for prefix in PUBLIC_METHOD_PREFIXES {
        if method.starts_with(prefix) {
            return MethodAccessLevel::Public;
        }
    }
    MethodAccessLevel::Protected
}

/// Peer credentials extracted from `SO_PEERCRED` on Unix sockets.
///
/// Uses only stable subset: `uid` (stable since 1.75) and `pid`.
#[derive(Debug, Clone)]
pub struct PeerCredentials {
    /// Process ID of the caller (if available).
    pub pid: Option<u32>,
    /// User ID of the caller.
    pub uid: u32,
}

/// How the caller connected.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConnectionOrigin {
    /// Local Unix domain socket.
    Unix,
    /// TCP loopback (127.0.0.1 / ::1).
    Loopback,
    /// Remote TCP connection.
    Remote,
}

/// Identity and authorization context for an incoming RPC call.
#[derive(Debug, Clone)]
pub struct CallerContext {
    /// Optional bearer / capability token sent in the request.
    pub bearer_token: Option<String>,
    /// Peer credentials from `SO_PEERCRED` (Unix socket only).
    pub peer: Option<PeerCredentials>,
    /// Where the connection came from.
    pub origin: ConnectionOrigin,
}

impl CallerContext {
    /// Build a caller context for a Unix socket with no peer credentials.
    ///
    /// `SO_PEERCRED` extraction is deferred until `peer_credentials_unix_socket`
    /// stabilizes. The gate operates on bearer tokens and connection origin.
    #[must_use]
    pub const fn unix() -> Self {
        Self {
            bearer_token: None,
            peer: None,
            origin: ConnectionOrigin::Unix,
        }
    }

    /// Build a caller context for loopback TCP with no peer credentials.
    #[must_use]
    pub const fn loopback() -> Self {
        Self {
            bearer_token: None,
            peer: None,
            origin: ConnectionOrigin::Loopback,
        }
    }

    /// Build a caller context for remote TCP with no peer credentials.
    #[must_use]
    pub const fn remote() -> Self {
        Self {
            bearer_token: None,
            peer: None,
            origin: ConnectionOrigin::Remote,
        }
    }

    /// Attach a bearer token to this context.
    #[must_use]
    pub fn with_bearer_token(mut self, token: String) -> Self {
        self.bearer_token = Some(token);
        self
    }
}

/// Enforcement mode for the method gate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EnforcementMode {
    /// Log violations but allow all calls (backward-compatible default).
    Permissive,
    /// Reject unauthenticated calls to protected methods.
    Enforced,
}

impl EnforcementMode {
    /// Resolve from `BIOMEOS_AUTH_MODE` env var.
    /// Defaults to `Permissive` if unset or unrecognized.
    #[must_use]
    pub fn from_env() -> Self {
        match std::env::var("BIOMEOS_AUTH_MODE")
            .unwrap_or_default()
            .to_lowercase()
            .as_str()
        {
            "enforced" | "enforce" | "strict" => Self::Enforced,
            _ => Self::Permissive,
        }
    }

    /// Human-readable label for diagnostics and `auth.mode` responses.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Permissive => "permissive",
            Self::Enforced => "enforced",
        }
    }
}

/// Pre-dispatch gate that checks caller authorization before method execution.
#[derive(Debug, Clone)]
pub struct MethodGate {
    mode: EnforcementMode,
}

impl MethodGate {
    /// Create a gate with the given enforcement mode.
    #[must_use]
    pub const fn new(mode: EnforcementMode) -> Self {
        Self { mode }
    }

    /// Create a gate from the environment (`BIOMEOS_AUTH_MODE`).
    #[must_use]
    pub fn from_env() -> Self {
        Self::new(EnforcementMode::from_env())
    }

    /// Current enforcement mode.
    #[must_use]
    pub const fn mode(&self) -> EnforcementMode {
        self.mode
    }

    /// Pre-dispatch authorization check.
    ///
    /// Returns `Ok(())` if the call should proceed.
    ///
    /// # Errors
    ///
    /// Returns `JsonRpcError` with `PERMISSION_DENIED` (-32001) when a
    /// protected method is called without a valid capability token and the
    /// gate is in `Enforced` mode.
    pub fn check(&self, method: &str, caller: &CallerContext) -> Result<(), JsonRpcError> {
        let level = classify_method(method);

        if level == MethodAccessLevel::Public {
            return Ok(());
        }

        let authorized = caller.bearer_token.is_some();

        if authorized {
            return Ok(());
        }

        match self.mode {
            EnforcementMode::Permissive => {
                tracing::warn!(
                    method,
                    caller_uid = caller.peer.as_ref().map(|p| p.uid),
                    caller_pid = caller.peer.as_ref().and_then(|p| p.pid),
                    origin = ?caller.origin,
                    "method gate: unauthenticated call to protected method (permissive — allowing)"
                );
                Ok(())
            }
            EnforcementMode::Enforced => {
                tracing::warn!(
                    method,
                    caller_uid = caller.peer.as_ref().map(|p| p.uid),
                    caller_pid = caller.peer.as_ref().and_then(|p| p.pid),
                    origin = ?caller.origin,
                    "method gate: REJECTED unauthenticated call to protected method"
                );
                Err(JsonRpcError::permission_denied(method))
            }
        }
    }

    /// Handle the `auth.check` introspection method.
    #[must_use]
    pub fn handle_auth_check(&self, caller: &CallerContext) -> serde_json::Value {
        serde_json::json!({
            "authenticated": caller.bearer_token.is_some(),
            "mode": self.mode.as_str(),
        })
    }

    /// Handle the `auth.mode` introspection method.
    #[must_use]
    pub fn handle_auth_mode(&self) -> serde_json::Value {
        serde_json::json!({
            "mode": self.mode.as_str(),
        })
    }

    /// Handle the `auth.peer_info` introspection method.
    #[must_use]
    pub fn handle_auth_peer_info(&self, caller: &CallerContext) -> serde_json::Value {
        serde_json::json!({
            "origin": format!("{:?}", caller.origin),
            "has_token": caller.bearer_token.is_some(),
            "peer_uid": caller.peer.as_ref().map(|p| p.uid),
            "peer_pid": caller.peer.as_ref().and_then(|p| p.pid),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn health_methods_are_public() {
        assert_eq!(classify_method("health.check"), MethodAccessLevel::Public);
        assert_eq!(
            classify_method("health.liveness"),
            MethodAccessLevel::Public
        );
        assert_eq!(
            classify_method("health.readiness"),
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
    fn graph_methods_are_protected() {
        assert_eq!(
            classify_method("graph.execute"),
            MethodAccessLevel::Protected
        );
        assert_eq!(classify_method("graph.save"), MethodAccessLevel::Protected);
    }

    #[test]
    fn capability_call_is_protected() {
        assert_eq!(
            classify_method("capability.call"),
            MethodAccessLevel::Protected
        );
    }

    #[test]
    fn topology_methods_are_protected() {
        assert_eq!(
            classify_method("topology.get"),
            MethodAccessLevel::Protected
        );
    }

    #[test]
    fn composition_methods_are_protected() {
        assert_eq!(
            classify_method("composition.health"),
            MethodAccessLevel::Protected
        );
    }

    #[test]
    fn empty_method_is_protected() {
        assert_eq!(classify_method(""), MethodAccessLevel::Protected);
    }

    #[test]
    fn unregistered_methods_are_protected() {
        assert_eq!(
            classify_method("custom.action"),
            MethodAccessLevel::Protected
        );
    }

    #[test]
    fn loopback_context_has_no_peer() {
        let ctx = CallerContext::loopback();
        assert!(ctx.peer.is_none());
        assert!(ctx.bearer_token.is_none());
        assert_eq!(ctx.origin, ConnectionOrigin::Loopback);
    }

    #[test]
    fn unix_context_has_correct_origin() {
        let ctx = CallerContext::unix();
        assert_eq!(ctx.origin, ConnectionOrigin::Unix);
    }

    #[test]
    fn remote_context_has_correct_origin() {
        let ctx = CallerContext::remote();
        assert_eq!(ctx.origin, ConnectionOrigin::Remote);
    }

    #[test]
    fn with_bearer_token_attaches_token() {
        let ctx = CallerContext::loopback().with_bearer_token("tok123".to_owned());
        assert_eq!(ctx.bearer_token.as_deref(), Some("tok123"));
    }

    #[test]
    fn enforcement_mode_as_str() {
        assert_eq!(EnforcementMode::Permissive.as_str(), "permissive");
        assert_eq!(EnforcementMode::Enforced.as_str(), "enforced");
    }

    #[test]
    fn public_method_always_passes() {
        let gate = MethodGate::new(EnforcementMode::Enforced);
        let caller = CallerContext::loopback();
        assert!(gate.check("health.check", &caller).is_ok());
        assert!(gate.check("identity.get", &caller).is_ok());
        assert!(gate.check("capabilities.list", &caller).is_ok());
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
        let result = gate.check("graph.execute", &caller);
        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.code, -32_001);
        assert!(err.message.contains("graph.execute"));
    }

    #[test]
    fn protected_method_passes_in_enforced_mode_with_token() {
        let gate = MethodGate::new(EnforcementMode::Enforced);
        let caller = CallerContext::loopback().with_bearer_token("valid-token".to_owned());
        assert!(gate.check("graph.execute", &caller).is_ok());
    }

    #[test]
    fn gate_error_includes_method_in_data() {
        let gate = MethodGate::new(EnforcementMode::Enforced);
        let caller = CallerContext::loopback();
        let err = gate.check("graph.validate", &caller).unwrap_err();
        let method_in_data = err
            .data
            .as_ref()
            .and_then(|d| d.get("method"))
            .and_then(serde_json::Value::as_str);
        assert_eq!(method_in_data, Some("graph.validate"));
    }

    #[test]
    fn auth_check_unauthenticated() {
        let gate = MethodGate::new(EnforcementMode::Permissive);
        let caller = CallerContext::loopback();
        let result = gate.handle_auth_check(&caller);
        assert_eq!(result["authenticated"], false);
        assert_eq!(result["mode"], "permissive");
    }

    #[test]
    fn auth_check_authenticated() {
        let gate = MethodGate::new(EnforcementMode::Enforced);
        let caller = CallerContext::loopback().with_bearer_token("tok".to_owned());
        let result = gate.handle_auth_check(&caller);
        assert_eq!(result["authenticated"], true);
        assert_eq!(result["mode"], "enforced");
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
}
