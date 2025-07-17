//! Main monitoring dashboard coordinator

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{broadcast, RwLock};
use uuid::Uuid;

use crate::{
    ecosystem_integration::EcosystemServiceRegistry, BiomeError, BiomeResult,
    CrossPrimalCoordinator, FederationOptimizer, PredictiveHealthAnalytics,
};

use super::{
    alerts::AlertManager,
    collectors::{FederationMetricsCollector, MetricCollector, SystemMetricsCollector},
    config::DashboardConfig,
    metrics::MetricsProcessor,
    notifications::NotificationManager,
    types::{DashboardEvent, DashboardEventType, DashboardMetricsState, DashboardSubscriber},
};

/// Real-time monitoring dashboard coordinator
pub struct MonitoringDashboard {
    /// Cross-primal coordinator for metrics collection
    coordinator: Arc<CrossPrimalCoordinator>,
    /// Predictive health analytics
    health_analytics: Arc<PredictiveHealthAnalytics>,
    /// Federation optimizer for resource insights
    federation_optimizer: Arc<FederationOptimizer>,
    /// Service registry for discovery
    service_registry: Arc<EcosystemServiceRegistry>,
    /// Dashboard configuration
    config: DashboardConfig,
    /// Real-time metrics state
    metrics_state: Arc<RwLock<DashboardMetricsState>>,
    /// Alert manager
    alert_manager: Arc<AlertManager>,
    /// Metric collectors
    metric_collectors: Arc<RwLock<HashMap<String, Arc<dyn MetricCollector>>>>,
    /// Dashboard subscribers
    subscribers: Arc<RwLock<HashMap<String, DashboardSubscriber>>>,
    /// Event broadcaster
    event_broadcaster: Arc<broadcast::Sender<DashboardEvent>>,
    /// Metrics processor
    metrics_processor: Arc<RwLock<MetricsProcessor>>,
    /// Notification manager
    notification_manager: Arc<NotificationManager>,
    /// Dashboard running state
    running: Arc<RwLock<bool>>,
}

impl MonitoringDashboard {
    /// Create a new monitoring dashboard
    pub async fn new(
        coordinator: Arc<CrossPrimalCoordinator>,
        health_analytics: Arc<PredictiveHealthAnalytics>,
        federation_optimizer: Arc<FederationOptimizer>,
        service_registry: Arc<EcosystemServiceRegistry>,
        config: DashboardConfig,
    ) -> BiomeResult<Self> {
        let (event_broadcaster, _) = broadcast::channel(1000);
        let event_broadcaster = Arc::new(event_broadcaster);

        let metrics_processor =
            Arc::new(RwLock::new(MetricsProcessor::new(config.metrics_retention)));
        let notification_manager = Arc::new(NotificationManager::new());

        let dashboard = Self {
            coordinator,
            health_analytics,
            federation_optimizer,
            service_registry,
            config,
            metrics_state: Arc::new(RwLock::new(DashboardMetricsState::default())),
            alert_manager: Arc::new(AlertManager::new()),
            metric_collectors: Arc::new(RwLock::new(HashMap::new())),
            subscribers: Arc::new(RwLock::new(HashMap::new())),
            event_broadcaster,
            metrics_processor,
            notification_manager,
            running: Arc::new(RwLock::new(false)),
        };

        // Initialize default collectors
        dashboard.initialize_default_collectors().await?;

        Ok(dashboard)
    }

    /// Initialize default metric collectors
    async fn initialize_default_collectors(&self) -> BiomeResult<()> {
        let mut collectors = self.metric_collectors.write().await;

        // System metrics collector
        collectors.insert(
            "system".to_string(),
            Arc::new(SystemMetricsCollector::new()),
        );

        // Federation metrics collector
        collectors.insert(
            "federation".to_string(),
            Arc::new(FederationMetricsCollector::new()),
        );

        Ok(())
    }

    /// Start the monitoring dashboard
    pub async fn start(&self) -> BiomeResult<()> {
        {
            let mut running = self.running.write().await;
            if *running {
                return Err(BiomeError::RuntimeError(
                    "Dashboard already running".to_string(),
                ));
            }
            *running = true;
        }

        // Start metrics collection loop
        let metrics_task = self.start_metrics_collection();

        // Start alert evaluation loop
        let alerts_task = self.start_alert_evaluation();

        // Start dashboard update loop
        let update_task = self.start_dashboard_updates();

        // Start cleanup task
        let cleanup_task = self.start_cleanup_task();

        // Run all tasks concurrently
        tokio::try_join!(metrics_task, alerts_task, update_task, cleanup_task)?;

        Ok(())
    }

    /// Stop the monitoring dashboard
    pub async fn stop(&self) -> BiomeResult<()> {
        let mut running = self.running.write().await;
        *running = false;
        Ok(())
    }

    /// Start metrics collection loop
    async fn start_metrics_collection(&self) -> BiomeResult<()> {
        let mut interval = tokio::time::interval(Duration::from_secs(self.config.metrics_interval));

        loop {
            interval.tick().await;

            {
                let running = self.running.read().await;
                if !*running {
                    break;
                }
            }

            if let Err(e) = self.collect_metrics().await {
                eprintln!("Error collecting metrics: {}", e);
            }
        }

        Ok(())
    }

    /// Start alert evaluation loop
    async fn start_alert_evaluation(&self) -> BiomeResult<()> {
        let mut interval = tokio::time::interval(Duration::from_secs(self.config.alert_interval));

        loop {
            interval.tick().await;

            {
                let running = self.running.read().await;
                if !*running {
                    break;
                }
            }

            if let Err(e) = self.evaluate_alerts().await {
                eprintln!("Error evaluating alerts: {}", e);
            }
        }

        Ok(())
    }

    /// Start dashboard updates loop
    async fn start_dashboard_updates(&self) -> BiomeResult<()> {
        let mut interval = tokio::time::interval(Duration::from_secs(self.config.update_frequency));

        loop {
            interval.tick().await;

            {
                let running = self.running.read().await;
                if !*running {
                    break;
                }
            }

            if let Err(e) = self.update_dashboard().await {
                eprintln!("Error updating dashboard: {}", e);
            }
        }

        Ok(())
    }

    /// Start cleanup task
    async fn start_cleanup_task(&self) -> BiomeResult<()> {
        let mut interval = tokio::time::interval(Duration::from_secs(3600)); // Every hour

        loop {
            interval.tick().await;

            {
                let running = self.running.read().await;
                if !*running {
                    break;
                }
            }

            if let Err(e) = self.cleanup_old_data().await {
                eprintln!("Error cleaning up old data: {}", e);
            }
        }

        Ok(())
    }

    /// Collect metrics from all collectors
    async fn collect_metrics(&self) -> BiomeResult<()> {
        let collectors = self.metric_collectors.read().await;
        let mut all_metrics = Vec::new();

        for (name, collector) in collectors.iter() {
            match collector.collect_metrics().await {
                Ok(metrics) => {
                    all_metrics.extend(metrics);
                }
                Err(e) => {
                    eprintln!("Error collecting metrics from {}: {}", name, e);
                }
            }
        }

        // Store metrics
        {
            let mut processor = self.metrics_processor.write().await;
            processor.store_metrics(all_metrics);
        }

        Ok(())
    }

    /// Evaluate alerts
    async fn evaluate_alerts(&self) -> BiomeResult<()> {
        // Get current metrics for alert evaluation
        let metrics = self.get_current_metrics().await?;

        // Evaluate alerts
        self.alert_manager.evaluate_alerts(&metrics).await?;

        Ok(())
    }

    /// Update dashboard state
    async fn update_dashboard(&self) -> BiomeResult<()> {
        let new_state = self.generate_dashboard_state().await?;

        {
            let mut state = self.metrics_state.write().await;
            *state = new_state;
        }

        // Notify subscribers
        self.notify_subscribers().await?;

        // Broadcast update event
        let event = DashboardEvent {
            id: Uuid::new_v4().to_string(),
            event_type: DashboardEventType::MetricsUpdated,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            data: serde_json::json!({}),
        };

        let _ = self.event_broadcaster.send(event);

        Ok(())
    }

    /// Generate current dashboard state
    async fn generate_dashboard_state(&self) -> BiomeResult<DashboardMetricsState> {
        // This would aggregate all metrics and generate the complete dashboard state
        // For now, return default state
        Ok(DashboardMetricsState::default())
    }

    /// Get current metrics for alert evaluation
    async fn get_current_metrics(&self) -> BiomeResult<HashMap<String, f64>> {
        // This would extract current metric values for alert evaluation
        // For now, return mock data
        let mut metrics = HashMap::new();
        metrics.insert("cpu_usage".to_string(), 75.0);
        metrics.insert("memory_usage".to_string(), 85.0);
        metrics.insert("disk_usage".to_string(), 90.0);
        Ok(metrics)
    }

    /// Notify subscribers of updates
    async fn notify_subscribers(&self) -> BiomeResult<()> {
        let subscribers = self.subscribers.read().await;
        let _state = self.metrics_state.read().await;

        for subscriber in subscribers.values() {
            // Send update to subscriber (implementation would depend on subscriber type)
            // For now, just log
            println!("Notifying subscriber: {}", subscriber.name);
        }

        Ok(())
    }

    /// Clean up old data
    async fn cleanup_old_data(&self) -> BiomeResult<()> {
        let mut processor = self.metrics_processor.write().await;
        processor.cleanup_old_metrics();
        Ok(())
    }

    /// Add metric collector
    pub async fn add_metric_collector(
        &self,
        name: String,
        collector: Arc<dyn MetricCollector>,
    ) -> BiomeResult<()> {
        let mut collectors = self.metric_collectors.write().await;
        collectors.insert(name, collector);
        Ok(())
    }

    /// Remove metric collector
    pub async fn remove_metric_collector(&self, name: &str) -> BiomeResult<()> {
        let mut collectors = self.metric_collectors.write().await;
        collectors.remove(name);
        Ok(())
    }

    /// Subscribe to dashboard updates
    pub async fn subscribe(
        &self,
        subscriber: DashboardSubscriber,
    ) -> BiomeResult<broadcast::Receiver<DashboardEvent>> {
        let mut subscribers = self.subscribers.write().await;
        subscribers.insert(subscriber.id.clone(), subscriber);

        Ok(self.event_broadcaster.subscribe())
    }

    /// Unsubscribe from dashboard updates
    pub async fn unsubscribe(&self, subscriber_id: &str) -> BiomeResult<()> {
        let mut subscribers = self.subscribers.write().await;
        subscribers.remove(subscriber_id);
        Ok(())
    }

    /// Get current dashboard state
    pub async fn get_dashboard_state(&self) -> DashboardMetricsState {
        self.metrics_state.read().await.clone()
    }

    /// Get alert manager
    pub fn get_alert_manager(&self) -> Arc<AlertManager> {
        self.alert_manager.clone()
    }

    /// Get metrics processor
    pub fn get_metrics_processor(&self) -> Arc<RwLock<MetricsProcessor>> {
        self.metrics_processor.clone()
    }

    /// Get notification manager
    pub fn get_notification_manager(&self) -> Arc<NotificationManager> {
        self.notification_manager.clone()
    }

    /// Check if dashboard is running
    pub async fn is_running(&self) -> bool {
        *self.running.read().await
    }
}
