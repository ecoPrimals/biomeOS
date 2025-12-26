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

use crate::clients::base::PrimalHttpClient;
use crate::primal_client::{HealthStatus, PrimalClient};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Squirrel AI and intelligence client
///
/// # Example
/// ```no_run
/// use biomeos_core::clients::squirrel::SquirrelClient;
/// use serde_json::json;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let squirrel = SquirrelClient::new("http://localhost:8001");
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
    http: PrimalHttpClient,
    endpoint: String,
}

impl SquirrelClient {
    /// Create a new Squirrel client
    ///
    /// # Arguments
    /// * `endpoint` - Squirrel endpoint URL (discovered via capability query for "ai")
    pub fn new(endpoint: impl Into<String>) -> Self {
        let endpoint = endpoint.into();
        Self {
            http: PrimalHttpClient::new(&endpoint),
            endpoint,
        }
    }

    /// Analyze system state for optimization opportunities
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
    /// let squirrel = SquirrelClient::new("http://localhost:8001");
    /// let state = json!({"cpu": 75, "memory": 60});
    /// let analysis = squirrel.analyze_system_optimization(&state).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn analyze_system_optimization(
        &self,
        system_state: &Value,
    ) -> Result<OptimizationAnalysis> {
        let response = self
            .http
            .post("/api/v1/ai/optimize", system_state.clone())
            .await?;

        serde_json::from_value(response)
            .map_err(|e| anyhow::anyhow!("Failed to parse optimization analysis: {}", e))
    }

    /// Get AI inference prediction
    ///
    /// # Arguments
    /// * `model` - Model name to use for inference
    /// * `input` - Input data for the model
    ///
    /// # Errors
    /// Returns an error if the inference request fails.
    pub async fn infer(&self, model: &str, input: &Value) -> Result<InferenceResult> {
        let body = serde_json::json!({
            "model": model,
            "input": input
        });

        let response = self.http.post("/api/v1/ai/infer", body).await?;

        serde_json::from_value(response)
            .map_err(|e| anyhow::anyhow!("Failed to parse inference result: {}", e))
    }

    /// Detect patterns in data
    ///
    /// # Arguments
    /// * `data` - Data to analyze for patterns
    ///
    /// # Errors
    /// Returns an error if the pattern detection request fails.
    pub async fn detect_patterns(&self, data: &Value) -> Result<Vec<Pattern>> {
        let response = self.http.post("/api/v1/ai/patterns", data.clone()).await?;

        serde_json::from_value(response)
            .map_err(|e| anyhow::anyhow!("Failed to parse patterns: {}", e))
    }

    /// Get decision support recommendations
    ///
    /// # Arguments
    /// * `context` - Decision context
    ///
    /// # Errors
    /// Returns an error if the request fails.
    pub async fn decision_support(&self, context: &Value) -> Result<Vec<Recommendation>> {
        let response = self
            .http
            .post("/api/v1/ai/decision", context.clone())
            .await?;

        serde_json::from_value(response)
            .map_err(|e| anyhow::anyhow!("Failed to parse recommendations: {}", e))
    }
}

#[async_trait]
impl PrimalClient for SquirrelClient {
    fn name(&self) -> &str {
        "squirrel"
    }

    fn endpoint(&self) -> &str {
        &self.endpoint
    }

    async fn is_available(&self) -> bool {
        self.health_check().await.is_ok()
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let response = self.http.get("/health").await?;
        Ok(HealthStatus {
            healthy: response["status"] == "healthy",
            message: response["message"]
                .as_str()
                .unwrap_or("Unknown")
                .to_string(),
            details: Some(response),
        })
    }

    async fn request(&self, method: &str, path: &str, body: Option<Value>) -> Result<Value> {
        match method {
            "GET" => self.http.get(path).await,
            "POST" => self.http.post(path, body.unwrap_or(Value::Null)).await,
            _ => anyhow::bail!("Unsupported method: {}", method),
        }
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

    #[test]
    fn test_squirrel_client_creation() {
        let client = SquirrelClient::new("http://localhost:8001");
        assert_eq!(client.name(), "squirrel");
        assert_eq!(client.endpoint(), "http://localhost:8001");
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
