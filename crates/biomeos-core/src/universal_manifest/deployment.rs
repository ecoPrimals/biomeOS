//! Deployment Preferences Module

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Deployment strategy options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DeploymentStrategy {
    /// Automatic deployment based on available resources
    Automatic,
    /// Manual deployment with explicit configuration
    Manual,
    /// Hybrid deployment combining automatic and manual
    Hybrid,
    /// Edge-focused deployment
    Edge,
    /// Cloud-focused deployment
    Cloud,
    /// Distributed deployment across multiple locations
    Distributed,
}

/// Deployment preferences
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentPreferences {
    /// Deployment strategy
    pub strategy: DeploymentStrategy,
    /// Primal preferences
    pub primal_preferences: Vec<PrimalPreference>,
    /// Region preferences
    pub region_preferences: Vec<String>,
}

/// Primal preference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalPreference {
    /// Primal type
    pub primal_type: String,
    /// Priority
    pub priority: u32,
    /// Constraints
    pub constraints: HashMap<String, String>,
} 