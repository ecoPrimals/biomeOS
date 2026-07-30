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
use crate::capability_domains::register_capability_provider;
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

        register_capability_provider(&capability, &primal_name);

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
    /// Prune capabilities for primals that fail health checks consecutively.
    ///
    /// Uses a strike counter to prevent the "capability wipe cycle" where
    /// socket contention during concurrent discovery sweeps causes mass
    /// false-positive pruning. A primal is only unregistered after
    /// `PRUNE_STRIKE_THRESHOLD` (3) consecutive failures. A single successful
    /// health check resets the counter to zero.
    pub async fn prune_stale_registrations(&self) -> (usize, usize) {
        use std::collections::HashSet;

        const PRUNE_STRIKE_THRESHOLD: u8 = 3;

        let registry = self.capability_registry.read().await;

        let mut endpoints: HashSet<(Arc<str>, String)> = HashSet::new();
        for providers in registry.values() {
            for reg in providers {
                endpoints.insert((reg.primal_name.clone(), reg.endpoint.display_string()));
            }
        }
        drop(registry);

        let probed = endpoints.len();
        let mut failed_primals: HashSet<Arc<str>> = HashSet::new();
        let mut healthy_primals: HashSet<Arc<str>> = HashSet::new();

        for (primal_name, _endpoint_str) in &endpoints {
            let registry = self.capability_registry.read().await;
            let endpoint = registry
                .values()
                .flat_map(|v| v.iter())
                .find(|r| &r.primal_name == primal_name)
                .map(|r| r.endpoint.clone());
            drop(registry);

            if let Some(ep) = endpoint {
                if Self::check_endpoint_health(&ep).await {
                    healthy_primals.insert(primal_name.clone());
                } else {
                    failed_primals.insert(primal_name.clone());
                }
            }
        }

        // Update strike counters
        let mut strikes = self.prune_strikes.write().await;

        // Reset strikes for healthy primals
        for primal in &healthy_primals {
            strikes.remove(primal);
        }

        // Increment strikes for failed primals, collect those exceeding threshold
        let mut dead_primals: Vec<Arc<str>> = Vec::new();
        for primal in &failed_primals {
            let count = strikes.entry(primal.clone()).or_insert(0);
            *count = count.saturating_add(1);
            if *count >= PRUNE_STRIKE_THRESHOLD {
                dead_primals.push(primal.clone());
            }
        }

        drop(strikes);

        // Only unregister primals that exceeded the strike threshold
        let mut pruned = 0;
        for primal in &dead_primals {
            pruned += self.unregister_primal(primal).await;
            // Clear strike entry after successful prune
            self.prune_strikes.write().await.remove(primal);
        }

        if pruned > 0 {
            info!(
                "🧹 Stale prune sweep: probed {probed} endpoints, pruned {pruned} registrations \
                 ({} dead primals after {PRUNE_STRIKE_THRESHOLD} consecutive failures)",
                dead_primals.len()
            );
        } else if !failed_primals.is_empty() {
            debug!(
                "🧹 Stale prune sweep: probed {probed} endpoints, {} failed (strike incremented, threshold={PRUNE_STRIKE_THRESHOLD})",
                failed_primals.len()
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
                let capabilities = probe_primal_capabilities_standalone(socket_str.as_str()).await;

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

    /// Persist the capability registry to disk as a warm-cache snapshot.
    ///
    /// Written to `socket_dir/capability-registry.json`. On restart, this file
    /// is loaded before live socket probing, eliminating the cold-start window
    /// where capabilities are unavailable ("socket evaporation" fix).
    pub async fn persist_capability_registry(&self, socket_dir: &std::path::Path) {
        let registry = self.capability_registry.read().await;
        let entries: Vec<&super::types::RegisteredCapability> =
            registry.values().flatten().collect();

        if entries.is_empty() {
            return;
        }

        let path = socket_dir.join("capability-registry.json");
        match serde_json::to_string(&entries) {
            Ok(json) => {
                if let Err(e) = tokio::fs::write(&path, json).await {
                    warn!("Failed to persist capability registry: {e}");
                } else {
                    debug!(
                        "📋 Persisted {} capability entries to {}",
                        entries.len(),
                        path.display()
                    );
                }
            }
            Err(e) => warn!("Failed to serialize capability registry: {e}"),
        }
    }

    /// Load a persisted capability registry snapshot from disk.
    ///
    /// Entries are loaded as warm-cache hints (source: "persisted"). Live
    /// discovery will overwrite them with confirmed endpoints. Returns the
    /// number of capabilities loaded.
    pub async fn load_persisted_capability_registry(&self, socket_dir: &std::path::Path) -> usize {
        let path = socket_dir.join("capability-registry.json");
        let contents = match tokio::fs::read_to_string(&path).await {
            Ok(c) => c,
            Err(_) => return 0,
        };

        let entries: Vec<super::types::RegisteredCapability> = match serde_json::from_str(&contents)
        {
            Ok(e) => e,
            Err(e) => {
                warn!("Failed to parse persisted capability registry: {e}");
                return 0;
            }
        };

        let mut count = 0;
        for entry in entries {
            if let Err(e) = self
                .register_capability(
                    entry.capability.as_ref(),
                    entry.primal_name.as_ref(),
                    entry.endpoint,
                    "persisted",
                )
                .await
            {
                debug!("   Skipped persisted entry {}: {e}", entry.capability);
            } else {
                count += 1;
            }
        }

        if count > 0 {
            info!(
                "📋 Loaded {} persisted capabilities from {}",
                count,
                path.display()
            );
        }

        count
    }
}
