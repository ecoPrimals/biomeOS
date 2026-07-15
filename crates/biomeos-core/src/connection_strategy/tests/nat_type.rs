// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[test]
fn test_nat_type_from_detection() {
    assert_eq!(NatType::from_detection("symmetric"), NatType::Symmetric);
    assert_eq!(NatType::from_detection("Symmetric"), NatType::Symmetric);
    assert_eq!(NatType::from_detection("full_cone"), NatType::FullCone);
    assert_eq!(NatType::from_detection("full-cone"), NatType::FullCone);
    assert_eq!(NatType::from_detection("none"), NatType::None);
    assert_eq!(NatType::from_detection("public"), NatType::None);
    assert_eq!(
        NatType::from_detection("address_restricted"),
        NatType::AddressRestricted
    );
    assert_eq!(
        NatType::from_detection("port_restricted"),
        NatType::PortRestricted
    );
    assert_eq!(NatType::from_detection("garbage"), NatType::Unknown);
}

#[test]
fn test_nat_type_properties() {
    assert!(NatType::Symmetric.is_symmetric());
    assert!(!NatType::FullCone.is_symmetric());
    assert!(!NatType::None.is_symmetric());

    assert!(NatType::None.supports_direct_punch());
    assert!(NatType::FullCone.supports_direct_punch());
    assert!(NatType::PortRestricted.supports_direct_punch());
    assert!(!NatType::Symmetric.supports_direct_punch());
    assert!(!NatType::Unknown.supports_direct_punch());
}

#[test]
fn test_nat_type_serialization_roundtrip() {
    for nat in &[
        NatType::None,
        NatType::FullCone,
        NatType::AddressRestricted,
        NatType::PortRestricted,
        NatType::Symmetric,
        NatType::Unknown,
    ] {
        let json = serde_json::to_string(nat).expect("serialize");
        let deserialized: NatType = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized, *nat);
    }
}

#[test]
fn test_nat_type_from_detection_open() {
    assert_eq!(NatType::from_detection("open"), NatType::None);
}

#[test]
fn test_nat_type_from_detection_fullcone() {
    assert_eq!(NatType::from_detection("fullcone"), NatType::FullCone);
}

#[test]
fn test_nat_type_from_detection_mixed_case() {
    assert_eq!(NatType::from_detection("SYMMETRIC"), NatType::Symmetric);
    assert_eq!(NatType::from_detection("Full_Cone"), NatType::FullCone);
}

#[test]
fn nat_type_address_restricted_supports_punch() {
    assert!(NatType::AddressRestricted.supports_direct_punch());
    assert!(!NatType::Unknown.supports_direct_punch());
}
