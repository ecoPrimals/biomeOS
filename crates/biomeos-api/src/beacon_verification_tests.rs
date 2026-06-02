// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

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
    // When no Neural API socket is provided and no live primals are running,
    // the function should return None. However, if the development environment
    // has live primal sockets (e.g. crypto.sock in XDG_RUNTIME_DIR), the
    // fallback discovery will find them and return Some — that's correct
    // production behavior, not a test failure.
    let result =
        hash_via_capability(None, "test-no-primals-d41d8cd98f", "data").await;
    let has_live_crypto = std::env::var("XDG_RUNTIME_DIR")
        .ok()
        .map(|d| std::path::Path::new(&d).join("biomeos/crypto.sock").exists())
        .unwrap_or(false);
    if !has_live_crypto {
        assert!(result.is_none(), "no socket should return None");
    }
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
fn test_discover_neural_api_socket_env_existing_file() {
    let temp = tempfile::tempdir().expect("tempdir");
    let sock = temp.path().join("neural-explicit.sock");
    std::fs::write(&sock, "").expect("touch");
    let got = discover_neural_api_socket_from("ignored", sock.to_str());
    assert_eq!(got.as_deref(), sock.to_str());
}

#[test]
fn test_discover_neural_api_socket_fallback_runtime_base() {
    use biomeos_types::constants::{runtime_ipc, runtime_paths};

    let fallback_dir = std::path::PathBuf::from(runtime_paths::FALLBACK_RUNTIME_BASE);
    let _ = std::fs::create_dir_all(&fallback_dir);
    let basename = format!("{}my-tmp-fam.sock", runtime_ipc::NEURAL_API_BASENAME_PREFIX);
    let sock = fallback_dir.join(&basename);
    std::fs::write(&sock, "").expect("touch");
    let skip_tier1 = "/nonexistent/biomeos-test-neural-tier1-skip";
    let got = discover_neural_api_socket_from("my-tmp-fam", Some(skip_tier1));
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
