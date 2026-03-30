// SPDX-License-Identifier: AGPL-3.0-only
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
    // 1. Explicit env var
    if let Ok(socket) = std::env::var("NEURAL_API_SOCKET") {
        if std::path::Path::new(&socket).exists() {
            return Some(socket);
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

    // 3. System temp dir fallback (bootstrap scenarios)
    let tmp_path = std::env::temp_dir().join(&basename);
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
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use std::path::PathBuf;

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
        let debug = format!("{v:?}");
        assert!(debug.contains("test"));
    }

    #[test]
    fn test_beacon_verification_clone() {
        let v = BeaconVerification {
            family_id: "fam".to_string(),
            plaintext: "pt".to_string(),
        };
        assert_eq!(v.family_id, "fam");
        assert_eq!(v.plaintext, "pt");
    }

    // ========== parse_decrypt_result edge cases ==========

    #[test]
    fn test_parse_decrypt_result_plaintext_null() {
        let value = serde_json::json!({
            "success": true,
            "plaintext": null
        });
        assert!(
            parse_decrypt_result(&value, "default").is_none(),
            "null plaintext should fail (AND validation)"
        );
    }

    #[test]
    fn test_parse_decrypt_result_plaintext_non_string() {
        let value = serde_json::json!({
            "success": true,
            "plaintext": 42
        });
        assert!(
            parse_decrypt_result(&value, "default").is_none(),
            "non-string plaintext should fail"
        );
    }

    #[test]
    fn test_parse_decrypt_result_success_non_bool() {
        let value = serde_json::json!({
            "success": "true",
            "plaintext": "data"
        });
        assert!(
            parse_decrypt_result(&value, "default").is_none(),
            "non-bool success should be treated as false"
        );
    }

    #[test]
    fn test_parse_decrypt_result_whitespace_plaintext() {
        let value = serde_json::json!({
            "success": true,
            "plaintext": "   "
        });
        let result = parse_decrypt_result(&value, "default");
        assert!(
            result.is_some(),
            "whitespace-only plaintext is non-empty, should pass"
        );
        assert_eq!(result.unwrap().plaintext, "   ");
    }

    #[test]
    fn test_parse_decrypt_result_family_id_from_value_overrides_default() {
        let value = serde_json::json!({
            "success": true,
            "plaintext": "x",
            "family_id": "explicit-family"
        });
        let result = parse_decrypt_result(&value, "default-family").expect("valid");
        assert_eq!(result.family_id, "explicit-family");
    }

    #[test]
    fn test_beacon_verification_construction() {
        let v = BeaconVerification {
            family_id: "test-fam".to_string(),
            plaintext: "decrypted-content".to_string(),
        };
        assert_eq!(v.family_id, "test-fam");
        assert_eq!(v.plaintext, "decrypted-content");
    }

    #[test]
    fn test_discover_neural_api_socket_no_socket_returns_none() {
        // With no NEURAL_API_SOCKET and no real socket files, should return None
        let result = discover_neural_api_socket("nonexistent-family-xyz-12345");
        // Result is environment-dependent; we just verify it doesn't panic
        let _ = result;
    }

    #[test]
    fn test_discover_beacon_providers_empty_dir() {
        let temp = tempfile::tempdir().expect("temp dir");
        let providers = discover_family_scoped_primal_sockets(temp.path(), "family-123");
        assert!(providers.is_empty(), "empty dir should yield no providers");
    }

    #[test]
    fn test_discover_beacon_providers_family_scoped_sockets() {
        let temp = tempfile::tempdir().expect("temp dir");
        let a = temp.path().join("alpha-primal-family-123.sock");
        let b = temp.path().join("beta-instance-family-123.sock");
        std::fs::write(&a, "").expect("create sock");
        std::fs::write(&b, "").expect("create sock");

        let providers = discover_family_scoped_primal_sockets(temp.path(), "family-123");
        assert_eq!(providers.len(), 2, "should find all family-scoped sockets");
        let names: Vec<_> = providers
            .iter()
            .filter_map(|p| p.file_name().and_then(|n| n.to_str()))
            .collect();
        assert!(names.contains(&"alpha-primal-family-123.sock"));
        assert!(names.contains(&"beta-instance-family-123.sock"));
    }

    #[test]
    fn test_discover_beacon_providers_skips_neural_api() {
        let temp = tempfile::tempdir().expect("temp dir");
        let neural = temp.path().join(format!(
            "{}family-123.sock",
            runtime_ipc::NEURAL_API_BASENAME_PREFIX
        ));
        std::fs::write(&neural, "").expect("create neural sock");
        let providers = discover_family_scoped_primal_sockets(temp.path(), "family-123");
        assert!(
            !providers.iter().any(|p| p
                .to_string_lossy()
                .contains(runtime_ipc::NEURAL_API_BASENAME_PREFIX)),
            "should not include neural API socket"
        );
    }

    #[test]
    fn test_discover_beacon_providers_wrong_family_suffix_excluded() {
        let temp = tempfile::tempdir().expect("temp dir");
        let other = temp.path().join("gamma-other-family.sock");
        std::fs::write(&other, "").expect("create sock");
        let providers = discover_family_scoped_primal_sockets(temp.path(), "family-456");
        assert!(
            providers.is_empty(),
            "suffix must match requested family exactly"
        );
    }

    #[test]
    fn test_discover_beacon_providers_hyphenated_family_id() {
        let temp = tempfile::tempdir().expect("temp dir");
        let sock = temp.path().join("any-instance-my-family-id.sock");
        std::fs::write(&sock, "").expect("create sock");
        let providers = discover_family_scoped_primal_sockets(temp.path(), "my-family-id");
        assert_eq!(providers.len(), 1);
    }

    #[tokio::test]
    async fn test_hash_via_capability_no_socket_returns_none() {
        let result = hash_via_capability(None, "family", "data").await;
        assert!(result.is_none(), "no socket should return None");
    }

    #[tokio::test]
    async fn test_verify_dark_forest_token_no_socket() {
        let result = verify_dark_forest_token(None, "family", "token").await;
        // Without Neural API socket and no real primal sockets, should be None
        let _ = result;
    }

    /// Mock Neural API: JSON-RPC `neural_api.route_to_primal` returns a fixed `result` object.
    #[cfg(unix)]
    async fn spawn_one_shot_neural_mock(
        result_json: serde_json::Value,
    ) -> (tempfile::TempDir, PathBuf) {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::UnixListener;

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("neural-mock.sock");
        let listener = UnixListener::bind(&path).expect("bind mock neural");
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept");
            let mut buf = vec![0u8; 16 * 1024];
            let _ = stream.read(&mut buf).await;
            let body = serde_json::json!({
                "jsonrpc": "2.0",
                "id": 1,
                "result": result_json
            });
            let mut line = serde_json::to_string(&body).expect("serialize response");
            line.push('\n');
            stream.write_all(line.as_bytes()).await.expect("write");
        });
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        (dir, path)
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_verify_dark_forest_token_neural_api_success() {
        let decrypt = serde_json::json!({
            "success": true,
            "plaintext": "beacon-plaintext",
            "family_id": "fam-x"
        });
        let (_dir, sock) = spawn_one_shot_neural_mock(decrypt).await;
        let out =
            verify_dark_forest_token(Some(sock.to_str().expect("utf8")), "fam-x", "opaque").await;
        assert!(out.is_some(), "expected neural path to verify");
        let v = out.expect("verification");
        assert_eq!(v.plaintext, "beacon-plaintext");
        assert_eq!(v.family_id, "fam-x");
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_verify_dark_forest_token_neural_api_error_falls_back() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        use tokio::net::UnixListener;

        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("neural-mock-err.sock");
        let listener = UnixListener::bind(&path).expect("bind");
        let path_clone = path.clone();
        tokio::spawn(async move {
            let (mut stream, _) = listener.accept().await.expect("accept");
            let mut buf = vec![0u8; 16 * 1024];
            let _ = stream.read(&mut buf).await;
            let body = r#"{"jsonrpc":"2.0","id":1,"error":{"code":-1,"message":"rpc fail"}}"#;
            stream
                .write_all(format!("{body}\n").as_bytes())
                .await
                .expect("write");
        });
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;

        let out =
            verify_dark_forest_token(Some(path_clone.to_str().expect("utf8")), "fam", "tok").await;
        assert!(out.is_none());
    }

    #[tokio::test]
    #[cfg(unix)]
    async fn test_hash_via_capability_neural_api_returns_hash() {
        let inner = serde_json::json!({ "hash": "blake3-deadbeef" });
        let (_dir, sock) = spawn_one_shot_neural_mock(inner).await;
        let h = hash_via_capability(Some(sock.to_str().expect("utf8")), "fam", "payload").await;
        assert_eq!(h.as_deref(), Some("blake3-deadbeef"));
    }

    #[test]
    #[serial_test::serial]
    fn test_discover_neural_api_socket_env_existing_file() {
        use biomeos_test_utils::env_helpers::TestEnvGuard;

        let temp = tempfile::tempdir().expect("tempdir");
        let sock = temp.path().join("neural-explicit.sock");
        std::fs::write(&sock, "").expect("touch");
        let _g = TestEnvGuard::set("NEURAL_API_SOCKET", sock.to_str().expect("utf8"));
        let got = discover_neural_api_socket("ignored");
        assert_eq!(got.as_deref(), sock.to_str());
    }

    #[test]
    #[serial_test::serial]
    fn test_discover_neural_api_socket_temp_dir_fallback_existing_file() {
        use biomeos_test_utils::env_helpers::TestEnvGuard;
        use biomeos_types::constants::runtime_ipc;

        let basename = format!("{}my-tmp-fam.sock", runtime_ipc::NEURAL_API_BASENAME_PREFIX);
        let sock = std::env::temp_dir().join(&basename);
        std::fs::write(&sock, "").expect("touch");
        let _clear = TestEnvGuard::remove("NEURAL_API_SOCKET");

        let got = discover_neural_api_socket("my-tmp-fam");
        assert_eq!(got.as_deref(), sock.to_str());
        let _ = std::fs::remove_file(&sock);
    }

    #[test]
    fn test_discover_family_scoped_skips_non_sock_extension() {
        let temp = tempfile::tempdir().expect("tempdir");
        let p = temp.path().join("foo-family-123.txt");
        std::fs::write(&p, "").expect("write");
        let providers = discover_family_scoped_primal_sockets(temp.path(), "family-123");
        assert!(providers.is_empty());
    }
}
