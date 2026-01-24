//! Universal Primal Client implementation

use reqwest::Method;
use serde::{de::DeserializeOwned, Serialize};
use std::sync::{Arc, RwLock};
use tracing::{debug, info, warn};

use super::{
    adapters::format::FormatAdapter,
    adapters::protocol::ProtocolAdapter,
    cache::ClientCache,
    config::ClientConfig,
    discovery::{DiscoveryClient, EnvDiscoveryClient},
    error::{ApiError, Result},
    handle::{PrimalHandle, PrimalId, PrimalMetadata},
};

/// Universal Primal Client
///
/// Agnostic API negotiation layer for communicating with any primal.
pub struct UniversalPrimalClient {
    config: ClientConfig,
    format_adapter: FormatAdapter,     // ✅ Concrete enum, not Arc<dyn>
    protocol_adapter: ProtocolAdapter, // ✅ Concrete enum, not Arc<dyn>
    discovery: Arc<dyn DiscoveryClient>,
    cache: Arc<RwLock<ClientCache>>,
}

impl UniversalPrimalClient {
    /// Create a new universal client with given configuration
    pub fn new(config: ClientConfig) -> Self {
        Self {
            config,
            format_adapter: FormatAdapter::default(), // ✅ Use enum default
            protocol_adapter: ProtocolAdapter::default(), // ✅ Use enum default
            discovery: Arc::new(EnvDiscoveryClient::new()),
            cache: Arc::new(RwLock::new(ClientCache::new())),
        }
    }

    /// Set format adapter
    pub fn with_format_adapter(mut self, adapter: FormatAdapter) -> Self {
        self.format_adapter = adapter;
        self
    }

    /// Set protocol adapter
    pub fn with_protocol_adapter(mut self, adapter: ProtocolAdapter) -> Self {
        self.protocol_adapter = adapter;
        self
    }

    /// Discover primal by capability
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use biomeos_core::primal_client::UniversalPrimalClient;
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = UniversalPrimalClient::new(Default::default());
    /// let beardog = client.discover_primal("security").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn discover_primal(&self, capability: &str) -> Result<PrimalHandle> {
        info!("🔍 Discovering primal with capability: {}", capability);

        // Check cache first
        {
            let cache = self.cache.read().unwrap();
            if let Some(primals) = cache.get_primals(capability, self.config.cache.discovery_ttl) {
                if let Some(primal) = primals.first() {
                    debug!("✅ Found primal in cache: {}", primal.name);
                    return Ok(primal.clone());
                }
            }
        }

        // Check explicit endpoints
        if let Some(endpoint) = self.config.endpoints.get(capability) {
            debug!("Using explicit endpoint for {}: {}", capability, endpoint);
            let mut handle = PrimalHandle::new(PrimalId::new(capability), capability.to_string());
            handle
                .endpoints
                .push(super::handle::Endpoint::new(endpoint.clone(), "http"));
            handle.capabilities.push(capability.to_string());
            return Ok(handle);
        }

        // Discover via discovery client
        let primals = self.discovery.discover(capability).await?;

        if primals.is_empty() {
            warn!("❌ No primals found with capability: {}", capability);
            return Err(ApiError::PrimalNotFound {
                capability: capability.to_string(),
            });
        }

        // Cache discovered primals
        {
            let mut cache = self.cache.write().unwrap();
            cache.set_primals(capability.to_string(), primals.clone());
        }

        let primal = primals.into_iter().next().unwrap();
        info!("✅ Discovered primal: {} ({})", primal.name, primal.id);

        Ok(primal)
    }

    /// Call primal endpoint with automatic format negotiation
    ///
    /// # Example
    ///
    /// ```ignore
    /// # use biomeos_core::primal_client::UniversalPrimalClient;
    /// # use serde::Deserialize;
    /// # #[derive(Deserialize)]
    /// # struct IdentityResponse {
    /// #     encryption_tag: String,
    /// # }
    /// # async fn example() -> anyhow::Result<()> {
    /// let client = UniversalPrimalClient::new(Default::default());
    /// let beardog = client.discover_primal("security").await?;
    ///
    /// let identity: IdentityResponse = client
    ///     .call(&beardog, "get_identity", ())
    ///     .await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn call<Req, Res>(
        &self,
        primal: &PrimalHandle,
        operation: &str,
        request: Req,
    ) -> Result<Res>
    where
        Req: Serialize + Send,
        Res: DeserializeOwned + Send,
    {
        debug!("📞 Calling {}.{}", primal.name, operation);

        // Get endpoint
        let endpoint = primal.primary_endpoint().ok_or_else(|| ApiError::Other {
            message: format!("Primal {} has no endpoints", primal.id),
        })?;

        // Build full URL
        // Currently uses convention-based paths; future: schema-driven path resolution
        let url = format!("{}/api/v1/{}", endpoint.url, operation);

        // Serialize request body
        let body = if std::mem::size_of::<Req>() > 0 {
            Some(serde_json::to_vec(&request)?)
        } else {
            None
        };

        // Determine HTTP method
        // Currently uses simple heuristic; future: schema-driven method selection
        let method = if body.is_some() {
            Method::POST
        } else {
            Method::GET
        };

        // Make request via protocol adapter
        let response = self.protocol_adapter.request(&url, method, body).await?;

        // Parse response via format adapter
        let result: Res = self.format_adapter.parse(response).await?;

        debug!("✅ Call successful: {}.{}", primal.name, operation);
        Ok(result)
    }

    /// Get primal metadata
    pub async fn get_metadata(&self, primal: &PrimalHandle) -> Result<PrimalMetadata> {
        // Future: Call primal's /api/identity or /api/metadata endpoint
        // For now, return handle-based metadata
        Ok(PrimalMetadata {
            name: primal.name.clone(),
            version: "unknown".to_string(),
            capabilities: primal
                .capabilities
                .iter()
                .map(|c| super::handle::Capability::new(c.clone(), "1.0"))
                .collect(),
            api_version: "v1".to_string(),
            schema_url: None,
            health_endpoint: None,
        })
    }

    /// Check if primal supports capability
    pub fn has_capability(&self, primal: &PrimalHandle, capability: &str) -> bool {
        primal.has_capability(capability)
    }
}

impl Default for UniversalPrimalClient {
    fn default() -> Self {
        Self::new(ClientConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_client_creation() {
        let _client = UniversalPrimalClient::new(ClientConfig::default());
    }
}
