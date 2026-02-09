//! Authorization Module
//!
//! Handles authorization checks via BearDog security primal.
//!
//! ## Network Effect Phase 1: Authorization
//!
//! Checks:
//! - User has permission to assign this device
//! - Primal accepts this device type
//!
//! ## Graceful Degradation
//!
//! If BearDog is not available, authorization is granted by default.

use crate::primal_client::BearDogClient;
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
    /// Authorize device assignment via BearDog
    ///
    /// Falls back to allowing the operation if BearDog is unavailable.
    pub async fn authorize_device_assignment(
        beardog: &Option<BearDogClient>,
        user_id: &str,
        device_id: &str,
        primal_id: &str,
    ) -> Result<AuthorizationResult> {
        debug!(
            "Authorizing device assignment: user={}, device={}, primal={}",
            user_id, device_id, primal_id
        );

        // Check if BearDog is available
        if let Some(ref beardog) = beardog {
            info!("🔒 BearDog available - checking authorization");

            // Call BearDog to check authorization
            match beardog
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
                        .and_then(|v| v.as_bool())
                        .unwrap_or(false)
                    {
                        info!("✅ BearDog authorization: Approved");
                        Ok(AuthorizationResult::Authorized)
                    } else {
                        let reason = result
                            .get("reason")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Authorization denied")
                            .to_string();
                        info!("❌ BearDog authorization: Denied - {}", reason);
                        Ok(AuthorizationResult::Denied(reason))
                    }
                }
                Err(e) => {
                    // BearDog might not support this method yet
                    warn!("⚠️ BearDog call failed: {} - falling back to allow", e);
                    info!("✅ BearDog authorization: Approved (fallback)");
                    Ok(AuthorizationResult::Authorized)
                }
            }
        } else {
            warn!("⚠️ No security primal (BearDog) available");
            warn!("⚠️ Allowing assignment without authorization (graceful degradation)");
            info!("✅ Authorization: Approved (no security primal)");
            Ok(AuthorizationResult::Authorized)
        }
    }

    /// Get the current user ID from BearDog session or environment
    ///
    /// Falls back to "anonymous" if no session is available.
    pub async fn get_current_user_id(beardog: &Option<BearDogClient>) -> String {
        // Try to get from BearDog session
        if let Some(ref beardog) = beardog {
            if let Ok(result) = beardog
                .call("auth.get_current_user", serde_json::json!({}))
                .await
            {
                if let Some(user_id) = result.get("user_id").and_then(|v| v.as_str()) {
                    return user_id.to_string();
                }
            }
        }

        // Fall back to environment variable
        if let Ok(user) = std::env::var("BIOMEOS_USER") {
            return user;
        }

        // Fall back to system user
        if let Ok(user) = std::env::var("USER") {
            return user;
        }

        // Default to anonymous
        "anonymous".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_authorization_no_beardog() {
        let result = Authorization::authorize_device_assignment(
            &None,
            "test-user",
            "test-device",
            "test-primal",
        )
        .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), AuthorizationResult::Authorized);
    }

    #[tokio::test]
    async fn test_get_current_user_no_beardog() {
        let user_id = Authorization::get_current_user_id(&None).await;
        // Should return something (env var or "anonymous")
        assert!(!user_id.is_empty());
    }

    #[tokio::test]
    async fn test_authorization_no_beardog_graceful_degradation() {
        // Tests that authorization is granted by default when BearDog is unavailable
        let result = Authorization::authorize_device_assignment(
            &None,
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
            _ => panic!("Expected Denied result"),
        }
    }

    #[test]
    fn test_authorization_result_debug() {
        let authorized = AuthorizationResult::Authorized;
        let denied = AuthorizationResult::Denied("test reason".to_string());

        // Test Debug trait
        assert!(format!("{:?}", authorized).contains("Authorized"));
        assert!(format!("{:?}", denied).contains("Denied"));
        assert!(format!("{:?}", denied).contains("test reason"));
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
    async fn test_get_current_user_with_env_var() {
        // Save original value
        let original = std::env::var("BIOMEOS_USER").ok();

        // Set env var
        std::env::set_var("BIOMEOS_USER", "test-env-user");

        let user_id = Authorization::get_current_user_id(&None).await;
        assert_eq!(user_id, "test-env-user");

        // Restore original
        if let Some(val) = original {
            std::env::set_var("BIOMEOS_USER", val);
        } else {
            std::env::remove_var("BIOMEOS_USER");
        }
    }
}
