// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Config map and storage class manifest types.

use super::volume::VolumeReclaimPolicy;
use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Config specification
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConfigSpec {
    /// Config metadata
    #[serde(default)]
    pub metadata: ConfigMetadata,

    /// Config data
    #[serde(default)]
    pub data: HashMap<String, ConfigData>,

    /// Binary data
    #[serde(default)]
    pub binary_data: HashMap<String, Bytes>,

    /// Immutable
    #[serde(default)]
    pub immutable: bool,
}

/// Config metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigMetadata {
    /// Config name
    pub name: String,

    /// Config description
    pub description: Option<String>,

    /// Config labels
    pub labels: HashMap<String, String>,

    /// Config annotations
    pub annotations: HashMap<String, String>,
}

/// Config data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConfigData {
    /// String value
    String(String),
    /// YAML value
    Yaml(serde_yaml::Value),
    /// JSON value
    Json(serde_json::Value),
    /// TOML value
    Toml(String),
    /// Properties value
    Properties(HashMap<String, String>),
    /// File reference
    File(String),
    /// Template
    Template {
        /// Template string
        template: String,
        /// Template variables
        variables: HashMap<String, String>,
    },
}

/// Storage class specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageClassSpec {
    /// Storage class metadata
    pub metadata: StorageClassMetadata,

    /// Provisioner
    pub provisioner: String,

    /// Parameters
    pub parameters: HashMap<String, String>,

    /// Reclaim policy
    pub reclaim_policy: VolumeReclaimPolicy,

    /// Allow volume expansion
    pub allow_volume_expansion: bool,

    /// Volume binding mode
    pub volume_binding_mode: VolumeBindingMode,

    /// Allowed topologies
    pub allowed_topologies: Vec<TopologySelectorTerm>,
}

/// Storage class metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageClassMetadata {
    /// Storage class name
    pub name: String,

    /// Storage class description
    pub description: Option<String>,

    /// Storage class labels
    pub labels: HashMap<String, String>,

    /// Storage class annotations
    pub annotations: HashMap<String, String>,
}

/// Volume binding modes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VolumeBindingMode {
    /// Bind immediately when claim is created
    Immediate,
    /// Wait for first consumer pod to bind
    WaitForFirstConsumer,
}

/// Topology selector term
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologySelectorTerm {
    /// Match label expressions
    pub match_label_expressions: Vec<TopologySelectorLabelRequirement>,
}

/// Topology selector label requirement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologySelectorLabelRequirement {
    /// Key
    pub key: String,
    /// Values
    pub values: Vec<String>,
}

impl Default for ConfigMetadata {
    fn default() -> Self {
        Self {
            name: "default-config".to_string(),
            description: None,
            labels: HashMap::new(),
            annotations: HashMap::new(),
        }
    }
}
