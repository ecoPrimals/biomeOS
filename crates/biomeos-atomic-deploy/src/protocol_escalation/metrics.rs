//! Metrics and status retrieval for protocol escalation

use serde_json::{json, Value};

use crate::living_graph::LivingGraph;

use super::config::EscalationConfig;

/// Get protocol status for all connections (for JSON-RPC API)
pub async fn get_protocol_status(graph: &LivingGraph, config: &EscalationConfig) -> Value {
    let connections = graph.get_all_connections().await;
    let summary = graph.get_protocol_summary().await;

    let connection_status: Vec<Value> = connections
        .iter()
        .map(|c| {
            json!({
                "from": c.from,
                "to": c.to,
                "protocol": format!("{:?}", c.protocol),
                "requests": c.metrics.request_count,
                "avg_latency_us": c.metrics.avg_latency_us,
                "p99_latency_us": c.metrics.p99_latency_us,
                "error_rate": c.metrics.error_rate(),
                "escalation_attempts": c.escalation_attempts,
                "fallback_count": c.fallback_count,
            })
        })
        .collect();

    json!({
        "connections": connection_status,
        "summary": {
            "json_rpc": summary.json_rpc,
            "tarpc": summary.tarpc,
            "hybrid": summary.hybrid,
            "degraded": summary.degraded,
            "total": summary.total(),
        },
        "config": {
            "auto_escalate": config.auto_escalate,
            "min_requests": config.min_requests,
            "latency_threshold_us": config.latency_threshold_us,
            "check_interval_secs": config.check_interval_secs,
        }
    })
}

/// Get metrics for a specific connection (for JSON-RPC API)
pub async fn get_connection_metrics(graph: &LivingGraph, from: &str, to: &str) -> Option<Value> {
    let conn = graph.get_connection(from, to).await?;

    Some(json!({
        "connection": {
            "from": conn.from,
            "to": conn.to,
            "protocol": format!("{:?}", conn.protocol),
        },
        "metrics": {
            "request_count": conn.metrics.request_count,
            "error_count": conn.metrics.error_count,
            "total_latency_us": conn.metrics.total_latency_us,
            "avg_latency_us": conn.metrics.avg_latency_us,
            "p50_latency_us": conn.metrics.p50_latency_us,
            "p95_latency_us": conn.metrics.p95_latency_us,
            "p99_latency_us": conn.metrics.p99_latency_us,
            "max_latency_us": conn.metrics.max_latency_us,
            "error_rate": conn.metrics.error_rate(),
        },
        "history": {
            "escalation_attempts": conn.escalation_attempts,
            "fallback_count": conn.fallback_count,
        }
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_get_status() {
        let graph = LivingGraph::new("test-family");
        graph.register_connection("a", "b").await;
        graph.register_connection("b", "c").await;

        let config = EscalationConfig::default();
        let status = get_protocol_status(&graph, &config).await;

        assert_eq!(status["summary"]["total"], 2);
        assert_eq!(status["summary"]["json_rpc"], 2);
    }

    #[tokio::test]
    async fn test_get_status_empty_graph() {
        let graph = LivingGraph::new("test-family");
        let config = EscalationConfig::default();
        let status = get_protocol_status(&graph, &config).await;

        assert_eq!(status["summary"]["total"], 0);
        assert_eq!(status["summary"]["json_rpc"], 0);
        assert_eq!(status["summary"]["tarpc"], 0);
        assert!(status["connections"].is_array());
        assert!(status["config"]["auto_escalate"].as_bool().unwrap());
    }

    #[tokio::test]
    async fn test_get_status_with_config_info() {
        let graph = LivingGraph::new("test-family");
        let config = EscalationConfig {
            min_requests: 50,
            latency_threshold_us: 250,
            check_interval_secs: 5,
            ..Default::default()
        };
        let status = get_protocol_status(&graph, &config).await;

        assert_eq!(status["config"]["min_requests"], 50);
        assert_eq!(status["config"]["latency_threshold_us"], 250);
        assert_eq!(status["config"]["check_interval_secs"], 5);
    }

    #[tokio::test]
    async fn test_get_connection_metrics_existing() {
        let graph = LivingGraph::new("test-family");
        graph.register_connection("songbird", "beardog").await;

        let metrics = get_connection_metrics(&graph, "songbird", "beardog").await;

        assert!(metrics.is_some());
        let m = metrics.expect("metrics");
        assert_eq!(m["connection"]["from"], "songbird");
        assert_eq!(m["connection"]["to"], "beardog");
        assert!(m["metrics"]["request_count"].is_number());
    }

    #[tokio::test]
    async fn test_get_connection_metrics_nonexistent() {
        let graph = LivingGraph::new("test-family");

        let metrics = get_connection_metrics(&graph, "a", "b").await;
        assert!(metrics.is_none());
    }

    #[tokio::test]
    async fn test_multiple_connections_status() {
        let graph = LivingGraph::new("test-family");
        graph.register_connection("a", "b").await;
        graph.register_connection("b", "c").await;
        graph.register_connection("a", "c").await;

        let config = EscalationConfig::default();
        let status = get_protocol_status(&graph, &config).await;

        assert_eq!(status["summary"]["total"], 3);
        let connections = status["connections"].as_array().expect("array");
        assert_eq!(connections.len(), 3);
    }

    #[tokio::test]
    async fn test_status_connection_details() {
        let graph = LivingGraph::new("test-family");
        graph.register_connection("songbird", "beardog").await;

        let config = EscalationConfig::default();
        let status = get_protocol_status(&graph, &config).await;

        let connections = status["connections"].as_array().expect("array");
        assert_eq!(connections.len(), 1);

        let conn = &connections[0];
        assert_eq!(conn["from"], "songbird");
        assert_eq!(conn["to"], "beardog");
        assert_eq!(conn["protocol"], "JsonRpc");
        assert_eq!(conn["requests"], 0);
        assert_eq!(conn["escalation_attempts"], 0);
        assert_eq!(conn["fallback_count"], 0);
    }

    #[tokio::test]
    async fn test_connection_metrics_detailed_fields() {
        let graph = LivingGraph::new("test-family");
        graph.register_connection("songbird", "beardog").await;
        graph.record_request("songbird", "beardog", 100, true).await;
        graph.record_request("songbird", "beardog", 200, true).await;
        graph
            .record_request("songbird", "beardog", 300, false)
            .await;

        let metrics = get_connection_metrics(&graph, "songbird", "beardog")
            .await
            .expect("metrics should exist");

        assert_eq!(metrics["metrics"]["request_count"], 3);
        assert_eq!(metrics["metrics"]["error_count"], 1);
        assert!(metrics["metrics"]["avg_latency_us"].as_f64().unwrap() > 0.0);
        assert!(metrics["metrics"]["max_latency_us"].as_u64().unwrap() >= 300);
        assert_eq!(metrics["history"]["escalation_attempts"], 0);
        assert_eq!(metrics["history"]["fallback_count"], 0);
    }

    #[tokio::test]
    async fn test_status_after_protocol_update() {
        let graph = LivingGraph::new("test-family");
        graph.register_connection("a", "b").await;
        graph
            .update_connection_protocol("a", "b", crate::living_graph::ProtocolMode::Tarpc)
            .await;

        let config = EscalationConfig::default();
        let status = get_protocol_status(&graph, &config).await;

        assert_eq!(status["summary"]["tarpc"], 1);
        assert_eq!(status["summary"]["json_rpc"], 0);
        assert_eq!(status["summary"]["total"], 1);
    }
}
