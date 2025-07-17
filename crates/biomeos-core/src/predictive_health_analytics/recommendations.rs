//! Health recommendation system

use super::types::*;

/// Health recommendation
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthRecommendation {
    /// Recommendation type
    pub recommendation_type: RecommendationType,
    /// Priority level
    pub priority: RecommendationPriority,
    /// Recommendation description
    pub description: String,
    /// Expected impact percentage
    pub expected_impact: f64,
    /// Implementation effort required
    pub implementation_effort: ImplementationEffort,
    /// Timeline for implementation (seconds)
    pub timeline: u64,
}
