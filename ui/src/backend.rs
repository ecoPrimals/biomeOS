//! Backend integration module
//!
//! This module provides the backend integration layer for the biomeOS UI,
//! including live data services, event handling, and metrics collection.

use biomeos_core::integration::live_service::LiveService;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Live backend service for real-time data
pub struct LiveBackend {
    live_service: Arc<LiveService>,
    metrics: Arc<RwLock<DashboardMetrics>>,
    event_handlers: Vec<Box<dyn Fn(BackendEvent) + Send + Sync>>,
}

/// Backend events for UI updates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BackendEvent {
    /// System status changed
    SystemStatusChanged { status: String, timestamp: u64 },
    /// Primal discovered or updated
    PrimalDiscovered {
        primal_id: String,
        primal_type: String,
        endpoint: String,
    },
    /// Primal went offline
    PrimalOffline { primal_id: String, timestamp: u64 },
    /// Resource usage updated
    ResourceUsageUpdated {
        cpu_percent: f64,
        memory_percent: f64,
        disk_percent: f64,
    },
    /// Service health changed
    ServiceHealthChanged {
        service_id: String,
        old_health: String,
        new_health: String,
    },
    /// Deployment status changed
    DeploymentStatusChanged {
        deployment_id: String,
        status: String,
        progress: f64,
    },
}

/// Dashboard metrics for UI display
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardMetrics {
    /// System resource usage
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub disk_usage: f64,
    pub network_usage: NetworkUsage,

    /// Service statistics
    pub active_services: u32,
    pub healthy_services: u32,
    pub warning_services: u32,
    pub critical_services: u32,

    /// Primal statistics
    pub discovered_primals: u32,
    pub connected_primals: u32,
    pub offline_primals: u32,

    /// Deployment statistics
    pub active_deployments: u32,
    pub successful_deployments: u32,
    pub failed_deployments: u32,

    /// Performance metrics
    pub average_response_time: f64,
    pub requests_per_second: f64,
    pub error_rate: f64,

    /// Timestamp of last update
    pub last_updated: u64,
}

/// Network usage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkUsage {
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub packets_sent: u64,
    pub packets_received: u64,
    pub errors: u64,
}

impl Default for DashboardMetrics {
    fn default() -> Self {
        Self {
            cpu_usage: 0.0,
            memory_usage: 0.0,
            disk_usage: 0.0,
            network_usage: NetworkUsage {
                bytes_sent: 0,
                bytes_received: 0,
                packets_sent: 0,
                packets_received: 0,
                errors: 0,
            },
            active_services: 0,
            healthy_services: 0,
            warning_services: 0,
            critical_services: 0,
            discovered_primals: 0,
            connected_primals: 0,
            offline_primals: 0,
            active_deployments: 0,
            successful_deployments: 0,
            failed_deployments: 0,
            average_response_time: 0.0,
            requests_per_second: 0.0,
            error_rate: 0.0,
            last_updated: chrono::Utc::now().timestamp() as u64,
        }
    }
}

impl LiveBackend {
    /// Create a new live backend instance
    pub fn new(live_service: Arc<LiveService>) -> Self {
        Self {
            live_service,
            metrics: Arc::new(RwLock::new(DashboardMetrics::default())),
            event_handlers: Vec::new(),
        }
    }

    /// Get current dashboard metrics
    pub async fn get_metrics(&self) -> DashboardMetrics {
        self.metrics.read().await.clone()
    }

    /// Update dashboard metrics
    pub async fn update_metrics(&self, metrics: DashboardMetrics) {
        *self.metrics.write().await = metrics;
    }

    /// Add event handler
    pub fn add_event_handler<F>(&mut self, handler: F)
    where
        F: Fn(BackendEvent) + Send + Sync + 'static,
    {
        self.event_handlers.push(Box::new(handler));
    }

    /// Emit backend event
    pub fn emit_event(&self, event: BackendEvent) {
        for handler in &self.event_handlers {
            handler(event.clone());
        }
    }

    /// Get system status
    pub async fn get_system_status(&self) -> Option<HashMap<String, String>> {
        self.live_service.get_system_status().await
    }

    /// Get all workflow statuses
    pub async fn get_all_workflow_statuses(&self) -> HashMap<String, WorkflowStatus> {
        // Simulate workflow statuses
        let mut statuses = HashMap::new();

        statuses.insert(
            "gaming-tournament".to_string(),
            WorkflowStatus {
                id: "gaming-tournament".to_string(),
                name: "Gaming Tournament Platform".to_string(),
                state: "running".to_string(),
                progress: 85.0,
                current_step: "Deploying matchmaking service".to_string(),
                started_at: chrono::Utc::now().timestamp() as u64 - 3600,
                updated_at: chrono::Utc::now().timestamp() as u64,
            },
        );

        statuses.insert(
            "web-development".to_string(),
            WorkflowStatus {
                id: "web-development".to_string(),
                name: "Web Development Environment".to_string(),
                state: "completed".to_string(),
                progress: 100.0,
                current_step: "All services running".to_string(),
                started_at: chrono::Utc::now().timestamp() as u64 - 7200,
                updated_at: chrono::Utc::now().timestamp() as u64 - 600,
            },
        );

        statuses
    }

    /// Get discovered primals
    pub async fn get_discovered_primals(&self) -> HashMap<String, PrimalStatus> {
        // Simulate discovered primals
        let mut primals = HashMap::new();

        primals.insert(
            "toadstool".to_string(),
            PrimalStatus {
                id: "toadstool".to_string(),
                name: "ToadStool Compute".to_string(),
                primal_type: "Compute".to_string(),
                endpoint: "http://localhost:8080".to_string(),
                health: "Healthy".to_string(),
                capabilities: vec![
                    "container_runtime".to_string(),
                    "manifest_parsing".to_string(),
                ],
                last_seen: chrono::Utc::now().timestamp() as u64,
            },
        );

        primals.insert(
            "songbird".to_string(),
            PrimalStatus {
                id: "songbird".to_string(),
                name: "Songbird Orchestrator".to_string(),
                primal_type: "Orchestration".to_string(),
                endpoint: "http://localhost:8081".to_string(),
                health: "Healthy".to_string(),
                capabilities: vec![
                    "service_discovery".to_string(),
                    "load_balancing".to_string(),
                ],
                last_seen: chrono::Utc::now().timestamp() as u64,
            },
        );

        primals
    }

    /// Start background monitoring
    pub async fn start_monitoring(&self) {
        // Simulate background monitoring
        tokio::spawn(async {
            loop {
                tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
                // Update metrics, emit events, etc.
            }
        });
    }
}

/// Workflow status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkflowStatus {
    pub id: String,
    pub name: String,
    pub state: String,
    pub progress: f64,
    pub current_step: String,
    pub started_at: u64,
    pub updated_at: u64,
}

/// Primal status information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalStatus {
    pub id: String,
    pub name: String,
    pub primal_type: String,
    pub endpoint: String,
    pub health: String,
    pub capabilities: Vec<String>,
    pub last_seen: u64,
}
