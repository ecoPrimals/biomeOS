// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Surgical Domain Models
//!
//! Defines the protocol types for surgical simulation and medical training,
//! consumed by healthSpring (physics + anatomy) and petalTongue (rendering).
//!
//! These models bridge the gap from ludoSpring game interaction to
//! real-time surgical VR by providing:
//! - Surgical procedure definitions
//! - Tool-tissue interaction results
//! - Biosignal stream configuration
//! - Pharmacokinetic model parameters

use serde::{Deserialize, Serialize};

use crate::xr::{Pose6DoF, SurgicalInstrument};

// ---------------------------------------------------------------------------
// Surgical Procedure
// ---------------------------------------------------------------------------

/// A surgical procedure definition for training or simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurgicalProcedure {
    /// Procedure identifier (e.g., "laparoscopic_cholecystectomy")
    pub id: String,
    /// Human-readable name
    pub name: String,
    /// Body region
    pub region: String,
    /// Required instruments
    pub instruments: Vec<SurgicalInstrument>,
    /// Anatomy models to load
    pub anatomy_models: Vec<String>,
    /// Maximum allowed time in seconds (for training scoring)
    pub time_limit_secs: Option<u32>,
    /// Difficulty level (1 = beginner, 5 = expert)
    pub difficulty: u8,
}

// ---------------------------------------------------------------------------
// Tool-Tissue Interaction
// ---------------------------------------------------------------------------

/// Result of a tool interacting with tissue.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolTissueInteraction {
    /// Tool that caused the interaction
    pub instrument: SurgicalInstrument,
    /// Tool pose at interaction time
    pub tool_pose: Pose6DoF,
    /// Tissue layer that was contacted
    pub layer_name: String,
    /// Penetration depth in mm
    pub penetration_mm: f64,
    /// Reaction force in Newtons (fed back to haptic pipeline)
    pub reaction_force: [f64; 3],
    /// Whether tissue was damaged (tearing, cauterization, etc.)
    pub tissue_damaged: bool,
    /// Damage type if applicable
    pub damage_type: Option<DamageType>,
}

/// Types of tissue damage from tool interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DamageType {
    /// Clean cut (scalpel)
    Incision,
    /// Tissue tearing (excessive force)
    Tear,
    /// Thermal damage (cautery)
    Cauterization,
    /// Compression damage (retractor)
    Compression,
    /// Puncture (needle)
    Puncture,
}

// ---------------------------------------------------------------------------
// Biosignal Configuration
// ---------------------------------------------------------------------------

/// Biosignal type for streaming physiological data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BiosignalType {
    /// Electrocardiogram
    Ecg,
    /// Photoplethysmography (pulse oximetry)
    Ppg,
    /// Electrodermal activity (skin conductance)
    Eda,
    /// Electromyography
    Emg,
    /// Blood pressure (systolic/diastolic)
    BloodPressure,
    /// Respiratory rate
    Respiration,
    /// Core body temperature
    Temperature,
}

/// Configuration for a biosignal data stream.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiosignalStreamConfig {
    /// Signal types to stream
    pub signals: Vec<BiosignalType>,
    /// Sample rate in Hz
    pub sample_hz: u32,
    /// Window size in samples for running statistics
    pub window_size: u32,
    /// Whether to apply real-time filtering
    pub filter_enabled: bool,
}

impl Default for BiosignalStreamConfig {
    fn default() -> Self {
        Self {
            signals: vec![BiosignalType::Ecg, BiosignalType::Ppg],
            sample_hz: 250,
            window_size: 1000,
            filter_enabled: true,
        }
    }
}

/// A single biosignal data point.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BiosignalSample {
    /// Signal type
    pub signal: BiosignalType,
    /// Timestamp in microseconds since stream start
    pub timestamp_us: u64,
    /// Raw value
    pub value: f64,
    /// Filtered value (if filtering enabled)
    pub filtered: Option<f64>,
    /// Quality indicator (0.0 = noise, 1.0 = clean)
    pub quality: f64,
}

// ---------------------------------------------------------------------------
// Pharmacokinetic Model
// ---------------------------------------------------------------------------

/// Compartment model type for pharmacokinetics.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CompartmentModel {
    /// Single compartment (rapid distribution)
    OneCompartment,
    /// Two compartment (central + peripheral)
    TwoCompartment,
    /// Three compartment (Marsh or Schnider models for propofol, etc.)
    ThreeCompartment,
}

/// Parameters for a pharmacokinetic simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkModelParams {
    /// Drug identifier
    pub drug_id: String,
    /// Compartment model
    pub model: CompartmentModel,
    /// Patient weight in kg
    pub weight_kg: f64,
    /// Patient age in years
    pub age_years: u32,
    /// Dose in mg
    pub dose_mg: f64,
    /// Infusion rate in mg/hr (0 for bolus)
    pub infusion_rate_mg_hr: f64,
    /// Simulation duration in minutes
    pub duration_min: f64,
    /// Time step in seconds
    pub dt_secs: f64,
}

/// Result of a pharmacokinetic simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PkModelResult {
    /// Time points in minutes
    pub time_min: Vec<f64>,
    /// Plasma concentration (central compartment) in µg/mL
    pub plasma_concentration: Vec<f64>,
    /// Effect-site concentration in µg/mL (if applicable)
    pub effect_site_concentration: Option<Vec<f64>>,
    /// Peak concentration
    pub c_max: f64,
    /// Time to peak in minutes
    pub t_max_min: f64,
    /// Half-life in minutes
    pub half_life_min: f64,
}

// ---------------------------------------------------------------------------
// Surgical Session
// ---------------------------------------------------------------------------

/// State of a surgical simulation session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SurgicalSessionState {
    /// Setting up anatomy and instruments
    Setup,
    /// Active simulation
    Active,
    /// Paused (training review)
    Paused,
    /// Completed (scoring)
    Completed,
    /// Emergency stopped
    EmergencyStopped,
}

/// Summary metrics for a completed surgical session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurgicalSessionMetrics {
    /// Total elapsed time in seconds
    pub elapsed_secs: f64,
    /// Number of instrument changes
    pub instrument_changes: u32,
    /// Number of tissue damage events
    pub tissue_damage_events: u32,
    /// Total path length of dominant hand in mm
    pub hand_path_length_mm: f64,
    /// Economy of motion score (0.0–1.0, higher is better)
    pub economy_of_motion: f64,
    /// Accuracy score (0.0–1.0)
    pub accuracy: f64,
    /// Overall performance score (0.0–100.0)
    pub overall_score: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_surgical_procedure_serde() {
        let proc = SurgicalProcedure {
            id: "lap_chole".to_string(),
            name: "Laparoscopic Cholecystectomy".to_string(),
            region: "abdomen".to_string(),
            instruments: vec![
                SurgicalInstrument::Endoscope,
                SurgicalInstrument::Forceps,
                SurgicalInstrument::Cautery,
            ],
            anatomy_models: vec!["gallbladder".to_string(), "liver".to_string()],
            time_limit_secs: Some(3600),
            difficulty: 3,
        };
        let json = serde_json::to_string(&proc).unwrap();
        let back: SurgicalProcedure = serde_json::from_str(&json).unwrap();
        assert_eq!(back.id, "lap_chole");
        assert_eq!(back.instruments.len(), 3);
    }

    #[test]
    fn test_tool_tissue_interaction_serde() {
        let tti = ToolTissueInteraction {
            instrument: SurgicalInstrument::Scalpel,
            tool_pose: Pose6DoF::default(),
            layer_name: "dermis".to_string(),
            penetration_mm: 2.5,
            reaction_force: [0.0, 0.3, 0.1],
            tissue_damaged: true,
            damage_type: Some(DamageType::Incision),
        };
        let json = serde_json::to_string(&tti).unwrap();
        let back: ToolTissueInteraction = serde_json::from_str(&json).unwrap();
        assert!(back.tissue_damaged);
        assert_eq!(back.damage_type, Some(DamageType::Incision));
    }

    #[test]
    fn test_damage_type_serde() {
        for dt in [
            DamageType::Incision,
            DamageType::Tear,
            DamageType::Cauterization,
            DamageType::Compression,
            DamageType::Puncture,
        ] {
            let json = serde_json::to_string(&dt).unwrap();
            let back: DamageType = serde_json::from_str(&json).unwrap();
            assert_eq!(back, dt);
        }
    }

    #[test]
    fn test_biosignal_stream_config_default() {
        let config = BiosignalStreamConfig::default();
        assert_eq!(config.sample_hz, 250);
        assert!(config.filter_enabled);
        assert_eq!(config.signals.len(), 2);
    }

    #[test]
    fn test_biosignal_stream_config_default_roundtrip() {
        let val = BiosignalStreamConfig::default();
        let json = serde_json::to_string(&val).unwrap();
        let back: BiosignalStreamConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(val.sample_hz, back.sample_hz);
        assert_eq!(val.signals.len(), back.signals.len());
    }

    #[test]
    fn test_biosignal_sample_serde() {
        let sample = BiosignalSample {
            signal: BiosignalType::Ecg,
            timestamp_us: 1000,
            value: 0.85,
            filtered: Some(0.83),
            quality: 0.95,
        };
        let json = serde_json::to_string(&sample).unwrap();
        let back: BiosignalSample = serde_json::from_str(&json).unwrap();
        assert_eq!(back.signal, BiosignalType::Ecg);
        assert!((back.value - 0.85).abs() < 0.001);
    }

    #[test]
    fn test_pk_model_params_serde() {
        let params = PkModelParams {
            drug_id: "propofol".to_string(),
            model: CompartmentModel::ThreeCompartment,
            weight_kg: 70.0,
            age_years: 45,
            dose_mg: 200.0,
            infusion_rate_mg_hr: 0.0,
            duration_min: 60.0,
            dt_secs: 1.0,
        };
        let json = serde_json::to_string(&params).unwrap();
        let back: PkModelParams = serde_json::from_str(&json).unwrap();
        assert_eq!(back.drug_id, "propofol");
        assert_eq!(back.model, CompartmentModel::ThreeCompartment);
    }

    #[test]
    fn test_pk_model_result_serde() {
        let result = PkModelResult {
            time_min: vec![0.0, 1.0, 2.0],
            plasma_concentration: vec![0.0, 5.2, 3.8],
            effect_site_concentration: Some(vec![0.0, 1.1, 3.0]),
            c_max: 5.2,
            t_max_min: 1.0,
            half_life_min: 4.5,
        };
        let json = serde_json::to_string(&result).unwrap();
        let back: PkModelResult = serde_json::from_str(&json).unwrap();
        assert_eq!(back.time_min.len(), 3);
        assert!((back.c_max - 5.2).abs() < 0.001);
    }

    #[test]
    fn test_surgical_session_state_serde() {
        for state in [
            SurgicalSessionState::Setup,
            SurgicalSessionState::Active,
            SurgicalSessionState::Paused,
            SurgicalSessionState::Completed,
            SurgicalSessionState::EmergencyStopped,
        ] {
            let json = serde_json::to_string(&state).unwrap();
            let back: SurgicalSessionState = serde_json::from_str(&json).unwrap();
            assert_eq!(back, state);
        }
    }

    #[test]
    fn test_surgical_session_metrics_serde() {
        let metrics = SurgicalSessionMetrics {
            elapsed_secs: 1234.5,
            instrument_changes: 8,
            tissue_damage_events: 2,
            hand_path_length_mm: 4567.8,
            economy_of_motion: 0.72,
            accuracy: 0.85,
            overall_score: 78.5,
        };
        let json = serde_json::to_string(&metrics).unwrap();
        let back: SurgicalSessionMetrics = serde_json::from_str(&json).unwrap();
        assert_eq!(back.instrument_changes, 8);
        assert!((back.overall_score - 78.5).abs() < 0.01);
    }

    #[test]
    fn test_biosignal_type_serde() {
        for bt in [
            BiosignalType::Ecg,
            BiosignalType::Ppg,
            BiosignalType::Eda,
            BiosignalType::Emg,
            BiosignalType::BloodPressure,
            BiosignalType::Respiration,
            BiosignalType::Temperature,
        ] {
            let json = serde_json::to_string(&bt).unwrap();
            let back: BiosignalType = serde_json::from_str(&json).unwrap();
            assert_eq!(back, bt);
        }
    }

    #[test]
    fn test_compartment_model_serde() {
        for cm in [
            CompartmentModel::OneCompartment,
            CompartmentModel::TwoCompartment,
            CompartmentModel::ThreeCompartment,
        ] {
            let json = serde_json::to_string(&cm).unwrap();
            let back: CompartmentModel = serde_json::from_str(&json).unwrap();
            assert_eq!(back, cm);
        }
    }
}
