// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Property-based tests for biomeos-types.
//!
//! Tests roundtrip serialization, identifier validation, and primal name constants.

use biomeos_types::primal_names::{
    AUXILIARY_PRIMALS, BIOMEOS, BIOMEOS_DEVICE_MANAGEMENT, CORE_PRIMALS, PROVENANCE_PRIMALS,
    SPRING_PRIMALS, is_known_primal,
};
use biomeos_types::{
    FamilyId, JsonRpcError, JsonRpcInput, JsonRpcRequest, JsonRpcResponse, PrimalId,
};
use proptest::prelude::*;

fn arb_json_value() -> impl Strategy<Value = serde_json::Value> {
    prop_oneof![
        Just(serde_json::json!({})),
        Just(serde_json::json!(true)),
        Just(serde_json::json!(false)),
        any::<i64>().prop_map(|n| serde_json::json!(n)),
        "[a-zA-Z0-9_.-]{0,64}".prop_map(|s| serde_json::json!(s)),
    ]
}

proptest! {
    /// FamilyId roundtrip: any string → FamilyId → as_str → same string.
    #[test]
    fn family_id_roundtrip(s in "\\PC{0,256}") {
        let family = FamilyId::new(&s);
        assert_eq!(family.as_str(), s);
    }

    /// JsonRpcRequest serde roundtrip: any method + params → serialize → deserialize → equal.
    #[test]
    fn jsonrpc_request_serde_roundtrip(
        method in "[a-zA-Z0-9_.]{1,128}",
        params in arb_json_value(),
    ) {
        let req = JsonRpcRequest::new(&method, params);
        let json = serde_json::to_string(&req).expect("serialize");
        let parsed: JsonRpcRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(req.method.as_ref(), parsed.method.as_ref());
        assert_eq!(req.params, parsed.params);
    }

    /// JsonRpcResponse success roundtrip.
    #[test]
    fn jsonrpc_response_success_roundtrip(
        id in any::<u64>(),
        result in arb_json_value(),
    ) {
        let resp = JsonRpcResponse::success(serde_json::json!(id), result.clone());
        let json = serde_json::to_string(&resp).expect("serialize");
        let parsed: JsonRpcResponse = serde_json::from_str(&json).expect("deserialize");
        assert!(parsed.error.is_none());
        assert_eq!(parsed.result, Some(result));
        assert_eq!(parsed.id, serde_json::json!(id));
    }

    /// JsonRpcResponse error roundtrip.
    #[test]
    fn jsonrpc_response_error_roundtrip(
        id in any::<u64>(),
        code in any::<i64>(),
        message in "[a-zA-Z0-9 _.-]{0,128}",
    ) {
        let err = JsonRpcError { code, message: message.clone(), data: None };
        let resp = JsonRpcResponse::error(serde_json::json!(id), err);
        let json = serde_json::to_string(&resp).expect("serialize");
        let parsed: JsonRpcResponse = serde_json::from_str(&json).expect("deserialize");
        assert!(parsed.result.is_none());
        let parsed_err = parsed.error.expect("error present");
        assert_eq!(parsed_err.code, code);
        assert_eq!(parsed_err.message, message);
    }

    /// JsonRpcInput::parse single request roundtrip.
    #[test]
    fn jsonrpc_input_single_roundtrip(
        method in "[a-zA-Z0-9_.]{1,64}",
        params in arb_json_value(),
    ) {
        let req = JsonRpcRequest::new(&method, params);
        let json = serde_json::to_string(&req).expect("serialize");
        let parsed = JsonRpcInput::parse(&json).expect("parse single");
        match parsed {
            JsonRpcInput::Single(r) => assert_eq!(r.method.as_ref(), method.as_str()),
            JsonRpcInput::Batch(_) => panic!("expected Single, got Batch"),
        }
    }

    /// JsonRpcInput::parse batch roundtrip (1..=4 requests).
    #[test]
    fn jsonrpc_input_batch_roundtrip(
        methods in proptest::collection::vec("[a-zA-Z0-9_.]{1,32}", 1..=4),
    ) {
        let requests: Vec<_> = methods
            .iter()
            .map(|m| JsonRpcRequest::new(m, serde_json::json!({})))
            .collect();
        let json = serde_json::to_string(&requests).expect("serialize batch");
        let parsed = JsonRpcInput::parse(&json).expect("parse batch");
        match parsed {
            JsonRpcInput::Batch(batch) => {
                assert_eq!(batch.len(), methods.len());
                for (req, method) in batch.iter().zip(methods.iter()) {
                    assert_eq!(req.method.as_ref(), method.as_str());
                }
            }
            JsonRpcInput::Single(_) => panic!("expected Batch, got Single"),
        }
    }

    /// Notification roundtrip (no id).
    #[test]
    fn jsonrpc_notification_roundtrip(
        method in "[a-zA-Z0-9_.]{1,64}",
        params in arb_json_value(),
    ) {
        let notif = JsonRpcRequest::notification(&method, params.clone());
        assert!(notif.id.is_none());
        let json = serde_json::to_string(&notif).expect("serialize");
        let parsed: JsonRpcRequest = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(parsed.method.as_ref(), method.as_str());
        assert!(parsed.id.is_none());
        assert_eq!(parsed.params, Some(params));
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
