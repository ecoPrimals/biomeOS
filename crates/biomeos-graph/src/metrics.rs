// =============================================================================
// Metrics Collection & Storage - Neural API Learning (ecoBin!)
// =============================================================================
//
// Collects and stores graph execution metrics using sled (Pure Rust + ecoBin!)
//
// Deep Debt Principles:
// - 100% Pure Rust (sled - BearDog's proven solution!)
// - FULL cross-compilation (x86, ARM, macOS, RISC-V, etc.)
// - Modern async Rust
// - Safe database operations (no unsafe)
// - Clear error handling
// - TRUE ecoBin compliance!
//
// =============================================================================

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;

use crate::graph::GraphResult;

/// Metrics collector for graph executions (ecoBin compliant!)
#[derive(Clone)]
pub struct MetricsCollector {
    db: Arc<sled::Db>,
}

/// Aggregated metrics for a graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetrics {
    /// Graph name
    pub graph_name: String,

    /// Total executions
    pub total_executions: u64,

    /// Successful executions
    pub successful_executions: u64,

    /// Failed executions
    pub failed_executions: u64,

    /// Average duration in milliseconds
    pub avg_duration_ms: f64,

    /// Min duration in milliseconds
    pub min_duration_ms: u64,

    /// Max duration in milliseconds
    pub max_duration_ms: u64,

    /// Success rate (0.0 to 1.0)
    pub success_rate: f64,

    /// Most recent execution timestamp
    pub last_executed_at: chrono::DateTime<chrono::Utc>,
}

/// Aggregated metrics for a specific node
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeMetricsAggregate {
    /// Node ID
    pub node_id: String,

    /// Total executions
    pub total_executions: u64,

    /// Successful executions
    pub successful_executions: u64,

    /// Average duration in milliseconds
    pub avg_duration_ms: f64,

    /// Success rate
    pub success_rate: f64,
}

/// Execution record stored in database
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionRecord {
    pub id: i64,
    pub graph_name: String,
    pub success: bool,
    pub duration_ms: u64,
    pub executed_at: chrono::DateTime<chrono::Utc>,
    pub metadata: String, // JSON
}

impl MetricsCollector {
    /// Create a new metrics collector (sled - ecoBin compliant!)
    pub async fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let db = sled::open(db_path.as_ref()).context("Failed to open metrics database")?;

        Ok(Self { db: Arc::new(db) })
    }

    /// Record a graph execution (sled storage - ecoBin!)
    pub async fn record_execution(
        &self,
        graph_name: &str,
        result: &GraphResult,
        duration_ms: u64,
    ) -> Result<()> {
        let record = ExecutionRecord {
            id: chrono::Utc::now().timestamp_millis(),
            graph_name: graph_name.to_string(),
            success: result.success,
            duration_ms,
            executed_at: chrono::Utc::now(),
            metadata: serde_json::to_string(&result.node_results).unwrap_or_default(),
        };

        let key = format!("exec:{}:{}", graph_name, record.id);
        let value = serde_json::to_vec(&record).context("Failed to serialize record")?;

        self.db
            .insert(key.as_bytes(), value)
            .context("Failed to insert execution record")?;

        Ok(())
    }

    /// Get aggregated metrics for a graph (sled queries!)
    pub async fn get_graph_metrics(&self, graph_name: &str) -> Result<Option<GraphMetrics>> {
        let prefix = format!("exec:{}:", graph_name);

        let mut total = 0u64;
        let mut successful = 0u64;
        let mut total_duration = 0u64;
        let mut min_duration = u64::MAX;
        let mut max_duration_ms = 0u64;
        let mut last_executed: Option<chrono::DateTime<chrono::Utc>> = None;

        // Iterate through all records for this graph
        for item in self.db.scan_prefix(prefix.as_bytes()) {
            let (_key, value) = item.context("Failed to read database entry")?;

            let record: ExecutionRecord =
                serde_json::from_slice(&value).context("Failed to deserialize record")?;

            total += 1;
            if record.success {
                successful += 1;
            }
            total_duration += record.duration_ms;
            min_duration = min_duration.min(record.duration_ms);
            max_duration_ms = max_duration_ms.max(record.duration_ms);

            if last_executed.is_none() || record.executed_at > last_executed.unwrap() {
                last_executed = Some(record.executed_at);
            }
        }

        if total == 0 {
            return Ok(None);
        }

        Ok(Some(GraphMetrics {
            graph_name: graph_name.to_string(),
            total_executions: total,
            successful_executions: successful,
            failed_executions: total - successful,
            avg_duration_ms: (total_duration as f64) / (total as f64),
            min_duration_ms: if min_duration == u64::MAX {
                0
            } else {
                min_duration
            },
            max_duration_ms,
            success_rate: (successful as f64) / (total as f64),
            last_executed_at: last_executed.unwrap_or_else(chrono::Utc::now),
        }))
    }

    /// Get all tracked graphs
    pub async fn get_tracked_graphs(&self) -> Result<Vec<String>> {
        let mut graphs = std::collections::HashSet::new();

        for item in self.db.scan_prefix(b"exec:") {
            let (key, _) = item.context("Failed to read database entry")?;
            let key_str = String::from_utf8_lossy(&key);

            // Parse "exec:graph_name:timestamp" format
            let parts: Vec<&str> = key_str.split(':').collect();
            if parts.len() >= 2 {
                graphs.insert(parts[1].to_string());
            }
        }

        Ok(graphs.into_iter().collect())
    }

    /// Clear all metrics (for testing or reset)
    pub async fn clear_all(&self) -> Result<()> {
        self.db.clear().context("Failed to clear database")?;
        Ok(())
    }
}

// Stub implementations for compatibility
impl MetricsCollector {
    pub async fn record_node_execution(
        &self,
        _execution_id: i64,
        _node_id: &str,
        _primal_id: &str,
        _operation: &str,
        _success: bool,
        _duration_ms: u64,
        _error: Option<&str>,
    ) -> Result<()> {
        // Simplified: Node-level metrics can be added later if needed
        Ok(())
    }

    pub async fn get_node_metrics(
        &self,
        _graph_name: &str,
        _node_id: &str,
    ) -> Result<Option<NodeMetricsAggregate>> {
        // Simplified: Return None for now
        Ok(None)
    }

    pub async fn get_recent_executions(
        &self,
        _graph_name: &str,
        _limit: usize,
    ) -> Result<Vec<ExecutionRecord>> {
        // Simplified: Return empty for now
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_metrics_collection_ecobin() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("metrics.sled");

        let collector = MetricsCollector::new(&db_path).await.unwrap();

        // Record a successful execution
        let result = GraphResult {
            success: true,
            node_results: Default::default(),
            errors: vec![],
            duration_ms: 100,
        };

        collector
            .record_execution("test_graph", &result, 100)
            .await
            .unwrap();

        // Get metrics
        let metrics = collector.get_graph_metrics("test_graph").await.unwrap();
        assert!(metrics.is_some());

        let metrics = metrics.unwrap();
        assert_eq!(metrics.total_executions, 1);
        assert_eq!(metrics.successful_executions, 1);
        assert_eq!(metrics.success_rate, 1.0);
    }
}
