// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::{JsonRpcNotification, *};

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

#[test]
fn test_parse_event_nested_event_with_extra_fields() {
    let notification = JsonRpcNotification::for_test(serde_json::json!({
        "event": {
            "type": "topology_changed",
            "nodes": 3,
            "edges": 5,
            "change": "node_added"
        },
        "extra": "ignored"
    }));
    let result = RealTimeEventSubscriber::parse_event_for_test(&notification);
    assert!(result.is_ok());
    match result.unwrap() {
        RealTimeEvent::TopologyChanged { nodes, edges, .. } => {
            assert_eq!(nodes, 3);
            assert_eq!(edges, 5);
        }
        _ => panic!("Expected TopologyChanged"),
    }
}

#[test]
fn test_parse_event_primal_discovered_from_params() {
    let notification = JsonRpcNotification::for_test(serde_json::json!({
        "type": "primal_discovered",
        "primal_id": "p-123",
        "name": "TestPrimal",
        "primal_type": "security",
        "capabilities": ["crypto", "identity"]
    }));
    let result = RealTimeEventSubscriber::parse_event_for_test(&notification);
    assert!(result.is_ok());
    match result.unwrap() {
        RealTimeEvent::PrimalDiscovered {
            primal_id,
            capabilities,
            ..
        } => {
            assert_eq!(primal_id, "p-123");
            assert_eq!(capabilities.len(), 2);
        }
        _ => panic!("Expected PrimalDiscovered"),
    }
}

#[test]
fn test_parse_event_assignment_created_no_user() {
    let notification = JsonRpcNotification::for_test(serde_json::json!({
        "type": "assignment_created",
        "device_id": "gpu-1",
        "primal_id": "compute-1",
        "user_id": null
    }));
    let result = RealTimeEventSubscriber::parse_event_for_test(&notification);
    assert!(result.is_ok());
    match result.unwrap() {
        RealTimeEvent::AssignmentCreated { user_id, .. } => {
            assert!(user_id.is_none());
        }
        _ => panic!("Expected AssignmentCreated"),
    }
}

#[test]
fn test_all_event_variants_deserialize_from_json() {
    let variants = [
        r#"{"type":"device_removed","device_id":"d1"}"#,
        r#"{"type":"assignment_removed","device_id":"d1","primal_id":"p1"}"#,
        r#"{"type":"health_changed","primal_id":"p1","name":"P","old_health":"unknown","new_health":"healthy"}"#,
    ];
    for json in variants {
        let event: Result<RealTimeEvent, _> = serde_json::from_str(json);
        assert!(event.is_ok(), "Failed to parse: {json}");
    }
}

#[test]
fn test_parse_event_params_event_is_null() {
    let notification = JsonRpcNotification::for_test(serde_json::json!({
        "event": null,
        "type": "heartbeat",
        "timestamp": 1,
        "primals_count": 1,
        "healthy_count": 1
    }));
    let result = RealTimeEventSubscriber::parse_event_for_test(&notification);
    assert!(
        result.is_err(),
        "explicit null event should not deserialize as RealTimeEvent"
    );
}

#[test]
fn test_parse_event_params_event_wrong_shape() {
    let notification = JsonRpcNotification::for_test(serde_json::json!({
        "event": {
            "type": "heartbeat",
            "timestamp": "not_a_u64",
            "primals_count": 1,
            "healthy_count": 1
        }
    }));
    assert!(RealTimeEventSubscriber::parse_event_for_test(&notification).is_err());
}

#[test]
fn test_parse_sse_event_empty_input() {
    assert!(RealTimeEventSubscriber::parse_sse_event("").is_none());
}

#[test]
fn test_parse_sse_event_data_before_event_line() {
    let sse_text = "data: {\"type\":\"heartbeat\",\"timestamp\":1,\"primals_count\":1,\"healthy_count\":1}\nevent: heartbeat";
    let event = RealTimeEventSubscriber::parse_sse_event(sse_text);
    assert!(event.is_some());
    assert!(matches!(event.unwrap(), RealTimeEvent::Heartbeat { .. }));
}

#[test]
fn test_parse_sse_event_valid_json_unknown_variant() {
    let sse_text = "data: {\"type\":\"not_a_realtime_variant\",\"x\":1}";
    assert!(RealTimeEventSubscriber::parse_sse_event(sse_text).is_none());
}

#[test]
fn test_parse_sse_event_malformed_timestamp_type() {
    let sse_text = "data: {\"type\":\"heartbeat\",\"timestamp\":\"nan\",\"primals_count\":1,\"healthy_count\":1}";
    assert!(RealTimeEventSubscriber::parse_sse_event(sse_text).is_none());
}
