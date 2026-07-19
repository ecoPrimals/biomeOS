// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::extract_capabilities_from_response;
use serde_json::json;

// ── Legacy: result.capabilities (strings) ──

#[test]
fn extract_capabilities_prefers_string_entries() {
    let resp = json!({
        "result": {
            "capabilities": ["alpha", "beta"]
        }
    });
    assert_eq!(
        extract_capabilities_from_response(&resp),
        vec!["alpha".to_string(), "beta".to_string()]
    );
}

// ── Legacy: result.capabilities (objects with `name`) ──

#[test]
fn extract_capabilities_accepts_object_entries_with_name_field() {
    let resp = json!({
        "result": {
            "capabilities": [
                {"name": "from_object"},
                "plain"
            ]
        }
    });
    assert_eq!(
        extract_capabilities_from_response(&resp),
        vec!["from_object".to_string(), "plain".to_string()]
    );
}

// ── Format A: result is a flat string array ──

#[test]
fn format_a_flat_string_array() {
    let resp = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": ["crypto", "tls_crypto", "genetic", "security", "beacon",
                    "http", "discovery", "mesh", "onion"]
    });
    let caps = extract_capabilities_from_response(&resp);
    assert_eq!(caps.len(), 9);
    assert_eq!(caps[0], "crypto");
    assert_eq!(caps[8], "onion");
}

// ── Format B: result is an object array with `method` key ──

#[test]
fn format_b_object_array_with_method_key() {
    let resp = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": [
            {"method": "crypto.sign", "version": "1.0"},
            {"method": "crypto.verify"},
            {"method": "crypto.encrypt"}
        ]
    });
    let caps = extract_capabilities_from_response(&resp);
    assert_eq!(caps, vec!["crypto.sign", "crypto.verify", "crypto.encrypt"]);
}

// ── Format B: mixed strings and objects in result array ──

#[test]
fn format_b_mixed_strings_and_objects() {
    let resp = json!({
        "result": [
            "plain.cap",
            {"method": "obj.cap"}
        ]
    });
    assert_eq!(
        extract_capabilities_from_response(&resp),
        vec!["plain.cap", "obj.cap"]
    );
}

// ── Format C: result.method_info [{name: ...}] ──

#[test]
fn format_c_method_info_array() {
    let resp = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "method_info": [
                {"name": "mesh.peers", "description": "List peers"},
                {"name": "mesh.status", "description": "Mesh status"}
            ]
        }
    });
    assert_eq!(
        extract_capabilities_from_response(&resp),
        vec!["mesh.peers", "mesh.status"]
    );
}

// ── Format D: result.semantic_mappings {domain: {verb: ...}} ──

#[test]
fn format_d_semantic_mappings() {
    let resp = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "semantic_mappings": {
                "crypto": {
                    "sign": {},
                    "verify": {},
                    "encrypt": {}
                },
                "tls": {
                    "derive_secrets": {}
                }
            }
        }
    });
    let mut caps = extract_capabilities_from_response(&resp);
    caps.sort();
    assert_eq!(
        caps,
        vec![
            "crypto.encrypt",
            "crypto.sign",
            "crypto.verify",
            "tls.derive_secrets"
        ]
    );
}

// ── Format D: domain without verb sub-map ──

#[test]
fn format_d_domain_without_verb_map() {
    let resp = json!({
        "result": {
            "semantic_mappings": {
                "beacon": "flat-value-not-object"
            }
        }
    });
    assert_eq!(extract_capabilities_from_response(&resp), vec!["beacon"]);
}

// ── Legacy: result.methods ──

#[test]
fn extract_capabilities_falls_back_to_methods_array() {
    let resp = json!({
        "result": {
            "capabilities": [],
            "methods": ["mesh.peers", "health.ping"]
        }
    });
    assert_eq!(
        extract_capabilities_from_response(&resp),
        vec!["mesh.peers".to_string(), "health.ping".to_string()]
    );
}

#[test]
fn extract_capabilities_returns_empty_for_unrecognized_shape() {
    let resp = json!({
        "result": {
            "capabilities": [],
            "methods": []
        }
    });
    assert!(extract_capabilities_from_response(&resp).is_empty());
}

// ── Format E: provided_capabilities [{type, methods}] (BearDog wire format) ──

#[test]
fn format_e_provided_capabilities_beardog() {
    let resp = json!({
        "jsonrpc": "2.0",
        "id": 1,
        "result": {
            "primal": "beardog",
            "version": "0.9.0",
            "provided_capabilities": [
                {
                    "type": "security",
                    "methods": ["sign", "verify", "encrypt", "decrypt"],
                    "version": "1.0.0"
                },
                {
                    "type": "crypto",
                    "methods": ["blake3_hash", "hmac_sha256"],
                    "version": "1.0.0"
                },
                {
                    "type": "beacon",
                    "methods": ["generate", "get_id"],
                    "version": "1.0.0"
                }
            ]
        }
    });
    let mut caps = extract_capabilities_from_response(&resp);
    caps.sort();
    assert_eq!(
        caps,
        vec![
            "beacon",
            "beacon.generate",
            "beacon.get_id",
            "crypto",
            "crypto.blake3_hash",
            "crypto.hmac_sha256",
            "security",
            "security.decrypt",
            "security.encrypt",
            "security.sign",
            "security.verify",
        ]
    );
}

#[test]
fn format_e_provided_capabilities_type_only() {
    let resp = json!({
        "result": {
            "provided_capabilities": [
                {"type": "storage"},
                {"type": "compute"}
            ]
        }
    });
    let caps = extract_capabilities_from_response(&resp);
    assert_eq!(caps, vec!["storage", "compute"]);
}
