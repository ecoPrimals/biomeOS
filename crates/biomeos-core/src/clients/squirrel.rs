// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! Squirrel client for AI and intelligence services
//!
//! Squirrel is the AI and intelligence primal. It provides:
//! - AI inference and predictions
//! - System optimization analysis
//! - Pattern recognition
//! - Decision support
//!
//! # Transport Evolution
//!
//! **NEW**: Auto-discovery via Unix socket (JSON-RPC 2.0)
//! - **PRIMARY**: JSON-RPC over Unix socket (100x faster, secure)
//! - **FALLBACK**: HTTP REST API (deprecated, legacy only)
//!
//! # Quick Start
//!
//! ```no_run
//! use biomeos_core::clients::squirrel::SquirrelClient;
//! use serde_json::json;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Auto-discover via Unix socket
//!     let squirrel = SquirrelClient::discover("nat0").await?;
//!
//!     // Analyze system for optimization opportunities
//!     let system_state = json!({"cpu": 75, "memory": 60});
//!     let analysis = squirrel.analyze_system_optimization(&system_state).await?;
//!     println!("Optimization score: {}", analysis.score);
//!
//!     Ok(())
//! }
//! ```

use crate::clients::transport::{TransportClient, TransportPreference};
use crate::primal_client::{HealthStatus, PrimalClient};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Squirrel AI and intelligence client
///
/// Uses JSON-RPC 2.0 over Unix sockets for fast, secure communication.
///
/// # Example
/// ```no_run
/// use biomeos_core::clients::squirrel::SquirrelClient;
/// use serde_json::json;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Auto-discover via Unix socket
///     let squirrel = SquirrelClient::discover("nat0").await?;
///
///     // Analyze system for optimization opportunities
///     let system_state = json!({"cpu": 75, "memory": 60});
///     let analysis = squirrel.analyze_system_optimization(&system_state).await?;
///     println!("Optimization score: {}", analysis.score);
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SquirrelClient {
    transport: TransportClient,
    family_id: String,
}

impl SquirrelClient {
    /// Auto-discover Squirrel via Unix socket
    ///
    /// Searches for Squirrel's Unix socket in XDG runtime directory.
    /// Falls back to HTTP if Unix socket not available.
    ///
    /// # Arguments
    /// * `family_id` - Genetic family ID (e.g., "nat0")
    ///
    /// # Returns
    /// SquirrelClient configured with JSON-RPC over Unix socket (primary)
    /// or HTTP (fallback)
    ///
    /// # Example
    /// ```no_run
    /// use biomeos_core::clients::squirrel::SquirrelClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let squirrel = SquirrelClient::discover("nat0").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn discover(family_id: &str) -> Result<Self> {
        let transport = TransportClient::discover_with_preference(
            "squirrel",
            family_id,
            TransportPreference::UnixSocket,
        ).await
            .context("Failed to discover Squirrel. Is it running?")?;
        
        Ok(Self {
            transport,
            family_id: family_id.to_string(),
        })
    }
    
    /// Create from explicit endpoint (HTTP fallback)
    ///
    /// **DEPRECATED**: Use `discover()` for Unix socket support (100x faster)
    ///
    /// # Arguments
    /// * `endpoint` - HTTP endpoint URL (e.g., "http://localhost:8001")
    /// * `family_id` - Genetic family ID
    #[deprecated(note = "Use SquirrelClient::discover() for Unix socket support")]
    pub async fn from_endpoint(endpoint: impl Into<String>, family_id: &str) -> Result<Self> {
        let _endpoint = endpoint.into();
        let transport = TransportClient::discover_with_preference(
            "squirrel",
            family_id,
            TransportPreference::Http
        ).await
            .context("Failed to create HTTP client")?;
        
        Ok(Self {
            transport,
            family_id: family_id.to_string(),
        })
    }
    
    /// Legacy constructor (DEPRECATED)
    ///
    /// **BREAKING**: This method is now async. Use `discover()` instead.
    #[deprecated(note = "Use SquirrelClient::discover() instead")]
    pub fn new(_endpoint: impl Into<String>) -> Self {
        panic!("SquirrelClient::new() is deprecated. Use SquirrelClient::discover() instead.");
    }

    /// Analyze system state for optimization opportunities
    ///
    /// Uses Squirrel's JSON-RPC API: `ai.optimize_system`
    ///
    /// # Arguments
    /// * `system_state` - Current system state as JSON
    ///
    /// # Returns
    /// Analysis results with optimization suggestions
    ///
    /// # Errors
    /// Returns an error if the analysis request fails.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::squirrel::SquirrelClient;
    /// # use serde_json::json;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let squirrel = SquirrelClient::discover("nat0").await?;
    /// let state = json!({"cpu": 75, "memory": 60});
    /// let analysis = squirrel.analyze_system_optimization(&state).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn analyze_system_optimization(
        &self,
        system_state: &Value,
    ) -> Result<OptimizationAnalysis> {
        let response = self.transport.call(
            "ai.optimize_system",
            Some(system_state.clone())
        ).await
            .context("Failed to call ai.optimize_system")?;

        serde_json::from_value(response)
            .context("Failed to parse optimization analysis from response")
    }

    /// Get AI inference prediction
    ///
    /// Uses Squirrel's JSON-RPC API: `ai.infer`
    ///
    /// # Arguments
    /// * `model` - Model name to use for inference
    /// * `input` - Input data for the model
    ///
    /// # Errors
    /// Returns an error if the inference request fails.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::squirrel::SquirrelClient;
    /// # use serde_json::json;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let squirrel = SquirrelClient::discover("nat0").await?;
    /// let result = squirrel.infer("sentiment-analysis", &json!({"text": "Great!"})).await?;
    /// println!("Confidence: {}", result.confidence);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn infer(&self, model: &str, input: &Value) -> Result<InferenceResult> {
        let response = self.transport.call(
            "ai.infer",
            Some(serde_json::json!({
                "model": model,
                "input": input,
                "family_id": self.family_id
            }))
        ).await
            .context("Failed to call ai.infer")?;

        serde_json::from_value(response)
            .context("Failed to parse inference result from response")
    }

    /// Detect patterns in data
    ///
    /// Uses Squirrel's JSON-RPC API: `ai.detect_patterns`
    ///
    /// # Arguments
    /// * `data` - Data to analyze for patterns
    ///
    /// # Errors
    /// Returns an error if the pattern detection request fails.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::squirrel::SquirrelClient;
    /// # use serde_json::json;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let squirrel = SquirrelClient::discover("nat0").await?;
    /// let patterns = squirrel.detect_patterns(&json!({"values": [1,2,3,5,8,13]})).await?;
    /// println!("Found {} patterns", patterns.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn detect_patterns(&self, data: &Value) -> Result<Vec<Pattern>> {
        let response = self.transport.call(
            "ai.detect_patterns",
            Some(data.clone())
        ).await
            .context("Failed to call ai.detect_patterns")?;

        serde_json::from_value(response)
            .context("Failed to parse patterns from response")
    }

    /// Get decision support recommendations
    ///
    /// Uses Squirrel's JSON-RPC API: `ai.decision_support`
    ///
    /// # Arguments
    /// * `context` - Decision context
    ///
    /// # Errors
    /// Returns an error if the request fails.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::squirrel::SquirrelClient;
    /// # use serde_json::json;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let squirrel = SquirrelClient::discover("nat0").await?;
    /// let context = json!({"scenario": "scaling", "current_load": 80});
    /// let recommendations = squirrel.decision_support(&context).await?;
    /// println!("Recommendations: {}", recommendations.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn decision_support(&self, context: &Value) -> Result<Vec<Recommendation>> {
        let response = self.transport.call(
            "ai.decision_support",
            Some(context.clone())
        ).await
            .context("Failed to call ai.decision_support")?;

        serde_json::from_value(response)
            .context("Failed to parse recommendations from response")
    }
}

#[async_trait]
impl PrimalClient for SquirrelClient {
    fn name(&self) -> &str {
        "squirrel"
    }

    fn endpoint(&self) -> String {
        self.transport.endpoint().to_string()
    }

    async fn is_available(&self) -> bool {
        self.health_check().await.is_ok()
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        self.transport.health_check().await
    }

    async fn request(&self, method: &str, _path: &str, body: Option<Value>) -> Result<Value> {
        self.transport.call(method, body).await
    }
}

/// Optimization analysis result from Squirrel
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct OptimizationAnalysis {
    /// Overall optimization score (0-100)
    pub score: f64,

    /// Identified optimization opportunities
    pub opportunities: Vec<String>,

    /// Estimated improvement if opportunities are implemented
    pub estimated_improvement: EstimatedImprovement,

    /// Timestamp of analysis
    pub timestamp: String,
}

/// Estimated improvement from optimization
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct EstimatedImprovement {
    /// Performance improvement percentage
    pub performance: String,

    /// Resource efficiency improvement percentage
    pub resource_efficiency: String,

    /// Cost savings percentage
    pub cost_savings: String,
}

/// AI inference result
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct InferenceResult {
    /// Model used for inference
    pub model: String,

    /// Inference predictions/outputs
    pub predictions: Value,

    /// Confidence score (0.0-1.0)
    pub confidence: f64,

    /// Processing time in milliseconds
    pub processing_time_ms: u64,
}

/// Detected pattern in data
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Pattern {
    /// Pattern type identifier
    pub pattern_type: String,

    /// Pattern description
    pub description: String,

    /// Confidence level (0.0-1.0)
    pub confidence: f64,

    /// Pattern metadata
    #[serde(default)]
    pub metadata: Value,
}

/// Decision support recommendation
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Recommendation {
    /// Recommendation title
    pub title: String,

    /// Detailed recommendation
    pub description: String,

    /// Priority level (1-5, 5 being highest)
    pub priority: u8,

    /// Confidence in recommendation (0.0-1.0)
    pub confidence: f64,

    /// Rationale for recommendation
    pub rationale: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Integration test using harvested binary from plasmidBin/
    ///
    /// Start Squirrel manually:
    /// ```bash
    /// ./plasmidBin/primals/squirrel --family nat0
    /// ```
    #[ignore = "Requires running Squirrel from plasmidBin/primals/squirrel"]
    #[tokio::test]
    async fn test_squirrel_client_creation() {
        let client = SquirrelClient::discover("nat0").await.unwrap();
        assert_eq!(client.name(), "squirrel");
    }

    #[test]
    fn test_optimization_analysis_deserialization() {
        let json = serde_json::json!({
            "score": 85.5,
            "opportunities": ["Optimize query", "Add caching"],
            "estimated_improvement": {
                "performance": "15%",
                "resource_efficiency": "20%",
                "cost_savings": "10%"
            },
            "timestamp": "2025-12-24T12:00:00Z"
        });

        let analysis: OptimizationAnalysis = serde_json::from_value(json).unwrap();
        assert_eq!(analysis.score, 85.5);
        assert_eq!(analysis.opportunities.len(), 2);
    }
}
