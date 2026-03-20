// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Health check API handler
//!
//! Provides health check endpoint for monitoring and load balancer integration.

use axum::{Json, extract::State};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use crate::AppState;

/// Health check response
#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
    pub mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uptime_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timestamp: Option<String>,
}

/// GET /api/v1/health
/// Health check endpoint for monitoring and load balancers
pub async fn health(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        mode: if state.is_standalone_mode() {
            "standalone"
        } else {
            "live"
        }
        .to_string(),
        uptime_seconds: None, // Could be added with startup time tracking
        timestamp: Some(chrono::Utc::now().to_rfc3339()),
    })
}

/// GET /api/v1/health/ready
/// Readiness probe endpoint (Kubernetes-style)
pub async fn readiness(State(state): State<Arc<AppState>>) -> Json<HealthResponse> {
    // In live mode, could check if discovery service is available
    let is_ready = true; // Standalone always ready; live could add discovery/db checks

    Json(HealthResponse {
        status: if is_ready { "ready" } else { "not_ready" }.to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        mode: if state.is_standalone_mode() {
            "standalone"
        } else {
            "live"
        }
        .to_string(),
        uptime_seconds: None,
        timestamp: Some(chrono::Utc::now().to_rfc3339()),
    })
}

/// GET /api/v1/health/live
/// Liveness probe endpoint (Kubernetes-style)
pub async fn liveness(_state: State<Arc<AppState>>) -> Json<HealthResponse> {
    // Liveness is always true if the server is responding
    Json(HealthResponse {
        status: "alive".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        mode: "live".to_string(),
        uptime_seconds: None,
        timestamp: Some(chrono::Utc::now().to_rfc3339()),
    })
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use crate::AppState;
    use std::sync::Arc;

    #[test]
    fn test_health_response_serialization() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            version: "1.0.0".to_string(),
            mode: "standalone".to_string(),
            uptime_seconds: Some(3600),
            timestamp: Some("2026-02-04T12:00:00Z".to_string()),
        };

        let json = serde_json::to_string(&response).expect("should serialize");
        assert!(json.contains("healthy"));
        assert!(json.contains("1.0.0"));
        assert!(json.contains("standalone"));
        assert!(json.contains("3600"));
    }

    #[test]
    fn test_health_response_optional_fields_skip_none() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            version: "1.0.0".to_string(),
            mode: "live".to_string(),
            uptime_seconds: None,
            timestamp: None,
        };

        let json = serde_json::to_string(&response).expect("should serialize");
        assert!(json.contains("healthy"));
        assert!(!json.contains("uptime_seconds")); // Should skip None
        assert!(!json.contains("timestamp")); // Should skip None
    }

    #[test]
    fn test_health_response_deserialization() {
        let json = r#"{
            "status": "healthy",
            "version": "1.0.0",
            "mode": "standalone",
            "uptime_seconds": 3600,
            "timestamp": "2026-02-04T12:00:00Z"
        }"#;

        let response: HealthResponse = serde_json::from_str(json).expect("should deserialize");
        assert_eq!(response.status, "healthy");
        assert_eq!(response.version, "1.0.0");
        assert_eq!(response.mode, "standalone");
        assert_eq!(response.uptime_seconds, Some(3600));
    }

    #[tokio::test]
    async fn test_health_endpoint_standalone_mode() {
        let state = Arc::new(
            AppState::builder()
                .config(crate::Config {
                    standalone_mode: true,
                    ..Default::default()
                })
                .build_with_defaults()
                .expect("should build"),
        );

        let response = health(State(state)).await;
        assert_eq!(response.status, "healthy");
        assert_eq!(response.mode, "standalone");
        assert!(!response.version.is_empty());
        assert!(response.timestamp.is_some());
    }

    #[tokio::test]
    async fn test_health_endpoint_live_mode() {
        let state = Arc::new(
            AppState::builder()
                .config(crate::Config {
                    standalone_mode: false,
                    ..Default::default()
                })
                .build_with_defaults()
                .expect("should build"),
        );

        let response = health(State(state)).await;
        assert_eq!(response.status, "healthy");
        assert_eq!(response.mode, "live");
        assert!(!response.version.is_empty());
    }

    #[tokio::test]
    async fn test_readiness_endpoint_standalone_mode() {
        let state = Arc::new(
            AppState::builder()
                .config(crate::Config {
                    standalone_mode: true,
                    ..Default::default()
                })
                .build_with_defaults()
                .expect("should build"),
        );

        let response = readiness(State(state)).await;
        assert_eq!(response.status, "ready");
        assert_eq!(response.mode, "standalone");
    }

    #[tokio::test]
    async fn test_readiness_endpoint_live_mode() {
        let state = Arc::new(
            AppState::builder()
                .config(crate::Config {
                    standalone_mode: false,
                    ..Default::default()
                })
                .build_with_defaults()
                .expect("should build"),
        );

        let response = readiness(State(state)).await;
        // Should be ready in live mode (currently always returns ready)
        assert_eq!(response.status, "ready");
        assert_eq!(response.mode, "live");
    }

    #[tokio::test]
    async fn test_liveness_endpoint() {
        let state = Arc::new(
            AppState::builder()
                .build_with_defaults()
                .expect("should build"),
        );

        let response = liveness(State(state)).await;
        assert_eq!(response.status, "alive");
        assert_eq!(response.mode, "live");
        assert!(!response.version.is_empty());
    }

    #[test]
    fn test_health_response_all_statuses() {
        let statuses = vec!["healthy", "ready", "alive", "not_ready"];

        for status in statuses {
            let response = HealthResponse {
                status: status.to_string(),
                version: "1.0.0".to_string(),
                mode: "live".to_string(),
                uptime_seconds: None,
                timestamp: None,
            };

            let json = serde_json::to_string(&response).expect("should serialize");
            assert!(json.contains(status));
        }
    }

    #[test]
    fn test_health_response_version_format() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            version: "0.1.0".to_string(),
            mode: "standalone".to_string(),
            uptime_seconds: None,
            timestamp: None,
        };

        // Version should be a valid semver-like string
        assert!(response.version.contains('.'));
    }

    #[test]
    fn test_health_response_timestamp_format() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            version: "1.0.0".to_string(),
            mode: "live".to_string(),
            uptime_seconds: None,
            timestamp: Some(chrono::Utc::now().to_rfc3339()),
        };

        // Timestamp should be valid RFC3339
        assert!(response.timestamp.is_some());
        let ts = response.timestamp.unwrap();
        assert!(ts.contains('T')); // RFC3339 format
        assert!(ts.ends_with('Z') || ts.contains('+')); // UTC or timezone
    }

    #[tokio::test]
    async fn test_health_response_consistency() {
        let state = Arc::new(
            AppState::builder()
                .build_with_defaults()
                .expect("should build"),
        );

        let health_resp = health(State(state.clone())).await;
        let readiness_resp = readiness(State(state.clone())).await;
        let liveness_resp = liveness(State(state)).await;

        // All should have same version
        assert_eq!(health_resp.version, readiness_resp.version);
        assert_eq!(readiness_resp.version, liveness_resp.version);

        // All should have timestamps
        assert!(health_resp.timestamp.is_some());
        assert!(readiness_resp.timestamp.is_some());
        assert!(liveness_resp.timestamp.is_some());
    }

    #[test]
    fn test_health_response_debug() {
        let response = HealthResponse {
            status: "healthy".to_string(),
            version: "1.0.0".to_string(),
            mode: "standalone".to_string(),
            uptime_seconds: Some(3600),
            timestamp: None,
        };
        let debug_str = format!("{response:?}");
        assert!(debug_str.contains("healthy"));
        assert!(debug_str.contains("1.0.0"));
    }

    #[test]
    fn test_health_response_serde_roundtrip() {
        let response = HealthResponse {
            status: "ready".to_string(),
            version: "0.2.0".to_string(),
            mode: "live".to_string(),
            uptime_seconds: Some(7200),
            timestamp: Some("2026-03-14T12:00:00Z".to_string()),
        };
        let json = serde_json::to_string(&response).unwrap();
        let parsed: HealthResponse = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.status, response.status);
        assert_eq!(parsed.uptime_seconds, response.uptime_seconds);
    }
}
