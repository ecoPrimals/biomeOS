// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::collections::HashMap;

use super::super::*;
use super::common::roundtrip_json;

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
fn tcp_route_spec_roundtrip() {
    let spec = TcpRouteSpec {
        match_conditions: vec![TcpMatchCondition {
            destination_subnets: vec!["10.0.0.0/8".into()],
            port: Some(9000),
            source_labels: HashMap::new(),
            gateways: vec!["mesh".into()],
        }],
        route: vec![TcpRouteDestination {
            destination: DestinationSpec {
                host: "reviews".into(),
                subset: None,
                port: Some(PortSelector::Number(9080)),
            },
            weight: Some(100),
        }],
    };
    roundtrip_json(&spec);
}

#[test]
fn tls_route_spec_roundtrip() {
    let spec = TlsRouteSpec {
        match_conditions: vec![TlsMatchCondition {
            sni_hosts: vec!["api.example.com".into()],
            destination_subnets: vec![],
            port: Some(443),
            source_labels: HashMap::new(),
            gateways: vec![],
        }],
        route: vec![TlsRouteDestination {
            destination: DestinationSpec {
                host: "api.example.com".into(),
                subset: Some("stable".into()),
                port: None,
            },
            weight: None,
        }],
    };
    roundtrip_json(&spec);
}

#[test]
fn http_redirect_and_rewrite_roundtrip() {
    roundtrip_json(&HttpRedirect {
        uri: Some("/new".into()),
        authority: Some("other.example".into()),
        redirect_code: Some(302),
    });
    roundtrip_json(&HttpRewrite {
        uri: Some("/v2".into()),
        authority: None,
    });
}

#[test]
fn subset_spec_with_traffic_policy_roundtrip() {
    let spec = SubsetSpec {
        name: "canary".into(),
        labels: HashMap::new(),
        traffic_policy: Some(TrafficPolicySpec {
            load_balancer: Some(LoadBalancerSpec {
                simple: Some(LoadBalancerAlgorithm::LeastConn),
                consistent_hash: None,
            }),
            connection_pool: None,
            outlier_detection: None,
            tls: None,
        }),
    };
    roundtrip_json(&spec);
}

#[test]
fn http_route_with_redirect_instead_of_route_roundtrip() {
    let spec = VirtualServiceSpec {
        name: "redirector".into(),
        hosts: vec!["old.example".into()],
        gateways: vec!["mesh".into()],
        http: vec![HttpRouteSpec {
            match_conditions: vec![HttpMatchCondition {
                uri: Some(StringMatch::Prefix("/".into())),
                scheme: None,
                method: None,
                authority: None,
                headers: HashMap::new(),
                query_params: HashMap::new(),
            }],
            route: vec![],
            redirect: Some(HttpRedirect {
                uri: Some("https://new.example".into()),
                authority: None,
                redirect_code: Some(301),
            }),
            rewrite: None,
            timeout: None,
            retries: None,
        }],
        tcp: vec![],
        tls: vec![],
    };
    roundtrip_json(&spec);
}

#[test]
fn http_route_rewrite_preserves_method_match_roundtrip() {
    let spec = VirtualServiceSpec {
        name: "rewrite-only".into(),
        hosts: vec!["api".into()],
        gateways: vec![],
        http: vec![HttpRouteSpec {
            match_conditions: vec![HttpMatchCondition {
                uri: Some(StringMatch::Exact("/v1".into())),
                scheme: None,
                method: Some(StringMatch::Exact("POST".into())),
                authority: None,
                headers: HashMap::new(),
                query_params: HashMap::new(),
            }],
            route: vec![HttpRouteDestination {
                destination: DestinationSpec {
                    host: "api".into(),
                    subset: None,
                    port: Some(PortSelector::Name("http".into())),
                },
                weight: Some(100),
                headers: None,
            }],
            redirect: None,
            rewrite: Some(HttpRewrite {
                uri: Some("/v2".into()),
                authority: Some("internal".into()),
            }),
            timeout: Some(5),
            retries: None,
        }],
        tcp: vec![],
        tls: vec![],
    };
    roundtrip_json(&spec);
}

#[test]
fn gateway_spec_and_server_spec_roundtrip() {
    let spec = MeshIngressSpec {
        gateways: vec![GatewaySpec {
            name: "gw".into(),
            selector: HashMap::from([("istio".into(), "ingress".into())]),
            servers: vec![ServerSpec {
                port: PortSpec {
                    number: 8443,
                    name: "https".into(),
                    protocol: "HTTPS".into(),
                },
                hosts: vec!["*".into()],
                tls: Some(TlsSpec {
                    mode: TlsMode::Mutual,
                    credential_name: None,
                    server_certificate: Some("cert.pem".into()),
                    private_key: Some("key.pem".into()),
                }),
            }],
        }],
        virtual_services: vec![],
    };
    roundtrip_json(&spec);
}
