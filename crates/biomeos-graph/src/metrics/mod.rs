// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

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

/// Result of a graph execution (used by `record_execution`).
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

/// Parameters for recording a node-level execution.
#[derive(Debug, Clone)]
pub struct NodeExecutionParams<'a> {
    /// Unique execution ID (timestamp-based) for this graph run.
    pub execution_id: i64,
    /// Name of the graph being executed.
    pub graph_name: &'a str,
    /// ID of the node within the graph.
    pub node_id: &'a str,
    /// Primal ID (reserved for future use).
    pub primal_id: &'a str,
    /// Operation name (reserved for future use).
    pub operation: &'a str,
    /// Whether the node execution succeeded.
    pub success: bool,
    /// Execution duration in milliseconds.
    pub duration_ms: u64,
    /// Error message if execution failed (reserved for future use).
    pub error: Option<&'a str>,
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
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let path = db_path.as_ref();
        let db = Database::create(path)
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to open metrics database")?;

        // Ensure the metrics table exists (redb creates tables on first write)
        let txn = db
            .begin_write()
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to initialize metrics database")?;
        {
            let _ = txn
                .open_table(METRICS_TABLE)
                .map_err(|e| anyhow::anyhow!("{e}"))
                .context("Failed to create metrics table")?;
        }
        txn.commit()
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to commit initialization")?;

        Ok(Self { db: Arc::new(db) })
    }

    /// Record a graph execution (redb storage - ecoBin!)
    ///
    /// When `execution_id` is `Some`, that value is used for the record id (for deterministic tests).
    /// Otherwise uses `chrono::Utc::now().timestamp_millis()`.
    pub fn record_execution(
        &self,
        graph_name: &str,
        result: &GraphResult,
        duration_ms: u64,
        execution_id: Option<i64>,
    ) -> Result<()> {
        let id = execution_id.unwrap_or_else(|| chrono::Utc::now().timestamp_millis());
        let record = ExecutionRecord {
            id,
            graph_name: graph_name.to_string(),
            success: result.success,
            duration_ms,
            executed_at: chrono::Utc::now(),
            metadata: serde_json::to_string(&result.node_results).unwrap_or_else(|e| {
                tracing::warn!("JSON parse fallback: {}", e);
                String::default()
            }),
        };

        let key = format!("exec:{}:{}", graph_name, record.id);
        let value = serde_json::to_vec(&record).context("Failed to serialize record")?;

        let txn = self
            .db
            .begin_write()
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to begin write transaction")?;
        {
            let mut table = txn
                .open_table(METRICS_TABLE)
                .map_err(|e| anyhow::anyhow!("{e}"))
                .context("Failed to open metrics table")?;
            table
                .insert(key.as_str(), value.as_slice())
                .map_err(|e| anyhow::anyhow!("{e}"))
                .context("Failed to insert execution record")?;
        }
        txn.commit()
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to commit transaction")?;

        Ok(())
    }

    /// Get aggregated metrics for a graph (redb queries!)
    pub fn get_graph_metrics(&self, graph_name: &str) -> Result<Option<GraphMetrics>> {
        let prefix = format!("exec:{graph_name}:");
        let end = prefix_end(&prefix);

        let txn = self
            .db
            .begin_read()
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to begin read transaction")?;
        let table = txn
            .open_table(METRICS_TABLE)
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to open metrics table")?;

        let mut total = 0u64;
        let mut successful = 0u64;
        let mut total_duration = 0u64;
        let mut min_duration = u64::MAX;
        let mut max_duration_ms = 0u64;
        let mut last_executed: Option<chrono::DateTime<chrono::Utc>> = None;

        for item in table
            .range(prefix.as_str()..end.as_str())
            .map_err(|e| anyhow::anyhow!("{e}"))?
        {
            let (_key, value) = item
                .map_err(|e| anyhow::anyhow!("{e}"))
                .context("Failed to read database entry")?;
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
    pub fn get_tracked_graphs(&self) -> Result<Vec<String>> {
        let mut graphs = std::collections::HashSet::new();
        let end = prefix_end("exec:");

        let txn = self
            .db
            .begin_read()
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to begin read transaction")?;
        let table = txn
            .open_table(METRICS_TABLE)
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to open metrics table")?;

        for item in table
            .range("exec:"..end.as_str())
            .map_err(|e| anyhow::anyhow!("{e}"))?
        {
            let (key, _) = item
                .map_err(|e| anyhow::anyhow!("{e}"))
                .context("Failed to read database entry")?;
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
    pub fn clear_all(&self) -> Result<()> {
        let txn = self
            .db
            .begin_read()
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to begin read transaction")?;
        let table = txn
            .open_table(METRICS_TABLE)
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to open metrics table")?;

        let keys: Vec<String> = table
            .range::<&str>(""..)
            .map_err(|e| anyhow::anyhow!("{e}"))?
            .filter_map(|item| item.map(|(k, _)| k.value().to_string()).ok())
            .collect();

        drop(table);
        drop(txn);

        let write_txn = self
            .db
            .begin_write()
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to begin write transaction")?;
        {
            let mut table = write_txn
                .open_table(METRICS_TABLE)
                .map_err(|e| anyhow::anyhow!("{e}"))
                .context("Failed to open metrics table")?;
            for key in keys {
                table
                    .remove(key.as_str())
                    .map_err(|e| anyhow::anyhow!("{e}"))?;
            }
        }
        write_txn
            .commit()
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to commit transaction")?;

        Ok(())
    }
}

impl MetricsCollector {
    /// Record a node-level execution for metrics aggregation.
    /// Call with the same `graph_name` used in `record_execution` for this run.
    pub fn record_node_execution(&self, params: NodeExecutionParams<'_>) -> Result<()> {
        let record = NodeExecutionRecord {
            graph_name: params.graph_name.to_string(),
            node_id: params.node_id.to_string(),
            duration_ms: params.duration_ms,
            success: params.success,
            executed_at: chrono::Utc::now(),
        };
        let key = format!(
            "node_exec:{}:{}:{}:{}",
            params.graph_name,
            params.node_id,
            params.execution_id,
            record.executed_at.timestamp_millis()
        );
        let value = serde_json::to_vec(&record).context("Failed to serialize node record")?;

        let txn = self
            .db
            .begin_write()
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to begin write transaction")?;
        {
            let mut table = txn
                .open_table(METRICS_TABLE)
                .map_err(|e| anyhow::anyhow!("{e}"))
                .context("Failed to open metrics table")?;
            table
                .insert(key.as_str(), value.as_slice())
                .map_err(|e| anyhow::anyhow!("{e}"))
                .context("Failed to insert node execution record")?;
        }
        txn.commit()
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to commit transaction")?;

        Ok(())
    }

    /// Get aggregated metrics for a specific node within a graph.
    pub fn get_node_metrics(
        &self,
        graph_name: &str,
        node_id: &str,
    ) -> Result<Option<NodeMetricsAggregate>> {
        let prefix = format!("node_exec:{graph_name}:{node_id}:");
        let end = prefix_end(&prefix);
        let mut total = 0u64;
        let mut successful = 0u64;
        let mut total_duration = 0u64;

        let txn = self
            .db
            .begin_read()
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to begin read transaction")?;
        let table = txn
            .open_table(METRICS_TABLE)
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to open metrics table")?;

        for item in table
            .range(prefix.as_str()..end.as_str())
            .map_err(|e| anyhow::anyhow!("{e}"))?
        {
            let (_key, value) = item
                .map_err(|e| anyhow::anyhow!("{e}"))
                .context("Failed to read database entry")?;
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
    pub fn get_recent_executions(
        &self,
        graph_name: &str,
        limit: usize,
    ) -> Result<Vec<ExecutionRecord>> {
        let prefix = format!("exec:{graph_name}:");
        let end = prefix_end(&prefix);
        let mut records: Vec<ExecutionRecord> = Vec::new();

        let txn = self
            .db
            .begin_read()
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to begin read transaction")?;
        let table = txn
            .open_table(METRICS_TABLE)
            .map_err(|e| anyhow::anyhow!("{e}"))
            .context("Failed to open metrics table")?;

        for item in table
            .range(prefix.as_str()..end.as_str())
            .map_err(|e| anyhow::anyhow!("{e}"))?
        {
            let (_key, value) = item
                .map_err(|e| anyhow::anyhow!("{e}"))
                .context("Failed to read database entry")?;
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
mod tests;
