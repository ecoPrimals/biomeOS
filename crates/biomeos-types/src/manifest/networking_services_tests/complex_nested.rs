// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::collections::HashMap;

use super::super::*;
use super::common::roundtrip_json;

#[test]
fn complex_nested_service_mesh_spec_roundtrip() {
    let spec = ServiceMeshSpec {
        mesh_type: ServiceMeshType::Istio,
        config: ServiceMeshConfig {
            mtls_enabled: true,
            telemetry: Some(MeshTelemetrySpec {
                tracing_enabled: true,
                metrics_enabled: true,
                access_logs_enabled: true,
                sampling_rate: Some(1.0),
            }),
            ingress: Some(MeshIngressSpec {
                gateways: vec![GatewaySpec {
                    name: "gateway".into(),
                    selector: std::iter::once(("app".into(), "istio-ingress".into())).collect(),
                    servers: vec![ServerSpec {
                        port: PortSpec {
                            number: 443,
                            name: "https".into(),
                            protocol: "TLS".into(),
                        },
                        hosts: vec!["*".into()],
                        tls: Some(TlsSpec {
                            mode: TlsMode::Simple,
                            credential_name: Some("cert".into()),
                            server_certificate: None,
                            private_key: None,
                        }),
                    }],
                }],
                virtual_services: vec![],
            }),
            egress: None,
        },
        traffic_policies: vec![],
        security_policies: vec![],
    };
    roundtrip_json(&spec);
}

#[test]
fn complex_nested_service_entry_spec_roundtrip() {
    let spec = ServiceEntrySpec {
        name: "external-api".into(),
        hosts: vec!["api.example.com".into()],
        ports: vec![ServiceEntryPort {
            number: 443,
            name: "https".into(),
            protocol: "TLS".into(),
        }],
        location: ServiceLocation::MeshExternal,
        resolution: ServiceResolution::DNS,
    };
    roundtrip_json(&spec);
}

#[test]
fn mesh_security_policy_spec_roundtrip() {
    let spec = MeshSecurityPolicySpec {
        name: "deny-all".into(),
        namespace: Some("default".into()),
        action: SecurityAction::Deny,
        rules: vec![SecurityRuleSpec {
            from: vec![Source {
                principals: vec!["*".into()],
                namespaces: vec![],
                ip_blocks: vec![],
            }],
            to: vec![Operation {
                hosts: vec!["*".into()],
                ports: vec!["*".into()],
                methods: vec![],
                paths: vec![],
            }],
            when: vec![Condition {
                key: "request.auth.claims[iss]".into(),
                values: vec!["https://accounts.google.com".into()],
            }],
        }],
    };
    roundtrip_json(&spec);
}

#[test]
fn mesh_egress_spec_roundtrip() {
    let spec = MeshEgressSpec {
        service_entries: vec![ServiceEntrySpec {
            name: "egress-wiki".into(),
            hosts: vec!["wiki.example.org".into()],
            ports: vec![ServiceEntryPort {
                number: 443,
                name: "https".into(),
                protocol: "TLS".into(),
            }],
            location: ServiceLocation::MeshExternal,
            resolution: ServiceResolution::DNS,
        }],
        destination_rules: vec![DestinationRuleSpec {
            name: "wiki-dr".into(),
            host: "wiki.example.org".into(),
            traffic_policy: None,
            subsets: vec![SubsetSpec {
                name: "v1".into(),
                labels: HashMap::from([("version".into(), "v1".into())]),
                traffic_policy: None,
            }],
        }],
    };
    roundtrip_json(&spec);
}

#[test]
fn mesh_egress_spec_destination_rules_only_roundtrip() {
    let spec = MeshEgressSpec {
        service_entries: vec![],
        destination_rules: vec![DestinationRuleSpec {
            name: "dr-only".into(),
            host: "svc.local".into(),
            traffic_policy: Some(TrafficPolicySpec {
                load_balancer: None,
                connection_pool: None,
                outlier_detection: None,
                tls: Some(ClientTlsSettings {
                    mode: ClientTlsMode::Simple,
                    client_certificate: None,
                    private_key: None,
                    ca_certificates: None,
                    subject_alternative_names: vec!["svc.local".into()],
                    sni: None,
                }),
            }),
            subsets: vec![],
        }],
    };
    roundtrip_json(&spec);
}

#[test]
fn complex_nested_traffic_policy_spec_roundtrip() {
    let spec = TrafficPolicySpec {
        load_balancer: Some(LoadBalancerSpec {
            simple: Some(LoadBalancerAlgorithm::RoundRobin),
            consistent_hash: Some(ConsistentHashSpec {
                http_header_name: Some("x-user-id".into()),
                http_cookie: Some(HttpCookieSpec {
                    name: "session".into(),
                    path: Some("/".into()),
                    ttl: Some(3600),
                }),
                use_source_ip: Some(false),
                ring_hash: Some(RingHashSpec {
                    minimum_ring_size: Some(1024),
                    maximum_ring_size: Some(65536),
                }),
            }),
        }),
        connection_pool: Some(ConnectionPoolSpec {
            tcp: Some(TcpSettingsSpec {
                max_connections: Some(100),
                connect_timeout: Some(5),
                tcp_no_delay: Some(true),
            }),
            http: Some(HttpSettingsSpec {
                http1_max_pending_requests: Some(1024),
                http2_max_requests: Some(1000),
                max_requests_per_connection: Some(2),
                max_retries: Some(3),
                idle_timeout: Some(30),
                h2_upgrade_policy: Some(H2UpgradePolicy::Upgrade),
            }),
        }),
        outlier_detection: Some(OutlierDetectionSpec {
            consecutive_errors: Some(5),
            interval: Some(10),
            base_ejection_time: Some(30),
            max_ejection_percent: Some(50),
            min_health_percent: Some(50),
        }),
        tls: Some(ClientTlsSettings {
            mode: ClientTlsMode::IstioMutual,
            client_certificate: None,
            private_key: None,
            ca_certificates: None,
            subject_alternative_names: vec![],
            sni: Some("reviews.example.com".into()),
        }),
    };
    roundtrip_json(&spec);
}
