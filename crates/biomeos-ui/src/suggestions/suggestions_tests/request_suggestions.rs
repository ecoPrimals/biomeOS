// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::collections::HashMap;

use super::super::*;

#[tokio::test]
async fn test_request_suggestions_empty_context() {
    let mut manager = AISuggestionManager::new("test_family".to_string());

    let context = SuggestionContext {
        assignments: HashMap::new(),
        available_devices: vec![],
        running_primals: vec![],
        recent_events: None,
        preferences: None,
    };

    let suggestions = manager
        .request_suggestions(&context)
        .expect("request_suggestions should succeed");
    assert!(suggestions.is_empty());
    assert!(manager.active_suggestions.is_empty());
}

#[tokio::test]
async fn test_discover_ai_provider() {
    let mut manager = AISuggestionManager::new("test_family".to_string());

    let result = manager.discover_ai_provider().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_request_suggestions_without_ai() {
    let mut manager = AISuggestionManager::new("test_family".to_string());

    let context = SuggestionContext {
        assignments: HashMap::new(),
        available_devices: vec![DeviceInfo {
            id: "gpu0".to_string(),
            device_type: "gpu".to_string(),
            capabilities: vec!["compute".to_string()],
            current_assignment: None,
        }],
        running_primals: vec![PrimalInfo {
            id: "toadstool1".to_string(),
            name: "ToadStool".to_string(),
            primal_type: "compute".to_string(),
            capabilities: vec!["compute".to_string()],
            health: "healthy".to_string(),
            load: Some(0.5),
        }],
        recent_events: None,
        preferences: None,
    };

    let suggestions = manager.request_suggestions(&context).unwrap();
    assert!(!suggestions.is_empty());
    assert_eq!(
        suggestions[0].suggestion_type,
        SuggestionType::DeviceAssignment
    );
}

#[tokio::test]
async fn test_request_suggestions_with_context() {
    let mut manager = AISuggestionManager::new("test_family".to_string());
    manager.discover_ai_provider().await.unwrap();

    let context = SuggestionContext {
        assignments: HashMap::new(),
        available_devices: vec![DeviceInfo {
            id: "gpu0".to_string(),
            device_type: "gpu".to_string(),
            capabilities: vec!["ml".to_string()],
            current_assignment: None,
        }],
        running_primals: vec![PrimalInfo {
            id: "squirrel1".to_string(),
            name: "Squirrel".to_string(),
            primal_type: "ai".to_string(),
            capabilities: vec!["ml".to_string(), "ai".to_string()],
            health: "healthy".to_string(),
            load: Some(0.6),
        }],
        recent_events: None,
        preferences: None,
    };

    let suggestions = manager.request_suggestions(&context).unwrap();
    assert!(!suggestions.is_empty());
    assert_eq!(manager.active_suggestions.len(), suggestions.len());
}

#[tokio::test]
async fn test_get_active_suggestions() {
    let mut manager = AISuggestionManager::new("test_family".to_string());

    let suggestion = AISuggestion {
        id: "test1".to_string(),
        suggestion_type: SuggestionType::DeviceAssignment,
        confidence: 0.9,
        explanation: "Test".to_string(),
        action: SuggestedAction::AssignDevice {
            device_id: "device1".to_string(),
            primal_id: "primal1".to_string(),
            reason: "Test".to_string(),
        },
        impact: Impact {
            performance_improvement: Some(15.0),
            cost_change: None,
            affected_primals: vec![],
            risk_level: "low".to_string(),
        },
    };

    manager
        .active_suggestions
        .insert(suggestion.id.clone(), suggestion);

    let active = manager.get_active_suggestions();
    assert_eq!(active.len(), 1);
    assert_eq!(active[0].id, "test1");
}
