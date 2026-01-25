// Capability Translation Registry for Neural API
//
// Enables primals to speak in semantic capabilities while Neural API
// automatically translates to provider-specific method names.
//
// See: specs/CAPABILITY_TRANSLATION_ARCHITECTURE.md

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::io::AsyncWriteExt;
use tokio::net::UnixStream;
use tracing::{debug, info, trace};

/// Capability translation entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CapabilityTranslation {
    /// Semantic capability name (what consumers call)
    pub semantic: String,

    /// Provider primal ID
    pub provider: String,

    /// Actual method name on provider
    pub actual_method: String,

    /// Provider socket path
    pub socket: String,

    /// Parameter name mappings (semantic → actual)
    /// Example: {"private_key": "our_secret", "public_key": "their_public"}
    #[serde(default)]
    pub param_mappings: HashMap<String, String>,

    /// Optional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Capability Translation Registry
///
/// Maintains mappings between semantic capabilities and provider-specific methods
#[derive(Debug, Clone)]
pub struct CapabilityTranslationRegistry {
    /// Semantic capability → Translation
    translations: HashMap<String, CapabilityTranslation>,

    /// Provider → List of semantic capabilities they provide
    provider_capabilities: HashMap<String, Vec<String>>,

    /// Next RPC ID
    next_id: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl CapabilityTranslationRegistry {
    /// Create new empty registry
    pub fn new() -> Self {
        Self {
            translations: HashMap::new(),
            provider_capabilities: HashMap::new(),
            next_id: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(1)),
        }
    }

    /// Register a capability translation
    ///
    /// # Arguments
    ///
    /// * `semantic` - Semantic capability name (e.g., "crypto.generate_keypair")
    /// * `provider` - Provider primal ID (e.g., "beardog")
    /// * `actual_method` - Actual method name on provider (e.g., "x25519_generate_ephemeral")
    /// * `socket` - Provider socket path
    ///
    /// # Example
    ///
    /// ```ignore
    /// registry.register_translation(
    ///     "crypto.generate_keypair",
    ///     "beardog",
    ///     "x25519_generate_ephemeral",
    ///     "/tmp/beardog-nat0.sock",
    ///     None  // No parameter mappings
    /// );
    /// ```
    pub fn register_translation(
        &mut self,
        semantic: impl Into<String>,
        provider: impl Into<String>,
        actual_method: impl Into<String>,
        socket: impl Into<String>,
        param_mappings: Option<HashMap<String, String>>,
    ) {
        let semantic = semantic.into();
        let provider = provider.into();
        let actual_method = actual_method.into();
        let socket = socket.into();

        debug!(
            "📝 Registering translation: {} → {} ({}) {}",
            semantic,
            actual_method,
            provider,
            if param_mappings.is_some() {
                "with param mappings"
            } else {
                ""
            }
        );

        let translation = CapabilityTranslation {
            semantic: semantic.clone(),
            provider: provider.clone(),
            actual_method,
            socket,
            param_mappings: param_mappings.unwrap_or_default(),
            metadata: HashMap::new(),
        };

        // Add to translations map
        self.translations.insert(semantic.clone(), translation);

        // Add to provider capabilities
        self.provider_capabilities
            .entry(provider)
            .or_insert_with(Vec::new)
            .push(semantic);
    }

    /// Get translation for a semantic capability
    pub fn get_translation(&self, semantic: &str) -> Option<&CapabilityTranslation> {
        self.translations.get(semantic)
    }

    /// Check if capability is available
    pub fn has_capability(&self, semantic: &str) -> bool {
        self.translations.contains_key(semantic)
    }

    /// List all capabilities provided by a specific provider
    pub fn provider_capabilities(&self, provider: &str) -> Vec<String> {
        self.provider_capabilities
            .get(provider)
            .cloned()
            .unwrap_or_default()
    }

    /// List all translations
    pub fn list_all(&self) -> Vec<&CapabilityTranslation> {
        self.translations.values().collect()
    }

    /// Call a capability with automatic translation
    ///
    /// # Arguments
    ///
    /// * `semantic` - Semantic capability name
    /// * `params` - Parameters to pass to the method
    ///
    /// # Returns
    ///
    /// The result from the provider
    ///
    /// # Example
    ///
    /// ```ignore
    /// let result = registry.call_capability(
    ///     "crypto.generate_keypair",
    ///     json!({"algorithm": "x25519"})
    /// ).await?;
    /// ```
    pub async fn call_capability(
        &self,
        semantic: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        // 1. Lookup translation
        let translation = self
            .get_translation(semantic)
            .ok_or_else(|| anyhow!("No provider for capability: {}", semantic))?;

        info!(
            "🔄 Translating {} → {} (provider: {}, socket: {})",
            semantic, translation.actual_method, translation.provider, translation.socket
        );
        debug!("   Params (semantic): {}", params);

        // 2. Apply parameter name mappings
        let mapped_params = if !translation.param_mappings.is_empty() {
            if let serde_json::Value::Object(obj) = params {
                let mut mapped_obj = serde_json::Map::new();
                for (key, value) in obj {
                    // Use mapping if available, otherwise keep original name
                    let actual_key = translation.param_mappings.get(&key).unwrap_or(&key);
                    mapped_obj.insert(actual_key.clone(), value);
                }
                debug!(
                    "   Params (after mapping): {}",
                    serde_json::Value::Object(mapped_obj.clone())
                );
                serde_json::Value::Object(mapped_obj)
            } else {
                params
            }
        } else {
            params
        };

        // 3. Connect to provider
        let mut stream = UnixStream::connect(&translation.socket)
            .await
            .map_err(|e| {
                anyhow!(
                    "Failed to connect to provider {} at {}: {}",
                    translation.provider,
                    translation.socket,
                    e
                )
            })?;

        // 4. Build RPC with ACTUAL method name and MAPPED parameters
        let id = self
            .next_id
            .fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        let rpc_request = serde_json::json!({
            "jsonrpc": "2.0",
            "method": translation.actual_method,
            "params": mapped_params,
            "id": id
        });

        let rpc_request_str = serde_json::to_string(&rpc_request)?;
        info!(
            "→ Provider RPC: method={}, socket={}",
            translation.actual_method, translation.socket
        );
        trace!("→ Full RPC request: {}", rpc_request_str);

        // 4. Send request
        let send_start = std::time::Instant::now();
        trace!(
            "→ Sending {} bytes to {}",
            rpc_request_str.len(),
            translation.socket
        );
        stream.write_all(rpc_request_str.as_bytes()).await?;
        stream.write_all(b"\n").await?; // Add newline for line-based protocols
        stream.flush().await?;
        debug!("✓ Request sent in {:?}", send_start.elapsed());

        // Shutdown write half to signal we're done sending
        // This prompts the provider to close after sending its response
        trace!("→ Shutting down write half");
        stream.shutdown().await?;
        debug!("✓ Write half shutdown");

        // 5. Read response with JSON-aware reading
        // BearDog sends complete JSON but keeps socket open, so we read until JSON is complete
        use tokio::io::AsyncReadExt;
        use tokio::time::{timeout, Duration};

        let read_start = std::time::Instant::now();
        trace!("→ Reading response (JSON-aware)");

        let mut buffer = Vec::new();
        let mut temp_buf = [0u8; 4096];
        let read_timeout = Duration::from_millis(100);

        loop {
            match timeout(read_timeout, stream.read(&mut temp_buf)).await {
                Ok(Ok(0)) => {
                    trace!("EOF received");
                    break;
                }
                Ok(Ok(n)) => {
                    buffer.extend_from_slice(&temp_buf[..n]);
                    trace!("Read {} bytes, total: {}", n, buffer.len());

                    // Check if we have complete JSON
                    if let Ok(s) = std::str::from_utf8(&buffer) {
                        if serde_json::from_str::<serde_json::Value>(s).is_ok() {
                            debug!(
                                "✓ Complete JSON detected, read {} bytes in {:?}",
                                buffer.len(),
                                read_start.elapsed()
                            );
                            break;
                        }
                    }
                }
                Ok(Err(e)) => {
                    return Err(anyhow!("Socket read error: {}", e));
                }
                Err(_) => {
                    // Timeout - check if we have valid JSON
                    trace!("Read timeout, checking for complete JSON...");
                    if !buffer.is_empty() {
                        if let Ok(s) = std::str::from_utf8(&buffer) {
                            if serde_json::from_str::<serde_json::Value>(s).is_ok() {
                                debug!(
                                    "✓ Complete JSON found after timeout, {} bytes in {:?}",
                                    buffer.len(),
                                    read_start.elapsed()
                                );
                                break;
                            }
                        }
                    }
                    // Provider is slow or socket issue
                    return Err(anyhow!(
                        "Timeout reading response from provider (no complete JSON received)"
                    ));
                }
            }
        }

        let response_line = String::from_utf8(buffer)?;
        info!(
            "← Provider RPC response received ({} bytes)",
            response_line.len()
        );
        trace!("← Full response: {}", response_line.trim());

        let rpc_response: serde_json::Value = serde_json::from_str(&response_line)?;

        // 6. Check for errors
        if let Some(error) = rpc_response.get("error") {
            return Err(anyhow!(
                "Provider {} error for {}: {}",
                translation.provider,
                semantic,
                error
            ));
        }

        // 7. Return result
        Ok(rpc_response["result"].clone())
    }

    /// Get statistics about the registry
    pub fn stats(&self) -> RegistryStats {
        RegistryStats {
            total_translations: self.translations.len(),
            total_providers: self.provider_capabilities.len(),
            capabilities_by_provider: self
                .provider_capabilities
                .iter()
                .map(|(k, v)| (k.clone(), v.len()))
                .collect(),
        }
    }
}

impl Default for CapabilityTranslationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry statistics
#[derive(Debug, Clone, Serialize)]
pub struct RegistryStats {
    pub total_translations: usize,
    pub total_providers: usize,
    pub capabilities_by_provider: HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_register_translation() {
        let mut registry = CapabilityTranslationRegistry::new();

        registry.register_translation(
            "crypto.generate_keypair",
            "beardog",
            "x25519_generate_ephemeral",
            "/tmp/beardog.sock",
            None,
        );

        assert!(registry.has_capability("crypto.generate_keypair"));

        let translation = registry.get_translation("crypto.generate_keypair").unwrap();
        assert_eq!(translation.semantic, "crypto.generate_keypair");
        assert_eq!(translation.provider, "beardog");
        assert_eq!(translation.actual_method, "x25519_generate_ephemeral");
        assert_eq!(translation.socket, "/tmp/beardog.sock");
    }

    #[test]
    fn test_provider_capabilities() {
        let mut registry = CapabilityTranslationRegistry::new();

        registry.register_translation(
            "crypto.generate_keypair",
            "beardog",
            "x25519_generate_ephemeral",
            "/tmp/beardog.sock",
            None,
        );

        registry.register_translation(
            "crypto.ecdh_derive",
            "beardog",
            "x25519_derive_secret",
            "/tmp/beardog.sock",
            None,
        );

        let caps = registry.provider_capabilities("beardog");
        assert_eq!(caps.len(), 2);
        assert!(caps.contains(&"crypto.generate_keypair".to_string()));
        assert!(caps.contains(&"crypto.ecdh_derive".to_string()));
    }

    #[test]
    fn test_list_all() {
        let mut registry = CapabilityTranslationRegistry::new();

        registry.register_translation(
            "crypto.generate_keypair",
            "beardog",
            "x25519_generate_ephemeral",
            "/tmp/beardog.sock",
            None,
        );

        registry.register_translation(
            "http.request",
            "songbird",
            "http_request",
            "/tmp/songbird.sock",
            None,
        );

        let all = registry.list_all();
        assert_eq!(all.len(), 2);
    }

    #[test]
    fn test_stats() {
        let mut registry = CapabilityTranslationRegistry::new();

        registry.register_translation(
            "crypto.generate_keypair",
            "beardog",
            "x25519_generate_ephemeral",
            "/tmp/beardog.sock",
            None,
        );

        registry.register_translation(
            "http.request",
            "songbird",
            "http_request",
            "/tmp/songbird.sock",
            None,
        );

        let stats = registry.stats();
        assert_eq!(stats.total_translations, 2);
        assert_eq!(stats.total_providers, 2);
        assert_eq!(stats.capabilities_by_provider["beardog"], 1);
        assert_eq!(stats.capabilities_by_provider["songbird"], 1);
    }
}
