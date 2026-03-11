//! Capability Domain Mappings
//!
//! Provides fallback resolution when the capability registry
//! does not have a registered provider. These mappings are loaded from
//! configuration in production (config/capability_registry.toml).
//!
//! # Design Principle
//!
//! biomeOS orchestrates, primals execute primitives.
//! The mapping is SEMANTIC → PROVIDER (not implementation-specific).

/// Capability domain configuration
/// Loaded from config/capability_registry.toml in production
pub(crate) struct CapabilityDomain {
    pub provider: &'static str,
    pub capabilities: &'static [&'static str],
}

/// Default capability domains for fallback resolution
/// These align with config/capability_registry.toml
pub(crate) const CAPABILITY_DOMAINS: &[CapabilityDomain] = &[
    // Security domain (BearDog)
    CapabilityDomain {
        provider: "beardog",
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
        provider: "songbird",
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
        provider: "nestgate",
        capabilities: &["storage", "versioning", "persistence"],
    },
    // Compute domain (ToadStool)
    CapabilityDomain {
        provider: "toadstool",
<<<<<<< Updated upstream
        capabilities: &["compute", "execution", "parsing", "hardware"],
=======
        capabilities: &["compute", "execution", "parsing", "hardware_learning"],
>>>>>>> Stashed changes
    },
    // AI domain (Squirrel)
    CapabilityDomain {
        provider: "squirrel",
        capabilities: &["ai", "mcp", "assistance", "ml"],
    },
    // Data domain (NestGate live providers)
    CapabilityDomain {
        provider: "nestgate",
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
];

/// Resolve capability to provider using domain mappings
///
/// Returns the provider primal name if found, None otherwise.
/// This is a fallback when the capability registry doesn't have a match.
pub(crate) fn capability_to_provider_fallback(capability: &str) -> Option<&'static str> {
    // Check each domain for matching capability
    for domain in CAPABILITY_DOMAINS {
        if domain.capabilities.contains(&capability) {
            return Some(domain.provider);
        }
    }

    // Try prefix matching (e.g., "crypto.encrypt" matches "crypto" domain)
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
<<<<<<< Updated upstream
            capability_to_provider_fallback("hardware"),
            Some("toadstool")
        );
    }

    #[test]
    fn test_hardware_learning_prefix_matching() {
        // compute.hardware.* should match "compute" domain via prefix
=======
            capability_to_provider_fallback("hardware_learning"),
            Some("toadstool")
        );
>>>>>>> Stashed changes
        assert_eq!(
            capability_to_provider_fallback("compute.hardware.observe"),
            Some("toadstool")
        );
        assert_eq!(
<<<<<<< Updated upstream
            capability_to_provider_fallback("compute.hardware.distill"),
            Some("toadstool")
        );
        assert_eq!(
            capability_to_provider_fallback("compute.hardware.apply"),
            Some("toadstool")
        );
        assert_eq!(
            capability_to_provider_fallback("compute.hardware.share"),
            Some("toadstool")
        );
        assert_eq!(
            capability_to_provider_fallback("compute.hardware.status"),
            Some("toadstool")
        );
=======
            capability_to_provider_fallback("compute.hardware.apply"),
            Some("toadstool")
        );
>>>>>>> Stashed changes
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
}
