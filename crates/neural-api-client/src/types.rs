// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Request/response types for Neural API Client

use bytes::Bytes;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;

/// Serde helpers for Bytes body (JSON-RPC returns body as string)
pub mod body_serde {
    use bytes::Bytes;
    use serde::{Deserialize, Deserializer, Serializer};

    pub fn deserialize<'de, D>(d: D) -> Result<Bytes, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(d)?;
        Ok(Bytes::from(s.into_bytes()))
    }

    pub fn serialize<S>(b: &Bytes, s: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let str = std::str::from_utf8(b).map_err(serde::ser::Error::custom)?;
        s.serialize_str(str)
    }
}

/// HTTP response from proxied request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpResponse {
    /// HTTP status code
    pub status: u16,
    /// Response headers
    pub headers: HashMap<String, String>,
    /// Response body
    #[serde(with = "body_serde")]
    pub body: Bytes,
}

impl HttpResponse {
    /// Get body as UTF-8 string slice
    pub fn body_str(&self) -> Result<&str, std::str::Utf8Error> {
        std::str::from_utf8(&self.body)
    }

    /// Get body as owned String
    pub fn body_string(&self) -> String {
        String::from_utf8_lossy(&self.body).into_owned()
    }
}

/// Information about discovered capability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityInfo {
    /// Capability name
    pub capability: String,
    /// Atomic type (if applicable)
    pub atomic_type: Option<String>,
    /// Primals providing this capability
    pub primals: Vec<PrimalInfo>,
    /// Primary socket to route to
    pub primary_socket: PathBuf,
}

/// Information about a discovered primal
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    /// Primal name
    pub name: String,
    /// Socket path
    pub socket: PathBuf,
    /// Health status
    pub healthy: bool,
    /// Capabilities this primal provides
    pub capabilities: Vec<String>,
}

/// Routing metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingMetrics {
    /// Total number of requests routed
    pub total_requests: usize,
    /// Individual metrics
    pub metrics: Vec<RoutingMetric>,
}

/// Individual routing metric
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingMetric {
    /// Request ID
    pub request_id: String,
    /// Capability requested
    pub capability: String,
    /// Method called
    pub method: String,
    /// Primals involved in routing
    pub routed_through: Vec<String>,
    /// Latency in milliseconds
    pub latency_ms: u64,
    /// Success status
    pub success: bool,
    /// Timestamp
    pub timestamp: String,
    /// Error message (if failed)
    pub error: Option<String>,
}

#[allow(clippy::unwrap_used, clippy::expect_used)]
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_http_response_serialize_deserialize() {
        let resp = HttpResponse {
            status: 200,
            headers: std::iter::once(("content-type".into(), "application/json".into())).collect(),
            body: Bytes::from(r#"{"result":"ok"}"#),
        };
        let json = serde_json::to_string(&resp).expect("serialize");
        let parsed: HttpResponse = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.status, 200);
        assert_eq!(parsed.body_str().unwrap(), r#"{"result":"ok"}"#);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_http_response_body_str_utf8() {
        let resp = HttpResponse {
            status: 200,
            headers: HashMap::new(),
            body: Bytes::from("hello world"),
        };
        assert_eq!(resp.body_str().unwrap(), "hello world");
        assert_eq!(resp.body_string(), "hello world");
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_http_response_body_string_lossy() {
        let invalid = vec![0xff, 0xfe, 0xfd];
        let resp = HttpResponse {
            status: 200,
            headers: HashMap::new(),
            body: Bytes::from(invalid),
        };
        assert!(resp.body_str().is_err());
        assert!(!resp.body_string().is_empty());
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_capability_info_serialize_deserialize() {
        let info = CapabilityInfo {
            capability: "security".into(),
            atomic_type: Some("beardog".into()),
            primals: vec![PrimalInfo {
                name: "beardog".into(),
                socket: PathBuf::from("/tmp/beardog.sock"),
                healthy: true,
                capabilities: vec!["crypto".into(), "identity".into()],
            }],
            primary_socket: PathBuf::from("/tmp/beardog.sock"),
        };
        let json = serde_json::to_string(&info).expect("serialize");
        let parsed: CapabilityInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.capability, "security");
        assert_eq!(parsed.primals.len(), 1);
        assert_eq!(parsed.primals[0].name, "beardog");
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_primal_info_serialize_deserialize() {
        let info = PrimalInfo {
            name: "songbird".into(),
            socket: PathBuf::from("/tmp/songbird.sock"),
            healthy: true,
            capabilities: vec!["discovery".into()],
        };
        let json = serde_json::to_string(&info).expect("serialize");
        let parsed: PrimalInfo = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.name, "songbird");
        assert!(parsed.healthy);
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_routing_metrics_serialize_deserialize() {
        let metrics = RoutingMetrics {
            total_requests: 42,
            metrics: vec![RoutingMetric {
                request_id: "req-1".into(),
                capability: "security".into(),
                method: "health.check".into(),
                routed_through: vec!["beardog".into()],
                latency_ms: 5,
                success: true,
                timestamp: "2025-01-01T00:00:00Z".into(),
                error: None,
            }],
        };
        let json = serde_json::to_string(&metrics).expect("serialize");
        let parsed: RoutingMetrics = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.total_requests, 42);
        assert_eq!(parsed.metrics.len(), 1);
        assert_eq!(parsed.metrics[0].request_id, "req-1");
    }

    #[test]
    #[allow(clippy::unwrap_used)]
    fn test_routing_metric_with_error_serialize_deserialize() {
        let metric = RoutingMetric {
            request_id: "req-2".into(),
            capability: "discovery".into(),
            method: "discover".into(),
            routed_through: vec![],
            latency_ms: 100,
            success: false,
            timestamp: "2025-01-01T00:00:00Z".into(),
            error: Some("Connection refused".into()),
        };
        let json = serde_json::to_string(&metric).expect("serialize");
        let parsed: RoutingMetric = serde_json::from_str(&json).expect("deserialize");
        assert!(!parsed.success);
        assert_eq!(parsed.error, Some("Connection refused".into()));
    }
}
