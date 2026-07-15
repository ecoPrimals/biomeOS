// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::capability_registry::*;
use crate::Capability;
use biomeos_types::PrimalId;
use std::collections::HashMap;

#[test]
fn test_registry_request_serialization() {
    let req = RegistryRequest::GetProvider {
        request_id: "req-1".to_string(),
        capability: Capability::Compute,
    };
    let json = serde_json::to_string(&req).expect("serialize");
    let restored: RegistryRequest = serde_json::from_str(&json).expect("deserialize");
    match (&req, &restored) {
        (
            RegistryRequest::GetProvider {
                request_id: r1,
                capability: c1,
            },
            RegistryRequest::GetProvider {
                request_id: r2,
                capability: c2,
            },
        ) => {
            assert_eq!(r1, r2);
            assert_eq!(c1, c2);
        }
        _ => panic!("mismatch"),
    }
}

#[test]
fn test_registry_response_serialization() {
    let resp = RegistryResponse {
        request_id: "req-1".to_string(),
        status: ResponseStatus::Success,
        data: Some(serde_json::json!({"message": "ok"})),
        error: None,
    };
    let json = serde_json::to_value(&resp).expect("serialize");
    let restored: RegistryResponse = serde_json::from_value(json).expect("deserialize");
    assert_eq!(resp.request_id, restored.request_id);
    assert!(matches!(restored.status, ResponseStatus::Success));
}

#[test]
fn test_register_params_serialization() {
    let params = RegisterParams {
        provides: vec![Capability::Security, Capability::Storage],
        requires: vec![Capability::Compute],
        socket_path: Some("/tmp/sock".to_string()),
        http_endpoint: None,
        metadata: Some({
            let mut m = HashMap::new();
            m.insert("key".to_string(), "value".to_string());
            m
        }),
    };
    let json = serde_json::to_value(&params).expect("serialize");
    let restored: RegisterParams = serde_json::from_value(json).expect("deserialize");
    assert_eq!(params.provides.len(), restored.provides.len());
    assert_eq!(params.requires.len(), restored.requires.len());
}

#[test]
fn test_registry_request_get_provider_roundtrip_json() {
    let req = RegistryRequest::GetProvider {
        request_id: "g1".to_string(),
        capability: Capability::Discovery,
    };
    let v = serde_json::to_value(&req).expect("to_value");
    let back: RegistryRequest = serde_json::from_value(v).expect("from_value");
    match back {
        RegistryRequest::GetProvider {
            request_id,
            capability,
        } => {
            assert_eq!(request_id, "g1");
            assert_eq!(capability, Capability::Discovery);
        }
        _ => panic!("expected GetProvider"),
    }
}

#[test]
fn test_primal_info_serialization_roundtrip() {
    let now = chrono::Utc::now();
    let info = PrimalInfo {
        id: PrimalId::new("beardog-localhost").unwrap(),
        provides: vec![Capability::Security, Capability::Compute],
        requires: vec![Capability::Storage],
        socket_path: Some("/tmp/beardog.sock".to_string()),
        http_endpoint: Some("http://localhost:8080".to_string()),
        metadata: {
            let mut m = HashMap::new();
            m.insert("version".to_string(), "1.0".to_string());
            m
        },
        registered_at: now,
        last_heartbeat: now,
    };
    let json = serde_json::to_value(&info).expect("serialize");
    let restored: PrimalInfo = serde_json::from_value(json).expect("deserialize");
    assert_eq!(restored.id, info.id);
    assert_eq!(restored.provides.len(), 2);
    assert_eq!(restored.requires.len(), 1);
    assert_eq!(restored.socket_path, info.socket_path);
}
