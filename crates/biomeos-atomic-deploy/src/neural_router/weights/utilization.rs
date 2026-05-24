// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Capability utilization tracking — method-level call frequency monitoring.
//!
//! Records how often each method is called and when it was last used.
//! Input feature layer for future learned routing: hot methods get
//! pre-staged, cold methods get lazy-loaded.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Tracks capability method utilization.
///
/// Records call counts and last-used timestamps per method.
#[derive(Debug, Default)]
pub struct CapabilityUtilizationTracker {
    /// method → (call_count, last_called_epoch_ms)
    counters: HashMap<String, (u64, u64)>,
}

impl CapabilityUtilizationTracker {
    /// Create a new empty tracker.
    pub fn new() -> Self {
        Self::default()
    }

    /// Record a call to a method.
    pub fn record(&mut self, method: &str) {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_millis() as u64;
        let entry = self.counters.entry(method.to_owned()).or_insert((0, 0));
        entry.0 += 1;
        entry.1 = now;
    }

    /// Get the call count for a method.
    pub fn call_count(&self, method: &str) -> u64 {
        self.counters.get(method).map_or(0, |e| e.0)
    }

    /// Get the last-called timestamp for a method (epoch ms).
    pub fn last_called(&self, method: &str) -> Option<u64> {
        self.counters.get(method).map(|e| e.1)
    }

    /// Number of distinct methods tracked.
    pub fn tracked_methods(&self) -> usize {
        self.counters.len()
    }

    /// Get the top-N hottest methods by call count.
    pub fn hot_methods(&self, n: usize) -> Vec<MethodUtilization> {
        let mut methods: Vec<_> = self
            .counters
            .iter()
            .map(|(method, (count, last))| MethodUtilization {
                method: method.clone(),
                call_count: *count,
                last_called_epoch_ms: *last,
            })
            .collect();
        methods.sort_by_key(|m| std::cmp::Reverse(m.call_count));
        methods.truncate(n);
        methods
    }

    /// Get the coldest methods (called fewer than `threshold` times).
    pub fn cold_methods(&self, threshold: u64) -> Vec<MethodUtilization> {
        self.counters
            .iter()
            .filter(|(_, (count, _))| *count < threshold)
            .map(|(method, (count, last))| MethodUtilization {
                method: method.clone(),
                call_count: *count,
                last_called_epoch_ms: *last,
            })
            .collect()
    }

    /// Summary statistics for the utilization tracker.
    pub fn summary(&self) -> UtilizationSummary {
        let total_calls: u64 = self.counters.values().map(|(c, _)| c).sum();
        let max_calls = self.counters.values().map(|(c, _)| *c).max().unwrap_or(0);
        let min_calls = self.counters.values().map(|(c, _)| *c).min().unwrap_or(0);
        UtilizationSummary {
            tracked_methods: self.counters.len(),
            total_calls,
            max_calls_single_method: max_calls,
            min_calls_single_method: min_calls,
        }
    }

    /// Serialize to JSON (for RPC responses).
    pub fn to_json(&self) -> serde_json::Value {
        let summary = self.summary();
        let hot = self.hot_methods(10);
        serde_json::json!({
            "tracked_methods": summary.tracked_methods,
            "total_calls": summary.total_calls,
            "max_calls_single_method": summary.max_calls_single_method,
            "min_calls_single_method": summary.min_calls_single_method,
            "hot_methods": hot.iter().map(|m| serde_json::json!({
                "method": m.method,
                "call_count": m.call_count,
                "last_called_epoch_ms": m.last_called_epoch_ms,
            })).collect::<Vec<_>>(),
        })
    }
}

/// A single method's utilization data.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodUtilization {
    /// Method name (e.g. "crypto.sha256").
    pub method: String,
    /// Number of times this method has been called.
    pub call_count: u64,
    /// Unix epoch milliseconds when this method was last called.
    pub last_called_epoch_ms: u64,
}

/// Summary of capability utilization across all tracked methods.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtilizationSummary {
    /// Number of distinct methods tracked.
    pub tracked_methods: usize,
    /// Sum of all method calls.
    pub total_calls: u64,
    /// Highest call count for a single method.
    pub max_calls_single_method: u64,
    /// Lowest call count for a single method.
    pub min_calls_single_method: u64,
}
