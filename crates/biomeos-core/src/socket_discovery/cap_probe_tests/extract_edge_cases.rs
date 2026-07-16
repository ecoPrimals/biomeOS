// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::extract_capabilities_from_response;
use serde_json::json;

#[test]
fn extract_capabilities_object_entries_skip_invalid() {
    let resp = json!({
        "result": {
            "capabilities": [
                {"not_name": "x"},
                "keep"
            ]
        }
    });
    assert_eq!(
        extract_capabilities_from_response(&resp),
        vec!["keep".to_string()]
    );
}

#[test]
fn extract_capabilities_methods_empty_then_warn_empty() {
    let resp = json!({
        "result": { "methods": [] }
    });
    assert!(extract_capabilities_from_response(&resp).is_empty());
}

// ── L2/L3 Wire Standard: result.methods as objects ──

#[test]
fn l2_methods_as_objects_with_name_key() {
    let resp = json!({
        "result": {
            "methods": [
                {"name": "crypto.sign", "version": "1.0"},
                {"name": "crypto.verify"},
                "health.check"
            ]
        }
    });
    assert_eq!(
        extract_capabilities_from_response(&resp),
        vec!["crypto.sign", "crypto.verify", "health.check"]
    );
}

#[test]
fn l2_methods_as_objects_with_method_key() {
    let resp = json!({
        "result": {
            "methods": [
                {"method": "storage.get"},
                {"method": "storage.put"}
            ]
        }
    });
    assert_eq!(
        extract_capabilities_from_response(&resp),
        vec!["storage.get", "storage.put"]
    );
}

// ── L3 Wire Standard: provided_capabilities with method objects ──

#[test]
fn l3_provided_capabilities_methods_as_objects() {
    let resp = json!({
        "result": {
            "provided_capabilities": [
                {
                    "type": "security",
                    "methods": [
                        {"name": "sign", "cost": "low"},
                        {"name": "verify", "cost": "low"},
                        "encrypt"
                    ]
                }
            ]
        }
    });
    let mut caps = extract_capabilities_from_response(&resp);
    caps.sort();
    assert_eq!(
        caps,
        vec![
            "security",
            "security.encrypt",
            "security.sign",
            "security.verify"
        ]
    );
}
