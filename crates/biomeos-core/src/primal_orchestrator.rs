//! Primal Orchestrator - Async, concurrent primal lifecycle management
//!
//! Handles complex startup choreography:
//! - BearDog → Songbird (crypto provider → discovery)
//! - Songbird → BearDog fleet (discovery → crypto cluster)
//! - Songbird → Songbird → Network (cascading discovery)
//! - Concurrent health monitoring
//! - Automatic recovery

use std::{collections::HashMap, sync::Arc, time::Duration};

use async_trait::async_trait;
use tokio::{
    sync::RwLock,
    time::{sleep, timeout},
};
use tracing::{debug, error, info, instrument, warn};

use biomeos_types::{
    error::{BiomeError, BiomeResult},
    identifiers::{Endpoint, PrimalId},
};

use crate::{
    capabilities::Capability, discovery_modern::HealthStatus, retry::RetryPolicy,
    socket_discovery::SocketDiscovery,
};

/// Primal health monitor using JSON-RPC over Unix sockets.
///
/// This is the TRUE PRIMAL health monitoring implementation:
/// - Uses Unix sockets, not HTTP
/// - Calls `health.check` JSON-RPC method
/// - Tracks primal status with atomic state
#[derive(Clone)]
pub struct PrimalHealthMonitor {
    /// Registered primals: id → socket path
    primals: Arc<RwLock<HashMap<PrimalId, String>>>,

    /// Primal health status: id → healthy
    status: Arc<RwLock<HashMap<PrimalId, bool>>>,

    /// Check interval
    interval: std::time::Duration,

    /// Running flag
    running: Arc<std::sync::atomic::AtomicBool>,
}

impl PrimalHealthMonitor {
    pub fn builder() -> PrimalHealthMonitorBuilder {
        PrimalHealthMonitorBuilder {
            interval: std::time::Duration::from_secs(30),
        }
    }

    /// Start the health monitoring background task.
    ///
    /// Periodically calls `health.check` on all registered primals.
    pub async fn start_monitoring(&self) -> anyhow::Result<()> {
        tracing::info!("🏥 Health monitor started (JSON-RPC over Unix sockets)");

        self.running
            .store(true, std::sync::atomic::Ordering::SeqCst);

        let primals = self.primals.clone();
        let status = self.status.clone();
        let interval = self.interval;
        let running = self.running.clone();

        tokio::spawn(async move {
            let mut interval_timer = tokio::time::interval(interval);

            while running.load(std::sync::atomic::Ordering::SeqCst) {
                interval_timer.tick().await;

                let primals_snapshot = primals.read().await.clone();

                for (id, socket_path) in primals_snapshot {
                    let healthy = Self::check_primal_health(&socket_path).await;
                    status.write().await.insert(id.clone(), healthy);

                    if !healthy {
                        tracing::warn!("🏥 Primal {} is unhealthy", id);
                    }
                }
            }

            tracing::info!("🏥 Health monitor stopped");
        });

        Ok(())
    }

    /// Check a primal's health via JSON-RPC.
    async fn check_primal_health(socket_path: &str) -> bool {
        use std::path::Path;
        use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
        use tokio::net::UnixStream;

        let socket = Path::new(socket_path);
        if !socket.exists() {
            return false;
        }

        // Connect and send health.check
        let stream = match UnixStream::connect(socket).await {
            Ok(s) => s,
            Err(_) => return false,
        };

        let (reader, mut writer) = stream.into_split();
        let request = r#"{"jsonrpc":"2.0","method":"health.check","id":1}"#;

        if writer
            .write_all(format!("{}\n", request).as_bytes())
            .await
            .is_err()
        {
            return false;
        }

        // Read response
        let mut reader = BufReader::new(reader);
        let mut response = String::new();

        match tokio::time::timeout(
            std::time::Duration::from_secs(5),
            reader.read_line(&mut response),
        )
        .await
        {
            Ok(Ok(_)) => {
                // Check for success
                response.contains("healthy") || response.contains("\"result\"")
            }
            _ => false,
        }
    }

    /// Stop the health monitor.
    pub fn stop(&self) {
        self.running
            .store(false, std::sync::atomic::Ordering::SeqCst);
    }

    /// Register a primal for health monitoring.
    ///
    /// Supports both URL-based endpoints and direct socket paths.
    pub async fn register(&self, id: PrimalId, endpoint: biomeos_types::identifiers::Endpoint) {
        // Extract socket path from endpoint URL
        // Unix socket URLs: unix:///tmp/primal.sock or file:///tmp/primal.sock
        let url = endpoint.url();
        let socket_path = if url.scheme() == "unix" || url.scheme() == "file" {
            url.path().to_string()
        } else {
            // For HTTP URLs, use SocketDiscovery for capability-based path building
            tracing::warn!(
                "🏥 Primal {} uses HTTP endpoint ({}), discovering socket path",
                id,
                url
            );
            // Get family_id from env or use default
            let family_id = std::env::var("FAMILY_ID")
                .or_else(|_| std::env::var("BIOMEOS_FAMILY_ID"))
                .unwrap_or_else(|_| "default".to_string());
            let discovery = SocketDiscovery::new(family_id);
            discovery
                .build_socket_path(id.as_ref())
                .to_string_lossy()
                .to_string()
        };

        tracing::debug!("🏥 Registering primal {} at {}", id, socket_path);
        self.primals.write().await.insert(id.clone(), socket_path);
        self.status.write().await.insert(id, true); // Assume healthy initially
    }

    /// Register a primal by direct socket path.
    pub async fn register_socket(&self, id: PrimalId, socket_path: impl Into<String>) {
        let socket_path = socket_path.into();
        tracing::debug!("🏥 Registering primal {} at {}", id, socket_path);
        self.primals.write().await.insert(id.clone(), socket_path);
        self.status.write().await.insert(id, true);
    }

    /// Unregister a primal from health monitoring.
    pub async fn unregister(&self, id: &PrimalId) {
        tracing::debug!("🏥 Unregistering primal {}", id);
        self.primals.write().await.remove(id);
        self.status.write().await.remove(id);
    }

    /// Get the health status of a primal.
    pub async fn is_healthy(&self, id: &PrimalId) -> Option<bool> {
        self.status.read().await.get(id).copied()
    }

    /// Get all primal health statuses.
    pub async fn all_status(&self) -> HashMap<PrimalId, bool> {
        self.status.read().await.clone()
    }
}

pub struct PrimalHealthMonitorBuilder {
    interval: std::time::Duration,
}

impl PrimalHealthMonitorBuilder {
    /// Set the health check interval.
    pub fn interval(mut self, interval: std::time::Duration) -> Self {
        self.interval = interval;
        self
    }

    pub fn build(self) -> PrimalHealthMonitor {
        PrimalHealthMonitor {
            primals: Arc::new(RwLock::new(HashMap::new())),
            status: Arc::new(RwLock::new(HashMap::new())),
            interval: self.interval,
            running: Arc::new(std::sync::atomic::AtomicBool::new(false)),
        }
    }
}

/// Represents a primal's lifecycle state
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrimalState {
    /// Not yet started
    Pending,
    /// Currently starting up
    Starting,
    /// Healthy and operational
    Running,
    /// Started but degraded
    Degraded,
    /// Failed to start or crashed
    Failed { reason: String },
    /// Intentionally stopped
    Stopped,
}

/// Represents a primal that can be orchestrated
#[async_trait]
pub trait ManagedPrimal: Send + Sync {
    /// Get the primal's ID
    fn id(&self) -> &PrimalId;

    /// Get capabilities this primal provides
    fn provides(&self) -> &[Capability];

    /// Get capabilities this primal requires
    fn requires(&self) -> &[Capability];

    /// Get the primal's endpoint (if running)
    async fn endpoint(&self) -> Option<Endpoint>;

    /// Start the primal
    async fn start(&self) -> BiomeResult<()>;

    /// Stop the primal
    async fn stop(&self) -> BiomeResult<()>;

    /// Check if the primal is healthy
    async fn health_check(&self) -> BiomeResult<HealthStatus>;

    /// Get the startup timeout
    fn startup_timeout(&self) -> Duration {
        Duration::from_secs(30)
    }
}

/// Orchestrates primal lifecycle with dependency resolution
pub struct PrimalOrchestrator {
    primals: Arc<RwLock<HashMap<PrimalId, PrimalRecord>>>,
    health_monitor: Arc<PrimalHealthMonitor>,
    retry_policy: RetryPolicy,
}

struct PrimalRecord {
    primal: Arc<dyn ManagedPrimal>,
    state: PrimalState,
}

impl PrimalOrchestrator {
    /// Create a new orchestrator
    pub fn new(health_monitor: Arc<PrimalHealthMonitor>, retry_policy: RetryPolicy) -> Self {
        Self {
            primals: Arc::new(RwLock::new(HashMap::new())),
            health_monitor,
            retry_policy,
        }
    }

    /// Register a primal for orchestration
    #[instrument(skip(self, primal))]
    pub async fn register(&self, primal: Arc<dyn ManagedPrimal>) {
        let id = primal.id().clone();
        info!("Registering primal: {}", id);

        let mut primals = self.primals.write().await;
        primals.insert(
            id,
            PrimalRecord {
                primal,
                state: PrimalState::Pending,
            },
        );
    }

    /// Start all primals in dependency order
    #[instrument(skip(self))]
    pub async fn start_all(&self) -> BiomeResult<()> {
        info!("🚀 Starting all primals in dependency order...");

        // Build dependency graph
        let start_order = self.resolve_dependencies().await?;

        info!("📋 Start order: {:?}", start_order);

        // Start primals in order
        for primal_id in start_order {
            self.start_primal(&primal_id).await?;
        }

        info!("✅ All primals started successfully");
        Ok(())
    }

    /// Start a specific primal (with capability-based dependencies)
    #[instrument(skip(self))]
    pub async fn start_primal(&self, id: &PrimalId) -> BiomeResult<()> {
        info!("Starting primal: {}", id);

        // Get primal and its capability requirements
        let (primal, required_caps) = {
            let primals = self.primals.read().await;
            let record = primals.get(id).ok_or_else(|| {
                BiomeError::discovery_failed(
                    format!("Primal not found: {}", id),
                    Some(id.to_string()),
                )
            })?;

            if record.state == PrimalState::Running {
                info!("Primal {} already running", id);
                return Ok(());
            }

            (record.primal.clone(), record.primal.requires().to_vec())
        };

        // Start providers of required capabilities first
        for required_cap in &required_caps {
            debug!("Ensuring capability provider for: {}", required_cap);
            self.ensure_capability_provider(required_cap).await?;
        }

        // Update state to Starting
        {
            let mut primals = self.primals.write().await;
            if let Some(record) = primals.get_mut(id) {
                record.state = PrimalState::Starting;
            }
        }

        // Start the primal with retry
        let start_result = self
            .retry_policy
            .execute(|| async {
                primal
                    .start()
                    .await
                    .map_err(|e| anyhow::anyhow!("Start failed: {}", e))
            })
            .await;

        match start_result {
            Ok(_) => {
                info!("✅ Primal {} started", id);

                // Wait for health check with timeout
                match timeout(primal.startup_timeout(), self.wait_for_health(&primal)).await {
                    Ok(Ok(_)) => {
                        // Register with health monitor
                        if let Some(endpoint) = primal.endpoint().await {
                            self.health_monitor.register(id.clone(), endpoint).await;
                        }

                        // Update state to Running
                        let mut primals = self.primals.write().await;
                        if let Some(record) = primals.get_mut(id) {
                            record.state = PrimalState::Running;
                        }

                        info!("✅ Primal {} is healthy and running", id);
                        Ok(())
                    }
                    Ok(Err(e)) => {
                        error!("Primal {} failed health check: {}", id, e);
                        self.mark_failed(id, format!("Health check failed: {}", e))
                            .await;
                        Err(e)
                    }
                    Err(_) => {
                        let msg = format!("Startup timeout after {:?}", primal.startup_timeout());
                        error!("Primal {} {}", id, msg);
                        self.mark_failed(id, msg.clone()).await;
                        Err(BiomeError::timeout_error(msg, 30000, Some("primal_start")))
                    }
                }
            }
            Err(e) => {
                error!("Failed to start primal {}: {}", id, e);
                self.mark_failed(id, e.to_string()).await;
                Err(BiomeError::internal_error(
                    format!("Failed to start {}: {}", id, e),
                    Some("primal_start_failure"),
                ))
            }
        }
    }

    /// Ensure at least one provider for a capability is running
    fn ensure_capability_provider<'a>(
        &'a self,
        capability: &'a Capability,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = BiomeResult<()>> + Send + 'a>> {
        Box::pin(async move {
            let primals = self.primals.read().await;

            // Find primals that provide this capability
            let providers: Vec<_> = primals
                .iter()
                .filter(|(_, record)| record.primal.provides().contains(capability))
                .map(|(id, _)| id.clone())
                .collect();

            if providers.is_empty() {
                return Err(BiomeError::discovery_failed(
                    format!("No provider found for capability: {}", capability),
                    Some(format!("capability:{:?}", capability)),
                ));
            }

            drop(primals); // Release read lock before starting

            // Start first available provider (could extend with load balancing/health-based selection)
            for provider_id in providers {
                // Check if already running
                let state = self.get_state(&provider_id).await;
                if state == Some(PrimalState::Running) {
                    debug!(
                        "Capability {} already provided by {}",
                        capability, provider_id
                    );
                    return Ok(());
                }

                // Try to start this provider
                match self.start_primal(&provider_id).await {
                    Ok(_) => {
                        info!(
                            "✅ Started capability provider {} for {}",
                            provider_id, capability
                        );
                        return Ok(());
                    }
                    Err(e) => {
                        warn!(
                            "Failed to start provider {} for {}: {}",
                            provider_id, capability, e
                        );
                        // Continue to next provider
                    }
                }
            }

            Err(BiomeError::internal_error(
                format!(
                    "All providers for capability {} failed to start",
                    capability
                ),
                Some("capability_startup_failure"),
            ))
        })
    }

    /// Stop a specific primal
    #[instrument(skip(self))]
    pub async fn stop_primal(&self, id: &PrimalId) -> BiomeResult<()> {
        info!("Stopping primal: {}", id);

        let primal = {
            let primals = self.primals.read().await;
            let record = primals.get(id).ok_or_else(|| {
                BiomeError::discovery_failed(
                    format!("Primal not found: {}", id),
                    Some(id.to_string()),
                )
            })?;

            if record.state == PrimalState::Stopped {
                info!("Primal {} already stopped", id);
                return Ok(());
            }

            record.primal.clone()
        };

        // Unregister from health monitor
        self.health_monitor.unregister(id).await;

        // Stop the primal
        primal.stop().await.map_err(|e| {
            BiomeError::internal_error(
                format!("Failed to stop primal {}: {}", id, e),
                Some("primal_stop_failure"),
            )
        })?;

        // Update state
        let mut primals = self.primals.write().await;
        if let Some(record) = primals.get_mut(id) {
            record.state = PrimalState::Stopped;
        }

        info!("✅ Primal {} stopped", id);
        Ok(())
    }

    /// Stop all primals (in reverse dependency order)
    #[instrument(skip(self))]
    pub async fn stop_all(&self) -> BiomeResult<()> {
        info!("🛑 Stopping all primals...");

        // Get reverse start order
        let mut stop_order = self.resolve_dependencies().await?;
        stop_order.reverse();

        info!("📋 Stop order: {:?}", stop_order);

        // Stop primals in order
        for primal_id in stop_order {
            // Ignore errors during shutdown
            if let Err(e) = self.stop_primal(&primal_id).await {
                warn!("Error stopping {}: {}", primal_id, e);
            }
        }

        info!("✅ All primals stopped");
        Ok(())
    }

    /// Get the state of a primal
    pub async fn get_state(&self, id: &PrimalId) -> Option<PrimalState> {
        let primals = self.primals.read().await;
        primals.get(id).map(|r| r.state.clone())
    }

    /// Get all primal states
    pub async fn get_all_states(&self) -> HashMap<PrimalId, PrimalState> {
        let primals = self.primals.read().await;
        primals
            .iter()
            .map(|(id, record)| (id.clone(), record.state.clone()))
            .collect()
    }

    // Private helpers

    async fn resolve_dependencies(&self) -> BiomeResult<Vec<PrimalId>> {
        let primals = self.primals.read().await;

        // Build capability-based dependency graph
        let mut capability_providers: HashMap<Capability, Vec<PrimalId>> = HashMap::new();
        let mut primal_requirements: HashMap<PrimalId, Vec<Capability>> = HashMap::new();

        // Map: which primals provide which capabilities
        for (id, record) in primals.iter() {
            for cap in record.primal.provides() {
                capability_providers
                    .entry(cap.clone())
                    .or_default()
                    .push(id.clone());
            }

            primal_requirements.insert(id.clone(), record.primal.requires().to_vec());
        }

        // Build dependency graph based on capabilities
        let mut in_degree: HashMap<PrimalId, usize> = HashMap::new();
        let mut graph: HashMap<PrimalId, Vec<PrimalId>> = HashMap::new();

        for (consumer_id, required_caps) in primal_requirements.iter() {
            in_degree.entry(consumer_id.clone()).or_insert(0);

            for required_cap in required_caps {
                // Find providers of this capability
                if let Some(providers) = capability_providers.get(required_cap) {
                    for provider_id in providers {
                        // Provider must start before consumer
                        graph
                            .entry(provider_id.clone())
                            .or_default()
                            .push(consumer_id.clone());

                        *in_degree.entry(consumer_id.clone()).or_insert(0) += 1;
                    }
                }
            }
        }

        // Topological sort (Kahn's algorithm)
        let mut queue: Vec<PrimalId> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut result = Vec::new();

        while let Some(id) = queue.pop() {
            result.push(id.clone());

            if let Some(neighbors) = graph.get(&id) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push(neighbor.clone());
                        }
                    }
                }
            }
        }

        if result.len() != primals.len() {
            return Err(BiomeError::config_error(
                "Circular capability dependencies detected",
                Some("capability_deps"),
            ));
        }

        Ok(result)
    }

    async fn wait_for_health(&self, primal: &Arc<dyn ManagedPrimal>) -> BiomeResult<()> {
        let max_attempts = 10;
        let mut attempts = 0;

        loop {
            attempts += 1;
            debug!(
                "Health check attempt {}/{} for {}",
                attempts,
                max_attempts,
                primal.id()
            );

            match primal.health_check().await {
                Ok(status) if status.is_healthy() => {
                    debug!("Primal {} is healthy", primal.id());
                    return Ok(());
                }
                Ok(status) => {
                    debug!("Primal {} status: {:?}", primal.id(), status);
                }
                Err(e) => {
                    debug!("Health check failed for {}: {}", primal.id(), e);
                }
            }

            if attempts >= max_attempts {
                return Err(BiomeError::timeout_error(
                    format!("Health check timeout for {}", primal.id()),
                    30000,
                    Some("health_check"),
                ));
            }

            sleep(Duration::from_secs(2)).await;
        }
    }

    async fn mark_failed(&self, id: &PrimalId, reason: String) {
        let mut primals = self.primals.write().await;
        if let Some(record) = primals.get_mut(id) {
            record.state = PrimalState::Failed { reason };
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockPrimal {
        id: PrimalId,
        provides: Vec<Capability>,
        requires: Vec<Capability>,
    }

    #[async_trait]
    impl ManagedPrimal for MockPrimal {
        fn id(&self) -> &PrimalId {
            &self.id
        }

        fn provides(&self) -> &[Capability] {
            &self.provides
        }

        fn requires(&self) -> &[Capability] {
            &self.requires
        }

        async fn endpoint(&self) -> Option<Endpoint> {
            None
        }

        async fn start(&self) -> BiomeResult<()> {
            Ok(())
        }

        async fn stop(&self) -> BiomeResult<()> {
            Ok(())
        }

        async fn health_check(&self) -> BiomeResult<HealthStatus> {
            Ok(HealthStatus::Healthy)
        }
    }

    #[tokio::test]
    async fn test_capability_based_resolution() {
        let health_monitor = Arc::new(PrimalHealthMonitor::builder().build());
        let retry_policy = RetryPolicy::exponential(1, Duration::from_millis(100));

        let orchestrator = PrimalOrchestrator::new(health_monitor, retry_policy);

        // Create dependency chain by capability:
        // crypto_provider (provides Security) <- discovery (requires Security) <- app (requires Discovery)

        let crypto_provider = Arc::new(MockPrimal {
            id: PrimalId::new("crypto-provider-1".to_string()).unwrap(),
            provides: vec![Capability::Security],
            requires: vec![],
        });

        let discovery = Arc::new(MockPrimal {
            id: PrimalId::new("discovery-service-1".to_string()).unwrap(),
            provides: vec![Capability::Discovery],
            requires: vec![Capability::Security],
        });

        let app = Arc::new(MockPrimal {
            id: PrimalId::new("app-1".to_string()).unwrap(),
            provides: vec![],
            requires: vec![Capability::Discovery],
        });

        orchestrator.register(app).await;
        orchestrator.register(discovery).await;
        orchestrator.register(crypto_provider).await;

        let order = orchestrator.resolve_dependencies().await.unwrap();

        // Crypto provider should come first, then discovery, then app
        assert_eq!(order[0].to_string(), "crypto-provider-1");
        assert_eq!(order[1].to_string(), "discovery-service-1");
        assert_eq!(order[2].to_string(), "app-1");
    }
}
