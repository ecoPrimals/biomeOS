// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Validation Module
//!
//! Handles validation checks via Songbird service registry.
//!
//! ## Network Effect Phase 2: Validation
//!
//! Checks:
//! - Device is available (not already assigned)
//! - Primal is healthy and running
//! - No conflicts with existing assignments
//!
//! ## Graceful Degradation
//!
//! If Songbird is not available, validation passes by default.

use crate::primal_client::SongbirdClient;
use anyhow::Result;
use tracing::{debug, info, warn};

/// Result of validation check
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    /// Validation passed
    Valid,
    /// Validation failed with reason
    Invalid(String),
}

/// Validation handler
pub struct Validation;

impl Validation {
    /// Validate device assignment via Songbird
    ///
    /// Falls back to allowing the operation if Songbird is unavailable.
    pub async fn validate_device_assignment(
        songbird: Option<&SongbirdClient>,
        device_id: &str,
        primal_id: &str,
    ) -> Result<ValidationResult> {
        debug!(
            "Validating device assignment: device={}, primal={}",
            device_id, primal_id
        );

        if let Some(songbird) = songbird {
            info!("🎵 Songbird available - checking validation");

            // Call Songbird to validate the assignment
            match songbird
                .call(
                    "registry.validate_assignment",
                    serde_json::json!({
                        "device_id": device_id,
                        "primal_id": primal_id
                    }),
                )
                .await
            {
                Ok(result) => {
                    if result
                        .get("valid")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(true)
                    {
                        info!("✅ Songbird validation: Passed");
                        Ok(ValidationResult::Valid)
                    } else {
                        let reason = result
                            .get("reason")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Validation failed")
                            .to_string();
                        info!("❌ Songbird validation: Failed - {}", reason);
                        Ok(ValidationResult::Invalid(reason))
                    }
                }
                Err(e) => {
                    // Songbird might not support this method yet
                    warn!("⚠️ Songbird call failed: {} - falling back to valid", e);
                    info!("✅ Songbird validation: Passed (fallback)");
                    Ok(ValidationResult::Valid)
                }
            }
        } else {
            warn!("⚠️ No service registry (Songbird) available");
            warn!("⚠️ Allowing assignment without validation (graceful degradation)");
            info!("✅ Validation: Passed (no service registry)");
            Ok(ValidationResult::Valid)
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use biomeos_core::atomic_client::{JsonRpcRequest, JsonRpcResponse};
    use biomeos_test_utils::ready_signal;
    use biomeos_types::JsonRpcVersion;
    use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
    use tokio::net::UnixListener;

    async fn spawn_mock_songbird(
        valid: bool,
        reason: Option<&str>,
    ) -> (String, tokio::task::JoinHandle<()>) {
        let dir = tempfile::tempdir().unwrap();
        let socket_path = dir.path().join("songbird.sock");
        let path_str = socket_path.to_str().unwrap().to_string();

        let validation_response = if valid {
            serde_json::json!({"valid": true})
        } else {
            serde_json::json!({"valid": false, "reason": reason.unwrap_or("Validation failed")})
        };

        let path_for_listener = path_str.clone();
        let (mut ready_tx, ready_rx) = ready_signal();
        let handle = tokio::spawn(async move {
            let _dir = dir;
            let listener = UnixListener::bind(&path_for_listener).unwrap();
            ready_tx.signal();
            if let Ok((stream, _)) = listener.accept().await {
                let (reader, mut writer) = tokio::io::split(stream);
                let mut reader = BufReader::new(reader);
                let mut line = String::new();
                if reader.read_line(&mut line).await.is_ok()
                    && let Ok(req) = serde_json::from_str::<JsonRpcRequest>(&line)
                {
                    let response = JsonRpcResponse {
                        jsonrpc: JsonRpcVersion,
                        result: Some(validation_response),
                        error: None,
                        id: req.id.clone().unwrap_or(serde_json::Value::Null),
                    };
                    let _ = writer
                        .write_all(serde_json::to_string(&response).unwrap().as_bytes())
                        .await;
                    let _ = writer.write_all(b"\n").await;
                }
            }
        });

        ready_rx.wait().await.unwrap();
        (path_str, handle)
    }

    #[tokio::test]
    async fn test_validation_no_songbird() {
        let result =
            Validation::validate_device_assignment(None, "test-device", "test-primal").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ValidationResult::Valid);
    }

    #[tokio::test]
    async fn test_validation_no_songbird_graceful_degradation() {
        // Tests that validation passes by default when Songbird is unavailable
        let result =
            Validation::validate_device_assignment(None, "device-abc-123", "primal-xyz-456").await;

        // Should succeed with graceful degradation
        assert!(result.is_ok());
        let validation_result = result.unwrap();
        assert_eq!(validation_result, ValidationResult::Valid);
    }

    #[test]
    fn test_validation_result_valid() {
        let result = ValidationResult::Valid;
        assert_eq!(result, ValidationResult::Valid);
    }

    #[test]
    fn test_validation_result_invalid() {
        let reason = "Device already assigned".to_string();
        let result = ValidationResult::Invalid(reason.clone());

        match result {
            ValidationResult::Invalid(r) => assert_eq!(r, reason),
            ValidationResult::Valid => panic!("Expected Invalid result"),
        }
    }

    #[test]
    fn test_validation_result_debug() {
        let valid = ValidationResult::Valid;
        let invalid = ValidationResult::Invalid("test reason".to_string());

        // Test Debug trait
        assert!(format!("{valid:?}").contains("Valid"));
        assert!(format!("{invalid:?}").contains("Invalid"));
        assert!(format!("{invalid:?}").contains("test reason"));
    }

    #[test]
    fn test_validation_result_clone() {
        let original = ValidationResult::Invalid("clone test".to_string());
        let cloned = original.clone();

        assert_eq!(original, cloned);
    }

    #[test]
    fn test_validation_result_eq() {
        let valid1 = ValidationResult::Valid;
        let valid2 = ValidationResult::Valid;
        let invalid1 = ValidationResult::Invalid("same reason".to_string());
        let invalid2 = ValidationResult::Invalid("same reason".to_string());
        let invalid3 = ValidationResult::Invalid("different reason".to_string());

        assert_eq!(valid1, valid2);
        assert_eq!(invalid1, invalid2);
        assert_ne!(valid1, invalid1);
        assert_ne!(invalid1, invalid3);
    }

    #[tokio::test]
    async fn test_validation_songbird_invalid() {
        let (path, _handle) = spawn_mock_songbird(false, Some("Device already assigned")).await;
        let client = Some(crate::primal_client::SongbirdClient::with_socket(
            "songbird", &path,
        ));
        let result =
            Validation::validate_device_assignment(client.as_ref(), "test-device", "test-primal")
                .await;

        assert!(result.is_ok());
        match result.unwrap() {
            ValidationResult::Invalid(reason) => assert_eq!(reason, "Device already assigned"),
            ValidationResult::Valid => panic!("Expected Invalid"),
        }
    }

    #[tokio::test]
    async fn test_validation_songbird_valid() {
        let (path, _handle) = spawn_mock_songbird(true, None).await;
        let client = Some(crate::primal_client::SongbirdClient::with_socket(
            "songbird", &path,
        ));
        let result =
            Validation::validate_device_assignment(client.as_ref(), "test-device", "test-primal")
                .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ValidationResult::Valid);
    }

    #[tokio::test]
    async fn test_validation_songbird_call_fails_fallback() {
        let client = Some(crate::primal_client::SongbirdClient::with_socket(
            "songbird",
            "/nonexistent/songbird.sock",
        ));
        let result =
            Validation::validate_device_assignment(client.as_ref(), "test-device", "test-primal")
                .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ValidationResult::Valid);
    }
}
