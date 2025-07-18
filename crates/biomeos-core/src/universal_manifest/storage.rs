//! Storage Specifications Module

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Storage specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageSpec {
    /// Storage classes
    pub classes: Vec<StorageClass>,
    /// Volume specifications
    pub volumes: Vec<VolumeSpec>,
}

/// Storage class
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageClass {
    /// Class name
    pub name: String,
    /// Storage type
    pub storage_type: String,
    /// Parameters
    pub parameters: HashMap<String, String>,
}

/// Volume specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeSpec {
    /// Volume name
    pub name: String,
    /// Size in MB
    pub size_mb: u64,
    /// Storage class
    pub storage_class: String,
} 