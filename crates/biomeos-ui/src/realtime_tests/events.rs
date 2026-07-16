// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::{JsonRpcNotification, *};
use std::sync::Arc;

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
        timestamp: 1_705_329_600,
        primals_count: 12,
        healthy_count: 11,
    };

    let json = serde_json::to_string(&event).unwrap();
    assert!(json.contains("1705329600"));
    assert!(json.contains("12"));
}
