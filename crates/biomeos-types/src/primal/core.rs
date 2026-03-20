// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Core Primal Types
//!
//! This module contains the fundamental primal type definitions including
//! PrimalType, PrimalMetadata, and basic resource requirements.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Universal Primal Type - NO HARDCODED NAMES
///
/// This replaces both the old PrimalType struct and string-based primal_type fields.
/// It provides a flexible, extensible way to identify and categorize primals.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimalType {
    /// Service category (compute, storage, security, orchestration, etc.)
    pub category: String,

    /// Service name (discovered dynamically: toadstool, nestgate, etc.)
    pub name: String,

    /// Service version using semantic versioning
    pub version: String,

    /// Additional metadata for classification and discovery
    pub metadata: HashMap<String, String>,
}

impl PrimalType {
    /// Create a new primal type
    ///
    /// # Example
    /// ```
    /// use biomeos_types::primal::PrimalType;
    /// let primal = PrimalType::new("compute", "toadstool", "1.0.0");
    /// assert_eq!(primal.category, "compute");
    /// ```
    #[must_use]
    pub fn new(
        category: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        Self {
            category: category.into(),
            name: name.into(),
            version: version.into(),
            metadata: HashMap::new(),
        }
    }

    /// Create with metadata
    pub fn with_metadata(
        category: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
        metadata: HashMap<String, String>,
    ) -> Self {
        Self {
            category: category.into(),
            name: name.into(),
            version: version.into(),
            metadata,
        }
    }

    /// Check if this primal provides a specific category
    pub fn is_category(&self, category: &str) -> bool {
        self.category == category
    }

    /// Check if this primal has a specific name
    pub fn is_name(&self, name: &str) -> bool {
        self.name == name
    }

    /// Get the full identifier string
    pub fn identifier(&self) -> String {
        format!("{}:{}:{}", self.category, self.name, self.version)
    }

    /// Create a PrimalType from discovered service info
    ///
    /// This is the recommended way to create PrimalType instances from
    /// services discovered through capability-based discovery.
    ///
    /// # Example
    /// ```
    /// # use biomeos_types::PrimalType;
    /// // From discovered service info
    /// let primal = PrimalType::from_discovered("compute", "toadstool", "2.1.0");
    /// ```
    pub fn from_discovered(
        category: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
        Self::new(category, name, version)
    }

    /// Create a PrimalType for self-identification
    ///
    /// Uses the PRIMAL_NAME environment variable to identify this primal.
    /// Falls back to "unknown" if not set.
    ///
    /// # Example
    /// ```
    /// # use biomeos_types::PrimalType;
    /// // Self-identification (reads PRIMAL_NAME from env)
    /// let my_type = PrimalType::identify_self("compute", "1.0.0");
    /// ```
    pub fn identify_self(category: impl Into<String>, version: impl Into<String>) -> Self {
        let name = std::env::var("PRIMAL_NAME").unwrap_or_else(|_| "unknown".to_string());
        Self::new(category, name, version)
    }

    /// Create a community/custom primal type
    ///
    /// Community primals are always in the "community" category,
    /// with optional sub-category in metadata.
    ///
    /// # Example
    /// ```
    /// # use biomeos_types::PrimalType;
    /// let community_primal = PrimalType::community("my-primal", "compute");
    /// assert_eq!(community_primal.category, "community");
    /// ```
    pub fn community(name: impl Into<String>, sub_category: impl Into<String>) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("community".to_string(), "true".to_string());
        metadata.insert("sub_category".to_string(), sub_category.into());
        Self::with_metadata("community", name, "1.0.0", metadata)
    }
}

/// Resource requirements for primal operations
#[derive(Debug, Clone, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ResourceRequirements {
    /// CPU requirements in cores
    pub cpu: Option<u32>,

    /// Memory requirements in MB
    pub memory: Option<u64>,

    /// Disk space requirements in MB
    pub disk: Option<u64>,

    /// Network bandwidth requirements in Mbps
    pub network: Option<u64>,

    /// GPU requirements
    pub gpu: Option<u32>,

    /// Additional resource requirements  
    #[serde(default)]
    pub additional: Vec<(String, String)>,
}

/// Primal metadata information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalMetadata {
    /// Primal unique identifier
    pub id: Uuid,

    /// Primal type information
    pub primal_type: PrimalType,

    /// Display name for UI
    pub display_name: String,

    /// Detailed description
    pub description: String,

    /// Primal author/organization
    pub author: String,

    /// License information
    pub license: String,

    /// Repository URL
    pub repository: Option<String>,

    /// Documentation URL
    pub documentation: Option<String>,

    /// Tags for categorization and search
    pub tags: Vec<String>,

    /// When this primal was created
    pub created_at: DateTime<Utc>,

    /// When this primal was last updated
    pub updated_at: DateTime<Utc>,

    /// Primal version history
    pub version_history: Vec<String>,

    /// Resource requirements
    pub resource_requirements: ResourceRequirements,

    /// Additional metadata
    pub additional: HashMap<String, serde_json::Value>,
}

#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primal_type_new() {
        let pt = PrimalType::new("compute", "test-service", "1.0.0");
        assert_eq!(pt.category, "compute");
        assert_eq!(pt.name, "test-service");
        assert_eq!(pt.version, "1.0.0");
        assert!(pt.metadata.is_empty());
    }

    #[test]
    fn test_primal_type_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("key".to_string(), "value".to_string());

        let pt = PrimalType::with_metadata("storage", "test", "2.0.0", metadata);
        assert_eq!(pt.category, "storage");
        assert_eq!(pt.metadata.get("key"), Some(&"value".to_string()));
    }

    #[test]
    fn test_primal_type_is_category() {
        let pt = PrimalType::new("compute", "test", "1.0.0");
        assert!(pt.is_category("compute"));
        assert!(!pt.is_category("storage"));
    }

    #[test]
    fn test_primal_type_is_name() {
        let pt = PrimalType::new("compute", "toadstool", "1.0.0");
        assert!(pt.is_name("toadstool"));
        assert!(!pt.is_name("beardog"));
    }

    #[test]
    fn test_primal_type_identifier() {
        let pt = PrimalType::new("compute", "toadstool", "1.0.0");
        assert_eq!(pt.identifier(), "compute:toadstool:1.0.0");
    }

    #[test]
    fn test_primal_type_from_discovered_compute() {
        // Test capability-based construction (discovered at runtime)
        let pt = PrimalType::from_discovered("compute", "toadstool", "1.0.0");
        assert_eq!(pt.category, "compute");
        assert_eq!(pt.name, "toadstool");
        assert_eq!(pt.version, "1.0.0");
    }

    #[test]
    fn test_primal_type_from_discovered_orchestration() {
        // Test capability-based construction (discovered at runtime)
        let pt = PrimalType::from_discovered("orchestration", "songbird", "0.2.1");
        assert_eq!(pt.category, "orchestration");
        assert_eq!(pt.name, "songbird");
        assert_eq!(pt.version, "0.2.1");
    }

    #[test]
    fn test_primal_type_from_discovered_storage() {
        // Test capability-based construction (discovered at runtime)
        let pt = PrimalType::from_discovered("storage", "nestgate", "2.0.0");
        assert_eq!(pt.category, "storage");
        assert_eq!(pt.name, "nestgate");
        assert_eq!(pt.version, "2.0.0");
    }

    #[test]
    fn test_primal_type_from_discovered_security() {
        // Test capability-based construction (discovered at runtime)
        let pt = PrimalType::from_discovered("security", "beardog", "0.9.3");
        assert_eq!(pt.category, "security");
        assert_eq!(pt.name, "beardog");
        assert_eq!(pt.version, "0.9.3");
    }

    #[test]
    fn test_primal_type_from_discovered_ai() {
        // Test capability-based construction (discovered at runtime)
        let pt = PrimalType::from_discovered("ai", "squirrel", "1.0.0");
        assert_eq!(pt.category, "ai");
        assert_eq!(pt.name, "squirrel");
        assert_eq!(pt.version, "1.0.0");
    }

    #[test]
    fn test_primal_type_community() {
        let pt = PrimalType::community("my-primal", "custom");
        assert_eq!(pt.category, "community");
        assert_eq!(pt.name, "my-primal");
        assert_eq!(pt.metadata.get("community"), Some(&"true".to_string()));
    }

    #[test]
    fn test_resource_requirements_default() {
        let rr = ResourceRequirements::default();
        assert!(rr.cpu.is_none());
        assert!(rr.memory.is_none());
        assert!(rr.disk.is_none());
        assert!(rr.network.is_none());
        assert!(rr.gpu.is_none());
        assert!(rr.additional.is_empty());
    }

    #[test]
    fn test_resource_requirements_with_values() {
        let rr = ResourceRequirements {
            cpu: Some(4),
            memory: Some(8192),
            disk: Some(102_400),
            network: Some(1000),
            gpu: Some(1),
            additional: vec![("custom".to_string(), "value".to_string())],
        };
        assert_eq!(rr.cpu, Some(4));
        assert_eq!(rr.memory, Some(8192));
        assert_eq!(rr.disk, Some(102_400));
        assert_eq!(rr.network, Some(1000));
        assert_eq!(rr.gpu, Some(1));
        assert_eq!(rr.additional.len(), 1);
    }

    #[test]
    fn test_primal_type_serialization() {
        let pt = PrimalType::new("compute", "test", "1.0.0");
        let json = serde_json::to_string(&pt).unwrap();
        let deserialized: PrimalType = serde_json::from_str(&json).unwrap();
        assert_eq!(pt, deserialized);
    }

    #[test]
    fn test_resource_requirements_serialization() {
        let rr = ResourceRequirements {
            cpu: Some(2),
            memory: Some(4096),
            ..Default::default()
        };
        let json = serde_json::to_string(&rr).unwrap();
        let deserialized: ResourceRequirements = serde_json::from_str(&json).unwrap();
        assert_eq!(rr, deserialized);
    }

    #[test]
    fn test_primal_metadata_creation() {
        // Use capability-based construction (discovered at runtime)
        let primal_type = PrimalType::from_discovered("compute", "toadstool", "1.0.0");

        let metadata = PrimalMetadata {
            id: Uuid::new_v4(),
            primal_type,
            display_name: "ToadStool Compute".to_string(),
            description: "Compute primal".to_string(),
            author: "biomeOS".to_string(),
            license: "MIT".to_string(),
            repository: Some("https://github.com/example/toadstool".to_string()),
            documentation: None,
            tags: vec!["compute".to_string(), "gpu".to_string()],
            created_at: Utc::now(),
            updated_at: Utc::now(),
            version_history: vec!["1.0.0".to_string()],
            resource_requirements: ResourceRequirements::default(),
            additional: HashMap::new(),
        };

        assert_eq!(metadata.display_name, "ToadStool Compute");
        assert_eq!(metadata.primal_type.name, "toadstool");
        assert_eq!(metadata.tags.len(), 2);
    }
}
