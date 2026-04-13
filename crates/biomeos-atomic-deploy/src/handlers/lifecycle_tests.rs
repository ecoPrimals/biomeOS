// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project
//
// Sibling tests for `lifecycle.rs` (handler API surface).

#![expect(clippy::unwrap_used, reason = "test")]
#![expect(clippy::expect_used, reason = "test")]

use serde_json::json;

use super::lifecycle::LifecycleHandler;
use crate::neural_graph::GraphNode;

#[tokio::test]
async fn status_empty_reports_zero_healthy() {
    let handler = LifecycleHandler::new("sibling-lc-fam");
    let status = handler.status().await.expect("status");
    assert_eq!(status["count"], 0);
    assert_eq!(status["healthy"], 0);
    assert!(status["primals"].as_array().unwrap().is_empty());
}

#[tokio::test]
async fn shutdown_all_on_empty_manager_completes() {
    let handler = LifecycleHandler::new("sibling-lc-shut");
    let out = handler.shutdown_all().await.expect("shutdown_all");
    assert_eq!(out["shutdown"], "complete");
    let status = handler.status().await.expect("status");
    assert_eq!(status["count"], 0);
}

#[tokio::test]
async fn resurrect_rejects_missing_name_field() {
    let handler = LifecycleHandler::new("sibling-lc-res");
    let err = handler
        .resurrect(&Some(json!({})))
        .await
        .expect_err("missing name");
    assert!(err.to_string().contains("name"));
}

#[tokio::test]
async fn apoptosis_rejects_missing_name_field() {
    let handler = LifecycleHandler::new("sibling-lc-apo");
    let err = handler
        .apoptosis(&Some(json!({ "reason": "user_request" })))
        .await
        .expect_err("missing name");
    assert!(err.to_string().contains("name"));
}

#[tokio::test]
async fn get_success_includes_family_and_metrics_block() {
    let handler = LifecycleHandler::new("sibling-lc-get");
    handler
        .register(&Some(json!({
            "name": "p-sibling",
            "socket_path": "/tmp/p-sibling.sock",
            "pid": 7
        })))
        .await
        .expect("register");
    let g = handler
        .get(&Some(json!({ "name": "p-sibling" })))
        .await
        .expect("get");
    assert_eq!(g["family_id"], "sibling-lc-get");
    assert!(g.get("metrics").is_some());
    assert_eq!(g["state"], "incubating");
}

#[tokio::test]
async fn status_increments_count_after_register() {
    let handler = LifecycleHandler::new("sibling-lc-st");
    handler
        .register(&Some(json!({
            "name": "only-one",
            "socket_path": "/tmp/only-one.sock"
        })))
        .await
        .expect("register");
    let status = handler.status().await.expect("status");
    assert_eq!(status["count"], 1);
    let primals = status["primals"].as_array().unwrap();
    assert_eq!(primals[0]["name"], "only-one");
}

#[tokio::test]
async fn apoptosis_default_reason_string_when_omitted() {
    let handler = LifecycleHandler::new("sibling-lc-reason");
    handler
        .register(&Some(json!({
            "name": "r1",
            "socket_path": "/tmp/r1.sock",
            "pid": 1
        })))
        .await
        .expect("register");
    let out = handler
        .apoptosis(&Some(json!({ "name": "r1" })))
        .await
        .expect("apoptosis");
    assert_eq!(out["reason"], "user_request");
}

// =========================================================================
// Enriched composition dashboard (lifecycle.composition)
// =========================================================================

fn make_graph_node(id: &str, depends_on: Vec<String>) -> GraphNode {
    GraphNode {
        id: id.to_string(),
        depends_on,
        ..Default::default()
    }
}

#[tokio::test]
async fn composition_enriched_includes_capabilities_and_edges() {
    let handler = LifecycleHandler::new("test-family");

    let mut node_with_caps = make_graph_node("beardog", vec![]);
    node_with_caps.capabilities = vec!["crypto".to_string(), "security".to_string()];

    handler
        .register(&Some(json!({
            "name": "beardog",
            "socket_path": "/tmp/beardog.sock",
            "pid": 100,
            "deployment_node": serde_json::to_value(&node_with_caps).unwrap()
        })))
        .await
        .expect("register beardog");

    let songbird_node = make_graph_node("songbird", vec!["beardog".to_string()]);
    handler
        .register(&Some(json!({
            "name": "songbird",
            "socket_path": "/tmp/songbird.sock",
            "pid": 200,
            "deployment_node": serde_json::to_value(&songbird_node).unwrap()
        })))
        .await
        .expect("register songbird");

    let comp = handler.composition().await.expect("composition");

    assert_eq!(comp["total"], 2);
    let caps = comp["capabilities_live"]
        .as_array()
        .expect("capabilities array");
    assert!(caps.iter().any(|c| c == "crypto"));
    assert!(caps.iter().any(|c| c == "security"));

    let edges = comp["dependency_graph"].as_array().expect("edges array");
    assert!(
        edges
            .iter()
            .any(|e| e["from"] == "beardog" && e["to"] == "songbird"),
        "dependency edge beardog→songbird should exist"
    );

    let all_primals: Vec<&serde_json::Value> = comp["degraded"]
        .as_array()
        .unwrap()
        .iter()
        .chain(comp["active"].as_array().unwrap().iter())
        .chain(comp["dead"].as_array().unwrap().iter())
        .collect();
    for p in &all_primals {
        assert!(p.get("capabilities").is_some());
        assert!(p.get("health").is_some());
        assert!(p.get("state_details").is_some());
        assert!(p.get("depends_on").is_some());
    }
}

#[tokio::test]
async fn composition_empty_returns_healthy_defaults() {
    let handler = LifecycleHandler::new("test-family");
    let comp = handler.composition().await.expect("composition");
    assert_eq!(comp["total"], 0);
    assert_eq!(comp["health_ratio"], 1.0);
    assert!(comp["composition_healthy"].as_bool().unwrap());
    assert!(comp["capabilities_live"].as_array().unwrap().is_empty());
    assert!(comp["dependency_graph"].as_array().unwrap().is_empty());
}

// =========================================================================
// Composition health (COMPOSITION_HEALTH_STANDARD)
// =========================================================================

#[tokio::test]
async fn composition_health_empty_returns_unavailable_subsystems() {
    let handler = LifecycleHandler::new("test-family");
    let health = handler.composition_health(&None).await.expect("health");
    assert!(health["healthy"].as_bool().is_some());
    assert!(health["deploy_graph"].as_str().is_some());
    assert!(health["subsystems"].is_object());
    let subs = health["subsystems"].as_object().unwrap();
    assert_eq!(subs["tower"], "unavailable");
    assert_eq!(subs["mesh"], "unavailable");
}

#[tokio::test]
async fn composition_health_with_incubating_tower_shows_degraded() {
    let handler = LifecycleHandler::new("test-family");

    handler
        .register(&Some(json!({
            "name": "beardog-server",
            "socket_path": "/tmp/beardog.sock",
            "pid": 1
        })))
        .await
        .expect("register beardog");
    handler
        .register(&Some(json!({
            "name": "songbird-orch",
            "socket_path": "/tmp/songbird.sock",
            "pid": 2
        })))
        .await
        .expect("register songbird");

    let health = handler.composition_health(&None).await.expect("health");
    let subs = health["subsystems"].as_object().unwrap();

    assert_eq!(subs["tower"], "degraded");
    assert_eq!(subs["mesh"], "degraded");
    assert_eq!(subs["node"], "unavailable");
    assert_eq!(subs["nest"], "unavailable");
    assert!(!health["healthy"].as_bool().unwrap());
}

// =========================================================================
// Tests migrated from inline #[cfg(test)] in lifecycle.rs
// =========================================================================

#[tokio::test]
async fn handler_creation_empty() {
    let handler = LifecycleHandler::new("test-family");
    let status = handler.status().await.expect("status");
    assert_eq!(status["count"], 0);
    assert!(status["primals"].as_array().expect("primals").is_empty());
}

#[tokio::test]
async fn handler_with_explicit_manager() {
    use crate::lifecycle_manager::LifecycleManager;
    use std::sync::Arc;
    use tokio::sync::RwLock;

    let manager = Arc::new(RwLock::new(LifecycleManager::new("custom-family")));
    let handler = LifecycleHandler::with_manager(manager);
    let status = handler.status().await.expect("status");
    assert_eq!(status["count"], 0);
}

#[tokio::test]
async fn register_returns_full_state() {
    let handler = LifecycleHandler::new("test-family");

    let params = json!({
        "name": "test-primal",
        "socket_path": "/tmp/test-primal.sock",
        "pid": 12345
    });

    let result = handler.register(&Some(params)).await.expect("register");
    assert_eq!(result["registered"], "test-primal");
    assert_eq!(result["state"], "incubating");
    assert_eq!(result["socket_path"], "/tmp/test-primal.sock");
    assert_eq!(result["pid"], 12345);

    let status = handler.status().await.expect("status");
    assert_eq!(status["count"], 1);
    assert_eq!(status["healthy"], 0);

    let primals = status["primals"].as_array().expect("primals");
    let p = &primals[0];
    assert_eq!(p["name"], "test-primal");
    assert_eq!(p["state"], "incubating");
    assert!(p.get("details").is_some());
}

#[tokio::test]
async fn register_with_deployment_node_tracks_deps() {
    let handler = LifecycleHandler::new("test-family");

    let deployment_node = make_graph_node("beardog", vec![]);
    let params = json!({
        "name": "beardog",
        "socket_path": "/tmp/beardog.sock",
        "pid": 42,
        "deployment_node": serde_json::to_value(&deployment_node).unwrap()
    });

    let result = handler.register(&Some(params)).await.expect("register");
    assert_eq!(result["registered"], "beardog");

    let get_result = handler
        .get(&Some(json!({"name": "beardog"})))
        .await
        .expect("get");
    assert_eq!(get_result["name"], "beardog");
    assert!(get_result.get("depends_on").is_some());
    assert!(get_result.get("depended_by").is_some());
}

#[tokio::test]
async fn register_without_pid_is_null() {
    let handler = LifecycleHandler::new("test-family");

    let params = json!({
        "name": "no-pid-primal",
        "socket_path": "/tmp/no-pid.sock"
    });

    let result = handler.register(&Some(params)).await.expect("register");
    assert_eq!(result["registered"], "no-pid-primal");
    assert!(result["pid"].is_null());
}

#[tokio::test]
async fn get_missing_params_errors() {
    let handler = LifecycleHandler::new("test-family");
    let err = handler
        .get(&None)
        .await
        .expect_err("get with None should fail");
    assert!(err.to_string().contains("Missing parameters"));
}

#[tokio::test]
async fn get_missing_name_errors() {
    let handler = LifecycleHandler::new("test-family");
    let err = handler
        .get(&Some(json!({})))
        .await
        .expect_err("get with empty params should fail");
    assert!(err.to_string().contains("name"));
}

#[tokio::test]
async fn get_nonexistent_primal_returns_error_body() {
    let handler = LifecycleHandler::new("test-family");
    let result = handler
        .get(&Some(json!({"name": "nonexistent"})))
        .await
        .expect("get returns Ok with error in body");
    assert!(result.get("error").is_some());
    assert!(
        result["error"]
            .as_str()
            .expect("error string")
            .contains("nonexistent")
    );
}

#[tokio::test]
async fn register_missing_params_errors() {
    let handler = LifecycleHandler::new("test-family");
    let err = handler
        .register(&None)
        .await
        .expect_err("register with None should fail");
    assert!(err.to_string().contains("Missing parameters"));
}

#[tokio::test]
async fn register_missing_name_errors() {
    let handler = LifecycleHandler::new("test-family");
    let err = handler
        .register(&Some(json!({"socket_path": "/tmp/x.sock"})))
        .await
        .expect_err("register without name should fail");
    assert!(err.to_string().contains("name"));
}

#[tokio::test]
async fn register_missing_socket_path_errors() {
    let handler = LifecycleHandler::new("test-family");
    let err = handler
        .register(&Some(json!({"name": "x"})))
        .await
        .expect_err("register without socket_path should fail");
    assert!(err.to_string().contains("socket_path"));
}

#[tokio::test]
async fn resurrect_missing_params_errors() {
    let handler = LifecycleHandler::new("test-family");
    let err = handler
        .resurrect(&None)
        .await
        .expect_err("resurrect with None should fail");
    assert!(err.to_string().contains("Missing parameters"));
}

#[tokio::test]
async fn resurrect_nonexistent_returns_error_body() {
    let handler = LifecycleHandler::new("test-family");
    let result = handler
        .resurrect(&Some(json!({"name": "ghost"})))
        .await
        .expect("resurrect returns Ok");
    assert!(result.get("error").is_some());
    assert!(result["error"].as_str().unwrap().contains("ghost"));
}

#[tokio::test]
async fn apoptosis_missing_params_errors() {
    let handler = LifecycleHandler::new("test-family");
    let err = handler
        .apoptosis(&None)
        .await
        .expect_err("apoptosis with None should fail");
    assert!(err.to_string().contains("Missing parameters"));
}

#[tokio::test]
async fn apoptosis_user_request_transitions_to_dead() {
    let handler = LifecycleHandler::new("test-family");
    handler
        .register(&Some(json!({
            "name": "victim",
            "socket_path": "/tmp/victim.sock",
            "pid": 9999
        })))
        .await
        .expect("register");

    let result = handler
        .apoptosis(&Some(json!({"name": "victim"})))
        .await
        .expect("apoptosis");
    assert_eq!(result["initiated"], "victim");
    assert_eq!(result["reason"], "user_request");
    assert_eq!(result["state"], "apoptosis");

    let status = handler.status().await.expect("status");
    let primals = status["primals"].as_array().expect("primals");
    let victim = primals
        .iter()
        .find(|p| p["name"] == "victim")
        .expect("victim");
    assert_eq!(victim["state"], "dead");
}

#[tokio::test]
async fn apoptosis_all_reasons_map_correctly() {
    let reasons = [
        ("ecosystem_health", "ecosystem_health"),
        ("resource_pressure", "resource_pressure"),
        ("system_shutdown", "system_shutdown"),
        ("unknown_reason", "unknown_reason"),
    ];

    for (reason_param, expected_reason) in reasons {
        let handler = LifecycleHandler::new("test-family");
        let name = format!("primal-{reason_param}");
        handler
            .register(&Some(json!({
                "name": name,
                "socket_path": format!("/tmp/{}.sock", name),
                "pid": 1
            })))
            .await
            .expect("register");

        let result = handler
            .apoptosis(&Some(json!({
                "name": name,
                "reason": reason_param
            })))
            .await
            .expect("apoptosis");
        assert_eq!(
            result["reason"].as_str(),
            Some(expected_reason),
            "reason {reason_param} should map to {expected_reason}"
        );
    }
}

#[tokio::test]
async fn shutdown_all_kills_everything() {
    let handler = LifecycleHandler::new("test-family");
    for name in &["a", "b", "c"] {
        handler
            .register(&Some(json!({
                "name": name,
                "socket_path": format!("/tmp/{}.sock", name),
                "pid": 1
            })))
            .await
            .expect("register");
    }

    let result = handler.shutdown_all().await.expect("shutdown_all");
    assert_eq!(result["shutdown"], "complete");
    assert!(result["message"].as_str().unwrap().contains("All primals"));

    let status = handler.status().await.expect("status");
    assert_eq!(status["count"], 3);
    let primals = status["primals"].as_array().expect("primals");
    for p in primals {
        assert_eq!(p["state"], "dead");
    }
}

#[tokio::test]
async fn resurrect_registered_primal_succeeds() {
    let handler = LifecycleHandler::new("test-family");
    handler
        .register(&Some(json!({
            "name": "resurrect-me",
            "socket_path": "/tmp/resurrect-me.sock",
            "pid": 1234
        })))
        .await
        .expect("register");

    let result = handler
        .resurrect(&Some(json!({"name": "resurrect-me"})))
        .await
        .expect("resurrect");
    assert_eq!(result["requested"], "resurrect-me");
    assert!(result["message"].as_str().unwrap().contains("Resurrection"));
}

#[tokio::test]
async fn get_full_serialization_includes_all_fields() {
    let handler = LifecycleHandler::new("test-family");
    handler
        .register(&Some(json!({
            "name": "full-details",
            "socket_path": "/tmp/full.sock",
            "pid": 9999
        })))
        .await
        .expect("register");

    let result = handler
        .get(&Some(json!({"name": "full-details"})))
        .await
        .expect("get");

    assert_eq!(result["name"], "full-details");
    assert_eq!(result["family_id"], "test-family");
    assert_eq!(result["socket_path"], "/tmp/full.sock");
    assert_eq!(result["pid"], 9999);
    assert_eq!(result["state"], "incubating");

    assert!(result.get("state_details").is_some());
    assert!(result.get("depends_on").is_some());
    assert!(result.get("depended_by").is_some());
    assert!(result.get("metrics").is_some());
    assert!(result.get("health_config").is_some());
    assert!(result.get("resurrection_config").is_some());

    let metrics = &result["metrics"];
    assert!(metrics.get("total_uptime_secs").is_some());
    assert!(metrics.get("resurrection_count").is_some());
    assert!(metrics.get("health_failures").is_some());
    assert!(metrics.get("last_health_latency_ms").is_some());
    assert!(metrics.get("requests_served").is_some());

    let health_config = &result["health_config"];
    assert!(health_config.get("check_interval_secs").is_some());
    assert!(health_config.get("timeout_secs").is_some());
    assert!(health_config.get("failure_threshold").is_some());
    assert!(health_config.get("health_method").is_some());

    let res_config = &result["resurrection_config"];
    assert!(res_config.get("enabled").is_some());
    assert!(res_config.get("max_attempts").is_some());
    assert!(res_config.get("base_delay_secs").is_some());
    assert!(res_config.get("max_delay_secs").is_some());
}

#[tokio::test]
async fn status_healthy_count_tracks_correctly() {
    let handler = LifecycleHandler::new("test-family");
    handler
        .register(&Some(json!({
            "name": "p1",
            "socket_path": "/tmp/p1.sock",
            "pid": 1
        })))
        .await
        .expect("register");

    let status = handler.status().await.expect("status");
    assert_eq!(status["count"], 1);
    assert_eq!(status["healthy"], 0);

    handler
        .apoptosis(&Some(json!({"name": "p1"})))
        .await
        .expect("apoptosis");

    let status = handler.status().await.expect("status");
    assert_eq!(status["healthy"], 0);
}

#[tokio::test]
async fn incubating_state_details_include_timeout() {
    let handler = LifecycleHandler::new("test-family");
    handler
        .register(&Some(json!({
            "name": "incubating",
            "socket_path": "/tmp/inc.sock",
            "pid": 1
        })))
        .await
        .expect("register");

    let status = handler.status().await.expect("status");
    let primals = status["primals"].as_array().expect("primals");
    let p = primals
        .iter()
        .find(|x| x["name"] == "incubating")
        .expect("primal");
    assert_eq!(p["state"], "incubating");
    let details = &p["details"];
    assert!(details.get("started_at").is_some());
    assert!(details.get("timeout_ms").is_some());
}

#[tokio::test]
async fn dead_state_details_include_reason() {
    let handler = LifecycleHandler::new("test-family");
    handler
        .register(&Some(json!({
            "name": "to-die",
            "socket_path": "/tmp/die.sock",
            "pid": 1
        })))
        .await
        .expect("register");

    handler
        .apoptosis(&Some(json!({"name": "to-die", "reason": "user_request"})))
        .await
        .expect("apoptosis");

    let status = handler.status().await.expect("status");
    let primals = status["primals"].as_array().expect("primals");
    let p = primals
        .iter()
        .find(|x| x["name"] == "to-die")
        .expect("primal");
    assert_eq!(p["state"], "dead");
    let details = &p["details"];
    assert!(details.get("since").is_some());
    assert!(details.get("reason").is_some());
}
