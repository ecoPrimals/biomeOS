// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! XR Rendering Adapter
//!
//! Provides stereo 3D rendering negotiation and frame submission
//! with petalTongue. This adapter sits between the biomeOS graph executor
//! and petalTongue's rendering pipeline, gating stereo features behind
//! `VisualOutputCapability::ThreeD`.
//!
//! ## Protocol
//!
//! Stereo rendering uses JSON-RPC over Unix socket:
//! - `xr.negotiate_stereo` — exchange StereoConfig, get render target handles
//! - `xr.submit_frame` — submit a completed stereo frame pair
//! - `xr.get_render_targets` — query current render target configuration
//! - `xr.begin_session` / `xr.end_session` — session lifecycle

use crate::primal_client::PetalTongueClient;
use anyhow::Result;
use biomeos_types::xr::{StereoConfig, VisualOutputCapability};
use tracing::{debug, info, warn};

/// Stereo rendering adapter for petalTongue.
///
/// Manages the lifecycle of a stereo 3D rendering session:
/// 1. Negotiate capabilities (resolution, refresh rate, IPD)
/// 2. Begin session (allocate render targets)
/// 3. Submit frames each tick
/// 4. End session (release resources)
pub struct StereoRenderAdapter {
    config: StereoConfig,
    session_active: bool,
}

/// Render target handles returned by petalTongue after negotiation.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RenderTargets {
    /// Left eye render target identifier
    pub left_eye: String,
    /// Right eye render target identifier
    pub right_eye: String,
    /// Negotiated per-eye resolution (may differ from requested)
    pub resolution: (u32, u32),
    /// Negotiated refresh rate
    pub refresh_hz: u32,
}

impl StereoRenderAdapter {
    /// Create a new adapter with the given stereo configuration.
    pub fn new(config: StereoConfig) -> Self {
        Self {
            config,
            session_active: false,
        }
    }

    /// Create an adapter with default Quest-3-class configuration.
    pub fn with_defaults() -> Self {
        Self::new(StereoConfig::default())
    }

    /// Check whether a visual output capability supports stereo rendering.
    pub fn supports_stereo(cap: &VisualOutputCapability) -> bool {
        matches!(cap, VisualOutputCapability::ThreeD(_))
    }

    /// Negotiate stereo rendering with petalTongue.
    ///
    /// Sends the desired `StereoConfig` and receives back the actual
    /// render targets that petalTongue allocated. The returned targets
    /// may have different resolution/refresh if the hardware can't match.
    pub async fn negotiate(&self, petaltongue: &PetalTongueClient) -> Result<RenderTargets> {
        info!(
            "Negotiating stereo: {}x{} @ {}Hz, IPD={}mm",
            self.config.eye_resolution.0,
            self.config.eye_resolution.1,
            self.config.refresh_hz,
            self.config.ipd_mm
        );

        let params = serde_json::json!({
            "eye_resolution": [self.config.eye_resolution.0, self.config.eye_resolution.1],
            "refresh_hz": self.config.refresh_hz,
            "ipd_mm": self.config.ipd_mm,
            "fov_degrees": self.config.fov_degrees,
            "color_format": self.config.color_format,
        });

        let result = petaltongue.call("xr.negotiate_stereo", params).await?;
        let targets: RenderTargets = serde_json::from_value(result)?;

        info!(
            "Stereo negotiated: {}x{} @ {}Hz",
            targets.resolution.0, targets.resolution.1, targets.refresh_hz
        );

        Ok(targets)
    }

    /// Begin a stereo rendering session.
    ///
    /// Must be called after `negotiate()` and before `submit_frame()`.
    pub async fn begin_session(&mut self, petaltongue: &PetalTongueClient) -> Result<()> {
        if self.session_active {
            warn!("Stereo session already active");
            return Ok(());
        }

        let params = serde_json::json!({
            "config": {
                "eye_resolution": [self.config.eye_resolution.0, self.config.eye_resolution.1],
                "refresh_hz": self.config.refresh_hz,
            }
        });

        petaltongue.call("xr.begin_session", params).await?;
        self.session_active = true;
        info!("Stereo session started");
        Ok(())
    }

    /// Submit a completed stereo frame.
    ///
    /// `frame_id` is a monotonic frame counter.
    /// `timestamp_us` is the predicted display time in microseconds.
    pub async fn submit_frame(
        &self,
        petaltongue: &PetalTongueClient,
        frame_id: u64,
        timestamp_us: u64,
    ) -> Result<()> {
        if !self.session_active {
            return Err(anyhow::anyhow!("No active stereo session"));
        }

        debug!("Submitting stereo frame {} @ {}us", frame_id, timestamp_us);

        let params = serde_json::json!({
            "frame_id": frame_id,
            "timestamp_us": timestamp_us,
        });

        petaltongue.call("xr.submit_frame", params).await?;
        Ok(())
    }

    /// End the stereo rendering session and release resources.
    pub async fn end_session(&mut self, petaltongue: &PetalTongueClient) -> Result<()> {
        if !self.session_active {
            return Ok(());
        }

        petaltongue
            .call("xr.end_session", serde_json::json!({}))
            .await?;
        self.session_active = false;
        info!("Stereo session ended");
        Ok(())
    }

    /// Whether a session is currently active.
    pub fn is_session_active(&self) -> bool {
        self.session_active
    }

    /// Get the current stereo configuration.
    pub fn config(&self) -> &StereoConfig {
        &self.config
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    #[test]
    fn test_supports_stereo() {
        assert!(!StereoRenderAdapter::supports_stereo(
            &VisualOutputCapability::TwoD
        ));
        assert!(StereoRenderAdapter::supports_stereo(
            &VisualOutputCapability::ThreeD(StereoConfig::default())
        ));
        assert!(!StereoRenderAdapter::supports_stereo(
            &VisualOutputCapability::Passthrough
        ));
    }

    #[test]
    fn test_adapter_defaults() {
        let adapter = StereoRenderAdapter::with_defaults();
        assert!(!adapter.is_session_active());
        assert_eq!(adapter.config().refresh_hz, 90);
        assert_eq!(adapter.config().eye_resolution, (2064, 2208));
    }

    #[test]
    fn test_adapter_custom_config() {
        let config = StereoConfig {
            eye_resolution: (1440, 1600),
            refresh_hz: 120,
            ipd_mm: 65,
            fov_degrees: 110,
            color_format: "rgba16f".to_string(),
        };
        let adapter = StereoRenderAdapter::new(config);
        assert_eq!(adapter.config().refresh_hz, 120);
        assert_eq!(adapter.config().eye_resolution, (1440, 1600));
    }

    #[test]
    fn test_render_targets_serde() {
        let targets = RenderTargets {
            left_eye: "rt_left_0".to_string(),
            right_eye: "rt_right_0".to_string(),
            resolution: (2064, 2208),
            refresh_hz: 90,
        };
        let json = serde_json::to_string(&targets).unwrap();
        let back: RenderTargets = serde_json::from_str(&json).unwrap();
        assert_eq!(back.left_eye, "rt_left_0");
        assert_eq!(back.resolution, (2064, 2208));
    }

    #[tokio::test]
    async fn test_submit_frame_without_session() {
        let adapter = StereoRenderAdapter::with_defaults();
        let client =
            crate::primal_client::PrimalClient::with_socket("petaltongue", "/nonexistent.sock");
        let result = adapter.submit_frame(&client, 0, 0).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("No active stereo"));
    }

    #[tokio::test]
    async fn test_end_session_noop_when_inactive() {
        let mut adapter = StereoRenderAdapter::with_defaults();
        let client =
            crate::primal_client::PrimalClient::with_socket("petaltongue", "/nonexistent.sock");
        let result = adapter.end_session(&client).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_stereo_config_default() {
        let config = StereoConfig::default();
        assert_eq!(config.eye_resolution, (2064, 2208));
        assert_eq!(config.refresh_hz, 90);
        assert_eq!(config.ipd_mm, 63);
    }

    #[test]
    fn test_adapter_new_custom() {
        let config = StereoConfig {
            eye_resolution: (1920, 1080),
            refresh_hz: 72,
            ipd_mm: 64,
            fov_degrees: 90,
            color_format: "rgba8".to_string(),
        };
        let adapter = StereoRenderAdapter::new(config);
        assert!(!adapter.is_session_active());
        assert_eq!(adapter.config().eye_resolution, (1920, 1080));
        assert_eq!(adapter.config().refresh_hz, 72);
    }

    #[test]
    fn test_supports_stereo_three_d_with_config() {
        let config = StereoConfig::default();
        let cap = VisualOutputCapability::ThreeD(config);
        assert!(StereoRenderAdapter::supports_stereo(&cap));
    }

    #[test]
    fn test_supports_stereo_two_d() {
        assert!(!StereoRenderAdapter::supports_stereo(
            &VisualOutputCapability::TwoD
        ));
    }

    #[test]
    fn test_supports_stereo_passthrough() {
        assert!(!StereoRenderAdapter::supports_stereo(
            &VisualOutputCapability::Passthrough
        ));
    }

    #[test]
    fn test_render_targets_debug() {
        let targets = RenderTargets {
            left_eye: "left".to_string(),
            right_eye: "right".to_string(),
            resolution: (1920, 1080),
            refresh_hz: 90,
        };
        let debug_str = format!("{targets:?}");
        assert!(debug_str.contains("left"));
        assert!(debug_str.contains("1920"));
    }

    #[tokio::test]
    async fn test_begin_session_already_active() {
        let config = StereoConfig::default();
        let mut adapter = StereoRenderAdapter::new(config);
        adapter.session_active = true;
        let client =
            crate::primal_client::PrimalClient::with_socket("petaltongue", "/nonexistent.sock");
        let result = adapter.begin_session(&client).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    #[ignore = "requires petalTongue socket"]
    async fn test_negotiate() {
        let adapter = StereoRenderAdapter::with_defaults();
        let client =
            crate::primal_client::PrimalClient::with_socket("petaltongue", "/tmp/petaltongue.sock");
        let _ = adapter.negotiate(&client).await;
    }

    #[tokio::test]
    #[ignore = "requires petalTongue socket"]
    async fn test_begin_session_and_submit_frame() {
        let mut adapter = StereoRenderAdapter::with_defaults();
        let client =
            crate::primal_client::PrimalClient::with_socket("petaltongue", "/tmp/petaltongue.sock");
        let _ = adapter.begin_session(&client).await;
        let _ = adapter.submit_frame(&client, 1, 16666).await;
    }

    #[tokio::test]
    async fn test_negotiate_with_nonexistent_socket_returns_err() {
        let adapter = StereoRenderAdapter::with_defaults();
        let client =
            crate::primal_client::PrimalClient::with_socket("petaltongue", "/nonexistent.sock");
        let result = adapter.negotiate(&client).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_begin_session_with_nonexistent_socket_returns_err() {
        let mut adapter = StereoRenderAdapter::with_defaults();
        let client =
            crate::primal_client::PrimalClient::with_socket("petaltongue", "/nonexistent.sock");
        let result = adapter.begin_session(&client).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_end_session_when_active_but_call_fails() {
        let mut adapter = StereoRenderAdapter::with_defaults();
        adapter.session_active = true;
        let client =
            crate::primal_client::PrimalClient::with_socket("petaltongue", "/nonexistent.sock");
        let result = adapter.end_session(&client).await;
        assert!(result.is_err());
        assert!(adapter.is_session_active());
    }
}
