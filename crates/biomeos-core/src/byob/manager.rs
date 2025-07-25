//! BYOB Manager Implementation
//!
//! Core management functionality for BYOB workspaces

use crate::byob::{ByobManager, ResourceUsage};
use anyhow::Result;

/// Team metrics for monitoring
#[derive(Debug, Clone)]
pub struct TeamMetrics {
    pub team_id: String,
    pub resource_efficiency: f64,
    pub uptime_hours: u64,
}

impl ByobManager {
    /// Validate team resource usage against quota
    pub fn validate_resource_usage(
        &self,
        team_id: &str,
        requested_resources: &ResourceUsage,
    ) -> Result<bool> {
        if let Some(workspace) = self.teams.get(team_id) {
            let quota = &workspace.config.quota;

            // Check if requested resources would exceed quota
            let total_cpu =
                workspace.resource_usage.used_cpu_cores + requested_resources.used_cpu_cores;
            let total_memory =
                workspace.resource_usage.used_memory_gb + requested_resources.used_memory_gb;
            let total_storage =
                workspace.resource_usage.used_storage_gb + requested_resources.used_storage_gb;
            let total_bandwidth = workspace.resource_usage.used_bandwidth_mbps
                + requested_resources.used_bandwidth_mbps;

            if total_cpu > quota.max_cpu_cores
                || total_memory > quota.max_memory_gb
                || total_storage > quota.max_storage_gb
                || total_bandwidth > quota.max_bandwidth_mbps
            {
                return Ok(false);
            }

            Ok(true)
        } else {
            Err(anyhow::anyhow!("Team workspace not found: {}", team_id))
        }
    }

    /// Update team resource usage
    pub fn update_team_resource_usage(
        &mut self,
        team_id: &str,
        resource_delta: &ResourceUsage,
    ) -> Result<()> {
        if let Some(workspace) = self.teams.get_mut(team_id) {
            workspace.resource_usage.used_cpu_cores += resource_delta.used_cpu_cores;
            workspace.resource_usage.used_memory_gb += resource_delta.used_memory_gb;
            workspace.resource_usage.used_storage_gb += resource_delta.used_storage_gb;
            workspace.resource_usage.used_bandwidth_mbps += resource_delta.used_bandwidth_mbps;
            workspace.resource_usage.active_deployments += resource_delta.active_deployments;
            workspace.last_active = chrono::Utc::now();

            Ok(())
        } else {
            Err(anyhow::anyhow!("Team workspace not found: {}", team_id))
        }
    }

    /// Get team metrics for monitoring
    pub fn get_team_metrics(&self, team_id: &str) -> Option<TeamMetrics> {
        if let Some(workspace) = self.teams.get(team_id) {
            let uptime = chrono::Utc::now()
                .signed_duration_since(workspace.created_at)
                .num_hours() as u64;

            // Calculate resource efficiency (simplified)
            let quota = &workspace.config.quota;
            let usage = &workspace.resource_usage;

            let efficiency = ((usage.used_cpu_cores / quota.max_cpu_cores)
                + (usage.used_memory_gb / quota.max_memory_gb)
                + (usage.used_storage_gb / quota.max_storage_gb))
                / 3.0;

            Some(TeamMetrics {
                team_id: team_id.to_string(),
                resource_efficiency: efficiency,
                uptime_hours: uptime,
            })
        } else {
            None
        }
    }
}
