// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[test]
fn test_inject_signing_with_existing_metadata() {
    let toml = "[graph]\nid = \"test\"\n\n[graph.metadata]\nauthor = \"me\"\n";
    let result = inject_signing_metadata(toml, "abc123", "sig456", "pub789");
    assert!(result.contains("content_hash = \"abc123\""));
    assert!(result.contains("signature = \"sig456\""));
    assert!(result.contains("signed_by = \"pub789\""));
    assert!(result.contains("author = \"me\""));
}

#[test]
fn test_inject_signing_without_metadata() {
    let toml = "[graph]\nid = \"test\"\n";
    let result = inject_signing_metadata(toml, "abc", "sig", "pub");
    assert!(result.contains("[graph.metadata]"));
    assert!(result.contains("content_hash = \"abc\""));
}

#[test]
fn test_strip_old_signing_idempotent() {
    let toml = "[graph.metadata]\ncontent_hash = \"old\"\nsignature = \"old\"\nsigned_by = \"old\"\nauthor = \"me\"\n";
    let stripped = strip_old_signing(toml);
    assert!(!stripped.contains("content_hash"));
    assert!(!stripped.contains("signature"));
    assert!(!stripped.contains("signed_by"));
    assert!(stripped.contains("author = \"me\""));
}

#[test]
fn test_extract_signing_metadata() {
    let toml =
        "[graph.metadata]\ncontent_hash = \"abc\"\nsignature = \"def\"\nsigned_by = \"012\"\n";
    let (h, s, p) = extract_signing_metadata(toml);
    assert_eq!(h.unwrap(), "abc");
    assert_eq!(s.unwrap(), "def");
    assert_eq!(p.unwrap(), "012");
}

#[test]
fn test_extract_signing_metadata_none() {
    let toml = "[graph]\nid = \"test\"\n";
    let (h, s, p) = extract_signing_metadata(toml);
    assert!(h.is_none());
    assert!(s.is_none());
    assert!(p.is_none());
}

#[test]
fn test_extract_toml_string_value_malformed() {
    assert!(extract_toml_string_value(" = noquotes").is_none());
    assert!(extract_toml_string_value("= \"unclosed").is_none());
}
