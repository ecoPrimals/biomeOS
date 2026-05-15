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

    // 2. Map primal names to their capability-domain socket names.
    //    ToadStool binds as `compute-{family_id}.sock`, NestGate as `storage-{family_id}.sock`.
    //    If the domain-based socket exists, prefer it over the primal-name-based path.
    let domain_alias = match primal {
        "toadstool" => Some("compute"),
        "nestgate" => Some("storage"),
        _ => None,
    };

    // ToadStool uses dual-socket mode (tarpc + JSON-RPC). biomeOS forwards via
    // JSON-RPC, so always produce the .jsonrpc.sock path for ToadStool.
    let prefers_jsonrpc = primal == "toadstool";

    let paths = biomeos_types::paths::SystemPaths::new_lazy();

    if let Some(domain) = domain_alias {
        let domain_id = format!("{domain}-{family_id}");
        let jsonrpc_id = format!("{domain}-{family_id}.jsonrpc");
        let jsonrpc_path = paths.primal_socket(&jsonrpc_id);
        if jsonrpc_path.exists() {
            return jsonrpc_path.to_string_lossy().to_string();
        }
        let domain_path = paths.primal_socket(&domain_id);
        if domain_path.exists() {
            return domain_path.to_string_lossy().to_string();
        }
    }

    if prefers_jsonrpc {
        let jsonrpc_path = paths.primal_socket(&format!("{primal}-{family_id}.jsonrpc"));
        return jsonrpc_path.to_string_lossy().to_string();
    }

    // 3. XDG-compliant resolution via SystemPaths (primal-name based with family suffix)
    //    Prefer .jsonrpc.sock variants since biomeOS forwards via JSON-RPC.
    let primal_id = format!("{primal}-{family_id}");
    let jsonrpc_family = paths.primal_socket(&format!("{primal}-{family_id}.jsonrpc"));
    if jsonrpc_family.exists() {
        return jsonrpc_family.to_string_lossy().to_string();
    }
    let family_path = paths.primal_socket(&primal_id);
    if family_path.exists() {
        return family_path.to_string_lossy().to_string();
    }

    // 4. Fallback: plain `{primal}.sock` for primals that don't yet support --socket
    //    (e.g. loamspine, sweetgrass, petaltongue bind to `{primal}.sock` without family suffix)
    let plain_jsonrpc = paths.primal_socket(&format!("{primal}.jsonrpc"));
    if plain_jsonrpc.exists() {
        return plain_jsonrpc.to_string_lossy().to_string();
    }
    let plain_path = paths.primal_socket(primal);
    if plain_path.exists() {
        return plain_path.to_string_lossy().to_string();
    }

    // Default: return the family-suffixed path even if it doesn't exist yet
    family_path.to_string_lossy().to_string()
}
