// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Runtime TOML loader for capability translations.
//!
//! Loads `config/capability_registry.toml` at startup, replacing the compiled
//! defaults in `defaults.rs` with a data-driven source. This decouples the
//! translation table from the binary: primals can evolve their method names
//! without recompiling biomeOS.
//!
//! Falls back to compiled defaults if the TOML file is missing or malformed.

use std::collections::HashMap;
use std::path::Path;

use serde::Deserialize;
use tracing::{debug, info, warn};

use super::CapabilityTranslationRegistry;
use super::socket;

#[derive(Debug, Deserialize)]
struct RegistryToml {
    #[serde(default)]
    translations: HashMap<String, HashMap<String, TranslationEntry>>,
    #[serde(default)]
    domains: HashMap<String, DomainEntry>,
}

#[derive(Debug, Deserialize)]
struct DomainEntry {
    #[allow(dead_code)]
    #[serde(default)]
    provider: String,
    #[serde(default)]
    ribocipher: bool,
    #[allow(dead_code)]
    #[serde(default)]
    capabilities: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct TranslationEntry {
    provider: String,
    method: String,
    #[serde(default)]
    ribocipher: bool,
}

/// Load capability translations from a TOML registry file.
///
/// Parses `[translations.<domain>]` sections from the given path.
/// Each entry maps a semantic method name to a provider and actual method.
/// Provider names are resolved via env overrides (same as compiled defaults).
///
/// Returns the number of translations loaded, or `None` if the file doesn't
/// exist or can't be parsed (caller should fall back to compiled defaults).
pub fn load_from_registry_toml(
    registry: &mut CapabilityTranslationRegistry,
    toml_path: &Path,
    family_id: &str,
) -> Option<usize> {
    if !toml_path.exists() {
        debug!(
            "capability_registry.toml not found at {}, using compiled defaults",
            toml_path.display()
        );
        return None;
    }

    let contents = match std::fs::read_to_string(toml_path) {
        Ok(c) => c,
        Err(e) => {
            warn!(
                "Failed to read {}: {e} — falling back to compiled defaults",
                toml_path.display()
            );
            return None;
        }
    };

    let parsed: RegistryToml = match toml::from_str(&contents) {
        Ok(p) => p,
        Err(e) => {
            warn!(
                "Failed to parse {}: {e} — falling back to compiled defaults",
                toml_path.display()
            );
            return None;
        }
    };

    let provider_overrides = resolve_provider_overrides();
    let mut count = 0;

    // Build domain→ribocipher lookup for inheritance
    let domain_ribocipher: HashMap<&str, bool> = parsed
        .domains
        .iter()
        .map(|(name, entry)| (name.as_str(), entry.ribocipher))
        .collect();

    for (domain, entries) in &parsed.translations {
        let domain_requires_ribocipher = domain_ribocipher
            .get(domain.as_str())
            .copied()
            .unwrap_or(false);

        for (semantic_key, entry) in entries {
            let actual_provider = provider_overrides
                .get(domain.as_str())
                .filter(|s| !s.is_empty())
                .map_or(entry.provider.as_str(), |s| s.as_str());

            let socket_path = socket::resolve_primal_socket(actual_provider, family_id);

            let use_ribocipher = entry.ribocipher || domain_requires_ribocipher;

            registry.register_translation_full(
                semantic_key,
                actual_provider,
                &entry.method,
                socket_path,
                None,
                use_ribocipher,
            );
            count += 1;
        }
    }

    info!(
        "📚 Loaded {} capability translations from {}",
        count,
        toml_path.display()
    );
    Some(count)
}

fn resolve_provider_overrides() -> HashMap<&'static str, String> {
    let resolve = |env_key: &str| -> Option<String> { std::env::var(env_key).ok() };

    let mut map = HashMap::new();
    if let Some(v) = resolve(biomeos_types::env_config::vars::SECURITY_PROVIDER) {
        map.insert("beacon", v.clone());
        map.insert("crypto", v.clone());
        map.insert("security", v.clone());
        map.insert("genetic", v);
    }
    if let Some(v) = resolve(biomeos_types::env_config::vars::NETWORK_PROVIDER) {
        map.insert("network", v.clone());
        map.insert("discovery", v.clone());
        map.insert("mesh", v);
    }
    if let Some(v) = resolve(biomeos_types::env_config::vars::STORAGE_PROVIDER) {
        map.insert("storage", v.clone());
        map.insert("content", v);
    }
    if let Some(v) = resolve("BIOMEOS_COMPUTE_PROVIDER") {
        map.insert("compute", v);
    }
    if let Some(v) = resolve("BIOMEOS_AI_PROVIDER") {
        map.insert("ai", v);
    }
    if let Some(v) = resolve("BIOMEOS_DAG_PROVIDER") {
        map.insert("dag", v);
    }
    if let Some(v) = resolve("BIOMEOS_HISTORY_PROVIDER") {
        map.insert("commit", v);
    }
    if let Some(v) = resolve("BIOMEOS_ATTRIBUTION_PROVIDER") {
        map.insert("attribution", v);
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;
    use tempfile::NamedTempFile;

    #[test]
    fn load_missing_file_returns_none() {
        let mut registry = CapabilityTranslationRegistry::new();
        let result = load_from_registry_toml(
            &mut registry,
            Path::new("/nonexistent/path.toml"),
            "test-family",
        );
        assert!(result.is_none());
    }

    #[test]
    fn load_valid_toml() {
        let toml_content = r#"
[translations.crypto]
"crypto.sign" = { provider = "beardog", method = "sign_ed25519" }
"crypto.verify" = { provider = "beardog", method = "verify_ed25519" }

[translations.storage]
"storage.put" = { provider = "nestgate", method = "storage.put" }
"#;
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(toml_content.as_bytes()).unwrap();

        let mut registry = CapabilityTranslationRegistry::new();
        let result = load_from_registry_toml(&mut registry, file.path(), "test-family");

        assert_eq!(result, Some(3));
        assert!(registry.has_capability("crypto.sign"));
        assert!(registry.has_capability("storage.put"));
    }

    #[test]
    fn load_malformed_toml_returns_none() {
        let mut file = NamedTempFile::new().unwrap();
        file.write_all(b"this is not valid toml {{{{").unwrap();

        let mut registry = CapabilityTranslationRegistry::new();
        let result = load_from_registry_toml(&mut registry, file.path(), "test-family");

        assert!(result.is_none());
    }
}
