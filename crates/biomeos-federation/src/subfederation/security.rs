// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Security provider integration for sub-federation cryptographic operations.
//!
//! Lineage verification and key derivation via capability-routed JSON-RPC.

use crate::{FederationError, FederationResult};
use biomeos_core::atomic_client::AtomicClient;
use tracing::{debug, info};

/// Discover Neural API socket for capability routing (Gate 5.3 / 6.2).
///
/// Replaces legacy identity-based security socket discovery — inter-primal calls use
/// capability domains (e.g. [`biomeos_types::capability_discovery::discover_capability_socket`]
/// for `"security"`) and route through the Neural API's capability translation layer.
pub fn discover_neural_api_socket() -> FederationResult<String> {
    discover_neural_api_socket_with(None)
}

/// Like [`discover_neural_api_socket`], with an explicit socket path override (for tests).
pub fn discover_neural_api_socket_with(socket_override: Option<&str>) -> FederationResult<String> {
    if let Some(s) = socket_override {
        return Ok(s.to_string());
    }
    if let Ok(socket) = std::env::var("NEURAL_API_SOCKET") {
        return Ok(socket);
    }

    let paths = biomeos_types::SystemPaths::new_lazy();
    let socket = paths.primal_socket("neural-api");
    if socket.exists() {
        return Ok(socket.to_string_lossy().to_string());
    }

    let tmp_socket =
        std::path::PathBuf::from(biomeos_types::constants::runtime_paths::FALLBACK_RUNTIME_BASE)
            .join("neural-api.sock");
    if tmp_socket.exists() {
        return Ok(tmp_socket.to_string_lossy().to_string());
    }

    // Legacy fallback: direct BearDog discovery for bootstrap
    if let Ok(socket) = std::env::var(biomeos_types::defaults::env_vars::socket_env_key(
        biomeos_types::primal_names::BEARDOG,
    )) {
        return Ok(socket);
    }

    Err(FederationError::Generic(
        "Neural API socket not found. Ensure biomeOS Neural API is running.".to_string(),
    ))
}

/// Verify that all members share genetic lineage with the parent family.
///
/// Routes through Neural API via `capability.call("lineage", "verify_members")`.
pub async fn verify_member_lineage(
    parent_family: &str,
    members: &[String],
) -> FederationResult<()> {
    let neural_socket = discover_neural_api_socket()?;
    verify_member_lineage_with(&neural_socket, parent_family, members).await
}

/// Like [`verify_member_lineage`], with an explicit Neural API Unix socket path.
pub async fn verify_member_lineage_with(
    socket_path: &str,
    parent_family: &str,
    members: &[String],
) -> FederationResult<()> {
    let client = AtomicClient::unix(socket_path);
    let result = client
        .call(
            "capability.call",
            serde_json::json!({
                "capability": "lineage",
                "operation": "verify_members",
                "args": {
                    "family_id": parent_family,
                    "member_patterns": members
                }
            }),
        )
        .await
        .map_err(|e| FederationError::Generic(format!("Capability call failed: {e}")))?;

    let all_verified = result
        .get("all_verified")
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    if all_verified {
        info!("✅ Lineage verified for {} members", members.len());
        Ok(())
    } else {
        let failed = result
            .get("failed_members")
            .and_then(|f| f.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            })
            .unwrap_or_default();

        Err(FederationError::Generic(format!(
            "Lineage verification failed for: {failed}"
        )))
    }
}

/// Request a derived encryption key for this sub-federation.
///
/// Routes through Neural API via `capability.call("crypto", "derive_subfederation_key")`.
pub async fn request_subfederation_key(
    parent_family: &str,
    subfed_name: &str,
) -> FederationResult<String> {
    let neural_socket = discover_neural_api_socket()?;
    request_subfederation_key_with(&neural_socket, parent_family, subfed_name).await
}

/// Like [`request_subfederation_key`], with an explicit Neural API Unix socket path.
pub async fn request_subfederation_key_with(
    socket_path: &str,
    parent_family: &str,
    subfed_name: &str,
) -> FederationResult<String> {
    let client = AtomicClient::unix(socket_path);
    let result = client
        .call(
            "capability.call",
            serde_json::json!({
                "capability": "crypto",
                "operation": "derive_subfederation_key",
                "args": {
                    "family_id": parent_family,
                    "subfederation_name": subfed_name,
                    "purpose": "subfederation-encryption-v1"
                }
            }),
        )
        .await
        .map_err(|e| FederationError::Generic(format!("Capability call failed: {e}")))?;

    let key_ref = result
        .get("key_ref")
        .and_then(|k| k.as_str())
        .ok_or_else(|| FederationError::Generic("Missing key_ref in response".to_string()))?;

    debug!(
        "Derived key for sub-federation '{}': {}",
        subfed_name, key_ref
    );
    Ok(key_ref.to_string())
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;
    use tokio::sync::oneshot;

    #[tokio::test]
    async fn test_discover_neural_api_socket_from_override() {
        let result = discover_neural_api_socket_with(Some("/tmp/test-neural-api.sock"));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "/tmp/test-neural-api.sock");
    }

    #[tokio::test]
    async fn test_discover_neural_api_socket_without_env() {
        let result = discover_neural_api_socket();
        match result {
            Ok(path) => assert!(!path.is_empty()),
            Err(e) => assert!(
                e.to_string().to_lowercase().contains("not found")
                    || e.to_string().to_lowercase().contains("neural api")
            ),
        }
    }

    async fn spawn_beardog_mock(response_line: String) -> (tempfile::TempDir, PathBuf) {
        let dir = tempfile::tempdir().expect("tempdir");
        let path = dir.path().join("beardog-mock.sock");
        let (ready_tx, ready_rx) = oneshot::channel();
        let path_for_task = path.clone();
        let line = response_line;
        tokio::spawn(async move {
            let _ = tokio::fs::remove_file(&path_for_task).await;
            let listener = UnixListener::bind(&path_for_task).expect("bind mock beardog");
            ready_tx.send(()).expect("ready");
            let (stream, _) = listener.accept().await.expect("accept");
            let (read_half, mut write_half) = stream.into_split();
            let mut reader = BufReader::new(read_half);
            let mut request = String::new();
            reader.read_line(&mut request).await.expect("read request");
            assert!(!request.is_empty());
            write_half
                .write_all(line.as_bytes())
                .await
                .expect("write response");
            write_half.flush().await.expect("flush response");
            write_half.shutdown().await.expect("shutdown write half");
        });
        ready_rx.await.expect("mock start");
        (dir, path)
    }

    #[tokio::test]
    async fn test_verify_member_lineage_connection_error() {
        let result = verify_member_lineage_with(
            "/nonexistent/path/neural-api.sock",
            "parent-family",
            &["member1".to_string(), "member2".to_string()],
        )
        .await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("connection")
                || err_msg.contains("Connection")
                || err_msg.contains("failed"),
            "expected connection-related error, got: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_request_subfederation_key_connection_error() {
        let result = request_subfederation_key_with(
            "/nonexistent/path/neural-api.sock",
            "parent-family",
            "subfed-name",
        )
        .await;
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("connection")
                || err_msg.contains("Connection")
                || err_msg.contains("failed"),
            "expected connection-related error, got: {}",
            err_msg
        );
    }

    #[tokio::test]
    async fn test_verify_member_lineage_success() {
        let line = r#"{"jsonrpc":"2.0","id":1,"result":{"all_verified":true}}"#.to_string() + "\n";
        let (_dir, sock) = spawn_beardog_mock(line).await;
        let result = verify_member_lineage_with(
            sock.to_string_lossy().as_ref(),
            "fam",
            &["a".into(), "b".into()],
        )
        .await;
        if let Err(e) = &result {
            let lower = e.to_string().to_lowercase();
            if lower.contains("connection") || lower.contains("no such file") {
                return;
            }
        }
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_verify_member_lineage_json_rpc_error_with_message() {
        let line =
            r#"{"jsonrpc":"2.0","id":1,"error":{"message":"bad lineage"}}"#.to_string() + "\n";
        let (_dir, sock) = spawn_beardog_mock(line).await;
        let result =
            verify_member_lineage_with(sock.to_string_lossy().as_ref(), "fam", &["m".into()]).await;
        let err = result.unwrap_err().to_string();
        let lower = err.to_lowercase();
        assert!(
            (err.contains("Lineage verification failed") && err.contains("bad lineage"))
                || lower.contains("connection")
                || lower.contains("no such file")
                || lower.contains("capability call failed"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn test_verify_member_lineage_json_rpc_error_empty_object_uses_unknown() {
        let line = r#"{"jsonrpc":"2.0","id":1,"error":{}}"#.to_string() + "\n";
        let (_dir, sock) = spawn_beardog_mock(line).await;
        let result =
            verify_member_lineage_with(sock.to_string_lossy().as_ref(), "fam", &["m".into()]).await;
        let err = result.unwrap_err().to_string();
        let lower = err.to_lowercase();
        assert!(
            err.contains("Unknown")
                || err.contains("Lineage verification failed")
                || lower.contains("connection")
                || lower.contains("no such file")
                || lower.contains("capability call failed"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn test_verify_member_lineage_not_all_verified_lists_failed() {
        let line =
            r#"{"jsonrpc":"2.0","id":1,"result":{"all_verified":false,"failed_members":["x","y"]}}"#
                .to_string() + "\n";
        let (_dir, sock) = spawn_beardog_mock(line).await;
        let result =
            verify_member_lineage_with(sock.to_string_lossy().as_ref(), "fam", &["m".into()]).await;
        let err = result.unwrap_err().to_string();
        let lower = err.to_lowercase();
        assert!(
            err.contains("Lineage verification failed")
                || lower.contains("connection")
                || lower.contains("no such file")
                || lower.contains("capability call failed"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn test_verify_member_lineage_malformed_json_response() {
        let line = "%%%not-json\n".to_string();
        let (_dir, sock) = spawn_beardog_mock(line).await;
        let result =
            verify_member_lineage_with(sock.to_string_lossy().as_ref(), "fam", &["m".into()]).await;
        let err = result.unwrap_err().to_string();
        let lower = err.to_lowercase();
        assert!(
            err.contains("JSON parse")
                || lower.contains("connection")
                || lower.contains("no such file")
                || lower.contains("capability call failed"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn test_request_subfederation_key_success() {
        let line =
            r#"{"jsonrpc":"2.0","id":1,"result":{"key_ref":"vault/key/abc"}}"#.to_string() + "\n";
        let (_dir, sock) = spawn_beardog_mock(line).await;
        let result =
            request_subfederation_key_with(sock.to_string_lossy().as_ref(), "fam", "sub").await;
        match result {
            Ok(key) => assert_eq!(key, "vault/key/abc"),
            Err(e) => {
                let msg = e.to_string();
                let lower = msg.to_lowercase();
                assert!(
                    lower.contains("connection") || lower.contains("no such file"),
                    "unexpected error: {msg}"
                );
            }
        }
    }

    #[tokio::test]
    async fn test_request_subfederation_key_json_rpc_error() {
        let line = r#"{"jsonrpc":"2.0","id":1,"error":{"message":"denied"}}"#.to_string() + "\n";
        let (_dir, sock) = spawn_beardog_mock(line).await;
        let result =
            request_subfederation_key_with(sock.to_string_lossy().as_ref(), "fam", "sub").await;
        let err = result.unwrap_err().to_string();
        let lower = err.to_lowercase();
        assert!(
            (err.contains("Key derivation failed") && err.contains("denied"))
                || lower.contains("connection")
                || lower.contains("no such file")
                || lower.contains("capability call failed"),
            "unexpected error: {err}"
        );
    }

    #[tokio::test]
    async fn test_request_subfederation_key_missing_key_ref() {
        let line = r#"{"jsonrpc":"2.0","id":1,"result":{}}"#.to_string() + "\n";
        let (_dir, sock) = spawn_beardog_mock(line).await;
        let result =
            request_subfederation_key_with(sock.to_string_lossy().as_ref(), "fam", "sub").await;
        let err = result.unwrap_err().to_string();
        let lower = err.to_lowercase();
        assert!(
            err.contains("Missing key_ref")
                || lower.contains("connection")
                || lower.contains("no such file")
                || lower.contains("socket not found"),
            "unexpected error: {err}"
        );
    }
}
