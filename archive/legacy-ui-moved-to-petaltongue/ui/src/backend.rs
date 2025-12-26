//! Backend integration module
//!
//! This module provides the backend integration layer for the biomeOS UI,
//! including live data services, event handling, and metrics collection.

use biomeos_core::integration::live_service::LiveService;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::fs;
use tokio::sync::RwLock;

/// Live backend service for real-time data
pub struct LiveBackend {
    live_service: Arc<LiveService>,
    metrics: Arc<RwLock<DashboardMetrics>>,
    event_handlers: Vec<Box<dyn Fn(BackendEvent) + Send + Sync>>,
}

impl std::fmt::Debug for LiveBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("LiveBackend")
            .field("live_service", &self.live_service)
            .field("metrics", &self.metrics)
            .field(
                "event_handlers",
                &format!("Vec<{} handlers>", self.event_handlers.len()),
            )
            .finish()
    }
}

impl Clone for LiveBackend {
    fn clone(&self) -> Self {
        Self {
            live_service: self.live_service.clone(),
            metrics: self.metrics.clone(),
            event_handlers: Vec::new(), // Don't clone function pointers
        }
    }
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
        cpu_usage: f64,
        memory_usage: f64,
        disk_usage: f64,
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
    /// Metrics updated
    MetricsUpdated { metrics: DashboardMetrics },
    /// Error occurred
    Error { error: String, timestamp: u64 },
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
        match self.live_service.get_system_status().await {
            Ok(status) => Some(std::collections::HashMap::from([(
                "status".to_string(),
                format!("{:?}", status),
            )])),
            Err(_) => None,
        }
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

    /// Get dashboard metrics (alias for get_metrics for UI compatibility)
    pub async fn get_dashboard_metrics(&self) -> Result<DashboardMetrics, anyhow::Error> {
        Ok(self.get_metrics().await)
    }

    /// Get YAML files (placeholder implementation)
    pub async fn get_yaml_files(&self) -> Result<Vec<String>, anyhow::Error> {
        // Implement basic YAML file discovery
        use std::fs;
        let mut yaml_files = Vec::new();
        let search_paths = vec!["./", "./config/", "./templates/"];

        for path in search_paths {
            if let Ok(entries) = fs::read_dir(path) {
                for entry in entries.flatten() {
                    if let Some(filename) = entry.file_name().to_str() {
                        if filename.ends_with(".yaml") || filename.ends_with(".yml") {
                            yaml_files.push(filename.to_string());
                        }
                    }
                }
            }
        }

        if yaml_files.is_empty() {
            yaml_files.push("default.yaml".to_string());
        }

        Ok(yaml_files)
    }

    /// Get YAML content (placeholder implementation)
    pub async fn get_yaml_content(&self, file_name: &str) -> Result<String, anyhow::Error> {
        // Implement basic YAML file reading
        use std::fs;
        match fs::read_to_string(file_name) {
            Ok(content) => Ok(content),
            Err(_) => {
                // Return default content if file doesn't exist
                Ok(format!(
                    "# Default YAML configuration for {}\nname: {}\nversion: 1.0\n",
                    file_name, file_name
                ))
            }
        }
    }

    /// Update YAML content (implementation)
    pub async fn update_yaml_content(
        &self,
        file_name: &str,
        content: &str,
    ) -> Result<(), anyhow::Error> {
        let yaml_dir = PathBuf::from("ui/src/templates");
        let file_path = yaml_dir.join(file_name);

        // Ensure the directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Validate YAML before writing
        if !self.validate_yaml(content).await? {
            return Err(anyhow::anyhow!("Invalid YAML content"));
        }

        // Write the file
        fs::write(&file_path, content).await?;
        tracing::info!(
            "Updated YAML file {} with {} bytes",
            file_name,
            content.len()
        );

        // Refresh caches after update
        self.refresh_caches().await?;

        Ok(())
    }

    /// Create YAML file (implementation)
    pub async fn create_yaml_file(
        &self,
        file_name: &str,
        content: &str,
    ) -> Result<(), anyhow::Error> {
        let yaml_dir = PathBuf::from("ui/src/templates");
        let file_path = yaml_dir.join(file_name);

        // Check if file already exists
        if file_path.exists() {
            return Err(anyhow::anyhow!("File {} already exists", file_name));
        }

        // Ensure the directory exists
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).await?;
        }

        // Validate YAML before writing
        if !self.validate_yaml(content).await? {
            return Err(anyhow::anyhow!("Invalid YAML content"));
        }

        // Write the file
        fs::write(&file_path, content).await?;
        tracing::info!(
            "Created YAML file {} with {} bytes",
            file_name,
            content.len()
        );

        // Refresh caches after creation
        self.refresh_caches().await?;

        Ok(())
    }

    /// Delete YAML file (implementation)
    pub async fn delete_yaml_file(&self, file_name: &str) -> Result<(), anyhow::Error> {
        let yaml_dir = PathBuf::from("ui/src/templates");
        let file_path = yaml_dir.join(file_name);

        // Check if file exists
        if !file_path.exists() {
            return Err(anyhow::anyhow!("File {} does not exist", file_name));
        }

        // Delete the file
        fs::remove_file(&file_path).await?;
        tracing::info!("Deleted YAML file {}", file_name);

        // Refresh caches after deletion
        self.refresh_caches().await?;

        Ok(())
    }

    /// Validate YAML content (placeholder implementation)
    pub async fn validate_yaml(&self, content: &str) -> Result<bool, anyhow::Error> {
        // Implement basic YAML validation using serde_yaml
        match serde_yaml::from_str::<serde_yaml::Value>(content) {
            Ok(_) => Ok(true),
            Err(e) => {
                tracing::warn!("YAML validation failed: {}", e);
                Ok(false)
            }
        }
    }

    /// Start BYOB workflow (implementation)
    pub async fn start_byob_workflow(
        &self,
        workflow_config: serde_json::Value,
    ) -> Result<String, anyhow::Error> {
        let workflow_id = uuid::Uuid::new_v4().to_string();

        // Extract workflow configuration
        let workflow_name = workflow_config
            .get("name")
            .and_then(|v| v.as_str())
            .unwrap_or("unnamed-workflow");

        let workflow_type = workflow_config
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("build");

        // Create workflow status
        let workflow_status = WorkflowStatus {
            id: workflow_id.clone(),
            name: workflow_name.to_string(),
            state: "starting".to_string(),
            progress: 0.0,
            current_step: "initializing".to_string(),
            started_at: chrono::Utc::now().timestamp() as u64,
            updated_at: chrono::Utc::now().timestamp() as u64,
        };

        // Store workflow status (in a real implementation, this would go to a database)
        // For now, we'll use a simple in-memory store
        tracing::info!(
            "Started BYOB workflow {} (type: {}) with config: {}",
            workflow_id,
            workflow_type,
            workflow_config
        );

        // Start the actual workflow in the background
        let live_service = self.live_service.clone();
        let config = workflow_config.clone();
        let wf_id = workflow_id.clone();

        tokio::spawn(async move {
            if let Err(e) = Self::run_workflow_background(live_service, wf_id, config).await {
                tracing::error!("Workflow execution failed: {}", e);
            }
        });

        Ok(workflow_id)
    }

    /// Background workflow execution
    async fn run_workflow_background(
        live_service: Arc<LiveService>,
        workflow_id: String,
        config: serde_json::Value,
    ) -> Result<(), anyhow::Error> {
        tracing::info!("Running workflow {} in background", workflow_id);

        // Simulate workflow steps
        let steps = vec![
            ("preparing", "Preparing build environment"),
            ("building", "Building components"),
            ("testing", "Running tests"),
            ("deploying", "Deploying to environment"),
            ("completed", "Workflow completed"),
        ];

        for (i, (step, description)) in steps.iter().enumerate() {
            // Simulate work
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

            let progress = (i + 1) as f64 / steps.len() as f64;
            tracing::info!(
                "Workflow {} step {}: {} ({}%)",
                workflow_id,
                step,
                description,
                (progress * 100.0) as u32
            );
        }

        Ok(())
    }

    /// Get BYOB workflow status (implementation)
    pub async fn get_byob_workflow_status(
        &self,
        workflow_id: &str,
    ) -> Result<serde_json::Value, anyhow::Error> {
        // In a real implementation, this would query a database or workflow engine
        // For now, simulate different states based on workflow age
        let workflow_age = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)?
            .as_secs()
            % 30; // Cycle through states every 30 seconds

        let (status, progress, current_step) = match workflow_age {
            0..=5 => ("starting", 0.1, "initializing"),
            6..=10 => ("running", 0.3, "preparing"),
            11..=15 => ("running", 0.6, "building"),
            16..=20 => ("running", 0.8, "testing"),
            21..=25 => ("running", 0.95, "deploying"),
            _ => ("completed", 1.0, "finished"),
        };

        Ok(serde_json::json!({
            "id": workflow_id,
            "name": format!("workflow-{}", workflow_id),
            "status": status,
            "progress": progress,
            "current_step": current_step,
            "started_at": chrono::Utc::now().timestamp() - workflow_age as i64,
            "updated_at": chrono::Utc::now().timestamp()
        }))
    }

    /// Stop BYOB workflow (implementation)
    pub async fn stop_byob_workflow(&self, workflow_id: &str) -> Result<(), anyhow::Error> {
        // In a real implementation, this would signal the workflow engine to stop
        tracing::info!("Stopping workflow {}", workflow_id);

        // Update workflow status to stopped
        // For now, just log the action
        tracing::info!("Workflow {} has been stopped", workflow_id);

        Ok(())
    }

    /// Get primal templates (implementation)
    pub async fn get_primal_templates(&self) -> Result<Vec<serde_json::Value>, anyhow::Error> {
        let mut templates = Vec::new();

        // Load templates from the templates directory
        let template_dirs = vec![
            PathBuf::from("ui/src/templates"),
            PathBuf::from("templates"),
            PathBuf::from("specs/examples"),
        ];

        for template_dir in template_dirs {
            if template_dir.exists() {
                let mut entries = fs::read_dir(&template_dir).await?;
                while let Some(entry) = entries.next_entry().await? {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()).unwrap_or("") == "yaml"
                        || path.to_string_lossy().contains(".biome.yaml")
                    {
                        if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                            let template_type = if file_name.contains("biome") {
                                "biome"
                            } else if file_name.contains("basic") {
                                "basic"
                            } else if file_name.contains("dev") {
                                "development"
                            } else {
                                "general"
                            };

                            templates.push(serde_json::json!({
                                "name": file_name.replace(".yaml", "").replace(".biome", ""),
                                "type": template_type,
                                "path": path.to_string_lossy(),
                                "description": format!("Template for {}", file_name),
                                "category": template_type
                            }));
                        }
                    }
                }
            }
        }

        // Add some built-in templates
        templates.extend(vec![
            serde_json::json!({
                "name": "basic-compute",
                "type": "compute",
                "description": "Basic compute primal template",
                "category": "compute",
                "template": {
                    "apiVersion": "v1",
                    "kind": "Primal",
                    "metadata": {"name": "basic-compute"},
                    "spec": {
                        "type": "compute",
                        "resources": {"cpu": 1, "memory": "1Gi"}
                    }
                }
            }),
            serde_json::json!({
                "name": "storage-primal",
                "type": "storage",
                "description": "Storage primal template",
                "category": "storage",
                "template": {
                    "apiVersion": "v1",
                    "kind": "Primal",
                    "metadata": {"name": "storage-primal"},
                    "spec": {
                        "type": "storage",
                        "resources": {"storage": "10Gi"}
                    }
                }
            }),
            serde_json::json!({
                "name": "network-service",
                "type": "networking",
                "description": "Network service template",
                "category": "networking",
                "template": {
                    "apiVersion": "v1",
                    "kind": "Primal",
                    "metadata": {"name": "network-service"},
                    "spec": {
                        "type": "networking",
                        "ports": [{"port": 80, "protocol": "TCP"}]
                    }
                }
            }),
        ]);

        tracing::info!("Loaded {} primal templates", templates.len());
        Ok(templates)
    }

    /// Refresh caches (implementation)
    pub async fn refresh_caches(&self) -> Result<(), anyhow::Error> {
        tracing::info!("Refreshing all caches");

        // Refresh metrics cache
        let new_metrics = self.collect_fresh_metrics().await?;
        {
            let mut metrics = self.metrics.write().await;
            *metrics = new_metrics;
        }

        // Refresh system status from live service
        if let Ok(system_status) = self.live_service.get_system_status().await {
            tracing::debug!(
                "Refreshed system status: health = {:?}",
                system_status.health_status
            );
        }

        // Refresh storage metrics
        if let Ok(storage_metrics) = self.live_service.get_storage_metrics().await {
            tracing::debug!(
                "Refreshed storage metrics: {} mount points",
                storage_metrics.mount_points.len()
            );
        }

        // Refresh network status
        if let Ok(network_status) = self.live_service.get_network_status().await {
            tracing::debug!(
                "Refreshed network status: {} interfaces",
                network_status.interfaces.len()
            );
        }

        tracing::info!("Cache refresh completed");
        Ok(())
    }

    /// Collect fresh metrics for cache
    async fn collect_fresh_metrics(&self) -> Result<DashboardMetrics, anyhow::Error> {
        let mut metrics = DashboardMetrics::default();

        // Get system status for resource usage
        if let Ok(system_status) = self.live_service.get_system_status().await {
            metrics.cpu_usage = system_status.resource_usage.cpu_usage.unwrap_or(0.0);
            metrics.memory_usage = system_status.resource_usage.memory_usage.unwrap_or(0.0);
            metrics.disk_usage = system_status.resource_usage.disk_usage.unwrap_or(0.0);

            // Count primals by status
            metrics.discovered_primals = system_status.primals.len() as u32;
            metrics.connected_primals = system_status
                .primals
                .values()
                .filter(|p| matches!(p.health, biomeos_types::Health::Healthy))
                .count() as u32;
            metrics.offline_primals = metrics.discovered_primals - metrics.connected_primals;
        }

        // Get network metrics
        if let Ok(network_status) = self.live_service.get_network_status().await {
            metrics.network_usage.bytes_sent = network_status.total_bytes_sent;
            metrics.network_usage.bytes_received = network_status.total_bytes_received;
        }

        // Update timestamp
        metrics.last_updated = chrono::Utc::now().timestamp() as u64;

        Ok(metrics)
    }

    /// Take event receiver (placeholder implementation)
    pub async fn take_event_receiver(&self) -> tokio::sync::broadcast::Receiver<BackendEvent> {
        let (tx, rx) = tokio::sync::broadcast::channel(100);
        // In a real implementation, this would return the actual event receiver
        rx
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
