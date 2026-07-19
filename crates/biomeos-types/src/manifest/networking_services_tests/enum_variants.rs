// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;
use super::common::roundtrip_json;

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
