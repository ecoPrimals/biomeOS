// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Ionic token claims parsing and scope matching (JH-2).
//!
//! Bearer tokens in BearDog ionic format (`header.payload.signature`,
//! each segment base64-encoded) carry scope patterns and resource envelope
//! fields. This module handles local parsing only — signature verification
//! is delegated to BearDog via the [`super::verifier`] module.

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
            .map_or(0, |d| d.as_secs());
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
