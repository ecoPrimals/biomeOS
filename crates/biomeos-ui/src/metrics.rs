//! Metrics Collection and Display Module  
//!
//! This module provides utilities for collecting, aggregating, and displaying
//! system metrics using the unified ResourceMetrics from biomeos-types.

use crate::{ResourceMetrics, Health};
use std::collections::{HashMap, VecDeque};
use chrono::{DateTime, Utc};

/// Time series metrics storage
#[derive(Debug, Clone)]
pub struct TimeSeriesMetrics {
    /// Maximum number of data points to keep
    max_points: usize,
    /// Data points with timestamps
    data_points: VecDeque<(DateTime<Utc>, ResourceMetrics)>,
}

impl TimeSeriesMetrics {
    /// Create a new time series with specified capacity
    pub fn new(max_points: usize) -> Self {
        Self {
            max_points,
            data_points: VecDeque::with_capacity(max_points),
        }
    }
    
    /// Add a new data point
    pub fn add_point(&mut self, timestamp: DateTime<Utc>, metrics: ResourceMetrics) {
        self.data_points.push_back((timestamp, metrics));
        
        // Remove old points if we exceed capacity
        if self.data_points.len() > self.max_points {
            self.data_points.pop_front();
        }
    }
    
    /// Get all data points
    pub fn data_points(&self) -> &VecDeque<(DateTime<Utc>, ResourceMetrics)> {
        &self.data_points
    }
    
    /// Get data points within a time range
    pub fn data_points_in_range(&self, start: DateTime<Utc>, end: DateTime<Utc>) -> Vec<(DateTime<Utc>, ResourceMetrics)> {
        self.data_points
            .iter()
            .filter(|(timestamp, _)| *timestamp >= start && *timestamp <= end)
            .cloned()
            .collect()
    }
    
    /// Get the latest data point
    pub fn latest(&self) -> Option<&(DateTime<Utc>, ResourceMetrics)> {
        self.data_points.back()
    }
    
    /// Get average metrics over the time series
    pub fn average(&self) -> Option<ResourceMetrics> {
        if self.data_points.is_empty() {
            return None;
        }
        
        let mut cpu_sum = 0.0;
        let mut memory_sum = 0.0;
        let mut disk_sum = 0.0;
        let mut count = 0;
        
        for (_, metrics) in &self.data_points {
            if let Some(cpu) = metrics.cpu_usage {
                cpu_sum += cpu;
            }
            if let Some(memory) = metrics.memory_usage {
                memory_sum += memory;
            }
            if let Some(disk) = metrics.disk_usage {
                disk_sum += disk;
            }
            count += 1;
        }
        
        Some(ResourceMetrics {
            cpu_usage: Some(cpu_sum / count as f64),
            memory_usage: Some(memory_sum / count as f64),
            disk_usage: Some(disk_sum / count as f64),
            network_io: self.data_points.back().and_then(|(_, m)| m.network_io.clone()),
        })
    }
    
    /// Get peak usage values
    pub fn peak_usage(&self) -> Option<ResourceMetrics> {
        if self.data_points.is_empty() {
            return None;
        }
        
        let mut max_cpu: f64 = 0.0;
        let mut max_memory: f64 = 0.0;
        let mut max_disk: f64 = 0.0;
        
        for (_, metrics) in &self.data_points {
            if let Some(cpu) = metrics.cpu_usage {
                max_cpu = max_cpu.max(cpu);
            }
            if let Some(memory) = metrics.memory_usage {
                max_memory = max_memory.max(memory);
            }
            if let Some(disk) = metrics.disk_usage {
                max_disk = max_disk.max(disk);
            }
        }
        
        Some(ResourceMetrics {
            cpu_usage: Some(max_cpu),
            memory_usage: Some(max_memory),
            disk_usage: Some(max_disk),
            network_io: None,
        })
    }
}

/// Metrics aggregator for collecting metrics from multiple sources
#[derive(Debug)]
pub struct MetricsAggregator {
    /// Time series for different metric categories
    system_metrics: TimeSeriesMetrics,
    service_metrics: HashMap<String, TimeSeriesMetrics>,
    
    /// Alert thresholds
    cpu_warning_threshold: f64,
    cpu_critical_threshold: f64,
    memory_warning_threshold: f64,
    memory_critical_threshold: f64,
    disk_warning_threshold: f64,
    disk_critical_threshold: f64,
}

impl MetricsAggregator {
    /// Create a new metrics aggregator
    pub fn new() -> Self {
        Self {
            system_metrics: TimeSeriesMetrics::new(1000), // Keep 1000 points
            service_metrics: HashMap::new(),
            cpu_warning_threshold: 0.7,
            cpu_critical_threshold: 0.9,
            memory_warning_threshold: 0.8,
            memory_critical_threshold: 0.95,
            disk_warning_threshold: 0.8,
            disk_critical_threshold: 0.95,
        }
    }
    
    /// Add system metrics
    pub fn add_system_metrics(&mut self, metrics: ResourceMetrics) {
        self.system_metrics.add_point(Utc::now(), metrics);
    }
    
    /// Add metrics for a specific service
    pub fn add_service_metrics(&mut self, service_id: &str, metrics: ResourceMetrics) {
        let service_metrics = self.service_metrics
            .entry(service_id.to_string())
            .or_insert_with(|| TimeSeriesMetrics::new(500)); // Keep 500 points per service
        
        service_metrics.add_point(Utc::now(), metrics);
    }
    
    /// Get system metrics time series
    pub fn system_metrics(&self) -> &TimeSeriesMetrics {
        &self.system_metrics
    }
    
    /// Get metrics for a specific service
    pub fn service_metrics(&self, service_id: &str) -> Option<&TimeSeriesMetrics> {
        self.service_metrics.get(service_id)
    }
    
    /// Get all service IDs with metrics
    pub fn service_ids(&self) -> Vec<&String> {
        self.service_metrics.keys().collect()
    }
    
    /// Analyze current system health based on latest metrics
    pub fn analyze_system_health(&self) -> Health {
        let latest_metrics = match self.system_metrics.latest() {
            Some((_, metrics)) => metrics,
            None => return Health::Unknown { 
                reason: "No metrics available".to_string(),
                last_known: None 
            },
        };
        
        let cpu_usage = latest_metrics.cpu_usage.unwrap_or(0.0);
        let memory_usage = latest_metrics.memory_usage.unwrap_or(0.0);
        let disk_usage = latest_metrics.disk_usage.unwrap_or(0.0);
        
        // Check critical thresholds
        if cpu_usage >= self.cpu_critical_threshold ||
           memory_usage >= self.memory_critical_threshold ||
           disk_usage >= self.disk_critical_threshold {
            
            let mut issues = Vec::new();
            
            if cpu_usage >= self.cpu_critical_threshold {
                issues.push(biomeos_types::HealthIssue {
                    id: format!("cpu-critical-{}", Utc::now().timestamp()),
                    category: biomeos_types::HealthIssueCategory::Resource,
                    severity: biomeos_types::HealthIssueSeverity::Critical,
                    message: format!("CPU usage critical: {:.1}%", cpu_usage * 100.0),
                    detected_at: Utc::now(),
                    details: [("cpu_usage".to_string(), serde_json::json!(cpu_usage))].into_iter().collect(),
                    remediation: vec![
                        biomeos_types::RemediationAction {
                            id: format!("cpu-check-{}", chrono::Utc::now().timestamp()),
                            action_type: biomeos_types::RemediationActionType::Manual,
                            description: "Check for runaway processes".to_string(),
                            automated: false,
                            command: Some("Check for runaway processes".to_string()),
                            estimated_duration_secs: Some(300),
                        },
                        biomeos_types::RemediationAction {
                            id: format!("cpu-scale-{}", chrono::Utc::now().timestamp()),
                            action_type: biomeos_types::RemediationActionType::Manual,
                            description: "Consider scaling resources".to_string(),
                            automated: false,
                            command: Some("Consider scaling resources".to_string()),
                            estimated_duration_secs: Some(600),
                        },
                    ],
                });
            }
            
            if memory_usage >= self.memory_critical_threshold {
                issues.push(biomeos_types::HealthIssue {
                    id: format!("memory-critical-{}", Utc::now().timestamp()),
                    category: biomeos_types::HealthIssueCategory::Resource,
                    severity: biomeos_types::HealthIssueSeverity::Critical,
                    message: format!("Memory usage critical: {:.1}%", memory_usage * 100.0),
                    detected_at: Utc::now(),
                    details: [("memory_usage".to_string(), serde_json::json!(memory_usage))].into_iter().collect(),
                    remediation: vec![
                        biomeos_types::RemediationAction {
                            id: format!("memory-check-{}", chrono::Utc::now().timestamp()),
                            action_type: biomeos_types::RemediationActionType::Manual,
                            description: "Check for memory leaks".to_string(),
                            automated: false,
                            command: Some("Check for memory leaks".to_string()),
                            estimated_duration_secs: Some(300),
                        },
                        biomeos_types::RemediationAction {
                            id: format!("memory-restart-{}", chrono::Utc::now().timestamp()),
                            action_type: biomeos_types::RemediationActionType::Manual,
                            description: "Restart high-memory services".to_string(),
                            automated: false,
                            command: Some("Restart high-memory services".to_string()),
                            estimated_duration_secs: Some(120),
                        },
                    ],
                });
            }
            
            if disk_usage >= self.disk_critical_threshold {
                issues.push(biomeos_types::HealthIssue {
                    id: format!("disk-critical-{}", Utc::now().timestamp()),
                    category: biomeos_types::HealthIssueCategory::Resource,
                    severity: biomeos_types::HealthIssueSeverity::Critical,
                    message: format!("Disk usage critical: {:.1}%", disk_usage * 100.0),
                    detected_at: Utc::now(),
                    details: [("disk_usage".to_string(), serde_json::json!(disk_usage))].into_iter().collect(),
                    remediation: vec![
                        biomeos_types::RemediationAction {
                            id: format!("disk-cleanup-{}", chrono::Utc::now().timestamp()),
                            action_type: biomeos_types::RemediationActionType::Manual,
                            description: "Clean up temporary files".to_string(),
                            automated: false,
                            command: Some("Clean up temporary files".to_string()),
                            estimated_duration_secs: Some(180),
                        },
                        biomeos_types::RemediationAction {
                            id: format!("disk-archive-{}", chrono::Utc::now().timestamp()),
                            action_type: biomeos_types::RemediationActionType::Manual,
                            description: "Archive old logs".to_string(),
                            automated: false,
                            command: Some("Archive old logs".to_string()),
                            estimated_duration_secs: Some(300),
                        },
                    ],
                });
            }
            
            return Health::Critical {
                issues,
                affected_capabilities: vec!["compute".to_string(), "storage".to_string()],
            };
        }
        
        // Check warning thresholds
        if cpu_usage >= self.cpu_warning_threshold ||
           memory_usage >= self.memory_warning_threshold ||
           disk_usage >= self.disk_warning_threshold {
            
            let mut issues = Vec::new();
            let mut impact_score = 0.0;
            
            if cpu_usage >= self.cpu_warning_threshold {
                issues.push(biomeos_types::HealthIssue {
                    id: format!("cpu-warning-{}", Utc::now().timestamp()),
                    category: biomeos_types::HealthIssueCategory::Performance,
                    severity: biomeos_types::HealthIssueSeverity::Medium,
                    message: format!("CPU usage elevated: {:.1}%", cpu_usage * 100.0),
                    detected_at: Utc::now(),
                    details: [("cpu_usage".to_string(), serde_json::json!(cpu_usage))].into_iter().collect(),
                    remediation: vec![biomeos_types::RemediationAction {
                        id: format!("cpu-monitor-{}", chrono::Utc::now().timestamp()),
                        action_type: biomeos_types::RemediationActionType::Restart,
                        description: "Monitor CPU usage trends".to_string(),
                        automated: true,
                        command: Some("Monitor CPU usage trends".to_string()),
                        estimated_duration_secs: Some(3600),
                    }],
                });
                impact_score += 0.3;
            }
            
            if memory_usage >= self.memory_warning_threshold {
                issues.push(biomeos_types::HealthIssue {
                    id: format!("memory-warning-{}", Utc::now().timestamp()),
                    category: biomeos_types::HealthIssueCategory::Performance,
                    severity: biomeos_types::HealthIssueSeverity::Medium,
                    message: format!("Memory usage elevated: {:.1}%", memory_usage * 100.0),
                    detected_at: Utc::now(),
                    details: [("memory_usage".to_string(), serde_json::json!(memory_usage))].into_iter().collect(),
                    remediation: vec![biomeos_types::RemediationAction {
                        id: format!("memory-monitor-{}", chrono::Utc::now().timestamp()),
                        action_type: biomeos_types::RemediationActionType::Restart,
                        description: "Monitor memory usage trends".to_string(),
                        automated: true,
                        command: Some("Monitor memory usage trends".to_string()),
                        estimated_duration_secs: Some(3600),
                    }],
                });
                impact_score += 0.2;
            }
            
            if disk_usage >= self.disk_warning_threshold {
                issues.push(biomeos_types::HealthIssue {
                    id: format!("disk-warning-{}", Utc::now().timestamp()),
                    category: biomeos_types::HealthIssueCategory::Performance,
                    severity: biomeos_types::HealthIssueSeverity::Medium,
                    message: format!("Disk usage elevated: {:.1}%", disk_usage * 100.0),
                    detected_at: Utc::now(),
                    details: [("disk_usage".to_string(), serde_json::json!(disk_usage))].into_iter().collect(),
                    remediation: vec![biomeos_types::RemediationAction {
                        id: format!("disk-monitor-{}", chrono::Utc::now().timestamp()),
                        action_type: biomeos_types::RemediationActionType::Restart,
                        description: "Monitor disk usage trends".to_string(),
                        automated: true,
                        command: Some("Monitor disk usage trends".to_string()),
                        estimated_duration_secs: Some(3600),
                    }],
                });
                impact_score += 0.1;
            }
            
            return Health::Degraded {
                issues,
                impact_score: Some(impact_score),
            };
        }
        
        Health::Healthy
    }
    
    /// Set alert thresholds
    pub fn set_thresholds(
        &mut self,
        cpu_warning: f64,
        cpu_critical: f64,
        memory_warning: f64,
        memory_critical: f64,
        disk_warning: f64,
        disk_critical: f64,
    ) {
        self.cpu_warning_threshold = cpu_warning;
        self.cpu_critical_threshold = cpu_critical;
        self.memory_warning_threshold = memory_warning;
        self.memory_critical_threshold = memory_critical;
        self.disk_warning_threshold = disk_warning;
        self.disk_critical_threshold = disk_critical;
    }
    
    /// Get metrics summary for display
    pub fn get_summary(&self) -> MetricsSummary {
        let current = self.system_metrics.latest().map(|(_, m)| m.clone());
        let average = self.system_metrics.average();
        let peak = self.system_metrics.peak_usage();
        
        MetricsSummary {
            current_metrics: current,
            average_metrics: average,
            peak_metrics: peak,
            data_points_count: self.system_metrics.data_points.len(),
            health: self.analyze_system_health(),
        }
    }
}

impl Default for MetricsAggregator {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary of metrics for UI display
#[derive(Debug, Clone)]
pub struct MetricsSummary {
    pub current_metrics: Option<ResourceMetrics>,
    pub average_metrics: Option<ResourceMetrics>,
    pub peak_metrics: Option<ResourceMetrics>,
    pub data_points_count: usize,
    pub health: Health,
}

/// Utility functions for metrics formatting
pub mod formatting {
    /// Format percentage with 1 decimal place
    pub fn format_percentage(value: Option<f64>) -> String {
        match value {
            Some(v) => format!("{:.1}%", v * 100.0),
            None => "N/A".to_string(),
        }
    }
    
    /// Format bytes with appropriate units
    pub fn format_bytes(bytes: f64) -> String {
        use biomeos_types::files::SIZE_UNITS;
        let mut size = bytes;
        let mut unit_index = 0;
        
        while size >= 1024.0 && unit_index < SIZE_UNITS.len() - 1 {
            size /= 1024.0;
            unit_index += 1;
        }
        
        format!("{:.1} {}", size, SIZE_UNITS[unit_index])
    }
    
    /// Format network throughput 
    pub fn format_throughput(bytes_per_sec: f64) -> String {
        format!("{}/s", format_bytes(bytes_per_sec))
    }
    
    /// Format duration in human readable form
    pub fn format_uptime(seconds: u64) -> String {
        let days = seconds / 86400;
        let hours = (seconds % 86400) / 3600;
        let minutes = (seconds % 3600) / 60;
        
        if days > 0 {
            format!("{}d {}h {}m", days, hours, minutes)
        } else if hours > 0 {
            format!("{}h {}m", hours, minutes)
        } else {
            format!("{}m", minutes)
        }
    }
} 