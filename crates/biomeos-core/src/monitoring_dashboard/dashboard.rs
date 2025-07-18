//! Main monitoring dashboard coordinator

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::{
    BiomeError, BiomeResult, universal_biomeos_manager::UniversalBiomeOSManager,
    primal_clients::CapabilityCategory,
};

use super::{
    alerts::AlertManager,
    collectors::{MetricCollector, SystemMetricsCollector},
    config::DashboardConfig,
    metrics::MetricsProcessor,
    notifications::NotificationManager,
    types::{DashboardEvent, DashboardEventType, DashboardMetricsState, DashboardSubscriber},
};

/// Universal monitoring dashboard that works with any primal ecosystem
pub struct UniversalMonitoringDashboard {
    /// Dashboard configuration
    config: DashboardConfig,
    /// Dashboard state
    state: Arc<RwLock<DashboardMetricsState>>,
    /// Event broadcaster
    event_tx: broadcast::Sender<DashboardEvent>,
    /// Alert manager
    alert_manager: AlertManager,
    /// Notification manager
    notification_manager: NotificationManager,
    /// Metric collectors
    metric_collectors: HashMap<String, Box<dyn MetricCollector>>,
    /// System metrics collector
    system_collector: SystemMetricsCollector,
    /// Metrics processor
    metrics_processor: MetricsProcessor,
    /// Universal biomeOS manager for ecosystem access
    biomeos_manager: Option<Arc<UniversalBiomeOSManager>>,
}

impl UniversalMonitoringDashboard {
    /// Create a new universal monitoring dashboard
    pub fn new(config: DashboardConfig) -> Self {
        let (event_tx, _) = broadcast::channel(1000);
        let alert_manager = AlertManager::new();
        let notification_manager = NotificationManager::new();
        let system_collector = SystemMetricsCollector::new();
        let metrics_processor = MetricsProcessor::new();
        
        Self {
            config,
            state: Arc::new(RwLock::new(DashboardMetricsState::new())),
            event_tx,
            alert_manager,
            notification_manager,
            metric_collectors: HashMap::new(),
            system_collector,
            metrics_processor,
            biomeos_manager: None,
        }
    }
    
    /// Set the universal biomeOS manager for ecosystem access
    pub fn set_biomeos_manager(&mut self, manager: Arc<UniversalBiomeOSManager>) {
        self.biomeos_manager = Some(manager);
    }
    
    /// Start the monitoring dashboard
    pub async fn start(&mut self) -> BiomeResult<()> {
        // Start system metrics collection
        self.system_collector.start().await?;
        
        // Start alert manager
        self.alert_manager.start().await?;
        
        // Start notification manager
        self.notification_manager.start().await?;
        
        // Start metrics processor
        self.metrics_processor.start().await?;
        
        // Start ecosystem monitoring if available
        if let Some(manager) = &self.biomeos_manager {
            self.start_ecosystem_monitoring(manager.clone()).await?;
        }
        
        // Emit startup event
        self.emit_event(DashboardEvent {
            event_type: DashboardEventType::SystemStarted,
            timestamp: chrono::Utc::now(),
            details: "Universal monitoring dashboard started".to_string(),
        }).await?;
        
        Ok(())
    }
    
    /// Start ecosystem monitoring using the universal manager
    async fn start_ecosystem_monitoring(&mut self, manager: Arc<UniversalBiomeOSManager>) -> BiomeResult<()> {
        // Monitor ecosystem health
        let manager_clone = manager.clone();
        let event_tx_clone = self.event_tx.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                
                match manager_clone.get_ecosystem_health().await {
                    Ok(ecosystem_status) => {
                        let event = DashboardEvent {
                            event_type: DashboardEventType::EcosystemHealthUpdate,
                            timestamp: chrono::Utc::now(),
                            details: format!("Ecosystem health: {:?}, Primals: {}", 
                                ecosystem_status.health, ecosystem_status.primals.len()),
                        };
                        
                        if let Err(e) = event_tx_clone.send(event) {
                            eprintln!("Failed to send ecosystem health event: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to get ecosystem health: {}", e);
                    }
                }
            }
        });
        
        // Monitor available capabilities
        let manager_clone = manager.clone();
        let event_tx_clone = self.event_tx.clone();
        
        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                
                match manager_clone.get_available_capabilities().await {
                    Ok(capabilities) => {
                        let event = DashboardEvent {
                            event_type: DashboardEventType::CapabilityUpdate,
                            timestamp: chrono::Utc::now(),
                            details: format!("Available capabilities: {:?}", 
                                capabilities.keys().collect::<Vec<&CapabilityCategory>>()),
                        };
                        
                        if let Err(e) = event_tx_clone.send(event) {
                            eprintln!("Failed to send capability update event: {}", e);
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to get available capabilities: {}", e);
                    }
                }
            }
        });
        
        Ok(())
    }
    
    /// Stop the monitoring dashboard
    pub async fn stop(&mut self) -> BiomeResult<()> {
        // Stop all collectors
        self.system_collector.stop().await?;
        
        // Stop alert manager
        self.alert_manager.stop().await?;
        
        // Stop notification manager
        self.notification_manager.stop().await?;
        
        // Stop metrics processor
        self.metrics_processor.stop().await?;
        
        // Emit shutdown event
        self.emit_event(DashboardEvent {
            event_type: DashboardEventType::SystemStopped,
            timestamp: chrono::Utc::now(),
            details: "Universal monitoring dashboard stopped".to_string(),
        }).await?;
        
        Ok(())
    }
    
    /// Get current dashboard state
    pub async fn get_state(&self) -> DashboardMetricsState {
        let state = self.state.read().await;
        state.clone()
    }
    
    /// Subscribe to dashboard events
    pub fn subscribe(&self) -> broadcast::Receiver<DashboardEvent> {
        self.event_tx.subscribe()
    }
    
    /// Emit a dashboard event
    async fn emit_event(&self, event: DashboardEvent) -> BiomeResult<()> {
        self.event_tx.send(event)
            .map_err(|e| BiomeError::Other(format!("Failed to emit event: {}", e)))?;
        Ok(())
    }
    
    /// Get dashboard metrics for API
    pub async fn get_metrics(&self) -> BiomeResult<HashMap<String, serde_json::Value>> {
        let mut metrics = HashMap::new();
        
        // Get system metrics
        let system_metrics = self.system_collector.collect_metrics().await?;
        metrics.insert("system".to_string(), serde_json::to_value(system_metrics)?);
        
        // Get ecosystem metrics if available
        if let Some(manager) = &self.biomeos_manager {
            let ecosystem_health = manager.get_ecosystem_health().await?;
            metrics.insert("ecosystem".to_string(), serde_json::to_value(ecosystem_health)?);
            
            let capabilities = manager.get_available_capabilities().await?;
            metrics.insert("capabilities".to_string(), serde_json::to_value(capabilities)?);
        }
        
        // Get alert metrics
        let alert_metrics = self.alert_manager.get_metrics().await?;
        metrics.insert("alerts".to_string(), serde_json::to_value(alert_metrics)?);
        
        Ok(metrics)
    }
    
    /// Refresh ecosystem discovery
    pub async fn refresh_ecosystem(&self) -> BiomeResult<()> {
        if let Some(manager) = &self.biomeos_manager {
            manager.refresh_ecosystem().await?;
            
            self.emit_event(DashboardEvent {
                event_type: DashboardEventType::EcosystemRefreshed,
                timestamp: chrono::Utc::now(),
                details: "Ecosystem discovery refreshed".to_string(),
            }).await?;
        }
        
        Ok(())
    }
}
