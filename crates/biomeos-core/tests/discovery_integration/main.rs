// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Integration tests for BiomeOS discovery system
//!
//! These tests validate the complete discovery workflow:
//! - Primal discovery across different architectures (REST, CLI, mDNS)
//! - Capability-based queries
//! - Multi-primal coordination
//! - Federation discovery
//! - Runtime adaptation
//!
//! **Concurrency-First Design**:
//! - Service polling uses exponential backoff for efficiency
//! - Minimal delays, fast failure detection
//! - Optimized for concurrent test execution

mod live_service;
mod primal_discovery;
mod resilience;

use std::path::Path;
use std::time::Duration;

pub(crate) fn http_get(url: &str, timeout_secs: u64) -> Result<(u16, String), String> {
    ureq::get(url)
        .timeout(Duration::from_secs(timeout_secs))
        .call()
        .map(|resp| {
            let status = resp.status();
            let body = resp.into_string().unwrap_or_default();
            (status, body)
        })
        .map_err(|e| e.to_string())
}

/// Wait for service with exponential backoff (production-grade polling).
///
/// Uses ureq (pure Rust, ecoBin v2.0 compliant) instead of reqwest.
pub(crate) async fn wait_for_service(url: &str, max_attempts: u32) -> bool {
    let url = url.to_string();
    let mut delay_ms = 10u64;

    for attempt in 0..max_attempts {
        let url_clone = url.clone();
        let result = tokio::task::spawn_blocking(move || {
            ureq::get(&url_clone)
                .timeout(Duration::from_secs(1))
                .call()
                .map_err(|e| e.to_string())
        })
        .await;

        if let Ok(Ok(response)) = result {
            if response.status() >= 200 && response.status() < 300 {
                return true;
            }
        }

        if attempt < max_attempts - 1 {
            tokio::time::sleep(Duration::from_millis(delay_ms)).await;
            delay_ms = (delay_ms * 2).min(500);
        }
    }
    false
}

pub(crate) fn find_primal_binary(name: &str) -> Option<std::path::PathBuf> {
    let locations = vec![
        format!("primals/{}", name),
        format!("../phase1/{}/target/release/{}", name, name),
    ];

    for location in locations {
        let path = Path::new(&location);
        if path.exists() {
            return Some(path.to_path_buf());
        }
    }
    None
}
