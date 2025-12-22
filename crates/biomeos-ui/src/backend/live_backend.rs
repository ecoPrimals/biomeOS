use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use anyhow::Result;
use tracing::{info, warn};
use std::path::PathBuf;
use tokio::fs;

use biomeos_core::integration::{
    LiveService, SystemStatus, NetworkInterface
};

/// Live UI Backend - connects to real biomeOS integration service
pub struct LiveBackend {
    integration_service: Arc<LiveService>,
    system_status_cache: RwLock<Option<SystemStatus>>,
    yaml_content_cache: RwLock<HashMap<String, String>>,
    event_sender: mpsc::UnboundedSender<BackendEvent>,
    event_receiver: RwLock<Option<mpsc::UnboundedReceiver<BackendEvent>>>,
}

#[derive(Debug, Clone)]
pub enum BackendEvent {
    SystemStatusUpdated(SystemStatus),
    YamlFileChanged(String),
    Error(String),
}

impl LiveBackend {
    /// Create new live backend
    pub async fn new() -> Result<Arc<Self>> {
        info!("🚀 Initializing LIVE UI Backend");
        
        // Initialize live integration service
        let integration_service = Arc::new(
            LiveService::new().await?
        );
        
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        let backend = Arc::new(Self {
            integration_service,
            system_status_cache: RwLock::new(None),
            yaml_content_cache: RwLock::new(HashMap::new()),
            event_sender,
            event_receiver: RwLock::new(Some(event_receiver)),
        });

        // Start background system monitoring
        let backend_clone = backend.clone();
        tokio::spawn(async move {
            backend_clone.start_monitoring_tasks().await;
        });

        info!("✅ LIVE UI Backend initialized successfully");
        Ok(backend)
    }

    /// Start monitoring tasks
    async fn start_monitoring_tasks(&self) {
        info!("Starting system monitoring tasks");
        
        loop {
            // Get system status periodically
            if let Ok(status) = self.integration_service.get_system_status().await {
                // Update cache
                {
                    let mut cache = self.system_status_cache.write().await;
                    *cache = Some(status.clone());
                }
                
                // Send event
                if self.event_sender.send(BackendEvent::SystemStatusUpdated(status)).is_err() {
                    warn!("Failed to send system status update event");
                }
            }
            
            // Wait before next check
            tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        }
    }

    /// Get system status
    pub async fn get_system_status(&self) -> Result<SystemStatus> {
        self.integration_service.get_system_status().await
    }

    /// Get discovered primals
    pub async fn get_discovered_primals(&self) -> Vec<String> {
        self.integration_service.get_discovered_primals().await
    }

    /// Get YAML files from templates directory
    pub async fn get_yaml_files(&self) -> Result<HashMap<String, String>> {
        let mut yaml_files = HashMap::new();
        let yaml_dir = PathBuf::from("ui/src/templates");
        
        if yaml_dir.exists() {
            let mut entries = fs::read_dir(&yaml_dir).await?;
            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("yaml") {
                    if let Some(file_name) = path.file_name().and_then(|n| n.to_str()) {
                        if let Ok(content) = fs::read_to_string(&path).await {
                            yaml_files.insert(file_name.to_string(), content);
                        }
                    }
                }
            }
        }
        
        Ok(yaml_files)
    }

    /// Get YAML file content
    pub async fn get_yaml_content(&self, file_name: &str) -> Result<String> {
        let yaml_dir = PathBuf::from("ui/src/templates");
        let file_path = yaml_dir.join(file_name);
        
        if file_path.exists() {
            Ok(fs::read_to_string(&file_path).await?)
        } else {
            Err(anyhow::anyhow!("File {} not found", file_name))
        }
    }

    /// Update YAML file content
    pub async fn update_yaml_content(&self, file_name: &str, content: &str) -> Result<()> {
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
        info!("Updated YAML file {} with {} bytes", file_name, content.len());
        
        // Send event
        if self.event_sender.send(BackendEvent::YamlFileChanged(file_name.to_string())).is_err() {
            warn!("Failed to send YAML file changed event");
        }
        
        Ok(())
    }

    /// Create YAML file
    pub async fn create_yaml_file(&self, file_name: &str, content: &str) -> Result<()> {
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
        info!("Created YAML file {} with {} bytes", file_name, content.len());
        
        // Send event
        if self.event_sender.send(BackendEvent::YamlFileChanged(file_name.to_string())).is_err() {
            warn!("Failed to send YAML file changed event");
        }
        
        Ok(())
    }

    /// Delete YAML file
    pub async fn delete_yaml_file(&self, file_name: &str) -> Result<()> {
        let yaml_dir = PathBuf::from("ui/src/templates");
        let file_path = yaml_dir.join(file_name);

        // Check if file exists
        if !file_path.exists() {
            return Err(anyhow::anyhow!("File {} does not exist", file_name));
        }

        // Delete the file
        fs::remove_file(&file_path).await?;
        info!("Deleted YAML file {}", file_name);
        
        // Send event
        if self.event_sender.send(BackendEvent::YamlFileChanged(file_name.to_string())).is_err() {
            warn!("Failed to send YAML file changed event");
        }
        
        Ok(())
    }

    /// Get dashboard metrics (alias for refresh_and_get_metrics)
    pub async fn get_dashboard_metrics(&self) -> Result<DashboardMetrics> {
        self.refresh_and_get_metrics().await
    }

    /// Make validate_yaml public
    pub async fn validate_yaml(&self, content: &str) -> Result<bool> {
        // Simple YAML validation - try to parse it
        match serde_yaml::from_str::<serde_yaml::Value>(content) {
            Ok(_) => Ok(true),
            Err(_) => Ok(false),
        }
    }

    /// Placeholder for workflow methods - simplified for UI compatibility
    pub async fn start_byob_workflow(&self, _workflow_config: serde_json::Value) -> Result<String> {
        // Return a mock workflow ID for now
        Ok(uuid::Uuid::new_v4().to_string())
    }

    pub async fn get_byob_workflow_status(&self, _workflow_id: &str) -> Result<serde_json::Value> {
        // Return a mock status
        Ok(serde_json::json!({"status": "running", "progress": 50}))
    }

    pub async fn stop_byob_workflow(&self, _workflow_id: &str) -> Result<()> {
        // Mock stop - always succeeds
        Ok(())
    }

    pub async fn get_all_workflow_statuses(&self) -> Vec<(String, serde_json::Value)> {
        // Return empty list for now
        vec![]
    }

    pub async fn get_primal_templates(&self) -> Result<Vec<String>> {
        // Return mock template list
        Ok(vec![
            "basic_service.yaml".to_string(),
            "web_service.yaml".to_string(),
            "database.yaml".to_string(),
        ])
    }

    pub async fn refresh_caches(&self) -> Result<()> {
        // Clear and refresh the YAML content cache
        let mut cache = self.yaml_content_cache.write().await;
        cache.clear();
        info!("Caches refreshed");
        Ok(())
    }

    /// Refresh cache and get updated dashboard metrics
    pub async fn refresh_and_get_metrics(&self) -> Result<DashboardMetrics> {
        let status = self.integration_service.get_system_status().await?;
        
        // Update cache
        {
            let mut cache = self.system_status_cache.write().await;
            *cache = Some(status.clone());
        }

        Ok(DashboardMetrics {
            cpu_usage: status.resource_usage.cpu_usage.unwrap_or(0.0),
            memory_usage: status.resource_usage.memory_usage.unwrap_or(0.0),
            memory_total: 100, // Default values since exact memory info not available
            memory_used: 50,
            network_interfaces: vec![], // NetworkInterface not readily available
            disk_usage: vec![], // Disk usage simplified
            yaml_files_count: self.get_yaml_files().await?.len(),
            uptime: status.uptime.num_seconds() as u64,
        })
    }

    /// Take event receiver
    pub async fn take_event_receiver(&self) -> Result<mpsc::UnboundedReceiver<BackendEvent>> {
        let mut receiver_lock = self.event_receiver.write().await;
        receiver_lock.take().ok_or_else(|| anyhow::anyhow!("Event receiver already taken"))
    }
}

/// Dashboard metrics for UI
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct DashboardMetrics {
    pub cpu_usage: f64,
    pub memory_usage: f64,
    pub memory_total: u64,
    pub memory_used: u64,
    pub network_interfaces: Vec<NetworkInterface>,
    pub disk_usage: Vec<serde_json::Value>,
    pub yaml_files_count: usize,
    pub uptime: u64,
} 