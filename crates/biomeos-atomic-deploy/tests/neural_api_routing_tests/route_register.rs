// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::create_test_router;
use biomeos_core::TransportEndpoint;

#[tokio::test]
async fn test_route_register_batch_tcp() {
    let router = create_test_router();

    let endpoint = TransportEndpoint::parse("tcp://192.0.2.100:9001").unwrap();
    let capabilities = ["crypto.sign", "crypto.verify", "crypto.encrypt"];

    for cap in capabilities {
        router
            .register_capability(cap, "beardog", endpoint.clone(), "route.register@pixel")
            .await
            .unwrap();
    }

    let registry = router.list_capabilities().await;
    for cap in capabilities {
        assert!(
            registry.contains_key(cap),
            "capability '{cap}' should be registered"
        );
    }
}

#[tokio::test]
async fn test_route_register_batch_abstract_socket() {
    let router = create_test_router();

    let endpoint = TransportEndpoint::parse("@biomeos-beardog").unwrap();
    let capabilities = ["security", "beacon", "genetic"];

    for cap in capabilities {
        router
            .register_capability(cap, "beardog", endpoint.clone(), "route.register")
            .await
            .unwrap();
    }

    let all = router.list_capabilities().await;
    assert_eq!(all.len(), 3);
}

#[tokio::test]
async fn test_route_register_batch_http() {
    let router = create_test_router();

    let endpoint = TransportEndpoint::parse("http://gate2.local:8080").unwrap();
    let capabilities = ["http.request", "http.proxy"];

    for cap in capabilities {
        router
            .register_capability(cap, "songbird", endpoint.clone(), "route.register@gate2")
            .await
            .unwrap();
    }

    let providers = router
        .get_capability_providers("http.request")
        .await
        .expect("should have providers");
    assert_eq!(providers.len(), 1);
    assert_eq!(providers[0].primal_name.as_ref(), "songbird");
    assert_eq!(providers[0].source.as_ref(), "route.register@gate2");
}

#[tokio::test]
async fn test_route_register_transport_parse_failure() {
    assert!(
        TransportEndpoint::parse("").is_none(),
        "empty string should not parse"
    );
    assert!(
        TransportEndpoint::parse("just-a-name").is_none(),
        "bare name without path or port should not parse"
    );
}

#[tokio::test]
async fn test_route_register_all_transport_types() {
    let router = create_test_router();

    let transports = [
        ("tcp://10.0.0.1:9001", "beardog-tcp"),
        ("http://10.0.0.2:8080", "songbird-http"),
        ("@biomeos-nestgate", "nestgate-abstract"),
        ("/run/user/1000/toadstool.sock", "toadstool-unix"),
    ];

    for (transport_str, primal) in transports {
        let endpoint = TransportEndpoint::parse(transport_str)
            .unwrap_or_else(|| panic!("should parse: {transport_str}"));
        router
            .register_capability("test.cap", primal, endpoint, "route.register")
            .await
            .unwrap();
    }

    let providers = router
        .get_capability_providers("test.cap")
        .await
        .expect("should have providers");
    assert_eq!(providers.len(), 4);
}

#[tokio::test]
async fn test_route_register_source_tag_with_gate() {
    let router = create_test_router();

    let endpoint = TransportEndpoint::parse("tcp://192.0.2.132:9001").unwrap();
    router
        .register_capability("crypto", "beardog", endpoint, "cross-gate@gate2")
        .await
        .unwrap();

    let providers = router
        .get_capability_providers("crypto")
        .await
        .expect("should have providers");
    assert_eq!(providers.len(), 1);
    assert!(
        providers[0].source.contains("gate2"),
        "source should contain gate label"
    );
}
