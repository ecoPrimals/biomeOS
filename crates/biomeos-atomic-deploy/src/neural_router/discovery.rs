// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Capability-based primal discovery

use anyhow::{Context, Result, anyhow};
use std::sync::Arc;
use tracing::{debug, info, warn};

use crate::capability_domains::capability_to_provider_fallback;
use crate::nucleation::SocketNucleation;
use biomeos_core::TransportEndpoint;
use biomeos_core::atomic_client::AtomicClient;

use super::NeuralRouter;
use super::types::{AtomicType, DiscoveredAtomic, DiscoveredPrimal};

impl NeuralRouter {
    /// Discover primals by capability category
    async fn discover_by_capability_category(&self, capability: &str) -> Result<DiscoveredAtomic> {
        let category = match capability {
            "crypto_sign" | "crypto.sign" | "crypto" | "security" | "encryption" => "security",
            "discovery" => "discovery",
            "ai" | "ai.routing" | "ai.text_generation" | "ai.coordination" => "ai",
            _ => {
                return Err(anyhow!(
                    "Capability '{capability}' does not map to a known category (security, discovery, ai)"
                ));
            }
        };

        debug!(
            "   Mapping capability '{}' to category '{}'",
            capability, category
        );

        let registry = self.capability_registry.read().await;

        let mut matching_providers = Vec::new();
        for (registered_cap, providers) in registry.iter() {
            if registered_cap == category || registered_cap.starts_with(&format!("{category}.")) {
                matching_providers.extend(providers.iter().cloned());
            }
        }

        if matching_providers.is_empty() {
            return Err(anyhow!(
                "No primals found providing '{}' capability. Available capabilities: {:?}",
                category,
                registry.keys().collect::<Vec<_>>()
            ));
        }

        let primary = &matching_providers[0];
        info!(
            "   ✅ Found primal via capability category: {} → {} (provides {})",
            capability, primary.primal_name, category
        );

        let mut primals = Vec::new();
        for provider in &matching_providers {
            let healthy = Self::check_endpoint_health(&provider.endpoint).await;
            primals.push(DiscoveredPrimal {
                name: provider.primal_name.clone(),
                endpoint: provider.endpoint.clone(),
                capabilities: vec![category.to_string()],
                healthy,
                last_check: chrono::Utc::now(),
            });
        }

        Ok(DiscoveredAtomic {
            capability: Arc::from(capability),
            primals,
            atomic_type: None,
            primary_endpoint: primary.endpoint.clone(),
        })
    }

    /// Discover primal(s) by capability
    pub async fn discover_capability(&self, capability: &str) -> Result<DiscoveredAtomic> {
        info!("🔍 Discovering capability: {}", capability);

        if let Some(providers) = self.get_capability_providers(capability).await {
            if !providers.is_empty() {
                let primary = &providers[0];
                info!(
                    "   ✅ Found in registry: {} → {}",
                    capability, primary.primal_name
                );

                let mut primals = Vec::new();
                for provider in &providers {
                    let healthy = Self::check_endpoint_health(&provider.endpoint).await;
                    primals.push(DiscoveredPrimal {
                        name: provider.primal_name.clone(),
                        endpoint: provider.endpoint.clone(),
                        capabilities: vec![capability.to_string()],
                        healthy,
                        last_check: chrono::Utc::now(),
                    });
                }

                return Ok(DiscoveredAtomic {
                    capability: Arc::from(capability),
                    primals,
                    atomic_type: None,
                    primary_endpoint: primary.endpoint.clone(),
                });
            }
        }

        warn!("   ⚠️  Capability not in registry, trying capability category discovery");
        match capability {
            "secure_http" | "http.request" | "http.post" | "http.get" => {
                self.discover_tower_atomic().await
            }
            "secure_storage" => self.discover_nest_atomic().await,
            "secure_compute" => self.discover_node_atomic().await,
            "crypto_sign" | "crypto.sign" | "crypto" | "security" | "encryption" => {
                self.discover_by_capability_category(capability).await
            }
            "discovery" => self.discover_by_capability_category(capability).await,
            "ai" | "ai.routing" | "ai.text_generation" | "ai.coordination" => {
                self.discover_by_capability_category(capability).await
            }
            _ => Err(anyhow!(
                "Capability '{}' not registered. Available: {:?}",
                capability,
                self.capability_registry
                    .read()
                    .await
                    .keys()
                    .collect::<Vec<_>>()
            )),
        }
    }

    /// Discover Tower Atomic (security + discovery capabilities)
    async fn discover_tower_atomic(&self) -> Result<DiscoveredAtomic> {
        debug!("   Discovering Tower Atomic (security + discovery capabilities)");

        let security_primal = self
            .find_primal_by_capability("security")
            .await
            .context("Tower Atomic requires a primal with 'security' capability")?;

        let discovery_primal = self
            .find_primal_by_capability("discovery")
            .await
            .context("Tower Atomic requires a primal with 'discovery' capability")?;

        if !security_primal.healthy || !discovery_primal.healthy {
            warn!(
                "   ⚠️  Tower Atomic unhealthy: security={}, discovery={}",
                security_primal.healthy, discovery_primal.healthy
            );
        }

        info!(
            "   ✅ Tower Atomic discovered: {} (security) + {} (discovery)",
            security_primal.name, discovery_primal.name
        );

        let primary = discovery_primal.endpoint.clone();
        Ok(DiscoveredAtomic {
            capability: Arc::from("secure_http"),
            primals: vec![security_primal, discovery_primal],
            atomic_type: Some(AtomicType::Tower),
            primary_endpoint: primary,
        })
    }

    /// Discover Nest Atomic (Tower + storage capability)
    async fn discover_nest_atomic(&self) -> Result<DiscoveredAtomic> {
        debug!("   Discovering Nest Atomic (Tower + storage capability)");

        let tower = self.discover_tower_atomic().await?;

        let storage_primal = self
            .find_primal_by_capability("storage")
            .await
            .context("Nest Atomic requires a primal with 'storage' capability")?;

        let primary = storage_primal.endpoint.clone();
        let mut primals = tower.primals;
        primals.push(storage_primal);

        info!(
            "   ✅ Nest Atomic discovered: Tower + {} (storage)",
            primals.last().map_or("?", |p| p.name.as_ref())
        );

        Ok(DiscoveredAtomic {
            capability: Arc::from("secure_storage"),
            primals,
            atomic_type: Some(AtomicType::Nest),
            primary_endpoint: primary,
        })
    }

    /// Discover Node Atomic (Tower + compute capability)
    async fn discover_node_atomic(&self) -> Result<DiscoveredAtomic> {
        debug!("   Discovering Node Atomic (Tower + compute capability)");

        let tower = self.discover_tower_atomic().await?;

        let compute_primal = self
            .find_primal_by_capability("compute")
            .await
            .context("Node Atomic requires a primal with 'compute' capability")?;

        let primary = compute_primal.endpoint.clone();
        let mut primals = tower.primals;
        primals.push(compute_primal);

        info!(
            "   ✅ Node Atomic discovered: Tower + {} (compute)",
            primals.last().map_or("?", |p| p.name.as_ref())
        );

        Ok(DiscoveredAtomic {
            capability: Arc::from("secure_compute"),
            primals,
            atomic_type: Some(AtomicType::Node),
            primary_endpoint: primary,
        })
    }

    /// Find primal by capability
    async fn find_primal_by_capability(&self, capability: &str) -> Result<DiscoveredPrimal> {
        let registry = self.capability_registry.read().await;

        if let Some(providers) = registry.get(capability) {
            if let Some(provider) = providers.first() {
                debug!(
                    "   📖 Registry hit: {} provides '{}'",
                    provider.primal_name, capability
                );

                let healthy = self.quick_health_check(&provider.endpoint).await;

                return Ok(DiscoveredPrimal {
                    name: provider.primal_name.clone(),
                    endpoint: provider.endpoint.clone(),
                    capabilities: vec![capability.to_string()],
                    healthy,
                    last_check: chrono::Utc::now(),
                });
            }
        }

        let fallback_primal = capability_to_provider_fallback(capability);

        if let Some(primal) = fallback_primal {
            debug!(
                "   ⚠️  Registry miss: using fallback mapping {} → {}",
                capability, primal
            );
            self.find_primal_by_socket(primal).await
        } else {
            Err(anyhow!(
                "No primal found for capability '{capability}'. Register a provider or check the capability name."
            ))
        }
    }

    /// Find primal by socket pattern (runtime discovery)
    pub(crate) async fn find_primal_by_socket(
        &self,
        primal_name: &str,
    ) -> Result<DiscoveredPrimal> {
        {
            let cache = self.discovered_primals.read().await;
            if let Some(primal) = cache.get(primal_name) {
                debug!("   📦 Cache hit: {}", primal_name);
                return Ok(primal.clone());
            }
        }

        let mut nucleation = SocketNucleation::default();
        let socket_path = nucleation.assign_socket(primal_name, &self.family_id);

        if !socket_path.exists() {
            return Err(anyhow!(
                "Primal '{}' not found: socket {} does not exist",
                primal_name,
                socket_path.display()
            ));
        }

        let endpoint = TransportEndpoint::UnixSocket {
            path: socket_path.clone(),
        };
        let healthy = self.quick_health_check(&endpoint).await;

        let primal = DiscoveredPrimal {
            name: Arc::from(primal_name),
            endpoint: endpoint.clone(),
            capabilities: vec![],
            healthy,
            last_check: chrono::Utc::now(),
        };

        {
            let mut cache = self.discovered_primals.write().await;
            cache.insert(primal_name.to_string(), primal.clone());
        }

        debug!(
            "   ✅ Discovered: {} @ {} (healthy: {})",
            primal_name,
            endpoint.display_string(),
            healthy
        );

        Ok(primal)
    }

    /// Transport-aware health check via `AtomicClient`
    ///
    /// Works for all transport tiers: Unix, abstract, TCP, HTTP.
    /// No filesystem assumption — abstract sockets and TCP endpoints are probed
    /// via an actual JSON-RPC `health.check` call.
    async fn quick_health_check(&self, endpoint: &TransportEndpoint) -> bool {
        let health_timeout = std::time::Duration::from_millis(500);

        let client = AtomicClient::from_endpoint(endpoint.clone()).with_timeout(health_timeout);

        match client.call("health.check", serde_json::json!({})).await {
            Ok(response) => response
                .get("healthy")
                .and_then(|h| h.as_bool())
                .unwrap_or(true),
            Err(_) => {
                debug!(
                    "   ⚠️ Health check failed for {}",
                    endpoint.display_string()
                );
                false
            }
        }
    }

    /// Transport-aware health check (static, for use without `&self`)
    ///
    /// Uses `AtomicClient::from_endpoint` which handles all transports
    /// correctly. No `Path::exists()` guard — abstract sockets and TCP
    /// endpoints are probed via connection attempt + JSON-RPC call.
    async fn check_endpoint_health(endpoint: &TransportEndpoint) -> bool {
        use tokio::time::{Duration, timeout};

        let probe = async {
            let client =
                AtomicClient::from_endpoint(endpoint.clone()).with_timeout(Duration::from_secs(2));

            let response = client.call("health.check", serde_json::json!({})).await?;
            Ok::<bool, anyhow::Error>(
                response
                    .get("healthy")
                    .and_then(|h| h.as_bool())
                    .unwrap_or(false),
            )
        };

        match timeout(Duration::from_secs(3), probe).await {
            Ok(Ok(healthy)) => healthy,
            _ => false,
        }
    }
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
    use super::super::AtomicType;
    use super::*;
    use std::path::PathBuf;

    fn unix_endpoint(path: &str) -> TransportEndpoint {
        TransportEndpoint::UnixSocket {
            path: PathBuf::from(path),
        }
    }

    #[tokio::test]
    async fn test_discover_capability_unregistered() {
        let router = NeuralRouter::new("test-family");
        let result = router
            .discover_capability("nonexistent_capability_xyz")
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("not registered")
                || err.contains("Capability")
                || err.contains("not found"),
            "expected capability error, got: {err}"
        );
    }

    #[tokio::test]
    async fn test_discover_capability_registered() {
        let router = NeuralRouter::new("test-family");
        router
            .register_capability(
                "security",
                "beardog",
                unix_endpoint("/tmp/beardog-test.sock"),
                "test",
            )
            .await
            .expect("register");

        let result = router.discover_capability("security").await;
        assert!(result.is_ok());
        let atomic = result.unwrap();
        assert_eq!(atomic.capability.as_ref(), "security");
        assert_eq!(atomic.primals.len(), 1);
        assert_eq!(atomic.primals[0].name.as_ref(), "beardog");
    }

    #[tokio::test]
    async fn test_find_primal_by_socket_nonexistent() {
        use biomeos_test_utils::{remove_test_env, set_test_env};

        set_test_env("XDG_RUNTIME_DIR", "/nonexistent/path/for/tests");
        let router = NeuralRouter::new("test-family-xyz");
        let result = router.find_primal_by_socket("beardog").await;
        remove_test_env("XDG_RUNTIME_DIR");

        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("not found") || err.contains("does not exist"),
            "expected socket not found, got: {err}"
        );
    }

    #[tokio::test]
    async fn test_discover_by_category_empty_registry_security() {
        let router = NeuralRouter::new("empty-reg");
        let err = router.discover_capability("security").await.unwrap_err();
        assert!(
            err.to_string().contains("No primals") || err.to_string().contains("not registered"),
            "got: {err}"
        );
    }

    #[tokio::test]
    async fn test_discover_capability_unknown_category_string() {
        let router = NeuralRouter::new("x");
        let err = router
            .discover_capability("totally_unknown_capability_xyz")
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("not registered") || err.to_string().contains("does not map"),
            "got: {err}"
        );
    }

    #[tokio::test]
    async fn test_discover_capability_http_alias_requires_registry() {
        let router = NeuralRouter::new("http-test");
        let err = router.discover_capability("http.get").await.unwrap_err();
        assert!(!err.to_string().is_empty());
    }

    #[tokio::test]
    async fn test_discover_capability_ai_category_empty_registry() {
        let router = NeuralRouter::new("ai-test");
        let err = router
            .discover_capability("ai.text_generation")
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("No primals") || err.to_string().contains("not registered")
        );
    }

    #[tokio::test]
    async fn test_discover_registered_sets_primary_endpoint() {
        let router = NeuralRouter::new("ps");
        let ep = unix_endpoint("/tmp/neural-discovery-unit.sock");
        router
            .register_capability("storage", "nest", ep.clone(), "t")
            .await
            .unwrap();
        let atomic = router.discover_capability("storage").await.unwrap();
        assert_eq!(atomic.primary_endpoint, ep);
        assert_eq!(atomic.primals.len(), 1);
    }

    #[tokio::test]
    async fn test_discover_tower_atomic_via_secure_http_alias() {
        let router = NeuralRouter::new("tower-fam");
        router
            .register_capability(
                "security",
                "beardog",
                unix_endpoint("/tmp/tower-security.sock"),
                "t",
            )
            .await
            .unwrap();
        router
            .register_capability(
                "discovery",
                "songbird",
                unix_endpoint("/tmp/tower-discovery.sock"),
                "t",
            )
            .await
            .unwrap();
        let atomic = router.discover_capability("http.get").await.expect("tower");
        assert_eq!(atomic.capability.as_ref(), "secure_http");
        assert_eq!(atomic.primals.len(), 2);
        assert!(matches!(atomic.atomic_type, Some(AtomicType::Tower)));
    }

    #[tokio::test]
    async fn test_discover_nest_atomic_requires_storage() {
        let router = NeuralRouter::new("nest-fam");
        router
            .register_capability("security", "bd", unix_endpoint("/tmp/nest-bd.sock"), "t")
            .await
            .unwrap();
        router
            .register_capability("discovery", "sb", unix_endpoint("/tmp/nest-sb.sock"), "t")
            .await
            .unwrap();
        let err = router
            .discover_capability("secure_storage")
            .await
            .unwrap_err();
        assert!(
            err.to_string().contains("storage") || err.to_string().contains("not found"),
            "{}",
            err
        );
    }

    #[tokio::test]
    async fn test_discover_capability_category_discovery_unknown_maps_error() {
        let router = NeuralRouter::new("cat-reg");
        router
            .register_capability(
                "discovery.meta",
                "songbird",
                unix_endpoint("/tmp/discovery-meta.sock"),
                "t",
            )
            .await
            .unwrap();
        let atomic = router
            .discover_capability("discovery")
            .await
            .expect("discovery");
        assert!(
            atomic.primals.iter().any(|p| p.name.as_ref() == "songbird"),
            "{atomic:?}"
        );
    }

    #[tokio::test]
    async fn test_discover_capability_http_post_alias() {
        let router = NeuralRouter::new("http-post");
        let err = router.discover_capability("http.post").await.unwrap_err();
        assert!(!err.to_string().is_empty());
    }

    #[tokio::test]
    async fn test_register_capability_tcp_endpoint() {
        let router = NeuralRouter::new("tcp-test");
        let ep = TransportEndpoint::TcpSocket {
            host: Arc::from("192.168.1.100"),
            port: 9001,
        };
        router
            .register_capability("crypto.sign", "beardog", ep.clone(), "cross-gate")
            .await
            .unwrap();
        let providers = router
            .get_capability_providers("crypto.sign")
            .await
            .unwrap();
        assert_eq!(providers.len(), 1);
        assert_eq!(providers[0].endpoint, ep);
        assert_eq!(providers[0].primal_name.as_ref(), "beardog");
    }

    #[tokio::test]
    async fn test_register_capability_abstract_socket() {
        let router = NeuralRouter::new("abstract-test");
        let ep = TransportEndpoint::AbstractSocket {
            name: Arc::from("biomeos_squirrel_abc123"),
        };
        router
            .register_capability("storage.put", "squirrel", ep.clone(), "primal_announcement")
            .await
            .unwrap();
        let providers = router
            .get_capability_providers("storage.put")
            .await
            .unwrap();
        assert_eq!(providers[0].endpoint, ep);
    }

    #[tokio::test]
    async fn test_register_capability_http_endpoint() {
        let router = NeuralRouter::new("http-ep-test");
        let ep = TransportEndpoint::HttpJsonRpc {
            host: Arc::from("songbird.local"),
            port: 8080,
        };
        router
            .register_capability("discovery.mesh", "songbird", ep.clone(), "beacon")
            .await
            .unwrap();
        let providers = router
            .get_capability_providers("discovery.mesh")
            .await
            .unwrap();
        assert_eq!(providers[0].endpoint, ep);
    }
}
