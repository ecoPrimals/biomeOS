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

        // If we have a capability, try known primals
        if let Some(cap) = &query.capability {
            let primal_names = providers_for_capability(cap);
            for name in primal_names {
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
        socket_dir: &PathBuf,
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

        // Try to determine capability from name
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

    fn method_for_dir(&self, dir: &PathBuf) -> DiscoveryMethod {
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

/// Get known provider names for a capability
pub fn providers_for_capability(cap: &PrimalCapability) -> &'static [&'static str] {
    // Match on category and name
    match (cap.category.as_str(), cap.name.as_str()) {
        ("encryption", _) | ("security", _) => &["beardog"],
        ("networking", _) => &["songbird"],
        ("compute", _) => &["toadstool"],
        ("storage", _) | ("data", _) => &["nestgate"],
        ("ai", _) | ("ml", _) => &["squirrel"],
        _ => &[],
    }
}

/// Infer capability from primal name
pub fn capability_from_primal_name(name: &str) -> PrimalCapability {
    match name.to_lowercase().as_str() {
        "beardog" => PrimalCapability::encryption(),
        "songbird" => PrimalCapability::networking(),
        "toadstool" => PrimalCapability::compute(),
        "nestgate" => PrimalCapability::storage(),
        "squirrel" => PrimalCapability::ai(),
        _ => PrimalCapability::custom(name),
    }
}

#[cfg(test)]
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
    fn test_capability_providers() {
        let providers = providers_for_capability(&PrimalCapability::encryption());
        assert!(providers.contains(&"beardog"));
    }

    #[test]
    fn test_capability_from_name() {
        assert_eq!(
            capability_from_primal_name("beardog").category,
            "encryption"
        );
        assert_eq!(
            capability_from_primal_name("songbird").category,
            "networking"
        );
    }
}
