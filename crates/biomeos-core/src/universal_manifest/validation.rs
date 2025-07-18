//! Validation Specifications Module

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Validation specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSpec {
    /// Validation rules
    pub rules: Vec<ValidationRule>,
    /// Validation policies
    pub policies: Vec<ValidationPolicy>,
}

/// Validation rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRule {
    /// Rule name
    pub name: String,
    /// Rule type
    pub rule_type: String,
    /// Rule condition
    pub condition: String,
}

/// Validation policy
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationPolicy {
    /// Policy name
    pub name: String,
    /// Policy rules
    pub rules: Vec<String>,
}

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Error field
    pub field: String,
    /// Error severity
    pub severity: ValidationSeverity,
}

/// Validation severity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationSeverity {
    /// Error
    Error,
    /// Warning
    Warning,
    /// Info
    Info,
} 