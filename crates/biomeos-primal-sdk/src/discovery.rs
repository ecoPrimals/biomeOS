// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Runtime Primal Discovery Patterns
//!
//! This module provides runtime discovery of primals without hardcoding paths.
//!
//! AGPL-3.0-or-later License
//!
//! # Deep Debt Principles
//!
//! - **Discover, Don't Hardcode**: Find primals at runtime via standard paths
//! - **XDG Compliant**: Use XDG runtime directory structure
//! - **5-Tier Resolution**: Follow PRIMAL_DEPLOYMENT_STANDARD hierarchy
//! - **Capability-Based**: Discover by capability, not by name

use anyhow::Result;
use biomeos_types::defaults::DEFAULT_FAMILY_ID;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::PrimalCapability;

mod runtime;
use runtime::{bootstrap_capability_hint, probe_primary_capability, store_runtime_capability_hint};

/// A discovered primal with its runtime information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveredPrimal {
    /// Primal name (e.g., "beardog")
    pub name: String,

    /// Socket path for communication
    pub socket_path: PathBuf,

    /// Primary capability
    pub capability: PrimalCapability,

    /// How it was discovered
    pub discovered_via: DiscoveryMethod,

    /// Is the primal currently responding
    pub is_healthy: bool,
}

/// How a primal was discovered
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMethod {
    /// Explicit environment variable
    Environment(String),
    /// XDG runtime directory
    XdgRuntime,
    /// Standard /run/user path
    RunUser,
    /// Android /data/local/tmp
    AndroidData,
    /// Fallback /tmp
    TmpFallback,
    /// Neural API registry
    NeuralApi,
}

/// Query for discovering primals
#[derive(Debug, Clone, Default)]
pub struct DiscoveryQuery {
    /// Search by name
    pub name: Option<String>,

    /// Search by capability
    pub capability: Option<PrimalCapability>,

    /// Only return healthy primals
    pub healthy_only: bool,

    /// Maximum results
    pub limit: Option<usize>,
}

impl DiscoveryQuery {
    /// Create query for capability
    #[must_use]
    pub fn capability(cap: PrimalCapability) -> Self {
        Self {
            capability: Some(cap),
            healthy_only: true,
            ..Default::default()
        }
    }

    /// Create query for specific primal
    pub fn primal(name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..Default::default()
        }
    }
}

/// Runtime primal discovery service
pub struct PrimalDiscovery {
    /// Family ID for socket directory
    family_id: String,
}

impl PrimalDiscovery {
    /// Create new discovery service
    pub fn new(family_id: impl Into<String>) -> Self {
        Self {
            family_id: family_id.into(),
        }
    }

    /// Static helper to find a primal by capability using default family
    pub async fn find_by_capability(cap: PrimalCapability) -> Result<DiscoveredPrimal> {
        Self::find_by_capability_in(cap, &Self::resolve_socket_dir()).await
    }

    /// Like [`Self::find_by_capability`], but uses `socket_dir` instead of resolving from the environment.
    pub async fn find_by_capability_in(
        cap: PrimalCapability,
        socket_dir: &Path,
    ) -> Result<DiscoveredPrimal> {
        let discovery = Self::new(DEFAULT_FAMILY_ID);
        discovery
            .discover_capability_in(cap.clone(), socket_dir)
            .await?
            .ok_or_else(|| anyhow::anyhow!("No primal found for capability: {cap:?}"))
    }

    /// Discover primals matching query
    pub async fn discover(&self, query: &DiscoveryQuery) -> Result<Vec<DiscoveredPrimal>> {
        self.discover_in(query, &Self::resolve_socket_dir()).await
    }

    /// Like [`Self::discover`], but uses `socket_dir` instead of resolving from the environment.
    pub async fn discover_in(
        &self,
        query: &DiscoveryQuery,
        socket_dir: &Path,
    ) -> Result<Vec<DiscoveredPrimal>> {
        let mut results = Vec::new();

        // If we have a specific name, try that directly
        if let Some(name) = &query.name
            && let Some(primal) = self.try_discover_primal(socket_dir, name).await
        {
            results.push(primal);
        }

        // If we have a capability, try known primals from taxonomy
        if let Some(cap) = &query.capability {
            for name in providers_for_capability(cap) {
                if let Some(primal) = self.try_discover_primal(socket_dir, name).await {
                    if query.healthy_only && !primal.is_healthy {
                        continue;
                    }
                    results.push(primal);
                }
            }
        }

        // Apply limit
        if let Some(limit) = query.limit {
            results.truncate(limit);
        }

        Ok(results)
    }

    /// Discover first primal for capability
    pub async fn discover_capability(
        &self,
        capability: PrimalCapability,
    ) -> Result<Option<DiscoveredPrimal>> {
        self.discover_capability_in(capability, &Self::resolve_socket_dir())
            .await
    }

    /// Like [`Self::discover_capability`], but uses `socket_dir` instead of resolving from the environment.
    pub async fn discover_capability_in(
        &self,
        capability: PrimalCapability,
        socket_dir: &Path,
    ) -> Result<Option<DiscoveredPrimal>> {
        let query = DiscoveryQuery::capability(capability);
        let results = self.discover_in(&query, socket_dir).await?;
        Ok(results.into_iter().next())
    }

    /// Discover specific primal by name
    pub async fn discover_primal(&self, name: &str) -> Option<DiscoveredPrimal> {
        self.discover_primal_in(name, &Self::resolve_socket_dir())
            .await
    }

    /// Like [`Self::discover_primal`], but uses `socket_dir` instead of resolving from the environment.
    pub async fn discover_primal_in(
        &self,
        name: &str,
        socket_dir: &Path,
    ) -> Option<DiscoveredPrimal> {
        self.try_discover_primal(socket_dir, name).await
    }

    /// Discover primals providing a capability by querying the capability taxonomy.
    ///
    /// Uses `CapabilityTaxonomy::resolve_to_primal()` for bootstrap hints, then
    /// scans the socket directory. Returns names of discovered primals.
    pub async fn discover_by_capability(&self, capability: &str) -> Result<Vec<String>> {
        self.discover_by_capability_in(capability, &Self::resolve_socket_dir())
            .await
    }

    /// Like [`Self::discover_by_capability`], but uses `socket_dir` instead of resolving from the environment.
    pub async fn discover_by_capability_in(
        &self,
        capability: &str,
        socket_dir: &Path,
    ) -> Result<Vec<String>> {
        let provider_names: Vec<&str> =
            biomeos_types::CapabilityTaxonomy::resolve_to_primal(capability)
                .map(|p| vec![p])
                .unwrap_or_default();
        let mut discovered = Vec::new();
        for name in provider_names {
            if self.try_discover_primal(socket_dir, name).await.is_some() {
                discovered.push(name.to_string());
            }
        }
        Ok(discovered)
    }

    /// Socket directory resolution: explicit override → `SystemPaths` (XDG-compliant).
    fn resolve_socket_dir() -> PathBuf {
        // Explicit override takes priority
        if let Ok(dir) = std::env::var(biomeos_types::env_config::vars::SOCKET_DIR) {
            return PathBuf::from(dir);
        }

        // SystemPaths handles XDG_RUNTIME_DIR → /run/user/$UID → tmp fallback
        biomeos_types::SystemPaths::new_lazy()
            .runtime_dir()
            .to_path_buf()
    }

    /// Try to discover a specific primal
    async fn try_discover_primal(
        &self,
        socket_dir: &std::path::Path,
        name: &str,
    ) -> Option<DiscoveredPrimal> {
        // Standard socket naming: {primal}-{family}.sock
        let socket_name = format!("{}-{}.sock", name, self.family_id);
        let socket_path = socket_dir.join(&socket_name);

        // Also try without family suffix
        let alt_socket_path = socket_dir.join(format!("{name}.sock"));

        let (path, method) = if socket_path.exists() {
            (socket_path, Self::method_for_dir(socket_dir))
        } else if alt_socket_path.exists() {
            (alt_socket_path, Self::method_for_dir(socket_dir))
        } else {
            return None;
        };

        // Quick health check — UDS probe on Unix, assume unhealthy on Windows
        #[cfg(unix)]
        let is_healthy = tokio::net::UnixStream::connect(&path).await.is_ok();
        #[cfg(windows)]
        let is_healthy = false;

        // Populate runtime cache from live capability introspection when possible.
        #[cfg(unix)]
        if is_healthy && let Some(cap) = probe_primary_capability(&path).await {
            store_runtime_capability_hint(name, cap);
        }

        // Runtime cache first, then last-resort static bootstrap hints.
        let capability = bootstrap_capability_hint(name);

        Some(DiscoveredPrimal {
            name: name.to_string(),
            socket_path: path,
            capability,
            discovered_via: method,
            is_healthy,
        })
    }

    fn method_for_dir(dir: &std::path::Path) -> DiscoveryMethod {
        use biomeos_types::constants::runtime_paths;
        let path_str = dir.to_string_lossy();
        if path_str.contains("XDG_RUNTIME_DIR")
            || path_str.contains(runtime_paths::LINUX_RUNTIME_DIR_PREFIX)
        {
            DiscoveryMethod::XdgRuntime
        } else if path_str.contains(runtime_paths::ANDROID_RUNTIME_BASE) {
            DiscoveryMethod::AndroidData
        } else if path_str.starts_with(runtime_paths::FALLBACK_RUNTIME_BASE) {
            DiscoveryMethod::TmpFallback
        } else {
            DiscoveryMethod::RunUser
        }
    }
}

/// Get known provider names for a capability using the capability taxonomy.
///
/// Uses `biomeos_types::CapabilityTaxonomy` for capability→primal resolution.
/// Returns bootstrap hints only; in sovereign mode, primals self-register at runtime.
#[must_use]
pub fn providers_for_capability(cap: &PrimalCapability) -> Vec<&'static str> {
    // Try category first (e.g., "encryption", "security", "compute")
    for key in [cap.category.as_str(), cap.name.as_str()] {
        if let Some(primal) = biomeos_types::CapabilityTaxonomy::resolve_to_primal(key) {
            return vec![primal];
        }
    }
    // Aliases for taxonomy compatibility
    let aliases: &[(&str, &str)] = &[
        ("security", "encryption"),
        ("registry", "discovery"),
        ("networking", "discovery"),
        ("crypto", "encryption"),
        ("http", "discovery"),
    ];
    for (alias, canonical) in aliases {
        if (cap.category.eq_ignore_ascii_case(alias) || cap.name.eq_ignore_ascii_case(alias))
            && let Some(primal) = biomeos_types::CapabilityTaxonomy::resolve_to_primal(canonical)
        {
            return vec![primal];
        }
    }
    // Science: taxonomy has no single default; bootstrap hints use canonical constants
    if cap.category.eq_ignore_ascii_case("science") || cap.name.eq_ignore_ascii_case("science") {
        return vec![
            biomeos_types::primal_names::WETSPRING,
            biomeos_types::primal_names::NEURALSPRING,
        ];
    }
    Vec::new()
}

#[cfg(test)]
#[path = "discovery_tests.rs"]
mod tests;
