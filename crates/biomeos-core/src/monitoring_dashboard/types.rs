/// Alert severity enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Monitoring dashboard types

use crate::{
    primal_clients::CapabilityCategory,
    universal_biomeos_manager::EcosystemHealth,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Dashboard event types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DashboardEventType {
    SystemStarted,
    SystemStopped,
    EcosystemHealthUpdate,
    CapabilityUpdate,
    EcosystemRefreshed,
    AlertTriggered,
    AlertResolved,
    MetricsCollected,
    ErrorOccurred,
}

/// Dashboard event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardEvent {
    pub event_type: DashboardEventType,
    pub timestamp: DateTime<Utc>,
    pub details: String,
}

/// Dashboard metrics state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMetricsState {
    pub system_metrics: SystemMetrics,
    pub ecosystem_metrics: EcosystemMetrics,
    pub alert_metrics: AlertMetrics,
    pub last_updated: DateTime<Utc>,
}

/// System metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_io: NetworkMetrics,
    pub uptime: u64,
}

/// Network metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkMetrics {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
}

/// Ecosystem metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcosystemMetrics {
    pub health: EcosystemHealth,
    pub active_primals: u32,
    pub available_capabilities: HashMap<CapabilityCategory, u32>,
    pub total_services: u32,
    pub last_discovery: DateTime<Utc>,
}

/// Alert metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertMetrics {
    pub active_alerts: u32,
    pub resolved_alerts: u32,
    pub alert_rate: f64,
    pub last_alert: Option<DateTime<Utc>>,
}

/// Performance metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub response_time: f64,
    pub throughput: f64,
    pub error_rate: f64,
    pub availability: f64,
}

/// Resource utilization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilization {
    pub cpu_utilization: f64,
    pub memory_utilization: f64,
    pub disk_utilization: f64,
    pub network_utilization: f64,
}

/// Dashboard subscriber trait
pub trait DashboardSubscriber: Send + Sync {
    fn on_event(&self, event: &DashboardEvent);
}

impl DashboardMetricsState {
    pub fn new() -> Self {
        Self {
            system_metrics: SystemMetrics {
                cpu_usage: 0.0,
                memory_usage: 0.0,
                disk_usage: 0.0,
                network_io: NetworkMetrics {
                    bytes_sent: 0,
                    bytes_received: 0,
                    packets_sent: 0,
                    packets_received: 0,
                },
                uptime: 0,
            },
            ecosystem_metrics: EcosystemMetrics {
                health: EcosystemHealth::Unknown,
                active_primals: 0,
                available_capabilities: HashMap::new(),
                total_services: 0,
                last_discovery: Utc::now(),
            },
            alert_metrics: AlertMetrics {
                active_alerts: 0,
                resolved_alerts: 0,
                alert_rate: 0.0,
                last_alert: None,
            },
            last_updated: Utc::now(),
        }
    }
}

impl Default for DashboardMetricsState {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Unknown,
    Improving,
    Degrading,
    Up,
    Down,
    Stable,
}
