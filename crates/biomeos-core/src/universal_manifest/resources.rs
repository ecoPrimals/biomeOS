//! Resource Specifications Module
//!
//! This module defines resource specifications for biomes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Global resource specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalResourceSpec {
    /// Resource pools
    pub pools: Vec<ResourcePool>,
    /// Resource quotas
    pub quotas: HashMap<String, ResourceQuota>,
    /// Resource scheduling
    pub scheduling: ResourceScheduling,
}

/// Resource pool specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourcePool {
    /// Pool name
    pub name: String,
    /// Pool type
    pub pool_type: String,
    /// Resources
    pub resources: HashMap<String, f64>,
}

/// Resource quota specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuota {
    /// Quota name
    pub name: String,
    /// Limits
    pub limits: HashMap<String, f64>,
}

/// Resource scheduling specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceScheduling {
    /// Scheduling strategy
    pub strategy: String,
    /// Affinity rules
    pub affinity: Vec<String>,
    /// Anti-affinity rules
    pub anti_affinity: Vec<String>,
} 