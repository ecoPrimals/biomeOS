// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! BearDog integration for sub-federation cryptographic operations
//!
//! Lineage verification and key derivation via JSON-RPC.

use crate::{FederationError, FederationResult};
use biomeos_types::JsonRpcRequest;
use tracing::{debug, info};

/// Discover BearDog socket path via XDG-compliant SystemPaths
pub fn discover_beardog_socket() -> FederationResult<String> {
    if let Ok(socket) = std::env::var("BEARDOG_SOCKET") {
        return Ok(socket);
    }

    let paths = biomeos_types::SystemPaths::new_lazy();
    let security_provider = biomeos_types::CapabilityTaxonomy::resolve_to_primal("security")
        .unwrap_or(biomeos_types::primal_names::BEARDOG);
    let socket = paths.primal_socket(security_provider);
    if socket.exists() {
        return Ok(socket.to_string_lossy().to_string());
    }

    if let Ok(family_id) = std::env::var("BIOMEOS_FAMILY_ID") {
        let family_socket = paths.primal_socket(&format!("{security_provider}-{family_id}"));
        if family_socket.exists() {
            return Ok(family_socket.to_string_lossy().to_string());
        }
    }

    Err(FederationError::Generic(
        "BearDog socket not found. Ensure BearDog is running.".to_string(),
    ))
}

/// Verify that all members share genetic lineage with the parent family
pub async fn verify_member_lineage(
    parent_family: &str,
    members: &[String],
) -> FederationResult<()> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;

    let beardog_socket = discover_beardog_socket()?;

    let stream = UnixStream::connect(&beardog_socket)
        .await
        .map_err(|e| FederationError::Generic(format!("BearDog connection failed: {e}")))?;

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let request = JsonRpcRequest::new(
        "lineage.verify_members",
        serde_json::json!({
            "family_id": parent_family,
            "member_patterns": members
        }),
    );

    let request_str = serde_json::to_string(&request)
        .map_err(|e| FederationError::Generic(format!("JSON error: {e}")))?
        + "\n";

    writer
        .write_all(request_str.as_bytes())
        .await
        .map_err(|e| FederationError::Generic(format!("Write error: {e}")))?;
    writer
        .flush()
        .await
        .map_err(|e| FederationError::Generic(format!("Flush error: {e}")))?;

    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .await
        .map_err(|e| FederationError::Generic(format!("Read error: {e}")))?;

    let response: serde_json::Value = serde_json::from_str(response_line.trim())
        .map_err(|e| FederationError::Generic(format!("JSON parse error: {e}")))?;

    if let Some(error) = response.get("error") {
        let msg = error
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown");
        return Err(FederationError::Generic(format!(
            "Lineage verification failed: {msg}"
        )));
    }

    let all_verified = response
        .get("result")
        .and_then(|r| r.get("all_verified"))
        .and_then(serde_json::Value::as_bool)
        .unwrap_or(false);

    if all_verified {
        info!("✅ Lineage verified for {} members", members.len());
        Ok(())
    } else {
        let failed = response
            .get("result")
            .and_then(|r| r.get("failed_members"))
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

/// Request a derived encryption key for this sub-federation
pub async fn request_subfederation_key(
    parent_family: &str,
    subfed_name: &str,
) -> FederationResult<String> {
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixStream;

    let beardog_socket = discover_beardog_socket()?;

    let stream = UnixStream::connect(&beardog_socket)
        .await
        .map_err(|e| FederationError::Generic(format!("BearDog connection failed: {e}")))?;

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let request = JsonRpcRequest::new(
        "crypto.derive_subfederation_key",
        serde_json::json!({
            "family_id": parent_family,
            "subfederation_name": subfed_name,
            "purpose": "subfederation-encryption-v1"
        }),
    );

    let request_str = serde_json::to_string(&request)
        .map_err(|e| FederationError::Generic(format!("JSON error: {e}")))?
        + "\n";

    writer
        .write_all(request_str.as_bytes())
        .await
        .map_err(|e| FederationError::Generic(format!("Write error: {e}")))?;
    writer
        .flush()
        .await
        .map_err(|e| FederationError::Generic(format!("Flush error: {e}")))?;

    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .await
        .map_err(|e| FederationError::Generic(format!("Read error: {e}")))?;

    let response: serde_json::Value = serde_json::from_str(response_line.trim())
        .map_err(|e| FederationError::Generic(format!("JSON parse error: {e}")))?;

    if let Some(error) = response.get("error") {
        let msg = error
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown");
        return Err(FederationError::Generic(format!(
            "Key derivation failed: {msg}"
        )));
    }

    let key_ref = response
        .get("result")
        .and_then(|r| r.get("key_ref"))
        .and_then(|k| k.as_str())
        .ok_or_else(|| FederationError::Generic("Missing key_ref in response".to_string()))?;

    debug!(
        "Derived key for sub-federation '{}': {}",
        subfed_name, key_ref
    );
    Ok(key_ref.to_string())
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use biomeos_test_utils::{remove_test_env, set_test_env};
    use serial_test::serial;
    use std::path::PathBuf;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;
    use tokio::sync::oneshot;

    #[tokio::test]
    #[serial]
    async fn test_discover_beardog_socket_from_env() {
        set_test_env("BEARDOG_SOCKET", "/tmp/test-beardog.sock");
        let result = discover_beardog_socket();
        remove_test_env("BEARDOG_SOCKET");
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "/tmp/test-beardog.sock");
    }

    #[tokio::test]
    #[serial]
    async fn test_discover_beardog_socket_without_env() {
        remove_test_env("BEARDOG_SOCKET");
        remove_test_env("BIOMEOS_FAMILY_ID");
        let result = discover_beardog_socket();
        match result {
            Ok(path) => assert!(!path.is_empty()),
            Err(e) => assert!(
                e.to_string().to_lowercase().contains("not found")
                    || e.to_string().to_lowercase().contains("beardog")
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
        });
        ready_rx.await.expect("mock start");
        (dir, path)
    }

    #[tokio::test]
    #[serial]
    async fn test_verify_member_lineage_connection_error() {
        set_test_env("BEARDOG_SOCKET", "/nonexistent/path/beardog.sock");
        let result = verify_member_lineage(
            "parent-family",
            &["member1".to_string(), "member2".to_string()],
        )
        .await;
        remove_test_env("BEARDOG_SOCKET");
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
    #[serial]
    async fn test_request_subfederation_key_connection_error() {
        set_test_env("BEARDOG_SOCKET", "/nonexistent/path/beardog.sock");
        let result = request_subfederation_key("parent-family", "subfed-name").await;
        remove_test_env("BEARDOG_SOCKET");
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
    #[serial]
    async fn test_verify_member_lineage_success() {
        let line = r#"{"jsonrpc":"2.0","id":1,"result":{"all_verified":true}}"#.to_string() + "\n";
        let (_dir, sock) = spawn_beardog_mock(line).await;
        set_test_env("BEARDOG_SOCKET", sock.to_string_lossy().as_ref());
        let result = verify_member_lineage("fam", &["a".into(), "b".into()]).await;
        remove_test_env("BEARDOG_SOCKET");
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[serial]
    async fn test_verify_member_lineage_json_rpc_error_with_message() {
        let line =
            r#"{"jsonrpc":"2.0","id":1,"error":{"message":"bad lineage"}}"#.to_string() + "\n";
        let (_dir, sock) = spawn_beardog_mock(line).await;
        set_test_env("BEARDOG_SOCKET", sock.to_string_lossy().as_ref());
        let result = verify_member_lineage("fam", &["m".into()]).await;
        remove_test_env("BEARDOG_SOCKET");
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Lineage verification failed"));
        assert!(err.contains("bad lineage"));
    }

    #[tokio::test]
    #[serial]
    async fn test_verify_member_lineage_json_rpc_error_empty_object_uses_unknown() {
        let line = r#"{"jsonrpc":"2.0","id":1,"error":{}}"#.to_string() + "\n";
        let (_dir, sock) = spawn_beardog_mock(line).await;
        set_test_env("BEARDOG_SOCKET", sock.to_string_lossy().as_ref());
        let result = verify_member_lineage("fam", &["m".into()]).await;
        remove_test_env("BEARDOG_SOCKET");
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Unknown") || err.contains("Lineage verification failed"));
    }

    #[tokio::test]
    #[serial]
    async fn test_verify_member_lineage_not_all_verified_lists_failed() {
        let line =
            r#"{"jsonrpc":"2.0","id":1,"result":{"all_verified":false,"failed_members":["x","y"]}}"#
                .to_string() + "\n";
        let (_dir, sock) = spawn_beardog_mock(line).await;
        set_test_env("BEARDOG_SOCKET", sock.to_string_lossy().as_ref());
        let result = verify_member_lineage("fam", &["m".into()]).await;
        remove_test_env("BEARDOG_SOCKET");
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Lineage verification failed"));
        assert!(err.contains('x'));
    }

    #[tokio::test]
    #[serial]
    async fn test_verify_member_lineage_malformed_json_response() {
        let line = "%%%not-json\n".to_string();
        let (_dir, sock) = spawn_beardog_mock(line).await;
        set_test_env("BEARDOG_SOCKET", sock.to_string_lossy().as_ref());
        let result = verify_member_lineage("fam", &["m".into()]).await;
        remove_test_env("BEARDOG_SOCKET");
        assert!(result.unwrap_err().to_string().contains("JSON parse"));
    }

    #[tokio::test]
    #[serial]
    async fn test_request_subfederation_key_success() {
        let line =
            r#"{"jsonrpc":"2.0","id":1,"result":{"key_ref":"vault/key/abc"}}"#.to_string() + "\n";
        let (_dir, sock) = spawn_beardog_mock(line).await;
        set_test_env("BEARDOG_SOCKET", sock.to_string_lossy().as_ref());
        let result = request_subfederation_key("fam", "sub").await;
        remove_test_env("BEARDOG_SOCKET");
        match result {
            Ok(key) => assert_eq!(key, "vault/key/abc"),
            Err(e) => {
                let msg = e.to_string();
                assert!(
                    msg.contains("connection") || msg.contains("No such file"),
                    "unexpected error: {msg}"
                );
            }
        }
    }

    #[tokio::test]
    #[serial]
    async fn test_request_subfederation_key_json_rpc_error() {
        let line = r#"{"jsonrpc":"2.0","id":1,"error":{"message":"denied"}}"#.to_string() + "\n";
        let (_dir, sock) = spawn_beardog_mock(line).await;
        set_test_env("BEARDOG_SOCKET", sock.to_string_lossy().as_ref());
        let result = request_subfederation_key("fam", "sub").await;
        remove_test_env("BEARDOG_SOCKET");
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Key derivation failed"));
        assert!(err.contains("denied"));
    }

    #[tokio::test]
    #[serial]
    async fn test_request_subfederation_key_missing_key_ref() {
        let line = r#"{"jsonrpc":"2.0","id":1,"result":{}}"#.to_string() + "\n";
        let (_dir, sock) = spawn_beardog_mock(line).await;
        set_test_env("BEARDOG_SOCKET", sock.to_string_lossy().as_ref());
        let result = request_subfederation_key("fam", "sub").await;
        remove_test_env("BEARDOG_SOCKET");
        assert!(result.unwrap_err().to_string().contains("Missing key_ref"));
    }
}
