// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::handlers::capability::CapabilityCallOutcome;
use crate::neural_api_server::rpc::DispatchOutcome;
use serde_json::json;

#[test]
fn dispatch_capability_call_success_with_routing_trace() {
    let outcome = super::super::super::dispatch_capability_call(
        Ok(CapabilityCallOutcome {
            result: json!({"ok": true}),
            routing_trace: Some(json!({"phases": []})),
        }),
        json!(7),
    );
    match outcome {
        DispatchOutcome::Success(v) => {
            assert_eq!(v["jsonrpc"], "2.0");
            assert_eq!(v["result"]["ok"], true);
            assert!(v["_routing_trace"].is_object());
            assert_eq!(v["id"], 7);
        }
        other => panic!("expected Success, got: {other:?}"),
    }
}

#[test]
fn dispatch_capability_call_success_without_routing_trace() {
    let outcome = super::super::super::dispatch_capability_call(
        Ok(CapabilityCallOutcome {
            result: json!({"value": 1}),
            routing_trace: None,
        }),
        json!(null),
    );
    match outcome {
        DispatchOutcome::Success(v) => {
            assert!(v.get("_routing_trace").is_none());
            assert_eq!(v["result"]["value"], 1);
        }
        other => panic!("expected Success, got: {other:?}"),
    }
}

#[test]
fn dispatch_capability_call_preserves_ipc_json_rpc_error() {
    let err = biomeos_types::IpcError::JsonRpcError {
        primal: "nestgate".to_string(),
        code: -32001,
        message: "permission denied".to_string(),
    };
    let outcome = super::super::super::dispatch_capability_call(Err(err.into()), json!(3));
    match outcome {
        DispatchOutcome::ApplicationError { code, message, id } => {
            assert_eq!(code, -32001);
            assert_eq!(message, "permission denied");
            assert_eq!(id, 3);
        }
        other => panic!("expected ApplicationError, got: {other:?}"),
    }
}

#[test]
fn dispatch_capability_call_uses_generic_code_for_other_errors() {
    let outcome = super::super::super::dispatch_capability_call(
        Err(anyhow::anyhow!("upstream down")),
        json!(1),
    );
    match outcome {
        DispatchOutcome::ApplicationError { code, message, .. } => {
            assert_eq!(code, -32603);
            assert!(message.contains("upstream down"));
        }
        other => panic!("expected ApplicationError, got: {other:?}"),
    }
}
