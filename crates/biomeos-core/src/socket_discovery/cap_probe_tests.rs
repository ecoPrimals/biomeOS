// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::or_fun_call,
    reason = "test assertions use unwrap/expect for clarity"
)]

use super::{extract_capabilities_from_response, probe_unix_socket_capabilities_list};
use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixListener;

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

// ── Socket probe tests ──

#[tokio::test]
async fn probe_nonexistent_socket_returns_empty() {
    let path = std::env::temp_dir().join("biomeos_cap_probe_absent_sock.sock");
    let _ = std::fs::remove_file(&path);
    assert!(!path.exists());
    let caps = probe_unix_socket_capabilities_list(&path).await;
    assert!(caps.is_empty());
}

#[tokio::test]
async fn probe_reads_capabilities_from_capabilities_list_response() {
    let dir = tempfile::tempdir().unwrap();
    let sock_path = dir.path().join("probe.sock");
    let path_for_client = sock_path.clone();
    let listener = UnixListener::bind(&sock_path).unwrap();

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let req_id = serde_json::from_str::<serde_json::Value>(&line)
            .ok()
            .and_then(|v| v.get("id").cloned())
            .unwrap_or(json!(1));
        let body = json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "result": {
                "capabilities": ["cap.one", "cap.two"]
            }
        });
        let mut stream = reader.into_inner();
        let line = format!("{}\n", serde_json::to_string(&body).unwrap());
        stream.write_all(line.as_bytes()).await.unwrap();
        stream.flush().await.unwrap();
    });

    let caps = probe_unix_socket_capabilities_list(&path_for_client).await;
    server.await.expect("server task");
    assert_eq!(caps, vec!["cap.one".to_string(), "cap.two".to_string()]);
}

#[tokio::test]
async fn probe_falls_back_to_capability_list_singular() {
    let dir = tempfile::tempdir().unwrap();
    let sock_path = dir.path().join("fallback.sock");
    let path_for_client = sock_path.clone();
    let listener = UnixListener::bind(&sock_path).unwrap();

    let server = tokio::spawn(async move {
        // Connection 1: capabilities.list (plural) → return JSON-RPC error
        let (stream, _) = listener.accept().await.unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let req_id = serde_json::from_str::<serde_json::Value>(&line)
            .ok()
            .and_then(|v| v.get("id").cloned())
            .unwrap_or(json!(1));
        let body = json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "error": {"code": -32601, "message": "Method not found"}
        });
        let mut stream = reader.into_inner();
        let resp = format!("{}\n", serde_json::to_string(&body).unwrap());
        stream.write_all(resp.as_bytes()).await.unwrap();
        stream.flush().await.unwrap();

        // Connection 2: capability.list (singular) → return capabilities
        let (stream, _) = listener.accept().await.unwrap();
        let mut reader = BufReader::new(stream);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let req_id = serde_json::from_str::<serde_json::Value>(&line)
            .ok()
            .and_then(|v| v.get("id").cloned())
            .unwrap_or(json!(1));
        let body = json!({
            "jsonrpc": "2.0",
            "id": req_id,
            "result": ["crypto", "security", "beacon"]
        });
        let mut stream = reader.into_inner();
        let resp = format!("{}\n", serde_json::to_string(&body).unwrap());
        stream.write_all(resp.as_bytes()).await.unwrap();
        stream.flush().await.unwrap();
    });

    let caps = probe_unix_socket_capabilities_list(&path_for_client).await;
    server.await.expect("server task");
    assert_eq!(caps, vec!["crypto", "security", "beacon"]);
}

#[tokio::test]
async fn probe_invalid_json_line_returns_empty() {
    let dir = tempfile::tempdir().unwrap();
    let sock_path = dir.path().join("bad-json.sock");
    let path_for_client = sock_path.clone();
    let listener = UnixListener::bind(&sock_path).unwrap();

    let server = tokio::spawn(async move {
        // Two connections: both return garbage
        for _ in 0..2 {
            let (stream, _) = listener.accept().await.unwrap();
            let mut reader = BufReader::new(stream);
            let mut line = String::new();
            reader.read_line(&mut line).await.unwrap();
            let mut stream = reader.into_inner();
            stream.write_all(b"not-json\n").await.unwrap();
            stream.flush().await.unwrap();
        }
    });

    let caps = probe_unix_socket_capabilities_list(&path_for_client).await;
    server.await.expect("server task");
    assert!(caps.is_empty());
}

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
