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
        songbird: &Option<SongbirdClient>,
        device_id: &str,
        primal_id: &str,
    ) -> Result<ValidationResult> {
        debug!(
            "Validating device assignment: device={}, primal={}",
            device_id, primal_id
        );

        if let Some(ref songbird) = songbird {
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
                        .and_then(|v| v.as_bool())
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
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_validation_no_songbird() {
        let result =
            Validation::validate_device_assignment(&None, "test-device", "test-primal").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), ValidationResult::Valid);
    }
}
