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
}
