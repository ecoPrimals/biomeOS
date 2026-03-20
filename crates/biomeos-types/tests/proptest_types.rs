// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Property-based tests for biomeos-types.
//!
//! Tests roundtrip serialization, identifier validation, and primal name constants.

use biomeos_types::primal_names::{
    AUXILIARY_PRIMALS, BIOMEOS, BIOMEOS_DEVICE_MANAGEMENT, CORE_PRIMALS, PROVENANCE_PRIMALS,
    SPRING_PRIMALS, is_known_primal,
};
use biomeos_types::{FamilyId, JsonRpcRequest, PrimalId};
use proptest::prelude::*;

proptest! {
    /// FamilyId roundtrip: any string → FamilyId → as_str → same string.
    #[test]
    fn family_id_roundtrip(s in "\\PC{0,256}") {
        let family = FamilyId::new(&s);
        assert_eq!(family.as_str(), s);
    }

    /// JsonRpcRequest serde roundtrip: any method + params → serialize → deserialize → equal.
    /// Note: params = null serializes as null and deserializes as None, so we exclude null.
    #[test]
    fn jsonrpc_request_serde_roundtrip(
        method in "[a-zA-Z0-9_.]{1,128}",
        params in prop_oneof![
            Just(serde_json::json!({})),
            Just(serde_json::json!(true)),
            Just(serde_json::json!(false)),
            any::<i64>().prop_map(|n| serde_json::json!(n)),
            "[a-zA-Z0-9_.-]{0,64}".prop_map(|s| serde_json::json!(s)),
        ],
    ) {
        let req = JsonRpcRequest::new(&method, params);
        let json = serde_json::to_string(&req).expect("serialize");
        let parsed: JsonRpcRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(req.method.as_ref(), parsed.method.as_ref());
        assert_eq!(req.params, parsed.params);
    }

    /// PrimalId validation: any string that passes validation (alphanumeric, dash, underscore)
    /// creates a valid id with matching as_str().
    #[test]
    fn primal_id_valid_roundtrip(s in "[a-zA-Z0-9_-]{1,256}") {
        let id = PrimalId::new(&s).expect("valid primal id");
        assert_eq!(id.as_str(), s);
    }

    /// Primal name constants: is_known_primal is always true for each constant.
    #[test]
    fn is_known_primal_constants(idx in 0usize..21usize) {
        let all: Vec<&'static str> = CORE_PRIMALS
            .iter()
            .chain(PROVENANCE_PRIMALS.iter())
            .chain(SPRING_PRIMALS.iter())
            .chain(AUXILIARY_PRIMALS.iter())
            .chain([BIOMEOS, BIOMEOS_DEVICE_MANAGEMENT].iter())
            .copied()
            .collect();
        let name = all[idx];
        assert!(is_known_primal(name), "constant {name:?} must be known primal");
    }
}
