// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

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
/// This replaces the old `Vec<String>` capabilities with a structured approach.
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
    pub fn new(
        category: impl Into<String>,
        name: impl Into<String>,
        version: impl Into<String>,
    ) -> Self {
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
    #[must_use]
    pub fn identifier(&self) -> String {
        format!("{}:{}:{}", self.category, self.name, self.version)
    }

    /// Check if this capability matches a requirement
    #[must_use]
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
    /// 50th percentile (median) latency in milliseconds
    pub p50_ms: u32,
    /// 95th percentile latency in milliseconds
    pub p95_ms: u32,
    /// 99th percentile latency in milliseconds
    pub p99_ms: u32,
    /// Maximum latency in milliseconds
    pub max_ms: u32,
}

impl PrimalCapability {
    /// Get display name for UI
    #[must_use]
    pub fn display_name(&self) -> String {
        format!("{} {}", self.name, self.version)
    }

    /// Get icon based on capability category
    #[must_use]
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
    #[must_use]
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

    /// Create a compute capability
    #[must_use]
    pub fn compute() -> Self {
        Self::new("compute", "compute", "1.0")
    }

    /// Create a storage capability
    #[must_use]
    pub fn storage() -> Self {
        Self::new("storage", "storage", "1.0")
    }

    /// Create a networking capability
    #[must_use]
    pub fn networking() -> Self {
        Self::new("networking", "networking", "1.0")
    }

    /// Create a security capability
    #[must_use]
    pub fn security() -> Self {
        Self::new("security", "security", "1.0")
    }

    /// Create an AI capability
    #[must_use]
    pub fn ai() -> Self {
        Self::new("ai", "ai", "1.0")
    }

    /// Create a machine learning capability
    #[must_use]
    pub fn machine_learning() -> Self {
        Self::new("ml", "machine-learning", "1.0")
    }

    /// Create an analytics capability
    #[must_use]
    pub fn analytics() -> Self {
        Self::new("analytics", "analytics", "1.0")
    }

    /// Create a gaming capability
    #[must_use]
    pub fn gaming() -> Self {
        Self::new("gaming", "gaming", "1.0")
    }

    /// Create a web development capability
    #[must_use]
    pub fn web_development() -> Self {
        Self::new("web", "web-development", "1.0")
    }

    /// Create an orchestration capability
    #[must_use]
    pub fn orchestration() -> Self {
        Self::new("orchestration", "orchestration", "1.0")
    }

    /// Create an authentication capability
    #[must_use]
    pub fn authentication() -> Self {
        Self::new("authentication", "authentication", "1.0")
    }

    /// Create an encryption capability
    #[must_use]
    pub fn encryption() -> Self {
        Self::new("encryption", "encryption", "1.0")
    }

    /// Create a data processing capability
    #[must_use]
    pub fn data_processing() -> Self {
        Self::new("data", "data-processing", "1.0")
    }

    /// Create a service discovery capability
    #[must_use]
    pub fn service_discovery() -> Self {
        Self::new("networking", "service-discovery", "1.0")
    }

    /// Create a load balancing capability
    #[must_use]
    pub fn load_balancing() -> Self {
        Self::new("networking", "load-balancing", "1.0")
    }

    /// Create a science capability (wetSpring, neuralSpring, etc.)
    #[must_use]
    pub fn science() -> Self {
        Self::new("science", "science", "1.0")
    }

    /// Create a custom capability with the given name
    pub fn custom(name: impl Into<String>) -> Self {
        Self::new("custom", name, "1.0")
    }
}

#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_primal_capability_new() {
        let cap = PrimalCapability::new("compute", "test-cap", "1.0.0");
        assert_eq!(cap.category, "compute");
        assert_eq!(cap.name, "test-cap");
        assert_eq!(cap.version, "1.0.0");
        assert!(cap.parameters.is_empty());
        assert!(cap.performance.is_none());
    }

    #[test]
    fn test_primal_capability_with_parameters() {
        let params = vec![("key".to_string(), "value".to_string())];
        let cap = PrimalCapability::with_parameters("storage", "blob-storage", "2.0", params);
        assert_eq!(cap.category, "storage");
        assert_eq!(cap.parameters.len(), 1);
    }

    #[test]
    fn test_primal_capability_identifier() {
        let cap = PrimalCapability::new("compute", "gpu-compute", "1.5.0");
        assert_eq!(cap.identifier(), "compute:gpu-compute:1.5.0");
    }

    #[test]
    fn test_primal_capability_matches() {
        let cap = PrimalCapability::new("security", "encryption", "1.0");
        assert!(cap.matches("security", "encryption"));
        assert!(!cap.matches("security", "auth"));
        assert!(!cap.matches("compute", "encryption"));
    }

    #[test]
    fn test_primal_capability_display_name() {
        let cap = PrimalCapability::new("ai", "ml-inference", "2.0");
        assert_eq!(cap.display_name(), "ml-inference 2.0");
    }

    #[test]
    fn test_primal_capability_icons() {
        assert_eq!(PrimalCapability::compute().icon(), "💻");
        assert_eq!(PrimalCapability::storage().icon(), "💾");
        assert_eq!(PrimalCapability::networking().icon(), "🌐");
        assert_eq!(PrimalCapability::security().icon(), "🔐");
        assert_eq!(PrimalCapability::ai().icon(), "🤖");
        assert_eq!(PrimalCapability::machine_learning().icon(), "🧠");
        assert_eq!(PrimalCapability::analytics().icon(), "📊");
        assert_eq!(PrimalCapability::gaming().icon(), "🎮");
        assert_eq!(PrimalCapability::web_development().icon(), "🌍");
        assert_eq!(PrimalCapability::orchestration().icon(), "🎵");
        assert_eq!(PrimalCapability::authentication().icon(), "🔑");
        assert_eq!(PrimalCapability::encryption().icon(), "🔒");
        assert_eq!(PrimalCapability::custom("test").icon(), "⚙️");
    }

    #[test]
    fn test_primal_capability_descriptions() {
        assert!(!PrimalCapability::compute().description().is_empty());
        assert!(!PrimalCapability::storage().description().is_empty());
        assert!(!PrimalCapability::security().description().is_empty());
        assert!(!PrimalCapability::ai().description().is_empty());
        assert!(!PrimalCapability::custom("x").description().is_empty());
    }

    #[test]
    fn test_convenience_constructors() {
        let cap = PrimalCapability::compute();
        assert_eq!(cap.category, "compute");

        let cap = PrimalCapability::storage();
        assert_eq!(cap.category, "storage");

        let cap = PrimalCapability::networking();
        assert_eq!(cap.category, "networking");

        let cap = PrimalCapability::security();
        assert_eq!(cap.category, "security");

        let cap = PrimalCapability::ai();
        assert_eq!(cap.category, "ai");

        let cap = PrimalCapability::machine_learning();
        assert_eq!(cap.category, "ml");

        let cap = PrimalCapability::analytics();
        assert_eq!(cap.category, "analytics");

        let cap = PrimalCapability::gaming();
        assert_eq!(cap.category, "gaming");

        let cap = PrimalCapability::web_development();
        assert_eq!(cap.category, "web");

        let cap = PrimalCapability::orchestration();
        assert_eq!(cap.category, "orchestration");

        let cap = PrimalCapability::authentication();
        assert_eq!(cap.category, "authentication");

        let cap = PrimalCapability::encryption();
        assert_eq!(cap.category, "encryption");

        let cap = PrimalCapability::data_processing();
        assert_eq!(cap.category, "data");

        let cap = PrimalCapability::service_discovery();
        assert_eq!(cap.category, "networking");
        assert_eq!(cap.name, "service-discovery");

        let cap = PrimalCapability::load_balancing();
        assert_eq!(cap.category, "networking");
        assert_eq!(cap.name, "load-balancing");

        let cap = PrimalCapability::custom("my-custom");
        assert_eq!(cap.category, "custom");
        assert_eq!(cap.name, "my-custom");
    }

    #[test]
    fn test_capability_metadata() {
        let meta = CapabilityMetadata {
            name: "test-capability".to_string(),
            version: "1.0.0".to_string(),
            description: "A test capability".to_string(),
            parameters: vec![CapabilityParameter {
                name: "input".to_string(),
                param_type: "string".to_string(),
                required: true,
                default_value: None,
                description: Some("Input parameter".to_string()),
            }],
            security_requirements: vec!["tls".to_string()],
            resource_requirements: ResourceRequirements::default(),
        };

        assert_eq!(meta.name, "test-capability");
        assert_eq!(meta.parameters.len(), 1);
        assert_eq!(meta.security_requirements.len(), 1);
    }

    #[test]
    fn test_capability_parameter() {
        let param = CapabilityParameter {
            name: "count".to_string(),
            param_type: "integer".to_string(),
            required: false,
            default_value: Some(serde_json::json!(10)),
            description: Some("Number of items".to_string()),
        };

        assert_eq!(param.name, "count");
        assert!(!param.required);
        assert!(param.default_value.is_some());
    }

    #[test]
    fn test_capability_performance() {
        let perf = CapabilityPerformance {
            throughput_ops_per_sec: Some(10000),
            latency_ms: Some(LatencyCharacteristics {
                p50_ms: 5,
                p95_ms: 15,
                p99_ms: 50,
                max_ms: 200,
            }),
            resource_requirements: None,
            availability_sla: Some(9999), // 99.99%
        };

        assert_eq!(perf.throughput_ops_per_sec, Some(10000));
        assert_eq!(perf.availability_sla, Some(9999));
        let latency = perf.latency_ms.unwrap();
        assert_eq!(latency.p50_ms, 5);
        assert_eq!(latency.p99_ms, 50);
    }

    #[test]
    fn test_latency_characteristics() {
        let latency = LatencyCharacteristics {
            p50_ms: 10,
            p95_ms: 25,
            p99_ms: 100,
            max_ms: 500,
        };

        assert!(latency.p50_ms < latency.p95_ms);
        assert!(latency.p95_ms < latency.p99_ms);
        assert!(latency.p99_ms < latency.max_ms);
    }

    #[test]
    fn test_primal_capability_serialization() {
        let cap = PrimalCapability::new("compute", "gpu-compute", "1.0");
        let json = serde_json::to_string(&cap).unwrap();
        let deserialized: PrimalCapability = serde_json::from_str(&json).unwrap();
        assert_eq!(cap, deserialized);
    }
}
