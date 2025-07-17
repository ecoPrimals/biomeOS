//! Health report generation

use super::predictions::*;
use super::recommendations::*;
use super::trends::*;
use super::types::*;

/// Comprehensive health report
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct HealthReport {
    /// Primal identifier
    pub primal_id: String,
    /// Report timestamp
    pub timestamp: u64,
    /// Current health snapshot
    pub current_snapshot: HealthSnapshot,
    /// Trend analysis results
    pub trend_analysis: TrendAnalysis,
    /// Health predictions
    pub predictions: HealthPrediction,
    /// Overall health status
    pub overall_status: OverallHealthStatus,
    /// Health recommendations
    pub recommendations: Vec<HealthRecommendation>,
}
