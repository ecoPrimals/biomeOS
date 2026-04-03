// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Capacity Module
//!
//! Handles capacity checks via the compute capability provider.
//!
//! ## Network Effect Phase 3: Capacity Check
//!
//! Checks:
//! - Primal has capacity for device
//! - Resource requirements can be met
//!
//! ## Graceful Degradation
//!
//! If no compute provider is available, capacity check passes by default.

use crate::primal_client::ComputeClient;
use anyhow::Result;
use tracing::{debug, info, warn};

/// Result of capacity check
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Check primal capacity via the compute provider
    ///
    /// Falls back to allowing the operation if compute is unavailable.
    pub async fn check_primal_capacity(
        compute: Option<&ComputeClient>,
        device_id: &str,
        primal_id: &str,
    ) -> Result<CapacityResult> {
        debug!(
            "Checking primal capacity: device={}, primal={}",
            device_id, primal_id
        );

        if let Some(compute) = compute {
            info!("Compute provider available — checking capacity");

            match compute
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
                        .and_then(serde_json::Value::as_bool)
                        .unwrap_or(true)
                    {
                        info!("✅ Compute capacity: available");
                        Ok(CapacityResult::Available)
                    } else {
                        let reason = result
                            .get("reason")
                            .and_then(|v| v.as_str())
                            .unwrap_or("Insufficient capacity")
                            .to_string();
                        info!("❌ Compute capacity: insufficient — {}", reason);
                        Ok(CapacityResult::Insufficient { reason })
                    }
                }
                Err(e) => {
                    warn!(
                        "⚠️ Compute provider call failed: {} — falling back to available",
                        e
                    );
                    info!("✅ Compute capacity: available (fallback)");
                    Ok(CapacityResult::Available)
                }
            }
        } else {
            warn!("⚠️ No compute provider available");
            warn!("⚠️ Allowing assignment without capacity check (graceful degradation)");
            info!("✅ Capacity: available (no compute provider)");
            Ok(CapacityResult::Available)
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

    async fn spawn_mock_compute(
        available: bool,
        reason: Option<&str>,
    ) -> (String, tokio::task::JoinHandle<()>) {
        let dir = tempfile::tempdir().unwrap();
        let socket_path = dir.path().join("compute.sock");
        let path_str = socket_path.to_str().unwrap().to_string();

        let capacity_response = if available {
            serde_json::json!({"available": true})
        } else {
            serde_json::json!({"available": false, "reason": reason.unwrap_or("Insufficient capacity")})
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
                        result: Some(capacity_response),
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
    async fn test_capacity_no_compute() {
        let result = Capacity::check_primal_capacity(None, "test-device", "test-primal").await;

        assert!(result.is_ok());
        assert!(matches!(result.unwrap(), CapacityResult::Available));
    }

    #[tokio::test]
    async fn test_capacity_no_compute_graceful_degradation() {
        // Capacity check passes by default when no compute provider is available
        let result =
            Capacity::check_primal_capacity(None, "device-abc-123", "primal-xyz-456").await;

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
            CapacityResult::Available => panic!("Expected Insufficient result"),
        }
    }

    #[test]
    fn test_capacity_result_debug() {
        let available = CapacityResult::Available;
        let insufficient = CapacityResult::Insufficient {
            reason: "test reason".to_string(),
        };

        // Test Debug trait
        assert!(format!("{available:?}").contains("Available"));
        assert!(format!("{insufficient:?}").contains("Insufficient"));
        assert!(format!("{insufficient:?}").contains("test reason"));
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

    #[tokio::test]
    async fn test_capacity_compute_insufficient() {
        let (path, _handle) = spawn_mock_compute(false, Some("Not enough memory")).await;
        let client = Some(crate::primal_client::ComputeClient::with_socket(
            "compute", &path,
        ));
        let result =
            Capacity::check_primal_capacity(client.as_ref(), "test-device", "test-primal").await;

        assert!(result.is_ok());
        match result.unwrap() {
            CapacityResult::Insufficient { reason } => assert_eq!(reason, "Not enough memory"),
            CapacityResult::Available => panic!("Expected Insufficient"),
        }
    }

    #[tokio::test]
    async fn test_capacity_compute_available() {
        let (path, _handle) = spawn_mock_compute(true, None).await;
        let client = Some(crate::primal_client::ComputeClient::with_socket(
            "compute", &path,
        ));
        let result =
            Capacity::check_primal_capacity(client.as_ref(), "test-device", "test-primal").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), CapacityResult::Available);
    }

    #[tokio::test]
    async fn test_capacity_compute_call_fails_fallback() {
        let client = Some(crate::primal_client::ComputeClient::with_socket(
            "compute",
            "/nonexistent/compute.sock",
        ));
        let result =
            Capacity::check_primal_capacity(client.as_ref(), "test-device", "test-primal").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), CapacityResult::Available);
    }
}
