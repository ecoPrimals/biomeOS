//! Shared Beacon Verification — Deep Debt Evolution
//!
//! Single source of truth for Dark Forest beacon verification.
//! All verification flows use this module — no duplicate crypto logic.
//!
//! ## Deep Debt Principles
//!
//! - **No direct primal knowledge**: Routes through Neural API capability.call
//! - **No hardcoded socket paths**: Discovers at runtime via SystemPaths
//! - **No duplicate logic**: One function, used everywhere
//! - **Fail-closed**: Returns `Err` on any ambiguity
//!
//! ## Architecture
//!
//! ```text
//! dark_forest_gate.rs ─┐
//!                      ├─→ beacon_verification::verify_dark_forest_token()
//! rendezvous.rs ───────┘       │
//!                              ├─→ Neural API (preferred)
//!                              │   └─→ capability.call("birdsong", "decrypt")
//!                              │
//!                              └─→ Direct socket discovery (fallback)
//!                                  └─→ AtomicClient → discovered provider
//! ```

use base64::Engine;
use serde_json::Value;
use tracing::{debug, warn};

/// Result of a successful beacon verification
#[derive(Debug, Clone)]
pub struct BeaconVerification {
    /// Family ID from decryption result (or local family_id)
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
/// 1. **Neural API** `capability.call("birdsong", "decrypt")` — preferred, semantic routing
/// 2. **Direct socket discovery** — fallback when Neural API is not running
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

    // FALLBACK: Direct socket discovery
    let paths = biomeos_types::paths::SystemPaths::new_lazy();
    let runtime_dir = paths.runtime_dir();
    let socket_path = runtime_dir.join(format!("beardog-{}.sock", family_id));

    if socket_path.exists() {
        let client = biomeos_core::AtomicClient::unix(socket_path.to_string_lossy().as_ref())
            .with_timeout(std::time::Duration::from_secs(5));

        if let Ok(r) = client
            .call("crypto.blake3_hash", serde_json::json!({ "data": encoded }))
            .await
        {
            if let Some(hash) = r.get("hash").and_then(|h| h.as_str()) {
                return Some(hash.to_string());
            }
        }
    }

    None
}

/// Discover the Neural API socket for the current family
///
/// Resolution order:
/// 1. `NEURAL_API_SOCKET` environment variable
/// 2. XDG runtime dir: `neural-api-{family_id}.sock`
/// 3. `/tmp` fallback: `neural-api-{family_id}.sock`
pub fn discover_neural_api_socket(family_id: &str) -> Option<String> {
    // 1. Explicit env var
    if let Ok(socket) = std::env::var("NEURAL_API_SOCKET") {
        if std::path::Path::new(&socket).exists() {
            return Some(socket);
        }
    }

    // 2. XDG runtime dir
    let paths = biomeos_types::paths::SystemPaths::new_lazy();
    let xdg_path = format!(
        "{}/neural-api-{}.sock",
        paths.runtime_dir().display(),
        family_id
    );
    if std::path::Path::new(&xdg_path).exists() {
        return Some(xdg_path);
    }

    // 3. System temp dir fallback (bootstrap scenarios)
    let tmp_path = std::env::temp_dir().join(format!("neural-api-{}.sock", family_id));
    if tmp_path.exists() {
        return Some(tmp_path.to_string_lossy().to_string());
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

    // Discover any primal providing birdsong.decrypt
    // Deep Debt: capability-based discovery, not name-based
    let providers = discover_beacon_providers(runtime_dir, family_id);

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
                continue;
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
        .and_then(|v| v.as_bool())
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

/// Discover beacon provider sockets in the runtime directory
///
/// Deep Debt: Scans for *any* primal socket that might provide birdsong,
/// not just hardcoded names. Falls back to known providers as a last resort.
fn discover_beacon_providers(
    runtime_dir: &std::path::Path,
    family_id: &str,
) -> Vec<std::path::PathBuf> {
    let mut providers = Vec::new();

    // Try known beacon-capable primals with family-scoped sockets
    for primal in &["beardog", "songbird"] {
        let path = runtime_dir.join(format!("{}-{}.sock", primal, family_id));
        if path.exists() {
            providers.push(path);
        }
    }

    // Also scan for any other sockets in the runtime dir that might be beacon providers
    // Deep Debt: runtime discovery, not hardcoded enumeration
    if let Ok(entries) = std::fs::read_dir(runtime_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
                // Skip already-discovered sockets and non-family sockets
                if name.ends_with(".sock")
                    && name.contains(family_id)
                    && !name.starts_with("beardog-")
                    && !name.starts_with("songbird-")
                    && !name.starts_with("neural-api-")
                {
                    providers.push(path);
                }
            }
        }
    }

    providers
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_decrypt_result_valid() {
        let value = serde_json::json!({
            "success": true,
            "plaintext": "decrypted-beacon-data",
            "family_id": "test-family"
        });
        let result = parse_decrypt_result(&value, "default");
        assert!(result.is_some());
        let r = result.unwrap();
        assert_eq!(r.family_id, "test-family");
        assert_eq!(r.plaintext, "decrypted-beacon-data");
    }

    #[test]
    fn test_parse_decrypt_result_success_false() {
        let value = serde_json::json!({
            "success": false,
            "plaintext": "some-data"
        });
        // success is false → should fail even with plaintext
        assert!(parse_decrypt_result(&value, "default").is_none());
    }

    #[test]
    fn test_parse_decrypt_result_empty_plaintext() {
        let value = serde_json::json!({
            "success": true,
            "plaintext": ""
        });
        // empty plaintext → should fail even with success
        assert!(parse_decrypt_result(&value, "default").is_none());
    }

    #[test]
    fn test_parse_decrypt_result_missing_both() {
        let value = serde_json::json!({});
        assert!(parse_decrypt_result(&value, "default").is_none());
    }

    #[test]
    fn test_parse_decrypt_result_uses_default_family() {
        let value = serde_json::json!({
            "success": true,
            "plaintext": "data"
        });
        let result = parse_decrypt_result(&value, "fallback-family");
        assert!(result.is_some());
        assert_eq!(result.unwrap().family_id, "fallback-family");
    }

    #[test]
    fn test_discover_neural_api_socket_env_override() {
        // With no real socket, should return None
        // (env var would need to point to existing file)
        let result = discover_neural_api_socket("nonexistent-family");
        // Result depends on environment, but shouldn't panic
        let _ = result;
    }

    #[test]
    fn test_beacon_verification_debug() {
        let v = BeaconVerification {
            family_id: "test".to_string(),
            plaintext: "data".to_string(),
        };
        let debug = format!("{:?}", v);
        assert!(debug.contains("test"));
    }

    #[test]
    fn test_beacon_verification_clone() {
        let v = BeaconVerification {
            family_id: "fam".to_string(),
            plaintext: "pt".to_string(),
        };
        let c = v.clone();
        assert_eq!(c.family_id, "fam");
        assert_eq!(c.plaintext, "pt");
    }
}
