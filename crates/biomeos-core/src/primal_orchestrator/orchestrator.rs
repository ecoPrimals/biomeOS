// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Primal Orchestrator - Async, concurrent primal lifecycle management
//!
//! Handles complex startup choreography with dependency resolution.

use std::{collections::HashMap, sync::Arc, time::Duration};

use tokio::{
    sync::RwLock,
    time::{sleep, timeout},
};
use tracing::{debug, error, info, instrument, warn};

use biomeos_types::{
    error::{BiomeError, BiomeResult},
    identifiers::PrimalId,
};

#[cfg(test)]
use biomeos_types::identifiers::Endpoint;

#[cfg(test)]
use crate::discovery_modern::HealthStatus;

use crate::capabilities::Capability;
use crate::retry::RetryPolicy;

use super::health_monitor::PrimalHealthMonitor;
use super::state::{ManagedPrimal, PrimalRecord, PrimalState};

/// Orchestrates primal lifecycle with dependency resolution
pub struct PrimalOrchestrator {
    primals: Arc<RwLock<HashMap<PrimalId, PrimalRecord>>>,
    health_monitor: Arc<PrimalHealthMonitor>,
    retry_policy: RetryPolicy,
}

impl PrimalOrchestrator {
    /// Create a new orchestrator
    #[must_use]
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

        let start_order = self.resolve_dependencies().await?;

        info!("📋 Start order: {:?}", start_order);

        for primal_id in start_order {
            self.start_primal(&primal_id).await?;
        }

        info!("✅ All primals started successfully");
        Ok(())
    }

    /// Start a specific primal (with capability-based dependencies)
    #[instrument(skip(self))]
    pub fn start_primal<'a>(
        &'a self,
        id: &'a PrimalId,
    ) -> std::pin::Pin<Box<dyn std::future::Future<Output = BiomeResult<()>> + Send + 'a>> {
        Box::pin(self.start_primal_inner(id))
    }

    async fn start_primal_inner(&self, id: &PrimalId) -> BiomeResult<()> {
        info!("Starting primal: {}", id);

        let (primal, required_caps) = {
            let primals = self.primals.read().await;
            let record = primals.get(id).ok_or_else(|| {
                BiomeError::discovery_failed(
                    format!("Primal not found: {id}"),
                    Some(id.to_string()),
                )
            })?;

            if record.state == PrimalState::Running {
                info!("Primal {} already running", id);
                return Ok(());
            }

            (record.primal.clone(), record.primal.requires().to_vec())
        };

        for required_cap in &required_caps {
            debug!("Ensuring capability provider for: {}", required_cap);
            self.ensure_capability_provider(required_cap).await?;
        }

        {
            let mut primals = self.primals.write().await;
            if let Some(record) = primals.get_mut(id) {
                record.state = PrimalState::Starting;
            }
        }

        let start_result = self
            .retry_policy
            .execute(|| async {
                primal
                    .start()
                    .await
                    .map_err(|e| anyhow::anyhow!("Start failed: {e}"))
            })
            .await;

        match start_result {
            Ok(()) => {
                info!("✅ Primal {} started", id);

                match timeout(primal.startup_timeout(), self.wait_for_health(&primal)).await {
                    Ok(Ok(())) => {
                        if let Some(endpoint) = primal.endpoint().await {
                            self.health_monitor.register(id.clone(), endpoint).await;
                        }

                        let mut primals = self.primals.write().await;
                        if let Some(record) = primals.get_mut(id) {
                            record.state = PrimalState::Running;
                        }

                        info!("✅ Primal {} is healthy and running", id);
                        Ok(())
                    }
                    Ok(Err(e)) => {
                        error!("Primal {} failed health check: {}", id, e);
                        self.mark_failed(id, format!("Health check failed: {e}"))
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
                    format!("Failed to start {id}: {e}"),
                    Some("primal_start_failure"),
                ))
            }
        }
    }

    /// Ensure at least one provider for a capability is running
    async fn ensure_capability_provider(&self, capability: &Capability) -> BiomeResult<()> {
        let providers: Vec<_> = {
            let primals = self.primals.read().await;
            primals
                .iter()
                .filter(|(_, record)| record.primal.provides().contains(capability))
                .map(|(id, _)| id.clone())
                .collect()
        };

        if providers.is_empty() {
            return Err(BiomeError::discovery_failed(
                format!("No provider found for capability: {capability}"),
                Some(format!("capability:{capability:?}")),
            ));
        }

        for provider_id in providers {
            let state = self.get_state(&provider_id).await;
            if state == Some(PrimalState::Running) {
                debug!(
                    "Capability {} already provided by {}",
                    capability, provider_id
                );
                return Ok(());
            }

            match self.start_primal(&provider_id).await {
                Ok(()) => {
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
                }
            }
        }

        Err(BiomeError::internal_error(
            format!("All providers for capability {capability} failed to start"),
            Some("capability_startup_failure"),
        ))
    }

    /// Stop a specific primal
    #[instrument(skip(self))]
    pub async fn stop_primal(&self, id: &PrimalId) -> BiomeResult<()> {
        info!("Stopping primal: {}", id);

        let primal = {
            let primals = self.primals.read().await;
            let record = primals.get(id).ok_or_else(|| {
                BiomeError::discovery_failed(
                    format!("Primal not found: {id}"),
                    Some(id.to_string()),
                )
            })?;

            if record.state == PrimalState::Stopped {
                info!("Primal {} already stopped", id);
                return Ok(());
            }

            record.primal.clone()
        };

        self.health_monitor.unregister(id).await;

        primal.stop().await.map_err(|e| {
            BiomeError::internal_error(
                format!("Failed to stop primal {id}: {e}"),
                Some("primal_stop_failure"),
            )
        })?;

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

        let mut stop_order = self.resolve_dependencies().await?;
        stop_order.reverse();

        info!("📋 Stop order: {:?}", stop_order);

        for primal_id in stop_order {
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

    /// Resolve dependency order for startup (capability-based)
    pub async fn resolve_dependencies(&self) -> BiomeResult<Vec<PrimalId>> {
        let primals = self.primals.read().await;

        let mut capability_providers: HashMap<Capability, Vec<PrimalId>> = HashMap::new();
        let mut primal_requirements: HashMap<PrimalId, Vec<Capability>> = HashMap::new();

        for (id, record) in primals.iter() {
            for cap in record.primal.provides() {
                capability_providers
                    .entry(cap.clone())
                    .or_default()
                    .push(id.clone());
            }

            primal_requirements.insert(id.clone(), record.primal.requires().to_vec());
        }

        let mut in_degree: HashMap<PrimalId, usize> = HashMap::new();
        let mut graph: HashMap<PrimalId, Vec<PrimalId>> = HashMap::new();

        for (consumer_id, required_caps) in &primal_requirements {
            in_degree.entry(consumer_id.clone()).or_insert(0);

            for required_cap in required_caps {
                if let Some(providers) = capability_providers.get(required_cap) {
                    for provider_id in providers {
                        graph
                            .entry(provider_id.clone())
                            .or_default()
                            .push(consumer_id.clone());

                        *in_degree.entry(consumer_id.clone()).or_insert(0) += 1;
                    }
                }
            }
        }

        let mut queue: Vec<PrimalId> = in_degree
            .iter()
            .filter(|&(_, &degree)| degree == 0)
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

#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};

    struct MockPrimal {
        id: PrimalId,
        provides: Vec<Capability>,
        requires: Vec<Capability>,
    }

    #[async_trait::async_trait]
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

    struct InstrumentedPrimal {
        id: PrimalId,
        provides: Vec<Capability>,
        requires: Vec<Capability>,
        started: AtomicBool,
        start_count: AtomicU32,
        stop_count: AtomicU32,
        fail_start: AtomicBool,
        fail_health: AtomicBool,
    }

    impl InstrumentedPrimal {
        fn new(name: &str, provides: Vec<Capability>, requires: Vec<Capability>) -> Self {
            Self {
                id: PrimalId::new(name).expect("valid name"),
                provides,
                requires,
                started: AtomicBool::new(false),
                start_count: AtomicU32::new(0),
                stop_count: AtomicU32::new(0),
                fail_start: AtomicBool::new(false),
                fail_health: AtomicBool::new(false),
            }
        }
    }

    #[async_trait::async_trait]
    impl ManagedPrimal for InstrumentedPrimal {
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
            self.start_count.fetch_add(1, Ordering::SeqCst);
            if self.fail_start.load(Ordering::SeqCst) {
                return Err(BiomeError::internal_error(
                    "mock start failure",
                    Some("test"),
                ));
            }
            self.started.store(true, Ordering::SeqCst);
            Ok(())
        }
        async fn stop(&self) -> BiomeResult<()> {
            self.stop_count.fetch_add(1, Ordering::SeqCst);
            self.started.store(false, Ordering::SeqCst);
            Ok(())
        }
        async fn health_check(&self) -> BiomeResult<HealthStatus> {
            if self.fail_health.load(Ordering::SeqCst) {
                Ok(HealthStatus::Unhealthy)
            } else {
                Ok(HealthStatus::Healthy)
            }
        }
        fn startup_timeout(&self) -> Duration {
            Duration::from_secs(2)
        }
    }

    fn pid(name: &str) -> PrimalId {
        PrimalId::new(name).expect("valid primal id")
    }

    fn make_orchestrator() -> PrimalOrchestrator {
        let monitor = Arc::new(PrimalHealthMonitor::builder().build());
        let retry = RetryPolicy::exponential(1, Duration::from_millis(10));
        PrimalOrchestrator::new(monitor, retry)
    }

    #[tokio::test]
    async fn test_orchestrator_register_and_get_state() {
        let orch = make_orchestrator();
        let primal = Arc::new(InstrumentedPrimal::new("test-svc", vec![], vec![]));

        assert_eq!(orch.get_state(&pid("test-svc")).await, None);

        orch.register(primal).await;
        assert_eq!(
            orch.get_state(&pid("test-svc")).await,
            Some(PrimalState::Pending)
        );
    }

    #[tokio::test]
    async fn test_orchestrator_get_all_states_empty() {
        let orch = make_orchestrator();
        assert!(orch.get_all_states().await.is_empty());
    }

    #[tokio::test]
    async fn test_orchestrator_get_all_states_multiple() {
        let orch = make_orchestrator();
        orch.register(Arc::new(InstrumentedPrimal::new("a", vec![], vec![])))
            .await;
        orch.register(Arc::new(InstrumentedPrimal::new("b", vec![], vec![])))
            .await;

        let states = orch.get_all_states().await;
        assert_eq!(states.len(), 2);
        assert_eq!(states[&pid("a")], PrimalState::Pending);
        assert_eq!(states[&pid("b")], PrimalState::Pending);
    }

    #[tokio::test]
    async fn test_orchestrator_start_primal_not_found() {
        let orch = make_orchestrator();
        let result = orch.start_primal(&pid("nonexistent")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_orchestrator_stop_primal_not_found() {
        let orch = make_orchestrator();
        let result = orch.stop_primal(&pid("nonexistent")).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_orchestrator_start_and_state_transition() {
        let orch = make_orchestrator();
        let primal = Arc::new(InstrumentedPrimal::new("healthy-svc", vec![], vec![]));
        orch.register(primal.clone()).await;

        let result = orch.start_primal(&pid("healthy-svc")).await;
        assert!(result.is_ok(), "healthy primal should start: {result:?}");

        assert_eq!(
            orch.get_state(&pid("healthy-svc")).await,
            Some(PrimalState::Running)
        );

        assert_eq!(primal.start_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_orchestrator_start_already_running() {
        let orch = make_orchestrator();
        let primal = Arc::new(InstrumentedPrimal::new("already", vec![], vec![]));
        orch.register(primal.clone()).await;

        orch.start_primal(&pid("already"))
            .await
            .expect("first start");
        assert_eq!(primal.start_count.load(Ordering::SeqCst), 1);

        orch.start_primal(&pid("already"))
            .await
            .expect("second start");
        assert_eq!(
            primal.start_count.load(Ordering::SeqCst),
            1,
            "should not call start() again"
        );
    }

    #[tokio::test]
    async fn test_orchestrator_stop_primal() {
        let orch = make_orchestrator();
        let primal = Arc::new(InstrumentedPrimal::new("stoppable", vec![], vec![]));
        orch.register(primal.clone()).await;

        orch.start_primal(&pid("stoppable")).await.expect("start");
        assert_eq!(
            orch.get_state(&pid("stoppable")).await,
            Some(PrimalState::Running)
        );

        orch.stop_primal(&pid("stoppable")).await.expect("stop");
        assert_eq!(
            orch.get_state(&pid("stoppable")).await,
            Some(PrimalState::Stopped)
        );
        assert_eq!(primal.stop_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn test_orchestrator_stop_already_stopped() {
        let orch = make_orchestrator();
        let primal = Arc::new(InstrumentedPrimal::new("double-stop", vec![], vec![]));
        orch.register(primal.clone()).await;

        orch.start_primal(&pid("double-stop")).await.expect("start");
        orch.stop_primal(&pid("double-stop")).await.expect("stop 1");
        orch.stop_primal(&pid("double-stop"))
            .await
            .expect("stop 2 should short-circuit");
        assert_eq!(
            primal.stop_count.load(Ordering::SeqCst),
            1,
            "stop() should only be called once"
        );
    }

    #[tokio::test]
    async fn test_orchestrator_start_with_failed_start() {
        let orch = make_orchestrator();
        let primal = Arc::new(InstrumentedPrimal::new("fail-start", vec![], vec![]));
        primal.fail_start.store(true, Ordering::SeqCst);
        orch.register(primal.clone()).await;

        let result = orch.start_primal(&pid("fail-start")).await;
        assert!(result.is_err(), "start should fail");

        match orch.get_state(&pid("fail-start")).await {
            Some(PrimalState::Failed { reason }) => {
                assert!(!reason.is_empty());
            }
            other => panic!("expected Failed state, got {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_capability_based_resolution() {
        let health_monitor = Arc::new(PrimalHealthMonitor::builder().build());
        let retry_policy = RetryPolicy::exponential(1, Duration::from_millis(100));

        let orchestrator = PrimalOrchestrator::new(health_monitor, retry_policy);

        let crypto_provider = Arc::new(MockPrimal {
            id: PrimalId::new("crypto-provider-1").expect("valid id"),
            provides: vec![Capability::Security],
            requires: vec![],
        });

        let discovery = Arc::new(MockPrimal {
            id: PrimalId::new("discovery-service-1").expect("valid id"),
            provides: vec![Capability::Discovery],
            requires: vec![Capability::Security],
        });

        let app = Arc::new(MockPrimal {
            id: PrimalId::new("app-1").expect("valid id"),
            provides: vec![],
            requires: vec![Capability::Discovery],
        });

        orchestrator.register(app).await;
        orchestrator.register(discovery).await;
        orchestrator.register(crypto_provider).await;

        let order = orchestrator
            .resolve_dependencies()
            .await
            .expect("should resolve");

        assert_eq!(order[0].to_string(), "crypto-provider-1");
        assert_eq!(order[1].to_string(), "discovery-service-1");
        assert_eq!(order[2].to_string(), "app-1");
    }

    #[tokio::test]
    async fn test_resolution_independent_primals() {
        let orch = make_orchestrator();

        orch.register(Arc::new(InstrumentedPrimal::new(
            "alpha",
            vec![Capability::Security],
            vec![],
        )))
        .await;
        orch.register(Arc::new(InstrumentedPrimal::new(
            "beta",
            vec![Capability::Discovery],
            vec![],
        )))
        .await;

        let order = orch.resolve_dependencies().await.expect("should resolve");
        assert_eq!(order.len(), 2);
    }

    #[tokio::test]
    async fn test_resolution_circular_deps() {
        let orch = make_orchestrator();

        orch.register(Arc::new(InstrumentedPrimal::new(
            "a",
            vec![Capability::Security],
            vec![Capability::Discovery],
        )))
        .await;
        orch.register(Arc::new(InstrumentedPrimal::new(
            "b",
            vec![Capability::Discovery],
            vec![Capability::Security],
        )))
        .await;

        let result = orch.resolve_dependencies().await;
        assert!(result.is_err(), "circular deps should fail");
    }

    #[tokio::test]
    async fn test_resolution_empty() {
        let orch = make_orchestrator();
        let order = orch.resolve_dependencies().await.expect("empty is ok");
        assert!(order.is_empty());
    }

    #[tokio::test]
    async fn test_start_all_independent() {
        let orch = make_orchestrator();
        orch.register(Arc::new(InstrumentedPrimal::new("a", vec![], vec![])))
            .await;
        orch.register(Arc::new(InstrumentedPrimal::new("b", vec![], vec![])))
            .await;

        orch.start_all().await.expect("start_all should succeed");

        let states = orch.get_all_states().await;
        assert!(states.values().all(|s| *s == PrimalState::Running));
    }

    #[tokio::test]
    async fn test_stop_all() {
        let orch = make_orchestrator();
        orch.register(Arc::new(InstrumentedPrimal::new("a", vec![], vec![])))
            .await;
        orch.register(Arc::new(InstrumentedPrimal::new("b", vec![], vec![])))
            .await;

        orch.start_all().await.expect("start");
        orch.stop_all().await.expect("stop_all should succeed");

        let states = orch.get_all_states().await;
        assert!(states.values().all(|s| *s == PrimalState::Stopped));
    }

    #[tokio::test]
    async fn test_start_all_with_deps() {
        let orch = make_orchestrator();
        orch.register(Arc::new(InstrumentedPrimal::new(
            "security",
            vec![Capability::Security],
            vec![],
        )))
        .await;
        orch.register(Arc::new(InstrumentedPrimal::new(
            "discovery",
            vec![Capability::Discovery],
            vec![Capability::Security],
        )))
        .await;

        orch.start_all().await.expect("start_all with deps");

        let states = orch.get_all_states().await;
        assert_eq!(states[&pid("security")], PrimalState::Running);
        assert_eq!(states[&pid("discovery")], PrimalState::Running);
    }

    #[test]
    fn test_managed_primal_default_timeout() {
        let primal = MockPrimal {
            id: pid("default-timeout"),
            provides: vec![],
            requires: vec![],
        };
        assert_eq!(primal.startup_timeout(), Duration::from_secs(30));
    }

    #[test]
    fn test_instrumented_primal_custom_timeout() {
        let primal = InstrumentedPrimal::new("custom-timeout", vec![], vec![]);
        assert_eq!(primal.startup_timeout(), Duration::from_secs(2));
    }
}
