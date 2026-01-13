// =============================================================================
// Metrics Collection & Storage - Neural API Learning
// =============================================================================
//
// Collects and stores graph execution metrics to enable learning and
// optimization over time.
//
// Deep Debt Principles:
// - Modern async Rust with SQLite
// - Safe database operations (no unsafe)
// - Clear error handling
// - Efficient queries for analysis
//
// =============================================================================

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::RwLock;

use crate::graph::GraphResult;

/// Metrics collector for graph executions
#[derive(Clone)]
pub struct MetricsCollector {
    db_path: String,
    conn: Arc<RwLock<Option<rusqlite::Connection>>>,
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
    /// Create a new metrics collector
    pub async fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let path_str = db_path.as_ref().to_string_lossy().to_string();

        let collector = Self {
            db_path: path_str.clone(),
            conn: Arc::new(RwLock::new(None)),
        };

        // Initialize database
        collector.init_db().await?;

        Ok(collector)
    }

    /// Initialize the database schema
    async fn init_db(&self) -> Result<()> {
        let conn =
            rusqlite::Connection::open(&self.db_path).context("Failed to open metrics database")?;

        // Create executions table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS graph_executions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                graph_name TEXT NOT NULL,
                success BOOLEAN NOT NULL,
                duration_ms INTEGER NOT NULL,
                executed_at TEXT NOT NULL,
                metadata TEXT
            )",
            [],
        )?;

        // Create node_metrics table
        conn.execute(
            "CREATE TABLE IF NOT EXISTS node_metrics (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                execution_id INTEGER NOT NULL,
                node_id TEXT NOT NULL,
                primal_id TEXT NOT NULL,
                operation TEXT NOT NULL,
                success BOOLEAN NOT NULL,
                duration_ms INTEGER NOT NULL,
                error TEXT,
                started_at TEXT NOT NULL,
                completed_at TEXT NOT NULL,
                FOREIGN KEY (execution_id) REFERENCES graph_executions(id)
            )",
            [],
        )?;

        // Create indexes for performance
        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_graph_name 
             ON graph_executions(graph_name)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_executed_at 
             ON graph_executions(executed_at DESC)",
            [],
        )?;

        conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_node_id 
             ON node_metrics(node_id)",
            [],
        )?;

        *self.conn.write().await = Some(conn);

        Ok(())
    }

    /// Store graph execution result
    pub async fn store_execution(
        &self,
        graph_name: &str,
        result: &GraphResult,
        duration: Duration,
    ) -> Result<i64> {
        let mut conn_guard = self.conn.write().await;
        let conn = conn_guard
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))?;

        let executed_at = chrono::Utc::now();
        let duration_ms = duration.as_millis() as u64;

        // Insert execution record
        conn.execute(
            "INSERT INTO graph_executions 
             (graph_name, success, duration_ms, executed_at, metadata)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                graph_name,
                result.success,
                duration_ms,
                executed_at.to_rfc3339(),
                serde_json::to_string(&result.node_results)?
            ],
        )?;

        let execution_id = conn.last_insert_rowid();

        // Insert node metrics (if available in context)
        // Note: In the new model, metrics are tracked separately
        let empty_metrics: Vec<crate::graph::NodeMetrics> = vec![];
        for metric in &empty_metrics {
            conn.execute(
                "INSERT INTO node_metrics 
                 (execution_id, node_id, primal_id, operation, success, 
                  duration_ms, error, started_at, completed_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                rusqlite::params![
                    execution_id,
                    &metric.node_id,
                    "unknown", // primal_id not in simplified NodeMetrics
                    "unknown", // operation not in simplified NodeMetrics
                    metric.success,
                    metric.duration_ms,
                    None::<String>, // error not in simplified NodeMetrics
                    "",             // started_at not in simplified NodeMetrics
                    "",             // completed_at not in simplified NodeMetrics
                ],
            )?;
        }

        Ok(execution_id)
    }

    /// Get aggregated metrics for a graph
    pub async fn get_graph_metrics(&self, graph_name: &str) -> Result<Option<GraphMetrics>> {
        let mut conn_guard = self.conn.write().await;
        let conn = conn_guard
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))?;

        let mut stmt = conn.prepare(
            "SELECT 
                COUNT(*) as total,
                SUM(CASE WHEN success = 1 THEN 1 ELSE 0 END) as successful,
                AVG(duration_ms) as avg_duration,
                MIN(duration_ms) as min_duration,
                MAX(duration_ms) as max_duration,
                MAX(executed_at) as last_executed
             FROM graph_executions
             WHERE graph_name = ?1",
        )?;

        let result = stmt.query_row([graph_name], |row| {
            let total: u64 = row.get(0)?;

            if total == 0 {
                return Ok(None);
            }

            let successful: u64 = row.get(1)?;
            let failed = total - successful;
            let avg_duration: f64 = row.get(2)?;
            let min_duration: u64 = row.get(3)?;
            let max_duration: u64 = row.get(4)?;
            let last_executed_str: String = row.get(5)?;

            let last_executed = chrono::DateTime::parse_from_rfc3339(&last_executed_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            Ok(Some(GraphMetrics {
                graph_name: graph_name.to_string(),
                total_executions: total,
                successful_executions: successful,
                failed_executions: failed,
                avg_duration_ms: avg_duration,
                min_duration_ms: min_duration,
                max_duration_ms: max_duration,
                success_rate: successful as f64 / total as f64,
                last_executed_at: last_executed,
            }))
        })?;

        Ok(result)
    }

    /// Get metrics for a specific node
    pub async fn get_node_metrics(&self, node_id: &str) -> Result<Option<NodeMetricsAggregate>> {
        let mut conn_guard = self.conn.write().await;
        let conn = conn_guard
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))?;

        let mut stmt = conn.prepare(
            "SELECT 
                COUNT(*) as total,
                SUM(CASE WHEN success = 1 THEN 1 ELSE 0 END) as successful,
                AVG(duration_ms) as avg_duration
             FROM node_metrics
             WHERE node_id = ?1",
        )?;

        let result = stmt.query_row([node_id], |row| {
            let total: u64 = row.get(0)?;

            if total == 0 {
                return Ok(None);
            }

            let successful: u64 = row.get(1)?;
            let avg_duration: f64 = row.get(2)?;

            Ok(Some(NodeMetricsAggregate {
                node_id: node_id.to_string(),
                total_executions: total,
                successful_executions: successful,
                avg_duration_ms: avg_duration,
                success_rate: successful as f64 / total as f64,
            }))
        })?;

        Ok(result)
    }

    /// Find the slowest node in a graph based on historical data
    pub async fn find_bottleneck(&self, graph_name: &str) -> Result<Option<String>> {
        let mut conn_guard = self.conn.write().await;
        let conn = conn_guard
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))?;

        let mut stmt = conn.prepare(
            "SELECT nm.node_id, AVG(nm.duration_ms) as avg_duration
             FROM node_metrics nm
             JOIN graph_executions ge ON nm.execution_id = ge.id
             WHERE ge.graph_name = ?1
             GROUP BY nm.node_id
             ORDER BY avg_duration DESC
             LIMIT 1",
        )?;

        let result = match stmt.query_row([graph_name], |row| {
            let node_id: String = row.get(0)?;
            Ok(node_id)
        }) {
            Ok(node_id) => Some(node_id),
            Err(rusqlite::Error::QueryReturnedNoRows) => None,
            Err(e) => return Err(e.into()),
        };

        Ok(result)
    }

    /// Get recent execution history
    pub async fn get_recent_executions(
        &self,
        graph_name: &str,
        limit: usize,
    ) -> Result<Vec<ExecutionRecord>> {
        let mut conn_guard = self.conn.write().await;
        let conn = conn_guard
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))?;

        let mut stmt = conn.prepare(
            "SELECT id, graph_name, success, duration_ms, executed_at, metadata
             FROM graph_executions
             WHERE graph_name = ?1
             ORDER BY executed_at DESC
             LIMIT ?2",
        )?;

        let rows = stmt.query_map(rusqlite::params![graph_name, limit as i64], |row| {
            let executed_at_str: String = row.get(4)?;
            let executed_at = chrono::DateTime::parse_from_rfc3339(&executed_at_str)
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(|_| chrono::Utc::now());

            Ok(ExecutionRecord {
                id: row.get(0)?,
                graph_name: row.get(1)?,
                success: row.get(2)?,
                duration_ms: row.get(3)?,
                executed_at,
                metadata: row.get(5)?,
            })
        })?;

        let mut records = Vec::new();
        for row in rows {
            records.push(row?);
        }

        Ok(records)
    }

    /// Clear old metrics (data retention)
    pub async fn cleanup_old_metrics(&self, older_than_days: u32) -> Result<usize> {
        let mut conn_guard = self.conn.write().await;
        let conn = conn_guard
            .as_mut()
            .ok_or_else(|| anyhow::anyhow!("Database not initialized"))?;

        let cutoff_date = chrono::Utc::now() - chrono::Duration::days(older_than_days as i64);

        let deleted = conn.execute(
            "DELETE FROM graph_executions WHERE executed_at < ?1",
            [cutoff_date.to_rfc3339()],
        )?;

        Ok(deleted)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[tokio::test]
    async fn test_metrics_storage() {
        let temp_db = NamedTempFile::new().unwrap();
        let collector = MetricsCollector::new(temp_db.path()).await.unwrap();

        // Create a sample result
        let result = GraphResult {
            success: true,
            node_results: std::collections::HashMap::new(),
            errors: vec![],
            duration_ms: 100,
        };

        // Store execution
        let id = collector
            .store_execution("test-graph", &result, Duration::from_millis(100))
            .await
            .unwrap();

        assert!(id > 0);

        // Retrieve metrics
        let metrics = collector.get_graph_metrics("test-graph").await.unwrap();
        assert!(metrics.is_some());

        let metrics = metrics.unwrap();
        assert_eq!(metrics.total_executions, 1);
        assert_eq!(metrics.successful_executions, 1);
    }
}
