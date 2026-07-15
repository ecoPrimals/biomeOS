// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::common::{healthy_transport, healthy_tunnel};
use super::super::super::TransportEndpoint;

#[test]
fn test_compute_overall_status_both_healthy() {
    let security = healthy_tunnel();
    let transport = healthy_transport();
    assert_eq!(
        super::super::compute_overall_status(&security, &transport),
        super::super::super::HealthStatus::Healthy
    );
}

#[test]
fn test_compute_overall_status_security_degraded() {
    let security = super::super::TunnelHealth {
        status: super::super::super::HealthStatus::Degraded,
        ..healthy_tunnel()
    };
    let transport = healthy_transport();
    assert_eq!(
        super::super::compute_overall_status(&security, &transport),
        super::super::super::HealthStatus::Degraded
    );
}

#[test]
fn test_compute_overall_status_transport_unhealthy() {
    let security = healthy_tunnel();
    let transport = super::super::TransportHealth {
        status: super::super::super::HealthStatus::Unhealthy,
        ..healthy_transport()
    };
    assert_eq!(
        super::super::compute_overall_status(&security, &transport),
        super::super::super::HealthStatus::Unhealthy
    );
}

#[test]
fn test_compute_overall_status_security_unhealthy() {
    let security = super::super::TunnelHealth {
        status: super::super::super::HealthStatus::Unhealthy,
        ..healthy_tunnel()
    };
    let transport = healthy_transport();
    assert_eq!(
        super::super::compute_overall_status(&security, &transport),
        super::super::super::HealthStatus::Unhealthy
    );
}

#[test]
fn test_compute_overall_status_both_degraded() {
    let security = super::super::TunnelHealth {
        status: super::super::super::HealthStatus::Degraded,
        ..healthy_tunnel()
    };
    let transport = super::super::TransportHealth {
        status: super::super::super::HealthStatus::Degraded,
        ..healthy_transport()
    };
    assert_eq!(
        super::super::compute_overall_status(&security, &transport),
        super::super::super::HealthStatus::Degraded
    );
}

#[test]
fn test_compute_overall_status_transport_degraded_security_healthy() {
    let security = healthy_tunnel();
    let transport = super::super::TransportHealth {
        status: super::super::super::HealthStatus::Degraded,
        ..healthy_transport()
    };
    assert_eq!(
        super::super::compute_overall_status(&security, &transport),
        super::super::super::HealthStatus::Degraded
    );
}

#[test]
fn test_classify_degradation_transport_latency() {
    let transport = super::super::TransportHealth {
        connection_status: super::super::super::HealthStatus::Degraded,
        latency_ms: Some(500),
        packet_loss: None,
        status: super::super::super::HealthStatus::Degraded,
    };
    assert_eq!(
        super::super::classify_degradation(&healthy_tunnel(), &transport),
        super::super::DegradationCause::TransportLatency
    );
}

#[test]
fn test_classify_degradation_transport_packet_loss() {
    let transport = super::super::TransportHealth {
        connection_status: super::super::super::HealthStatus::Degraded,
        latency_ms: Some(10),
        packet_loss: Some(5.0),
        status: super::super::super::HealthStatus::Degraded,
    };
    assert_eq!(
        super::super::classify_degradation(&healthy_tunnel(), &transport),
        super::super::DegradationCause::TransportPacketLoss
    );
}

#[test]
fn test_classify_degradation_auth_failure() {
    let security = super::super::TunnelHealth {
        encryption_status: super::super::super::HealthStatus::Degraded,
        status: super::super::super::HealthStatus::Degraded,
        ..healthy_tunnel()
    };
    assert_eq!(
        super::super::classify_degradation(&security, &healthy_transport()),
        super::super::DegradationCause::AuthFailure
    );
}

#[test]
fn test_classify_degradation_unknown_without_metrics() {
    let security = super::super::TunnelHealth {
        status: super::super::super::HealthStatus::Degraded,
        ..healthy_tunnel()
    };
    let transport = super::super::TransportHealth {
        connection_status: super::super::super::HealthStatus::Healthy,
        latency_ms: None,
        packet_loss: None,
        status: super::super::super::HealthStatus::Healthy,
    };
    assert_eq!(
        super::super::classify_degradation(&security, &transport),
        super::super::DegradationCause::Unknown
    );
}

#[test]
fn test_parse_tcp_fallback_value_host_port() {
    let parsed = super::super::parse_tcp_fallback_value("127.0.0.1:9100").expect("parsed");
    assert_eq!(parsed.0, "127.0.0.1");
    assert_eq!(parsed.1, 9100);
}

#[test]
fn test_build_tcp_fallback_from_uds_endpoint() {
    let endpoint = TransportEndpoint {
        node_id: "node-a".to_string(),
        address: "/tmp/node-a.sock".to_string(),
        port: 0,
        protocol: "uds".to_string(),
        secure: true,
    };
    let fallback = super::super::build_tcp_fallback_endpoint(&endpoint, "127.0.0.1".to_string(), 9100)
        .expect("fallback");
    assert_eq!(fallback.protocol, "tcp");
    assert_eq!(fallback.address, "127.0.0.1");
    assert_eq!(fallback.port, 9100);
}
