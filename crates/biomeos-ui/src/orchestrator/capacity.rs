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

use crate::primal_client::{PrimalClient, ToadStoolClient};
use anyhow::Result;
use tracing::{debug, info, warn};

/// Result of capacity check
#[derive(Debug, Clone, PartialEq)]
pub enum CapacityResult {
    /// Capacity available
    Available,
    /// Capacity insufficient with details
    Insufficient { reason: String },
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
}
