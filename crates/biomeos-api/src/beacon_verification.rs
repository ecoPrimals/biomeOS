// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Shared Dark Forest beacon verification
//!
//! Single source of truth for beacon token verification. All verification flows use this
//! module — no duplicate crypto logic.
//!
//! ## WateringHole / runtime discovery
//!
//! - **No hardcoded primal names**: Atomic sockets are found by scanning the runtime
//!   directory for family-scoped `*.sock` files (`{instance}-{family_id}.sock`), not by
//!   enumerating known primal identifiers.
//! - **Neural API first**: When available, verification routes through
//!   `capability.call` on the Neural API socket (semantic routing).
//! - **Fail-closed**: Returns `None` on any ambiguity or failure.
//!
//! ## Architecture
//!
//! ```text
//! dark_forest_gate.rs ─┐
//!                      ├─→ beacon_verification::verify_dark_forest_token()
//! rendezvous.rs ───────┘       │
//!                              ├─→ Neural API (preferred)
//!                              │   └─→ capability routing (e.g. birdsong.decrypt)
//!                              │
//!                              └─→ Direct socket discovery (fallback)
//!                                  └─→ AtomicClient → any family-scoped primal socket
//! ```

use base64::Engine;
use biomeos_types::constants::runtime_ipc;
use serde_json::Value;
use std::path::{Path, PathBuf};
use tracing::{debug, warn};

/// Result of a successful beacon verification
#[derive(Debug, Clone)]
pub struct BeaconVerification {
    /// Family ID from decryption result (or local `family_id`)
    pub family_id: String,
    /// Decrypted plaintext (for lineage hashing)
    pub plaintext: String,
}

/// Verify a Dark Forest token via Neural API capability routing
///
/// This is the **single source of truth** for all beacon verification.
/// Used by both the Dark Forest gate middleware and rendezvous handlers.
///
/// ## Resolution Order
///
/// 1. **Neural API** — preferred, semantic capability routing
/// 2. **Direct socket discovery** — fallback when Neural API is not running: try
///    `birdsong.decrypt` on every family-scoped primal socket under the runtime dir
///
/// ## Security
///
/// - Both `success` AND non-empty `plaintext` must be present (AND, not OR)
/// - Returns `None` on any verification failure — fail-closed
/// - No information leaked about why verification failed
pub async fn verify_dark_forest_token(
    neural_api_socket: Option<&str>,
    family_id: &str,
    token: &str,
) -> Option<BeaconVerification> {
    // PRIMARY: Try Neural API capability routing
    if let Some(socket) = neural_api_socket {
        if let Some(result) = verify_via_neural_api(socket, family_id, token).await {
            return Some(result);
        }
        // Neural API failed — fall through to direct discovery
        debug!("🌲 Neural API verification failed, trying direct socket discovery");
    }

    // FALLBACK: Direct socket discovery (when Neural API is unavailable)
    verify_via_socket_discovery(family_id, token).await
}

/// Hash a token via Neural API or direct crypto routing
///
/// Used by rendezvous to create lineage-based slot keys without
/// exposing the raw token.
pub async fn hash_via_capability(
    neural_api_socket: Option<&str>,
    family_id: &str,
    data: &str,
) -> Option<String> {
    let encoded = base64::engine::general_purpose::STANDARD.encode(data.as_bytes());

    // PRIMARY: Neural API
    if let Some(socket) = neural_api_socket {
        if let Ok(client) = neural_api_client::NeuralApiClient::new(socket) {
            let result = client
                .route_to_primal(
                    "crypto",
                    "crypto.blake3_hash",
                    serde_json::json!({ "data": encoded }),
                )
                .await;

            if let Ok(value) = result {
                if let Some(hash) = value.get("hash").and_then(|h| h.as_str()) {
                    return Some(hash.to_string());
                }
            }
        }
    }

    // FALLBACK: Capability-named socket (e.g. `crypto.sock`), then any family-scoped primal
    let discovery = biomeos_core::socket_discovery::SocketDiscovery::new(family_id);
    if let Some(sock) = discovery
        .discover_capability(biomeos_types::constants::capability::CRYPTO)
        .await
    {
        if let Some(hash) = try_blake3_hash(&sock.path, &encoded).await {
            return Some(hash);
        }
    }

    let paths = biomeos_types::paths::SystemPaths::new_lazy();
    for socket_path in discover_family_scoped_primal_sockets(paths.runtime_dir(), family_id) {
        if let Some(hash) = try_blake3_hash(&socket_path, &encoded).await {
            return Some(hash);
        }
    }

    None
}

async fn try_blake3_hash(socket_path: &Path, encoded_data: &str) -> Option<String> {
    let client = biomeos_core::AtomicClient::unix(socket_path.to_string_lossy().as_ref())
        .with_timeout(std::time::Duration::from_secs(5));

    let r = client
        .call(
            "crypto.blake3_hash",
            serde_json::json!({ "data": encoded_data }),
        )
        .await
        .ok()?;

    r.get("hash").and_then(|h| h.as_str()).map(str::to_string)
}

/// Discover the Neural API socket for the current family
///
/// Resolution order:
/// 1. `NEURAL_API_SOCKET` environment variable
/// 2. XDG runtime dir: `{NEURAL_API_BASENAME_PREFIX}{family_id}.sock`
/// 3. `/tmp` fallback: same basename under the temp dir
#[must_use]
pub fn discover_neural_api_socket(family_id: &str) -> Option<String> {
    discover_neural_api_socket_from(family_id, None)
}

/// Like [`discover_neural_api_socket`], with an optional tier-1 socket path.
///
/// - `None`: read `NEURAL_API_SOCKET` from the environment (same as [`discover_neural_api_socket`]).
/// - `Some(path)` if the path exists: return it (environment is not consulted for tier 1).
/// - `Some(path)` if the path does not exist: skip tier 1 without reading `NEURAL_API_SOCKET` (for tests and explicit fall-through to XDG/tmp).
#[must_use]
pub fn discover_neural_api_socket_from(
    family_id: &str,
    neural_api_socket_tier1: Option<&str>,
) -> Option<String> {
    // 1. Tier 1: explicit path or NEURAL_API_SOCKET
    match neural_api_socket_tier1 {
        Some(path) => {
            if std::path::Path::new(path).exists() {
                return Some(path.to_string());
            }
        }
        None => {
            if let Ok(socket) = std::env::var(biomeos_types::env_config::vars::NEURAL_API_SOCKET) {
                if std::path::Path::new(&socket).exists() {
                    return Some(socket);
                }
            }
        }
    }

    // 2. XDG runtime dir
    let paths = biomeos_types::paths::SystemPaths::new_lazy();
    let basename = format!(
        "{}{family_id}.sock",
        runtime_ipc::NEURAL_API_BASENAME_PREFIX
    );
    let xdg_path = paths.runtime_dir().join(&basename);
    if xdg_path.exists() {
        return Some(xdg_path.to_string_lossy().to_string());
    }

    // 3. Fallback runtime base (bootstrap scenarios)
    let fallback =
        std::path::PathBuf::from(biomeos_types::constants::runtime_paths::FALLBACK_RUNTIME_BASE)
            .join(&basename);
    if fallback.exists() {
        return Some(fallback.to_string_lossy().to_string());
    }

    None
}

// ── Internal implementation ──────────────────────────────────────────────────

/// Verify via Neural API capability.call (preferred path)
async fn verify_via_neural_api(
    neural_socket: &str,
    family_id: &str,
    token: &str,
) -> Option<BeaconVerification> {
    let client = match neural_api_client::NeuralApiClient::new(neural_socket) {
        Ok(c) => c,
        Err(e) => {
            warn!("🌲 Neural API client creation failed: {}", e);
            return None;
        }
    };

    let result = client
        .route_to_primal(
            "beacon",
            "birdsong.decrypt",
            serde_json::json!({
                "family_id": family_id,
                "ciphertext": token
            }),
        )
        .await;

    match result {
        Ok(value) => parse_decrypt_result(&value, family_id),
        Err(e) => {
            warn!("🌲 Neural API verification failed: {}", e);
            None
        }
    }
}

/// Verify via direct socket discovery (fallback)
async fn verify_via_socket_discovery(family_id: &str, token: &str) -> Option<BeaconVerification> {
    let paths = biomeos_types::paths::SystemPaths::new_lazy();
    let runtime_dir = paths.runtime_dir();

    let providers = discover_family_scoped_primal_sockets(runtime_dir, family_id);

    for socket_path in providers {
        let client = biomeos_core::AtomicClient::unix(socket_path.to_string_lossy().as_ref())
            .with_timeout(std::time::Duration::from_secs(5));

        let params = serde_json::json!({
            "family_id": family_id,
            "ciphertext": token
        });

        match client.call("birdsong.decrypt", params).await {
            Ok(value) => {
                if let Some(result) = parse_decrypt_result(&value, family_id) {
                    debug!(
                        "🌲 Token verified via {} (socket fallback)",
                        socket_path.display()
                    );
                    return Some(result);
                }
            }
            Err(e) => {
                debug!(
                    "🌲 {} decrypt failed: {} — trying next",
                    socket_path.display(),
                    e
                );
            }
        }
    }

    warn!("🌲 No beacon provider could verify token");
    None
}

/// Parse decryption result with strict AND validation
///
/// Both `success: true` AND non-empty `plaintext` required.
/// This prevents accepting partial/failed decryptions.
fn parse_decrypt_result(value: &Value, default_family_id: &str) -> Option<BeaconVerification> {
    let success = value
        .get("success")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    let plaintext = value
        .get("plaintext")
        .and_then(|v| v.as_str())
        .unwrap_or("");

    // CRITICAL: Both conditions must be true (AND, not OR)
    if success && !plaintext.is_empty() {
        let family_id = value
            .get("family_id")
            .and_then(|h| h.as_str())
            .unwrap_or(default_family_id)
            .to_string();

        Some(BeaconVerification {
            family_id,
            plaintext: plaintext.to_string(),
        })
    } else {
        None
    }
}

/// Discover atomic primal sockets for this family under `runtime_dir`
///
/// **Inclusive rule** (WateringHole): any regular file whose name ends with
/// `-{family_id}.sock` is treated as a candidate primal IPC endpoint. Verification
/// succeeds only if `birdsong.decrypt` (or `crypto.blake3_hash` for hashing) responds.
///
/// The Neural API control socket shares the same suffix pattern but is not a primal
/// atomic provider; it is skipped using [`runtime_ipc::NEURAL_API_BASENAME_PREFIX`]
/// (infrastructure IPC, not a primal name).
fn discover_family_scoped_primal_sockets(runtime_dir: &Path, family_id: &str) -> Vec<PathBuf> {
    let suffix = format!("-{family_id}.sock");
    let mut providers = Vec::new();

    let Ok(entries) = std::fs::read_dir(runtime_dir) else {
        return providers;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };

        if !name
            .rsplit_once('.')
            .is_some_and(|(_, ext)| ext.eq_ignore_ascii_case("sock"))
        {
            continue;
        }

        if !name.ends_with(&suffix) {
            continue;
        }

        if name.starts_with(runtime_ipc::NEURAL_API_BASENAME_PREFIX) {
            continue;
        }

        providers.push(path);
    }

    providers.sort();
    providers
}

#[cfg(test)]
#[path = "beacon_verification_tests.rs"]
mod tests;
