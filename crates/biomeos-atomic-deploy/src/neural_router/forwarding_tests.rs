// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Unit tests for [`super::forwarding`] helpers.

#![expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]

use super::forwarding::parse_security_bytes_param;
use serde_json::json;

#[test]
fn parse_security_bytes_missing_key() {
    let err = parse_security_bytes_param(&json!({}), "data").unwrap_err();
    assert!(err.contains("missing param: data"));
}

#[test]
fn parse_security_bytes_valid_base64() {
    let b = parse_security_bytes_param(&json!({"data": "aGVsbG8="}), "data").unwrap();
    assert_eq!(b.as_ref(), b"hello");
}

#[test]
fn parse_security_bytes_invalid_base64() {
    let err = parse_security_bytes_param(&json!({"data": "@@@not-base64@@@"}), "data").unwrap_err();
    assert!(!err.is_empty());
}

#[test]
fn parse_security_bytes_json_byte_array() {
    let b = parse_security_bytes_param(&json!({"data": [104, 105]}), "data").unwrap();
    assert_eq!(b.as_ref(), b"hi");
}

#[test]
fn parse_security_bytes_json_byte_array_skips_non_u64_elements() {
    let b =
        parse_security_bytes_param(&json!({"data": [65u64, null, "x", 66u64]}), "data").unwrap();
    assert_eq!(b.as_ref(), b"AB");
}

#[test]
fn parse_security_bytes_wrong_type_number() {
    let err = parse_security_bytes_param(&json!({"data": 42}), "data").unwrap_err();
    assert!(err.contains("must be base64 string or byte array"));
}

#[test]
fn parse_security_bytes_wrong_type_object() {
    let err = parse_security_bytes_param(&json!({"data": {"x": 1}}), "data").unwrap_err();
    assert!(err.contains("must be base64 string or byte array"));
}

#[test]
fn parse_security_bytes_empty_array() {
    let b = parse_security_bytes_param(&json!({"sig": []}), "sig").unwrap();
    assert!(b.is_empty());
}
