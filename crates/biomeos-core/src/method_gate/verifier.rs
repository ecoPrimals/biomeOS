// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Token verification abstraction (primalSpring pattern).
//!
//! Production uses [`BearDogVerifier`] (IPC to BearDog's `auth.verify_ionic`).
//! Tests use [`NoopVerifier`] to avoid requiring a live BearDog process.

use super::ionic::IonicTokenClaims;

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
        let path = std::env::var(biomeos_types::env_config::vars::BEARDOG_SOCKET)
            .ok()
            .map(std::path::PathBuf::from)
            .or_else(|| {
                biomeos_types::paths::SystemPaths::new()
                    .ok()
                    .map(|p| p.primal_socket(biomeos_types::primal_names::BEARDOG))
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

    /// Exposed for testing — the configured socket path.
    #[cfg(test)]
    pub(crate) fn socket_path(&self) -> &std::path::Path {
        &self.socket_path
    }
}

impl TokenVerifier for BearDogVerifier {
    /// Sync fallback — parses locally (no IPC). Use `verify_async` for
    /// full federation verification in async contexts.
    fn verify(&self, token: &str) -> Option<IonicTokenClaims> {
        IonicTokenClaims::parse(token)
    }
}
