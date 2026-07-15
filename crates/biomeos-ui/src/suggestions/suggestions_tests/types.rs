// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::collections::HashMap;

use super::super::*;

#[test]
fn test_suggestion_type_serialization() {
    let types = vec![
        SuggestionType::DeviceAssignment,
        SuggestionType::TopologyOptimization,
        SuggestionType::BottleneckPrediction,
        SuggestionType::ResourceReallocation,
        SuggestionType::PerformanceImprovement,
    ];

    for suggestion_type in types {
        let json = serde_json::to_string(&suggestion_type).unwrap();
        let deserialized: SuggestionType = serde_json::from_str(&json).unwrap();
        assert_eq!(suggestion_type, deserialized);
    }
}

#[test]
fn test_suggested_action_assign_device_serialization() {
    let action = SuggestedAction::AssignDevice {
        device_id: "gpu0".to_string(),
        primal_id: "toadstool1".to_string(),
        reason: "Better performance".to_string(),
    };

    let json = serde_json::to_string(&action).unwrap();
    assert!(json.contains("gpu0"));
    assert!(json.contains("toadstool1"));

    let deserialized: SuggestedAction = serde_json::from_str(&json).unwrap();
    match deserialized {
        SuggestedAction::AssignDevice {
            device_id,
            primal_id,
            ..
        } => {
            assert_eq!(device_id, "gpu0");
            assert_eq!(primal_id, "toadstool1");
        }
        _ => panic!("Wrong action type"),
    }
}

#[test]
fn test_suggested_action_remove_assignment() {
    let action = SuggestedAction::RemoveAssignment {
        device_id: "gpu0".to_string(),
        primal_id: "toadstool1".to_string(),
        reason: "Underutilized".to_string(),
    };

    let json = serde_json::to_string(&action).unwrap();
    let deserialized: SuggestedAction = serde_json::from_str(&json).unwrap();

    match deserialized {
        SuggestedAction::RemoveAssignment { device_id, .. } => {
            assert_eq!(device_id, "gpu0");
        }
        _ => panic!("Wrong action type"),
    }
}

#[test]
fn test_suggested_action_reallocate_resources() {
    let action = SuggestedAction::ReallocateResources {
        from_primal: "primal1".to_string(),
        to_primal: "primal2".to_string(),
        resource_type: "cpu".to_string(),
        amount: "2 cores".to_string(),
    };

    let json = serde_json::to_string(&action).unwrap();
    assert!(json.contains("primal1"));
    assert!(json.contains("cpu"));
}

#[test]
fn test_suggested_action_add_capacity() {
    let action = SuggestedAction::AddCapacity {
        primal_type: "compute".to_string(),
        estimated_need: "4 GPUs".to_string(),
    };

    let json = serde_json::to_string(&action).unwrap();
    assert!(json.contains("compute"));
    assert!(json.contains("4 GPUs"));
}

#[test]
fn test_suggested_action_optimize_config() {
    let action = SuggestedAction::OptimizeConfig {
        primal_id: "toadstool1".to_string(),
        config_key: "max_workers".to_string(),
        suggested_value: serde_json::json!(8),
    };

    let json = serde_json::to_string(&action).unwrap();
    assert!(json.contains("max_workers"));
}

#[test]
fn test_impact_struct() {
    let impact = Impact {
        performance_improvement: Some(25.5),
        cost_change: Some("-10%".to_string()),
        affected_primals: vec!["primal1".to_string(), "primal2".to_string()],
        risk_level: "low".to_string(),
    };

    assert!(
        impact
            .performance_improvement
            .is_some_and(|v| (v - 25.5).abs() < f32::EPSILON)
    );
    assert_eq!(impact.cost_change, Some("-10%".to_string()));
    assert_eq!(impact.affected_primals.len(), 2);
    assert_eq!(impact.risk_level, "low");
}

#[test]
fn test_suggestion_feedback_variants() {
    let accepted = SuggestionFeedback::Accepted;
    let rejected = SuggestionFeedback::Rejected {
        reason: "Not needed".to_string(),
    };
    let dismissed = SuggestionFeedback::Dismissed;
    let modified = SuggestionFeedback::Modified {
        changes: "Reduced scope".to_string(),
    };

    let json = serde_json::to_string(&accepted).unwrap();
    assert!(json.contains("Accepted"));

    let json = serde_json::to_string(&rejected).unwrap();
    assert!(json.contains("Not needed"));

    let json = serde_json::to_string(&dismissed).unwrap();
    let _: SuggestionFeedback = serde_json::from_str(&json).unwrap();

    let json = serde_json::to_string(&modified).unwrap();
    assert!(json.contains("Reduced scope"));
}

#[tokio::test]
async fn test_ai_suggestion_complete_struct() {
    let suggestion = AISuggestion {
        id: "complete_test".to_string(),
        suggestion_type: SuggestionType::ResourceReallocation,
        confidence: 0.88,
        explanation: "Rebalance resources for optimal performance".to_string(),
        action: SuggestedAction::ReallocateResources {
            from_primal: "overloaded_primal".to_string(),
            to_primal: "underutilized_primal".to_string(),
            resource_type: "cpu_cores".to_string(),
            amount: "4".to_string(),
        },
        impact: Impact {
            performance_improvement: Some(18.5),
            cost_change: Some("$0".to_string()),
            affected_primals: vec![
                "overloaded_primal".to_string(),
                "underutilized_primal".to_string(),
            ],
            risk_level: "low".to_string(),
        },
    };

    assert_eq!(suggestion.id, "complete_test");
    assert!((suggestion.confidence - 0.88).abs() < f32::EPSILON);
    assert!(suggestion.confidence > 0.5);
    assert_eq!(suggestion.impact.affected_primals.len(), 2);

    let json = serde_json::to_string(&suggestion).unwrap();
    let deserialized: AISuggestion = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.id, "complete_test");
    assert!((deserialized.confidence - 0.88).abs() < f32::EPSILON);
}

#[test]
fn test_device_info_struct() {
    let device = DeviceInfo {
        id: "test_device".to_string(),
        device_type: "gpu".to_string(),
        capabilities: vec!["compute".to_string(), "ml".to_string()],
        current_assignment: None,
    };

    assert_eq!(device.id, "test_device");
    assert_eq!(device.capabilities.len(), 2);
    assert!(device.capabilities.contains(&"ml".to_string()));
    assert!(device.current_assignment.is_none());
}

#[test]
fn test_primal_info_struct() {
    let primal = PrimalInfo {
        id: "primal_test".to_string(),
        name: "TestPrimal".to_string(),
        primal_type: "compute".to_string(),
        capabilities: vec!["processing".to_string()],
        health: "healthy".to_string(),
        load: Some(0.65),
    };

    assert_eq!(primal.name, "TestPrimal");
    assert!(primal.load.is_some_and(|v| (v - 0.65).abs() < f32::EPSILON));
    assert!(primal.load.unwrap() < 0.8);
}

#[test]
fn test_suggestion_context_creation() {
    let mut assignments = HashMap::new();
    assignments.insert("device1".to_string(), "primal1".to_string());

    let mut preferences = HashMap::new();
    preferences.insert("prefer_low_cost".to_string(), "true".to_string());

    let context = SuggestionContext {
        assignments,
        available_devices: vec![DeviceInfo {
            id: "device2".to_string(),
            device_type: "gpu".to_string(),
            capabilities: vec!["ml".to_string()],
            current_assignment: None,
        }],
        running_primals: vec![PrimalInfo {
            id: "primal1".to_string(),
            name: "ToadStool".to_string(),
            primal_type: "compute".to_string(),
            capabilities: vec!["compute".to_string()],
            health: "healthy".to_string(),
            load: Some(0.5),
        }],
        recent_events: Some(vec![
            "device_added".to_string(),
            "primal_started".to_string(),
        ]),
        preferences: Some(preferences),
    };

    assert_eq!(context.assignments.len(), 1);
    assert_eq!(context.available_devices.len(), 1);
    assert_eq!(context.running_primals.len(), 1);
    assert_eq!(context.recent_events.as_ref().unwrap().len(), 2);
    assert_eq!(
        context.preferences.as_ref().unwrap().get("prefer_low_cost"),
        Some(&"true".to_string())
    );
}
