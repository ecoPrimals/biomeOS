// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::coordinator::NucleusClient;
use super::common::{
    MockCap, MockIdentityAcceptName, MockPhysical, MockTrust, sample_discovered, sample_endpoint,
    sample_proof, test_client,
};
use crate::Registry;
use crate::discovery::DiscoveryRequest;
use crate::trust::TrustLevel;
use biomeos_types::CapabilityTaxonomy;
use std::sync::Arc;

#[tokio::test]
async fn test_discover_happy_path_one_primal() {
    let p = sample_discovered("beardog", vec![sample_endpoint()]);
    let client = test_client(vec![p], true, "beardog", false, false);
    let req = DiscoveryRequest::new(CapabilityTaxonomy::Encryption);
    let out = client.discover(req).await.expect("discover");
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].name, "beardog");
    assert_eq!(out[0].trust_level, TrustLevel::Verified);
}

#[tokio::test]
async fn test_discover_empty_when_layer1_empty() {
    let client = test_client(vec![], true, "x", false, false);
    let req = DiscoveryRequest::new(CapabilityTaxonomy::Encryption);
    let out = client.discover(req).await.expect("discover");
    assert!(out.is_empty());
}

#[tokio::test]
async fn test_discover_skips_when_identity_fails() {
    let p = sample_discovered("p1", vec![sample_endpoint()]);
    let client = test_client(vec![p], false, "p1", false, false);
    let req = DiscoveryRequest::new(CapabilityTaxonomy::Encryption);
    let out = client.discover(req).await.expect("discover");
    assert!(out.is_empty());
}

#[tokio::test]
async fn test_discover_skips_when_capability_fails() {
    let p = sample_discovered("p1", vec![sample_endpoint()]);
    let client = test_client(vec![p], true, "p1", true, false);
    let req = DiscoveryRequest::new(CapabilityTaxonomy::Encryption);
    let out = client.discover(req).await.expect("discover");
    assert!(out.is_empty());
}

#[tokio::test]
async fn test_discover_uses_known_when_trust_fails() {
    let p = sample_discovered("p1", vec![sample_endpoint()]);
    let client = test_client(vec![p], true, "p1", false, true);
    let req = DiscoveryRequest::new(CapabilityTaxonomy::Encryption);
    let out = client.discover(req).await.expect("discover");
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].trust_level, TrustLevel::Known);
}

#[tokio::test]
async fn test_discover_err_when_no_endpoints() {
    let p = sample_discovered("p1", vec![]);
    let client = test_client(vec![p], true, "p1", false, false);
    let req = DiscoveryRequest::new(CapabilityTaxonomy::Encryption);
    let err = client.discover(req).await.unwrap_err();
    assert!(err.to_string().contains("No endpoints") || err.to_string().contains("endpoints"));
}

#[tokio::test]
async fn test_discover_second_primal_after_first_skipped() {
    let bad = sample_discovered("bad", vec![sample_endpoint()]);
    let good = sample_discovered("good", vec![sample_endpoint()]);
    let client = NucleusClient::from_layers_for_test(
        Arc::new(MockPhysical {
            out: vec![bad, good],
        }),
        Arc::new(MockIdentityAcceptName {
            accept: "good",
            proof: sample_proof("good"),
        }),
        Arc::new(MockCap { fail: false }),
        Arc::new(MockTrust { err: false }),
        Arc::new(Registry::new()),
    );
    let req = DiscoveryRequest::new(CapabilityTaxonomy::Encryption);
    let out = client.discover(req).await.expect("discover");
    assert_eq!(out.len(), 1);
    assert_eq!(out[0].name, "good");
}

#[tokio::test]
async fn test_registry_accessor_from_injected_client() {
    let client = test_client(vec![], true, "x", false, false);
    let reg = client.registry();
    assert!(std::sync::Arc::strong_count(&reg) >= 1);
}
