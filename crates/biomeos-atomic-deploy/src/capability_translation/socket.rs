// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Socket path resolution for capability translation.
//!
//! Resolves Unix socket paths for primals using environment variable overrides
//! and XDG-compliant fallbacks.

/// Resolve socket path for a primal
///
/// Priority:
/// 1. `$PRIMAL_SOCKET` environment variable (e.g., `$BEARDOG_SOCKET`)
/// 2. `SystemPaths::new_lazy().primal_socket()` (XDG-compliant, handles
///    `XDG_RUNTIME_DIR`, `/run/user/{uid}`, and `/tmp` fallbacks)
#[must_use]
pub fn resolve_primal_socket(primal: &str, family_id: &str) -> String {
    resolve_primal_socket_with(primal, family_id, None)
}

/// Like [`resolve_primal_socket`], but allows supplying a socket path directly (for tests).
///
/// When `socket_override` is `Some`, it is returned and environment is not consulted for that override.
#[must_use]
pub fn resolve_primal_socket_with(
    primal: &str,
    family_id: &str,
    socket_override: Option<&str>,
) -> String {
    if let Some(socket) = socket_override {
        return socket.to_string();
    }

    // 1. Check environment variable override (primal-specific)
    let env_var = format!("{}_SOCKET", primal.to_uppercase());
    if let Ok(socket) = std::env::var(&env_var) {
        return socket;
    }

    // 2. XDG-compliant resolution via SystemPaths
    let primal_id = format!("{primal}-{family_id}");
    biomeos_types::paths::SystemPaths::new_lazy()
        .primal_socket(&primal_id)
        .to_string_lossy()
        .to_string()
}
