// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Extended Reality (XR) types for VR, AR, and immersive systems.
//!
//! Provides the type-safe foundation for:
//! - Stereo 3D rendering configuration
//! - Motion capture and 6DoF tracking
//! - Haptic feedback devices
//! - Surgical/medical tool tracking
//!
//! These types are consumed by petalTongue (rendering), ludoSpring (interaction),
//! and healthSpring (surgical simulation).

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Visual Output Capabilities
// ---------------------------------------------------------------------------

/// Visual output capability describing the rendering target.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[derive(Default)]
pub enum VisualOutputCapability {
    /// Standard 2D rendering (single viewport)
    #[default]
    TwoD,
    /// Stereoscopic 3D rendering (dual viewport for VR/AR headsets)
    ThreeD(StereoConfig),
    /// Passthrough AR (camera feed + overlay)
    Passthrough,
}

/// Configuration for stereoscopic 3D rendering.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StereoConfig {
    /// Per-eye resolution (width, height)
    pub eye_resolution: (u32, u32),
    /// Target refresh rate in Hz
    pub refresh_hz: u32,
    /// Interpupillary distance in millimeters
    pub ipd_mm: u32,
    /// Field of view in degrees (horizontal)
    pub fov_degrees: u32,
    /// Color format (e.g., "rgba8", "rgba16f")
    pub color_format: String,
}

impl Default for StereoConfig {
    fn default() -> Self {
        Self {
            eye_resolution: (2064, 2208),
            refresh_hz: 90,
            ipd_mm: 63,
            fov_degrees: 100,
            color_format: "rgba8".to_string(),
        }
    }
}

// ---------------------------------------------------------------------------
// Motion Capture / Tracking
// ---------------------------------------------------------------------------

/// Tracked device type in a motion capture system.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TrackedDeviceType {
    /// Head-mounted display (6DoF)
    Head,
    /// Left hand / controller
    LeftHand,
    /// Right hand / controller
    RightHand,
    /// Surgical tool / instrument
    Tool,
    /// Reference tracker (lighthouse, base station)
    Reference,
    /// Custom tracked object
    Custom,
}

/// 6DoF pose (position + orientation).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Pose6DoF {
    /// Position in meters (x, y, z)
    pub position: [f64; 3],
    /// Orientation as quaternion (x, y, z, w)
    pub orientation: [f64; 4],
    /// Linear velocity in m/s (optional)
    pub velocity: Option<[f64; 3]>,
    /// Angular velocity in rad/s (optional)
    pub angular_velocity: Option<[f64; 3]>,
}

impl Default for Pose6DoF {
    fn default() -> Self {
        Self {
            position: [0.0, 0.0, 0.0],
            orientation: [0.0, 0.0, 0.0, 1.0], // identity quaternion
            velocity: None,
            angular_velocity: None,
        }
    }
}

/// Tracking frame containing poses for all tracked devices.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrackingFrame {
    /// Frame number (monotonic)
    pub frame: u64,
    /// Timestamp in microseconds since session start
    pub timestamp_us: u64,
    /// Poses keyed by device type
    pub devices: std::collections::HashMap<String, Pose6DoF>,
    /// Tracking confidence (0.0 = lost, 1.0 = full confidence)
    pub confidence: f64,
}

/// Configuration for a motion capture adapter.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotionCaptureConfig {
    /// Tracking backend (e.g., "openxr", "steamvr", "custom")
    pub backend: String,
    /// Target tracking rate in Hz
    pub tracking_hz: u32,
    /// Devices to track
    pub tracked_devices: Vec<TrackedDeviceType>,
    /// Prediction horizon in ms (for motion-to-photon latency reduction)
    pub prediction_ms: f64,
}

impl Default for MotionCaptureConfig {
    fn default() -> Self {
        Self {
            backend: "openxr".to_string(),
            tracking_hz: 1000,
            tracked_devices: vec![
                TrackedDeviceType::Head,
                TrackedDeviceType::LeftHand,
                TrackedDeviceType::RightHand,
            ],
            prediction_ms: 10.0,
        }
    }
}

// ---------------------------------------------------------------------------
// Haptic Feedback
// ---------------------------------------------------------------------------

/// Haptic feedback device type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HapticDeviceType {
    /// Controller rumble motor
    Rumble,
    /// Precision haptic actuator (e.g., HD haptics)
    PrecisionActuator,
    /// Force feedback device (surgical tools, steering wheels)
    ForceFeedback,
    /// Electrotactile device
    Electrotactile,
}

/// A haptic feedback command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HapticCommand {
    /// Target device
    pub device: HapticDeviceType,
    /// Target hand/location
    pub target: TrackedDeviceType,
    /// Intensity (0.0 = off, 1.0 = maximum)
    pub intensity: f64,
    /// Duration in milliseconds
    pub duration_ms: u32,
    /// Frequency in Hz (for vibration-based haptics)
    pub frequency_hz: Option<f64>,
    /// Force vector in Newtons (for force feedback devices)
    pub force_vector: Option<[f64; 3]>,
}

/// Haptic device capabilities reported during discovery.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HapticDeviceCapabilities {
    /// Device type
    pub device_type: HapticDeviceType,
    /// Maximum force in Newtons (for force feedback)
    pub max_force_n: Option<f64>,
    /// Maximum frequency in Hz (for vibration)
    pub max_frequency_hz: Option<f64>,
    /// Degrees of freedom for force feedback
    pub force_dof: u8,
    /// Update rate in Hz
    pub update_hz: u32,
}

// ---------------------------------------------------------------------------
// Medical / Surgical Domain Types
// ---------------------------------------------------------------------------

/// Surgical instrument type for tool simulation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurgicalInstrument {
    /// Scalpel / cutting tool
    Scalpel,
    /// Forceps / grasping tool
    Forceps,
    /// Endoscopic camera
    Endoscope,
    /// Cautery / electrosurgical device
    Cautery,
    /// Needle driver (suturing)
    NeedleDriver,
    /// Retractor
    Retractor,
    /// Custom instrument
    Custom,
}

/// Tissue material properties for physics simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TissueMaterial {
    /// Material identifier (e.g., "`liver_parenchyma`", "`skin_dermis`")
    pub id: String,
    /// Young's modulus in kPa
    pub youngs_modulus_kpa: f64,
    /// Poisson's ratio (0.0–0.5)
    pub poissons_ratio: f64,
    /// Density in kg/m^3
    pub density_kg_m3: f64,
    /// Damping coefficient
    pub damping: f64,
    /// Maximum stress before tearing (kPa)
    pub tear_threshold_kpa: Option<f64>,
}

impl Default for TissueMaterial {
    fn default() -> Self {
        Self {
            id: "soft_tissue".to_string(),
            youngs_modulus_kpa: 5.0,
            poissons_ratio: 0.45,
            density_kg_m3: 1060.0,
            damping: 0.1,
            tear_threshold_kpa: Some(100.0),
        }
    }
}

/// Anatomy model definition for rendering and interaction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnatomyModel {
    /// Model identifier
    pub id: String,
    /// Body region (e.g., "abdomen", "thorax", "head")
    pub region: String,
    /// Mesh file path or URI
    pub mesh_uri: String,
    /// Tissue layers with material properties
    pub layers: Vec<AnatomyLayer>,
}

/// A layer in an anatomy model (skin, fascia, muscle, organ, etc.).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnatomyLayer {
    /// Layer name
    pub name: String,
    /// Depth from surface in mm
    pub depth_mm: f64,
    /// Material properties
    pub material: TissueMaterial,
    /// Whether this layer is currently visible in rendering
    pub visible: bool,
}

#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;

    fn assert_f64_array_eq<const N: usize>(actual: &[f64; N], expected: &[f64; N]) {
        for (a, e) in actual.iter().zip(expected.iter()) {
            assert!((a - e).abs() < f64::EPSILON, "expected {e}, got {a}");
        }
    }

    #[test]
    fn test_visual_output_default() {
        let cap = VisualOutputCapability::default();
        assert_eq!(cap, VisualOutputCapability::TwoD);
    }

    #[test]
    fn test_stereo_config_default() {
        let config = StereoConfig::default();
        assert_eq!(config.refresh_hz, 90);
        assert_eq!(config.eye_resolution, (2064, 2208));
    }

    #[test]
    fn test_visual_output_3d_serde() {
        let cap = VisualOutputCapability::ThreeD(StereoConfig::default());
        let json = serde_json::to_string(&cap).unwrap();
        assert!(json.contains("three_d"));
        let back: VisualOutputCapability = serde_json::from_str(&json).unwrap();
        assert!(matches!(back, VisualOutputCapability::ThreeD(_)));
    }

    #[test]
    fn test_pose6dof_default() {
        let pose = Pose6DoF::default();
        assert_f64_array_eq(&pose.position, &[0.0, 0.0, 0.0]);
        assert_f64_array_eq(&pose.orientation, &[0.0, 0.0, 0.0, 1.0]);
        assert!(pose.velocity.is_none());
    }

    #[test]
    fn test_pose6dof_serde() {
        let pose = Pose6DoF {
            position: [1.0, 2.0, 3.0],
            orientation: [0.0, 0.707, 0.0, 0.707],
            velocity: Some([0.1, 0.0, 0.0]),
            angular_velocity: None,
        };
        let json = serde_json::to_string(&pose).unwrap();
        let back: Pose6DoF = serde_json::from_str(&json).unwrap();
        assert_f64_array_eq(&back.position, &pose.position);
    }

    #[test]
    fn test_motion_capture_config_default() {
        let config = MotionCaptureConfig::default();
        assert_eq!(config.backend, "openxr");
        assert_eq!(config.tracking_hz, 1000);
        assert_eq!(config.tracked_devices.len(), 3);
    }

    #[test]
    fn test_haptic_command_serde() {
        let cmd = HapticCommand {
            device: HapticDeviceType::ForceFeedback,
            target: TrackedDeviceType::RightHand,
            intensity: 0.5,
            duration_ms: 100,
            frequency_hz: None,
            force_vector: Some([0.0, -1.0, 0.0]),
        };
        let json = serde_json::to_string(&cmd).unwrap();
        let back: HapticCommand = serde_json::from_str(&json).unwrap();
        assert!((back.intensity - 0.5).abs() < f64::EPSILON);
        assert_f64_array_eq(back.force_vector.as_ref().unwrap(), &[0.0, -1.0, 0.0]);
    }

    #[test]
    fn test_tissue_material_default() {
        let mat = TissueMaterial::default();
        assert_eq!(mat.id, "soft_tissue");
        assert!((mat.youngs_modulus_kpa - 5.0).abs() < 0.01);
        assert!(mat.tear_threshold_kpa.is_some());
    }

    #[test]
    fn test_anatomy_model_serde() {
        let model = AnatomyModel {
            id: "liver".to_string(),
            region: "abdomen".to_string(),
            mesh_uri: "models/liver.glb".to_string(),
            layers: vec![AnatomyLayer {
                name: "parenchyma".to_string(),
                depth_mm: 0.0,
                material: TissueMaterial::default(),
                visible: true,
            }],
        };
        let json = serde_json::to_string(&model).unwrap();
        let back: AnatomyModel = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "liver");
        assert_eq!(back.layers.len(), 1);
    }

    #[test]
    fn test_surgical_instrument_serde() {
        let instruments = vec![
            SurgicalInstrument::Scalpel,
            SurgicalInstrument::Forceps,
            SurgicalInstrument::Endoscope,
            SurgicalInstrument::Cautery,
            SurgicalInstrument::NeedleDriver,
            SurgicalInstrument::Retractor,
            SurgicalInstrument::Custom,
        ];
        for instr in instruments {
            let json = serde_json::to_string(&instr).unwrap();
            let back: SurgicalInstrument = serde_json::from_str(&json).unwrap();
            assert_eq!(back, instr);
        }
    }

    #[test]
    fn test_tracked_device_type_serde() {
        let devices = vec![
            TrackedDeviceType::Head,
            TrackedDeviceType::LeftHand,
            TrackedDeviceType::RightHand,
            TrackedDeviceType::Tool,
            TrackedDeviceType::Reference,
            TrackedDeviceType::Custom,
        ];
        for dev in devices {
            let json = serde_json::to_string(&dev).unwrap();
            let back: TrackedDeviceType = serde_json::from_str(&json).unwrap();
            assert_eq!(back, dev);
        }
    }

    #[test]
    fn test_haptic_device_type_serde() {
        let types = vec![
            HapticDeviceType::Rumble,
            HapticDeviceType::PrecisionActuator,
            HapticDeviceType::ForceFeedback,
            HapticDeviceType::Electrotactile,
        ];
        for t in types {
            let json = serde_json::to_string(&t).unwrap();
            let back: HapticDeviceType = serde_json::from_str(&json).unwrap();
            assert_eq!(back, t);
        }
    }

    #[test]
    fn test_tracking_frame_serde() {
        let mut devices = std::collections::HashMap::new();
        devices.insert("head".to_string(), Pose6DoF::default());
        devices.insert(
            "left_hand".to_string(),
            Pose6DoF {
                position: [-0.3, 1.0, -0.2],
                ..Default::default()
            },
        );

        let frame = TrackingFrame {
            frame: 42,
            timestamp_us: 700_000,
            devices,
            confidence: 0.95,
        };
        let json = serde_json::to_string(&frame).unwrap();
        let back: TrackingFrame = serde_json::from_str(&json).unwrap();
        assert_eq!(back.frame, 42);
        assert_eq!(back.devices.len(), 2);
    }

    #[test]
    fn test_haptic_device_capabilities_serde() {
        let caps = HapticDeviceCapabilities {
            device_type: HapticDeviceType::ForceFeedback,
            max_force_n: Some(5.0),
            max_frequency_hz: None,
            force_dof: 3,
            update_hz: 1000,
        };
        let json = serde_json::to_string(&caps).unwrap();
        let back: HapticDeviceCapabilities = serde_json::from_str(&json).unwrap();
        assert_eq!(back.force_dof, 3);
        assert_eq!(back.max_force_n, Some(5.0));
    }
}
