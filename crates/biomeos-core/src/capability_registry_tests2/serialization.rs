// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::capability_registry::*;
use crate::Capability;

#[test]
fn test_registry_request_register_serialization() {
    let req = RegistryRequest::Register {
        id: "beardog-localhost".to_string(),
        request_id: "req-1".to_string(),
        params: RegisterParams {
            provides: vec![Capability::Security],
            requires: vec![],
            socket_path: Some("/tmp/beardog.sock".to_string()),
            http_endpoint: None,
            metadata: None,
        },
    };
    let json = serde_json::to_string(&req).expect("serialize");
    assert!(json.contains("register"));
    assert!(json.contains("beardog-localhost"));
    let restored: RegistryRequest = serde_json::from_str(&json).expect("deserialize");
    match restored {
        RegistryRequest::Register { id, .. } => assert_eq!(id, "beardog-localhost"),
        _ => panic!("Expected Register variant"),
    }
}

#[test]
fn test_registry_request_unregister_serialization() {
    let req = RegistryRequest::Unregister {
        request_id: "req-2".to_string(),
        primal_id: "songbird-localhost".to_string(),
    };
    let json = serde_json::to_string(&req).expect("serialize");
    assert!(json.contains("unregister"));
    let restored: RegistryRequest = serde_json::from_str(&json).expect("deserialize");
    match restored {
        RegistryRequest::Unregister { primal_id, .. } => {
            assert_eq!(primal_id, "songbird-localhost");
        }
        _ => panic!("Expected Unregister variant"),
    }
}

#[test]
fn test_registry_request_heartbeat_serialization() {
    let req = RegistryRequest::Heartbeat {
        request_id: "req-3".to_string(),
        primal_id: "beardog-localhost".to_string(),
    };
    let json = serde_json::to_string(&req).expect("serialize");
    assert!(json.contains("heartbeat"));
    let restored: RegistryRequest = serde_json::from_str(&json).expect("deserialize");
    match restored {
        RegistryRequest::Heartbeat { primal_id, .. } => assert_eq!(primal_id, "beardog-localhost"),
        _ => panic!("Expected Heartbeat variant"),
    }
}

#[test]
fn test_registry_request_list_primals_serialization() {
    let req = RegistryRequest::ListPrimals {
        request_id: "req-4".to_string(),
    };
    let json = serde_json::to_string(&req).expect("serialize");
    assert!(json.contains("list_primals"));
    let restored: RegistryRequest = serde_json::from_str(&json).expect("deserialize");
    match restored {
        RegistryRequest::ListPrimals { request_id } => assert_eq!(request_id, "req-4"),
        _ => panic!("Expected ListPrimals variant"),
    }
}

#[test]
fn test_registry_response_error_status() {
    let resp = RegistryResponse {
        request_id: "req-1".to_string(),
        status: ResponseStatus::Error,
        data: None,
        error: Some("Invalid primal ID".to_string()),
    };
    let json = serde_json::to_value(&resp).expect("serialize");
    assert_eq!(json["status"], "error");
    assert_eq!(json["error"], "Invalid primal ID");
}

#[test]
fn test_registry_response_not_found_status() {
    let resp = RegistryResponse {
        request_id: "req-1".to_string(),
        status: ResponseStatus::NotFound,
        data: None,
        error: Some("No provider found".to_string()),
    };
    let json = serde_json::to_value(&resp).expect("serialize");
    assert_eq!(json["status"], "not_found");
}
