// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Method access classification for the pre-dispatch gate.

/// Access level for a JSON-RPC method.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MethodAccessLevel {
    /// Health probes, identity, capability advertisement — always allowed.
    Public,
    /// Orchestration methods — allowed for local callers (Unix/loopback)
    /// without a token, but require a token from remote callers.
    LocalTrusted,
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

/// Methods trusted for local callers (Unix socket / loopback) without a token.
/// These are orchestration methods that nucleus-deploy and other local tools call.
const LOCAL_TRUSTED_PREFIXES: &[&str] = &["composition.", "graph.", "deploy."];

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
    for prefix in LOCAL_TRUSTED_PREFIXES {
        if method.starts_with(prefix) {
            return MethodAccessLevel::LocalTrusted;
        }
    }
    MethodAccessLevel::Protected
}
