//! Core Primal Types
//!
//! This module contains the fundamental primal type definitions including
//! PrimalType, PrimalMetadata, and basic resource requirements.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Universal Primal Type - NO HARDCODED NAMES
/// 
/// This replaces both the old PrimalType struct and string-based primal_type fields.
/// It provides a flexible, extensible way to identify and categorize primals.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
    pub fn new(category: impl Into<String>, name: impl Into<String>, version: impl Into<String>) -> Self {
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

    /// Convenience constructors for known primal types
    pub fn toadstool() -> Self {
        Self::new("compute", "toadstool", "1.0.0")
    }

    pub fn songbird() -> Self {
        Self::new("orchestration", "songbird", "1.0.0")
    }

    pub fn nestgate() -> Self {
        Self::new("storage", "nestgate", "1.0.0")
    }

    pub fn beardog() -> Self {
        Self::new("security", "beardog", "1.0.0")
    }

    pub fn squirrel() -> Self {
        Self::new("ai", "squirrel", "1.0.0")
    }

    /// Create a community/custom primal type
    pub fn community(name: impl Into<String>, _category: impl Into<String>) -> Self {
        let mut metadata = HashMap::new();
        metadata.insert("community".to_string(), "true".to_string());
        Self::with_metadata("community", name, "1.0.0", metadata)
    }
}

/// Resource requirements for primal operations
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    pub additional: Vec<(String, String)>,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            cpu: None,
            memory: None,
            disk: None,
            network: None,
            gpu: None,
            additional: Vec::new(),
        }
    }
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