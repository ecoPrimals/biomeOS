// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Default capability translations for Neural API.
//!
//! Defines the domain→provider mappings and semantic→actual method translations
//! used when loading defaults. Providers are resolved via environment variables
//! for runtime capability substitution.

use biomeos_types::primal_names::{BEARDOG, NESTGATE, SONGBIRD, SQUIRREL, TOADSTOOL};
use tracing::{debug, info};

use super::CapabilityTranslationRegistry;
use super::socket;

/// Method translation tuple: (semantic_name, actual_method_name)
type MethodTranslation = (&'static str, &'static str);

/// Domain provider mapping: (primal_name, domain_name, method_translations)
type DomainProvider = (&'static str, &'static str, &'static [MethodTranslation]);

/// Load default translations into the registry.
///
/// Resolves providers via environment variables (BIOMEOS_*_PROVIDER).
/// When BIOMEOS_STRICT_DISCOVERY is set, unset providers are skipped.
pub fn load_defaults_into(registry: &mut CapabilityTranslationRegistry) -> usize {
    let family_id = biomeos_core::family_discovery::get_family_id();
    let mut count = 0;

    // DEEP DEBT EVOLUTION: Provider resolution is ENV-FIRST.
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

    let domain_providers: &[DomainProvider] = &[
        // Security domain - cryptographic operations
        (
            BEARDOG,
            "security",
            &[
                ("beacon.generate", "beacon.generate"),
                ("beacon.get_id", "beacon.get_id"),
                ("beacon.get_seed", "beacon.get_seed"),
                ("beacon.encrypt", "beacon.encrypt"),
                ("beacon.decrypt", "beacon.decrypt"),
                ("beacon.try_decrypt", "beacon.try_decrypt"),
                ("crypto.encrypt", "chacha20_poly1305_encrypt"),
                ("crypto.decrypt", "chacha20_poly1305_decrypt"),
                ("crypto.generate_keypair", "x25519_generate_ephemeral"),
                ("crypto.blake3_hash", "blake3_hash"),
                ("crypto.hmac", "hmac_sha256"),
                ("crypto.sign", "sign_ed25519"),
                ("crypto.verify", "verify_ed25519"),
                ("crypto.sha3_256", "crypto.sha3_256"),
                ("onion.hash_checksum", "crypto.sha3_256"),
                ("onion.generate_identity", "crypto.ed25519_generate_keypair"),
                ("onion.session_key", "crypto.x25519_generate_ephemeral"),
                ("onion.derive_shared", "crypto.x25519_derive_secret"),
                ("onion.encrypt", "crypto.chacha20_poly1305_encrypt"),
                ("onion.decrypt", "crypto.chacha20_poly1305_decrypt"),
                ("onion.hkdf_extract", "crypto.hmac_sha256"),
                ("onion.hkdf_expand", "crypto.hmac_sha256"),
                ("security.generate_jwt", "generate_jwt_secret"),
                ("relay.authorize", "relay.authorize"),
            ],
        ),
        // Network domain
        (
            SONGBIRD,
            "network",
            &[
                ("network.beacon_exchange", "beacon_exchange"),
                ("network.discover_peers", "discover_peers"),
                ("network.http_request", "http_request"),
                ("discovery.find_primals", "find_primals"),
                ("stun.discover", "stun.get_public_address"),
                ("stun.detect_nat_type", "stun.detect_nat_type"),
                ("mesh.status", "mesh.status"),
                ("mesh.find_path", "mesh.find_path"),
                ("mesh.announce", "mesh.announce"),
                ("mesh.peers", "mesh.list_peers"),
                ("mesh.health_check", "mesh.health_check"),
                ("stun.probe_port_pattern", "stun.probe_port_pattern"),
                ("punch.request", "punch.request"),
                ("punch.status", "punch.status"),
                ("punch.coordinate", "punch.coordinate"),
                ("relay.serve", "relay.serve"),
                ("relay.status", "relay.status"),
                ("relay.allocate", "relay.allocate"),
                ("onion.create_service", "onion.create_service"),
                ("onion.get_address", "onion.get_address"),
                ("onion.connect", "onion.connect"),
                ("onion.status", "onion.status"),
            ],
        ),
        // Storage domain
        (
            NESTGATE,
            "storage",
            &[
                ("storage.put", "storage.put"),
                ("storage.get", "storage.get"),
                ("storage.delete", "storage.delete"),
                ("storage.retrieve", "storage.retrieve"),
            ],
        ),
        // Compute domain
        (
            TOADSTOOL,
            "compute",
            &[
                ("compute.execute", "execute"),
                ("compute.parse", "parse"),
                ("compute.dispatch.submit", "dispatch_binary"),
                ("compute.dispatch.status", "dispatch_status"),
                ("compute.dispatch.cancel", "dispatch_cancel"),
                ("compute.hardware.observe", "hw_learn.observe"),
                ("compute.hardware.distill", "hw_learn.distill"),
                ("compute.hardware.apply", "hw_learn.apply"),
            ],
        ),
        // AI domain
        (
            SQUIRREL,
            "ai",
            &[
                ("ai.query", "query"),
                ("ai.suggest", "suggest"),
                ("mcp.call", "mcp_call"),
            ],
        ),
    ];

    // Health domain translations (provider-agnostic, resolved to biomeOS itself)
    let health_translations: &[MethodTranslation] = &[
        ("health.ping", "health.check"),
        ("health.status", "health.check"),
    ];
    for (semantic, method) in health_translations {
        registry.register_translation(*semantic, "biomeos", *method, "local", None);
        count += 1;
    }
    debug!(
        "📦 Loaded {} health semantic aliases",
        health_translations.len()
    );

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

        let socket = socket::resolve_primal_socket(actual_provider, &family_id);

        for (semantic, method) in *translations {
            registry.register_translation(
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
