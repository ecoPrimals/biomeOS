// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Parameters for a node's capability invocation.
///
/// This is a flexible map that gets validated against
/// the capability's expected parameters.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeParams(HashMap<String, ParamValue>);

impl NodeParams {
    /// Create new empty params.
    #[must_use]
    pub fn new() -> Self {
        Self(HashMap::new())
    }

    /// Get a parameter value.
    #[must_use]
    pub fn get(&self, key: &str) -> Option<&ParamValue> {
        self.0.get(key)
    }

    /// Get a string parameter.
    #[must_use]
    pub fn get_string(&self, key: &str) -> Option<&str> {
        self.0.get(key).and_then(|v| v.as_str())
    }

    /// Get a bool parameter.
    pub fn get_bool(&self, key: &str) -> Option<bool> {
        self.0.get(key).and_then(ParamValue::as_bool)
    }

    /// Get an integer parameter.
    pub fn get_i64(&self, key: &str) -> Option<i64> {
        self.0.get(key).and_then(ParamValue::as_i64)
    }

    /// Insert a parameter.
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<ParamValue>) {
        self.0.insert(key.into(), value.into());
    }

    /// Iterate over parameters.
    pub fn iter(&self) -> impl Iterator<Item = (&String, &ParamValue)> {
        self.0.iter()
    }

    /// Convert to JSON for JSON-RPC call.
    #[must_use]
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::to_value(&self.0).unwrap_or(serde_json::Value::Null)
    }
}

/// A parameter value.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ParamValue {
    /// String value (may contain ${VAR} references)
    String(String),
    /// Integer value
    Integer(i64),
    /// Float value
    Float(f64),
    /// Boolean value
    Bool(bool),
    /// Array of values
    Array(Vec<Self>),
    /// Nested object
    Object(HashMap<String, Self>),
}

impl ParamValue {
    /// Get as string if this is a string.
    #[must_use]
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Self::String(s) => Some(s),
            _ => None,
        }
    }

    /// Get as bool if this is a bool.
    #[must_use]
    pub const fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Get as i64 if this is an integer.
    #[must_use]
    pub const fn as_i64(&self) -> Option<i64> {
        match self {
            Self::Integer(i) => Some(*i),
            _ => None,
        }
    }

    /// Get as array if this is an array.
    #[must_use]
    pub const fn as_array(&self) -> Option<&Vec<Self>> {
        match self {
            Self::Array(a) => Some(a),
            _ => None,
        }
    }
}

impl From<String> for ParamValue {
    fn from(s: String) -> Self {
        Self::String(s)
    }
}

impl From<&str> for ParamValue {
    fn from(s: &str) -> Self {
        Self::String(s.to_string())
    }
}

impl From<i64> for ParamValue {
    fn from(i: i64) -> Self {
        Self::Integer(i)
    }
}

impl From<bool> for ParamValue {
    fn from(b: bool) -> Self {
        Self::Bool(b)
    }
}
