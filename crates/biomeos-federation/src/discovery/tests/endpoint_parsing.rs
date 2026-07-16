// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Discovery tests - extracted to keep discovery/mod.rs under 1000 lines

#![expect(clippy::expect_used, reason = "test assertions")]

use std::net::SocketAddr;
use std::path::PathBuf;

use super::super::*;

#[test]
fn test_endpoint_parsing_unix() {
    let ep = PrimalDiscovery::parse_endpoint("unix:///tmp/test.sock");
    match ep {
        Some(PrimalEndpoint::UnixSocket { path }) => {
            assert_eq!(path, PathBuf::from("/tmp/test.sock"));
        }
        other => panic!("expected UnixSocket, got {other:?}"),
    }
}

#[test]
fn test_endpoint_parsing_udp() {
    let ep = PrimalDiscovery::parse_endpoint("udp://127.0.0.1:8080");
    match ep {
        Some(PrimalEndpoint::Udp { addr }) => {
            assert_eq!(addr.port(), 8080);
            assert_eq!(addr.ip().to_string(), "127.0.0.1");
        }
        other => panic!("expected Udp, got {other:?}"),
    }
}

#[test]
fn test_endpoint_parsing_http() {
    let ep = PrimalDiscovery::parse_endpoint("http://localhost:3000");
    match ep {
        Some(PrimalEndpoint::Http { url }) => {
            assert_eq!(url, "http://localhost:3000");
        }
        other => panic!("expected Http, got {other:?}"),
    }
}

#[test]
fn test_endpoint_parsing_https() {
    let ep = PrimalDiscovery::parse_endpoint("https://example.com/api");
    match ep {
        Some(PrimalEndpoint::Http { url }) => {
            assert_eq!(url, "https://example.com/api");
        }
        other => panic!("expected Http, got {other:?}"),
    }
}

#[test]
fn test_endpoint_parsing_invalid() {
    assert!(PrimalDiscovery::parse_endpoint("ftp://host").is_none());
    assert!(PrimalDiscovery::parse_endpoint("random-string").is_none());
    assert!(PrimalDiscovery::parse_endpoint("").is_none());
}

#[test]
fn test_endpoint_parsing_invalid_udp_addr() {
    assert!(PrimalDiscovery::parse_endpoint("udp://not-an-addr").is_none());
}

#[test]
fn test_primal_endpoint_serde_unix() {
    let ep = PrimalEndpoint::UnixSocket {
        path: PathBuf::from("/tmp/test.sock"),
    };
    let json = serde_json::to_string(&ep).expect("serialize");
    assert!(json.contains("unix_socket"));
    let restored: PrimalEndpoint = serde_json::from_str(&json).expect("deserialize");
    assert!(matches!(restored, PrimalEndpoint::UnixSocket { .. }));
}

#[test]
fn test_primal_endpoint_serde_http() {
    let ep = PrimalEndpoint::Http {
        url: "http://example.com".into(),
    };
    let json = serde_json::to_string(&ep).expect("serialize");
    assert!(json.contains("http"));
    let restored: PrimalEndpoint = serde_json::from_str(&json).expect("deserialize");
    match restored {
        PrimalEndpoint::Http { url } => assert_eq!(url, "http://example.com"),
        other => panic!("expected Http, got {other:?}"),
    }
}

#[test]
fn test_primal_endpoint_serde_udp() {
    let addr: SocketAddr = "127.0.0.1:9000".parse().expect("valid addr");
    let ep = PrimalEndpoint::Udp { addr };
    let json = serde_json::to_string(&ep).expect("serialize");
    assert!(json.contains("udp"));
    let restored: PrimalEndpoint = serde_json::from_str(&json).expect("deserialize");
    match restored {
        PrimalEndpoint::Udp { addr: a } => assert_eq!(a.port(), 9000),
        other => panic!("expected Udp, got {other:?}"),
    }
}

#[test]
fn test_primal_endpoint_debug() {
    let ep = PrimalEndpoint::UnixSocket {
        path: PathBuf::from("/a"),
    };
    assert!(format!("{ep:?}").contains("UnixSocket"));
}

#[test]
fn test_primal_endpoint_clone() {
    let ep = PrimalEndpoint::Http {
        url: "http://x".into(),
    };
    let cloned = ep;
    assert!(matches!(cloned, PrimalEndpoint::Http { .. }));
}
