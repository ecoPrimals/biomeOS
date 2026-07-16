// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::{DerivationParams, DeviceLineage, generate_device_entropy};

#[test]
fn test_device_lineage_serialization() {
    let lineage = super::sample_lineage();
    let json = serde_json::to_string(&lineage).expect("serialize");
    assert!(json.contains("device-123"));
    assert!(json.contains("tower"));
    assert!(json.contains("1894e909e454"));
    let parsed: DeviceLineage = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.device_id, "device-123");
    assert_eq!(parsed.node_id, "tower");
}

#[test]
fn test_derivation_params_serialization() {
    let params = DerivationParams {
        family_seed: "c2VlZA==".to_string(),
        device_id: "dev-001".to_string(),
        node_id: "tower".to_string(),
        device_entropy: Some("ZW50cm9weQ==".to_string()),
        purpose: "device-lineage".to_string(),
    };
    let json = serde_json::to_string(&params).expect("serialize");
    assert!(json.contains("family_seed"));
    assert!(json.contains("dev-001"));
}

#[test]
fn test_generate_device_entropy() {
    let entropy1 = generate_device_entropy().expect("device entropy generation");
    let entropy2 = generate_device_entropy().expect("device entropy generation");
    assert_eq!(entropy1.len(), 32);
    assert_eq!(entropy2.len(), 32);
    assert_ne!(entropy1, entropy2);
}
