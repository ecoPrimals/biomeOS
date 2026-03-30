// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Capability Domain Mappings
//!
//! Two-tier resolution: runtime `CapabilityRegistry` loaded from
//! `config/capability_registry.toml`, with a compiled-in const fallback
//! (`CAPABILITY_DOMAINS`) for zero-config environments.
//!
//! # Design Principle
//!
//! biomeOS orchestrates, primals execute primitives.
//! The mapping is SEMANTIC → PROVIDER (not implementation-specific).

use biomeos_types::primal_names::{
    BEARDOG, LOAMSPINE, NESTGATE, RHIZOCRYPT, SONGBIRD, SQUIRREL, SWEETGRASS, TOADSTOOL,
};
use std::collections::HashMap;
use std::path::Path;

/// Capability domain configuration
/// Loaded from `config/capability_registry.toml` in production
pub struct CapabilityDomain {
    pub provider: &'static str,
    pub capabilities: &'static [&'static str],
}

/// Default capability domains for fallback resolution
/// These align with `config/capability_registry.toml`
pub const CAPABILITY_DOMAINS: &[CapabilityDomain] = &[
    // Security domain (BearDog)
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
    // Network domain (Songbird)
    CapabilityDomain {
        provider: SONGBIRD,
        capabilities: &[
            "discovery",
            "http",
            "orchestration",
            "federation",
            "network",
        ],
    },
    // Storage domain (NestGate)
    CapabilityDomain {
        provider: NESTGATE,
        capabilities: &["storage", "versioning", "persistence"],
    },
    // Compute domain (ToadStool)
    CapabilityDomain {
        provider: TOADSTOOL,
        capabilities: &["compute", "execution", "parsing", "hardware_learning"],
    },
    // AI domain (Squirrel)
    CapabilityDomain {
        provider: SQUIRREL,
        capabilities: &["ai", "mcp", "assistance", "ml"],
    },
    // Data domain (NestGate live providers)
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
    // Science domain (wetSpring)
    CapabilityDomain {
        provider: "wetspring",
        capabilities: &[
            "science",
            "biodiversity",
            "spectral",
            "metagenomics",
            "kinetics",
            "monitoring",
        ],
    },
    // Neural science domain (neuralSpring)
    CapabilityDomain {
        provider: "neuralspring",
        capabilities: &[
            "spectral_analysis",
            "anderson_localization",
            "hessian_eigenanalysis",
            "agent_coordination",
            "training_trajectory",
        ],
    },
    // Ecology domain (airSpring)
    CapabilityDomain {
        provider: "airspring",
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
    // Game science domain (ludoSpring)
    CapabilityDomain {
        provider: "ludospring",
        capabilities: &[
            "game",
            "ludology",
            "interaction_design",
            "procedural_generation",
            "accessibility_scoring",
            "engagement_metrics",
        ],
    },
    // Visualization domain (petalTongue)
    CapabilityDomain {
        provider: "petaltongue",
        capabilities: &[
            "visualization",
            "ui",
            "interaction",
            "representation",
            "sensor_stream",
        ],
    },
    // XR / Immersive domain (petalTongue + ludoSpring)
    CapabilityDomain {
        provider: "petaltongue",
        capabilities: &["xr", "stereo", "vr", "ar", "tracking", "haptic", "mocap"],
    },
    // Medical / Surgical domain (healthSpring)
    CapabilityDomain {
        provider: "healthspring",
        capabilities: &[
            "medical",
            "surgical",
            "anatomy",
            "tissue",
            "biosignal",
            "pharmacokinetics",
        ],
    },
    // Ephemeral workspace domain (rhizoCrypt)
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
    // Permanent history domain (LoamSpine)
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
    // Attribution domain (sweetGrass)
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
];

/// Config-driven capability → provider registry.
///
/// Loads `[domains.*]` sections from `config/capability_registry.toml`.
/// Falls back to the compiled-in `CAPABILITY_DOMAINS` const for capabilities
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
    /// 3. Compiled-in fallback table
    pub fn resolve(&self, capability: &str) -> Option<String> {
        if let Some(provider) = self.config_map.get(capability) {
            return Some(provider.clone());
        }

        if let Some(prefix) = capability.split('.').next() {
            if let Some(provider) = self.config_map.get(prefix) {
                return Some(provider.clone());
            }
        }

        capability_to_provider_fallback(capability).map(str::to_string)
    }

    /// Number of config-loaded entries (excludes compiled-in fallback).
    #[cfg(test)]
    pub fn config_entry_count(&self) -> usize {
        self.config_map.len()
    }
}

/// Resolve capability to provider using the compiled-in domain table.
///
/// This is the lowest-priority fallback when neither the neural-api router
/// nor the TOML config have a match. Prefer `CapabilityRegistry::resolve`.
pub fn capability_to_provider_fallback(capability: &str) -> Option<&'static str> {
    for domain in CAPABILITY_DOMAINS {
        if domain.capabilities.contains(&capability) {
            return Some(domain.provider);
        }
    }

    if let Some(prefix) = capability.split('.').next() {
        for domain in CAPABILITY_DOMAINS {
            if domain.capabilities.contains(&prefix) {
                return Some(domain.provider);
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_capability_domain_struct_access() {
        let domain = &CAPABILITY_DOMAINS[0];
        assert_eq!(domain.provider, "beardog");
        assert!(domain.capabilities.contains(&"security"));
        assert!(domain.capabilities.contains(&"crypto"));
    }

    #[test]
    fn test_capability_to_provider_security_domain() {
        // Security capabilities should map to beardog
        assert_eq!(capability_to_provider_fallback("security"), Some("beardog"));
        assert_eq!(capability_to_provider_fallback("crypto"), Some("beardog"));
        assert_eq!(
            capability_to_provider_fallback("encryption"),
            Some("beardog")
        );
        assert_eq!(capability_to_provider_fallback("genetic"), Some("beardog"));
        assert_eq!(capability_to_provider_fallback("beacon"), Some("beardog"));
        assert_eq!(capability_to_provider_fallback("tls"), Some("beardog"));
        assert_eq!(capability_to_provider_fallback("jwt"), Some("beardog"));
    }

    #[test]
    fn test_capability_to_provider_network_domain() {
        // Network capabilities should map to songbird
        assert_eq!(
            capability_to_provider_fallback("discovery"),
            Some("songbird")
        );
        assert_eq!(capability_to_provider_fallback("http"), Some("songbird"));
        assert_eq!(
            capability_to_provider_fallback("orchestration"),
            Some("songbird")
        );
        assert_eq!(
            capability_to_provider_fallback("federation"),
            Some("songbird")
        );
        assert_eq!(capability_to_provider_fallback("network"), Some("songbird"));
    }

    #[test]
    fn test_capability_to_provider_storage_domain() {
        // Storage capabilities should map to nestgate
        assert_eq!(capability_to_provider_fallback("storage"), Some("nestgate"));
        assert_eq!(
            capability_to_provider_fallback("versioning"),
            Some("nestgate")
        );
        assert_eq!(
            capability_to_provider_fallback("persistence"),
            Some("nestgate")
        );
    }

    #[test]
    fn test_capability_to_provider_compute_domain() {
        assert_eq!(
            capability_to_provider_fallback("compute"),
            Some("toadstool")
        );
        assert_eq!(
            capability_to_provider_fallback("execution"),
            Some("toadstool")
        );
        assert_eq!(
            capability_to_provider_fallback("parsing"),
            Some("toadstool")
        );
        assert_eq!(
            capability_to_provider_fallback("hardware_learning"),
            Some("toadstool")
        );
        assert_eq!(
            capability_to_provider_fallback("compute.hardware.observe"),
            Some("toadstool")
        );
        assert_eq!(
            capability_to_provider_fallback("compute.hardware.apply"),
            Some("toadstool")
        );
    }

    #[test]
    fn test_capability_to_provider_ai_domain() {
        // AI capabilities should map to squirrel
        assert_eq!(capability_to_provider_fallback("ai"), Some("squirrel"));
        assert_eq!(capability_to_provider_fallback("mcp"), Some("squirrel"));
        assert_eq!(
            capability_to_provider_fallback("assistance"),
            Some("squirrel")
        );
        assert_eq!(capability_to_provider_fallback("ml"), Some("squirrel"));
    }

    #[test]
    fn test_capability_to_provider_prefix_matching() {
        // Prefix matching: "crypto.encrypt" should match "crypto" domain
        assert_eq!(
            capability_to_provider_fallback("crypto.encrypt"),
            Some("beardog")
        );
        assert_eq!(
            capability_to_provider_fallback("storage.put"),
            Some("nestgate")
        );
        assert_eq!(
            capability_to_provider_fallback("network.beacon_exchange"),
            Some("songbird")
        );
        assert_eq!(
            capability_to_provider_fallback("ai.query"),
            Some("squirrel")
        );
    }

    #[test]
    fn test_capability_to_provider_ecology_domain() {
        assert_eq!(
            capability_to_provider_fallback("ecology"),
            Some("airspring")
        );
        assert_eq!(capability_to_provider_fallback("et0"), Some("airspring"));
        assert_eq!(
            capability_to_provider_fallback("irrigation"),
            Some("airspring")
        );
        assert_eq!(
            capability_to_provider_fallback("water_balance"),
            Some("airspring")
        );
        assert_eq!(capability_to_provider_fallback("yield"), Some("airspring"));
        assert_eq!(
            capability_to_provider_fallback("agriculture"),
            Some("airspring")
        );
        assert_eq!(
            capability_to_provider_fallback("drought"),
            Some("airspring")
        );
        assert_eq!(
            capability_to_provider_fallback("statistics"),
            Some("airspring")
        );
        assert_eq!(
            capability_to_provider_fallback("ecology.et0_fao56"),
            Some("airspring")
        );
    }

    #[test]
    fn test_capability_to_provider_science_domains() {
        assert_eq!(
            capability_to_provider_fallback("science"),
            Some("wetspring")
        );
        assert_eq!(
            capability_to_provider_fallback("biodiversity"),
            Some("wetspring")
        );
        assert_eq!(
            capability_to_provider_fallback("kinetics"),
            Some("wetspring")
        );
        assert_eq!(
            capability_to_provider_fallback("monitoring"),
            Some("wetspring")
        );
        assert_eq!(
            capability_to_provider_fallback("spectral_analysis"),
            Some("neuralspring")
        );
        assert_eq!(capability_to_provider_fallback("data"), Some("nestgate"));
        assert_eq!(capability_to_provider_fallback("ncbi"), Some("nestgate"));
    }

    #[test]
    fn test_capability_to_provider_provenance_trio() {
        // Ephemeral workspace (rhizoCrypt)
        assert_eq!(
            capability_to_provider_fallback("ephemeral_workspace"),
            Some("rhizocrypt")
        );
        assert_eq!(capability_to_provider_fallback("dag"), Some("rhizocrypt"));
        assert_eq!(
            capability_to_provider_fallback("session"),
            Some("rhizocrypt")
        );
        assert_eq!(
            capability_to_provider_fallback("merkle"),
            Some("rhizocrypt")
        );
        assert_eq!(
            capability_to_provider_fallback("dehydration"),
            Some("rhizocrypt")
        );
        assert_eq!(capability_to_provider_fallback("slice"), Some("rhizocrypt"));
        assert_eq!(
            capability_to_provider_fallback("dag.create_session"),
            Some("rhizocrypt")
        );

        // Permanent history (LoamSpine)
        assert_eq!(
            capability_to_provider_fallback("permanent_storage"),
            Some("loamspine")
        );
        assert_eq!(
            capability_to_provider_fallback("linear_history"),
            Some("loamspine")
        );
        assert_eq!(capability_to_provider_fallback("spine"), Some("loamspine"));
        assert_eq!(
            capability_to_provider_fallback("certificate"),
            Some("loamspine")
        );
        assert_eq!(capability_to_provider_fallback("commit"), Some("loamspine"));
        assert_eq!(
            capability_to_provider_fallback("commit.session"),
            Some("loamspine")
        );

        // Attribution (sweetGrass)
        assert_eq!(
            capability_to_provider_fallback("attribution"),
            Some("sweetgrass")
        );
        assert_eq!(capability_to_provider_fallback("braid"), Some("sweetgrass"));
        assert_eq!(
            capability_to_provider_fallback("provenance"),
            Some("sweetgrass")
        );
        assert_eq!(
            capability_to_provider_fallback("contribution"),
            Some("sweetgrass")
        );
        assert_eq!(
            capability_to_provider_fallback("privacy"),
            Some("sweetgrass")
        );
        assert_eq!(
            capability_to_provider_fallback("provenance.create_braid"),
            Some("sweetgrass")
        );
    }

    #[test]
    fn test_capability_to_provider_unknown() {
        assert_eq!(capability_to_provider_fallback("unknown"), None);
        assert_eq!(capability_to_provider_fallback("random.capability"), None);
        assert_eq!(capability_to_provider_fallback(""), None);
    }

    #[test]
    fn test_capability_to_provider_xr_domain() {
        assert_eq!(capability_to_provider_fallback("xr"), Some("petaltongue"));
        assert_eq!(
            capability_to_provider_fallback("stereo"),
            Some("petaltongue")
        );
        assert_eq!(capability_to_provider_fallback("vr"), Some("petaltongue"));
        assert_eq!(
            capability_to_provider_fallback("tracking"),
            Some("petaltongue")
        );
        assert_eq!(
            capability_to_provider_fallback("haptic"),
            Some("petaltongue")
        );
        assert_eq!(
            capability_to_provider_fallback("mocap"),
            Some("petaltongue")
        );
        assert_eq!(
            capability_to_provider_fallback("xr.negotiate_stereo"),
            Some("petaltongue")
        );
    }

    #[test]
    fn test_capability_to_provider_medical_domain() {
        assert_eq!(
            capability_to_provider_fallback("medical"),
            Some("healthspring")
        );
        assert_eq!(
            capability_to_provider_fallback("surgical"),
            Some("healthspring")
        );
        assert_eq!(
            capability_to_provider_fallback("anatomy"),
            Some("healthspring")
        );
        assert_eq!(
            capability_to_provider_fallback("tissue"),
            Some("healthspring")
        );
        assert_eq!(
            capability_to_provider_fallback("biosignal"),
            Some("healthspring")
        );
        assert_eq!(
            capability_to_provider_fallback("pharmacokinetics"),
            Some("healthspring")
        );
        assert_eq!(
            capability_to_provider_fallback("medical.load_anatomy"),
            Some("healthspring")
        );
    }

    #[test]
    fn test_capability_domains_structure() {
        // Verify CAPABILITY_DOMAINS is properly structured
        assert!(!CAPABILITY_DOMAINS.is_empty(), "Should have domains");

        // Each domain should have a non-empty provider and capabilities
        for domain in CAPABILITY_DOMAINS {
            assert!(!domain.provider.is_empty(), "Provider should not be empty");
            assert!(
                !domain.capabilities.is_empty(),
                "Capabilities should not be empty"
            );
        }

        // Verify expected domains exist
        let providers: Vec<&str> = CAPABILITY_DOMAINS.iter().map(|d| d.provider).collect();
        assert!(providers.contains(&"beardog"));
        assert!(providers.contains(&"songbird"));
        assert!(providers.contains(&"nestgate"));
        assert!(providers.contains(&"toadstool"));
        assert!(providers.contains(&"squirrel"));
        assert!(providers.contains(&"wetspring"));
        assert!(providers.contains(&"neuralspring"));
        assert!(providers.contains(&"airspring"));
        assert!(providers.contains(&"ludospring"));
        assert!(providers.contains(&"petaltongue"));
        assert!(providers.contains(&"healthspring"));
        assert!(providers.contains(&"rhizocrypt"));
        assert!(providers.contains(&"loamspine"));
        assert!(providers.contains(&"sweetgrass"));
    }

    #[test]
    fn test_capability_to_provider_game_domain() {
        assert_eq!(capability_to_provider_fallback("game"), Some("ludospring"));
        assert_eq!(
            capability_to_provider_fallback("ludology"),
            Some("ludospring")
        );
        assert_eq!(
            capability_to_provider_fallback("interaction_design"),
            Some("ludospring")
        );
        assert_eq!(
            capability_to_provider_fallback("procedural_generation"),
            Some("ludospring")
        );
        assert_eq!(
            capability_to_provider_fallback("accessibility_scoring"),
            Some("ludospring")
        );
        assert_eq!(
            capability_to_provider_fallback("engagement_metrics"),
            Some("ludospring")
        );
        assert_eq!(
            capability_to_provider_fallback("game.analyze_ui"),
            Some("ludospring")
        );
        assert_eq!(
            capability_to_provider_fallback("game.evaluate_flow"),
            Some("ludospring")
        );
    }

    #[test]
    fn test_capability_to_provider_petaltongue_domain() {
        assert_eq!(
            capability_to_provider_fallback("visualization"),
            Some("petaltongue")
        );
        assert_eq!(capability_to_provider_fallback("ui"), Some("petaltongue"));
        assert_eq!(
            capability_to_provider_fallback("interaction"),
            Some("petaltongue")
        );
        assert_eq!(
            capability_to_provider_fallback("representation"),
            Some("petaltongue")
        );
        assert_eq!(
            capability_to_provider_fallback("visualization.render"),
            Some("petaltongue")
        );
        assert_eq!(
            capability_to_provider_fallback("ui.render"),
            Some("petaltongue")
        );
        assert_eq!(
            capability_to_provider_fallback("sensor_stream"),
            Some("petaltongue")
        );
    }

    #[test]
    fn test_capability_registry_from_toml() {
        let toml_content = r#"
[metadata]
version = "1.0.0"

[domains.security]
provider = "beardog"
capabilities = ["crypto", "encryption", "security"]

[domains.network]
provider = "songbird"
capabilities = ["discovery", "http"]
"#;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("test_registry.toml");
        std::fs::write(&path, toml_content).unwrap();

        let registry = CapabilityRegistry::from_toml(&path).unwrap();
        assert_eq!(registry.config_entry_count(), 5);
        assert_eq!(registry.resolve("crypto"), Some("beardog".into()));
        assert_eq!(registry.resolve("discovery"), Some("songbird".into()));
        assert_eq!(registry.resolve("crypto.encrypt"), Some("beardog".into()));
    }

    #[test]
    fn test_capability_registry_falls_back_to_const() {
        let registry = CapabilityRegistry::default();
        assert_eq!(registry.config_entry_count(), 0);
        assert_eq!(registry.resolve("security"), Some("beardog".into()));
        assert_eq!(registry.resolve("storage"), Some("nestgate".into()));
        assert_eq!(registry.resolve("unknown"), None);
    }

    #[test]
    fn test_capability_registry_skips_wildcard_provider() {
        let toml_content = r#"
[domains.health]
provider = "*"
capabilities = ["health.liveness", "health.readiness"]

[domains.storage]
provider = "nestgate"
capabilities = ["storage"]
"#;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("wildcard.toml");
        std::fs::write(&path, toml_content).unwrap();

        let registry = CapabilityRegistry::from_toml(&path).unwrap();
        assert_eq!(registry.config_entry_count(), 1);
        assert_eq!(registry.resolve("storage"), Some("nestgate".into()));
    }

    #[test]
    fn test_capability_registry_config_overrides_const() {
        let toml_content = r#"
[domains.security]
provider = "custom-sec-primal"
capabilities = ["security"]
"#;
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("override.toml");
        std::fs::write(&path, toml_content).unwrap();

        let registry = CapabilityRegistry::from_toml(&path).unwrap();
        assert_eq!(
            registry.resolve("security"),
            Some("custom-sec-primal".into()),
        );
    }

    #[test]
    fn test_capability_registry_from_real_config() {
        let config_path = std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("../../config/capability_registry.toml");
        if !config_path.exists() {
            eprintln!("Skipping: config/capability_registry.toml not found");
            return;
        }

        let registry = CapabilityRegistry::from_toml(&config_path).unwrap();
        assert!(
            registry.config_entry_count() > 40,
            "Real config should have 40+ capability entries, got {}",
            registry.config_entry_count()
        );
        assert_eq!(registry.resolve("crypto"), Some("beardog".into()));
        assert_eq!(registry.resolve("ecology"), Some("airspring".into()));
        assert_eq!(registry.resolve("game"), Some("ludospring".into()));
    }
}
