// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Capability Domain Mappings
//!
//! Three-tier resolution:
//! 1. Runtime `RUNTIME_CAPABILITY_REGISTRY` populated by live capability advertisements
//! 2. Config-driven `CapabilityRegistry` loaded from `config/capability_registry.toml`
//! 3. Compiled-in bootstrap hints (`BOOTSTRAP_CAPABILITY_HINTS`) as last-resort fallback
//!
//! # Design Principle
//!
//! Primals discover each other at runtime via capability advertisement, not via
//! compiled name tables. The bootstrap table exists only for zero-config cold start.
//!
//! biomeOS orchestrates, primals execute primitives.
//! The mapping is SEMANTIC → PROVIDER (not implementation-specific).

use biomeos_types::primal_names::{
    AIRSPRING, BARRACUDA, BEARDOG, BIOMEOS, CORALREEF, HEALTHSPRING, LOAMSPINE, LUDOSPRING,
    NESTGATE, NEURALSPRING, PETALTONGUE, RHIZOCRYPT, SONGBIRD, SQUIRREL, SWEETGRASS, TOADSTOOL,
    WETSPRING,
};
use dashmap::DashMap;
use std::collections::HashMap;
use std::path::Path;
use std::sync::LazyLock;

/// Lock-free runtime registry: capability name → advertising provider primal.
///
/// Populated by socket discovery and `discovery.register_capability` advertisements.
/// Takes precedence over compiled bootstrap hints.
static RUNTIME_CAPABILITY_REGISTRY: LazyLock<DashMap<String, String>> = LazyLock::new(DashMap::new);

/// Capability domain configuration for bootstrap hints.
pub struct CapabilityDomain {
    pub provider: &'static str,
    pub capabilities: &'static [&'static str],
}

/// Last-resort compile-time capability → provider hints.
///
/// Used only when live discovery and TOML config have no match. Primal names here
/// are **fallback-only** — runtime advertisements from socket discovery are authoritative.
pub const BOOTSTRAP_CAPABILITY_HINTS: &[CapabilityDomain] = &[
    // Security domain — fallback-only provider: BearDog
    CapabilityDomain {
        provider: BEARDOG,
        capabilities: &[
            "security",
            "crypto",
            "encryption",
            "genetic",
            "beacon",
            "tls",
            "jwt",
        ],
    },
    // Network domain — fallback-only provider: Songbird
    CapabilityDomain {
        provider: SONGBIRD,
        capabilities: &[
            "discovery",
            "http",
            "orchestration",
            "federation",
            "network",
            "mesh",
            "relay",
            "punch",
            "stun",
            "onion",
        ],
    },
    // Storage domain — fallback-only provider: NestGate
    CapabilityDomain {
        provider: NESTGATE,
        capabilities: &["storage", "versioning", "persistence"],
    },
    // Content domain — fallback-only provider: NestGate
    CapabilityDomain {
        provider: NESTGATE,
        capabilities: &["content", "content_addressed", "publishing"],
    },
    // Compute domain — fallback-only provider: ToadStool
    CapabilityDomain {
        provider: TOADSTOOL,
        capabilities: &["compute", "execution", "parsing", "hardware_learning"],
    },
    // GPU math/tensor/stats domain — fallback-only provider: barraCuda
    CapabilityDomain {
        provider: BARRACUDA,
        capabilities: &["math", "tensor", "stats", "noise", "activation", "rng"],
    },
    // Shader compilation domain — fallback-only provider: coralReef
    CapabilityDomain {
        provider: CORALREEF,
        capabilities: &["shader", "wgsl", "spirv"],
    },
    // AI domain — fallback-only provider: Squirrel
    CapabilityDomain {
        provider: SQUIRREL,
        capabilities: &["ai", "mcp", "assistance", "ml"],
    },
    // Data domain — fallback-only provider: NestGate
    CapabilityDomain {
        provider: NESTGATE,
        capabilities: &[
            "data",
            "ncbi",
            "noaa",
            "iris",
            "weather_data",
            "seismic_data",
        ],
    },
    // Science domain — fallback-only provider: wetSpring
    CapabilityDomain {
        provider: WETSPRING,
        capabilities: &[
            "science",
            "biodiversity",
            "spectral",
            "metagenomics",
            "kinetics",
            "monitoring",
        ],
    },
    // Neural science domain — fallback-only provider: neuralSpring
    CapabilityDomain {
        provider: NEURALSPRING,
        capabilities: &[
            "spectral_analysis",
            "anderson_localization",
            "hessian_eigenanalysis",
            "agent_coordination",
            "training_trajectory",
        ],
    },
    // Ecology domain — fallback-only provider: airSpring
    CapabilityDomain {
        provider: AIRSPRING,
        capabilities: &[
            "ecology",
            "et0",
            "irrigation",
            "water_balance",
            "yield",
            "agriculture",
            "soil_science",
            "drought",
            "statistics",
        ],
    },
    // Game science domain — fallback-only provider: ludoSpring
    CapabilityDomain {
        provider: LUDOSPRING,
        capabilities: &[
            "game",
            "ludology",
            "interaction_design",
            "procedural_generation",
            "accessibility_scoring",
            "engagement_metrics",
        ],
    },
    // Visualization domain — fallback-only provider: petalTongue
    CapabilityDomain {
        provider: PETALTONGUE,
        capabilities: &[
            "visualization",
            "ui",
            "interaction",
            "representation",
            "sensor_stream",
        ],
    },
    // XR / Immersive domain — fallback-only provider: petalTongue
    CapabilityDomain {
        provider: PETALTONGUE,
        capabilities: &["xr", "stereo", "vr", "ar", "tracking", "haptic", "mocap"],
    },
    // Medical / Surgical domain — fallback-only provider: healthSpring
    CapabilityDomain {
        provider: HEALTHSPRING,
        capabilities: &[
            "medical",
            "surgical",
            "anatomy",
            "tissue",
            "biosignal",
            "pharmacokinetics",
        ],
    },
    // Ephemeral workspace domain — fallback-only provider: rhizoCrypt
    CapabilityDomain {
        provider: RHIZOCRYPT,
        capabilities: &[
            "ephemeral_workspace",
            "dag",
            "session",
            "merkle",
            "dehydration",
            "slice",
            "vertex",
        ],
    },
    // Permanent history domain — fallback-only provider: LoamSpine
    CapabilityDomain {
        provider: LOAMSPINE,
        capabilities: &[
            "permanent_storage",
            "linear_history",
            "spine",
            "certificate",
            "temporal_anchor",
            "commit",
        ],
    },
    // Attribution domain — fallback-only provider: sweetGrass
    CapabilityDomain {
        provider: SWEETGRASS,
        capabilities: &[
            "attribution",
            "braid",
            "provenance",
            "contribution",
            "privacy",
            "prov_export",
        ],
    },
    // Composition health domain — fallback-only provider: biomeOS
    CapabilityDomain {
        provider: BIOMEOS,
        capabilities: &["composition"],
    },
];

/// Register a capability → provider mapping from live discovery.
///
/// Called when primals advertise capabilities via socket discovery or
/// `discovery.register_capability`. Also seeds the domain prefix (first segment
/// of dotted capabilities) when not already present.
pub fn register_capability_provider(capability: &str, provider: &str) {
    if capability.is_empty() || provider.is_empty() {
        return;
    }

    let provider_owned = provider.to_string();
    RUNTIME_CAPABILITY_REGISTRY.insert(capability.to_string(), provider_owned.clone());

    if let Some((prefix, rest)) = capability.split_once('.') {
        if !rest.is_empty() {
            RUNTIME_CAPABILITY_REGISTRY
                .entry(prefix.to_string())
                .or_insert(provider_owned);
        }
    }
}

/// Resolve capability to provider: runtime registry first, bootstrap hints second.
pub fn capability_to_provider(capability: &str) -> Option<String> {
    if let Some(provider) = RUNTIME_CAPABILITY_REGISTRY.get(capability) {
        return Some(provider.clone());
    }

    if let Some(prefix) = capability.split('.').next() {
        if prefix != capability {
            if let Some(provider) = RUNTIME_CAPABILITY_REGISTRY.get(prefix) {
                return Some(provider.clone());
            }
        }
    }

    capability_to_provider_fallback(capability).map(str::to_string)
}

/// Config-driven capability → provider registry.
///
/// Loads `[domains.*]` sections from `config/capability_registry.toml`.
/// Falls back to runtime registry and compiled bootstrap hints for capabilities
/// not found in the config.
#[derive(Debug, Clone, Default)]
pub struct CapabilityRegistry {
    /// capability name → provider primal name (from TOML config)
    config_map: HashMap<String, String>,
}

impl CapabilityRegistry {
    /// Load domain mappings from the capability registry TOML.
    ///
    /// Parses every `[domains.*]` section looking for `provider` and
    /// `capabilities` keys, building a reverse index.
    pub fn from_toml(path: &Path) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        let table: toml::Table = content.parse()?;
        let mut config_map = HashMap::new();

        if let Some(domains) = table.get("domains").and_then(|v| v.as_table()) {
            for (_domain_name, domain_value) in domains {
                if let Some(domain_table) = domain_value.as_table() {
                    let provider = domain_table
                        .get("provider")
                        .and_then(|v| v.as_str())
                        .unwrap_or_default();
                    if provider.is_empty() || provider == "*" {
                        continue;
                    }
                    if let Some(caps) = domain_table.get("capabilities").and_then(|v| v.as_array())
                    {
                        for cap in caps {
                            if let Some(cap_str) = cap.as_str() {
                                config_map.insert(cap_str.to_string(), provider.to_string());
                            }
                        }
                    }
                }
            }
        }

        Ok(Self { config_map })
    }

    /// Resolve a capability to its provider.
    ///
    /// 1. Exact match in config
    /// 2. Prefix match in config (e.g. `crypto.encrypt` → `crypto`)
    /// 3. Runtime registry + compiled bootstrap hints via [`capability_to_provider`]
    pub fn resolve(&self, capability: &str) -> Option<String> {
        if let Some(provider) = self.config_map.get(capability) {
            return Some(provider.clone());
        }

        if let Some(prefix) = capability.split('.').next() {
            if let Some(provider) = self.config_map.get(prefix) {
                return Some(provider.clone());
            }
        }

        capability_to_provider(capability)
    }

    /// Number of config-loaded entries (excludes runtime and bootstrap fallback).
    #[cfg(test)]
    pub fn config_entry_count(&self) -> usize {
        self.config_map.len()
    }
}

/// Resolve capability to provider using the compiled-in bootstrap hint table.
///
/// Lowest-priority fallback when neither live discovery nor TOML config have a
/// match. Prefer [`capability_to_provider`] for the full two-tier lookup.
pub fn capability_to_provider_fallback(capability: &str) -> Option<&'static str> {
    for domain in BOOTSTRAP_CAPABILITY_HINTS {
        if domain.capabilities.contains(&capability) {
            return Some(domain.provider);
        }
    }

    if let Some(prefix) = capability.split('.').next() {
        for domain in BOOTSTRAP_CAPABILITY_HINTS {
            if domain.capabilities.contains(&prefix) {
                return Some(domain.provider);
            }
        }
    }

    None
}

/// Clear the runtime registry (test isolation only).
#[cfg(test)]
pub(crate) fn clear_runtime_capability_registry() {
    RUNTIME_CAPABILITY_REGISTRY.clear();
}
