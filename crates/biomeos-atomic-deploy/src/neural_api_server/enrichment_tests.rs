// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Tests for `neural_api_server/enrichment.rs` forwarding param enrichment.

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use biomeos_core::TransportEndpoint;
use biomeos_core::method_gate::CallerContext;
use biomeos_types::env_config::vars;
use serde_json::{Value, json};
use std::path::PathBuf;

use super::NeuralApiServer;

const EMPTY_ENV: [(&str, Option<&str>); 0] = [];

fn make_ionic_token(payload: &Value) -> String {
    use base64::Engine;
    let header = json!({"alg":"EdDSA","typ":"ionic","ver":1});
    let h = base64::engine::general_purpose::STANDARD.encode(header.to_string().as_bytes());
    let p = base64::engine::general_purpose::STANDARD.encode(payload.to_string().as_bytes());
    let s = base64::engine::general_purpose::STANDARD.encode(b"fake-sig");
    format!("{h}.{p}.{s}")
}

fn create_test_server() -> NeuralApiServer {
    let temp = tempfile::tempdir().expect("tempdir");
    NeuralApiServer::new(
        temp.path(),
        "enrichment-test-family",
        temp.path().join("n.sock"),
    )
}

#[tokio::test]
async fn enrich_none_params_yields_empty_object() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let server = create_test_server();
        let caller = CallerContext::loopback();
        let enriched = server
            .enrich_for_forwarding(&None, &caller)
            .await
            .expect("enrichment result");
        assert_eq!(enriched, json!({}));
        assert!(enriched.as_object().unwrap().is_empty());
    })
    .await;
}

#[tokio::test]
async fn enrich_preserves_existing_request_fields() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let server = create_test_server();
        let params = Some(json!({
            "query": "status",
            "limit": 25
        }));
        let caller = CallerContext::loopback();
        let enriched = server
            .enrich_for_forwarding(&params, &caller)
            .await
            .expect("enrichment result");

        assert_eq!(enriched["query"], "status");
        assert_eq!(enriched["limit"], 25);
        assert!(enriched.get("_resource_envelope").is_none());
        assert!(enriched.get("_bearer_token").is_none());
        assert!(enriched.get("_token_verified").is_none());
    })
    .await;
}

#[tokio::test]
async fn enrich_injects_resource_envelope_from_ionic_claims() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let server = create_test_server();
        let token = make_ionic_token(&json!({
            "sub": "researcher",
            "scope": ["compute.*"],
            "exp": 9_999_999_999_u64,
            "resources": {
                "mem": 4096,
                "cpu": 2.5,
                "timeout_ms": 30_000,
                "method_allowlist": ["compute.run"]
            }
        }));
        let caller = CallerContext::loopback().with_bearer_token(token);
        let params = Some(json!({ "job": "train" }));

        let enriched = server
            .enrich_for_forwarding(&params, &caller)
            .await
            .expect("enrichment result");

        assert_eq!(enriched["job"], "train");
        let envelope = enriched
            .get("_resource_envelope")
            .expect("resource envelope injected");
        assert_eq!(envelope["mem"], 4096);
        assert_eq!(envelope["cpu"], 2.5);
        assert_eq!(envelope["timeout_ms"], 30_000);
        assert_eq!(envelope["method_allowlist"], json!(["compute.run"]));
    })
    .await;
}

#[tokio::test]
async fn enrich_injects_bearer_token_and_verification_flag() {
    temp_env::async_with_vars(
        [(vars::BEARDOG_SOCKET, Some("/tmp/nonexistent-beardog.sock"))],
        async {
            let server = create_test_server();
            let token = make_ionic_token(&json!({
                "sub": "caller",
                "scope": ["*"],
                "exp": 9_999_999_999_u64
            }));
            let caller = CallerContext::loopback().with_bearer_token(token.clone());
            let enriched = server
                .enrich_for_forwarding(&Some(json!({})), &caller)
                .await
                .expect("enrichment result");

            assert_eq!(enriched["_bearer_token"], json!(token));
            assert_eq!(
                enriched["_token_verified"], true,
                "ionic token should verify locally when BearDog IPC is unreachable"
            );
        },
    )
    .await;
}

#[tokio::test]
async fn enrich_injects_all_forwarding_headers_together() {
    temp_env::async_with_vars(
        [(vars::BEARDOG_SOCKET, Some("/tmp/nonexistent-beardog.sock"))],
        async {
            let server = create_test_server();
            let token = make_ionic_token(&json!({
                "sub": "researcher",
                "scope": ["compute.*"],
                "exp": 9_999_999_999_u64,
                "resources": {
                    "mem": 2048,
                    "cpu": 1.0,
                    "timeout_ms": 15_000,
                    "method_allowlist": ["compute.run"]
                }
            }));
            let caller = CallerContext::loopback().with_bearer_token(token.clone());
            let params = Some(json!({ "workload": "batch" }));

            let enriched = server
                .enrich_for_forwarding(&params, &caller)
                .await
                .expect("enrichment result");

            assert_eq!(enriched["workload"], "batch");
            assert_eq!(enriched["_bearer_token"], json!(token));
            assert_eq!(enriched["_token_verified"], true);
            assert_eq!(enriched["_resource_envelope"]["mem"], 2048);
        },
    )
    .await;
}

#[tokio::test]
async fn enrich_missing_context_leaves_forwarding_headers_absent() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let server = create_test_server();
        let caller = CallerContext::unix();
        let enriched = server
            .enrich_for_forwarding(&Some(json!({ "payload": true })), &caller)
            .await
            .expect("enrichment result");

        assert_eq!(enriched["payload"], true);
        assert!(enriched.get("_resource_envelope").is_none());
        assert!(enriched.get("_bearer_token").is_none());
        assert!(enriched.get("_token_verified").is_none());
    })
    .await;
}

#[tokio::test]
async fn enrich_non_object_params_skips_header_injection() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let server = create_test_server();
        let caller = CallerContext::loopback();
        let params = Some(json!(["array", "payload"]));
        let enriched = server
            .enrich_for_forwarding(&params, &caller)
            .await
            .expect("enrichment result");

        assert_eq!(enriched, json!(["array", "payload"]));
        assert!(enriched.get("_resource_envelope").is_none());
        assert!(enriched.get("_bearer_token").is_none());
        assert!(enriched.get("_token_verified").is_none());
    })
    .await;
}

#[tokio::test]
async fn enrich_token_verified_false_when_security_verifier_absent() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let mut server = create_test_server();
        server.security_verifier = None;
        let caller = CallerContext::loopback()
            .with_bearer_token("opaque-token-without-verifier".to_string());
        let enriched = server
            .enrich_for_forwarding(&Some(json!({ "op": "run" })), &caller)
            .await
            .expect("enrichment result");

        assert_eq!(enriched["op"], "run");
        assert_eq!(
            enriched["_bearer_token"],
            json!("opaque-token-without-verifier")
        );
        assert_eq!(enriched["_token_verified"], false);
    })
    .await;
}

#[tokio::test]
async fn enrich_bearer_without_resource_claims_omits_envelope() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let server = create_test_server();
        let caller =
            CallerContext::loopback().with_bearer_token("opaque-non-ionic-token".to_string());
        let enriched = server
            .enrich_for_forwarding(&Some(json!({ "op": "sign" })), &caller)
            .await
            .expect("enrichment result");

        assert_eq!(enriched["op"], "sign");
        assert!(enriched.get("_resource_envelope").is_none());
        assert_eq!(enriched["_bearer_token"], json!("opaque-non-ionic-token"));
        assert_eq!(enriched["_token_verified"], false);
    })
    .await;
}

#[tokio::test]
async fn weight_health_reports_routing_convergence_diagnostics() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let server = create_test_server();
        let health = server
            .handle_weight_health()
            .await
            .expect("weight health response");

        assert_eq!(health["healthy"], true);
        assert!(health.get("persistent").is_some());
        assert!(health.get("summary").is_some());
        assert_eq!(health["convergence"]["converging"], 0);
        assert_eq!(health["convergence"]["cold"], 0);
        assert_eq!(health["convergence"]["total_providers"], 0);
        assert_eq!(health["open_circuits"].as_array().unwrap().len(), 0);
        assert_eq!(health["shadow_routing"]["phase"], "active");
        assert!(health.get("perceptron").is_some());
        assert_eq!(health["training_data_buffered"], 0);
    })
    .await;
}

#[tokio::test]
async fn weight_health_reports_open_circuits_and_convergence_buckets() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let server = create_test_server();
        let endpoint = TransportEndpoint::UnixSocket {
            path: PathBuf::from("/tmp/enrichment-broken.sock"),
        };
        server
            .router
            .register_capability("crypto", "broken", endpoint.clone(), "test")
            .await
            .expect("register broken");
        server
            .router
            .register_capability("crypto", "healthy", endpoint, "test")
            .await
            .expect("register healthy");

        for _ in 0..5 {
            server
                .router
                .record_dispatch_outcome("crypto", "broken", false, 0)
                .await;
        }
        for _ in 0..10 {
            server
                .router
                .record_dispatch_outcome("crypto", "healthy", true, 20)
                .await;
        }

        let health = server
            .handle_weight_health()
            .await
            .expect("weight health with populated router");

        assert_eq!(health["healthy"], false);
        assert_eq!(health["convergence"]["converging"], 1);
        assert_eq!(health["convergence"]["cold"], 1);
        assert_eq!(health["convergence"]["total_providers"], 2);
        let open = health["open_circuits"].as_array().expect("open circuits");
        assert_eq!(open.len(), 1);
        assert_eq!(open[0]["provider"], "broken");
        assert_eq!(open[0]["capability"], "crypto");
        assert_eq!(open[0]["consecutive_failures"], 5);
    })
    .await;
}

#[tokio::test]
async fn training_data_drain_returns_rows_and_clears_buffer() {
    temp_env::async_with_vars(EMPTY_ENV, async {
        let server = create_test_server();
        let drained = server
            .handle_training_data_drain()
            .await
            .expect("training drain");

        assert_eq!(drained["count"], 0);
        assert_eq!(drained["rows"].as_array().unwrap().len(), 0);
        assert_eq!(
            drained["feature_dim"],
            crate::neural_router::perceptron::FEATURE_DIM
        );

        let again = server
            .handle_training_data_drain()
            .await
            .expect("second drain");
        assert_eq!(again["count"], 0);
    })
    .await;
}
