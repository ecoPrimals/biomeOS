//! Capacity Module
//!
//! Handles capacity checks via ToadStool compute primal.
//!
//! ## Network Effect Phase 3: Capacity Check
//!
//! Checks:
//! - Primal has capacity for device
//! - Resource requirements can be met
//!
//! ## Graceful Degradation
//!
//! If ToadStool is not available, capacity check passes by default.

use crate::primal_client::ToadStoolClient;
use anyhow::Result;
use tracing::{debug, info, warn};

/// Result of capacity check
#[derive(Debug, Clone, PartialEq)]
pub enum CapacityResult {
    /// Capacity available
    Available,
    /// Capacity insufficient with details
    Insufficient {
        /// Reason why capacity is insufficient
        reason: String,
    },
}

/// Capacity checker
pub struct Capacity;

impl Capacity {
    /// Check primal capacity via ToadStool
    ///
    /// Falls back to allowing the operation if ToadStool is unavailable.
    pub async fn check_primal_capacity(
        toadstool: &Option<ToadStoolClient>,
        device_id: &str,
        primal_id: &str,
    ) -> Result<CapacityResult> {
        debug!(
            "Checking primal capacity: device={}, primal={}",
            device_id, primal_id
        );

        if let Some(ref toadstool) = toadstool {
            info!("🍄 ToadStool available - checking capacity");

            // Call ToadStool to check resource capacity
            match toadstool
                .call(
                    "compute.check_capacity",
                    serde_json::json!({
                        "device_id": device_id,
                        "primal_id": primal_id
                    }),
                )
                .await
            {
                Ok(result) => {
                    if result
                        .get("available")
                        .and_then(|v| v.as_bool())
                        .unwrap_or(true)
                    {
                        info!("✅ ToadStool capacity: Available");
                        Ok(CapacityResult::Available)
                    } else {
                        let reason = result
                            .get("reason")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Insufficient capacity")
                            .to_string();
                        info!("❌ ToadStool capacity: Insufficient - {}", reason);
                        Ok(CapacityResult::Insufficient { reason })
                    }
                }
                Err(e) => {
                    // ToadStool might not support this method yet
                    warn!(
                        "⚠️ ToadStool call failed: {} - falling back to available",
                        e
                    );
                    info!("✅ ToadStool capacity: Available (fallback)");
                    Ok(CapacityResult::Available)
                }
            }
        } else {
            warn!("⚠️ No compute primal (ToadStool) available");
            warn!("⚠️ Allowing assignment without capacity check (graceful degradation)");
            info!("✅ Capacity: Available (no compute primal)");
            Ok(CapacityResult::Available)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_capacity_no_toadstool() {
        let result = Capacity::check_primal_capacity(&None, "test-device", "test-primal").await;

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), CapacityResult::Available));
    }

    #[tokio::test]
    async fn test_capacity_no_toadstool_graceful_degradation() {
        // Tests that capacity check passes by default when ToadStool is unavailable
        let result =
            Capacity::check_primal_capacity(&None, "device-abc-123", "primal-xyz-456").await;

        // Should succeed with graceful degradation
        assert!(result.is_ok());
        let capacity_result = result.unwrap();
        assert_eq!(capacity_result, CapacityResult::Available);
    }

    #[test]
    fn test_capacity_result_available() {
        let result = CapacityResult::Available;
        assert_eq!(result, CapacityResult::Available);
    }

    #[test]
    fn test_capacity_result_insufficient() {
        let reason = "Not enough memory".to_string();
        let result = CapacityResult::Insufficient {
            reason: reason.clone(),
        };

        match result {
            CapacityResult::Insufficient { reason: r } => assert_eq!(r, reason),
            _ => panic!("Expected Insufficient result"),
        }
    }

    #[test]
    fn test_capacity_result_debug() {
        let available = CapacityResult::Available;
        let insufficient = CapacityResult::Insufficient {
            reason: "test reason".to_string(),
        };

        // Test Debug trait
        assert!(format!("{:?}", available).contains("Available"));
        assert!(format!("{:?}", insufficient).contains("Insufficient"));
        assert!(format!("{:?}", insufficient).contains("test reason"));
    }

    #[test]
    fn test_capacity_result_clone() {
        let original = CapacityResult::Insufficient {
            reason: "clone test".to_string(),
        };
        let cloned = original.clone();

        assert_eq!(original, cloned);
    }

    #[test]
    fn test_capacity_result_eq() {
        let available1 = CapacityResult::Available;
        let available2 = CapacityResult::Available;
        let insufficient1 = CapacityResult::Insufficient {
            reason: "same reason".to_string(),
        };
        let insufficient2 = CapacityResult::Insufficient {
            reason: "same reason".to_string(),
        };
        let insufficient3 = CapacityResult::Insufficient {
            reason: "different reason".to_string(),
        };

        assert_eq!(available1, available2);
        assert_eq!(insufficient1, insufficient2);
        assert_ne!(available1, insufficient1);
        assert_ne!(insufficient1, insufficient3);
    }
}
