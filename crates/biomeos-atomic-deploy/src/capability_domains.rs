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
        capabilities: &["compute", "execution", "parsing"],
    },
    // AI domain (Squirrel)
    CapabilityDomain {
        provider: "squirrel",
        capabilities: &["ai", "mcp", "assistance", "ml"],
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
        // Compute capabilities should map to toadstool
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
    fn test_capability_to_provider_unknown() {
        // Unknown capabilities should return None
        assert_eq!(capability_to_provider_fallback("unknown"), None);
        assert_eq!(capability_to_provider_fallback("random.capability"), None);
        assert_eq!(capability_to_provider_fallback(""), None);
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
    }
}
