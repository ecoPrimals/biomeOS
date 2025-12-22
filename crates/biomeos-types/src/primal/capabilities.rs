//! Primal Capability System
//!
//! This module defines the capability system for primals, including
//! capability metadata, parameters, and performance characteristics.

use serde::{Deserialize, Serialize};

use super::core::ResourceRequirements;

/// Capability metadata information
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityMetadata {
    /// Capability name
    pub name: String,
    
    /// Capability version
    pub version: String,
    
    /// Capability description
    pub description: String,
    
    /// Required parameters
    pub parameters: Vec<CapabilityParameter>,
    
    /// Security requirements
    pub security_requirements: Vec<String>,
    
    /// Resource requirements
    pub resource_requirements: ResourceRequirements,
}

/// Capability parameter definition
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityParameter {
    /// Parameter name
    pub name: String,
    
    /// Parameter type
    pub param_type: String,
    
    /// Whether parameter is required
    pub required: bool,
    
    /// Default value if any
    pub default_value: Option<serde_json::Value>,
    
    /// Parameter description
    pub description: Option<String>,
}

/// Universal Primal Capabilities - fully extensible
/// 
/// This replaces the old Vec<String> capabilities with a structured approach.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PrimalCapability {
    /// Capability category (e.g., "compute", "storage", "security")
    pub category: String,

    /// Specific capability name (e.g., "container-execution", "file-storage", "encryption")
    pub name: String,

    /// Capability version
    pub version: String,

    /// Capability parameters and constraints (as key-value string pairs)
    pub parameters: Vec<(String, String)>,

    /// Performance characteristics
    pub performance: Option<CapabilityPerformance>,
}

impl PrimalCapability {
    /// Create a new capability
    pub fn new(category: impl Into<String>, name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            category: category.into(),
            name: name.into(),
            version: version.into(),
            parameters: Vec::new(),
            performance: None,
        }
    }

    /// Create with parameters
    pub fn with_parameters(
        category: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
        parameters: Vec<(String, String)>,
    ) -> Self {
        Self {
            category: category.into(),
            name: name.into(),
            version: version.into(),
            parameters,
            performance: None,
        }
    }

    /// Get the capability identifier
    pub fn identifier(&self) -> String {
        format!("{}:{}:{}", self.category, self.name, self.version)
    }

    /// Check if this capability matches a requirement
    pub fn matches(&self, category: &str, name: &str) -> bool {
        self.category == category && self.name == name
    }
}

/// Performance characteristics for capabilities
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct CapabilityPerformance {
    /// Throughput capacity (operations per second)
    pub throughput_ops_per_sec: Option<u64>,

    /// Latency characteristics (milliseconds)
    pub latency_ms: Option<LatencyCharacteristics>,

    /// Resource requirements
    pub resource_requirements: Option<ResourceRequirements>,

    /// Availability guarantees (as percentage * 100, e.g., 9999 = 99.99%)
    pub availability_sla: Option<u32>, // e.g., 9999 for 99.99%
}

/// Latency characteristics
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LatencyCharacteristics {
    pub p50_ms: u32,
    pub p95_ms: u32,
    pub p99_ms: u32,
    pub max_ms: u32,
}

impl PrimalCapability {
    /// Get display name for UI
    pub fn display_name(&self) -> String {
        format!("{} {}", self.name, self.version)
    }

    /// Get icon based on capability category
    pub fn icon(&self) -> &'static str {
        match self.category.as_str() {
            "compute" => "💻",
            "storage" => "💾", 
            "networking" => "🌐",
            "security" => "🔐",
            "ai" => "🤖",
            "ml" | "machine-learning" => "🧠",
            "analytics" => "📊",
            "gaming" => "🎮",
            "web" | "web-development" => "🌍",
            "orchestration" => "🎵",
            "authentication" => "🔑",
            "encryption" => "🔒",
            _ => "⚙️", // Default gear icon
        }
    }

    /// Get description based on capability category
    pub fn description(&self) -> &'static str {
        match self.category.as_str() {
            "compute" => "High-performance computing and processing",
            "storage" => "Data storage and persistence",
            "networking" => "Network communication and connectivity",
            "security" => "Security and access control",
            "ai" => "Artificial intelligence and automation",
            "ml" | "machine-learning" => "Machine learning and data science",
            "analytics" => "Data analysis and insights",
            "gaming" => "Gaming and entertainment",
            "web" | "web-development" => "Web development and deployment",
            "orchestration" => "Service orchestration and management",
            "authentication" => "User authentication and identity",
            "encryption" => "Data encryption and protection",
            _ => "General capability",
        }
    }

    // Convenience constructors for common capabilities
    pub fn compute() -> Self {
        Self::new("compute", "compute", "1.0")
    }

    pub fn storage() -> Self {
        Self::new("storage", "storage", "1.0")
    }

    pub fn networking() -> Self {
        Self::new("networking", "networking", "1.0")
    }

    pub fn security() -> Self {
        Self::new("security", "security", "1.0")
    }

    pub fn ai() -> Self {
        Self::new("ai", "ai", "1.0")
    }

    pub fn machine_learning() -> Self {
        Self::new("ml", "machine-learning", "1.0")
    }

    pub fn analytics() -> Self {
        Self::new("analytics", "analytics", "1.0")
    }

    pub fn gaming() -> Self {
        Self::new("gaming", "gaming", "1.0")
    }

    pub fn web_development() -> Self {
        Self::new("web", "web-development", "1.0")
    }

    pub fn orchestration() -> Self {
        Self::new("orchestration", "orchestration", "1.0")
    }

    pub fn authentication() -> Self {
        Self::new("authentication", "authentication", "1.0")
    }

    pub fn encryption() -> Self {
        Self::new("encryption", "encryption", "1.0")
    }

    pub fn data_processing() -> Self {
        Self::new("data", "data-processing", "1.0")
    }

    pub fn service_discovery() -> Self {
        Self::new("networking", "service-discovery", "1.0")
    }

    pub fn load_balancing() -> Self {
        Self::new("networking", "load-balancing", "1.0")
    }

    pub fn custom(name: impl Into<String>) -> Self {
        Self::new("custom", name, "1.0")
    }
} 