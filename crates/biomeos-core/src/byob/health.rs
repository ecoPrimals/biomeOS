//! BYOB (Bring Your Own Biome) health reporting
//!
//! This module contains health reporting functionality for BYOB deployments,
//! including team health reports and health-related utilities.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::HealthStatus;
use super::types::{ResourceUsage, ResourceQuota};

/// Team health report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TeamHealthReport {
    pub team_id: String,
    pub overall_health: HealthStatus,
    pub healthy_deployments: usize,
    pub total_deployments: usize,
    pub deployment_health: HashMap<String, HealthStatus>,
    pub resource_usage: ResourceUsage,
    pub resource_quota: ResourceQuota,
    pub generated_at: DateTime<Utc>,
}

impl TeamHealthReport {
    /// Create a new team health report
    pub fn new(
        team_id: String,
        overall_health: HealthStatus,
        healthy_deployments: usize,
        total_deployments: usize,
        deployment_health: HashMap<String, HealthStatus>,
        resource_usage: ResourceUsage,
        resource_quota: ResourceQuota,
    ) -> Self {
        Self {
            team_id,
            overall_health,
            healthy_deployments,
            total_deployments,
            deployment_health,
            resource_usage,
            resource_quota,
            generated_at: Utc::now(),
        }
    }

    /// Get health percentage for the team
    pub fn get_health_percentage(&self) -> f64 {
        if self.total_deployments == 0 {
            return 100.0;
        }
        (self.healthy_deployments as f64 / self.total_deployments as f64) * 100.0
    }

    /// Check if team is healthy
    pub fn is_healthy(&self) -> bool {
        matches!(self.overall_health, HealthStatus::Healthy)
    }

    /// Check if team has critical issues
    pub fn is_critical(&self) -> bool {
        matches!(self.overall_health, HealthStatus::Critical)
    }

    /// Get failed deployment count
    pub fn get_failed_deployments(&self) -> usize {
        self.deployment_health
            .values()
            .filter(|&status| matches!(status, HealthStatus::Critical))
            .count()
    }

    /// Get warning deployment count
    pub fn get_warning_deployments(&self) -> usize {
        self.deployment_health
            .values()
            .filter(|&status| matches!(status, HealthStatus::Warning))
            .count()
    }

    /// Get resource utilization percentage
    pub fn get_resource_utilization(&self) -> ResourceUtilizationReport {
        let cpu_utilization = if self.resource_quota.max_cpu_cores > 0.0 {
            (self.resource_usage.cpu_cores / self.resource_quota.max_cpu_cores) * 100.0
        } else {
            0.0
        };

        let memory_utilization = if self.resource_quota.max_memory_bytes > 0 {
            (self.resource_usage.memory_bytes as f64 / self.resource_quota.max_memory_bytes as f64) * 100.0
        } else {
            0.0
        };

        let storage_utilization = if self.resource_quota.max_storage_bytes > 0 {
            (self.resource_usage.storage_bytes as f64 / self.resource_quota.max_storage_bytes as f64) * 100.0
        } else {
            0.0
        };

        let deployment_utilization = if self.resource_quota.max_deployments > 0 {
            (self.resource_usage.active_deployments as f64 / self.resource_quota.max_deployments as f64) * 100.0
        } else {
            0.0
        };

        ResourceUtilizationReport {
            cpu_utilization,
            memory_utilization,
            storage_utilization,
            deployment_utilization,
        }
    }
}

/// Resource utilization report
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceUtilizationReport {
    pub cpu_utilization: f64,
    pub memory_utilization: f64,
    pub storage_utilization: f64,
    pub deployment_utilization: f64,
}

impl ResourceUtilizationReport {
    /// Check if any resource is over-utilized (>80%)
    pub fn is_over_utilized(&self) -> bool {
        self.cpu_utilization > 80.0 ||
        self.memory_utilization > 80.0 ||
        self.storage_utilization > 80.0 ||
        self.deployment_utilization > 80.0
    }

    /// Get highest utilization percentage
    pub fn get_highest_utilization(&self) -> f64 {
        [
            self.cpu_utilization,
            self.memory_utilization,
            self.storage_utilization,
            self.deployment_utilization,
        ]
        .iter()
        .cloned()
        .fold(0.0, f64::max)
    }

    /// Get resource utilization status
    pub fn get_utilization_status(&self) -> HealthStatus {
        let max_utilization = self.get_highest_utilization();
        
        if max_utilization > 90.0 {
            HealthStatus::Critical
        } else if max_utilization > 80.0 {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        }
    }
} 