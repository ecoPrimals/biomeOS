// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![allow(clippy::unwrap_used, clippy::expect_used)]

//! Proptest IPC fuzzing — prevents DoS via malformed JSON-RPC input.
//!
//! Absorbed from wetSpring (7 tests), healthSpring (18 tests), primalSpring (5 tests),
//! and rhizoCrypt (7 tests). Ensures that the JSON-RPC parsing layer never panics
//! on arbitrary byte sequences.

use biomeos_types::ipc::{extract_rpc_error, extract_rpc_result};
use biomeos_types::jsonrpc::{
    JsonRpcError, JsonRpcInput, JsonRpcRequest, JsonRpcResponse, JsonRpcVersion,
};
use proptest::prelude::*;

proptest! {
    /// Parsing arbitrary strings must never panic — it should return Ok or Err.
    #[test]
    fn parse_request_never_panics(input in "\\PC{0,4096}") {
        let _ = JsonRpcRequest::parse(&input);
    }

    /// Batch/single dispatch parsing must never panic.
    #[test]
    fn parse_input_never_panics(input in "\\PC{0,4096}") {
        let _ = JsonRpcInput::parse(&input);
    }

    /// Deeply nested JSON must not stack overflow.
    #[test]
    fn deeply_nested_json_no_panic(depth in 1_usize..256) {
        let open: String = "[".repeat(depth);
        let close: String = "]".repeat(depth);
        let input = format!("{open}{close}");
        let _ = JsonRpcInput::parse(&input);
    }

    /// extract_rpc_result must never panic on arbitrary result/error combos.
    #[test]
    fn extract_result_never_panics(
        has_result in any::<bool>(),
        has_error in any::<bool>(),
        code in any::<i64>(),
        message in "\\PC{0,256}",
    ) {
        let resp = JsonRpcResponse {
            jsonrpc: JsonRpcVersion,
            result: if has_result { Some(serde_json::json!({"ok": true})) } else { None },
            error: if has_error {
                Some(JsonRpcError {
                    code,
                    message,
                    data: None,
                })
            } else {
                None
            },
            id: serde_json::json!(1),
        };
        let _ = extract_rpc_result(&resp, "fuzz-primal");
    }

    /// extract_rpc_error must never panic on arbitrary responses.
    #[test]
    fn extract_error_never_panics(
        has_error in any::<bool>(),
        code in any::<i64>(),
        message in "\\PC{0,256}",
    ) {
        let resp = JsonRpcResponse {
            jsonrpc: JsonRpcVersion,
            result: Some(serde_json::json!(null)),
            error: if has_error {
                Some(JsonRpcError { code, message, data: None })
            } else {
                None
            },
            id: serde_json::json!(1),
        };
        let _ = extract_rpc_error(&resp, "fuzz-primal");
    }

    /// Giant payloads must not cause allocation bombs.
    #[test]
    fn large_method_name_no_panic(size in 1_usize..65536) {
        let method = "a".repeat(size);
        let input = format!(r#"{{"jsonrpc":"2.0","method":"{}","id":1}}"#, method);
        let _ = JsonRpcRequest::parse(&input);
    }

    /// Unicode method names must not panic.
    #[test]
    fn unicode_method_names(method in "[\\p{L}\\p{N}\\.]{1,128}") {
        let input = format!(r#"{{"jsonrpc":"2.0","method":"{}","id":1}}"#, method);
        let _ = JsonRpcRequest::parse(&input);
    }

    /// Null bytes in input must not panic.
    #[test]
    fn null_bytes_no_panic(prefix in "\\PC{0,128}", suffix in "\\PC{0,128}") {
        let input = format!("{}\0{}", prefix, suffix);
        let _ = JsonRpcInput::parse(&input);
    }
}
