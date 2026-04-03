// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Authorization Module
//!
//! Handles authorization checks via the security / encryption capability provider.
//!
//! ## Network Effect Phase 1: Authorization
//!
//! Checks:
//! - User has permission to assign this device
//! - Primal accepts this device type
//!
//! ## Graceful Degradation
//!
//! If no security provider is available, authorization is granted by default.

use crate::primal_client::SecurityClient;
use anyhow::Result;
use tracing::{debug, info, warn};

/// Result of authorization check
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AuthorizationResult {
    /// Authorization granted
    Authorized,
    /// Authorization denied with reason
    Denied(String),
}

/// Authorization handler
pub struct Authorization;

impl Authorization {
    /// Authorize device assignment via the security provider
    ///
    /// Falls back to allowing the operation if no security client is connected.
    pub async fn authorize_device_assignment(
        security: Option<&SecurityClient>,
        user_id: &str,
        device_id: &str,
        primal_id: &str,
    ) -> Result<AuthorizationResult> {
        debug!(
            "Authorizing device assignment: user={}, device={}, primal={}",
            user_id, device_id, primal_id
        );

        if let Some(security) = security {
            info!("🔒 Security provider available — checking authorization");

            match security
                .call(
                    "auth.check_device_assignment",
                    serde_json::json!({
                        "user_id": user_id,
                        "device_id": device_id,
                        "primal_id": primal_id
                    }),
                )
                .await
            {
                Ok(result) => {
                    if result
                        .get("authorized")
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(false)
                    {
                        info!("✅ Security provider authorization: approved");
                        Ok(AuthorizationResult::Authorized)
                    } else {
                        let reason = result
                            .get("reason")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Authorization denied")
                            .to_string();
                        info!("❌ Security provider authorization: denied — {}", reason);
                        Ok(AuthorizationResult::Denied(reason))
                    }
                }
                Err(e) => {
                    warn!(
                        "⚠️ Security provider call failed: {} — falling back to allow",
                        e
                    );
                    info!("✅ Security provider authorization: approved (fallback)");
                    Ok(AuthorizationResult::Authorized)
                }
            }
        } else {
            warn!("⚠️ No security provider available");
            warn!("⚠️ Allowing assignment without authorization (graceful degradation)");
            info!("✅ Authorization: approved (no security provider)");
            Ok(AuthorizationResult::Authorized)
        }
    }

    /// Get the current user ID from the security provider session or environment
    ///
    /// Falls back to "anonymous" if no session is available.
    pub async fn get_current_user_id(security: Option<&SecurityClient>) -> String {
        Self::get_current_user_id_with(security, None, None).await
    }

    /// Same as [`get_current_user_id`](Self::get_current_user_id), with optional overrides for
    /// `BIOMEOS_USER` and `USER` (use in tests to avoid mutating the process environment).
    pub async fn get_current_user_id_with(
        security: Option<&SecurityClient>,
        biomeos_user: Option<&str>,
        system_user: Option<&str>,
    ) -> String {
        if let Some(security) = security
            && let Ok(result) = security
                .call("auth.get_current_user", serde_json::json!({}))
                .await
            && let Some(user_id) = result.get("user_id").and_then(|v| v.as_str())
        {
            return user_id.to_string();
        }

        if let Some(user) = biomeos_user {
            return user.to_string();
        }

        if let Ok(user) = std::env::var("BIOMEOS_USER") {
            return user;
        }

        if let Some(user) = system_user {
            return user.to_string();
        }

        if let Ok(user) = std::env::var("USER") {
            return user;
        }

        "anonymous".to_string()
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

    async fn spawn_mock_security(
        authorized: bool,
        reason: Option<&str>,
    ) -> (String, tokio::task::JoinHandle<()>) {
        let dir = tempfile::tempdir().unwrap();
        let socket_path = dir.path().join("security.sock");
        let path_str = socket_path.to_str().unwrap().to_string();

        let auth_response = if authorized {
            serde_json::json!({"authorized": true})
        } else {
            serde_json::json!({"authorized": false, "reason": reason.unwrap_or("Denied")})
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
                        result: Some(auth_response),
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

    async fn spawn_mock_security_user(user_id: &str) -> (String, tokio::task::JoinHandle<()>) {
        let dir = tempfile::tempdir().unwrap();
        let socket_path = dir.path().join("security.sock");
        let path_str = socket_path.to_str().unwrap().to_string();
        let user_id = user_id.to_string();

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
                        result: Some(serde_json::json!({"user_id": user_id})),
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
    async fn test_authorization_no_security_provider() {
        let result = Authorization::authorize_device_assignment(
            None,
            "test-user",
            "test-device",
            "test-primal",
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), AuthorizationResult::Authorized);
    }

    #[tokio::test]
    async fn test_get_current_user_no_security_provider() {
        let user_id = Authorization::get_current_user_id(None).await;
        // Should return something (env var or "anonymous")
        assert!(!user_id.is_empty());
    }

    #[tokio::test]
    async fn test_authorization_no_security_graceful_degradation() {
        // Authorization is granted by default when no security provider is available
        let result = Authorization::authorize_device_assignment(
            None,
            "user-abc-123",
            "device-xyz-456",
            "primal-789",
        )
        .await;

        // Should succeed with graceful degradation
        assert!(result.is_ok());
        let auth_result = result.unwrap();
        assert_eq!(auth_result, AuthorizationResult::Authorized);
    }

    #[test]
    fn test_authorization_result_authorized() {
        let result = AuthorizationResult::Authorized;
        assert_eq!(result, AuthorizationResult::Authorized);
    }

    #[test]
    fn test_authorization_result_denied() {
        let reason = "Insufficient permissions".to_string();
        let result = AuthorizationResult::Denied(reason.clone());

        match result {
            AuthorizationResult::Denied(r) => assert_eq!(r, reason),
            AuthorizationResult::Authorized => panic!("Expected Denied result"),
        }
    }

    #[test]
    fn test_authorization_result_debug() {
        let authorized = AuthorizationResult::Authorized;
        let denied = AuthorizationResult::Denied("test reason".to_string());

        // Test Debug trait
        assert!(format!("{authorized:?}").contains("Authorized"));
        assert!(format!("{denied:?}").contains("Denied"));
        assert!(format!("{denied:?}").contains("test reason"));
    }

    #[test]
    fn test_authorization_result_clone() {
        let original = AuthorizationResult::Denied("clone test".to_string());
        let cloned = original.clone();

        assert_eq!(original, cloned);
    }

    #[test]
    fn test_authorization_result_eq() {
        let authorized1 = AuthorizationResult::Authorized;
        let authorized2 = AuthorizationResult::Authorized;
        let denied1 = AuthorizationResult::Denied("same reason".to_string());
        let denied2 = AuthorizationResult::Denied("same reason".to_string());
        let denied3 = AuthorizationResult::Denied("different reason".to_string());

        assert_eq!(authorized1, authorized2);
        assert_eq!(denied1, denied2);
        assert_ne!(authorized1, denied1);
        assert_ne!(denied1, denied3);
    }

    #[tokio::test]
    async fn test_authorization_security_denied() {
        let (path, _handle) = spawn_mock_security(false, Some("Insufficient permissions")).await;
        let client = Some(crate::primal_client::SecurityClient::with_socket(
            "security", &path,
        ));
        let result = Authorization::authorize_device_assignment(
            client.as_ref(),
            "test-user",
            "test-device",
            "test-primal",
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(
            result.unwrap(),
            AuthorizationResult::Denied("Insufficient permissions".to_string())
        );
    }

    #[tokio::test]
    async fn test_authorization_security_authorized() {
        let (path, _handle) = spawn_mock_security(true, None).await;
        let client = Some(crate::primal_client::SecurityClient::with_socket(
            "security", &path,
        ));
        let result = Authorization::authorize_device_assignment(
            client.as_ref(),
            "test-user",
            "test-device",
            "test-primal",
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), AuthorizationResult::Authorized);
    }

    #[tokio::test]
    async fn test_authorization_security_call_fails_fallback() {
        let client = Some(crate::primal_client::SecurityClient::with_socket(
            "security",
            "/nonexistent/security.sock",
        ));
        let result = Authorization::authorize_device_assignment(
            client.as_ref(),
            "test-user",
            "test-device",
            "test-primal",
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), AuthorizationResult::Authorized);
    }

    #[tokio::test]
    async fn test_get_current_user_from_security_provider() {
        let (path, _handle) = spawn_mock_security_user("security-user-123").await;
        let client = Some(crate::primal_client::SecurityClient::with_socket(
            "security", &path,
        ));
        let user_id = Authorization::get_current_user_id(client.as_ref()).await;
        assert_eq!(user_id, "security-user-123");
    }

    #[tokio::test]
    async fn test_get_current_user_with_env_var() {
        let user_id =
            Authorization::get_current_user_id_with(None, Some("test-env-user"), None).await;
        assert_eq!(user_id, "test-env-user");
    }
}
