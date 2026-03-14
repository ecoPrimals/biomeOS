// SPDX-License-Identifier: AGPL-3.0-only
// Copyright (C) 2024–2026 ecoPrimals Project

//! Pathway-learning metrics collector for the Neural API.
//!
//! Tracks primal usage, co-occurrence, latency, and success rates
//! to enable Neural API Phase 3 (Pathway Learning) optimizations.

use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, SystemTime};
use tokio::sync::RwLock;

/// Maximum number of latency samples to retain per primal for p95 computation.
const MAX_LATENCY_SAMPLES: usize = 1000;

/// Metrics collected from graph executions for pathway learning.
#[derive(Debug, Clone)]
pub struct GraphExecutionMetrics {
    /// Unique identifier for the graph.
    pub graph_id: String,
    /// Primal IDs invoked during this execution.
    pub primals_invoked: Vec<String>,
    /// Per-primal latencies (primal_id -> duration).
    pub latencies: HashMap<String, Duration>,
    /// Whether the execution succeeded.
    pub success: bool,
    /// Total wall-clock duration of the execution.
    pub total_duration: Duration,
    /// When the execution was recorded.
    pub timestamp: SystemTime,
}

/// Aggregated stats for a single primal.
#[derive(Debug, Clone, Default)]
pub struct PrimalStats {
    /// Total number of invocations.
    pub invocation_count: u64,
    /// Number of successful invocations.
    pub success_count: u64,
    /// Number of failed invocations.
    pub failure_count: u64,
    /// Sum of all latencies for average computation.
    pub total_latency: Duration,
    /// Minimum observed latency.
    pub min_latency: Duration,
    /// Maximum observed latency.
    pub max_latency: Duration,
    /// Recent latency samples for p95 (capped at MAX_LATENCY_SAMPLES).
    pub latency_samples: Vec<Duration>,
}

impl PrimalStats {
    /// Returns the average latency across all invocations.
    #[must_use]
    pub fn avg_latency(&self) -> Duration {
        if self.invocation_count == 0 {
            return Duration::ZERO;
        }
        self.total_latency / u32::try_from(self.invocation_count).unwrap_or(u32::MAX)
    }

    /// Returns the 95th percentile latency from sampled latencies.
    #[must_use]
    pub fn p95_latency(&self) -> Duration {
        if self.latency_samples.is_empty() {
            return Duration::ZERO;
        }
        let mut sorted: Vec<Duration> = self.latency_samples.clone();
        sorted.sort();
        let idx = (sorted.len() as f64 * 0.95) as usize;
        let idx = idx.min(sorted.len().saturating_sub(1));
        sorted[idx]
    }
}

/// Co-occurrence entry: how often two primals appear in the same graph.
#[derive(Debug, Clone, Default)]
pub struct CoOccurrence {
    /// Number of times these two primals appeared together.
    pub count: u64,
}

/// Inner state of the metrics collector.
#[derive(Debug, Default)]
struct MetricsInner {
    primal_stats: HashMap<String, PrimalStats>,
    co_occurrences: HashMap<(String, String), CoOccurrence>,
    graph_execution_count: HashMap<String, u64>,
    total_executions: u64,
}

/// Central metrics collector for Neural API pathway learning.
#[derive(Debug, Clone)]
pub struct MetricsCollector {
    inner: Arc<RwLock<MetricsInner>>,
}

impl Default for MetricsCollector {
    fn default() -> Self {
        Self::new()
    }
}

impl MetricsCollector {
    /// Creates a new metrics collector.
    #[must_use]
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(MetricsInner::default())),
        }
    }

    /// Records a graph execution and updates all aggregated metrics.
    pub async fn record_execution(&self, metrics: GraphExecutionMetrics) {
        let mut inner = self.inner.write().await;

        inner.total_executions += 1;
        *inner
            .graph_execution_count
            .entry(metrics.graph_id.clone())
            .or_insert(0) += 1;

        for (a, b) in pairs(&metrics.primals_invoked) {
            let key = if a <= b {
                (a.clone(), b.clone())
            } else {
                (b.clone(), a.clone())
            };
            inner.co_occurrences.entry(key).or_default().count += 1;
        }

        for primal_id in &metrics.primals_invoked {
            let latency = metrics
                .latencies
                .get(primal_id)
                .copied()
                .unwrap_or(Duration::ZERO);
            let entry = inner.primal_stats.entry(primal_id.clone()).or_default();
            entry.invocation_count += 1;
            if metrics.success {
                entry.success_count += 1;
            } else {
                entry.failure_count += 1;
            }
            entry.total_latency += latency;
            if entry.invocation_count == 1 {
                entry.min_latency = latency;
                entry.max_latency = latency;
            } else {
                entry.min_latency = entry.min_latency.min(latency);
                entry.max_latency = entry.max_latency.max(latency);
            }
            if entry.latency_samples.len() < MAX_LATENCY_SAMPLES {
                entry.latency_samples.push(latency);
            } else {
                let idx = entry.invocation_count as usize % MAX_LATENCY_SAMPLES;
                entry.latency_samples[idx] = latency;
            }
        }
    }

    /// Returns a snapshot of per-primal statistics.
    pub async fn primal_stats(&self) -> HashMap<String, PrimalStats> {
        self.inner.read().await.primal_stats.clone()
    }

    /// Returns a snapshot of co-occurrence counts.
    pub async fn co_occurrences(&self) -> HashMap<(String, String), CoOccurrence> {
        self.inner.read().await.co_occurrences.clone()
    }

    /// Returns how many times the given graph has been executed.
    pub async fn graph_execution_count(&self, graph_id: &str) -> u64 {
        self.inner
            .read()
            .await
            .graph_execution_count
            .get(graph_id)
            .copied()
            .unwrap_or(0)
    }

    /// Returns the total number of graph executions recorded.
    pub async fn total_executions(&self) -> u64 {
        self.inner.read().await.total_executions
    }

    /// Returns the top `n` primals by invocation count, sorted descending.
    pub async fn top_primals(&self, n: usize) -> Vec<(String, u64)> {
        let inner = self.inner.read().await;
        let mut v: Vec<_> = inner
            .primal_stats
            .iter()
            .map(|(id, s)| (id.clone(), s.invocation_count))
            .collect();
        v.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
        v.truncate(n);
        v
    }

    /// Resets all collected metrics.
    pub async fn reset(&self) {
        let mut inner = self.inner.write().await;
        inner.primal_stats.clear();
        inner.co_occurrences.clear();
        inner.graph_execution_count.clear();
        inner.total_executions = 0;
    }
}

/// Returns all unordered pairs from a slice of strings.
fn pairs(primals: &[String]) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for (i, a) in primals.iter().enumerate() {
        for b in primals.iter().skip(i + 1) {
            out.push((a.clone(), b.clone()));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_metrics(
        graph_id: &str,
        primals: &[&str],
        latencies: &[(&str, u64)],
        success: bool,
        total_ms: u64,
    ) -> GraphExecutionMetrics {
        let latencies_map: HashMap<String, Duration> = latencies
            .iter()
            .map(|(k, v)| (k.to_string(), Duration::from_millis(*v)))
            .collect();
        GraphExecutionMetrics {
            graph_id: graph_id.to_string(),
            primals_invoked: primals.iter().map(|s| s.to_string()).collect(),
            latencies: latencies_map,
            success,
            total_duration: Duration::from_millis(total_ms),
            timestamp: SystemTime::now(),
        }
    }

    #[tokio::test]
    async fn test_record_single_execution() {
        let collector = MetricsCollector::new();
        collector
            .record_execution(make_metrics(
                "graph-1",
                &["primal-a", "primal-b"],
                &[("primal-a", 10), ("primal-b", 20)],
                true,
                30,
            ))
            .await;

        assert_eq!(collector.total_executions().await, 1);
        assert_eq!(collector.graph_execution_count("graph-1").await, 1);
        let stats = collector.primal_stats().await;
        assert_eq!(stats.get("primal-a").unwrap().invocation_count, 1);
        assert_eq!(stats.get("primal-a").unwrap().success_count, 1);
        assert_eq!(
            stats.get("primal-a").unwrap().min_latency,
            Duration::from_millis(10)
        );
        assert_eq!(stats.get("primal-b").unwrap().invocation_count, 1);
    }

    #[tokio::test]
    async fn test_record_multiple_executions() {
        let collector = MetricsCollector::new();
        for _ in 0..5 {
            collector
                .record_execution(make_metrics(
                    "graph-1",
                    &["primal-a"],
                    &[("primal-a", 100)],
                    true,
                    100,
                ))
                .await;
        }
        collector
            .record_execution(make_metrics(
                "graph-2",
                &["primal-b"],
                &[("primal-b", 50)],
                true,
                50,
            ))
            .await;

        assert_eq!(collector.total_executions().await, 6);
        assert_eq!(collector.graph_execution_count("graph-1").await, 5);
        assert_eq!(collector.graph_execution_count("graph-2").await, 1);
        let stats = collector.primal_stats().await;
        assert_eq!(stats.get("primal-a").unwrap().invocation_count, 5);
        assert_eq!(stats.get("primal-b").unwrap().invocation_count, 1);
    }

    #[tokio::test]
    async fn test_co_occurrence_tracking() {
        let collector = MetricsCollector::new();
        collector
            .record_execution(make_metrics(
                "g1",
                &["a", "b", "c"],
                &[("a", 1), ("b", 2), ("c", 3)],
                true,
                6,
            ))
            .await;
        collector
            .record_execution(make_metrics(
                "g2",
                &["a", "b"],
                &[("a", 1), ("b", 2)],
                true,
                3,
            ))
            .await;

        let co = collector.co_occurrences().await;
        assert_eq!(
            co.get(&("a".to_string(), "b".to_string())).unwrap().count,
            2
        );
        assert_eq!(
            co.get(&("a".to_string(), "c".to_string())).unwrap().count,
            1
        );
        assert_eq!(
            co.get(&("b".to_string(), "c".to_string())).unwrap().count,
            1
        );
    }

    #[tokio::test]
    async fn test_latency_stats() {
        let collector = MetricsCollector::new();
        collector
            .record_execution(make_metrics("g1", &["p"], &[("p", 10)], true, 10))
            .await;
        collector
            .record_execution(make_metrics("g1", &["p"], &[("p", 30)], true, 30))
            .await;
        collector
            .record_execution(make_metrics("g1", &["p"], &[("p", 20)], true, 20))
            .await;

        let stats = collector.primal_stats().await;
        let s = stats.get("p").unwrap();
        assert_eq!(s.invocation_count, 3);
        assert_eq!(s.min_latency, Duration::from_millis(10));
        assert_eq!(s.max_latency, Duration::from_millis(30));
        assert_eq!(s.avg_latency(), Duration::from_millis(20));
        assert!(s.p95_latency() >= Duration::from_millis(20));
    }

    #[tokio::test]
    async fn test_top_primals() {
        let collector = MetricsCollector::new();
        collector
            .record_execution(make_metrics(
                "g1",
                &["a", "b"],
                &[("a", 1), ("b", 1)],
                true,
                2,
            ))
            .await;
        for _ in 0..3 {
            collector
                .record_execution(make_metrics("g1", &["a"], &[("a", 1)], true, 1))
                .await;
        }
        collector
            .record_execution(make_metrics("g1", &["c"], &[("c", 1)], true, 1))
            .await;

        let top = collector.top_primals(2).await;
        assert_eq!(top.len(), 2);
        assert_eq!(top[0], ("a".to_string(), 4));
        assert_eq!(top[1], ("b".to_string(), 1));
    }

    #[tokio::test]
    async fn test_reset() {
        let collector = MetricsCollector::new();
        collector
            .record_execution(make_metrics("g1", &["a"], &[("a", 10)], true, 10))
            .await;

        assert_eq!(collector.total_executions().await, 1);
        collector.reset().await;
        assert_eq!(collector.total_executions().await, 0);
        assert_eq!(collector.graph_execution_count("g1").await, 0);
        assert!(collector.primal_stats().await.is_empty());
        assert!(collector.co_occurrences().await.is_empty());
    }
}
