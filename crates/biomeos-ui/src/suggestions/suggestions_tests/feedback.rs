// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::super::*;

#[tokio::test]
async fn test_feedback_modified_keeps_suggestion() {
    let mut manager = AISuggestionManager::new("test_family".to_string());

    let suggestion = AISuggestion {
        id: "test_modified".to_string(),
        suggestion_type: SuggestionType::DeviceAssignment,
        confidence: 0.85,
        explanation: "Modified suggestion".to_string(),
        action: SuggestedAction::AssignDevice {
            device_id: "gpu0".to_string(),
            primal_id: "primal1".to_string(),
            reason: "Test".to_string(),
        },
        impact: Impact {
            performance_improvement: Some(10.0),
            cost_change: None,
            affected_primals: vec![],
            risk_level: "low".to_string(),
        },
    };

    manager
        .active_suggestions
        .insert(suggestion.id.clone(), suggestion.clone());

    manager
        .send_feedback(
            &suggestion.id,
            &SuggestionFeedback::Modified {
                changes: "Adjusted parameters".to_string(),
            },
        )
        .expect("send_feedback should succeed");

    assert_eq!(
        manager.active_suggestions.len(),
        1,
        "Modified feedback should NOT remove suggestion"
    );
}

#[tokio::test]
async fn test_suggestion_feedback() {
    let mut manager = AISuggestionManager::new("test_family".to_string());

    let suggestion = AISuggestion {
        id: "test_suggestion".to_string(),
        suggestion_type: SuggestionType::DeviceAssignment,
        confidence: 0.8,
        explanation: "Test".to_string(),
        action: SuggestedAction::AssignDevice {
            device_id: "device1".to_string(),
            primal_id: "primal1".to_string(),
            reason: "Test".to_string(),
        },
        impact: Impact {
            performance_improvement: Some(10.0),
            cost_change: None,
            affected_primals: vec![],
            risk_level: "low".to_string(),
        },
    };

    manager
        .active_suggestions
        .insert(suggestion.id.clone(), suggestion.clone());
    assert_eq!(manager.active_suggestions.len(), 1);

    let result = manager.send_feedback(&suggestion.id, &SuggestionFeedback::Accepted);
    assert!(result.is_ok());

    assert_eq!(manager.active_suggestions.len(), 0);
}

#[tokio::test]
async fn test_feedback_accepted_removes_suggestion() {
    let mut manager = AISuggestionManager::new("test_family".to_string());

    let suggestion = AISuggestion {
        id: "test_accepted".to_string(),
        suggestion_type: SuggestionType::TopologyOptimization,
        confidence: 0.95,
        explanation: "Optimize".to_string(),
        action: SuggestedAction::OptimizeConfig {
            primal_id: "primal1".to_string(),
            config_key: "workers".to_string(),
            suggested_value: serde_json::json!(4),
        },
        impact: Impact {
            performance_improvement: Some(20.0),
            cost_change: Some("-5%".to_string()),
            affected_primals: vec!["primal1".to_string()],
            risk_level: "low".to_string(),
        },
    };

    manager
        .active_suggestions
        .insert(suggestion.id.clone(), suggestion.clone());
    assert_eq!(manager.active_suggestions.len(), 1);

    manager
        .send_feedback(&suggestion.id, &SuggestionFeedback::Accepted)
        .unwrap();
    assert_eq!(manager.active_suggestions.len(), 0);
}

#[tokio::test]
async fn test_feedback_rejected_removes_suggestion() {
    let mut manager = AISuggestionManager::new("test_family".to_string());

    let suggestion = AISuggestion {
        id: "test_rejected".to_string(),
        suggestion_type: SuggestionType::BottleneckPrediction,
        confidence: 0.7,
        explanation: "Potential bottleneck".to_string(),
        action: SuggestedAction::AddCapacity {
            primal_type: "storage".to_string(),
            estimated_need: "100GB".to_string(),
        },
        impact: Impact {
            performance_improvement: Some(5.0),
            cost_change: Some("+$50".to_string()),
            affected_primals: vec![],
            risk_level: "medium".to_string(),
        },
    };

    manager
        .active_suggestions
        .insert(suggestion.id.clone(), suggestion.clone());

    manager
        .send_feedback(
            &suggestion.id,
            &SuggestionFeedback::Rejected {
                reason: "Too expensive".to_string(),
            },
        )
        .unwrap();

    assert_eq!(manager.active_suggestions.len(), 0);
}

#[tokio::test]
async fn test_feedback_dismissed_keeps_suggestion() {
    let mut manager = AISuggestionManager::new("test_family".to_string());

    let suggestion = AISuggestion {
        id: "test_dismissed".to_string(),
        suggestion_type: SuggestionType::PerformanceImprovement,
        confidence: 0.85,
        explanation: "Improve perf".to_string(),
        action: SuggestedAction::ReallocateResources {
            from_primal: "primal1".to_string(),
            to_primal: "primal2".to_string(),
            resource_type: "memory".to_string(),
            amount: "1GB".to_string(),
        },
        impact: Impact {
            performance_improvement: Some(12.0),
            cost_change: None,
            affected_primals: vec!["primal1".to_string(), "primal2".to_string()],
            risk_level: "low".to_string(),
        },
    };

    manager
        .active_suggestions
        .insert(suggestion.id.clone(), suggestion.clone());

    manager
        .send_feedback(&suggestion.id, &SuggestionFeedback::Dismissed)
        .unwrap();

    assert_eq!(manager.active_suggestions.len(), 1);
}
