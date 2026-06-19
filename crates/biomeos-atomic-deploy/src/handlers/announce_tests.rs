// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Tests for `handlers/announce.rs` (`primal.announce`).

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use biomeos_types::env_config::vars;
use serde_json::{Value, json};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::capability_translation::CapabilityTranslationRegistry;
use crate::neural_router::NeuralRouter;

use super::LifecycleHandler;
use super::announce::handle_announce;

const EMPTY_ENV: [(&str, Option<&str>); 0] = [];

fn make_ionic_token(payload: &Value) -> String {
    use base64::Engine;
    let header = json!({"alg":"EdDSA","typ":"ionic","ver":1});
    let h = base64::engine::general_purpose::STANDARD.encode(header.to_string().as_bytes());
    let p = base64::engine::general_purpose::STANDARD.encode(payload.to_string().as_bytes());
    let s = base64::engine::general_purpose::STANDARD.encode(b"fake-sig");
    format!("{h}.{p}.{s}")
}

fn announce_fixture() -> (
    Arc<NeuralRouter>,
    Arc<RwLock<CapabilityTranslationRegistry>>,
    LifecycleHandler,
) {
    let router = Arc::new(NeuralRouter::new("announce-test-family"));
    let registry = Arc::new(RwLock::new(CapabilityTranslationRegistry::new()));
    let lifecycle = LifecycleHandler::new("announce-test-family");
    (router, registry, lifecycle)
}

#[tokio::test]
async fn successful_announce_registers_capabilities_methods_and_signal_tiers() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let (router, registry, lifecycle) = announce_fixture();
        let params = Some(json!({
            "primal": "beardog",
            "socket": "/tmp/beardog-announce.sock",
            "pid": 4242,
            "capabilities": ["crypto", "security"],
            "methods": ["crypto.encrypt", "crypto.sign"],
            "semantic_mappings": {
                "sha256": "crypto.blake3_hash"
            },
            "signal_tiers": ["tower", "node", "invalid-tier"],
            "cost_hints": { "crypto": 5.0 },
            "latency_estimates": { "crypto": 10 }
        }));

        let result = handle_announce(&router, &registry, &lifecycle, &None, &params)
            .await
            .expect("announce should succeed");

        assert_eq!(result["primal"], "beardog");
        assert_eq!(result["capabilities_registered"], 2);
        assert_eq!(result["methods_registered"], 3);
        assert_eq!(
            result["signal_tiers_joined"].as_array().unwrap(),
            &vec![json!("tower"), json!("node")]
        );
        assert_eq!(result["attestation_verified"], false);

        let caps = router.list_capabilities().await;
        assert!(caps.contains_key("crypto"));
        assert!(caps.contains_key("security"));
        let providers = caps.get("crypto").expect("crypto providers");
        assert_eq!(providers[0].primal_name.as_ref(), "beardog");

        let translations = registry.read().await;
        assert!(translations.get_translation("crypto.encrypt").is_some());
        assert!(translations.get_translation("sha256").is_some());

        let status = lifecycle.status().await.expect("lifecycle status");
        assert_eq!(status["count"], 1);
    })
    .await;
}

#[tokio::test]
async fn announce_rejects_missing_parameters() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let (router, registry, lifecycle) = announce_fixture();
        let err = handle_announce(&router, &registry, &lifecycle, &None, &None)
            .await
            .expect_err("None params should fail");
        assert!(err.to_string().contains("Missing parameters"));
    })
    .await;
}

#[tokio::test]
async fn announce_rejects_invalid_payload_shape() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let (router, registry, lifecycle) = announce_fixture();

        let err = handle_announce(
            &router,
            &registry,
            &lifecycle,
            &None,
            &Some(json!({ "primal": 42, "socket": "/tmp/x.sock" })),
        )
        .await
        .expect_err("wrong primal type");
        assert!(
            err.to_string().contains("Invalid primal.announce payload"),
            "unexpected error: {err}"
        );

        let err = handle_announce(
            &router,
            &registry,
            &lifecycle,
            &None,
            &Some(json!({ "socket": "/tmp/x.sock" })),
        )
        .await
        .expect_err("missing primal");
        assert!(
            err.to_string().contains("Invalid primal.announce payload"),
            "unexpected error: {err}"
        );
    })
    .await;
}

#[tokio::test]
async fn announce_auto_registers_domains_for_methods_missing_from_capabilities() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let (router, registry, lifecycle) = announce_fixture();
        let params = Some(json!({
            "primal": "songbird",
            "socket": "/tmp/songbird-announce.sock",
            "capabilities": [],
            "methods": ["mesh.announce", "relay.allocate"]
        }));

        let result = handle_announce(&router, &registry, &lifecycle, &None, &params)
            .await
            .expect("announce with inferred domains");

        assert_eq!(result["capabilities_registered"], 0);
        assert_eq!(result["methods_registered"], 2);

        let caps = router.list_capabilities().await;
        assert!(
            caps.contains_key("mesh"),
            "method domain should be registered on router when absent from capabilities"
        );
        assert!(caps.contains_key("relay"));

        let translations = registry.read().await;
        assert!(translations.get_translation("mesh.announce").is_some());
        assert!(translations.get_translation("relay.allocate").is_some());
    })
    .await;
}

#[tokio::test]
async fn announce_with_empty_capabilities_succeeds_with_zero_registrations() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let (router, registry, lifecycle) = announce_fixture();
        let params = Some(json!({
            "primal": "minimal",
            "socket": "/tmp/minimal.sock",
            "capabilities": [],
            "methods": []
        }));

        let result = handle_announce(&router, &registry, &lifecycle, &None, &params)
            .await
            .expect("minimal announce");

        assert_eq!(result["primal"], "minimal");
        assert_eq!(result["capabilities_registered"], 0);
        assert_eq!(result["methods_registered"], 0);
        assert_eq!(result["signal_tiers_joined"].as_array().unwrap().len(), 0);
        assert!(router.list_capabilities().await.is_empty());
    })
    .await;
}

#[tokio::test]
async fn announce_verifies_ionic_attestation_via_local_fallback() {
    temp_env::async_with_vars(
        [(vars::BEARDOG_SOCKET, Some("/tmp/nonexistent-beardog.sock"))],
        async {
            let (router, registry, lifecycle) = announce_fixture();
            let attestation = make_ionic_token(&json!({
                "sub": "beardog",
                "scope": ["*"],
                "exp": 9_999_999_999_u64
            }));
            let verifier = Some(biomeos_core::SecurityVerifier::new(
                std::path::PathBuf::from("/tmp/nonexistent-beardog.sock"),
            ));
            let params = Some(json!({
                "primal": "attested",
                "socket": "/tmp/attested.sock",
                "capabilities": ["crypto"],
                "attestation": attestation
            }));

            let result = handle_announce(&router, &registry, &lifecycle, &verifier, &params)
                .await
                .expect("attested announce");

            assert_eq!(result["attestation_verified"], true);
        },
    )
    .await;
}

#[tokio::test]
async fn announce_marks_invalid_attestation_unverified() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let (router, registry, lifecycle) = announce_fixture();
        let verifier = Some(biomeos_core::SecurityVerifier::new(
            std::path::PathBuf::from("/tmp/nonexistent-beardog.sock"),
        ));
        let params = Some(json!({
            "primal": "unverified",
            "socket": "/tmp/unverified.sock",
            "capabilities": ["crypto"],
            "attestation": "not-a-valid-token"
        }));

        let result = handle_announce(&router, &registry, &lifecycle, &verifier, &params)
            .await
            .expect("announce with bad attestation");

        assert_eq!(result["attestation_verified"], false);
    })
    .await;
}

#[tokio::test]
async fn announce_with_attestation_skips_verification_when_verifier_absent() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let (router, registry, lifecycle) = announce_fixture();
        let params = Some(json!({
            "primal": "attested-no-verifier",
            "socket": "/tmp/attested-no-verifier.sock",
            "capabilities": ["crypto"],
            "attestation": make_ionic_token(&json!({
                "sub": "attested-no-verifier",
                "scope": ["*"],
                "exp": 9_999_999_999_u64
            }))
        }));

        let result = handle_announce(&router, &registry, &lifecycle, &None, &params)
            .await
            .expect("announce with attestation but no verifier");

        assert_eq!(result["attestation_verified"], false);
        assert_eq!(result["capabilities_registered"], 1);
    })
    .await;
}

#[tokio::test]
async fn announce_seeds_routing_weights_from_cost_and_latency_hints() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let (router, registry, lifecycle) = announce_fixture();
        let params = Some(json!({
            "primal": "weighted",
            "socket": "/tmp/weighted.sock",
            "capabilities": ["compute"],
            "cost_hints": { "compute": 12.5 },
            "latency_estimates": { "compute": 75 }
        }));

        let result = handle_announce(&router, &registry, &lifecycle, &None, &params)
            .await
            .expect("announce with routing hints");

        assert_eq!(result["capabilities_registered"], 1);

        let weights = router.get_routing_weights().await;
        let w = weights
            .iter()
            .find(|w| w.provider.as_ref() == "weighted" && w.capability.as_ref() == "compute")
            .expect("compute weight for weighted");
        assert_eq!(w.cost_hint, Some(12.5));
        assert!((w.affinity - 0.6).abs() < f64::EPSILON);
    })
    .await;
}

#[tokio::test]
async fn announce_ignores_non_string_semantic_mapping_values() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let (router, registry, lifecycle) = announce_fixture();
        let params = Some(json!({
            "primal": "semantic",
            "socket": "/tmp/semantic.sock",
            "capabilities": ["crypto"],
            "methods": ["crypto.hash"],
            "semantic_mappings": {
                "consumer_hash": "crypto.hash",
                "ignored_number": 42,
                "ignored_object": { "nested": "crypto.hash" }
            }
        }));

        let result = handle_announce(&router, &registry, &lifecycle, &None, &params)
            .await
            .expect("announce with mixed semantic mappings");

        assert_eq!(result["methods_registered"], 2);
        let translations = registry.read().await;
        assert!(translations.get_translation("consumer_hash").is_some());
        assert!(translations.get_translation("ignored_number").is_none());
        assert!(translations.get_translation("ignored_object").is_none());
    })
    .await;
}

#[tokio::test]
async fn announce_parses_tcp_socket_endpoint() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let (router, registry, lifecycle) = announce_fixture();
        let params = Some(json!({
            "primal": "tcp-primal",
            "socket": "tcp://127.0.0.1:9876",
            "capabilities": ["network"]
        }));

        let result = handle_announce(&router, &registry, &lifecycle, &None, &params)
            .await
            .expect("tcp socket announce");

        assert_eq!(result["capabilities_registered"], 1);
        let caps = router.list_capabilities().await;
        let providers = caps.get("network").expect("network capability");
        assert_eq!(providers[0].primal_name.as_ref(), "tcp-primal");
        assert!(registry.read().await.list_translations("network").is_none());
    })
    .await;
}

#[tokio::test]
async fn announce_skips_methods_without_domain_separator() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let (router, registry, lifecycle) = announce_fixture();
        let params = Some(json!({
            "primal": "bare",
            "socket": "/tmp/bare.sock",
            "capabilities": ["compute"],
            "methods": ["nodomainmethod", "compute.run"]
        }));

        let result = handle_announce(&router, &registry, &lifecycle, &None, &params)
            .await
            .expect("announce");

        assert_eq!(result["methods_registered"], 1);
        let translations = registry.read().await;
        assert!(translations.get_translation("compute.run").is_some());
        assert!(translations.get_translation("nodomainmethod").is_none());
    })
    .await;
}
