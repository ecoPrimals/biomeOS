// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Songbird discovery and `discovery.register_capability` registration for device management.

#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use serde_json::{Value, json};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::time::timeout;
use tracing::info;

/// Register this server with Songbird for capability advertisement
///
/// This enables other primals and nodes to discover this device management
/// server via Songbird's UDP multicast discovery.
pub(super) async fn register_with_songbird(socket_path: &str) -> Result<()> {
    let songbird_socket = discover_songbird_socket()?;
    register_with_songbird_at(&songbird_socket, socket_path).await
}

/// Same as [`register_with_songbird`], but connects to an explicit Songbird Unix socket
/// (for tests and callers that inject discovery without process environment).
pub(super) async fn register_with_songbird_at(
    songbird_socket: &str,
    device_management_socket_path: &str,
) -> Result<()> {
    info!("📡 Registering with Songbird at: {}", songbird_socket);

    let stream = tokio::net::UnixStream::connect(songbird_socket)
        .await
        .context("Failed to connect to Songbird")?;

    let (reader, mut writer) = stream.into_split();
    let mut reader = BufReader::new(reader);

    let request = json!({
        "jsonrpc": "2.0",
        "method": "discovery.register_capability",
            "params": {
                "capability": "device.management",
                "endpoint": {
                    "type": "unix_socket",
                    "path": device_management_socket_path
                },
            "metadata": {
                "version": env!("CARGO_PKG_VERSION"),
                "description": "Device management and primal orchestration"
            }
        },
        "id": 1
    });

    let request_str = serde_json::to_string(&request)? + "\n";
    writer.write_all(request_str.as_bytes()).await?;
    writer.flush().await?;

    let mut response_line = String::new();
    timeout(
        Duration::from_secs(30),
        reader.read_line(&mut response_line),
    )
    .await
    .context("Songbird registration timeout (30s)")?
    .context("Failed to read Songbird response")?;

    let response: Value = serde_json::from_str(response_line.trim())?;

    if let Some(err) = response.get("error") {
        let msg = err
            .get("message")
            .and_then(|m| m.as_str())
            .unwrap_or("Unknown error");
        anyhow::bail!("Songbird registration failed: {msg}");
    }

    info!("✅ Registered with Songbird for UDP multicast discovery");
    Ok(())
}

/// Discover discovery provider socket via the 5-tier capability protocol.
///
/// Delegates to [`biomeos_types::capability_discovery::discover_capability_socket`].
pub(super) fn discover_songbird_socket() -> Result<String> {
    use biomeos_types::capability_discovery;
    discover_songbird_socket_with(&capability_discovery::std_env)
}

/// Same as [`discover_songbird_socket`], using an injectable environment lookup
/// (`std::env` in production; a `HashMap` closure in tests).
pub(super) fn discover_songbird_socket_with(
    env: &dyn Fn(&str) -> Option<String>,
) -> Result<String> {
    use biomeos_types::capability_discovery;
    capability_discovery::discover_capability_socket("discovery", env)
        .context("Discovery provider socket not found. Is Songbird running?")
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use std::collections::HashMap;
    use std::sync::Mutex;
    use tokio::io::AsyncBufReadExt;

    /// Serialize access to fixed `/tmp/biomeos` (empty family tier-4 path) across parallel tests.
    static TMP_BIOMEOS_EMPTY_FAMILY_LOCK: Mutex<()> = Mutex::new(());

    fn mock_env(vars: &HashMap<String, String>) -> impl Fn(&str) -> Option<String> + '_ {
        move |key: &str| vars.get(key).cloned()
    }

    #[test]
    fn test_discover_songbird_socket_env_override() {
        let mut env = HashMap::new();
        env.insert(
            "SONGBIRD_SOCKET".to_string(),
            "/custom/songbird.sock".to_string(),
        );
        let result = discover_songbird_socket_with(&mock_env(&env));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "/custom/songbird.sock");
    }

    #[test]
    fn test_discover_songbird_socket_not_found() {
        let mut env = HashMap::new();
        env.insert(
            "XDG_RUNTIME_DIR".to_string(),
            "/nonexistent_xdg_path_for_test".to_string(),
        );
        let result = discover_songbird_socket_with(&mock_env(&env));
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    fn test_discover_songbird_socket_xdg_runtime_exists() {
        let temp = tempfile::tempdir().unwrap();
        let socket_path = temp.path().join("biomeos").join("songbird.sock");
        std::fs::create_dir_all(socket_path.parent().unwrap()).unwrap();
        std::fs::File::create(&socket_path).unwrap();

        let mut env = HashMap::new();
        env.insert(
            "XDG_RUNTIME_DIR".to_string(),
            temp.path().to_string_lossy().into_owned(),
        );

        let result = discover_songbird_socket_with(&mock_env(&env));
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            format!("{}/biomeos/songbird.sock", temp.path().display())
        );
    }

    #[test]
    fn test_discover_songbird_socket_family_xdg_path() {
        let temp = tempfile::tempdir().unwrap();
        let socket_path = temp.path().join("biomeos").join("songbird-family99.sock");
        std::fs::create_dir_all(socket_path.parent().unwrap()).unwrap();
        std::fs::File::create(&socket_path).unwrap();

        let mut env = HashMap::new();
        env.insert(
            "XDG_RUNTIME_DIR".to_string(),
            temp.path().to_string_lossy().into_owned(),
        );
        env.insert("BIOMEOS_FAMILY_ID".to_string(), "family99".to_string());

        let result = discover_songbird_socket_with(&mock_env(&env));
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            format!("{}/biomeos/songbird-family99.sock", temp.path().display())
        );
    }

    #[test]
    fn test_discover_songbird_socket_family_tmp_biomeos_dir() {
        let fam = format!("testlegacy{}", std::process::id());
        let biomeos_dir = format!("/tmp/biomeos-{fam}");
        let legacy_socket = format!("{biomeos_dir}/songbird.sock");
        std::fs::create_dir_all(&biomeos_dir).unwrap();
        std::fs::File::create(&legacy_socket).unwrap();

        let mut env = HashMap::new();
        env.insert("XDG_RUNTIME_DIR".to_string(), "/nonexistent".to_string());
        env.insert("BIOMEOS_FAMILY_ID".to_string(), fam);

        let result = discover_songbird_socket_with(&mock_env(&env));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), legacy_socket);

        let _ = std::fs::remove_file(&legacy_socket);
        let _ = std::fs::remove_dir(&biomeos_dir);
    }

    #[test]
    fn test_discover_songbird_socket_common_pattern_tmp_biomeos_dir() {
        let _lock = TMP_BIOMEOS_EMPTY_FAMILY_LOCK.lock().unwrap();

        let biomeos_dir = "/tmp/biomeos";
        let tmp_socket = format!("{biomeos_dir}/songbird.sock");
        let _ = std::fs::create_dir_all(biomeos_dir);
        let existed = std::path::Path::new(&tmp_socket).exists();
        if !existed {
            std::fs::File::create(&tmp_socket).unwrap();
        }

        let mut env = HashMap::new();
        env.insert("XDG_RUNTIME_DIR".to_string(), "/nonexistent".to_string());

        let result = discover_songbird_socket_with(&mock_env(&env));
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), tmp_socket);

        if !existed {
            let _ = std::fs::remove_file(&tmp_socket);
        }
    }

    #[test]
    fn test_discover_songbird_socket_family_id_fallback() {
        let temp = tempfile::tempdir().unwrap();
        let biomeos_dir = temp.path().join("biomeos");
        std::fs::create_dir_all(&biomeos_dir).unwrap();
        let socket_path = biomeos_dir.join("songbird-fam2.sock");
        std::fs::File::create(&socket_path).unwrap();

        let mut env = HashMap::new();
        env.insert(
            "XDG_RUNTIME_DIR".to_string(),
            temp.path().to_string_lossy().into_owned(),
        );
        env.insert("FAMILY_ID".to_string(), "fam2".to_string());

        let result = discover_songbird_socket_with(&mock_env(&env));
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            format!("{}/biomeos/songbird-fam2.sock", temp.path().display())
        );
    }

    #[tokio::test]
    async fn test_register_with_songbird_success() {
        let temp = tempfile::tempdir().unwrap();
        let socket_path = temp.path().join("songbird.sock");

        let listener = tokio::net::UnixListener::bind(&socket_path).unwrap();

        let server_handle = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (reader, mut writer) = stream.into_split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let success_response = r#"{"jsonrpc":"2.0","result":{},"id":1}"#;
            writer
                .write_all((success_response.to_string() + "\n").as_bytes())
                .await
                .unwrap();
            writer.flush().await.unwrap();
        });

        let path_str = socket_path.to_str().unwrap();
        let result =
            register_with_songbird_at(path_str, "/run/user/1000/biomeos-device.sock").await;
        assert!(result.is_ok());

        server_handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_register_with_songbird_error_response() {
        let temp = tempfile::tempdir().unwrap();
        let socket_path = temp.path().join("songbird-err.sock");

        let listener = tokio::net::UnixListener::bind(&socket_path).unwrap();

        let server_handle = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (reader, mut writer) = stream.into_split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let err_response =
                r#"{"jsonrpc":"2.0","error":{"code":-1,"message":"Registration rejected"},"id":1}"#;
            writer
                .write_all((err_response.to_string() + "\n").as_bytes())
                .await
                .unwrap();
            writer.flush().await.unwrap();
        });

        let path_str = socket_path.to_str().unwrap();
        let result =
            register_with_songbird_at(path_str, "/run/user/1000/biomeos-device.sock").await;
        assert!(result.is_err());
        assert!(
            result
                .unwrap_err()
                .to_string()
                .contains("Registration rejected")
        );

        server_handle.await.unwrap();
    }

    #[tokio::test]
    async fn test_register_with_songbird_error_unknown_message() {
        let temp = tempfile::tempdir().unwrap();
        let socket_path = temp.path().join("songbird-err2.sock");

        let listener = tokio::net::UnixListener::bind(&socket_path).unwrap();

        let server_handle = tokio::spawn(async move {
            let (stream, _) = listener.accept().await.unwrap();
            let (reader, mut writer) = stream.into_split();
            let mut reader = BufReader::new(reader);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let err_response = r#"{"jsonrpc":"2.0","error":{"code":-1},"id":1}"#;
            writer
                .write_all((err_response.to_string() + "\n").as_bytes())
                .await
                .unwrap();
            writer.flush().await.unwrap();
        });

        let path_str = socket_path.to_str().unwrap();
        let result =
            register_with_songbird_at(path_str, "/run/user/1000/biomeos-device.sock").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown error"));

        server_handle.await.unwrap();
    }
}
