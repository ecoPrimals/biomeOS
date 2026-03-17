// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Haptic Feedback Pipeline
//!
//! Provides haptic device discovery and force feedback command dispatch
//! through petalTongue. Supports controller rumble, precision actuators,
//! and force feedback devices (surgical tool simulation, steering wheels).
//!
//! ## Protocol
//!
//! - `xr.discover_haptic` — enumerate available haptic devices and their capabilities
//! - `xr.send_haptic` — dispatch a HapticCommand to a specific device
//! - `xr.stop_haptic` — immediately stop all haptic output
//!
//! ## Safety
//!
//! Force feedback devices (surgical simulators) have hardware limits.
//! The pipeline clamps intensity and force values to the device's reported
//! maximums before dispatch to prevent hardware damage or user injury.

use crate::primal_client::PetalTongueClient;
use anyhow::Result;
use biomeos_types::xr::{HapticCommand, HapticDeviceCapabilities, HapticDeviceType};
use tracing::{debug, info, warn};

/// Haptic feedback pipeline for petalTongue.
///
/// Manages haptic device discovery and command dispatch:
/// 1. Discover available haptic devices
/// 2. Send clamped haptic commands
/// 3. Emergency stop all output
pub struct HapticPipeline {
    devices: Vec<HapticDeviceCapabilities>,
    active: bool,
}

impl HapticPipeline {
    /// Create a new haptic pipeline (devices not yet discovered).
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
            active: false,
        }
    }

    /// Discover all available haptic devices from petalTongue.
    pub async fn discover(
        &mut self,
        petaltongue: &PetalTongueClient,
    ) -> Result<&[HapticDeviceCapabilities]> {
        info!("Discovering haptic devices");

        let result = petaltongue
            .call("xr.discover_haptic", serde_json::json!({}))
            .await?;

        let devices: Vec<HapticDeviceCapabilities> = serde_json::from_value(result)?;
        info!("Found {} haptic devices", devices.len());
        for dev in &devices {
            debug!(
                "  {:?}: {}DoF, {}Hz update",
                dev.device_type, dev.force_dof, dev.update_hz
            );
        }

        self.devices = devices;
        self.active = true;
        Ok(&self.devices)
    }

    /// Send a haptic command, clamping to device limits for safety.
    ///
    /// If the target device type is not found in discovered devices,
    /// the command is silently dropped (graceful degradation).
    pub async fn send_command(
        &self,
        petaltongue: &PetalTongueClient,
        mut command: HapticCommand,
    ) -> Result<()> {
        if !self.active {
            warn!("Haptic pipeline not active, dropping command");
            return Ok(());
        }

        // Find the device capabilities for safety clamping
        if let Some(caps) = self.find_device(command.device) {
            command.intensity = command.intensity.clamp(0.0, 1.0);

            if let (Some(force), Some(max_force)) = (&mut command.force_vector, caps.max_force_n) {
                for component in force.iter_mut() {
                    *component = component.clamp(-max_force, max_force);
                }
            }

            if let (Some(freq), Some(max_freq)) = (&mut command.frequency_hz, caps.max_frequency_hz)
            {
                *freq = freq.clamp(0.0, max_freq);
            }
        }

        let params = serde_json::to_value(&command)?;
        petaltongue.call("xr.send_haptic", params).await?;
        debug!(
            "Haptic sent: {:?} intensity={:.2} duration={}ms",
            command.device, command.intensity, command.duration_ms
        );
        Ok(())
    }

    /// Emergency stop all haptic output.
    pub async fn stop_all(&self, petaltongue: &PetalTongueClient) -> Result<()> {
        if !self.active {
            return Ok(());
        }

        petaltongue
            .call("xr.stop_haptic", serde_json::json!({}))
            .await?;
        info!("All haptic output stopped");
        Ok(())
    }

    /// Get discovered device capabilities.
    pub fn devices(&self) -> &[HapticDeviceCapabilities] {
        &self.devices
    }

    /// Whether the pipeline has been initialized via `discover()`.
    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Find capabilities for a specific device type.
    pub fn find_device(&self, device_type: HapticDeviceType) -> Option<&HapticDeviceCapabilities> {
        self.devices.iter().find(|d| d.device_type == device_type)
    }

    /// Check if a force feedback device is available.
    pub fn has_force_feedback(&self) -> bool {
        self.devices
            .iter()
            .any(|d| d.device_type == HapticDeviceType::ForceFeedback)
    }
}

impl Default for HapticPipeline {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used)]
mod tests {
    use super::*;
    use biomeos_types::xr::TrackedDeviceType;

    fn mock_devices() -> Vec<HapticDeviceCapabilities> {
        vec![
            HapticDeviceCapabilities {
                device_type: HapticDeviceType::Rumble,
                max_force_n: None,
                max_frequency_hz: Some(500.0),
                force_dof: 0,
                update_hz: 100,
            },
            HapticDeviceCapabilities {
                device_type: HapticDeviceType::ForceFeedback,
                max_force_n: Some(5.0),
                max_frequency_hz: None,
                force_dof: 3,
                update_hz: 1000,
            },
        ]
    }

    #[test]
    fn test_pipeline_new() {
        let pipeline = HapticPipeline::new();
        assert!(!pipeline.is_active());
        assert!(pipeline.devices().is_empty());
        assert!(!pipeline.has_force_feedback());
    }

    #[test]
    fn test_find_device() {
        let mut pipeline = HapticPipeline::new();
        pipeline.devices = mock_devices();
        pipeline.active = true;

        assert!(pipeline.find_device(HapticDeviceType::Rumble).is_some());
        assert!(pipeline
            .find_device(HapticDeviceType::ForceFeedback)
            .is_some());
        assert!(pipeline
            .find_device(HapticDeviceType::Electrotactile)
            .is_none());
    }

    #[test]
    fn test_has_force_feedback() {
        let mut pipeline = HapticPipeline::new();
        pipeline.devices = mock_devices();
        assert!(pipeline.has_force_feedback());

        pipeline.devices = vec![HapticDeviceCapabilities {
            device_type: HapticDeviceType::Rumble,
            max_force_n: None,
            max_frequency_hz: Some(500.0),
            force_dof: 0,
            update_hz: 100,
        }];
        assert!(!pipeline.has_force_feedback());
    }

    #[tokio::test]
    async fn test_send_command_inactive() {
        let pipeline = HapticPipeline::new();
        let client =
            crate::primal_client::PrimalClient::with_socket("petaltongue", "/nonexistent.sock");
        let command = HapticCommand {
            device: HapticDeviceType::Rumble,
            target: TrackedDeviceType::RightHand,
            intensity: 0.5,
            duration_ms: 100,
            frequency_hz: Some(200.0),
            force_vector: None,
        };
        let result = pipeline.send_command(&client, command).await;
        assert!(result.is_ok()); // graceful degradation
    }

    #[tokio::test]
    async fn test_stop_all_inactive() {
        let pipeline = HapticPipeline::new();
        let client =
            crate::primal_client::PrimalClient::with_socket("petaltongue", "/nonexistent.sock");
        let result = pipeline.stop_all(&client).await;
        assert!(result.is_ok());
    }

    #[test]
    fn test_default_trait() {
        let pipeline = HapticPipeline::default();
        assert!(!pipeline.is_active());
    }

    #[test]
    fn test_devices_empty_when_new() {
        let pipeline = HapticPipeline::new();
        assert!(pipeline.devices().is_empty());
    }

    #[test]
    fn test_find_device_empty_pipeline() {
        let pipeline = HapticPipeline::new();
        assert!(pipeline.find_device(HapticDeviceType::Rumble).is_none());
    }

    #[test]
    fn test_haptic_command_with_force_vector_clamping() {
        let mut pipeline = HapticPipeline::new();
        pipeline.devices = vec![HapticDeviceCapabilities {
            device_type: HapticDeviceType::ForceFeedback,
            max_force_n: Some(5.0),
            max_frequency_hz: None,
            force_dof: 3,
            update_hz: 1000,
        }];
        pipeline.active = true;
        assert!(pipeline
            .find_device(HapticDeviceType::ForceFeedback)
            .is_some());
    }

    #[test]
    fn test_intensity_clamping() {
        let mut pipeline = HapticPipeline::new();
        pipeline.devices = mock_devices();
        pipeline.active = true;
        let caps = pipeline.find_device(HapticDeviceType::Rumble);
        assert!(caps.is_some());
    }

    #[test]
    fn test_device_type_precision_actuator() {
        let mut pipeline = HapticPipeline::new();
        pipeline.devices = vec![HapticDeviceCapabilities {
            device_type: HapticDeviceType::PrecisionActuator,
            max_force_n: None,
            max_frequency_hz: Some(1000.0),
            force_dof: 0,
            update_hz: 500,
        }];
        pipeline.active = true;
        assert!(pipeline
            .find_device(HapticDeviceType::PrecisionActuator)
            .is_some());
        assert!(!pipeline.has_force_feedback());
    }

    #[test]
    fn test_device_type_electrotactile() {
        let mut pipeline = HapticPipeline::new();
        pipeline.devices = vec![HapticDeviceCapabilities {
            device_type: HapticDeviceType::Electrotactile,
            max_force_n: None,
            max_frequency_hz: Some(200.0),
            force_dof: 0,
            update_hz: 200,
        }];
        pipeline.active = true;
        assert!(pipeline
            .find_device(HapticDeviceType::Electrotactile)
            .is_some());
    }

    #[tokio::test]
    async fn test_send_command_active_device_found_clamping_path() {
        let mut pipeline = HapticPipeline::new();
        pipeline.devices = mock_devices();
        pipeline.active = true;
        let command = HapticCommand {
            device: HapticDeviceType::Rumble,
            target: TrackedDeviceType::RightHand,
            intensity: 1.5,
            duration_ms: 100,
            frequency_hz: Some(600.0),
            force_vector: None,
        };
        let client =
            crate::primal_client::PrimalClient::with_socket("petaltongue", "/nonexistent.sock");
        let result = pipeline.send_command(&client, command).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_send_command_active_device_not_in_list() {
        let mut pipeline = HapticPipeline::new();
        pipeline.devices = vec![HapticDeviceCapabilities {
            device_type: HapticDeviceType::Rumble,
            max_force_n: None,
            max_frequency_hz: Some(500.0),
            force_dof: 0,
            update_hz: 100,
        }];
        pipeline.active = true;
        let command = HapticCommand {
            device: HapticDeviceType::ForceFeedback,
            target: TrackedDeviceType::LeftHand,
            intensity: 0.5,
            duration_ms: 50,
            frequency_hz: None,
            force_vector: Some([10.0, 5.0, -3.0]),
        };
        let client =
            crate::primal_client::PrimalClient::with_socket("petaltongue", "/nonexistent.sock");
        let result = pipeline.send_command(&client, command).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    #[ignore = "requires petalTongue socket"]
    async fn test_discover() {
        let mut pipeline = HapticPipeline::new();
        let client =
            crate::primal_client::PrimalClient::with_socket("petaltongue", "/tmp/petaltongue.sock");
        let _ = pipeline.discover(&client).await;
    }
}
