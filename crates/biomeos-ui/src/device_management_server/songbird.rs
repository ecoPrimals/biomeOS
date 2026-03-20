// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

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

    info!("📡 Registering with Songbird at: {}", songbird_socket);

    let stream = tokio::net::UnixStream::connect(&songbird_socket)
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
                "path": socket_path
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

    if response.get("error").is_some() {
        let msg = response["error"]["message"]
            .as_str()
            .unwrap_or("Unknown error");
        anyhow::bail!("Songbird registration failed: {msg}");
    }

    info!("✅ Registered with Songbird for UDP multicast discovery");
    Ok(())
}

/// Discover Songbird socket using XDG-compliant paths
pub(super) fn discover_songbird_socket() -> Result<String> {
    if let Ok(socket) = std::env::var("SONGBIRD_SOCKET") {
        return Ok(socket);
    }

    if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
        let socket = format!("{runtime}/biomeos/songbird.sock");
        if std::path::Path::new(&socket).exists() {
            return Ok(socket);
        }
    }

    if let Ok(family_id) =
        std::env::var("BIOMEOS_FAMILY_ID").or_else(|_| std::env::var("FAMILY_ID"))
    {
        if let Ok(runtime) = std::env::var("XDG_RUNTIME_DIR") {
            let socket = format!("{runtime}/biomeos/songbird-{family_id}.sock");
            if std::path::Path::new(&socket).exists() {
                return Ok(socket);
            }
        }
        let socket = format!("/tmp/songbird-{family_id}.sock");
        if std::path::Path::new(&socket).exists() {
            tracing::warn!("⚠️ Using legacy /tmp path: {}", socket);
            return Ok(socket);
        }
    }

    for pattern in &["/run/biomeos/songbird.sock", "/tmp/songbird.sock"] {
        if std::path::Path::new(pattern).exists() {
            if pattern.starts_with("/tmp") {
                tracing::warn!("⚠️ Using legacy /tmp path: {}", pattern);
            }
            return Ok((*pattern).to_string());
        }
    }

    anyhow::bail!("Songbird socket not found")
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use biomeos_test_utils::TestEnvGuard;
    use serial_test::serial;
    use tokio::io::AsyncBufReadExt;

    #[test]
    #[serial]
    fn test_discover_songbird_socket_env_override() {
        let _guard = TestEnvGuard::set("SONGBIRD_SOCKET", "/custom/songbird.sock");
        let result = discover_songbird_socket();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "/custom/songbird.sock");
    }

    #[test]
    #[serial]
    fn test_discover_songbird_socket_not_found() {
        let _guard_son = TestEnvGuard::remove("SONGBIRD_SOCKET");
        let _guard_xdg = TestEnvGuard::set("XDG_RUNTIME_DIR", "/nonexistent_xdg_path_for_test");
        let _guard_fam = TestEnvGuard::remove("BIOMEOS_FAMILY_ID");
        let _guard_legacy = TestEnvGuard::remove("FAMILY_ID");
        let result = discover_songbird_socket();
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("not found"));
    }

    #[test]
    #[serial]
    fn test_discover_songbird_socket_xdg_runtime_exists() {
        let temp = tempfile::tempdir().unwrap();
        let socket_path = temp.path().join("biomeos").join("songbird.sock");
        std::fs::create_dir_all(socket_path.parent().unwrap()).unwrap();
        std::fs::File::create(&socket_path).unwrap();

        let _guard_son = TestEnvGuard::remove("SONGBIRD_SOCKET");
        let _guard_xdg = TestEnvGuard::set("XDG_RUNTIME_DIR", temp.path().to_str().unwrap());
        let _guard_fam = TestEnvGuard::remove("BIOMEOS_FAMILY_ID");
        let _guard_legacy = TestEnvGuard::remove("FAMILY_ID");

        let result = discover_songbird_socket();
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            format!("{}/biomeos/songbird.sock", temp.path().display())
        );
    }

    #[test]
    #[serial]
    fn test_discover_songbird_socket_family_xdg_path() {
        let temp = tempfile::tempdir().unwrap();
        let socket_path = temp.path().join("biomeos").join("songbird-family99.sock");
        std::fs::create_dir_all(socket_path.parent().unwrap()).unwrap();
        std::fs::File::create(&socket_path).unwrap();

        let _guard_son = TestEnvGuard::remove("SONGBIRD_SOCKET");
        let _guard_xdg = TestEnvGuard::set("XDG_RUNTIME_DIR", temp.path().to_str().unwrap());
        let _guard_fam = TestEnvGuard::set("BIOMEOS_FAMILY_ID", "family99");
        let _guard_legacy = TestEnvGuard::remove("FAMILY_ID");

        let result = discover_songbird_socket();
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            format!("{}/biomeos/songbird-family99.sock", temp.path().display())
        );
    }

    #[test]
    #[serial]
    fn test_discover_songbird_socket_family_legacy_tmp() {
        let legacy_socket = "/tmp/songbird-testlegacy123.sock";
        let _ = std::fs::remove_file(legacy_socket);
        std::fs::File::create(legacy_socket).unwrap();

        let _guard_son = TestEnvGuard::remove("SONGBIRD_SOCKET");
        let _guard_xdg = TestEnvGuard::set("XDG_RUNTIME_DIR", "/nonexistent");
        let _guard_fam = TestEnvGuard::set("BIOMEOS_FAMILY_ID", "testlegacy123");
        let _guard_legacy = TestEnvGuard::remove("FAMILY_ID");

        let result = discover_songbird_socket();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), legacy_socket);

        let _ = std::fs::remove_file(legacy_socket);
    }

    #[test]
    #[serial]
    fn test_discover_songbird_socket_common_pattern_tmp() {
        let tmp_socket = "/tmp/songbird.sock";
        let existed = std::path::Path::new(tmp_socket).exists();
        if !existed {
            std::fs::File::create(tmp_socket).unwrap();
        }

        let _guard_son = TestEnvGuard::remove("SONGBIRD_SOCKET");
        let _guard_xdg = TestEnvGuard::set("XDG_RUNTIME_DIR", "/nonexistent");
        let _guard_fam = TestEnvGuard::remove("BIOMEOS_FAMILY_ID");
        let _guard_legacy = TestEnvGuard::remove("FAMILY_ID");

        let result = discover_songbird_socket();
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), tmp_socket);

        if !existed {
            let _ = std::fs::remove_file(tmp_socket);
        }
    }

    #[test]
    #[serial]
    fn test_discover_songbird_socket_family_id_fallback() {
        let temp = tempfile::tempdir().unwrap();
        let biomeos_dir = temp.path().join("biomeos");
        std::fs::create_dir_all(&biomeos_dir).unwrap();
        let socket_path = biomeos_dir.join("songbird-fam2.sock");
        std::fs::File::create(&socket_path).unwrap();

        let _guard_son = TestEnvGuard::remove("SONGBIRD_SOCKET");
        let _guard_xdg = TestEnvGuard::set("XDG_RUNTIME_DIR", temp.path().to_str().unwrap());
        let _guard_fam = TestEnvGuard::remove("BIOMEOS_FAMILY_ID");
        let _guard_legacy = TestEnvGuard::set("FAMILY_ID", "fam2");

        let result = discover_songbird_socket();
        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            format!("{}/biomeos/songbird-fam2.sock", temp.path().display())
        );
    }

    #[tokio::test]
    #[serial]
    async fn test_register_with_songbird_success() {
        let temp = tempfile::tempdir().unwrap();
        let socket_path = temp.path().join("songbird.sock");

        let _guard = TestEnvGuard::set("SONGBIRD_SOCKET", socket_path.to_str().unwrap());

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

        let result = register_with_songbird("/run/user/1000/biomeos-device.sock").await;
        assert!(result.is_ok());

        server_handle.await.unwrap();
    }

    #[tokio::test]
    #[serial]
    async fn test_register_with_songbird_error_response() {
        let temp = tempfile::tempdir().unwrap();
        let socket_path = temp.path().join("songbird-err.sock");

        let _guard = TestEnvGuard::set("SONGBIRD_SOCKET", socket_path.to_str().unwrap());

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

        let result = register_with_songbird("/run/user/1000/biomeos-device.sock").await;
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
    #[serial]
    async fn test_register_with_songbird_error_unknown_message() {
        let temp = tempfile::tempdir().unwrap();
        let socket_path = temp.path().join("songbird-err2.sock");

        let _guard = TestEnvGuard::set("SONGBIRD_SOCKET", socket_path.to_str().unwrap());

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

        let result = register_with_songbird("/run/user/1000/biomeos-device.sock").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("Unknown error"));

        server_handle.await.unwrap();
    }
}
