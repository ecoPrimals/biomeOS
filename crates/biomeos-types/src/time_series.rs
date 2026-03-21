// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Cross-Spring Time Series Exchange Format
//!
//! Implements the `ecoPrimals/time-series/v1` schema defined in
//! `wateringHole/CROSS_SPRING_DATA_FLOW_STANDARD.md`.
//!
//! Springs exchange time series data through `capability.call` using this
//! canonical format. No format negotiation is required — all springs speak
//! the same schema.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Schema identifier for version 1 of the cross-spring time series format.
pub const SCHEMA_V1: &str = "ecoPrimals/time-series/v1";

/// Cross-spring time series payload.
///
/// Canonical exchange format for time-indexed numeric data flowing between
/// springs via the Neural API `capability.call` mechanism.
///
/// # Required invariant
///
/// `timestamps.len() == values.len()` — enforced by [`CrossSpringTimeSeries::new`]
/// and [`CrossSpringTimeSeries::validate`].
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CrossSpringTimeSeries {
    /// Must be [`SCHEMA_V1`].
    pub schema: String,

    /// Variable name in `snake_case` (e.g. `soil_moisture_vol`, `player_heart_rate`).
    pub variable: String,

    /// SI unit or documented unit string (e.g. `m3/m3`, `bpm`, `dimensionless`).
    pub unit: String,

    /// Origin spring, experiment, and capability. Optional.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source: Option<TimeSeriesSource>,

    /// ISO 8601 UTC timestamps.
    pub timestamps: Vec<DateTime<Utc>>,

    /// `f64` values, same length as `timestamps`.
    pub values: Vec<f64>,

    /// Extensible key-value metadata (location, sensor, etc.).
    #[serde(default, skip_serializing_if = "HashMap::is_empty")]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Origin information for a time series.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct TimeSeriesSource {
    /// Originating spring (e.g. `airSpring`).
    pub spring: String,

    /// Experiment identifier.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub experiment: Option<String>,

    /// Capability that produced this data (e.g. `ecology.water_balance`).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub capability: Option<String>,
}

/// Validation errors for [`CrossSpringTimeSeries`].
#[derive(Debug, Clone, thiserror::Error)]
pub enum TimeSeriesError {
    /// Schema identifier does not match the expected version.
    #[error("schema must be \"{SCHEMA_V1}\", got \"{0}\"")]
    InvalidSchema(String),

    /// `timestamps` and `values` arrays have different lengths.
    #[error("timestamps ({0}) and values ({1}) must have the same length")]
    LengthMismatch(usize, usize),

    /// Variable name is empty.
    #[error("variable name must not be empty")]
    EmptyVariable,

    /// Unit string is empty.
    #[error("unit must not be empty")]
    EmptyUnit,
}

impl CrossSpringTimeSeries {
    /// Create a new time series with validation.
    ///
    /// # Errors
    ///
    /// Returns [`TimeSeriesError`] if the invariants are violated.
    pub fn new(
        variable: impl Into<String>,
        unit: impl Into<String>,
        timestamps: Vec<DateTime<Utc>>,
        values: Vec<f64>,
    ) -> Result<Self, TimeSeriesError> {
        let ts = Self {
            schema: SCHEMA_V1.to_string(),
            variable: variable.into(),
            unit: unit.into(),
            source: None,
            timestamps,
            values,
            metadata: HashMap::new(),
        };
        ts.validate()?;
        Ok(ts)
    }

    /// Attach source provenance.
    #[must_use]
    pub fn with_source(mut self, source: TimeSeriesSource) -> Self {
        self.source = Some(source);
        self
    }

    /// Insert a metadata key-value pair.
    #[must_use]
    pub fn with_metadata(mut self, key: impl Into<String>, value: serde_json::Value) -> Self {
        self.metadata.insert(key.into(), value);
        self
    }

    /// Validate all invariants.
    ///
    /// # Errors
    ///
    /// Returns the first violated invariant.
    pub fn validate(&self) -> Result<(), TimeSeriesError> {
        if self.schema != SCHEMA_V1 {
            return Err(TimeSeriesError::InvalidSchema(self.schema.clone()));
        }
        if self.variable.is_empty() {
            return Err(TimeSeriesError::EmptyVariable);
        }
        if self.unit.is_empty() {
            return Err(TimeSeriesError::EmptyUnit);
        }
        if self.timestamps.len() != self.values.len() {
            return Err(TimeSeriesError::LengthMismatch(
                self.timestamps.len(),
                self.values.len(),
            ));
        }
        Ok(())
    }

    /// Number of data points.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.values.len()
    }

    /// Whether the series contains no data points.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn sample_timestamps() -> Vec<DateTime<Utc>> {
        vec![
            Utc.with_ymd_and_hms(2023, 5, 1, 0, 0, 0).unwrap(),
            Utc.with_ymd_and_hms(2023, 5, 2, 0, 0, 0).unwrap(),
        ]
    }

    #[test]
    fn new_valid() {
        let ts = CrossSpringTimeSeries::new(
            "soil_moisture_vol",
            "m3/m3",
            sample_timestamps(),
            vec![0.32, 0.29],
        )
        .unwrap();
        assert_eq!(ts.schema, SCHEMA_V1);
        assert_eq!(ts.len(), 2);
        assert!(!ts.is_empty());
    }

    #[test]
    fn new_empty_is_valid() {
        let ts = CrossSpringTimeSeries::new("temperature", "K", vec![], vec![]).unwrap();
        assert!(ts.is_empty());
        assert_eq!(ts.len(), 0);
    }

    #[test]
    fn length_mismatch_rejected() {
        let err = CrossSpringTimeSeries::new(
            "soil_moisture_vol",
            "m3/m3",
            sample_timestamps(),
            vec![0.32],
        )
        .unwrap_err();
        assert!(matches!(err, TimeSeriesError::LengthMismatch(2, 1)));
    }

    #[test]
    fn empty_variable_rejected() {
        let err = CrossSpringTimeSeries::new("", "m3/m3", vec![], vec![]).unwrap_err();
        assert!(matches!(err, TimeSeriesError::EmptyVariable));
    }

    #[test]
    fn empty_unit_rejected() {
        let err = CrossSpringTimeSeries::new("temperature", "", vec![], vec![]).unwrap_err();
        assert!(matches!(err, TimeSeriesError::EmptyUnit));
    }

    #[test]
    fn with_source() {
        let ts = CrossSpringTimeSeries::new(
            "soil_moisture_vol",
            "m3/m3",
            sample_timestamps(),
            vec![0.32, 0.29],
        )
        .unwrap()
        .with_source(TimeSeriesSource {
            spring: "airSpring".into(),
            experiment: Some("exp022".into()),
            capability: Some("ecology.water_balance".into()),
        });
        assert_eq!(ts.source.as_ref().unwrap().spring, "airSpring");
    }

    #[test]
    fn with_metadata() {
        let ts = CrossSpringTimeSeries::new(
            "soil_moisture_vol",
            "m3/m3",
            sample_timestamps(),
            vec![0.32, 0.29],
        )
        .unwrap()
        .with_metadata("location", serde_json::json!("field_a"));
        assert_eq!(ts.metadata["location"], "field_a");
    }

    #[test]
    fn serde_roundtrip() {
        let ts = CrossSpringTimeSeries::new(
            "soil_moisture_vol",
            "m3/m3",
            sample_timestamps(),
            vec![0.32, 0.29],
        )
        .unwrap()
        .with_source(TimeSeriesSource {
            spring: "airSpring".into(),
            experiment: Some("exp022".into()),
            capability: Some("ecology.water_balance".into()),
        })
        .with_metadata("sensor", serde_json::json!("tdr_01"));

        let json = serde_json::to_string_pretty(&ts).unwrap();
        let parsed: CrossSpringTimeSeries = serde_json::from_str(&json).unwrap();
        assert_eq!(ts, parsed);
    }

    #[test]
    fn schema_v1_constant() {
        assert_eq!(SCHEMA_V1, "ecoPrimals/time-series/v1");
    }

    #[test]
    fn invalid_schema_rejected() {
        let mut ts = CrossSpringTimeSeries::new("temp", "K", vec![], vec![]).unwrap();
        ts.schema = "wrong/schema".into();
        let err = ts.validate().unwrap_err();
        assert!(matches!(err, TimeSeriesError::InvalidSchema(_)));
    }
}
