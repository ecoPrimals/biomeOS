// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Layer 1: Physical Discovery
//!
//! Node/primal discovery via Songbird or socket scanning fallback.

use crate::FederationResult;
use crate::capability::CapabilitySet;
use crate::discovery::{DiscoveredPrimal, PrimalDiscovery, PrimalEndpoint};
use crate::unix_socket_client::{JsonRpcRequest, UnixSocketClient};
use biomeos_types::constants::timeouts;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::{debug, warn};

/// Service info returned by the discovery provider (capability-agnostic).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredServiceInfo {
    pub id: String,
    pub name: String,
    pub address: String,
    pub port: u16,
    pub tags: Vec<String>,
    pub health: String,
}

/// Discovery response containing discovered services.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryResponse {
    pub services: Vec<DiscoveredServiceInfo>,
}

/// Convert discovered service address/port to `PrimalEndpoint` (testable pure function).
pub fn service_address_to_endpoint(service: &DiscoveredServiceInfo) -> PrimalEndpoint {
    if service.address.starts_with('/') {
        PrimalEndpoint::UnixSocket {
            path: PathBuf::from(&service.address),
        }
    } else {
        PrimalEndpoint::Http {
            url: format!("http://{}:{}", service.address, service.port),
        }
    }
}

/// Layer 1: Physical Discovery via Songbird
pub async fn layer1_physical_discovery_songbird(
    songbird: &UnixSocketClient,
    family_id: Option<&str>,
) -> FederationResult<Vec<DiscoveredPrimal>> {
    debug!("Layer 1: Physical Discovery (Songbird)");

    let request = JsonRpcRequest::new(
        "discover_by_family",
        serde_json::json!({
            "family_tags": [family_id.unwrap_or("*")],
            "timeout_ms": timeouts::DEFAULT_DISCOVERY_TIMEOUT_MS
        }),
    );

    match songbird.call(request).await {
        Ok(response) => {
            let result_value = response.result.unwrap_or_default();
            match serde_json::from_value::<DiscoveryResponse>(result_value) {
                Ok(discovery) => {
                    debug!("Songbird discovered {} services", discovery.services.len());

                    let primals: Vec<DiscoveredPrimal> = discovery
                        .services
                        .into_iter()
                        .map(|service| {
                            let capabilities = CapabilitySet::from_tags(&service.tags);

                            let endpoint = service_address_to_endpoint(&service);

                            DiscoveredPrimal {
                                name: service.name.clone(),
                                primal_type: service
                                    .tags
                                    .first()
                                    .cloned()
                                    .unwrap_or_else(|| "unknown".to_string()),
                                endpoints: vec![endpoint],
                                capabilities,
                                metadata: HashMap::from([
                                    ("id".to_string(), service.id),
                                    ("health".to_string(), service.health),
                                ]),
                            }
                        })
                        .collect();

                    Ok(primals)
                }
                Err(e) => {
                    warn!("Failed to parse Songbird response: {}, falling back", e);
                    layer1_physical_discovery_sockets().await
                }
            }
        }
        Err(e) => {
            warn!(
                "Songbird discovery failed: {}, falling back to socket scan",
                e
            );
            layer1_physical_discovery_sockets().await
        }
    }
}

/// Layer 1: Physical Discovery via socket scanning (fallback)
pub async fn layer1_physical_discovery_sockets() -> FederationResult<Vec<DiscoveredPrimal>> {
    debug!("Layer 1: Physical Discovery (socket scan fallback)");

    let mut basic_discovery = PrimalDiscovery::new();
    basic_discovery.discover().await
}

#[cfg(test)]
#[expect(clippy::expect_used, reason = "test")]
mod tests {
    use super::*;

    #[test]
    fn test_service_address_to_endpoint_unix() {
        let service = DiscoveredServiceInfo {
            id: "s1".to_string(),
            name: "test".to_string(),
            address: "/tmp/biomeos.sock".to_string(),
            port: 0,
            tags: vec![],
            health: "ok".to_string(),
        };
        let ep = service_address_to_endpoint(&service);
        match &ep {
            PrimalEndpoint::UnixSocket { path } => {
                assert_eq!(path.to_string_lossy(), "/tmp/biomeos.sock");
            }
            _ => panic!("Expected UnixSocket"),
        }
    }

    #[test]
    fn test_service_address_to_endpoint_http() {
        let service = DiscoveredServiceInfo {
            id: "s2".to_string(),
            name: "http-svc".to_string(),
            address: "127.0.0.1".to_string(),
            port: 9000,
            tags: vec!["discovery".to_string()],
            health: "healthy".to_string(),
        };
        let ep = service_address_to_endpoint(&service);
        match &ep {
            PrimalEndpoint::Http { url } => {
                assert_eq!(url, "http://127.0.0.1:9000");
            }
            _ => panic!("Expected Http"),
        }
    }

    #[test]
    fn test_service_address_to_endpoint_unix_relative_path() {
        let service = DiscoveredServiceInfo {
            id: "s3".to_string(),
            name: "rel".to_string(),
            address: "/var/run/sock".to_string(),
            port: 0,
            tags: vec![],
            health: "ok".to_string(),
        };
        let ep = service_address_to_endpoint(&service);
        match &ep {
            PrimalEndpoint::UnixSocket { path } => assert!(path.to_string_lossy().starts_with('/')),
            _ => panic!("Expected UnixSocket"),
        }
    }

    #[test]
    fn test_songbird_service_info_serde() {
        let json = r#"{"id":"s1","name":"songbird","address":"127.0.0.1","port":9000,"tags":["discovery"],"health":"healthy"}"#;
        let info: DiscoveredServiceInfo = serde_json::from_str(json).expect("deserialize");
        assert_eq!(info.name, "songbird");
        assert_eq!(info.port, 9000);
        assert_eq!(info.tags, vec!["discovery"]);
    }

    #[test]
    fn test_songbird_discovery_response_serde() {
        let json = r#"{"services":[{"id":"s1","name":"test","address":"/tmp/test.sock","port":0,"tags":[],"health":"ok"}]}"#;
        let resp: DiscoveryResponse = serde_json::from_str(json).expect("deserialize");
        assert_eq!(resp.services.len(), 1);
        assert_eq!(resp.services[0].name, "test");
    }

    #[test]
    fn test_service_address_to_endpoint_http_hostname() {
        let service = DiscoveredServiceInfo {
            id: "h1".to_string(),
            name: "host".to_string(),
            address: "songbird.internal".to_string(),
            port: 443,
            tags: vec!["discovery".to_string()],
            health: "ok".to_string(),
        };
        let ep = service_address_to_endpoint(&service);
        match ep {
            PrimalEndpoint::Http { url } => assert_eq!(url, "http://songbird.internal:443"),
            _ => panic!("Expected Http"),
        }
    }

    #[test]
    fn test_service_address_to_endpoint_unix_preserves_non_slash_as_http() {
        // Only paths starting with '/' become Unix sockets; hostnames stay HTTP.
        let service = DiscoveredServiceInfo {
            id: "rel".to_string(),
            name: "n".to_string(),
            address: "relative-not-unix".to_string(),
            port: 8080,
            tags: vec![],
            health: "ok".to_string(),
        };
        let ep = service_address_to_endpoint(&service);
        match ep {
            PrimalEndpoint::Http { url } => assert_eq!(url, "http://relative-not-unix:8080"),
            _ => panic!("Expected Http"),
        }
    }

    #[test]
    fn test_songbird_discovery_response_empty_services() {
        let json = r#"{"services":[]}"#;
        let resp: DiscoveryResponse = serde_json::from_str(json).expect("deserialize");
        assert!(resp.services.is_empty());
    }

    #[test]
    fn test_songbird_service_info_roundtrip() {
        let orig = DiscoveredServiceInfo {
            id: "id".to_string(),
            name: "n".to_string(),
            address: "10.0.0.1".to_string(),
            port: 1,
            tags: vec!["a".to_string(), "b".to_string()],
            health: "degraded".to_string(),
        };
        let v = serde_json::to_value(&orig).expect("serialize");
        let back: DiscoveredServiceInfo = serde_json::from_value(v).expect("deserialize");
        assert_eq!(orig.id, back.id);
        assert_eq!(orig.tags, back.tags);
    }
}
