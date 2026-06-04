// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

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

use crate::primal_client::UiClient;
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
    #[must_use]
    pub const fn new() -> Self {
        Self {
            devices: Vec::new(),
            active: false,
        }
    }

    /// Discover all available haptic devices from petalTongue.
    pub async fn discover(&mut self, ui: &UiClient) -> Result<&[HapticDeviceCapabilities]> {
        info!("Discovering haptic devices");

        let result = ui.call("xr.discover_haptic", serde_json::json!({})).await?;

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
    pub async fn send_command(&self, ui: &UiClient, mut command: HapticCommand) -> Result<()> {
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
        ui.call("xr.send_haptic", params).await?;
        debug!(
            "Haptic sent: {:?} intensity={:.2} duration={}ms",
            command.device, command.intensity, command.duration_ms
        );
        Ok(())
    }

    /// Emergency stop all haptic output.
    pub async fn stop_all(&self, ui: &UiClient) -> Result<()> {
        if !self.active {
            return Ok(());
        }

        ui.call("xr.stop_haptic", serde_json::json!({})).await?;
        info!("All haptic output stopped");
        Ok(())
    }

    /// Get discovered device capabilities.
    #[must_use]
    pub fn devices(&self) -> &[HapticDeviceCapabilities] {
        &self.devices
    }

    /// Whether the pipeline has been initialized via `discover()`.
    #[must_use]
    pub const fn is_active(&self) -> bool {
        self.active
    }

    /// Find capabilities for a specific device type.
    #[must_use]
    pub fn find_device(&self, device_type: HapticDeviceType) -> Option<&HapticDeviceCapabilities> {
        self.devices.iter().find(|d| d.device_type == device_type)
    }

    /// Check if a force feedback device is available.
    #[must_use]
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
#[path = "haptic_feedback_tests.rs"]
mod tests;
