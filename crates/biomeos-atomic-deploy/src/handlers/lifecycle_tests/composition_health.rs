// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::handlers::lifecycle::LifecycleHandler;

#[tokio::test]
async fn composition_health_empty_returns_unavailable_subsystems() {
    let handler = LifecycleHandler::new("test-family");
    let health = handler.composition_health(&None).await.expect("health");
    assert!(health["healthy"].as_bool().is_some());
    assert!(health["deploy_graph"].as_str().is_some());
    assert!(health["subsystems"].is_object());
    let subs = health["subsystems"].as_object().unwrap();
    assert_eq!(subs["tower"], "unavailable");
    assert_eq!(subs["mesh"]["status"], "unavailable");
}

#[tokio::test]
async fn composition_health_with_incubating_tower_shows_degraded() {
    let handler = LifecycleHandler::new("test-family");

    // Register primals with deployment_node capabilities so capability-based
    // subsystem detection recognizes them as tower/mesh providers.
    handler
        .register(&Some(serde_json::json!({
            "name": "security-provider",
            "socket_path": "/tmp/beardog.sock",
            "pid": 1,
            "deployment_node": {
                "id": "security-provider",
                "capabilities": ["crypto.sign", "crypto.encrypt", "security.audit"]
            }
        })))
        .await
        .expect("register security provider");
    handler
        .register(&Some(serde_json::json!({
            "name": "relay-provider",
            "socket_path": "/tmp/songbird.sock",
            "pid": 2,
            "deployment_node": {
                "id": "relay-provider",
                "capabilities": ["discovery.announce", "relay.allocate", "network.tls_connect"]
            }
        })))
        .await
        .expect("register relay provider");

    let health = handler.composition_health(&None).await.expect("health");
    let subs = health["subsystems"].as_object().unwrap();

    assert_eq!(subs["tower"], "degraded");
    assert_eq!(subs["mesh"]["status"], "degraded");
    assert_eq!(subs["node"], "unavailable");
    assert_eq!(subs["nest"], "unavailable");
    assert!(!health["healthy"].as_bool().unwrap());
}
