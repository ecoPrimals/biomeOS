// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Capability registry management for [`NeuralRouter`].
//!
//! Registration, unregistration, stale pruning, post-spawn probing,
//! lazy socket rescan, and cache invalidation. Extracted from `mod.rs`
//! to keep the parent module focused on struct definition, constructors,
//! and routing weight / composition API.

use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::Ordering;

use tracing::{debug, info, warn};

use super::NeuralRouter;
use super::probe_primal_capabilities_standalone;
use super::types::RegisteredCapability;
use super::weights;
use biomeos_core::TransportEndpoint;

impl NeuralRouter {
    /// Set the Neural API's own socket path so auto-discovery excludes it.
    pub async fn set_self_socket_path(&self, path: PathBuf) {
        *self.self_socket_path.write().await = Some(path);
    }

    /// Register a capability with a transport endpoint.
    ///
    /// Automatically seeds the routing weight table with a topology affinity
    /// derived from the endpoint's transport type (IPC vs TCP vs HTTP).
    pub async fn register_capability(
        &self,
        capability: impl Into<String>,
        primal_name: impl Into<String>,
        endpoint: TransportEndpoint,
        source: impl Into<String>,
    ) -> anyhow::Result<()> {
        let capability = capability.into();
        let primal_name = primal_name.into();

        info!(
            "✅ Registered capability: {} → {} @ {}",
            capability,
            primal_name,
            endpoint.display_string()
        );

        let topo_affinity = weights::topology_affinity_for_endpoint(&endpoint);

        let registration = RegisteredCapability {
            capability: Arc::from(capability.as_str()),
            primal_name: Arc::from(primal_name.as_str()),
            endpoint,
            registered_at: chrono::Utc::now(),
            source: Arc::from(source.into().as_str()),
        };

        let mut registry = self.capability_registry.write().await;
        let providers = registry.entry(capability.to_string()).or_default();

        if let Some(existing) = providers
            .iter_mut()
            .find(|r| r.primal_name == registration.primal_name)
        {
            debug!(
                "   Updating {} endpoint for {} (was {}, now {})",
                existing.primal_name,
                existing.capability,
                existing.endpoint.display_string(),
                registration.endpoint.display_string(),
            );
            existing.endpoint = registration.endpoint;
            existing.registered_at = registration.registered_at;
            existing.source = registration.source;
        } else {
            providers.push(registration);
        }

        drop(registry);

        let mut weights = self.routing_weights.write().await;
        weights.set_topology_affinity(&capability, &primal_name, topo_affinity);

        Ok(())
    }

    /// Convenience: register a capability bound to a Unix socket path
    pub async fn register_capability_unix(
        &self,
        capability: impl Into<String>,
        primal_name: impl Into<String>,
        socket_path: impl Into<PathBuf>,
        source: impl Into<String>,
    ) -> anyhow::Result<()> {
        let endpoint = TransportEndpoint::UnixSocket {
            path: socket_path.into(),
        };
        self.register_capability(capability, primal_name, endpoint, source)
            .await
    }

    /// List all registered capabilities
    pub async fn list_capabilities(
        &self,
    ) -> std::collections::HashMap<String, Vec<RegisteredCapability>> {
        self.capability_registry.read().await.clone()
    }

    /// Get providers for a specific capability
    pub async fn get_capability_providers(
        &self,
        capability: &str,
    ) -> Option<Vec<RegisteredCapability>> {
        self.capability_registry
            .read()
            .await
            .get(capability)
            .cloned()
    }

    /// Unregister all capabilities for a specific primal.
    ///
    /// Removes the primal from every capability entry in the registry.
    /// Empty capability entries are pruned. Returns the number of
    /// capability registrations removed.
    pub async fn unregister_primal(&self, primal_name: &str) -> usize {
        let mut registry = self.capability_registry.write().await;
        let mut removed = 0;

        for providers in registry.values_mut() {
            let before = providers.len();
            providers.retain(|r| r.primal_name.as_ref() != primal_name);
            removed += before - providers.len();
        }

        registry.retain(|_, providers| !providers.is_empty());

        if removed > 0 {
            info!(
                "🧹 Unregistered primal {primal_name}: removed {removed} capability registration(s)"
            );
        }
        removed
    }

    /// Probe all registered endpoints and remove registrations whose
    /// endpoints are unreachable.
    ///
    /// Returns `(probed, pruned)` — the total endpoints checked and how
    /// many were removed.
    pub async fn prune_stale_registrations(&self) -> (usize, usize) {
        use std::collections::HashSet;

        let registry = self.capability_registry.read().await;

        let mut endpoints: HashSet<(Arc<str>, String)> = HashSet::new();
        for providers in registry.values() {
            for reg in providers {
                endpoints.insert((reg.primal_name.clone(), reg.endpoint.display_string()));
            }
        }
        drop(registry);

        let probed = endpoints.len();
        let mut dead_primals: HashSet<Arc<str>> = HashSet::new();

        for (primal_name, _endpoint_str) in &endpoints {
            let registry = self.capability_registry.read().await;
            let endpoint = registry
                .values()
                .flat_map(|v| v.iter())
                .find(|r| &r.primal_name == primal_name)
                .map(|r| r.endpoint.clone());
            drop(registry);

            if let Some(ep) = endpoint {
                if !Self::check_endpoint_health(&ep).await {
                    dead_primals.insert(primal_name.clone());
                }
            }
        }

        let mut pruned = 0;
        for primal in &dead_primals {
            pruned += self.unregister_primal(primal).await;
        }

        if pruned > 0 {
            info!(
                "🧹 Stale prune sweep: probed {probed} endpoints, pruned {pruned} registrations ({} dead primals)",
                dead_primals.len()
            );
        } else {
            debug!("🧹 Stale prune sweep: probed {probed} endpoints, all healthy");
        }

        (probed, pruned)
    }

    /// Probe a newly-spawned primal and register its capabilities.
    ///
    /// Call after `wait_for_socket` / `wait_for_tcp_port` succeeds so the
    /// primal is guaranteed to be listening. This bridges the gap where
    /// primals spawned after boot discovery (or between lazy rescans) would
    /// otherwise remain invisible to `capability.call` routing.
    pub async fn register_spawned_primal(
        &self,
        primal_name: &str,
        socket_path: Option<&std::path::Path>,
        tcp_port: Option<u16>,
    ) -> usize {
        use biomeos_core::socket_discovery::cap_probe::probe_unix_socket_capabilities_list;

        let capabilities: Vec<String> = if let Some(port) = tcp_port {
            let addr = format!("{}:{}", biomeos_types::constants::DEFAULT_LOCALHOST, port);
            crate::neural_api_server::discovery_init::probe_tcp_capabilities_public(&addr).await
        } else if let Some(path) = socket_path {
            probe_unix_socket_capabilities_list(path).await
        } else {
            return 0;
        };

        if capabilities.is_empty() {
            debug!(
                "Post-spawn probe for {} returned no capabilities",
                primal_name
            );
            return 0;
        }

        let mut registered = 0;
        for cap in &capabilities {
            let result = if let Some(port) = tcp_port {
                let endpoint = biomeos_core::TransportEndpoint::TcpSocket {
                    host: Arc::from(biomeos_types::constants::DEFAULT_LOCALHOST),
                    port,
                };
                self.register_capability(cap, primal_name, endpoint, "post-spawn")
                    .await
            } else if let Some(path) = socket_path {
                self.register_capability_unix(cap, primal_name, path, "post-spawn")
                    .await
            } else {
                continue;
            };

            if let Err(e) = result {
                warn!(
                    "Failed to register {}.{} post-spawn: {}",
                    primal_name, cap, e
                );
            } else {
                registered += 1;
            }
        }

        if registered > 0 {
            info!(
                "✅ Post-spawn registered {} capabilities for {}",
                registered, primal_name
            );
        }
        registered
    }

    /// Invalidate discovery cache (force rediscovery)
    pub async fn invalidate_cache(&self) {
        self.discovered_primals.write().await.clear();
        self.lazy_rescan_attempted.store(false, Ordering::Relaxed);
        info!("🔄 Discovery cache invalidated");
    }

    /// Rescan socket directories for newly-appeared primals.
    ///
    /// Called lazily on the first `capability.call` miss (BM-04 fix). Only runs
    /// once per session — subsequent misses fast-fail. Reset via
    /// `invalidate_cache()` or `topology.rescan`.
    pub(crate) async fn lazy_rescan_sockets(&self) -> usize {
        if self.lazy_rescan_attempted.swap(true, Ordering::Relaxed) {
            return 0;
        }

        info!("🔄 Lazy rescan: capability miss triggered socket re-discovery");
        let socket_dirs = crate::handlers::TopologyHandler::get_socket_directories();
        let self_socket = self.self_socket_path.read().await.clone();
        let mut registered = 0usize;

        for socket_dir in &socket_dirs {
            let entries = match std::fs::read_dir(socket_dir) {
                Ok(e) => e,
                Err(_) => continue,
            };

            for entry in entries.flatten() {
                let path = entry.path();
                let filename = match path.file_name().and_then(|n| n.to_str()) {
                    Some(f) => f.to_string(),
                    None => continue,
                };

                if !filename.ends_with(".sock") {
                    continue;
                }

                if self_socket.as_ref().is_some_and(|s| *s == path) {
                    continue;
                }

                let primal_name = match filename.strip_suffix(".sock") {
                    Some(base) => base.split('-').next().unwrap_or(base).to_string(),
                    None => continue,
                };

                let socket_str = path.to_string_lossy().to_string();
                let capabilities =
                    probe_primal_capabilities_standalone(socket_str.as_str()).await;

                if capabilities.is_empty() {
                    debug!("   {} — no capabilities during lazy rescan", primal_name);
                    continue;
                }

                for cap in &capabilities {
                    if let Err(e) = self
                        .register_capability_unix(cap, &primal_name, &path, "lazy-rescan")
                        .await
                    {
                        warn!("   Failed to register {}.{}: {}", primal_name, cap, e);
                    }
                }

                info!(
                    "   🔍 Lazy rescan discovered {} — {} capabilities",
                    primal_name,
                    capabilities.len(),
                );
                registered += capabilities.len();
            }
        }

        if registered > 0 {
            info!("✅ Lazy rescan registered {} new capabilities", registered);
        }

        registered
    }

    /// Reset the lazy-rescan gate so the next miss triggers a fresh scan.
    pub fn reset_lazy_rescan(&self) {
        self.lazy_rescan_attempted.store(false, Ordering::Relaxed);
    }
}
