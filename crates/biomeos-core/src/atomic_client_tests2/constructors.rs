// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::TransportEndpoint;
use crate::atomic_client::{AtomicClient, DiscoverByCapabilityOpts, DiscoverOpts};
use crate::atomic_primal_client::{AtomicPrimalClient, ExecutionResult};
use std::collections::HashMap;
use std::sync::Arc;

#[test]
fn test_discover_opts_clone() {
    let mut m = HashMap::new();
    m.insert("K".to_string(), "v".to_string());
    let a = DiscoverOpts {
        family_id: Some("f"),
        env_overrides: Some(&m),
        tcp_tier2_override: Some("h:1"),
    };
    let b = a.clone();
    assert_eq!(a.family_id, b.family_id);
}

#[test]
fn test_discover_by_capability_opts_clone() {
    let a = DiscoverByCapabilityOpts {
        family_id: Some("g"),
        strict_discovery: Some(false),
    };
    assert_eq!(a.clone().strict_discovery, Some(false));
}

#[test]
fn test_atomic_client_from_endpoint_http_preserves_host_port() {
    let ep = TransportEndpoint::HttpJsonRpc {
        host: Arc::from("gw.example"),
        port: 8443,
    };
    let c = AtomicClient::from_endpoint(ep);
    assert!(matches!(
        c.endpoint(),
        TransportEndpoint::HttpJsonRpc { .. }
    ));
    assert!(c.socket_path().as_os_str().is_empty());
}

#[cfg(target_os = "linux")]
#[test]
fn test_atomic_client_abstract_socket_constructor() {
    let c = AtomicClient::abstract_socket("test-abs-name");
    assert!(matches!(
        c.endpoint(),
        TransportEndpoint::AbstractSocket { .. }
    ));
    assert!(c.is_available());
}

#[test]
fn test_atomic_primal_client_http_constructor() {
    let client = AtomicPrimalClient::tcp("beardog", "192.0.2.100", 9100);
    assert_eq!(client.primal_name(), "beardog");
    assert!(client.endpoint().display_string().contains("192.0.2.100"));
}

#[test]
fn test_execution_result_construction() {
    let result = ExecutionResult {
        stdout: "output".to_string(),
        stderr: "errors".to_string(),
        exit_code: Some(0),
    };
    assert_eq!(result.stdout, "output");
    assert_eq!(result.stderr, "errors");
    assert_eq!(result.exit_code, Some(0));
}

#[test]
fn test_execution_result_without_exit_code() {
    let result = ExecutionResult {
        stdout: String::new(),
        stderr: String::new(),
        exit_code: None,
    };
    assert!(result.exit_code.is_none());
}

#[test]
fn test_execution_result_serialization_roundtrip() {
    let result = ExecutionResult {
        stdout: "out".to_string(),
        stderr: "err".to_string(),
        exit_code: Some(1),
    };
    let json = serde_json::to_string(&result).expect("serialize");
    let parsed: ExecutionResult = serde_json::from_str(&json).expect("deserialize");
    assert_eq!(parsed.stdout, result.stdout);
    assert_eq!(parsed.stderr, result.stderr);
    assert_eq!(parsed.exit_code, result.exit_code);
}

#[test]
fn test_atomic_client_http_display() {
    let client = AtomicClient::http("api.example.com", 443);
    let endpoint = client.endpoint();
    assert!(
        endpoint.display_string().contains("api.example.com")
            || endpoint.display_string().contains("443")
    );
}

#[cfg(target_os = "linux")]
#[test]
fn test_atomic_client_abstract_socket_linux() {
    let client = AtomicClient::abstract_socket("test-abstract");
    assert!(matches!(
        client.endpoint(),
        TransportEndpoint::AbstractSocket { .. }
    ));
    assert!(client.is_available());
}

#[test]
fn test_transport_endpoint_debug_clone_roundtrip() {
    let e1 = TransportEndpoint::TcpSocket {
        host: Arc::from("h"),
        port: 1,
    };
    let e2 = e1.clone();
    assert_eq!(format!("{e1:?}"), format!("{e2:?}"));
}

#[test]
fn test_is_available_abstract_linux_only() {
    #[cfg(target_os = "linux")]
    {
        let c = AtomicClient::abstract_socket("abs-name-test");
        assert!(c.is_available());
    }
}
