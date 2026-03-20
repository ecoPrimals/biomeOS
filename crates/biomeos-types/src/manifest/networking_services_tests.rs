// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project
//
// Test module for manifest/networking_services.rs - included via #[path]

#![allow(clippy::unwrap_used)]

use super::*;
use std::collections::HashMap;

fn roundtrip_json<T: serde::Serialize + serde::de::DeserializeOwned>(value: &T) {
    let json = serde_json::to_string(value).unwrap();
    let parsed: T = serde_json::from_str(&json).unwrap();
    let json2 = serde_json::to_string(&parsed).unwrap();
    let v1: serde_json::Value = serde_json::from_str(&json).unwrap();
    let v2: serde_json::Value = serde_json::from_str(&json2).unwrap();
    assert_eq!(v1, v2, "roundtrip produced semantically different JSON");
}

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
fn service_mesh_type_roundtrip_all_variants() {
    roundtrip_json(&ServiceMeshType::Istio);
    roundtrip_json(&ServiceMeshType::Linkerd);
    roundtrip_json(&ServiceMeshType::Consul);
    roundtrip_json(&ServiceMeshType::Envoy);
    roundtrip_json(&ServiceMeshType::Custom("my-mesh".into()));
}

#[test]
fn tls_mode_roundtrip_all_variants() {
    roundtrip_json(&TlsMode::Passthrough);
    roundtrip_json(&TlsMode::Simple);
    roundtrip_json(&TlsMode::Mutual);
    roundtrip_json(&TlsMode::AutoPassthrough);
}

#[test]
fn load_balancer_algorithm_roundtrip_all_variants() {
    roundtrip_json(&LoadBalancerAlgorithm::RoundRobin);
    roundtrip_json(&LoadBalancerAlgorithm::LeastConn);
    roundtrip_json(&LoadBalancerAlgorithm::Random);
    roundtrip_json(&LoadBalancerAlgorithm::Passthrough);
}

#[test]
fn security_action_roundtrip_all_variants() {
    roundtrip_json(&SecurityAction::Allow);
    roundtrip_json(&SecurityAction::Deny);
    roundtrip_json(&SecurityAction::Audit);
}

#[test]
fn service_location_roundtrip_all_variants() {
    roundtrip_json(&ServiceLocation::MeshExternal);
    roundtrip_json(&ServiceLocation::MeshInternal);
}

#[test]
fn service_resolution_roundtrip_all_variants() {
    roundtrip_json(&ServiceResolution::None);
    roundtrip_json(&ServiceResolution::Static);
    roundtrip_json(&ServiceResolution::DNS);
}

#[test]
fn string_match_roundtrip_all_variants() {
    roundtrip_json(&StringMatch::Exact("exact".into()));
    roundtrip_json(&StringMatch::Prefix("prefix-".into()));
    roundtrip_json(&StringMatch::Regex("^/api/.*".into()));
}

#[test]
fn port_selector_roundtrip_all_variants() {
    roundtrip_json(&PortSelector::Number(443));
    roundtrip_json(&PortSelector::Name("https".into()));
}

#[test]
fn h2_upgrade_policy_roundtrip_all_variants() {
    roundtrip_json(&H2UpgradePolicy::Default);
    roundtrip_json(&H2UpgradePolicy::DoNotUpgrade);
    roundtrip_json(&H2UpgradePolicy::Upgrade);
}

#[test]
fn client_tls_mode_roundtrip_all_variants() {
    roundtrip_json(&ClientTlsMode::Disable);
    roundtrip_json(&ClientTlsMode::Simple);
    roundtrip_json(&ClientTlsMode::Mutual);
    roundtrip_json(&ClientTlsMode::IstioMutual);
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
fn complex_nested_virtual_service_spec_roundtrip() {
    let spec = VirtualServiceSpec {
        name: "reviews".into(),
        hosts: vec!["reviews".into()],
        gateways: vec!["mesh".into()],
        http: vec![HttpRouteSpec {
            match_conditions: vec![HttpMatchCondition {
                uri: Some(StringMatch::Prefix("/reviews".into())),
                scheme: None,
                method: Some(StringMatch::Exact("GET".into())),
                authority: None,
                headers: HashMap::new(),
                query_params: HashMap::new(),
            }],
            route: vec![HttpRouteDestination {
                destination: DestinationSpec {
                    host: "reviews".into(),
                    subset: Some("v1".into()),
                    port: Some(PortSelector::Number(9080)),
                },
                weight: Some(100),
                headers: None,
            }],
            redirect: None,
            rewrite: None,
            timeout: Some(10),
            retries: Some(HttpRetry {
                attempts: 3,
                per_try_timeout: Some(5),
                retry_on: Some("5xx".into()),
            }),
        }],
        tcp: vec![],
        tls: vec![],
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
