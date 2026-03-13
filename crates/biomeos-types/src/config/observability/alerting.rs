// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Alerting configuration types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::Duration;

/// Alerting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertingConfig {
    /// Enable alerting
    pub enabled: bool,
    /// Alert rules
    pub rules: Vec<AlertRule>,
    /// Notification channels
    pub channels: Vec<NotificationChannel>,
    /// Alert manager configuration
    pub manager: Option<AlertManagerConfig>,
}

/// Alert rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertRule {
    /// Rule name
    pub name: String,
    /// Rule expression
    pub expression: String,
    /// Rule severity
    pub severity: AlertSeverity,
    /// Evaluation interval
    pub interval: Duration,
    /// Alert duration
    pub duration: Duration,
    /// Rule labels
    pub labels: HashMap<String, String>,
    /// Rule annotations
    pub annotations: HashMap<String, String>,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum AlertSeverity {
    /// Informational alert
    Info,
    /// Warning-level alert
    Warning,
    /// Critical-level alert
    Critical,
    /// Emergency-level alert (requires immediate action)
    Emergency,
}

/// Notification channels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum NotificationChannel {
    /// Email notifications
    Email(EmailNotificationConfig),
    /// Slack notifications
    Slack(SlackNotificationConfig),
    /// Generic webhook notifications
    Webhook(WebhookNotificationConfig),
    /// PagerDuty notifications
    PagerDuty(PagerDutyNotificationConfig),
    /// Custom notification handler
    Custom(CustomNotificationConfig),
}

/// Email notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmailNotificationConfig {
    /// SMTP server
    pub smtp_server: String,
    /// SMTP port
    pub smtp_port: u16,
    /// SMTP username
    pub username: String,
    /// SMTP password
    pub password: String,
    /// From address
    pub from: String,
    /// To addresses
    pub to: Vec<String>,
    /// Subject template
    pub subject_template: String,
    /// Body template
    pub body_template: String,
}

/// Slack notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlackNotificationConfig {
    /// Webhook URL
    pub webhook_url: String,
    /// Channel
    pub channel: String,
    /// Username
    pub username: Option<String>,
    /// Message template
    pub message_template: String,
}

/// Webhook notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebhookNotificationConfig {
    /// Webhook URL
    pub url: String,
    /// HTTP method
    pub method: String,
    /// Headers
    pub headers: HashMap<String, String>,
    /// Body template
    pub body_template: String,
}

/// PagerDuty notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PagerDutyNotificationConfig {
    /// Integration key
    pub integration_key: String,
    /// Severity mapping
    pub severity_mapping: HashMap<AlertSeverity, String>,
}

/// Custom notification configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomNotificationConfig {
    /// Handler name
    pub handler: String,
    /// Configuration parameters
    pub config: HashMap<String, String>,
}

/// Alert manager configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertManagerConfig {
    /// Alert manager URL
    pub url: String,
    /// Authentication
    pub auth: Option<AlertManagerAuth>,
    /// Grouping configuration
    pub grouping: AlertGroupingConfig,
}

/// Alert manager authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertManagerAuth {
    /// Bearer token
    Bearer(String),
    /// HTTP basic auth
    Basic {
        /// Username
        username: String,
        /// Password
        password: String,
    },
}

/// Alert grouping configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertGroupingConfig {
    /// Group by labels
    pub group_by: Vec<String>,
    /// Group wait time
    pub group_wait: Duration,
    /// Group interval
    pub group_interval: Duration,
    /// Repeat interval
    pub repeat_interval: Duration,
}
