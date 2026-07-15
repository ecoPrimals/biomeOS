// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[test]
fn test_base64_encode_decode_roundtrip() {
    let data = b"hello world";
    let encoded = base64_encode(data);
    let decoded = base64_decode(&encoded).unwrap();
    assert_eq!(decoded.as_ref(), data);
}

#[test]
fn test_base64_encode_empty() {
    let encoded = base64_encode(b"");
    assert_eq!(encoded, "");
}

#[test]
fn test_base64_encode_single_byte() {
    let encoded = base64_encode(b"a");
    assert_eq!(encoded.len(), 4);
    assert!(encoded.ends_with("=="));
    let decoded = base64_decode(&encoded).unwrap();
    assert_eq!(decoded.as_ref(), b"a");
}

#[test]
fn test_base64_encode_two_bytes() {
    let encoded = base64_encode(b"ab");
    assert_eq!(encoded.len(), 4);
    assert!(encoded.ends_with('='));
    let decoded = base64_decode(&encoded).unwrap();
    assert_eq!(decoded.as_ref(), b"ab");
}

#[test]
fn test_base64_encode_three_bytes() {
    let encoded = base64_encode(b"abc");
    assert_eq!(encoded.len(), 4);
    assert!(!encoded.ends_with('='));
    let decoded = base64_decode(&encoded).unwrap();
    assert_eq!(decoded.as_ref(), b"abc");
}

#[test]
fn test_base64_decode_with_padding() {
    let decoded = base64_decode("YQ==").unwrap();
    assert_eq!(decoded.as_ref(), b"a");
}

#[test]
fn test_base64_decode_without_padding() {
    let decoded = base64_decode("YQ").unwrap();
    assert_eq!(decoded.as_ref(), b"a");
}

#[test]
fn test_base64_decode_ignores_invalid_chars() {
    // Invalid chars are filtered out
    let decoded = base64_decode("YQ==\n\t ").unwrap();
    assert_eq!(decoded.as_ref(), b"a");
}

#[test]
fn test_base64_decode_empty() {
    let decoded = base64_decode("").unwrap();
    assert!(decoded.is_empty());
}

#[test]
fn test_base64_encode_decode_large_data() {
    let data: Vec<u8> = (0u8..200).collect();
    let encoded = base64_encode(&data);
    let decoded = base64_decode(&encoded).unwrap();
    assert_eq!(decoded.as_ref(), data.as_slice());
}

#[test]
fn test_base64_decode_invalid_characters_filtered() {
    let decoded = base64_decode("Y\nQ\t=\r\n=").unwrap();
    assert_eq!(decoded.as_ref(), b"a");
}

#[test]
fn test_base64_decode_plus_slash() {
    let decoded = base64_decode("+/+").unwrap();
    assert!(!decoded.is_empty());
}

#[test]
fn test_base64_encode_all_padding_cases() {
    assert_eq!(base64_encode(&[0xFF, 0xFF, 0xFF]).len(), 4);
}
