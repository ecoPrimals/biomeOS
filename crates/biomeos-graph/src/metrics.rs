// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Metrics collection and storage for graph execution (ecoBin compliant).
//!
//! Uses redb for persistent storage. Records graph and node-level execution
//! metrics for aggregation and learning.

use anyhow::{Context, Result};
use redb::{Database, TableDefinition};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

/// Key-value table for metrics storage
const METRICS_TABLE: TableDefinition<&str, &[u8]> = TableDefinition::new("metrics");

/// Returns the exclusive end bound for a prefix range (e.g. "exec:graph:" -> "exec:graph;")
fn prefix_end(prefix: &str) -> String {
    let mut s = prefix.to_string();
    if let Some(last) = s.pop() {
        s.push(char::from_u32(last as u32 + 1).unwrap_or('\u{10ffff}'));
    }
    s
}

/// Result of a graph execution (used by record_execution).
#[derive(Debug, Clone, Default)]
pub struct GraphResult {
    /// Whether the graph execution completed successfully.
    pub success: bool,
    /// Per-node outputs keyed by node ID.
    pub node_results: HashMap<String, serde_json::Value>,
    /// Error messages from failed nodes.
    pub errors: Vec<String>,
    /// Total execution duration in milliseconds.
    pub duration_ms: u64,
}

/// Metrics collector for graph executions (ecoBin compliant!)
#[derive(Clone)]
pub struct MetricsCollector {
    db: Arc<Database>,
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
    /// Unique execution ID (timestamp-based).
    pub id: i64,
    /// Name of the graph that was executed.
    pub graph_name: String,
    /// Whether the execution succeeded.
    pub success: bool,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
    /// Wall-clock time when execution completed.
    pub executed_at: chrono::DateTime<chrono::Utc>,
    /// Execution metadata as JSON string (node results, etc.).
    pub metadata: String,
}

/// Node-level execution record (stored for aggregation)
#[derive(Debug, Clone, Serialize, Deserialize)]
struct NodeExecutionRecord {
    graph_name: String,
    node_id: String,
    duration_ms: u64,
    success: bool,
    executed_at: chrono::DateTime<chrono::Utc>,
}

impl MetricsCollector {
    /// Create a new metrics collector (redb - ecoBin compliant!)
    pub async fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let path = db_path.as_ref();
        let db = Database::create(path).map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to open metrics database")?;

        // Ensure the metrics table exists (redb creates tables on first write)
        let txn = db.begin_write().map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to initialize metrics database")?;
        {
            let _ = txn.open_table(METRICS_TABLE).map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to create metrics table")?;
        }
        txn.commit().map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to commit initialization")?;

        Ok(Self { db: Arc::new(db) })
    }

    /// Record a graph execution (redb storage - ecoBin!)
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
            metadata: serde_json::to_string(&result.node_results).unwrap_or_else(|e| {
                tracing::warn!("JSON parse fallback: {}", e);
                Default::default()
            }),
        };

        let key = format!("exec:{}:{}", graph_name, record.id);
        let value = serde_json::to_vec(&record).context("Failed to serialize record")?;

        let txn = self.db.begin_write().map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to begin write transaction")?;
        {
            let mut table = txn.open_table(METRICS_TABLE).map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to open metrics table")?;
            table.insert(key.as_str(), value.as_slice()).map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to insert execution record")?;
        }
        txn.commit().map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to commit transaction")?;

        Ok(())
    }

    /// Get aggregated metrics for a graph (redb queries!)
    pub async fn get_graph_metrics(&self, graph_name: &str) -> Result<Option<GraphMetrics>> {
        let prefix = format!("exec:{}:", graph_name);
        let end = prefix_end(&prefix);

        let txn = self.db.begin_read().map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to begin read transaction")?;
        let table = txn.open_table(METRICS_TABLE).map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to open metrics table")?;

        let mut total = 0u64;
        let mut successful = 0u64;
        let mut total_duration = 0u64;
        let mut min_duration = u64::MAX;
        let mut max_duration_ms = 0u64;
        let mut last_executed: Option<chrono::DateTime<chrono::Utc>> = None;

        for item in table.range(prefix.as_str()..end.as_str()).map_err(|e| anyhow::anyhow!("{}", e))? {
            let (_key, value) = item.map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to read database entry")?;
            let value = value.value();

            let record: ExecutionRecord =
                serde_json::from_slice(value).context("Failed to deserialize record")?;

            total += 1;
            if record.success {
                successful += 1;
            }
            total_duration += record.duration_ms;
            min_duration = min_duration.min(record.duration_ms);
            max_duration_ms = max_duration_ms.max(record.duration_ms);

            if last_executed.is_none_or(|prev| record.executed_at > prev) {
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
        let end = prefix_end("exec:");

        let txn = self.db.begin_read().map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to begin read transaction")?;
        let table = txn.open_table(METRICS_TABLE).map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to open metrics table")?;

        for item in table.range("exec:"..end.as_str()).map_err(|e| anyhow::anyhow!("{}", e))? {
            let (key, _) = item.map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to read database entry")?;
            let key_str = key.value();

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
        let txn = self.db.begin_read().map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to begin read transaction")?;
        let table = txn.open_table(METRICS_TABLE).map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to open metrics table")?;

        let keys: Vec<String> = table
            .range::<&str>(""..)
            .map_err(|e| anyhow::anyhow!("{}", e))?
            .filter_map(|item| {
                item.map(|(k, _)| k.value().to_string()).ok()
            })
            .collect();

        drop(table);
        drop(txn);

        let write_txn = self.db.begin_write().map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to begin write transaction")?;
        {
            let mut table = write_txn.open_table(METRICS_TABLE).map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to open metrics table")?;
            for key in keys {
                table.remove(key.as_str()).map_err(|e| anyhow::anyhow!("{}", e))?;
            }
        }
        write_txn.commit().map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to commit transaction")?;

        Ok(())
    }
}

impl MetricsCollector {
    /// Record a node-level execution for metrics aggregation.
    /// Call with the same graph_name used in record_execution for this run.
    #[allow(clippy::too_many_arguments)]
    pub async fn record_node_execution(
        &self,
        execution_id: i64,
        graph_name: &str,
        node_id: &str,
        _primal_id: &str,
        _operation: &str,
        success: bool,
        duration_ms: u64,
        _error: Option<&str>,
    ) -> Result<()> {
        let record = NodeExecutionRecord {
            graph_name: graph_name.to_string(),
            node_id: node_id.to_string(),
            duration_ms,
            success,
            executed_at: chrono::Utc::now(),
        };
        let key = format!(
            "node_exec:{}:{}:{}:{}",
            graph_name,
            node_id,
            execution_id,
            record.executed_at.timestamp_millis()
        );
        let value = serde_json::to_vec(&record).context("Failed to serialize node record")?;

        let txn = self.db.begin_write().map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to begin write transaction")?;
        {
            let mut table = txn.open_table(METRICS_TABLE).map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to open metrics table")?;
            table.insert(key.as_str(), value.as_slice()).map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to insert node execution record")?;
        }
        txn.commit().map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to commit transaction")?;

        Ok(())
    }

    /// Get aggregated metrics for a specific node within a graph.
    pub async fn get_node_metrics(
        &self,
        graph_name: &str,
        node_id: &str,
    ) -> Result<Option<NodeMetricsAggregate>> {
        let prefix = format!("node_exec:{}:{}:", graph_name, node_id);
        let end = prefix_end(&prefix);
        let mut total = 0u64;
        let mut successful = 0u64;
        let mut total_duration = 0u64;

        let txn = self.db.begin_read().map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to begin read transaction")?;
        let table = txn.open_table(METRICS_TABLE).map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to open metrics table")?;

        for item in table.range(prefix.as_str()..end.as_str()).map_err(|e| anyhow::anyhow!("{}", e))? {
            let (_key, value) = item.map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to read database entry")?;
            let value = value.value();
            let record: NodeExecutionRecord =
                serde_json::from_slice(value).context("Failed to deserialize node record")?;
            if record.graph_name == graph_name && record.node_id == node_id {
                total += 1;
                if record.success {
                    successful += 1;
                }
                total_duration += record.duration_ms;
            }
        }

        if total == 0 {
            return Ok(None);
        }

        Ok(Some(NodeMetricsAggregate {
            node_id: node_id.to_string(),
            total_executions: total,
            successful_executions: successful,
            avg_duration_ms: (total_duration as f64) / (total as f64),
            success_rate: (successful as f64) / (total as f64),
        }))
    }

    /// Get recent graph executions, sorted by id descending.
    pub async fn get_recent_executions(
        &self,
        graph_name: &str,
        limit: usize,
    ) -> Result<Vec<ExecutionRecord>> {
        let prefix = format!("exec:{}:", graph_name);
        let end = prefix_end(&prefix);
        let mut records: Vec<ExecutionRecord> = Vec::new();

        let txn = self.db.begin_read().map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to begin read transaction")?;
        let table = txn.open_table(METRICS_TABLE).map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to open metrics table")?;

        for item in table.range(prefix.as_str()..end.as_str()).map_err(|e| anyhow::anyhow!("{}", e))? {
            let (_key, value) = item.map_err(|e| anyhow::anyhow!("{}", e)).context("Failed to read database entry")?;
            let value = value.value();
            let record: ExecutionRecord =
                serde_json::from_slice(value).context("Failed to deserialize record")?;
            records.push(record);
        }

        if records.is_empty() {
            return Ok(vec![]);
        }

        // Sort by id descending (most recent first)
        records.sort_by(|a, b| b.id.cmp(&a.id));
        records.truncate(limit);
        Ok(records)
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_metrics_collection_ecobin() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("metrics.redb");

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

    #[tokio::test]
    async fn test_multiple_executions() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("metrics_multi.redb");

        let collector = MetricsCollector::new(&db_path).await.unwrap();

        // Record multiple executions (small delay ensures unique timestamps)
        for i in 0..5 {
            let result = GraphResult {
                success: i % 2 == 0, // Alternate success/failure
                node_results: Default::default(),
                errors: vec![],
                duration_ms: (i + 1) * 100,
            };

            collector
                .record_execution("multi_graph", &result, (i + 1) * 100)
                .await
                .unwrap();
            tokio::time::sleep(tokio::time::Duration::from_millis(2)).await;
        }

        let metrics = collector.get_graph_metrics("multi_graph").await.unwrap();
        assert!(metrics.is_some());

        let m = metrics.unwrap();
        assert_eq!(m.total_executions, 5);
        assert_eq!(m.successful_executions, 3); // 0, 2, 4 are successful
        assert_eq!(m.failed_executions, 2); // 1, 3 are failures
    }

    #[tokio::test]
    async fn test_no_metrics_for_unknown_graph() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("metrics_empty.redb");

        let collector = MetricsCollector::new(&db_path).await.unwrap();

        let metrics = collector.get_graph_metrics("nonexistent").await.unwrap();
        assert!(metrics.is_none());
    }

    #[tokio::test]
    async fn test_tracked_graphs() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("metrics_tracked.redb");

        let collector = MetricsCollector::new(&db_path).await.unwrap();

        // Record executions for multiple graphs
        for graph in &["graph_a", "graph_b", "graph_c"] {
            let result = GraphResult {
                success: true,
                node_results: Default::default(),
                errors: vec![],
                duration_ms: 100,
            };
            collector
                .record_execution(graph, &result, 100)
                .await
                .unwrap();
        }

        let graphs = collector.get_tracked_graphs().await.unwrap();
        assert_eq!(graphs.len(), 3);
        assert!(graphs.contains(&"graph_a".to_string()));
        assert!(graphs.contains(&"graph_b".to_string()));
        assert!(graphs.contains(&"graph_c".to_string()));
    }

    #[tokio::test]
    async fn test_clear_all() {
        let dir = tempdir().unwrap();
        let db_path = dir.path().join("metrics_clear.redb");

        let collector = MetricsCollector::new(&db_path).await.unwrap();

        // Add some data
        let result = GraphResult {
            success: true,
            node_results: Default::default(),
            errors: vec![],
            duration_ms: 100,
        };
        collector
            .record_execution("test", &result, 100)
            .await
            .unwrap();

        // Clear
        collector.clear_all().await.unwrap();

        // Verify cleared
        let graphs = collector.get_tracked_graphs().await.unwrap();
        assert!(graphs.is_empty());
    }

    #[test]
    fn test_graph_metrics_serialize() {
        let metrics = GraphMetrics {
            graph_name: "test".to_string(),
            total_executions: 10,
            successful_executions: 8,
            failed_executions: 2,
            avg_duration_ms: 150.5,
            min_duration_ms: 100,
            max_duration_ms: 200,
            success_rate: 0.8,
            last_executed_at: chrono::Utc::now(),
        };
        let json = serde_json::to_string(&metrics).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("10"));
        assert!(json.contains("0.8"));
    }

    #[test]
    fn test_execution_record_serialize() {
        let record = ExecutionRecord {
            id: 12345,
            graph_name: "test_graph".to_string(),
            success: true,
            duration_ms: 150,
            executed_at: chrono::Utc::now(),
            metadata: "{}".to_string(),
        };
        let json = serde_json::to_string(&record).unwrap();
        assert!(json.contains("test_graph"));
        assert!(json.contains("150"));
    }

    #[test]
    fn test_graph_result_default() {
        let result = GraphResult::default();
        assert!(!result.success);
        assert!(result.node_results.is_empty());
        assert!(result.errors.is_empty());
        assert_eq!(result.duration_ms, 0);
    }

    #[test]
    fn test_node_metrics_aggregate_serde_roundtrip() {
        let m = NodeMetricsAggregate {
            node_id: "node1".to_string(),
            total_executions: 10,
            successful_executions: 8,
            avg_duration_ms: 50.5,
            success_rate: 0.8,
        };
        let json = serde_json::to_string(&m).unwrap();
        let restored: NodeMetricsAggregate = serde_json::from_str(&json).unwrap();
        assert_eq!(m.node_id, restored.node_id);
        assert_eq!(m.success_rate, restored.success_rate);
    }
}
