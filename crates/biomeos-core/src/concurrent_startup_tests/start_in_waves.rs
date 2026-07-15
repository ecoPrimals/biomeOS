// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::MockPrimal;
use crate::capabilities::Capability;
use crate::concurrent_startup::start_in_waves;
use crate::primal_orchestrator::{PrimalHealthMonitor, PrimalOrchestrator};
use crate::retry::RetryPolicy;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_start_in_waves_linear_chain() {
    let primals: Vec<Arc<dyn crate::primal_orchestrator::ManagedPrimal>> = vec![
        Arc::new(MockPrimal::new(
            "beardog",
            vec![Capability::Security],
            vec![],
        )),
        Arc::new(MockPrimal::new(
            "songbird",
            vec![Capability::Discovery],
            vec![Capability::Security],
        )),
        Arc::new(MockPrimal::new(
            "nestgate",
            vec![Capability::Storage],
            vec![Capability::Discovery],
        )),
    ];

    let monitor = Arc::new(PrimalHealthMonitor::builder().build());
    let retry = RetryPolicy::exponential(1, Duration::from_millis(10));
    let orchestrator = Arc::new(PrimalOrchestrator::new(monitor, retry));

    for p in &primals {
        orchestrator.register(Arc::clone(p)).await;
    }

    let result = start_in_waves(&orchestrator, primals).await;
    assert!(result.is_ok(), "start_in_waves should succeed: {result:?}");
}

#[tokio::test]
async fn test_start_in_waves_single_primal() {
    let primals: Vec<Arc<dyn crate::primal_orchestrator::ManagedPrimal>> = vec![Arc::new(
        MockPrimal::new("solo", vec![Capability::Security], vec![]),
    )];

    let monitor = Arc::new(PrimalHealthMonitor::builder().build());
    let retry = RetryPolicy::exponential(1, Duration::from_millis(10));
    let orchestrator = Arc::new(PrimalOrchestrator::new(monitor, retry));

    orchestrator.register(Arc::clone(&primals[0])).await;

    let result = start_in_waves(&orchestrator, primals).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_start_in_waves_parallel_wave() {
    let primals: Vec<Arc<dyn crate::primal_orchestrator::ManagedPrimal>> = vec![
        Arc::new(MockPrimal::new(
            "beardog",
            vec![Capability::Security],
            vec![],
        )),
        Arc::new(MockPrimal::new(
            "nestgate",
            vec![Capability::Storage],
            vec![Capability::Security],
        )),
        Arc::new(MockPrimal::new(
            "toadstool",
            vec![Capability::Compute],
            vec![Capability::Security],
        )),
    ];

    let monitor = Arc::new(PrimalHealthMonitor::builder().build());
    let retry = RetryPolicy::exponential(1, Duration::from_millis(10));
    let orchestrator = Arc::new(PrimalOrchestrator::new(monitor, retry));

    for p in &primals {
        orchestrator.register(Arc::clone(p)).await;
    }

    let result = start_in_waves(&orchestrator, primals).await;
    assert!(result.is_ok());
}
