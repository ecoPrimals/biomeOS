// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Motion Capture Adapter
//!
//! Provides 6DoF tracking integration between biomeOS graph nodes and
//! petalTongue's hardware tracking backend (OpenXR, SteamVR, or custom).
//!
//! ## Protocol
//!
//! Motion capture uses JSON-RPC over Unix socket:
//! - `xr.start_tracking` — begin tracking with a MotionCaptureConfig
//! - `xr.get_tracking_frame` — poll the latest TrackingFrame
//! - `xr.calibrate_tracking` — trigger device calibration
//! - `xr.stop_tracking` — stop tracking and release devices
//!
//! ## Integration
//!
//! In a continuous graph, the input node calls `poll_frame()` each tick
//! to get the latest device poses, which are then routed to game-logic,
//! physics, or surgical simulation nodes via the sensor event bus.

use crate::primal_client::UiClient;
use anyhow::Result;
use biomeos_types::xr::{MotionCaptureConfig, Pose6DoF, TrackedDeviceType, TrackingFrame};
use tracing::{debug, info, warn};

/// Motion capture adapter for petalTongue hardware tracking.
///
/// Manages the lifecycle of a tracking session:
/// 1. Configure backend and tracked devices
/// 2. Start tracking
/// 3. Poll frames each tick (1000 Hz internally, downsampled to graph tick rate)
/// 4. Stop tracking
pub struct MotionCaptureAdapter {
    config: MotionCaptureConfig,
    tracking_active: bool,
    frame_count: u64,
}

/// Calibration result from a tracking system.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct CalibrationResult {
    /// Whether calibration succeeded
    pub success: bool,
    /// Residual tracking error in mm (lower is better)
    pub residual_mm: f64,
    /// Number of calibration samples used
    pub samples: u32,
    /// Human-readable status message
    pub message: String,
}

impl MotionCaptureAdapter {
    /// Create a new adapter with the given configuration.
    #[must_use]
    pub const fn new(config: MotionCaptureConfig) -> Self {
        Self {
            config,
            tracking_active: false,
            frame_count: 0,
        }
    }

    /// Create an adapter with default OpenXR configuration (head + two hands).
    #[must_use]
    pub fn with_defaults() -> Self {
        Self::new(MotionCaptureConfig::default())
    }

    /// Create a surgical tracking adapter (head + two hands + tool).
    #[must_use]
    pub fn for_surgical() -> Self {
        Self::new(MotionCaptureConfig {
            backend: "openxr".to_string(),
            tracking_hz: 1000,
            tracked_devices: vec![
                TrackedDeviceType::Head,
                TrackedDeviceType::LeftHand,
                TrackedDeviceType::RightHand,
                TrackedDeviceType::Tool,
            ],
            prediction_ms: 5.0, // tighter prediction for surgical precision
        })
    }

    /// Start tracking on petalTongue.
    pub async fn start_tracking(&mut self, ui: &UiClient) -> Result<()> {
        if self.tracking_active {
            warn!("Tracking already active");
            return Ok(());
        }

        let device_names: Vec<String> = self
            .config
            .tracked_devices
            .iter()
            .map(|d| format!("{d:?}").to_lowercase())
            .collect();

        let params = serde_json::json!({
            "backend": self.config.backend,
            "tracking_hz": self.config.tracking_hz,
            "tracked_devices": device_names,
            "prediction_ms": self.config.prediction_ms,
        });

        ui.call("xr.start_tracking", params).await?;
        self.tracking_active = true;
        self.frame_count = 0;
        info!(
            "Tracking started: {} @ {}Hz, {} devices",
            self.config.backend,
            self.config.tracking_hz,
            self.config.tracked_devices.len()
        );
        Ok(())
    }

    /// Poll the latest tracking frame from petalTongue.
    ///
    /// Returns `None` if tracking is not active or no frame is available.
    pub async fn poll_frame(&mut self, ui: &UiClient) -> Result<Option<TrackingFrame>> {
        if !self.tracking_active {
            return Ok(None);
        }

        let result = ui
            .call("xr.get_tracking_frame", serde_json::json!({}))
            .await;

        match result {
            Ok(value) => {
                let frame: TrackingFrame = serde_json::from_value(value)?;
                self.frame_count += 1;
                debug!(
                    "Tracking frame {} (confidence: {:.2})",
                    frame.frame, frame.confidence
                );
                Ok(Some(frame))
            }
            Err(e) => {
                warn!("Failed to poll tracking frame: {}", e);
                Ok(None)
            }
        }
    }

    /// Trigger tracking calibration.
    pub async fn calibrate(&self, ui: &UiClient) -> Result<CalibrationResult> {
        info!("Starting tracking calibration");
        let result = ui
            .call("xr.calibrate_tracking", serde_json::json!({}))
            .await?;
        let cal: CalibrationResult = serde_json::from_value(result)?;
        info!(
            "Calibration {}: residual={:.2}mm",
            if cal.success { "succeeded" } else { "failed" },
            cal.residual_mm
        );
        Ok(cal)
    }

    /// Stop tracking and release devices.
    pub async fn stop_tracking(&mut self, ui: &UiClient) -> Result<()> {
        if !self.tracking_active {
            return Ok(());
        }

        ui.call("xr.stop_tracking", serde_json::json!({})).await?;
        self.tracking_active = false;
        info!("Tracking stopped after {} frames", self.frame_count);
        Ok(())
    }

    /// Whether tracking is currently active.
    #[must_use]
    pub const fn is_tracking_active(&self) -> bool {
        self.tracking_active
    }

    /// Total frames polled since tracking started.
    #[must_use]
    pub const fn frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Get the current configuration.
    #[must_use]
    pub const fn config(&self) -> &MotionCaptureConfig {
        &self.config
    }

    /// Extract a specific device pose from a tracking frame.
    #[must_use]
    pub fn get_device_pose(frame: &TrackingFrame, device: &str) -> Option<Pose6DoF> {
        frame.devices.get(device).copied()
    }

    #[cfg(test)]
    pub(crate) fn set_tracking_active_for_test(&mut self, v: bool) {
        self.tracking_active = v;
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use biomeos_test_utils::ready_signal;
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    #[test]
    fn test_adapter_defaults() {
        let adapter = MotionCaptureAdapter::with_defaults();
        assert!(!adapter.is_tracking_active());
        assert_eq!(adapter.frame_count(), 0);
        assert_eq!(adapter.config().backend, "openxr");
        assert_eq!(adapter.config().tracked_devices.len(), 3);
    }

    #[test]
    fn test_surgical_adapter() {
        let adapter = MotionCaptureAdapter::for_surgical();
        assert_eq!(adapter.config().tracked_devices.len(), 4);
        assert!(
            adapter
                .config()
                .tracked_devices
                .contains(&TrackedDeviceType::Tool)
        );
        assert!((adapter.config().prediction_ms - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_calibration_result_serde() {
        let cal = CalibrationResult {
            success: true,
            residual_mm: 0.42,
            samples: 256,
            message: "Calibration complete".to_string(),
        };
        let json = serde_json::to_string(&cal).unwrap();
        let back: CalibrationResult = serde_json::from_str(&json).unwrap();
        assert!(back.success);
        assert!((back.residual_mm - 0.42).abs() < 0.001);
    }

    #[test]
    fn test_get_device_pose() {
        let mut devices = std::collections::HashMap::new();
        devices.insert("head".to_string(), Pose6DoF::default());
        let frame = TrackingFrame {
            frame: 1,
            timestamp_us: 16666,
            devices,
            confidence: 0.99,
        };

        assert!(MotionCaptureAdapter::get_device_pose(&frame, "head").is_some());
        assert!(MotionCaptureAdapter::get_device_pose(&frame, "left_hand").is_none());
    }

    #[tokio::test]
    async fn test_poll_frame_when_inactive() {
        let mut adapter = MotionCaptureAdapter::with_defaults();
        let client = crate::primal_client::PrimalClient::with_socket("ui", "/nonexistent.sock");
        let result = adapter.poll_frame(&client).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_stop_tracking_noop_when_inactive() {
        let mut adapter = MotionCaptureAdapter::with_defaults();
        let client = crate::primal_client::PrimalClient::with_socket("ui", "/nonexistent.sock");
        let result = adapter.stop_tracking(&client).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_adapter_new() {
        let config = MotionCaptureConfig {
            backend: "steamvr".to_string(),
            tracking_hz: 90,
            tracked_devices: vec![TrackedDeviceType::Head],
            prediction_ms: 15.0,
        };
        let adapter = MotionCaptureAdapter::new(config);
        assert!(!adapter.is_tracking_active());
        assert_eq!(adapter.frame_count(), 0);
        assert_eq!(adapter.config().backend, "steamvr");
    }

    #[test]
    fn test_calibration_result_failed() {
        let cal = CalibrationResult {
            success: false,
            residual_mm: 2.5,
            samples: 128,
            message: "Tracking lost".to_string(),
        };
        let json = serde_json::to_string(&cal).unwrap();
        let back: CalibrationResult = serde_json::from_str(&json).unwrap();
        assert!(!back.success);
        assert!((back.residual_mm - 2.5).abs() < 0.001);
    }

    #[test]
    fn test_get_device_pose_empty_frame() {
        let frame = TrackingFrame {
            frame: 0,
            timestamp_us: 0,
            devices: std::collections::HashMap::new(),
            confidence: 0.0,
        };
        assert!(MotionCaptureAdapter::get_device_pose(&frame, "head").is_none());
    }

    #[test]
    fn test_get_device_pose_different_key_format() {
        let mut devices = std::collections::HashMap::new();
        devices.insert("left_hand".to_string(), Pose6DoF::default());
        let frame = TrackingFrame {
            frame: 1,
            timestamp_us: 16666,
            devices,
            confidence: 0.95,
        };
        assert!(MotionCaptureAdapter::get_device_pose(&frame, "left_hand").is_some());
        assert!(MotionCaptureAdapter::get_device_pose(&frame, "LeftHand").is_none());
    }

    #[test]
    fn test_motion_capture_config_default() {
        let config = MotionCaptureConfig::default();
        assert_eq!(config.backend, "openxr");
        assert_eq!(config.tracking_hz, 1000);
        assert_eq!(config.tracked_devices.len(), 3);
        assert!((config.prediction_ms - 10.0).abs() < 0.001);
    }

    #[tokio::test]
    async fn test_start_tracking_already_active() {
        let config = MotionCaptureConfig::default();
        let mut adapter = MotionCaptureAdapter::new(config);
        let client = crate::primal_client::PrimalClient::with_socket("ui", "/nonexistent.sock");
        adapter.set_tracking_active_for_test(true);
        let result = adapter.start_tracking(&client).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_calibration_result_debug() {
        let cal = CalibrationResult {
            success: true,
            residual_mm: 0.1,
            samples: 100,
            message: "OK".to_string(),
        };
        let s = format!("{cal:?}");
        assert!(s.contains("success"));
        assert!(s.contains("0.1"));
    }

    #[tokio::test]
    #[ignore = "requires petalTongue socket"]
    async fn test_start_tracking() {
        let mut adapter = MotionCaptureAdapter::with_defaults();
        let client = crate::primal_client::PrimalClient::with_socket("ui", "/tmp/ui.sock");
        let _ = adapter.start_tracking(&client).await;
    }

    #[tokio::test]
    #[ignore = "requires petalTongue socket"]
    async fn test_calibrate() {
        let adapter = MotionCaptureAdapter::with_defaults();
        let client = crate::primal_client::PrimalClient::with_socket("ui", "/tmp/ui.sock");
        let _ = adapter.calibrate(&client).await;
    }

    #[tokio::test]
    async fn test_poll_frame_when_active_but_call_fails_returns_none() {
        let mut adapter = MotionCaptureAdapter::with_defaults();
        adapter.set_tracking_active_for_test(true);
        let client = crate::primal_client::PrimalClient::with_socket("ui", "/nonexistent.sock");
        let result = adapter.poll_frame(&client).await;
        assert!(result.is_ok());
        assert!(result.unwrap().is_none());
    }

    #[tokio::test]
    async fn test_start_tracking_with_nonexistent_socket_returns_err() {
        let mut adapter = MotionCaptureAdapter::with_defaults();
        let client = crate::primal_client::PrimalClient::with_socket("ui", "/nonexistent.sock");
        let result = adapter.start_tracking(&client).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_calibrate_with_nonexistent_socket_returns_err() {
        let adapter = MotionCaptureAdapter::with_defaults();
        let client = crate::primal_client::PrimalClient::with_socket("ui", "/nonexistent.sock");
        let result = adapter.calibrate(&client).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_stop_tracking_when_active_but_call_fails_returns_err() {
        let mut adapter = MotionCaptureAdapter::with_defaults();
        adapter.set_tracking_active_for_test(true);
        let client = crate::primal_client::PrimalClient::with_socket("ui", "/nonexistent.sock");
        let result = adapter.stop_tracking(&client).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_start_tracking_success() {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_path = temp.path().join("ui.sock");
        let path = socket_path.clone();
        let (mut ready_tx, ready_rx) = ready_signal();
        let server = tokio::spawn(async move {
            let listener = tokio::net::UnixListener::bind(&path).expect("bind");
            ready_tx.signal();
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = vec![0u8; 8192];
                let n = stream.read(&mut buf).await.expect("read");
                let req: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse");
                assert_eq!(
                    req.get("method").and_then(|m| m.as_str()),
                    Some("xr.start_tracking")
                );
                let params = req.get("params").expect("params");
                assert_eq!(
                    params.get("backend").and_then(|v| v.as_str()),
                    Some("openxr")
                );
                assert!(params.get("tracked_devices").is_some());
                let resp = serde_json::json!({"jsonrpc":"2.0","result":{},"id":req["id"]});
                let line = format!("{resp}\n");
                stream.write_all(line.as_bytes()).await.expect("write");
                stream.flush().await.expect("flush");
            }
        });
        ready_rx.wait().await.unwrap();
        let mut adapter = MotionCaptureAdapter::with_defaults();
        let client = crate::primal_client::PrimalClient::with_socket("ui", &socket_path);
        adapter.start_tracking(&client).await.expect("start");
        assert!(adapter.is_tracking_active());
        assert_eq!(adapter.frame_count(), 0);
        server.abort();
    }

    #[tokio::test]
    async fn test_poll_frame_success_increments_count() {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_path = temp.path().join("ui.sock");
        let path = socket_path.clone();
        let frame = TrackingFrame {
            frame: 7,
            timestamp_us: 42,
            devices: std::collections::HashMap::new(),
            confidence: 0.88,
        };
        let frame_val = serde_json::to_value(&frame).expect("frame json");
        let (mut ready_tx, ready_rx) = ready_signal();
        let server = tokio::spawn(async move {
            let listener = tokio::net::UnixListener::bind(&path).expect("bind");
            ready_tx.signal();
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = vec![0u8; 8192];
                let n = stream.read(&mut buf).await.expect("read");
                let req: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse");
                assert_eq!(
                    req.get("method").and_then(|m| m.as_str()),
                    Some("xr.get_tracking_frame")
                );
                let resp = serde_json::json!({
                    "jsonrpc": "2.0",
                    "result": frame_val,
                    "id": req["id"]
                });
                let line = format!("{resp}\n");
                stream.write_all(line.as_bytes()).await.expect("write");
                stream.flush().await.expect("flush");
            }
        });
        ready_rx.wait().await.unwrap();
        let mut adapter = MotionCaptureAdapter::with_defaults();
        adapter.set_tracking_active_for_test(true);
        let client = crate::primal_client::PrimalClient::with_socket("ui", &socket_path);
        let out = adapter.poll_frame(&client).await.expect("poll");
        let f = out.expect("some frame");
        assert_eq!(f.frame, 7);
        assert_eq!(adapter.frame_count(), 1);
        server.abort();
    }

    #[tokio::test]
    async fn test_poll_frame_invalid_result_returns_err() {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_path = temp.path().join("ui.sock");
        let path = socket_path.clone();
        let (mut ready_tx, ready_rx) = ready_signal();
        let server = tokio::spawn(async move {
            let listener = tokio::net::UnixListener::bind(&path).expect("bind");
            ready_tx.signal();
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = vec![0u8; 8192];
                let n = stream.read(&mut buf).await.expect("read");
                let req: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse");
                let resp = serde_json::json!({
                    "jsonrpc": "2.0",
                    "result": "bad",
                    "id": req["id"]
                });
                let line = format!("{resp}\n");
                stream.write_all(line.as_bytes()).await.expect("write");
                stream.flush().await.expect("flush");
            }
        });
        ready_rx.wait().await.unwrap();
        let mut adapter = MotionCaptureAdapter::with_defaults();
        adapter.set_tracking_active_for_test(true);
        let client = crate::primal_client::PrimalClient::with_socket("ui", &socket_path);
        let err = adapter.poll_frame(&client).await.unwrap_err();
        assert!(!err.to_string().is_empty());
        server.abort();
    }

    #[tokio::test]
    async fn test_calibrate_success() {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_path = temp.path().join("ui.sock");
        let path = socket_path.clone();
        let cal = CalibrationResult {
            success: true,
            residual_mm: 0.12,
            samples: 64,
            message: "ok".to_string(),
        };
        let cal_val = serde_json::to_value(&cal).expect("cal json");
        let (mut ready_tx, ready_rx) = ready_signal();
        let server = tokio::spawn(async move {
            let listener = tokio::net::UnixListener::bind(&path).expect("bind");
            ready_tx.signal();
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = vec![0u8; 8192];
                let n = stream.read(&mut buf).await.expect("read");
                let req: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse");
                assert_eq!(
                    req.get("method").and_then(|m| m.as_str()),
                    Some("xr.calibrate_tracking")
                );
                let resp = serde_json::json!({
                    "jsonrpc": "2.0",
                    "result": cal_val,
                    "id": req["id"]
                });
                let line = format!("{resp}\n");
                stream.write_all(line.as_bytes()).await.expect("write");
                stream.flush().await.expect("flush");
            }
        });
        ready_rx.wait().await.unwrap();
        let adapter = MotionCaptureAdapter::with_defaults();
        let client = crate::primal_client::PrimalClient::with_socket("ui", &socket_path);
        let out = adapter.calibrate(&client).await.expect("calibrate");
        assert!(out.success);
        assert!((out.residual_mm - 0.12).abs() < 1e-9);
        server.abort();
    }

    #[tokio::test]
    async fn test_calibrate_failed_branch_logs_path() {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_path = temp.path().join("ui.sock");
        let path = socket_path.clone();
        let cal = CalibrationResult {
            success: false,
            residual_mm: 9.0,
            samples: 10,
            message: "fail".to_string(),
        };
        let cal_val = serde_json::to_value(&cal).expect("cal json");
        let (mut ready_tx, ready_rx) = ready_signal();
        let server = tokio::spawn(async move {
            let listener = tokio::net::UnixListener::bind(&path).expect("bind");
            ready_tx.signal();
            if let Ok((mut stream, _)) = listener.accept().await {
                let mut buf = vec![0u8; 8192];
                let n = stream.read(&mut buf).await.expect("read");
                let req: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse");
                let resp = serde_json::json!({
                    "jsonrpc": "2.0",
                    "result": cal_val,
                    "id": req["id"]
                });
                let line = format!("{resp}\n");
                stream.write_all(line.as_bytes()).await.expect("write");
                stream.flush().await.expect("flush");
            }
        });
        ready_rx.wait().await.unwrap();
        let adapter = MotionCaptureAdapter::with_defaults();
        let client = crate::primal_client::PrimalClient::with_socket("ui", &socket_path);
        let out = adapter.calibrate(&client).await.expect("calibrate");
        assert!(!out.success);
        server.abort();
    }

    #[tokio::test]
    async fn test_stop_tracking_success_after_start() {
        let temp = tempfile::tempdir().expect("temp dir");
        let socket_path = temp.path().join("ui.sock");
        let path = socket_path.clone();
        let (mut ready_tx, ready_rx) = ready_signal();
        let server = tokio::spawn(async move {
            let listener = tokio::net::UnixListener::bind(&path).expect("bind");
            ready_tx.signal();
            for expect_method in ["xr.start_tracking", "xr.stop_tracking"] {
                if let Ok((mut stream, _)) = listener.accept().await {
                    let mut buf = vec![0u8; 8192];
                    let n = stream.read(&mut buf).await.expect("read");
                    let req: serde_json::Value = serde_json::from_slice(&buf[..n]).expect("parse");
                    assert_eq!(
                        req.get("method").and_then(|m| m.as_str()),
                        Some(expect_method)
                    );
                    let resp = serde_json::json!({"jsonrpc":"2.0","result":{},"id":req["id"]});
                    let line = format!("{resp}\n");
                    stream.write_all(line.as_bytes()).await.expect("write");
                    stream.flush().await.expect("flush");
                }
            }
        });
        ready_rx.wait().await.unwrap();
        let mut adapter = MotionCaptureAdapter::with_defaults();
        let client = crate::primal_client::PrimalClient::with_socket("ui", &socket_path);
        adapter.start_tracking(&client).await.expect("start");
        assert!(adapter.is_tracking_active());
        adapter.stop_tracking(&client).await.expect("stop");
        assert!(!adapter.is_tracking_active());
        server.abort();
    }
}
