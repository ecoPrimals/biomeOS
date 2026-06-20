// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Post-start health polling until a managed primal reports healthy.

use std::sync::Arc;
use std::time::Duration;

use biomeos_types::error::{BiomeError, BiomeResult};
use tokio::time::sleep;
use tracing::debug;

use super::state::ManagedPrimal;

pub(crate) async fn wait_for_managed_primal_health(
    primal: &Arc<dyn ManagedPrimal>,
) -> BiomeResult<()> {
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

#[cfg(test)]
#[expect(clippy::unwrap_used, reason = "test assertions")]
mod tests {
    // SPDX-License-Identifier: AGPL-3.0-or-later
    // Copyright 2025-2026 ecoPrimals Project

    use std::future::Future;
    use std::pin::Pin;
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU32, Ordering};

    use biomeos_types::error::{BiomeError, BiomeResult};
    use biomeos_types::identifiers::{Endpoint, PrimalId};

    use crate::capabilities::Capability;
    use crate::discovery_modern::HealthStatus;

    use super::*;

    struct MockPrimal {
        id: PrimalId,
        healthy_after: AtomicU32,
        call_count: AtomicU32,
        fail_with_error: bool,
    }

    impl MockPrimal {
        fn always_healthy() -> Arc<Self> {
            Arc::new(Self {
                id: PrimalId::new("test-primal").unwrap(),
                healthy_after: AtomicU32::new(1),
                call_count: AtomicU32::new(0),
                fail_with_error: false,
            })
        }

        fn healthy_on_attempt(n: u32) -> Arc<Self> {
            Arc::new(Self {
                id: PrimalId::new("test-primal").unwrap(),
                healthy_after: AtomicU32::new(n),
                call_count: AtomicU32::new(0),
                fail_with_error: false,
            })
        }

        fn always_unhealthy() -> Arc<Self> {
            Arc::new(Self {
                id: PrimalId::new("test-primal").unwrap(),
                healthy_after: AtomicU32::new(u32::MAX),
                call_count: AtomicU32::new(0),
                fail_with_error: false,
            })
        }

        fn always_errors() -> Arc<Self> {
            Arc::new(Self {
                id: PrimalId::new("test-primal").unwrap(),
                healthy_after: AtomicU32::new(u32::MAX),
                call_count: AtomicU32::new(0),
                fail_with_error: true,
            })
        }
    }

    impl ManagedPrimal for MockPrimal {
        fn id(&self) -> &PrimalId {
            &self.id
        }
        fn provides(&self) -> &[Capability] {
            &[]
        }
        fn requires(&self) -> &[Capability] {
            &[]
        }
        fn endpoint(&self) -> Pin<Box<dyn Future<Output = Option<Endpoint>> + Send + '_>> {
            Box::pin(async { None })
        }
        fn start(&self) -> Pin<Box<dyn Future<Output = BiomeResult<()>> + Send + '_>> {
            Box::pin(async { Ok(()) })
        }
        fn stop(&self) -> Pin<Box<dyn Future<Output = BiomeResult<()>> + Send + '_>> {
            Box::pin(async { Ok(()) })
        }
        fn health_check(
            &self,
        ) -> Pin<Box<dyn Future<Output = BiomeResult<HealthStatus>> + Send + '_>> {
            let count = self.call_count.fetch_add(1, Ordering::SeqCst) + 1;
            let threshold = self.healthy_after.load(Ordering::SeqCst);
            let fail_err = self.fail_with_error;
            Box::pin(async move {
                if fail_err {
                    return Err(BiomeError::internal_error(
                        "mock health check failure",
                        None::<String>,
                    ));
                }
                if count >= threshold {
                    Ok(HealthStatus::Healthy)
                } else {
                    Ok(HealthStatus::Unhealthy)
                }
            })
        }
    }

    #[tokio::test]
    async fn healthy_on_first_attempt() {
        let primal = MockPrimal::always_healthy();
        let result =
            wait_for_managed_primal_health(&(primal.clone() as Arc<dyn ManagedPrimal>)).await;
        assert!(result.is_ok());
        assert_eq!(primal.call_count.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn healthy_after_retries() {
        tokio::time::pause();
        let primal = MockPrimal::healthy_on_attempt(3);
        let result =
            wait_for_managed_primal_health(&(primal.clone() as Arc<dyn ManagedPrimal>)).await;
        assert!(result.is_ok());
        assert_eq!(primal.call_count.load(Ordering::SeqCst), 3);
    }

    #[tokio::test]
    async fn timeout_after_max_attempts() {
        tokio::time::pause();
        let primal = MockPrimal::always_unhealthy();
        let result =
            wait_for_managed_primal_health(&(primal.clone() as Arc<dyn ManagedPrimal>)).await;
        assert!(result.is_err());
        let err = result.unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("timeout") || msg.contains("Timeout"),
            "expected timeout error, got: {msg}"
        );
        assert_eq!(primal.call_count.load(Ordering::SeqCst), 10);
    }

    #[tokio::test]
    async fn error_during_health_check_retries() {
        tokio::time::pause();
        let primal = MockPrimal::always_errors();
        let result =
            wait_for_managed_primal_health(&(primal.clone() as Arc<dyn ManagedPrimal>)).await;
        assert!(result.is_err());
        assert_eq!(primal.call_count.load(Ordering::SeqCst), 10);
    }

    #[tokio::test]
    async fn healthy_on_last_attempt() {
        tokio::time::pause();
        let primal = MockPrimal::healthy_on_attempt(10);
        let result =
            wait_for_managed_primal_health(&(primal.clone() as Arc<dyn ManagedPrimal>)).await;
        assert!(result.is_ok());
        assert_eq!(primal.call_count.load(Ordering::SeqCst), 10);
    }
}
