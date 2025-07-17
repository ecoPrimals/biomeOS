//! Alert management for the monitoring dashboard

use super::config::{AlertConfig, AlertDestination, AlertSeverity};
use crate::BiomeResult;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Alert instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    /// Alert ID
    pub id: String,
    /// Alert name
    pub name: String,
    /// Alert severity
    pub severity: AlertSeverity,
    /// Alert message
    pub message: String,
    /// Alert timestamp
    pub timestamp: u64,
    /// Alert source
    pub source: String,
    /// Alert metadata
    pub metadata: HashMap<String, serde_json::Value>,
    /// Alert acknowledged
    pub acknowledged: bool,
    /// Alert resolved
    pub resolved: bool,
    /// Alert resolution time
    pub resolution_time: Option<u64>,
}

/// Alert manager
pub struct AlertManager {
    /// Alert configurations
    alert_configs: Arc<RwLock<HashMap<String, AlertConfig>>>,
    /// Active alerts
    active_alerts: Arc<RwLock<HashMap<String, Alert>>>,
    /// Alert history
    alert_history: Arc<RwLock<Vec<Alert>>>,
    /// Alert evaluator
    evaluator: Arc<AlertEvaluator>,
    /// Alert notifier
    notifier: Arc<AlertNotifier>,
}

/// Alert evaluator
pub struct AlertEvaluator {
    /// Evaluation state
    evaluation_state: Arc<RwLock<HashMap<String, EvaluationState>>>,
}

/// Evaluation state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationState {
    /// Last evaluation time
    pub last_evaluation: u64,
    /// Consecutive evaluations
    pub consecutive_evaluations: u32,
    /// Last value
    pub last_value: f64,
    /// Evaluation history
    pub history: Vec<EvaluationResult>,
}

/// Evaluation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EvaluationResult {
    /// Evaluation timestamp
    pub timestamp: u64,
    /// Evaluation value
    pub value: f64,
    /// Condition met
    pub condition_met: bool,
}

/// Alert notifier
pub struct AlertNotifier {
    /// Notification channels
    channels: Arc<RwLock<HashMap<String, Arc<dyn NotificationChannel>>>>,
    /// Notification history
    history: Arc<RwLock<Vec<NotificationRecord>>>,
}

/// Notification channel trait
#[async_trait]
pub trait NotificationChannel: Send + Sync {
    /// Send notification
    async fn send_notification(
        &self,
        alert: &Alert,
        destination: &AlertDestination,
    ) -> BiomeResult<()>;

    /// Get channel status
    async fn get_status(&self) -> BiomeResult<ChannelStatus>;
}

/// Channel status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelStatus {
    /// Channel name
    pub name: String,
    /// Channel enabled
    pub enabled: bool,
    /// Channel healthy
    pub healthy: bool,
    /// Last notification time
    pub last_notification: Option<u64>,
    /// Notification count
    pub notification_count: u64,
    /// Error count
    pub error_count: u64,
}

/// Notification record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotificationRecord {
    /// Notification ID
    pub id: String,
    /// Alert ID
    pub alert_id: String,
    /// Destination
    pub destination: AlertDestination,
    /// Notification timestamp
    pub timestamp: u64,
    /// Notification successful
    pub successful: bool,
    /// Error message if failed
    pub error: Option<String>,
}

impl AlertManager {
    /// Create a new alert manager
    pub fn new() -> Self {
        Self {
            alert_configs: Arc::new(RwLock::new(HashMap::new())),
            active_alerts: Arc::new(RwLock::new(HashMap::new())),
            alert_history: Arc::new(RwLock::new(Vec::new())),
            evaluator: Arc::new(AlertEvaluator::new()),
            notifier: Arc::new(AlertNotifier::new()),
        }
    }

    /// Add alert configuration
    pub async fn add_alert_config(&self, config: AlertConfig) -> BiomeResult<()> {
        let mut configs = self.alert_configs.write().await;
        configs.insert(config.name.clone(), config);
        Ok(())
    }

    /// Remove alert configuration
    pub async fn remove_alert_config(&self, name: &str) -> BiomeResult<()> {
        let mut configs = self.alert_configs.write().await;
        configs.remove(name);
        Ok(())
    }

    /// Evaluate alerts
    pub async fn evaluate_alerts(&self, metrics: &HashMap<String, f64>) -> BiomeResult<()> {
        let configs = self.alert_configs.read().await;

        for (name, config) in configs.iter() {
            if !config.enabled {
                continue;
            }

            if let Some(metric_value) = metrics.get(&config.condition.metric_name) {
                let should_alert = self
                    .evaluator
                    .evaluate_condition(&config.condition, *metric_value)
                    .await?;

                if should_alert {
                    self.trigger_alert(name, config, *metric_value).await?;
                }
            }
        }

        Ok(())
    }

    /// Trigger alert
    async fn trigger_alert(&self, name: &str, config: &AlertConfig, value: f64) -> BiomeResult<()> {
        let alert = Alert {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.to_string(),
            severity: config.severity.clone(),
            message: format!("Alert {} triggered with value {}", name, value),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            source: "monitoring_dashboard".to_string(),
            metadata: HashMap::new(),
            acknowledged: false,
            resolved: false,
            resolution_time: None,
        };

        // Add to active alerts
        let mut active_alerts = self.active_alerts.write().await;
        active_alerts.insert(alert.id.clone(), alert.clone());

        // Add to history
        let mut history = self.alert_history.write().await;
        history.push(alert.clone());

        // Send notifications
        self.notifier
            .send_notifications(&alert, &config.destinations)
            .await?;

        Ok(())
    }

    /// Acknowledge alert
    pub async fn acknowledge_alert(&self, alert_id: &str) -> BiomeResult<()> {
        let mut active_alerts = self.active_alerts.write().await;
        if let Some(alert) = active_alerts.get_mut(alert_id) {
            alert.acknowledged = true;
        }
        Ok(())
    }

    /// Resolve alert
    pub async fn resolve_alert(&self, alert_id: &str) -> BiomeResult<()> {
        let mut active_alerts = self.active_alerts.write().await;
        if let Some(alert) = active_alerts.get_mut(alert_id) {
            alert.resolved = true;
            alert.resolution_time = Some(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            );
        }
        Ok(())
    }

    /// Get active alerts
    pub async fn get_active_alerts(&self) -> HashMap<String, Alert> {
        self.active_alerts.read().await.clone()
    }

    /// Get alert history
    pub async fn get_alert_history(&self) -> Vec<Alert> {
        self.alert_history.read().await.clone()
    }
}

impl AlertEvaluator {
    /// Create a new alert evaluator
    pub fn new() -> Self {
        Self {
            evaluation_state: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Evaluate condition
    pub async fn evaluate_condition(
        &self,
        condition: &super::config::AlertCondition,
        value: f64,
    ) -> BiomeResult<bool> {
        let mut states = self.evaluation_state.write().await;
        let state = states
            .entry(condition.metric_name.clone())
            .or_insert_with(|| EvaluationState {
                last_evaluation: 0,
                consecutive_evaluations: 0,
                last_value: 0.0,
                history: Vec::new(),
            });

        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Check if condition is met
        let condition_met = match condition.operator {
            super::config::ComparisonOperator::GreaterThan => value > condition.threshold,
            super::config::ComparisonOperator::GreaterThanOrEqual => value >= condition.threshold,
            super::config::ComparisonOperator::LessThan => value < condition.threshold,
            super::config::ComparisonOperator::LessThanOrEqual => value <= condition.threshold,
            super::config::ComparisonOperator::Equal => {
                (value - condition.threshold).abs() < f64::EPSILON
            }
            super::config::ComparisonOperator::NotEqual => {
                (value - condition.threshold).abs() >= f64::EPSILON
            }
        };

        // Update evaluation state
        state.last_evaluation = current_time;
        state.last_value = value;
        state.history.push(EvaluationResult {
            timestamp: current_time,
            value,
            condition_met,
        });

        // Keep only recent history
        state
            .history
            .retain(|result| current_time - result.timestamp < condition.window);

        // Update consecutive evaluations
        if condition_met {
            state.consecutive_evaluations += 1;
        } else {
            state.consecutive_evaluations = 0;
        }

        // Check if we should trigger alert
        Ok(condition_met && state.consecutive_evaluations >= condition.consecutive_evaluations)
    }
}

impl AlertNotifier {
    /// Create a new alert notifier
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Send notifications
    pub async fn send_notifications(
        &self,
        alert: &Alert,
        destinations: &[AlertDestination],
    ) -> BiomeResult<()> {
        let channels = self.channels.read().await;

        for destination in destinations {
            let channel_name = destination.destination_type.to_string();
            if let Some(channel) = channels.get(&channel_name) {
                let result = channel.send_notification(alert, destination).await;

                let record = NotificationRecord {
                    id: uuid::Uuid::new_v4().to_string(),
                    alert_id: alert.id.clone(),
                    destination: destination.clone(),
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    successful: result.is_ok(),
                    error: result.err().map(|e| e.to_string()),
                };

                let mut history = self.history.write().await;
                history.push(record);
            }
        }

        Ok(())
    }

    /// Add notification channel
    pub async fn add_channel(&self, name: String, channel: Arc<dyn NotificationChannel>) {
        let mut channels = self.channels.write().await;
        channels.insert(name, channel);
    }

    /// Remove notification channel
    pub async fn remove_channel(&self, name: &str) {
        let mut channels = self.channels.write().await;
        channels.remove(name);
    }

    /// Get notification history
    pub async fn get_notification_history(&self) -> Vec<NotificationRecord> {
        self.history.read().await.clone()
    }
}

impl Default for AlertManager {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AlertEvaluator {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for AlertNotifier {
    fn default() -> Self {
        Self::new()
    }
}
