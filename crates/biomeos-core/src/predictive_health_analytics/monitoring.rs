//! Monitoring session management

use super::config::*;
use super::types::*;
use crate::PrimalIdentity;

/// Monitoring session
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MonitoringSession {
    /// Session identifier
    pub session_id: String,
    /// Target primals to monitor
    pub targets: Vec<PrimalIdentity>,
    /// Monitoring configuration
    pub config: MonitoringConfig,
    /// Session status
    pub status: MonitoringStatus,
    /// Session start time
    pub start_time: u64,
    /// Metrics to collect
    pub metrics: Vec<String>,
    /// Alert conditions
    pub alert_conditions: Vec<AlertCondition>,
    /// Analysis preferences
    pub analysis_preferences: AnalysisPreferences,
}
