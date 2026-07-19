// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::collections::HashMap;

use super::super::*;
use super::common::roundtrip_json;

#[test]
fn network_dns_spec_roundtrip() {
    let spec = NetworkDnsSpec {
        nameservers: vec!["192.0.2.53".into(), "198.51.100.53".into()],
        search: vec!["cluster.local".into()],
        options: vec![DnsOptionSpec {
            name: "ndots".into(),
            value: Some("5".into()),
        }],
    };
    roundtrip_json(&spec);
}

#[test]
fn dns_option_spec_roundtrip() {
    let spec = DnsOptionSpec {
        name: "timeout".into(),
        value: Some("2".into()),
    };
    roundtrip_json(&spec);
}

#[test]
fn ipam_spec_roundtrip() {
    let spec = IpamSpec {
        driver: "default".into(),
        config: vec![IpamConfigSpec {
            subnet: "10.0.0.0/24".into(),
            ip_range: Some("10.0.0.0/28".into()),
            gateway: Some("10.0.0.1".into()),
            aux_addresses: HashMap::new(),
        }],
        options: std::iter::once(("foo".into(), "bar".into())).collect(),
    };
    roundtrip_json(&spec);
}

#[test]
fn ipam_config_spec_roundtrip() {
    let spec = IpamConfigSpec {
        subnet: "172.16.0.0/16".into(),
        ip_range: None,
        gateway: None,
        aux_addresses: std::iter::once(("host1".into(), "172.16.0.2".into())).collect(),
    };
    roundtrip_json(&spec);
}

#[test]
fn service_mesh_config_roundtrip() {
    let spec = ServiceMeshConfig {
        mtls_enabled: true,
        telemetry: Some(MeshTelemetrySpec {
            tracing_enabled: true,
            metrics_enabled: true,
            access_logs_enabled: false,
            sampling_rate: Some(0.1),
        }),
        ingress: None,
        egress: None,
    };
    roundtrip_json(&spec);
}

#[test]
fn port_spec_roundtrip() {
    let spec = PortSpec {
        number: 443,
        name: "https".into(),
        protocol: "TLS".into(),
    };
    roundtrip_json(&spec);
}

#[test]
fn minimal_struct_construction() {
    let _dns = NetworkDnsSpec {
        nameservers: vec![],
        search: vec![],
        options: vec![],
    };
    let _ipam = IpamSpec {
        driver: "default".into(),
        config: vec![],
        options: HashMap::new(),
    };
    let mesh_config = ServiceMeshConfig {
        mtls_enabled: false,
        telemetry: None,
        ingress: None,
        egress: None,
    };
    assert!(!mesh_config.mtls_enabled);
}

#[test]
fn destination_rule_spec_minimal_roundtrip() {
    let spec = DestinationRuleSpec {
        name: "reviews".into(),
        host: "reviews".into(),
        traffic_policy: None,
        subsets: vec![],
    };
    roundtrip_json(&spec);
}

#[test]
fn mesh_ingress_with_egress_on_config_roundtrip() {
    let spec = ServiceMeshConfig {
        mtls_enabled: false,
        telemetry: None,
        ingress: Some(MeshIngressSpec {
            gateways: vec![],
            virtual_services: vec![],
        }),
        egress: Some(MeshEgressSpec {
            service_entries: vec![],
            destination_rules: vec![],
        }),
    };
    roundtrip_json(&spec);
}

#[test]
fn tls_route_spec_minimal_roundtrip() {
    let spec = TlsRouteSpec {
        match_conditions: vec![],
        route: vec![],
    };
    roundtrip_json(&spec);
}

#[test]
fn traffic_policy_tls_only_roundtrip() {
    let tp = TrafficPolicySpec {
        load_balancer: None,
        connection_pool: None,
        outlier_detection: None,
        tls: Some(ClientTlsSettings {
            mode: ClientTlsMode::Mutual,
            client_certificate: Some("/certs/client.pem".into()),
            private_key: Some("/certs/client-key.pem".into()),
            ca_certificates: Some("/certs/ca.pem".into()),
            subject_alternative_names: vec![],
            sni: Some("svc.internal".into()),
        }),
    };
    roundtrip_json(&tp);
}
