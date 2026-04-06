// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Tests for the service `networking` module.

#![expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]

use super::*;
use crate::health::HealthCheckConfig;

#[test]
fn test_service_networking_default() {
    let net = ServiceNetworking::default();
    assert!(matches!(net.network_mode, NetworkMode::Bridge));
    assert!(net.ports.is_empty());
    assert!(net.discovery.enabled);
    assert!(net.load_balancing.is_none());
}

#[test]
fn test_service_networking_serde_roundtrip() {
    let val = ServiceNetworking::default();
    let json = serde_json::to_string(&val).unwrap();
    let back: ServiceNetworking = serde_json::from_str(&json).unwrap();
    assert_eq!(
        format!("{:?}", val.network_mode),
        format!("{:?}", back.network_mode)
    );
    assert_eq!(val.ports.len(), back.ports.len());
}

#[test]
fn test_network_mode_serde() {
    for mode in [
        NetworkMode::Bridge,
        NetworkMode::Host,
        NetworkMode::Container("foo".to_string()),
        NetworkMode::Custom("custom".to_string()),
        NetworkMode::None,
    ] {
        let json = serde_json::to_string(&mode).unwrap();
        let back: NetworkMode = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{mode:?}"), format!("{back:?}"));
    }
}

#[test]
fn test_port_protocol_serde() {
    for proto in [
        PortProtocol::Tcp,
        PortProtocol::Udp,
        PortProtocol::Http,
        PortProtocol::Https,
        PortProtocol::Grpc,
        PortProtocol::WebSocket,
        PortProtocol::Custom("custom".to_string()),
    ] {
        let json = serde_json::to_string(&proto).unwrap();
        let back: PortProtocol = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{proto:?}"), format!("{back:?}"));
    }
}

#[test]
fn test_load_balancing_algorithm_serde() {
    for alg in [
        LoadBalancingAlgorithm::RoundRobin,
        LoadBalancingAlgorithm::LeastConnections,
        LoadBalancingAlgorithm::WeightedRoundRobin,
        LoadBalancingAlgorithm::IpHash,
        LoadBalancingAlgorithm::ConsistentHash,
        LoadBalancingAlgorithm::Random,
        LoadBalancingAlgorithm::Custom("custom".to_string()),
    ] {
        let json = serde_json::to_string(&alg).unwrap();
        let back: LoadBalancingAlgorithm = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{alg:?}"), format!("{back:?}"));
    }
}

#[test]
fn test_discovery_method_serde() {
    let dns = DiscoveryMethod::Dns {
        domain: "example.com".to_string(),
        ttl: 60,
    };
    let json = serde_json::to_string(&dns).unwrap();
    let back: DiscoveryMethod = serde_json::from_str(&json).unwrap();
    if let (
        DiscoveryMethod::Dns {
            domain: d1,
            ttl: t1,
        },
        DiscoveryMethod::Dns {
            domain: d2,
            ttl: t2,
        },
    ) = (dns, back)
    {
        assert_eq!(d1, d2);
        assert_eq!(t1, t2);
    } else {
        panic!("Expected Dns variant");
    }
}

#[test]
fn test_service_port_serde() {
    let port = ServicePort {
        name: "http".to_string(),
        port: 8080,
        target_port: Some(8080),
        protocol: PortProtocol::Tcp,
        expose: true,
        external_port: Some(80),
        load_balancer: None,
    };
    let json = serde_json::to_string(&port).unwrap();
    let back: ServicePort = serde_json::from_str(&json).unwrap();
    assert_eq!(port.name, back.name);
    assert_eq!(port.port, back.port);
}

#[test]
fn test_load_balancing_config_serde() {
    let config = LoadBalancingConfig {
        lb_type: LoadBalancerType::Application,
        target_groups: vec![],
        health_check: HealthCheckConfig::default(),
        settings: LoadBalancerSettings {
            connection_draining_timeout: 300,
            cross_zone_load_balancing: true,
            access_logs_enabled: false,
            access_logs_bucket: None,
            idle_timeout: 60,
            deletion_protection: false,
        },
    };
    let json = serde_json::to_string(&config).unwrap();
    let back: LoadBalancingConfig = serde_json::from_str(&json).unwrap();
    assert!(matches!(back.lb_type, LoadBalancerType::Application));
}

#[test]
fn test_traffic_management_serde() {
    let tm = TrafficManagement {
        traffic_splitting: None,
        circuit_breaker: Some(CircuitBreaker {
            failure_threshold: 5,
            recovery_timeout: 30,
            request_volume_threshold: 10,
            error_rate_threshold: 0.5,
            sleep_window: 60,
        }),
        rate_limiting: None,
        timeouts: None,
        retries: None,
    };
    let json = serde_json::to_string(&tm).unwrap();
    let back: TrafficManagement = serde_json::from_str(&json).unwrap();
    assert!(back.circuit_breaker.is_some());
}

#[test]
fn test_route_condition_serde() {
    let cond = RouteCondition::Header {
        name: "X-Version".to_string(),
        value: "v1".to_string(),
        operator: ConditionOperator::Equals,
    };
    let json = serde_json::to_string(&cond).unwrap();
    let back: RouteCondition = serde_json::from_str(&json).unwrap();
    if let RouteCondition::Header { name, .. } = back {
        assert_eq!(name, "X-Version");
    } else {
        panic!("Expected Header variant");
    }
}

#[test]
fn test_backoff_strategy_serde() {
    let fixed = BackoffStrategy::Fixed { delay: 5 };
    let json = serde_json::to_string(&fixed).unwrap();
    let back: BackoffStrategy = serde_json::from_str(&json).unwrap();
    if let BackoffStrategy::Fixed { delay } = back {
        assert_eq!(delay, 5);
    } else {
        panic!("Expected Fixed variant");
    }
}
