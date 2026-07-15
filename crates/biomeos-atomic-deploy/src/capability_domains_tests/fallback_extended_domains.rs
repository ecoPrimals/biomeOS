// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::super::capability_domains::*;

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
    // Verify BOOTSTRAP_CAPABILITY_HINTS is properly structured
    assert!(!BOOTSTRAP_CAPABILITY_HINTS.is_empty(), "Should have domains");

    // Each domain should have a non-empty provider and capabilities
    for domain in BOOTSTRAP_CAPABILITY_HINTS {
        assert!(!domain.provider.is_empty(), "Provider should not be empty");
        assert!(
            !domain.capabilities.is_empty(),
            "Capabilities should not be empty"
        );
    }

    // Verify expected domains exist
    let providers: Vec<&str> = BOOTSTRAP_CAPABILITY_HINTS.iter().map(|d| d.provider).collect();
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
