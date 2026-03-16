// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Unit tests for real-time event streaming (WebSocket/SSE).

#![allow(clippy::unwrap_used, clippy::expect_used)]

use super::{JsonRpcNotification, *};
use biomeos_test_utils::{remove_test_env, set_test_env};
use std::sync::Arc;

#[tokio::test]
async fn test_subscriber_creation() {
    let subscriber = RealTimeEventSubscriber::new("test_family".to_string());
    // Subscriber created with default URLs (none until discover_endpoints)
    let mut rx = subscriber.subscribe();
    assert!(rx.try_recv().is_err()); // No events yet
}

#[tokio::test]
#[ignore = "Requires running Neural API server — integration test"]
async fn test_discover_endpoints() {
    let mut subscriber = RealTimeEventSubscriber::new("test_family".to_string());

    // Discover endpoints
    let result = subscriber.discover_endpoints().await;
    assert!(result.is_ok());
    // Endpoints come from env vars when set
}

#[test]
fn test_event_serialization() {
    let event = RealTimeEvent::PrimalDiscovered {
        primal_id: "test_primal".to_string(),
        name: "Test Primal".to_string(),
        primal_type: "test".to_string(),
        capabilities: vec!["test".to_string()],
    };

    // Serialize and deserialize
    let json = serde_json::to_string(&event).unwrap();
    let deserialized: RealTimeEvent = serde_json::from_str(&json).unwrap();

    match deserialized {
        RealTimeEvent::PrimalDiscovered { primal_id, .. } => {
            assert_eq!(primal_id, "test_primal");
        }
        _ => panic!("Wrong event type"),
    }
}

#[tokio::test]
async fn test_event_broadcasting() {
    let subscriber = Arc::new(RealTimeEventSubscriber::new("test_family".to_string()));

    // Subscribe to events
    let mut rx1 = subscriber.subscribe();
    let mut rx2 = subscriber.subscribe();

    // Send test event
    let event = RealTimeEvent::Heartbeat {
        timestamp: 12345,
        primals_count: 5,
        healthy_count: 5,
    };

    subscriber.send_event(event);

    // Both receivers should get the event
    let event1 = rx1.try_recv();
    let event2 = rx2.try_recv();

    assert!(event1.is_ok());
    assert!(event2.is_ok());
}

#[test]
fn test_sse_event_parsing() {
    // Test valid SSE event format
    let sse_text = "event: graph_event\ndata: {\"type\":\"graph_event\",\"graph_id\":\"test123\",\"node_id\":\"node1\",\"event_type\":\"started\",\"timestamp\":\"2026-01-15T12:00:00Z\",\"details\":{}}";

    let event = RealTimeEventSubscriber::parse_sse_event(sse_text);
    assert!(event.is_some());

    match event.unwrap() {
        RealTimeEvent::GraphEvent { graph_id, .. } => {
            assert_eq!(graph_id, "test123");
        }
        _ => panic!("Expected GraphEvent"),
    }
}

#[test]
fn test_sse_event_parsing_no_event_type() {
    // SSE with only data field
    let sse_text = "data: {\"type\":\"heartbeat\",\"timestamp\":12345,\"primals_count\":5,\"healthy_count\":5}";

    let event = RealTimeEventSubscriber::parse_sse_event(sse_text);
    assert!(event.is_some());
}

#[test]
fn test_sse_event_parsing_invalid() {
    // Invalid JSON in data field
    let sse_text = "event: test\ndata: invalid json";

    let event = RealTimeEventSubscriber::parse_sse_event(sse_text);
    assert!(event.is_none());
}

#[test]
fn test_sse_event_parsing_no_data() {
    // SSE with no data field
    let sse_text = "event: test_event";

    let event = RealTimeEventSubscriber::parse_sse_event(sse_text);
    assert!(event.is_none());
}

#[test]
fn test_all_realtimeevent_variants() {
    // Test all event variants can be created
    let events = vec![
        RealTimeEvent::GraphEvent {
            graph_id: "g1".to_string(),
            node_id: Some("n1".to_string()),
            event_type: "started".to_string(),
            timestamp: "2026-01-15T12:00:00Z".to_string(),
            details: serde_json::json!({}),
        },
        RealTimeEvent::PrimalDiscovered {
            primal_id: "p1".to_string(),
            name: "TestPrimal".to_string(),
            primal_type: "test".to_string(),
            capabilities: vec!["cap1".to_string()],
        },
        RealTimeEvent::HealthChanged {
            primal_id: "p1".to_string(),
            name: "TestPrimal".to_string(),
            old_health: "unknown".to_string(),
            new_health: "healthy".to_string(),
        },
        RealTimeEvent::DeviceAdded {
            device_id: "d1".to_string(),
            device_type: "gpu".to_string(),
            capabilities: vec!["compute".to_string()],
        },
        RealTimeEvent::DeviceRemoved {
            device_id: "d1".to_string(),
        },
        RealTimeEvent::AssignmentCreated {
            device_id: "d1".to_string(),
            primal_id: "p1".to_string(),
            user_id: Some("u1".to_string()),
        },
        RealTimeEvent::AssignmentRemoved {
            device_id: "d1".to_string(),
            primal_id: "p1".to_string(),
        },
        RealTimeEvent::TopologyChanged {
            nodes: 10,
            edges: 15,
            change: "added_node".to_string(),
        },
        RealTimeEvent::Heartbeat {
            timestamp: 12345,
            primals_count: 5,
            healthy_count: 5,
        },
    ];

    // All variants should serialize successfully
    for event in events {
        let json = serde_json::to_string(&event).unwrap();
        assert!(!json.is_empty());
    }
}

#[test]
fn test_graph_event_serialization() {
    let event = RealTimeEvent::GraphEvent {
        graph_id: "test_graph".to_string(),
        node_id: Some("node1".to_string()),
        event_type: "completed".to_string(),
        timestamp: "2026-01-15T12:00:00Z".to_string(),
        details: serde_json::json!({"status": "success"}),
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("test_graph"));
    assert!(json.contains("completed"));

    let deserialized: RealTimeEvent = serde_json::from_str(&json).unwrap();
    match deserialized {
        RealTimeEvent::GraphEvent {
            graph_id,
            event_type,
            ..
        } => {
            assert_eq!(graph_id, "test_graph");
            assert_eq!(event_type, "completed");
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_health_changed_event() {
    let event = RealTimeEvent::HealthChanged {
        primal_id: "beardog-1".to_string(),
        name: "BearDog".to_string(),
        old_health: "degraded".to_string(),
        new_health: "healthy".to_string(),
    };

    let json = serde_json::to_string(&event).unwrap();
    let deserialized: RealTimeEvent = serde_json::from_str(&json).unwrap();

    match deserialized {
        RealTimeEvent::HealthChanged {
            old_health,
            new_health,
            ..
        } => {
            assert_eq!(old_health, "degraded");
            assert_eq!(new_health, "healthy");
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_device_added_event() {
    let event = RealTimeEvent::DeviceAdded {
        device_id: "gpu0".to_string(),
        device_type: "gpu".to_string(),
        capabilities: vec!["compute".to_string(), "ml".to_string()],
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("gpu0"));
    assert!(json.contains("compute"));
}

#[test]
fn test_assignment_events() {
    let created = RealTimeEvent::AssignmentCreated {
        device_id: "gpu0".to_string(),
        primal_id: "toadstool-1".to_string(),
        user_id: Some("user1".to_string()),
    };

    let removed = RealTimeEvent::AssignmentRemoved {
        device_id: "gpu0".to_string(),
        primal_id: "toadstool-1".to_string(),
    };

    // Both should serialize
    let json1 = serde_json::to_string(&created).unwrap();
    let json2 = serde_json::to_string(&removed).unwrap();

    assert!(json1.contains("gpu0"));
    assert!(json2.contains("gpu0"));
}

#[test]
fn test_topology_changed_event() {
    let event = RealTimeEvent::TopologyChanged {
        nodes: 25,
        edges: 40,
        change: "primal_added".to_string(),
    };

    let json = serde_json::to_string(&event).unwrap();
    let deserialized: RealTimeEvent = serde_json::from_str(&json).unwrap();

    match deserialized {
        RealTimeEvent::TopologyChanged { nodes, edges, .. } => {
            assert_eq!(nodes, 25);
            assert_eq!(edges, 40);
        }
        _ => panic!("Wrong event type"),
    }
}

#[test]
fn test_heartbeat_event() {
    let event = RealTimeEvent::Heartbeat {
        timestamp: 1705329600,
        primals_count: 12,
        healthy_count: 11,
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("1705329600"));
    assert!(json.contains("12"));
}

#[tokio::test]
async fn test_event_handler_creation() {
    let subscriber = Arc::new(RealTimeEventSubscriber::new("test_family".to_string()));
    let _handler = RealTimeEventHandler::new(subscriber);

    // Handler created successfully — reaching this point validates construction
}

#[tokio::test]
async fn test_subscriber_subscribe() {
    let subscriber = RealTimeEventSubscriber::new("test_family".to_string());
    let _rx1 = subscriber.subscribe();
    let _rx2 = subscriber.subscribe();

    // Both subscriptions created independently — reaching this validates the pattern
}

#[test]
fn test_sse_multiline_data() {
    // SSE with multiline data (valid JSON split across lines)
    let sse_text = "event: test\ndata: {\"type\":\"heartbeat\",\ndata: \"timestamp\":12345,\ndata: \"primals_count\":5,\"healthy_count\":5}";

    // This should fail to parse (our implementation expects data on one line)
    let event = RealTimeEventSubscriber::parse_sse_event(sse_text);
    assert!(event.is_none());
}

#[test]
fn test_jsonrpc_notification_structure() {
    // Test that we can parse JSON-RPC notifications
    let json = r#"{"jsonrpc":"2.0","method":"event.notify","params":{"event":{"type":"heartbeat","timestamp":12345,"primals_count":5,"healthy_count":5}}}"#;

    let notification: serde_json::Result<serde_json::Value> = serde_json::from_str(json);
    assert!(notification.is_ok());

    let notif = notification.unwrap();
    assert_eq!(notif["jsonrpc"], "2.0");
    assert_eq!(notif["method"], "event.notify");
}

#[test]
fn test_parse_event_from_params_event() {
    let notification = JsonRpcNotification::for_test(serde_json::json!({
        "event": {
            "type": "heartbeat",
            "timestamp": 12345,
            "primals_count": 5,
            "healthy_count": 5
        }
    }));

    let result = RealTimeEventSubscriber::parse_event_for_test(&notification);
    assert!(result.is_ok());
    match result.unwrap() {
        RealTimeEvent::Heartbeat {
            timestamp,
            primals_count,
            healthy_count,
        } => {
            assert_eq!(timestamp, 12345);
            assert_eq!(primals_count, 5);
            assert_eq!(healthy_count, 5);
        }
        _ => panic!("Expected Heartbeat event"),
    }
}

#[test]
fn test_parse_event_from_params_directly() {
    let notification = JsonRpcNotification::for_test(serde_json::json!({
        "type": "device_removed",
        "device_id": "gpu-0"
    }));

    let result = RealTimeEventSubscriber::parse_event_for_test(&notification);
    assert!(result.is_ok());
    match result.unwrap() {
        RealTimeEvent::DeviceRemoved { device_id } => assert_eq!(device_id, "gpu-0"),
        _ => panic!("Expected DeviceRemoved event"),
    }
}

#[test]
fn test_parse_event_invalid_json() {
    let notification = JsonRpcNotification::for_test(serde_json::json!({
        "type": "unknown_type",
        "invalid": "data"
    }));

    let result = RealTimeEventSubscriber::parse_event_for_test(&notification);
    assert!(result.is_err());
}

#[tokio::test]
async fn test_subscribe_sse_no_websocket_returns_ok() {
    let mut subscriber = RealTimeEventSubscriber::new("test_family".to_string());
    subscriber.set_urls_for_test(None, Some("http://localhost:9999/sse".to_string()));

    let result = subscriber.subscribe_sse().await;
    assert!(
        result.is_ok(),
        "subscribe_sse should return Ok when only SSE URL is set (graceful degradation)"
    );
}

#[tokio::test]
async fn test_discover_endpoints_no_env() {
    let mut subscriber = RealTimeEventSubscriber::new("test_family".to_string());

    let result = subscriber.discover_endpoints().await;
    assert!(result.is_ok());
    // With no env vars, subscribe_websocket should fail
    assert!(subscriber.subscribe_websocket().await.is_err());
}

#[tokio::test]
async fn test_subscribe_websocket_no_url_returns_err() {
    let subscriber = RealTimeEventSubscriber::new("test_family".to_string());

    let result = subscriber.subscribe_websocket().await;
    assert!(
        result.is_err(),
        "subscribe_websocket should fail when URL not discovered"
    );
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("WebSocket URL not discovered"),
        "Expected context about URL, got: {err}"
    );
}

#[tokio::test]
async fn test_subscribe_sse_no_url_returns_err() {
    let subscriber = RealTimeEventSubscriber::new("test_family".to_string());
    // sse_url is None by default
    let result = subscriber.subscribe_sse().await;
    assert!(
        result.is_err(),
        "subscribe_sse should fail when SSE URL not discovered"
    );
    let err = result.unwrap_err();
    assert!(
        err.to_string().contains("SSE URL not discovered"),
        "Expected context about SSE URL, got: {err}"
    );
}

#[tokio::test]
async fn test_process_events_handler_error_continues() {
    use std::sync::atomic::{AtomicU32, Ordering};

    let subscriber = Arc::new(RealTimeEventSubscriber::new("test_family".to_string()));
    let handler = RealTimeEventHandler::new(subscriber.clone());

    let processed = Arc::new(AtomicU32::new(0));
    let errored = Arc::new(std::sync::atomic::AtomicBool::new(false));

    let p = processed.clone();
    let e = errored.clone();
    let mut h = handler;
    let handle = tokio::spawn(async move {
        h.process_events(move |event| {
            p.fetch_add(1, Ordering::SeqCst);
            if matches!(event, RealTimeEvent::Heartbeat { .. }) {
                e.store(true, Ordering::SeqCst);
                Err(anyhow::anyhow!("simulated handler error"))
            } else {
                Ok(())
            }
        })
        .await
    });

    subscriber.send_event(RealTimeEvent::Heartbeat {
        timestamp: 1,
        primals_count: 1,
        healthy_count: 1,
    });
    subscriber.send_event(RealTimeEvent::Heartbeat {
        timestamp: 2,
        primals_count: 2,
        healthy_count: 2,
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert!(
        processed.load(Ordering::SeqCst) >= 1,
        "handler should have processed at least one event"
    );
    assert!(
        errored.load(Ordering::SeqCst),
        "handler should have seen the error path"
    );
    handle.abort();
}

#[tokio::test]
async fn test_process_events_receives_and_processes() {
    use std::sync::atomic::{AtomicU32, Ordering};

    let subscriber = Arc::new(RealTimeEventSubscriber::new("test_family".to_string()));
    let handler = RealTimeEventHandler::new(subscriber.clone());

    let event_count = Arc::new(AtomicU32::new(0));
    let ec = event_count.clone();
    let mut h = handler;
    let _handle = tokio::spawn(async move {
        h.process_events(move |_| {
            ec.fetch_add(1, Ordering::SeqCst);
            Ok(())
        })
        .await
    });

    subscriber.send_event(RealTimeEvent::Heartbeat {
        timestamp: 1,
        primals_count: 1,
        healthy_count: 1,
    });

    tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    assert!(
        event_count.load(Ordering::SeqCst) >= 1,
        "should have processed at least one event"
    );
}

#[test]
fn test_parse_event_params_empty_object() {
    let notification = JsonRpcNotification::for_test(serde_json::json!({}));
    let result = RealTimeEventSubscriber::parse_event_for_test(&notification);
    assert!(
        result.is_err(),
        "empty params should fail to parse as RealTimeEvent"
    );
}

#[test]
fn test_parse_event_params_null() {
    let notification = JsonRpcNotification::for_test(serde_json::Value::Null);
    let result = RealTimeEventSubscriber::parse_event_for_test(&notification);
    assert!(result.is_err(), "null params should fail to parse");
}

#[test]
fn test_sse_event_parsing_event_only_no_data() {
    let sse_text = "event: heartbeat";
    let event = RealTimeEventSubscriber::parse_sse_event(sse_text);
    assert!(
        event.is_none(),
        "event line without data should return None"
    );
}

#[test]
fn test_sse_event_parsing_multiple_event_lines() {
    let sse_text = "event: first\nevent: second\ndata: {\"type\":\"heartbeat\",\"timestamp\":1,\"primals_count\":1,\"healthy_count\":1}";
    let event = RealTimeEventSubscriber::parse_sse_event(sse_text);
    assert!(event.is_some(), "last event type and data should be used");
}

#[tokio::test]
async fn test_subscribe_sse_with_websocket_upgrades_to_websocket() {
    let mut subscriber = RealTimeEventSubscriber::new("test_family".to_string());
    subscriber.set_urls_for_test(
        Some("ws://localhost:9999/ws".to_string()),
        Some("http://localhost:9999/sse".to_string()),
    );
    let result = subscriber.subscribe_sse().await;
    assert!(
        result.is_err(),
        "subscribe_sse with both URLs delegates to subscribe_websocket which fails on invalid URL"
    );
}

#[tokio::test]
async fn test_discover_endpoints_with_ws_env() {
    set_test_env("BIOMEOS_WS_ENDPOINT", "ws://test.example/ws");
    let mut subscriber = RealTimeEventSubscriber::new("test_family".to_string());
    let result = subscriber.discover_endpoints().await;
    remove_test_env("BIOMEOS_WS_ENDPOINT");
    assert!(result.is_ok());
    assert!(subscriber.subscribe_websocket().await.is_err());
}

#[tokio::test]
async fn test_discover_endpoints_with_sse_env() {
    set_test_env("BIOMEOS_SSE_ENDPOINT", "http://test.example/sse");
    let mut subscriber = RealTimeEventSubscriber::new("test_family".to_string());
    let result = subscriber.discover_endpoints().await;
    remove_test_env("BIOMEOS_SSE_ENDPOINT");
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_discover_endpoints_with_both_env_vars() {
    set_test_env("BIOMEOS_WS_ENDPOINT", "ws://test.example/ws");
    set_test_env("BIOMEOS_SSE_ENDPOINT", "http://test.example/sse");
    let mut subscriber = RealTimeEventSubscriber::new("test_family".to_string());
    let result = subscriber.discover_endpoints().await;
    remove_test_env("BIOMEOS_WS_ENDPOINT");
    remove_test_env("BIOMEOS_SSE_ENDPOINT");
    assert!(result.is_ok());
}

#[test]
fn test_sse_event_parsing_empty_lines_and_whitespace() {
    let sse_text = "event: heartbeat\n\n  \ndata: {\"type\":\"heartbeat\",\"timestamp\":1,\"primals_count\":1,\"healthy_count\":1}\n";
    let event = RealTimeEventSubscriber::parse_sse_event(sse_text);
    assert!(event.is_some());
}

#[test]
fn test_sse_event_parsing_data_with_leading_trailing_whitespace() {
    let sse_text =
        "data:  {\"type\":\"heartbeat\",\"timestamp\":1,\"primals_count\":1,\"healthy_count\":1}  ";
    let event = RealTimeEventSubscriber::parse_sse_event(sse_text);
    assert!(event.is_some());
}

#[test]
fn test_parse_event_params_array_fails() {
    let notification = JsonRpcNotification::for_test(serde_json::json!([1, 2, 3]));
    let result = RealTimeEventSubscriber::parse_event_for_test(&notification);
    assert!(result.is_err());
}

#[test]
fn test_parse_event_params_string_fails() {
    let notification = JsonRpcNotification::for_test(serde_json::json!("not an event"));
    let result = RealTimeEventSubscriber::parse_event_for_test(&notification);
    assert!(result.is_err());
}

#[test]
fn test_parse_event_params_number_fails() {
    let notification = JsonRpcNotification::for_test(serde_json::json!(42));
    let result = RealTimeEventSubscriber::parse_event_for_test(&notification);
    assert!(result.is_err());
}

#[test]
fn test_jsonrpc_notification_for_test() {
    let notif = JsonRpcNotification::for_test(
        serde_json::json!({"type":"heartbeat","timestamp":1,"primals_count":1,"healthy_count":1}),
    );
    let result = RealTimeEventSubscriber::parse_event_for_test(&notif);
    assert!(result.is_ok());
}
