use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use anyhow::Result;
use tracing::{info, warn, error, debug};

use biomeos_core::integration::{
    LiveIntegrationService, SystemStatus, MemoryUsage, DiskUsage, NetworkInterface
};

/// Live UI Backend - connects to real biomeOS integration service
pub struct LiveBackend {
    integration_service: Arc<LiveIntegrationService>,
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
        
        // Get workspace root (biomeOS directory)
        let workspace_root = std::env::current_dir()?.canonicalize()?;
        
        // Initialize live integration service
        let integration_service = Arc::new(
            LiveIntegrationService::new(workspace_root).await?
        );
        
        // Start the integration service
        integration_service.start().await?;
        
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        let backend = Arc::new(Self {
            integration_service,
            system_status_cache: RwLock::new(None),
            yaml_content_cache: RwLock::new(HashMap::new()),
            event_sender,
            event_receiver: RwLock::new(Some(event_receiver)),
        });
        
        // Start background monitoring tasks
        let backend_clone = backend.clone();
        tokio::spawn(async move {
            backend_clone.start_monitoring_tasks().await;
        });
        
        info!("✅ Live UI Backend initialized successfully");
        Ok(backend)
    }

    /// Start background monitoring tasks
    async fn start_monitoring_tasks(&self) {
        info!("🔄 Starting background monitoring tasks");
        
        let service = self.integration_service.clone();
        let sender = self.event_sender.clone();
        
        // System status monitoring task (every 3 seconds)
        let service_clone = service.clone();
        let sender_clone = sender.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(3));
            
            loop {
                interval.tick().await;
                
                match service_clone.get_system_status().await {
                    Ok(status) => {
                        if sender_clone.send(BackendEvent::SystemStatusUpdated(status)).is_err() {
                            break; // Channel closed
                        }
                    }
                    Err(e) => {
                        warn!("Failed to get system status: {}", e);
                        if sender_clone.send(BackendEvent::Error(format!("System status error: {}", e))).is_err() {
                            break;
                        }
                    }
                }
            }
        });
        
        // YAML file monitoring task (every 5 seconds)
        let service_clone = service.clone();
        let sender_clone = sender.clone();
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(5));
            let mut last_yaml_files: HashMap<String, String> = HashMap::new();
            
            loop {
                interval.tick().await;
                
                let current_files = service_clone.get_yaml_files().await;
                
                // Check for changes
                for (file_name, content) in &current_files {
                    if let Some(last_content) = last_yaml_files.get(file_name) {
                        if content != last_content {
                            debug!("YAML file changed: {}", file_name);
                            if sender_clone.send(BackendEvent::YamlFileChanged(file_name.clone())).is_err() {
                                return;
                            }
                        }
                    }
                }
                
                last_yaml_files = current_files;
            }
        });
        
        info!("✅ Background monitoring tasks started");
    }

    /// Get event receiver (should be called once by UI)
    pub async fn take_event_receiver(&self) -> Option<mpsc::UnboundedReceiver<BackendEvent>> {
        self.event_receiver.write().await.take()
    }

    /// Get current system status (cached)
    pub async fn get_system_status(&self) -> Option<SystemStatus> {
        self.system_status_cache.read().await.clone()
    }

    /// Update system status cache
    pub async fn update_system_status_cache(&self, status: SystemStatus) {
        *self.system_status_cache.write().await = Some(status);
    }

    /// Get all YAML files
    pub async fn get_yaml_files(&self) -> Result<HashMap<String, String>> {
        let files = self.integration_service.get_yaml_files().await;
        *self.yaml_content_cache.write().await = files.clone();
        Ok(files)
    }

    /// Get specific YAML file content
    pub async fn get_yaml_content(&self, file_name: &str) -> Option<String> {
        // First check cache
        if let Some(content) = self.yaml_content_cache.read().await.get(file_name) {
            return Some(content.clone());
        }
        
        // Fall back to service
        self.integration_service.get_yaml_content(file_name).await
    }

    /// Update YAML file content (REAL FILE I/O)
    pub async fn update_yaml_content(&self, file_name: &str, content: String) -> Result<()> {
        info!("✏️ UI Backend: Updating YAML file {} (LIVE I/O)", file_name);
        
        // Update through integration service (writes to real file)
        self.integration_service.update_yaml_content(file_name, content.clone()).await?;
        
        // Update local cache
        self.yaml_content_cache.write().await.insert(file_name.to_string(), content);
        
        // Notify UI of change
        let _ = self.event_sender.send(BackendEvent::YamlFileChanged(file_name.to_string()));
        
        info!("✅ YAML file updated: {}", file_name);
        Ok(())
    }

    /// Create new YAML file
    pub async fn create_yaml_file(&self, file_name: &str, content: String) -> Result<()> {
        info!("📝 UI Backend: Creating new YAML file {} (LIVE I/O)", file_name);
        
        self.integration_service.create_yaml_file(file_name, content.clone()).await?;
        
        // Update cache
        self.yaml_content_cache.write().await.insert(file_name.to_string(), content);
        
        // Notify UI
        let _ = self.event_sender.send(BackendEvent::YamlFileChanged(file_name.to_string()));
        
        Ok(())
    }

    /// Delete YAML file
    pub async fn delete_yaml_file(&self, file_name: &str) -> Result<()> {
        info!("🗑️ UI Backend: Deleting YAML file {} (LIVE I/O)", file_name);
        
        self.integration_service.delete_yaml_file(file_name).await?;
        
        // Remove from cache
        self.yaml_content_cache.write().await.remove(file_name);
        
        // Notify UI
        let _ = self.event_sender.send(BackendEvent::YamlFileChanged(file_name.to_string()));
        
        Ok(())
    }

    /// Refresh all caches from integration service
    pub async fn refresh_caches(&self) -> Result<()> {
        info!("🔄 UI Backend: Refreshing all caches");
        
        // Refresh integration service cache
        self.integration_service.refresh_cache().await?;
        
        // Refresh local caches
        let yaml_files = self.integration_service.get_yaml_files().await;
        *self.yaml_content_cache.write().await = yaml_files;
        
        match self.integration_service.get_system_status().await {
            Ok(status) => {
                *self.system_status_cache.write().await = Some(status);
            }
            Err(e) => {
                warn!("Failed to refresh system status: {}", e);
            }
        }
        
        info!("✅ All caches refreshed");
        Ok(())
    }

    /// Validate YAML content
    pub fn validate_yaml(&self, content: &str) -> Result<()> {
        serde_yaml::from_str::<serde_yaml::Value>(content)?;
        Ok(())
    }

    /// Get live system metrics for dashboard
    pub async fn get_dashboard_metrics(&self) -> Result<DashboardMetrics> {
        let status = match self.get_system_status().await {
            Some(status) => status,
            None => self.integration_service.get_system_status().await?,
        };

        Ok(DashboardMetrics {
            cpu_usage: status.cpu_usage,
            memory_usage: status.memory_usage.usage_percent,
            memory_total: status.memory_usage.total,
            memory_used: status.memory_usage.used,
            network_interfaces: status.network_stats,
            disk_usage: status.disk_usage,
            yaml_files_count: self.yaml_content_cache.read().await.len(),
            uptime: status.uptime,
        })
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
    pub disk_usage: Vec<DiskUsage>,
    pub yaml_files_count: usize,
    pub uptime: u64,
} 