// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::or_fun_call,
    clippy::future_not_send,
    reason = "test assertions"
)]

use super::super::*;
use super::VALID_SHARED_SECRET_HEX;

// ── decode_shared_secret_to_key ──

#[test]
fn decode_shared_secret_valid_hex_produces_32_byte_key() {
    let key = decode_shared_secret_to_key(VALID_SHARED_SECRET_HEX).expect("valid hex");
    assert_eq!(key[0], 0x01);
    assert_eq!(key[31], 0x20);
}

#[test]
fn decode_shared_secret_rejects_wrong_length() {
    assert!(decode_shared_secret_to_key("0102").is_none());
    assert!(decode_shared_secret_to_key("").is_none());
}

#[test]
fn decode_shared_secret_rejects_invalid_hex() {
    assert!(decode_shared_secret_to_key("zzzz").is_none());
    let bad = "gg".repeat(32);
    assert!(decode_shared_secret_to_key(&bad).is_none());
}

#[test]
fn decode_shared_secret_rejects_non_hex_ascii() {
    assert!(decode_shared_secret_to_key("not-a-valid-shared-secret-hex!!!").is_none());
}
