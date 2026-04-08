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
