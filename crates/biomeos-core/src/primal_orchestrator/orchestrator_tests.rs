// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::sync::Arc;
use std::time::Duration;

use biomeos_types::error::{BiomeError, BiomeResult};
use biomeos_types::identifiers::{Endpoint, PrimalId};

use crate::capabilities::Capability;
use crate::discovery_modern::HealthStatus;
use crate::retry::RetryPolicy;

use super::health_monitor::PrimalHealthMonitor;
use super::{ManagedPrimal, PrimalOrchestrator, PrimalState};

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
