// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Runtime Primal Discovery Patterns
//!
//! This module provides runtime discovery of primals without hardcoding paths.
//!
//! AGPL-3.0-only License
//!
//! # Deep Debt Principles
//!
//! - **Discover, Don't Hardcode**: Find primals at runtime via standard paths
//! - **XDG Compliant**: Use XDG runtime directory structure
//! - **5-Tier Resolution**: Follow PRIMAL_DEPLOYMENT_STANDARD hierarchy
//! - **Capability-Based**: Discover by capability, not by name

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

use crate::PrimalCapability;

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
        let discovery = Self::new("default");
        discovery
            .discover_capability(cap.clone())
            .await?
            .ok_or_else(|| anyhow::anyhow!("No primal found for capability: {:?}", cap))
    }

    /// Discover primals matching query
    pub async fn discover(&self, query: &DiscoveryQuery) -> Result<Vec<DiscoveredPrimal>> {
        let socket_dir = self.resolve_socket_dir();
        let mut results = Vec::new();

        // If we have a specific name, try that directly
        if let Some(name) = &query.name {
            if let Some(primal) = self.try_discover_primal(&socket_dir, name).await {
                results.push(primal);
            }
        }

        // If we have a capability, try known primals from taxonomy
        if let Some(cap) = &query.capability {
            for name in providers_for_capability(cap) {
                if let Some(primal) = self.try_discover_primal(&socket_dir, name).await {
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
        let query = DiscoveryQuery::capability(capability);
        let results = self.discover(&query).await?;
        Ok(results.into_iter().next())
    }

    /// Discover specific primal by name
    pub async fn discover_primal(&self, name: &str) -> Option<DiscoveredPrimal> {
        let socket_dir = self.resolve_socket_dir();
        self.try_discover_primal(&socket_dir, name).await
    }

    /// Discover primals providing a capability by querying the capability taxonomy.
    ///
    /// Uses `CapabilityTaxonomy::resolve_to_primal()` for bootstrap hints, then
    /// scans the socket directory. Returns names of discovered primals.
    pub async fn discover_by_capability(&self, capability: &str) -> Result<Vec<String>> {
        let provider_names: Vec<&str> =
            biomeos_types::CapabilityTaxonomy::resolve_to_primal(capability)
                .map(|p| vec![p])
                .unwrap_or_default();
        let socket_dir = self.resolve_socket_dir();
        let mut discovered = Vec::new();
        for name in provider_names {
            if self.try_discover_primal(&socket_dir, name).await.is_some() {
                discovered.push(name.to_string());
            }
        }
        Ok(discovered)
    }

    /// 5-tier socket directory resolution per PRIMAL_DEPLOYMENT_STANDARD
    fn resolve_socket_dir(&self) -> PathBuf {
        // Tier 1: Explicit override
        if let Ok(dir) = std::env::var("BIOMEOS_SOCKET_DIR") {
            return PathBuf::from(dir);
        }

        // Tier 2: XDG runtime directory
        if let Ok(xdg) = std::env::var("XDG_RUNTIME_DIR") {
            return PathBuf::from(xdg).join("biomeos");
        }

        // Tier 3: Linux /run/user (get UID from environment or /proc)
        if let Ok(uid) = std::env::var("UID") {
            let run_user = PathBuf::from(format!("/run/user/{}/biomeos", uid));
            if run_user.parent().map(|p| p.exists()).unwrap_or(false) {
                return run_user;
            }
        }

        // Also try /proc/self as fallback for UID
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if let Ok(meta) = std::fs::metadata("/proc/self") {
                let uid = meta.uid();
                let run_user = PathBuf::from(format!("/run/user/{}/biomeos", uid));
                if run_user.parent().map(|p| p.exists()).unwrap_or(false) {
                    return run_user;
                }
            }
        }

        // Tier 4: Android
        let android = PathBuf::from("/data/local/tmp/biomeos");
        if android.parent().map(|p| p.exists()).unwrap_or(false) {
            return android;
        }

        // Tier 5: Fallback
        PathBuf::from("/tmp/biomeos")
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
        let alt_socket_path = socket_dir.join(format!("{}.sock", name));

        let (path, method) = if socket_path.exists() {
            (socket_path, self.method_for_dir(socket_dir))
        } else if alt_socket_path.exists() {
            (alt_socket_path, self.method_for_dir(socket_dir))
        } else {
            return None;
        };

        // Try to determine capability from name (bootstrap hint when found by path scan)
        #[allow(deprecated)]
        let capability = capability_from_primal_name(name);

        // Quick health check
        let is_healthy = tokio::net::UnixStream::connect(&path).await.is_ok();

        Some(DiscoveredPrimal {
            name: name.to_string(),
            socket_path: path,
            capability,
            discovered_via: method,
            is_healthy,
        })
    }

    fn method_for_dir(&self, dir: &std::path::Path) -> DiscoveryMethod {
        let path_str = dir.to_string_lossy();
        if path_str.contains("XDG_RUNTIME_DIR") || path_str.contains("/run/user/") {
            DiscoveryMethod::XdgRuntime
        } else if path_str.contains("/data/local/tmp") {
            DiscoveryMethod::AndroidData
        } else if path_str.starts_with("/tmp") {
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
        if cap.category.eq_ignore_ascii_case(alias) || cap.name.eq_ignore_ascii_case(alias) {
            if let Some(primal) = biomeos_types::CapabilityTaxonomy::resolve_to_primal(canonical) {
                return vec![primal];
            }
        }
    }
    // Science: taxonomy has no single default; bootstrap hints for wetspring/neuralspring
    if cap.category.eq_ignore_ascii_case("science") || cap.name.eq_ignore_ascii_case("science") {
        return vec!["wetspring", "neuralspring"];
    }
    Vec::new()
}

/// Infer capability from primal name.
///
/// **DEPRECATED**: Use capability-based discovery instead. Primals should be
/// discovered by capability via `discover_by_capability()`, not by name.
/// This exists only for bootstrap/legacy when a socket is found by path scan.
#[deprecated(
    since = "0.1.0",
    note = "Use capability-based discovery. Primals are discovered by capability, not name."
)]
pub fn capability_from_primal_name(name: &str) -> PrimalCapability {
    match name.to_lowercase().as_str() {
        "beardog" => PrimalCapability::encryption(),
        "songbird" => PrimalCapability::networking(),
        "toadstool" => PrimalCapability::compute(),
        "nestgate" => PrimalCapability::storage(),
        "squirrel" => PrimalCapability::ai(),
        "wetspring" | "neuralspring" => PrimalCapability::science(),
        _ => PrimalCapability::custom(name),
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn test_discovery_query_capability() {
        let cap = PrimalCapability::encryption();
        let query = DiscoveryQuery::capability(cap.clone());
        assert_eq!(query.capability, Some(cap));
        assert!(query.healthy_only);
    }

    #[test]
    fn test_discovery_query_primal() {
        let query = DiscoveryQuery::primal("beardog");
        assert_eq!(query.name, Some("beardog".to_string()));
        assert!(query.capability.is_none());
        assert!(!query.healthy_only);
    }

    #[test]
    fn test_discovery_query_default() {
        let query = DiscoveryQuery::default();
        assert!(query.name.is_none());
        assert!(query.capability.is_none());
        assert!(!query.healthy_only);
        assert!(query.limit.is_none());
    }

    #[test]
    fn test_capability_providers_encryption() {
        let providers = providers_for_capability(&PrimalCapability::encryption());
        assert!(providers.contains(&"beardog"));
    }

    #[test]
    fn test_capability_providers_security() {
        let providers = providers_for_capability(&PrimalCapability::new("security", "x", "1.0"));
        assert!(providers.contains(&"beardog"));
    }

    #[test]
    fn test_capability_providers_networking() {
        let providers = providers_for_capability(&PrimalCapability::networking());
        assert!(providers.contains(&"songbird"));
    }

    #[test]
    fn test_capability_providers_compute() {
        let providers = providers_for_capability(&PrimalCapability::compute());
        assert!(providers.contains(&"toadstool"));
    }

    #[test]
    fn test_capability_providers_storage() {
        let providers = providers_for_capability(&PrimalCapability::storage());
        assert!(providers.contains(&"nestgate"));
    }

    #[test]
    fn test_capability_providers_ai() {
        let providers = providers_for_capability(&PrimalCapability::ai());
        assert!(providers.contains(&"squirrel"));
    }

    #[test]
    fn test_capability_providers_science() {
        let providers = providers_for_capability(&PrimalCapability::science());
        assert!(providers.contains(&"wetspring"));
        assert!(providers.contains(&"neuralspring"));
    }

    #[test]
    fn test_capability_providers_unknown_empty() {
        let providers = providers_for_capability(&PrimalCapability::new("unknown", "x", "1.0"));
        assert!(providers.is_empty());
    }

    #[test]
    #[allow(deprecated)]
    fn test_capability_from_name_beardog() {
        assert_eq!(
            capability_from_primal_name("beardog").category,
            "encryption"
        );
    }

    #[test]
    #[allow(deprecated)]
    fn test_capability_from_name_songbird() {
        assert_eq!(
            capability_from_primal_name("songbird").category,
            "networking"
        );
    }

    #[test]
    #[allow(deprecated)]
    fn test_capability_from_name_toadstool() {
        assert_eq!(capability_from_primal_name("toadstool").category, "compute");
    }

    #[test]
    #[allow(deprecated)]
    fn test_capability_from_name_nestgate() {
        assert_eq!(capability_from_primal_name("nestgate").category, "storage");
    }

    #[test]
    #[allow(deprecated)]
    fn test_capability_from_name_squirrel() {
        assert_eq!(capability_from_primal_name("squirrel").category, "ai");
    }

    #[test]
    #[allow(deprecated)]
    fn test_capability_from_name_wetspring() {
        assert_eq!(capability_from_primal_name("wetspring").category, "science");
    }

    #[test]
    #[allow(deprecated)]
    fn test_capability_from_name_neuralspring() {
        assert_eq!(
            capability_from_primal_name("neuralspring").category,
            "science"
        );
    }

    #[test]
    #[allow(deprecated)]
    fn test_capability_from_name_unknown_custom() {
        let cap = capability_from_primal_name("unknownprimal");
        assert_eq!(cap.category, "custom");
        assert_eq!(cap.name, "unknownprimal");
    }

    #[test]
    #[allow(deprecated)]
    fn test_capability_from_name_case_insensitive() {
        assert_eq!(
            capability_from_primal_name("BEARDOG").category,
            "encryption"
        );
    }

    #[test]
    fn test_primal_discovery_new() {
        let discovery = PrimalDiscovery::new("my_family");
        // Resolve uses env - just verify construction
        let _ = discovery;
    }

    #[tokio::test]
    async fn test_discover_by_capability_returns_vec() {
        let discovery = PrimalDiscovery::new("test-family");
        // No primals running in test env; should return empty or discovered names
        let result = discovery.discover_by_capability("encryption").await;
        assert!(result.is_ok());
        let names = result.unwrap();
        // Names is Vec<String> - may be empty if no beardog socket
        assert!(names.is_empty() || names.contains(&"beardog".to_string()));
    }

    #[test]
    fn test_discovered_primal_serialization() {
        let primal = DiscoveredPrimal {
            name: "beardog".to_string(),
            socket_path: PathBuf::from("/run/user/1000/biomeos/beardog-default.sock"),
            capability: PrimalCapability::encryption(),
            discovered_via: DiscoveryMethod::XdgRuntime,
            is_healthy: true,
        };
        let json = serde_json::to_string(&primal).unwrap();
        assert!(json.contains("beardog"));
    }
}
