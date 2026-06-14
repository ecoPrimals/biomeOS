// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Pre-dispatch capability gate for JSON-RPC methods (JH-0 + JH-2).
//!
//! Implements the ecosystem-standard `MethodGate` pattern defined in
//! `primalSpring/wateringHole/METHOD_GATE_STANDARD.md`. Every incoming
//! RPC call passes through [`MethodGate::check`] *before* the dispatch
//! table, classifying methods as [`MethodAccessLevel::Public`] or
//! [`MethodAccessLevel::Protected`].
//!
//! ## Enforcement modes
//!
//! - **Permissive** (default): protected methods without a token are
//!   logged but allowed, preserving backward compatibility.
//! - **Enforced**: protected methods without a valid token are rejected
//!   with `PERMISSION_DENIED` (-32001). Scope is also checked.
//!
//! ## Ionic token support (JH-2)
//!
//! Bearer tokens in BearDog ionic format (`header.payload.signature`,
//! each segment base64-encoded) are parsed locally to extract scope
//! patterns and resource envelope fields. Scope matching follows the
//! primalSpring standard: `"*"` matches all, `"prefix.*"` matches
//! dot-boundary prefixes, exact string match otherwise.
//!
//! The gate reads its mode from the `BIOMEOS_AUTH_MODE` environment
//! variable (or falls back to `Permissive`).

mod classify;
mod ionic;
mod verifier;

pub use classify::{MethodAccessLevel, classify_method};
pub use ionic::{IonicTokenClaims, ResourceEnvelope, scope_covers_method};
pub use verifier::{LocalClaimsVerifier, SecurityVerifier, TokenVerifier};

/// Backward-compatible alias for [`SecurityVerifier`].
#[deprecated(since = "4.29.0", note = "renamed to SecurityVerifier")]
pub type BearDogVerifier = SecurityVerifier;

#[cfg(any(test, feature = "test-helpers"))]
pub use verifier::NoopVerifier;

use biomeos_types::JsonRpcError;

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
    /// Parsed ionic token claims (populated when the bearer token is
    /// in BearDog ionic format). `None` for opaque or missing tokens.
    pub claims: Option<IonicTokenClaims>,
    /// Peer credentials from `SO_PEERCRED` (Unix socket only).
    pub peer: Option<PeerCredentials>,
    /// Where the connection came from.
    pub origin: ConnectionOrigin,
}

impl CallerContext {
    /// Build a caller context for a Unix socket with no peer credentials.
    #[must_use]
    pub const fn unix() -> Self {
        Self {
            bearer_token: None,
            claims: None,
            peer: None,
            origin: ConnectionOrigin::Unix,
        }
    }

    /// Build a caller context for loopback TCP with no peer credentials.
    #[must_use]
    pub const fn loopback() -> Self {
        Self {
            bearer_token: None,
            claims: None,
            peer: None,
            origin: ConnectionOrigin::Loopback,
        }
    }

    /// Build a caller context for remote TCP with no peer credentials.
    #[must_use]
    pub const fn remote() -> Self {
        Self {
            bearer_token: None,
            claims: None,
            peer: None,
            origin: ConnectionOrigin::Remote,
        }
    }

    /// Attach a bearer token and parse its claims (if ionic format).
    #[must_use]
    pub fn with_bearer_token(mut self, token: String) -> Self {
        self.claims = IonicTokenClaims::parse(&token);
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
        match std::env::var(biomeos_types::env_config::vars::AUTH_MODE)
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
    /// Returns `Ok(())` if the call should proceed. In `Enforced` mode:
    /// - Checks token presence for protected methods.
    /// - Checks scope patterns (from ionic token claims) cover the method.
    /// - Checks resource envelope method allowlist if present.
    /// - Checks token expiry.
    ///
    /// # Errors
    ///
    /// Returns `JsonRpcError` with `PERMISSION_DENIED` (-32001) when a
    /// protected method is called without a valid capability token, or
    /// when the token's scope/allowlist does not cover the method.
    pub fn check(&self, method: &str, caller: &CallerContext) -> Result<(), JsonRpcError> {
        let level = classify_method(method);

        if level == MethodAccessLevel::Public {
            return Ok(());
        }

        // Local-trusted methods: allow from Unix/loopback without a token
        if level == MethodAccessLevel::LocalTrusted && caller.origin != ConnectionOrigin::Remote {
            return Ok(());
        }

        let has_token = caller.bearer_token.is_some();

        if !has_token {
            return self.handle_no_token(method, caller);
        }

        if let Some(ref claims) = caller.claims {
            self.validate_claims(method, claims)?;
        }

        Ok(())
    }

    fn handle_no_token(&self, method: &str, caller: &CallerContext) -> Result<(), JsonRpcError> {
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

    fn validate_claims(&self, method: &str, claims: &IonicTokenClaims) -> Result<(), JsonRpcError> {
        if claims.is_expired() {
            return self.mode_gate(method, "expired token");
        }
        if !claims.scope_covers_method(method) {
            return self.mode_gate(method, "token scope does not cover method");
        }
        if !claims.method_in_allowlist(method) {
            return self.mode_gate(method, "method not in resource envelope allowlist");
        }
        Ok(())
    }

    fn mode_gate(&self, method: &str, reason: &str) -> Result<(), JsonRpcError> {
        match self.mode {
            EnforcementMode::Permissive => {
                tracing::warn!(method, "method gate: {reason} (permissive — allowing)");
                Ok(())
            }
            EnforcementMode::Enforced => {
                tracing::warn!(method, "method gate: REJECTED — {reason}");
                Err(JsonRpcError::permission_denied(method))
            }
        }
    }

    /// Handle the `auth.check` introspection method.
    ///
    /// Returns a superset of the primalSpring contract fields:
    /// `{ authenticated, verified, enforcement, scopes, subject, expires_in }`.
    #[must_use]
    pub fn handle_auth_check(&self, caller: &CallerContext) -> serde_json::Value {
        let has_token = caller.bearer_token.is_some();
        let has_claims = caller.claims.is_some();
        let mut result = serde_json::json!({
            "authenticated": has_token,
            "verified": has_claims,
            "mode": self.mode.as_str(),
            "enforcement": self.mode.as_str(),
        });
        if let Some(ref claims) = caller.claims {
            result["subject"] = serde_json::json!(claims.sub);
            result["scope"] = serde_json::json!(claims.scope);
            result["scopes"] = serde_json::json!(claims.scope);
            result["expired"] = serde_json::json!(claims.is_expired());
            let expires_in = if claims.exp > 0 {
                let now = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_or(0, |d| d.as_secs());
                claims.exp.saturating_sub(now)
            } else {
                0
            };
            result["expires_in"] = serde_json::json!(expires_in);
            if let Some(ref env) = claims.resources {
                result["has_resource_envelope"] = serde_json::json!(true);
                result["resource_envelope"] = serde_json::json!({
                    "mem": env.mem,
                    "cpu": env.cpu,
                    "timeout_ms": env.timeout_ms,
                    "method_allowlist_count": env.method_allowlist.as_ref().map(Vec::len),
                });
            }
        }
        result
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
#[path = "tests.rs"]
mod tests;
