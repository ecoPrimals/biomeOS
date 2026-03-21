// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Integration tests for Sovereign Onion capability translations
//!
//! These tests verify that the semantic capability translations work correctly
//! for the TRUE PRIMAL pattern (Songbird → BearDog crypto delegation).
//!
//! Test categories:
//! 1. Translation registration and lookup
//! 2. Crypto capability routing
//! 3. Onion-specific translations
//! 4. Mesh and relay capability routing

#[cfg(test)]
mod tests {
    use crate::capability_translation::CapabilityTranslationRegistry;

    /// Create a registry pre-loaded with defaults
    fn create_registry_with_defaults() -> CapabilityTranslationRegistry {
        let mut registry = CapabilityTranslationRegistry::new();
        registry.load_defaults();
        registry
    }

    // =========================================================================
    // Test: Onion Crypto Translation Registration
    // =========================================================================

    #[test]
    fn test_onion_crypto_translations_registered() {
        let registry = create_registry_with_defaults();

        // SHA3-256 for .onion address (CRITICAL for Tor v3)
        assert!(
            registry.has_capability("crypto.sha3_256"),
            "SHA3-256 not registered"
        );
        assert!(
            registry.has_capability("onion.hash_checksum"),
            "onion.hash_checksum alias not registered"
        );

        // Ed25519 for identity keys
        assert!(
            registry.has_capability("onion.generate_identity"),
            "Ed25519 keypair for onion not registered"
        );

        // X25519 for session keys
        assert!(
            registry.has_capability("onion.session_key"),
            "X25519 for onion not registered"
        );
        assert!(
            registry.has_capability("onion.derive_shared"),
            "X25519 derive for onion not registered"
        );

        // ChaCha20-Poly1305 for encryption
        assert!(
            registry.has_capability("onion.encrypt"),
            "ChaCha20 encrypt for onion not registered"
        );
        assert!(
            registry.has_capability("onion.decrypt"),
            "ChaCha20 decrypt for onion not registered"
        );

        // HMAC for HKDF
        assert!(
            registry.has_capability("onion.hkdf_extract"),
            "HKDF extract not registered"
        );
        assert!(
            registry.has_capability("onion.hkdf_expand"),
            "HKDF expand not registered"
        );
    }

    // =========================================================================
    // Test: All Onion Capabilities Route to BearDog
    // =========================================================================

    #[test]
    fn test_onion_capabilities_route_to_beardog() {
        let registry = create_registry_with_defaults();

        let onion_crypto_caps = [
            "crypto.sha3_256",
            "onion.hash_checksum",
            "onion.generate_identity",
            "onion.session_key",
            "onion.derive_shared",
            "onion.encrypt",
            "onion.decrypt",
            "onion.hkdf_extract",
            "onion.hkdf_expand",
        ];

        for cap in &onion_crypto_caps {
            let translation = registry
                .get_translation(cap)
                .unwrap_or_else(|| panic!("Translation missing for {cap}"));

            assert_eq!(
                translation.provider, "beardog",
                "Capability {} should route to BearDog (TRUE PRIMAL), got {}",
                cap, translation.provider
            );
        }
    }

    // =========================================================================
    // Test: Mesh Capabilities Registered
    // =========================================================================

    #[test]
    fn test_mesh_capabilities_registered() {
        let registry = create_registry_with_defaults();

        let mesh_caps = [
            "mesh.status",
            "mesh.find_path",
            "mesh.announce",
            "mesh.peers",
            "mesh.health_check",
        ];

        for cap in &mesh_caps {
            assert!(
                registry.has_capability(cap),
                "Mesh capability {cap} not registered"
            );
        }
    }

    // =========================================================================
    // Test: Mesh Capabilities Route to Songbird
    // =========================================================================

    #[test]
    fn test_mesh_capabilities_route_to_songbird() {
        let registry = create_registry_with_defaults();

        let songbird_caps = [
            "mesh.status",
            "mesh.find_path",
            "mesh.announce",
            "mesh.peers",
            "mesh.health_check",
            "punch.request",
            "punch.status",
            "stun.discover",
            "stun.detect_nat_type",
            "relay.serve",
            "relay.status",
            "relay.allocate",
        ];

        for cap in &songbird_caps {
            let translation = registry
                .get_translation(cap)
                .unwrap_or_else(|| panic!("Translation missing for {cap}"));

            assert_eq!(
                translation.provider, "songbird",
                "Capability {} should route to Songbird, got {}",
                cap, translation.provider
            );
        }
    }

    // =========================================================================
    // Test: Hole Punch Capabilities
    // =========================================================================

    #[test]
    fn test_hole_punch_capabilities() {
        let registry = create_registry_with_defaults();

        // Hole punch request and status
        assert!(
            registry.has_capability("punch.request"),
            "punch.request not registered"
        );
        assert!(
            registry.has_capability("punch.status"),
            "punch.status not registered"
        );

        // Both should route to Songbird
        let request = registry.get_translation("punch.request").unwrap();
        let status = registry.get_translation("punch.status").unwrap();

        assert_eq!(request.provider, "songbird");
        assert_eq!(status.provider, "songbird");
    }

    // =========================================================================
    // Test: STUN Capabilities
    // =========================================================================

    #[test]
    fn test_stun_capabilities() {
        let registry = create_registry_with_defaults();

        assert!(
            registry.has_capability("stun.discover"),
            "stun.discover not registered"
        );
        assert!(
            registry.has_capability("stun.detect_nat_type"),
            "stun.detect_nat_type not registered"
        );

        let discover = registry.get_translation("stun.discover").unwrap();
        let nat_type = registry.get_translation("stun.detect_nat_type").unwrap();

        assert_eq!(discover.provider, "songbird");
        assert_eq!(nat_type.provider, "songbird");
    }

    // =========================================================================
    // Test: Relay Capabilities
    // =========================================================================

    #[test]
    fn test_relay_capabilities() {
        let registry = create_registry_with_defaults();

        let relay_caps = ["relay.serve", "relay.status", "relay.allocate"];

        for cap in &relay_caps {
            assert!(
                registry.has_capability(cap),
                "Relay capability {cap} not registered"
            );

            let translation = registry.get_translation(cap).unwrap();
            assert_eq!(
                translation.provider, "songbird",
                "Relay {cap} should route to Songbird"
            );
        }
    }

    // =========================================================================
    // Test: Provider Capability Summary
    // =========================================================================

    #[test]
    fn test_provider_capability_summary() {
        let registry = create_registry_with_defaults();

        // BearDog should have all crypto + onion crypto capabilities
        let beardog_caps = registry.provider_capabilities("beardog");

        // Expected minimum crypto capabilities
        let expected_beardog_caps = [
            "crypto.encrypt",
            "crypto.decrypt",
            "crypto.generate_keypair",
            "crypto.sha3_256",
        ];

        for cap in &expected_beardog_caps {
            assert!(
                beardog_caps.contains(&cap.to_string()),
                "BearDog missing capability: {cap}"
            );
        }

        // Songbird should have all network capabilities
        let songbird_caps = registry.provider_capabilities("songbird");

        let expected_songbird_caps = ["mesh.status", "punch.request", "stun.discover"];

        for cap in &expected_songbird_caps {
            assert!(
                songbird_caps.contains(&cap.to_string()),
                "Songbird missing capability: {cap}"
            );
        }
    }

    // =========================================================================
    // Test: Semantic to Actual Method Mapping
    // =========================================================================

    #[test]
    fn test_semantic_to_actual_mapping() {
        let registry = create_registry_with_defaults();

        // Test specific mappings
        let cases: Vec<(&str, &str)> = vec![
            ("crypto.encrypt", "chacha20_poly1305_encrypt"),
            ("crypto.decrypt", "chacha20_poly1305_decrypt"),
            ("crypto.sha3_256", "crypto.sha3_256"),
            ("onion.hash_checksum", "crypto.sha3_256"),
            ("onion.encrypt", "crypto.chacha20_poly1305_encrypt"),
            ("mesh.status", "mesh.status"),
            ("punch.request", "punch.request"),
        ];

        for (semantic, expected_actual) in cases {
            let translation = registry
                .get_translation(semantic)
                .unwrap_or_else(|| panic!("No translation for {semantic}"));

            assert_eq!(
                translation.actual_method, expected_actual,
                "Semantic {} should map to {}, got {}",
                semantic, expected_actual, translation.actual_method
            );
        }
    }

    // =========================================================================
    // Test: Domain Categorization (via provider_capabilities)
    // =========================================================================

    #[test]
    fn test_domain_categorization() {
        let registry = create_registry_with_defaults();

        // BearDog provides security/crypto capabilities
        let beardog_caps = registry.provider_capabilities("beardog");
        assert!(
            beardog_caps.iter().any(|c| c.starts_with("crypto.")),
            "BearDog should provide crypto capabilities"
        );

        // Songbird provides network/mesh capabilities
        let songbird_caps = registry.provider_capabilities("songbird");
        assert!(
            songbird_caps.iter().any(|c| c.starts_with("mesh.")),
            "Songbird should provide mesh capabilities"
        );
    }

    // =========================================================================
    // Test: TRUE PRIMAL Pattern Compliance
    // =========================================================================

    #[test]
    fn test_true_primal_pattern_compliance() {
        let registry = create_registry_with_defaults();

        // All crypto-related capabilities (including onion.) should route to BearDog
        let crypto_caps = [
            "crypto.encrypt",
            "crypto.decrypt",
            "crypto.sha3_256",
            "onion.hash_checksum",
            "onion.encrypt",
            "onion.decrypt",
            "onion.generate_identity",
            "onion.session_key",
            "onion.derive_shared",
            "onion.hkdf_extract",
            "onion.hkdf_expand",
        ];

        for cap in crypto_caps {
            if let Some(translation) = registry.get_translation(cap) {
                assert_eq!(
                    translation.provider, "beardog",
                    "Crypto capability {} violates TRUE PRIMAL - should route to BearDog, routes to {}",
                    cap, translation.provider
                );
            }
        }
    }

    // =========================================================================
    // Test: Sovereign Onion Full Capability Set
    // =========================================================================

    #[test]
    fn test_sovereign_onion_full_capability_set() {
        let registry = create_registry_with_defaults();

        // All capabilities needed for Sovereign Onion Service
        let required_caps = [
            // BearDog crypto (TRUE PRIMAL)
            ("crypto.sha3_256", "beardog"), // .onion address checksum
            ("onion.generate_identity", "beardog"), // Ed25519 identity
            ("onion.session_key", "beardog"), // X25519 session
            ("onion.derive_shared", "beardog"), // ECDH
            ("onion.encrypt", "beardog"),   // ChaCha20
            ("onion.decrypt", "beardog"),   // ChaCha20
            ("onion.hkdf_extract", "beardog"), // HKDF
            ("onion.hkdf_expand", "beardog"), // HKDF
            // Songbird network
            ("mesh.status", "songbird"),
            ("mesh.find_path", "songbird"),
            ("mesh.announce", "songbird"),
            ("punch.request", "songbird"),
            ("stun.discover", "songbird"),
            ("stun.detect_nat_type", "songbird"),
            ("relay.serve", "songbird"),
        ];

        for (cap, expected_provider) in required_caps {
            let translation = registry
                .get_translation(cap)
                .unwrap_or_else(|| panic!("Missing capability: {cap}"));

            assert_eq!(
                translation.provider, expected_provider,
                "Capability {} should route to {}, got {}",
                cap, expected_provider, translation.provider
            );
        }
    }
}
