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

// ── Ionic token claims parsing (JH-2) ──

/// Resource envelope carried inside an ionic token (JH-2).
///
/// Constrains what resources a token holder may request. Fields are
/// optional — absent means "no constraint on this dimension".
#[derive(Debug, Clone, Default, serde::Deserialize, serde::Serialize)]
pub struct ResourceEnvelope {
    /// Maximum memory in bytes (e.g. 1 GiB = 1_073_741_824).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mem: Option<u64>,
    /// Maximum CPU cores (fractional, e.g. 2.5).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cpu: Option<f64>,
    /// Maximum dispatch timeout in milliseconds.
    /// When set, biomeOS caps the forwarding timeout to this value.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
    /// Explicit method allowlist. When present, only these methods
    /// (checked *in addition to* scope patterns) are permitted.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method_allowlist: Option<Vec<String>>,
}

/// Parsed claims from a BearDog ionic token payload.
///
/// Token format: `base64(header).base64(payload).base64(signature)`.
/// This struct represents the decoded payload.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct IonicTokenClaims {
    /// Issuer DID (`did:key:z6Mk...`).
    #[serde(default)]
    pub iss: String,
    /// Subject identifier.
    #[serde(default)]
    pub sub: String,
    /// Scope patterns — `"*"`, `"prefix.*"`, or exact method name.
    #[serde(default)]
    pub scope: Vec<String>,
    /// Issued-at timestamp (Unix epoch seconds).
    #[serde(default)]
    pub iat: u64,
    /// Expiry timestamp (Unix epoch seconds).
    #[serde(default)]
    pub exp: u64,
    /// Unique token identifier.
    #[serde(default)]
    pub jti: String,
    /// Resource envelope (JH-2 extension).
    #[serde(default)]
    pub resources: Option<ResourceEnvelope>,
}

impl IonicTokenClaims {
    /// Parse an ionic token string into claims.
    ///
    /// Extracts the middle (payload) segment from a `header.payload.signature`
    /// token and base64-decodes + JSON-parses it. The signature is NOT verified
    /// here — that's BearDog's responsibility via `auth.verify_ionic`. This
    /// local parse is for scope and resource extraction only.
    ///
    /// Returns `None` if the token is not in ionic format or payload cannot be
    /// parsed (e.g. opaque tokens, legacy formats).
    #[must_use]
    pub fn parse(token: &str) -> Option<Self> {
        use base64::Engine;

        let parts: Vec<&str> = token.split('.').collect();
        if parts.len() != 3 {
            return None;
        }

        let payload_bytes = base64::engine::general_purpose::STANDARD
            .decode(parts[1])
            .or_else(|_| base64::engine::general_purpose::STANDARD_NO_PAD.decode(parts[1]))
            .or_else(|_| base64::engine::general_purpose::URL_SAFE_NO_PAD.decode(parts[1]))
            .ok()?;

        serde_json::from_slice(&payload_bytes).ok()
    }

    /// Check whether this token's scope covers the given method.
    ///
    /// Scope matching rules (primalSpring standard):
    /// - `"*"` matches everything.
    /// - `"prefix.*"` matches any method starting with `prefix.`.
    /// - Exact string match otherwise.
    #[must_use]
    pub fn scope_covers_method(&self, method: &str) -> bool {
        scope_covers_method(&self.scope, method)
    }

    /// Whether this token has expired based on the current system time.
    #[must_use]
    pub fn is_expired(&self) -> bool {
        if self.exp == 0 {
            return false;
        }
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        now > self.exp
    }

    /// Check whether a resource request fits within this token's envelope.
    ///
    /// Returns `true` if the request is within limits (or if no envelope
    /// is set). Checks `mem` and `cpu` when both request and envelope
    /// specify them.
    #[must_use]
    pub fn resource_allowed(&self, requested_mem: Option<u64>, requested_cpu: Option<f64>) -> bool {
        let Some(ref env) = self.resources else {
            return true;
        };
        if let (Some(limit), Some(req)) = (env.mem, requested_mem) {
            if req > limit {
                return false;
            }
        }
        if let (Some(limit), Some(req)) = (env.cpu, requested_cpu) {
            if req > limit {
                return false;
            }
        }
        true
    }

    /// Check whether a method is in the resource envelope's method allowlist.
    ///
    /// Returns `true` if no allowlist is set, or if the method is in the list.
    #[must_use]
    pub fn method_in_allowlist(&self, method: &str) -> bool {
        let Some(ref env) = self.resources else {
            return true;
        };
        let Some(ref allowlist) = env.method_allowlist else {
            return true;
        };
        allowlist.iter().any(|m| m == method)
    }

    /// Get the dispatch timeout cap from the resource envelope (if any).
    ///
    /// Returns `None` when no envelope or no `timeout_ms` is set.
    #[must_use]
    pub fn dispatch_timeout_ms(&self) -> Option<u64> {
        self.resources.as_ref().and_then(|e| e.timeout_ms)
    }
}

impl ResourceEnvelope {
    /// Serialize this envelope as a JSON value suitable for injection into
    /// forwarded request params (`_resource_envelope`).
    ///
    /// Downstream primals (e.g. ToadStool) read this field to enforce
    /// `cpu`, `mem`, and `timeout_ms` at the compute dispatch level.
    #[must_use]
    pub fn to_forwarding_value(&self) -> serde_json::Value {
        serde_json::json!({
            "mem": self.mem,
            "cpu": self.cpu,
            "timeout_ms": self.timeout_ms,
        })
    }
}

/// Check whether a set of scope patterns covers a method.
///
/// - `"*"` matches everything.
/// - `"prefix.*"` matches any method starting with `prefix.`.
/// - Exact string match otherwise.
#[must_use]
pub fn scope_covers_method(scope: &[String], method: &str) -> bool {
    if scope.is_empty() {
        return false;
    }
    for pattern in scope {
        if pattern == "*" {
            return true;
        }
        if let Some(prefix) = pattern.strip_suffix(".*") {
            if method.starts_with(prefix) && method.as_bytes().get(prefix.len()) == Some(&b'.') {
                return true;
            }
        }
        if pattern == method {
            return true;
        }
    }
    false
}

// ── Token verification abstraction (primalSpring pattern) ──

/// Trait for verifying bearer tokens.
///
/// Production uses `BearDogVerifier` (IPC to BearDog's `auth.verify_ionic`).
/// Tests use `NoopVerifier` to avoid requiring a live BearDog process.
/// Follows the primalSpring `TokenVerifier` pattern from `method_gate.rs`.
pub trait TokenVerifier: Send + Sync {
    /// Verify a bearer token and return parsed claims on success.
    ///
    /// Returns `None` if the token is invalid, expired, or the verifier
    /// cannot reach the issuing authority.
    fn verify(&self, token: &str) -> Option<IonicTokenClaims>;
}

/// Local-only verifier that parses ionic token claims without signature
/// verification. Used as the default when BearDog IPC is unavailable.
///
/// This is the same local parsing that `IonicTokenClaims::parse()` performs —
/// scope/expiry/resource checks still happen in `MethodGate::check()`.
pub struct LocalClaimsVerifier;

impl TokenVerifier for LocalClaimsVerifier {
    fn verify(&self, token: &str) -> Option<IonicTokenClaims> {
        IonicTokenClaims::parse(token)
    }
}

/// No-op verifier for testing. Accepts any token as valid with no claims.
#[cfg(any(test, feature = "test-helpers"))]
pub struct NoopVerifier;

#[cfg(any(test, feature = "test-helpers"))]
impl TokenVerifier for NoopVerifier {
    fn verify(&self, _token: &str) -> Option<IonicTokenClaims> {
        None
    }
}

/// BearDog IPC verifier for cross-primal token federation (JH-11).
///
/// Calls `auth.verify_ionic` on BearDog via IPC to cryptographically verify
/// a bearer token's signature. Falls back to `LocalClaimsVerifier` (parse-only)
/// if BearDog is unreachable, ensuring graceful degradation.
///
/// This is step 1 of the federation roadmap — verify-then-forward. Step 2
/// (offline verification via shared-key distribution) requires BearDog to
/// ship key distribution, which is tracked as JH-11 on BearDog's side.
#[derive(Clone)]
pub struct BearDogVerifier {
    socket_path: std::path::PathBuf,
}

impl BearDogVerifier {
    /// Create a new verifier targeting a BearDog socket.
    pub fn new(socket_path: std::path::PathBuf) -> Self {
        Self { socket_path }
    }

    /// Resolve the BearDog socket from environment or XDG defaults.
    pub fn from_env() -> Option<Self> {
        let path = std::env::var("BEARDOG_SOCKET")
            .ok()
            .map(std::path::PathBuf::from)
            .or_else(|| {
                biomeos_types::paths::SystemPaths::new()
                    .ok()
                    .map(|p| p.primal_socket("bearDog"))
            })?;
        Some(Self::new(path))
    }

    /// Async verification via IPC to BearDog's `auth.verify_ionic`.
    ///
    /// Returns parsed claims if BearDog confirms the token is valid.
    /// Falls back to local parsing if BearDog is unreachable.
    pub async fn verify_async(&self, token: &str) -> Option<IonicTokenClaims> {
        use crate::atomic_client::AtomicClient;
        use serde_json::json;

        let client = AtomicClient::unix(&self.socket_path)
            .with_timeout(biomeos_types::constants::timeouts::DEFAULT_IPC_TIMEOUT);

        match client
            .call("auth.verify_ionic", json!({ "token": token }))
            .await
        {
            Ok(result) => {
                if result.get("valid").and_then(serde_json::Value::as_bool) == Some(true) {
                    IonicTokenClaims::parse(token)
                } else {
                    None
                }
            }
            Err(_) => {
                // BearDog unreachable — degrade to local parsing.
                // In enforced mode, the MethodGate will still check expiry/scope.
                IonicTokenClaims::parse(token)
            }
        }
    }
}

impl TokenVerifier for BearDogVerifier {
    /// Sync fallback — parses locally (no IPC). Use `verify_async` for
    /// full federation verification in async contexts.
    fn verify(&self, token: &str) -> Option<IonicTokenClaims> {
        IonicTokenClaims::parse(token)
    }
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
    ///
    /// `SO_PEERCRED` extraction is deferred until `peer_credentials_unix_socket`
    /// stabilizes. The gate operates on bearer tokens and connection origin.
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

        let has_token = caller.bearer_token.is_some();

        if !has_token {
            return match self.mode {
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
            };
        }

        // Token present — check scope and envelope if claims are available.
        if let Some(ref claims) = caller.claims {
            if claims.is_expired() {
                return match self.mode {
                    EnforcementMode::Permissive => {
                        tracing::warn!(
                            method,
                            "method gate: expired token (permissive — allowing)"
                        );
                        Ok(())
                    }
                    EnforcementMode::Enforced => {
                        tracing::warn!(method, "method gate: REJECTED expired token");
                        Err(JsonRpcError::permission_denied(method))
                    }
                };
            }

            if !claims.scope_covers_method(method) {
                return match self.mode {
                    EnforcementMode::Permissive => {
                        tracing::warn!(
                            method,
                            scope = ?claims.scope,
                            "method gate: token scope does not cover method (permissive — allowing)"
                        );
                        Ok(())
                    }
                    EnforcementMode::Enforced => {
                        tracing::warn!(
                            method,
                            scope = ?claims.scope,
                            "method gate: REJECTED — token scope does not cover method"
                        );
                        Err(JsonRpcError::permission_denied(method))
                    }
                };
            }

            if !claims.method_in_allowlist(method) {
                return match self.mode {
                    EnforcementMode::Permissive => {
                        tracing::warn!(
                            method,
                            "method gate: method not in resource envelope allowlist (permissive — allowing)"
                        );
                        Ok(())
                    }
                    EnforcementMode::Enforced => {
                        tracing::warn!(
                            method,
                            "method gate: REJECTED — method not in resource envelope allowlist"
                        );
                        Err(JsonRpcError::permission_denied(method))
                    }
                };
            }
        }

        Ok(())
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
                    .map(|d| d.as_secs())
                    .unwrap_or(0);
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
mod tests {
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
    fn graph_methods_are_protected() {
        assert_eq!(
            classify_method("graph.execute"),
            MethodAccessLevel::Protected
        );
        assert_eq!(classify_method("graph.save"), MethodAccessLevel::Protected);
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
        let err = gate.check("graph.execute", &caller).unwrap_err();
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
        let err = gate.check("graph.execute", &caller).unwrap_err();
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
        let err = gate.check("graph.execute", &caller).unwrap_err();
        assert_eq!(err.code, -32_001);
    }

    #[test]
    fn method_allowlist_enforced() {
        let gate = MethodGate::new(EnforcementMode::Enforced);
        let token = make_ionic_token(&serde_json::json!({
            "scope": ["*"],
            "exp": 9999999999u64,
            "resources": { "method_allowlist": ["graph.list"] }
        }));
        let caller = CallerContext::loopback().with_bearer_token(token);
        assert!(gate.check("graph.list", &caller).is_ok());
        let err = gate.check("graph.execute", &caller).unwrap_err();
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
        let err = gate.check("graph.validate", &caller).unwrap_err();
        let method_in_data = err
            .data
            .as_ref()
            .and_then(|d| d.get("method"))
            .and_then(serde_json::Value::as_str);
        assert_eq!(method_in_data, Some("graph.validate"));
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
        // from_env reads BEARDOG_SOCKET or XDG paths; verify it doesn't panic.
        let _ = BearDogVerifier::from_env();
    }

    #[test]
    fn beardog_verifier_clone() {
        let v = BearDogVerifier::new(std::path::PathBuf::from("/tmp/bd.sock"));
        let v2 = v.clone();
        assert_eq!(v.socket_path, v2.socket_path);
    }
}
