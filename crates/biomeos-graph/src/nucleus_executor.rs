// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

// =============================================================================
// NUCLEUS-Based Primal Executor - Integration with Secure Discovery
// =============================================================================
//
// This replaces hardcoded primal discovery with NUCLEUS 5-layer secure discovery.
//
// Deep Debt Principles:
// - Delegates to NUCLEUS (no reimplementation)
// - Runtime discovery (no hardcoding)
// - Fast AND safe (async, no unsafe)
//
// =============================================================================

use async_trait::async_trait;
use biomeos_nucleus::{NucleusClient, DiscoveryRequest, VerifiedPrimal};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info, warn};

use crate::context::ExecutionContext;
use crate::error::{GraphError, Result};
use crate::executor::PrimalOperationExecutor;
use crate::graph::Operation;

/// NUCLEUS-based primal operation executor
///
/// Uses NUCLEUS for:
/// 1. Physical discovery (Songbird)
/// 2. Identity verification (BearDog)
/// 3. Capability verification (direct query)
/// 4. Trust evaluation (BearDog)
/// 5. Registry tracking (local state)
pub struct NucleusPrimalExecutor {
    /// NUCLEUS client (handles all 5 layers)
    nucleus: Arc<NucleusClient>,
    
    /// Cached verified primals (by capability)
    cache: Arc<RwLock<HashMap<String, Vec<VerifiedPrimal>>>>,
}

impl NucleusPrimalExecutor {
    /// Create a new NUCLEUS-based executor
    ///
    /// **Deep Debt Principle**: Discovers all primals at runtime via NUCLEUS
    pub async fn new() -> biomeos_nucleus::Result<Self> {
        info!("Initializing NUCLEUS-based primal executor");
        
        let nucleus = Arc::new(NucleusClient::new().await?);
        let cache = Arc::new(RwLock::new(HashMap::new()));
        
        info!("✅ NUCLEUS primal executor initialized");
        
        Ok(Self { nucleus, cache })
    }
    
    /// Resolve family ID from config/environment
    ///
    /// EVOLVED (Jan 27, 2026): XDG-compliant family resolution
    ///
    /// Priority:
    /// 1. BIOMEOS_FAMILY_ID environment variable
    /// 2. Family file in XDG data directory
    /// 3. None (discover all families)
    fn resolve_family_id() -> Option<String> {
        // Priority 1: Environment variable
        if let Ok(family) = std::env::var("BIOMEOS_FAMILY_ID") {
            if !family.is_empty() {
                debug!("Using family ID from environment: {}", family);
                return Some(family);
            }
        }

        // Priority 2: XDG data directory
        if let Ok(paths) = biomeos_types::SystemPaths::new() {
            let family_file = paths.data_dir().join("family_id");
            if family_file.exists() {
                if let Ok(contents) = std::fs::read_to_string(&family_file) {
                    let family = contents.trim().to_string();
                    if !family.is_empty() {
                        debug!("Using family ID from config file: {}", family);
                        return Some(family);
                    }
                }
            }
        }

        // Priority 3: Legacy /etc/biomeos/family_id
        let legacy_file = std::path::Path::new("/etc/biomeos/family_id");
        if legacy_file.exists() {
            if let Ok(contents) = std::fs::read_to_string(legacy_file) {
                let family = contents.trim().to_string();
                if !family.is_empty() {
                    debug!("Using family ID from legacy config: {}", family);
                    return Some(family);
                }
            }
        }

        // No family ID configured - discover all
        debug!("No family ID configured, will discover all families");
        None
    }
    
    /// Discover primals by capability (with caching)
    async fn discover_by_capability_cached(
        &self,
        capability: &str,
    ) -> biomeos_nucleus::Result<Vec<VerifiedPrimal>> {
        // Check cache first
        {
            let cache = self.cache.read().await;
            if let Some(primals) = cache.get(capability) {
                debug!(capability = %capability, count = primals.len(), "Using cached primals");
                return Ok(primals.clone());
            }
        }
        
        // Discover via NUCLEUS (all 5 layers)
        info!(capability = %capability, "Discovering primals via NUCLEUS");

        // EVOLVED (Jan 27, 2026): Get family from config or environment
        let family = Self::resolve_family_id();
        
        let request = DiscoveryRequest {
            capability: capability.to_string(),
            family,
            timeout: None,
        };
        
        let primals = self.nucleus.discover(request).await?;
        
        // Cache results
        {
            let mut cache = self.cache.write().await;
            cache.insert(capability.to_string(), primals.clone());
        }
        
        info!(
            capability = %capability,
            count = primals.len(),
            "Discovered and cached {} primals",
            primals.len()
        );
        
        Ok(primals)
    }
    
    /// Execute an operation via Unix socket JSON-RPC
    async fn execute_jsonrpc(
        &self,
        primal: &VerifiedPrimal,
        operation: &Operation,
        _context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        debug!(
            primal = %primal.name,
            operation = %operation.name,
            "Executing operation via JSON-RPC"
        );
        
        // Build JSON-RPC params from operation.params
        let params = operation.params.clone();
        
        // Call via Unix socket (using NUCLEUS's client utilities)
        let result: serde_json::Value = biomeos_nucleus::client::call_unix_socket_rpc(
            &primal.endpoint.address,
            &operation.name,  // Use operation.name as the method
            params,
        )
        .await
        .map_err(|e| GraphError::ExecutionFailed {
            node: operation.name.clone(),
            reason: format!("JSON-RPC call failed: {}", e),
        })?;
        
        debug!(
            primal = %primal.name,
            operation = %operation.name,
            "Operation completed successfully"
        );
        
        Ok(result)
    }
}

#[async_trait]
impl PrimalOperationExecutor for NucleusPrimalExecutor {
    /// Execute an operation on a primal
    async fn execute_operation(
        &self,
        primal_id: &str,
        operation: &Operation,
        context: &ExecutionContext,
    ) -> Result<serde_json::Value> {
        debug!(
            primal_id = %primal_id,
            operation = %operation.name,
            "Executing operation via NUCLEUS"
        );
        
        // Get primal from NUCLEUS registry
        let registry = self.nucleus.registry();
        
        // Parse primal_id (format: "primal:node")
        let parts: Vec<&str> = primal_id.split(':').collect();
        if parts.len() != 2 {
            return Err(GraphError::PrimalNotFound {
                primal_id: primal_id.to_string(),
                reason: "Invalid primal_id format (expected 'primal:node')".to_string(),
            });
        }
        
        let (primal_name, node_id) = (parts[0], parts[1]);
        
        // Get from registry
        let registered = registry.get(primal_name, node_id)
            .await
            .ok_or_else(|| GraphError::PrimalNotFound {
                primal_id: primal_id.to_string(),
                reason: "Primal not found in NUCLEUS registry".to_string(),
            })?;
        
        if !registered.healthy {
            warn!(
                primal_id = %primal_id,
                "Primal is marked unhealthy, attempting execution anyway"
            );
        }
        
        // Execute operation
        self.execute_jsonrpc(&registered.primal, operation, context).await
    }
    
    /// Discover primals by capability
    async fn discover_primals(&self) -> Result<Vec<(String, Vec<String>)>> {
        info!("Discovering all available primals via NUCLEUS");
        
        // Get all registered primals from NUCLEUS
        let registry = self.nucleus.registry();
        let all_primals = registry.list_all().await;
        
        // Convert to (primal_id, capabilities) format
        let result: Vec<(String, Vec<String>)> = all_primals
            .into_iter()
            .map(|registered| {
                let primal_id = format!("{}:{}", registered.primal.name, registered.primal.node_id);
                let capabilities = registered.primal.capabilities.clone();
                (primal_id, capabilities)
            })
            .collect();
        
        info!(count = result.len(), "Discovered {} primals total", result.len());
        
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    // Note: Real tests require running primals (Songbird, BearDog)
    // Unit tests here validate data structures only
    
    #[test]
    fn test_primal_id_parsing() {
        let primal_id = "beardog:node-alpha";
        let parts: Vec<&str> = primal_id.split(':').collect();
        
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0], "beardog");
        assert_eq!(parts[1], "node-alpha");
    }
    
    #[test]
    fn test_primal_id_format() {
        let name = "songbird";
        let node = "node-beta";
        let primal_id = format!("{}:{}", name, node);
        
        assert_eq!(primal_id, "songbird:node-beta");
    }
}

