// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::{JsonRpcNotification, *};

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
