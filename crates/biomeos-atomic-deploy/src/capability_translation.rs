// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Capability Translation Registry for Neural API
//!
//! This module provides semantic-to-actual method translation for capability-based
//! routing. It enables primals to communicate using semantic capabilities (e.g.,
//! "crypto.encrypt") while the registry automatically translates to provider-specific
//! method names (e.g., "chacha20_poly1305_encrypt").
//!
//! # Architecture
//!
//! ```text
//! Consumer                    Registry                      Provider
//!    │                           │                              │
//!    │ capability.call(          │                              │
//!    │   "crypto.encrypt",       │                              │
//!    │   params)                 │                              │
//!    │──────────────────────────>│                              │
//!    │                           │ Lookup translation           │
//!    │                           │ Map parameters               │
//!    │                           │ Resolve socket               │
//!    │                           │                              │
//!    │                           │ call(                        │
//!    │                           │   "chacha20_poly1305_encrypt",
//!    │                           │   mapped_params)             │
//!    │                           │─────────────────────────────>│
//!    │                           │                              │
//!    │                           │<─────────────────────────────│
//!    │<──────────────────────────│                              │
//! ```
//!
//! # Configuration
//!
//! Capability translations can be loaded from:
//! - TOML configuration files (see `config/capability_registry.toml`)
//! - Programmatic defaults via `load_defaults()`
//! - Manual registration via `register_translation()`
//!
//! # Socket Resolution
//!
//! Socket paths are resolved via:
//! 1. `$PRIMAL_SOCKET` environment variable (e.g., `$BEARDOG_SOCKET`)
//! 2. `SystemPaths::new_lazy().primal_socket()` -- XDG-compliant resolution
//!    (handles `XDG_RUNTIME_DIR`, `/run/user/{uid}`, and `/tmp` fallbacks)
//!
//! # Example
//!
//! ```ignore
//! use biomeos_atomic_deploy::capability_translation::CapabilityTranslationRegistry;
//!
//! let mut registry = CapabilityTranslationRegistry::new();
//! registry.load_defaults();
//!
//! // Call a capability - automatically translated to provider method
//! let result = registry.call_capability(
//!     "crypto.encrypt",
//!     serde_json::json!({"plaintext": "hello", "context": "test"})
//! ).await?;
//! ```
//!
//! See: `specs/CAPABILITY_TRANSLATION_ARCHITECTURE.md`

use anyhow::{anyhow, Result};
use biomeos_core::atomic_client::AtomicClient;
use biomeos_types::primal_names::{BEARDOG, NESTGATE, SONGBIRD, SQUIRREL, TOADSTOOL};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, trace};

// =============================================================================
// Type Definitions
// =============================================================================

/// Method translation tuple: (semantic_name, actual_method_name)
type MethodTranslation = (&'static str, &'static str);

/// Domain provider mapping: (primal_name, domain_name, method_translations)
type DomainProvider = (&'static str, &'static str, &'static [MethodTranslation]);

/// Collection of domain provider mappings for capability translation
type DomainProviderMappings = &'static [DomainProvider];

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

    /// Next RPC ID (reserved for tarpc request correlation)
    _next_id: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl CapabilityTranslationRegistry {
    /// Create new empty registry
    pub fn new() -> Self {
        Self {
            translations: HashMap::new(),
            provider_capabilities: HashMap::new(),
            _next_id: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(1)),
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
    ///     "/tmp/beardog-family.sock",
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
            .or_default()
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
    /// Uses Universal IPC v3.0 `AtomicClient` for multi-transport support.
    /// This enables Unix sockets, abstract sockets (Android), and TCP fallback.
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
            .ok_or_else(|| anyhow!("No provider for capability: {semantic}"))?;

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

        // 3. Create AtomicClient for provider socket (Universal IPC v3.0)
        let client = AtomicClient::unix(&translation.socket);

        info!(
            "→ Provider RPC: method={}, socket={}",
            translation.actual_method, translation.socket
        );
        trace!("→ Params: {}", mapped_params);

        // 4. Call provider with actual method name and mapped parameters
        let result = client
            .call(&translation.actual_method, mapped_params)
            .await
            .map_err(|e| {
                anyhow!(
                    "Provider {} error for {}: {}",
                    translation.provider,
                    semantic,
                    e
                )
            })?;

        info!("← Provider RPC response received");
        trace!("← Result: {}", result);

        // 5. Return result
        Ok(result)
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

    /// Load translations from a TOML configuration file
    ///
    /// This enables agnostic capability registration without hardcoding
    /// primal names in the source code.
    ///
    /// # Arguments
    ///
    /// * `config_path` - Path to the capability_registry.toml file
    /// * `socket_resolver` - Function to resolve socket paths for primals
    ///
    /// # Example
    ///
    /// ```ignore
    /// let socket_resolver = |primal: &str, family_id: &str| {
    ///     format!("/tmp/{}-{}.sock", primal, family_id)
    /// };
    /// registry.load_from_config("config/capability_registry.toml", socket_resolver)?;
    /// ```
    pub fn load_from_config<F>(
        &mut self,
        config_path: impl AsRef<std::path::Path>,
        socket_resolver: F,
    ) -> Result<usize>
    where
        F: Fn(&str, &str) -> String,
    {
        let config_content = std::fs::read_to_string(config_path.as_ref())
            .map_err(|e| anyhow!("Failed to read capability config: {e}"))?;

        let config: toml::Value = toml::from_str(&config_content)
            .map_err(|e| anyhow!("Failed to parse capability config: {e}"))?;

        let family_id = biomeos_core::family_discovery::get_family_id();
        let mut count = 0;

        // Load all translation tables
        if let Some(translations) = config.get("translations").and_then(|t| t.as_table()) {
            for (domain, domain_translations) in translations {
                if let Some(domain_table) = domain_translations.as_table() {
                    for (semantic, translation) in domain_table {
                        if let Some(trans_table) = translation.as_table() {
                            let provider = trans_table
                                .get("provider")
                                .and_then(|v| v.as_str())
                                .unwrap_or_default();
                            let method = trans_table
                                .get("method")
                                .and_then(|v| v.as_str())
                                .unwrap_or(semantic);

                            if !provider.is_empty() {
                                let socket = socket_resolver(provider, &family_id);
                                self.register_translation(
                                    semantic.clone(),
                                    provider,
                                    method,
                                    socket,
                                    None,
                                );
                                count += 1;
                            }
                        }
                    }
                }
                debug!("📦 Loaded {} translations from domain '{}'", count, domain);
            }
        }

        info!("📚 Loaded {} capability translations from config", count);
        Ok(count)
    }

    /// Load default translations with automatic socket resolution
    ///
    /// Socket resolution:
    /// 1. `$PRIMAL_SOCKET` environment variable (primal-specific override)
    /// 2. `SystemPaths::new_lazy().primal_socket()` (XDG-compliant)
    ///
    /// ## Capability-Based Provider Resolution
    ///
    /// Providers are resolved using environment variable overrides first:
    /// - `BIOMEOS_SECURITY_PROVIDER`: Primal providing security capabilities
    /// - `BIOMEOS_NETWORK_PROVIDER`: Primal providing network capabilities
    /// - `BIOMEOS_STORAGE_PROVIDER`: Primal providing storage capabilities
    /// - `BIOMEOS_COMPUTE_PROVIDER`: Primal providing compute capabilities
    /// - `BIOMEOS_AI_PROVIDER`: Primal providing AI capabilities
    ///
    /// This enables runtime capability substitution without code changes.
    pub fn load_defaults(&mut self) -> usize {
        let family_id = biomeos_core::family_discovery::get_family_id();
        let mut count = 0;

        // DEEP DEBT EVOLUTION: Provider resolution is ENV-FIRST.
        // In strict discovery mode, providers MUST be explicitly configured.
        // Bootstrap defaults only used when BIOMEOS_STRICT_DISCOVERY is not set.
        let strict = std::env::var("BIOMEOS_STRICT_DISCOVERY").is_ok();

        let resolve_provider = |env_key: &str, default: &str| -> String {
            match std::env::var(env_key) {
                Ok(v) => v,
                Err(_) if strict => {
                    tracing::warn!(
                        "BIOMEOS_STRICT_DISCOVERY: {} not set, skipping provider",
                        env_key
                    );
                    String::new()
                }
                Err(_) => default.to_string(),
            }
        };

        let security_provider = resolve_provider("BIOMEOS_SECURITY_PROVIDER", BEARDOG);
        let network_provider = resolve_provider("BIOMEOS_NETWORK_PROVIDER", SONGBIRD);
        let storage_provider = resolve_provider("BIOMEOS_STORAGE_PROVIDER", NESTGATE);
        let compute_provider = resolve_provider("BIOMEOS_COMPUTE_PROVIDER", TOADSTOOL);
        let ai_provider = resolve_provider("BIOMEOS_AI_PROVIDER", SQUIRREL);

        // Define domain → provider mappings
        // The semantic capability names (left) are stable API contracts
        // The actual method names (right) are provider-specific implementations
        // Type alias defined at module level to satisfy clippy::type_complexity
        let domain_providers: DomainProviderMappings = &[
            // Security domain - cryptographic operations
            (
                BEARDOG, // Default, overridden by security_provider
                "security",
                &[
                    // Beacon genetics
                    ("beacon.generate", "beacon.generate"),
                    ("beacon.get_id", "beacon.get_id"),
                    ("beacon.get_seed", "beacon.get_seed"),
                    ("beacon.encrypt", "beacon.encrypt"),
                    ("beacon.decrypt", "beacon.decrypt"),
                    ("beacon.try_decrypt", "beacon.try_decrypt"),
                    // Core crypto
                    ("crypto.encrypt", "chacha20_poly1305_encrypt"),
                    ("crypto.decrypt", "chacha20_poly1305_decrypt"),
                    ("crypto.generate_keypair", "x25519_generate_ephemeral"),
                    ("crypto.blake3_hash", "blake3_hash"),
                    ("crypto.hmac", "hmac_sha256"),
                    ("crypto.sign", "sign_ed25519"),
                    ("crypto.verify", "verify_ed25519"),
                    // SHA3-256 for .onion address derivation (Tor v3 spec)
                    ("crypto.sha3_256", "crypto.sha3_256"),
                    ("onion.hash_checksum", "crypto.sha3_256"),
                    // Onion identity keys
                    ("onion.generate_identity", "crypto.ed25519_generate_keypair"),
                    ("onion.session_key", "crypto.x25519_generate_ephemeral"),
                    ("onion.derive_shared", "crypto.x25519_derive_secret"),
                    ("onion.encrypt", "crypto.chacha20_poly1305_encrypt"),
                    ("onion.decrypt", "crypto.chacha20_poly1305_decrypt"),
                    ("onion.hkdf_extract", "crypto.hmac_sha256"),
                    ("onion.hkdf_expand", "crypto.hmac_sha256"),
                    // JWT
                    ("security.generate_jwt", "generate_jwt_secret"),
                    // Relay authorization (BearDog verifies lineage for relay sessions)
                    ("relay.authorize", "relay.authorize"),
                ],
            ),
            // Network domain - HTTP, discovery, peer communication, mesh relay
            (
                SONGBIRD, // Default, overridden by network_provider
                "network",
                &[
                    // HTTP operations
                    ("network.beacon_exchange", "beacon_exchange"),
                    ("network.discover_peers", "discover_peers"),
                    ("network.http_request", "http_request"),
                    ("discovery.find_primals", "find_primals"),
                    // STUN operations (NAT traversal)
                    ("stun.discover", "stun.get_public_address"),
                    ("stun.detect_nat_type", "stun.detect_nat_type"),
                    // Mesh relay operations (Sovereign Beacon Mesh)
                    ("mesh.status", "mesh.status"),
                    ("mesh.find_path", "mesh.find_path"),
                    ("mesh.announce", "mesh.announce"),
                    ("mesh.peers", "mesh.list_peers"),
                    ("mesh.health_check", "mesh.health_check"),
                    // STUN advanced operations (relay-assisted punch)
                    ("stun.probe_port_pattern", "stun.probe_port_pattern"),
                    // Hole punch coordination
                    ("punch.request", "punch.request"),
                    ("punch.status", "punch.status"),
                    ("punch.coordinate", "punch.coordinate"),
                    // Relay operations
                    ("relay.serve", "relay.serve"),
                    ("relay.status", "relay.status"),
                    ("relay.allocate", "relay.allocate"),
                    // Onion service (when enabled)
                    ("onion.create_service", "onion.create_service"),
                    ("onion.get_address", "onion.get_address"),
                    ("onion.connect", "onion.connect"),
                    ("onion.status", "onion.status"),
                ],
            ),
            // Storage domain - data persistence
            (
                NESTGATE, // Default, overridden by storage_provider
                "storage",
                &[
                    ("storage.put", "storage.put"),
                    ("storage.get", "storage.get"),
                    ("storage.delete", "storage.delete"),
                    ("storage.retrieve", "storage.retrieve"),
                ],
            ),
            // Compute domain - workload execution
            (
                TOADSTOOL, // Default, overridden by compute_provider
                "compute",
                &[("compute.execute", "execute"), ("compute.parse", "parse")],
            ),
            // AI domain - machine learning and inference
            (
                SQUIRREL, // Default, overridden by ai_provider
                "ai",
                &[
                    ("ai.query", "query"),
                    ("ai.suggest", "suggest"),
                    ("mcp.call", "mcp_call"),
                ],
            ),
        ];

        // Map domain names to runtime-resolved providers
        let provider_overrides: std::collections::HashMap<&str, String> = [
            ("security", security_provider),
            ("network", network_provider),
            ("storage", storage_provider),
            ("compute", compute_provider),
            ("ai", ai_provider),
        ]
        .into_iter()
        .collect();

        for (_default_provider, domain, translations) in domain_providers {
            // Use environment-overridden provider if available, otherwise use default
            // DEEP DEBT: In strict mode, empty provider means skip this domain
            let actual_provider = provider_overrides
                .get(domain)
                .filter(|s| !s.is_empty())
                .map(|s| s.as_str())
                .unwrap_or(_default_provider);

            if actual_provider.is_empty() {
                tracing::debug!(
                    "Skipping domain {} (no provider configured in strict mode)",
                    domain
                );
                continue;
            }

            let socket = resolve_primal_socket(actual_provider, &family_id);

            for (semantic, method) in *translations {
                self.register_translation(
                    *semantic,
                    actual_provider,
                    *method,
                    socket.clone(),
                    None,
                );
                count += 1;
            }

            debug!(
                "📦 Loaded {} default translations for {} ({})",
                translations.len(),
                domain,
                actual_provider
            );
        }

        info!("📚 Loaded {} default capability translations", count);
        count
    }
}

/// Resolve socket path for a primal
///
/// Priority:
/// 1. `$PRIMAL_SOCKET` environment variable (e.g., `$BEARDOG_SOCKET`)
/// 2. `SystemPaths::new_lazy().primal_socket()` (XDG-compliant, handles
///    `XDG_RUNTIME_DIR`, `/run/user/{uid}`, and `/tmp` fallbacks)
pub fn resolve_primal_socket(primal: &str, family_id: &str) -> String {
    // 1. Check environment variable override (primal-specific)
    let env_var = format!("{}_SOCKET", primal.to_uppercase());
    if let Ok(socket) = std::env::var(&env_var) {
        return socket;
    }

    // 2. XDG-compliant resolution via SystemPaths
    let primal_id = format!("{primal}-{family_id}");
    biomeos_types::paths::SystemPaths::new_lazy()
        .primal_socket(&primal_id)
        .to_string_lossy()
        .to_string()
}

impl Default for CapabilityTranslationRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Registry statistics
#[derive(Debug, Clone, Serialize)]
pub struct RegistryStats {
    /// Total capability translations registered
    pub total_translations: usize,
    /// Number of unique primal providers
    pub total_providers: usize,
    /// Capability count per provider (provider name → count)
    pub capabilities_by_provider: HashMap<String, usize>,
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used, clippy::expect_used)]

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

    #[test]
    fn test_load_defaults() {
        let mut registry = CapabilityTranslationRegistry::new();

        let count = registry.load_defaults();

        // Should load translations for all domains
        assert!(count > 0, "Should load at least some translations");

        // Check security domain translations exist
        assert!(
            registry.has_capability("beacon.generate"),
            "Should have beacon.generate"
        );
        assert!(
            registry.has_capability("crypto.encrypt"),
            "Should have crypto.encrypt"
        );

        // Check network domain translations exist
        assert!(
            registry.has_capability("network.beacon_exchange"),
            "Should have network.beacon_exchange"
        );

        // Check storage domain translations exist
        assert!(
            registry.has_capability("storage.put"),
            "Should have storage.put"
        );

        // Check AI domain translations exist
        assert!(registry.has_capability("ai.query"), "Should have ai.query");

        // Verify provider mappings
        let beardog_caps = registry.provider_capabilities("beardog");
        assert!(!beardog_caps.is_empty(), "BearDog should have capabilities");
        assert!(
            beardog_caps.contains(&"beacon.generate".to_string()),
            "BearDog should provide beacon.generate"
        );
    }

    /// NOTE: Environment variable tests are inherently flaky when run in parallel
    /// because `std::env` is process-global. These tests verify the resolution
    /// priority logic using a unique primal name to avoid collisions.
    #[test]
    fn test_resolve_primal_socket_env_override() {
        // Use a unique primal name to avoid collision with other tests
        let unique_primal = "testprimal_env_override";
        let env_var = format!("{}_SOCKET", unique_primal.to_uppercase());

        // Set environment variable override
        std::env::set_var(&env_var, "/custom/unique-test.sock");

        let socket = resolve_primal_socket(unique_primal, "test-family");
        assert_eq!(socket, "/custom/unique-test.sock");

        // Clean up
        std::env::remove_var(&env_var);
    }

    #[test]
    fn test_resolve_primal_socket_fallback() {
        // Use a unique primal name that won't have env override
        let unique_primal = "testprimal_fallback";

        let socket = resolve_primal_socket(unique_primal, "test-family");

        // Should contain the primal name and family ID regardless of path tier
        assert!(
            socket.contains(unique_primal),
            "Socket should contain primal name"
        );
        assert!(
            socket.contains("test-family"),
            "Socket should contain family ID"
        );
        assert!(socket.ends_with(".sock"), "Socket should end with .sock");
    }

    #[test]
    fn test_resolve_primal_socket_different_primals() {
        // Ensure no environment overrides
        std::env::remove_var("SONGBIRD_SOCKET");
        std::env::remove_var("NESTGATE_SOCKET");

        let songbird = resolve_primal_socket("songbird", "fam1");
        let nestgate = resolve_primal_socket("nestgate", "fam1");

        // Sockets should be different
        assert_ne!(songbird, nestgate);

        // Each should contain the primal name
        assert!(songbird.contains("songbird"));
        assert!(nestgate.contains("nestgate"));
    }

    #[test]
    fn test_registry_default_impl() {
        let registry = CapabilityTranslationRegistry::default();

        // Should be empty initially
        assert_eq!(registry.stats().total_translations, 0);
    }

    #[test]
    fn test_translation_with_param_mappings() {
        let mut registry = CapabilityTranslationRegistry::new();

        let mut param_mappings = HashMap::new();
        param_mappings.insert("private_key".to_string(), "our_secret".to_string());
        param_mappings.insert("public_key".to_string(), "their_public".to_string());

        registry.register_translation(
            "crypto.ecdh_derive",
            "beardog",
            "x25519_derive_secret",
            "/tmp/beardog.sock",
            Some(param_mappings),
        );

        let translation = registry.get_translation("crypto.ecdh_derive").unwrap();
        assert_eq!(
            translation.param_mappings.get("private_key"),
            Some(&"our_secret".to_string())
        );
        assert_eq!(
            translation.param_mappings.get("public_key"),
            Some(&"their_public".to_string())
        );
    }

    #[test]
    fn test_get_translation_unknown_capability() {
        let registry = CapabilityTranslationRegistry::new();
        assert!(registry.get_translation("nonexistent.capability").is_none());
        assert!(!registry.has_capability("nonexistent.capability"));
    }

    #[tokio::test]
    async fn test_call_capability_no_provider() {
        let registry = CapabilityTranslationRegistry::new();

        let result = registry
            .call_capability("unknown.capability", serde_json::json!({}))
            .await;

        assert!(result.is_err());
        assert!(result
            .unwrap_err()
            .to_string()
            .contains("No provider for capability"));
    }

    #[tokio::test]
    async fn test_call_capability_socket_connection_fails() {
        let mut registry = CapabilityTranslationRegistry::new();
        registry.register_translation(
            "test.fake_call",
            "fake_primal",
            "fake_method",
            "/nonexistent/path/does-not-exist-12345.sock",
            None,
        );

        let result = registry
            .call_capability("test.fake_call", serde_json::json!({}))
            .await;

        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(
            err_msg.contains("Provider")
                || err_msg.contains("connect")
                || err_msg.contains("socket"),
            "Expected provider/connection error, got: {err_msg}"
        );
    }

    #[test]
    fn test_capability_translation_struct() {
        let mut param_mappings = HashMap::new();
        param_mappings.insert("a".to_string(), "b".to_string());

        let translation = CapabilityTranslation {
            semantic: "test.semantic".to_string(),
            provider: "beardog".to_string(),
            actual_method: "actual_method".to_string(),
            socket: "/tmp/beardog.sock".to_string(),
            param_mappings: param_mappings.clone(),
            metadata: HashMap::new(),
        };

        assert_eq!(translation.semantic, "test.semantic");
        assert_eq!(translation.provider, "beardog");
        assert_eq!(translation.param_mappings.get("a"), Some(&"b".to_string()));
    }

    #[test]
    fn test_capability_translation_serde() {
        let translation = CapabilityTranslation {
            semantic: "crypto.encrypt".to_string(),
            provider: "beardog".to_string(),
            actual_method: "chacha20_encrypt".to_string(),
            socket: "/tmp/b.sock".to_string(),
            param_mappings: HashMap::new(),
            metadata: HashMap::new(),
        };

        let json = serde_json::to_string(&translation).expect("serialize");
        assert!(json.contains("crypto.encrypt"));
        let parsed: CapabilityTranslation = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.semantic, translation.semantic);
    }

    #[test]
    fn test_registry_stats_struct() {
        let mut registry = CapabilityTranslationRegistry::new();
        registry.register_translation("a", "p1", "m1", "/tmp/1.sock", None);
        registry.register_translation("b", "p1", "m2", "/tmp/1.sock", None);
        registry.register_translation("c", "p2", "m3", "/tmp/2.sock", None);

        let stats = registry.stats();
        assert_eq!(stats.total_translations, 3);
        assert_eq!(stats.total_providers, 2);
        assert_eq!(stats.capabilities_by_provider["p1"], 2);
        assert_eq!(stats.capabilities_by_provider["p2"], 1);
    }

    #[test]
    fn test_registry_stats_serialization() {
        let stats = RegistryStats {
            total_translations: 10,
            total_providers: 3,
            capabilities_by_provider: [("a".to_string(), 5), ("b".to_string(), 3)]
                .into_iter()
                .collect(),
        };
        let json = serde_json::to_string(&stats).expect("serialize");
        assert!(json.contains("10"));
        assert!(json.contains("3"));
    }
}
