// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Capability Translation Registry for Neural API
//!
//! This module provides semantic-to-actual method translation for capability-based
//! routing. It enables primals to communicate using semantic capabilities (e.g.,
//! "crypto.encrypt") while the registry automatically translates to provider-specific
//! method names (e.g., "`chacha20_poly1305_encrypt`").
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
//! See: `specs/CAPABILITY_TRANSLATION_ARCHITECTURE.md`

mod defaults;
mod socket;

use anyhow::{Result, anyhow};
use biomeos_core::atomic_client::AtomicClient;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, trace};

pub use socket::{resolve_primal_socket, resolve_primal_socket_with};

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
    #[serde(default)]
    pub param_mappings: HashMap<String, String>,

    /// Optional metadata
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Capability Translation Registry
#[derive(Debug, Clone)]
pub struct CapabilityTranslationRegistry {
    translations: HashMap<String, CapabilityTranslation>,
    provider_capabilities: HashMap<String, Vec<String>>,
    _next_id: std::sync::Arc<std::sync::atomic::AtomicU64>,
}

impl CapabilityTranslationRegistry {
    /// Create new empty registry
    #[must_use]
    pub fn new() -> Self {
        Self {
            translations: HashMap::new(),
            provider_capabilities: HashMap::new(),
            _next_id: std::sync::Arc::new(std::sync::atomic::AtomicU64::new(1)),
        }
    }

    /// Register a capability translation
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

        self.translations.insert(semantic.clone(), translation);
        self.provider_capabilities
            .entry(provider)
            .or_default()
            .push(semantic);
    }

    /// Get translation for a semantic capability
    #[must_use]
    pub fn get_translation(&self, semantic: &str) -> Option<&CapabilityTranslation> {
        self.translations.get(semantic)
    }

    /// Check if capability is available
    #[must_use]
    pub fn has_capability(&self, semantic: &str) -> bool {
        self.translations.contains_key(semantic)
    }

    /// List all capabilities provided by a specific provider
    #[must_use]
    pub fn provider_capabilities(&self, provider: &str) -> Vec<String> {
        self.provider_capabilities
            .get(provider)
            .cloned()
            .unwrap_or_default()
    }

    /// List all translations
    #[must_use]
    pub fn list_all(&self) -> Vec<&CapabilityTranslation> {
        self.translations.values().collect()
    }

    /// List translations for a specific capability domain.
    ///
    /// Returns `(semantic_name, actual_method)` pairs for all translations
    /// whose semantic name starts with the given prefix (e.g. `"crypto"` matches
    /// `"crypto.sha256"`, `"crypto.sign"`).
    #[must_use]
    pub fn list_translations(&self, domain: &str) -> Option<Vec<(String, String)>> {
        let prefix = format!("{domain}.");
        let matches: Vec<(String, String)> = self
            .translations
            .iter()
            .filter(|(k, _)| k.starts_with(&prefix) || k == &domain)
            .map(|(k, v)| (k.clone(), v.actual_method.clone()))
            .collect();
        if matches.is_empty() {
            None
        } else {
            Some(matches)
        }
    }

    /// Call a capability with automatic translation
    pub async fn call_capability(
        &self,
        semantic: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value> {
        let translation = self
            .get_translation(semantic)
            .ok_or_else(|| anyhow!("No provider for capability: {semantic}"))?;

        info!(
            "🔄 Translating {} → {} (provider: {}, socket: {})",
            semantic, translation.actual_method, translation.provider, translation.socket
        );
        debug!("   Params (semantic): {}", params);

        let mapped_params = if !translation.param_mappings.is_empty() {
            if let serde_json::Value::Object(obj) = params {
                let mut mapped_obj = serde_json::Map::new();
                for (key, value) in obj {
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

        let client = AtomicClient::unix(&translation.socket);

        info!(
            "→ Provider RPC: method={}, socket={}",
            translation.actual_method, translation.socket
        );
        trace!("→ Params: {}", mapped_params);

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

        Ok(result)
    }

    /// Get statistics about the registry
    #[must_use]
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
    pub fn load_from_config<F>(
        &mut self,
        config_path: impl AsRef<std::path::Path>,
        socket_resolver: F,
    ) -> Result<usize>
    where
        F: Fn(&str, &str) -> String,
    {
        self.load_from_config_for_family(config_path, socket_resolver, None)
    }

    /// Load translations from a TOML configuration file, using an explicit family_id.
    pub fn load_from_config_for_family<F>(
        &mut self,
        config_path: impl AsRef<std::path::Path>,
        socket_resolver: F,
        family_id_override: Option<&str>,
    ) -> Result<usize>
    where
        F: Fn(&str, &str) -> String,
    {
        let config_content = std::fs::read_to_string(config_path.as_ref())
            .map_err(|e| anyhow!("Failed to read capability config: {e}"))?;

        let config: toml::Value = toml::from_str(&config_content)
            .map_err(|e| anyhow!("Failed to parse capability config: {e}"))?;

        let family_id = family_id_override
            .map(String::from)
            .unwrap_or_else(biomeos_core::family_discovery::get_family_id);
        let mut count = 0;

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
    pub fn load_defaults(&mut self) -> usize {
        defaults::load_defaults_into(self)
    }

    /// Load defaults using an explicit family_id (avoids env/file discovery).
    pub fn load_defaults_for_family(&mut self, family_id: &str) -> usize {
        defaults::load_defaults_into_for_family(self, family_id)
    }

    /// [`load_defaults`](Self::load_defaults) with per-call environment overrides (for tests).
    pub fn load_defaults_with(
        &mut self,
        env_overrides: &std::collections::HashMap<&str, Option<&str>>,
    ) -> usize {
        defaults::load_defaults_into_with(self, env_overrides)
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
    /// Total capability translations registered
    pub total_translations: usize,
    /// Number of unique primal providers
    pub total_providers: usize,
    /// Capability count per provider
    pub capabilities_by_provider: HashMap<String, usize>,
}
