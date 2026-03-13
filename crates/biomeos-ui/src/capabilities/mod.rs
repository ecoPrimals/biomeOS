// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Capabilities Module
//!
//! Generic, capability-based services provided by biomeOS.
//! NO primal-specific code - pure TRUE PRIMAL architecture!
//!
//! ## Available Capabilities
//!
//! - `device_management` - Device and primal management for UI primals
//! - `xr_rendering` - Stereo 3D rendering negotiation (VR/AR)
//! - `motion_capture` - 6DoF tracking and motion capture integration
//! - `haptic_feedback` - Haptic device discovery and force feedback dispatch

pub mod device_management;
pub mod haptic_feedback;
pub mod motion_capture;
pub mod xr_rendering;

// Re-export for convenience
pub use device_management::DeviceManagementProvider;
pub use haptic_feedback::HapticPipeline;
pub use motion_capture::MotionCaptureAdapter;
pub use xr_rendering::StereoRenderAdapter;
