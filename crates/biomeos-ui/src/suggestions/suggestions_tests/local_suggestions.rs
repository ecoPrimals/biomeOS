// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use std::collections::HashMap;

use super::super::*;

#[tokio::test]
async fn test_suggestion_manager_creation() {
    let manager = AISuggestionManager::new("test_family".to_string());
    assert_eq!(manager.family_id, "test_family");
    assert!(manager.active_suggestions.is_empty());
}

#[tokio::test]
async fn test_local_suggestions_unassigned_device() {
    let context = SuggestionContext {
        assignments: HashMap::new(),
        available_devices: vec![DeviceInfo {
            id: "device1".to_string(),
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

    let suggestions = AISuggestionManager::generate_local_suggestions(&context);
    assert_eq!(suggestions.len(), 1);
    assert_eq!(
        suggestions[0].suggestion_type,
        SuggestionType::DeviceAssignment
    );
}

#[tokio::test]
async fn test_local_suggestions_overloaded_primal() {
    let context = SuggestionContext {
        assignments: HashMap::new(),
        available_devices: vec![],
        running_primals: vec![PrimalInfo {
            id: "toadstool1".to_string(),
            name: "ToadStool".to_string(),
            primal_type: "compute".to_string(),
            capabilities: vec!["compute".to_string()],
            health: "healthy".to_string(),
            load: Some(0.9),
        }],
        recent_events: None,
        preferences: None,
    };

    let suggestions = AISuggestionManager::generate_local_suggestions(&context);
    assert_eq!(suggestions.len(), 1);
    assert_eq!(
        suggestions[0].suggestion_type,
        SuggestionType::ResourceReallocation
    );
}

#[tokio::test]
async fn test_local_suggestions_device_already_assigned() {
    let context = SuggestionContext {
        assignments: HashMap::new(),
        available_devices: vec![DeviceInfo {
            id: "gpu0".to_string(),
            device_type: "gpu".to_string(),
            capabilities: vec!["compute".to_string()],
            current_assignment: Some("toadstool-1".to_string()),
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

    let suggestions = AISuggestionManager::generate_local_suggestions(&context);
    assert!(
        suggestions.is_empty(),
        "Already-assigned devices should not get assignment suggestions"
    );
}

#[tokio::test]
async fn test_local_suggestions_no_compatible_primal() {
    let context = SuggestionContext {
        assignments: HashMap::new(),
        available_devices: vec![DeviceInfo {
            id: "gpu0".to_string(),
            device_type: "gpu".to_string(),
            capabilities: vec!["cuda".to_string(), "ml".to_string()],
            current_assignment: None,
        }],
        running_primals: vec![PrimalInfo {
            id: "beardog1".to_string(),
            name: "BearDog".to_string(),
            primal_type: "security".to_string(),
            capabilities: vec!["crypto".to_string(), "identity".to_string()],
            health: "healthy".to_string(),
            load: Some(0.3),
        }],
        recent_events: None,
        preferences: None,
    };

    let suggestions = AISuggestionManager::generate_local_suggestions(&context);
    assert!(
        suggestions.is_empty(),
        "No suggestion when device capabilities don't match any primal"
    );
}

#[tokio::test]
async fn test_local_suggestions_load_boundary_0_8() {
    let context = SuggestionContext {
        assignments: HashMap::new(),
        available_devices: vec![],
        running_primals: vec![PrimalInfo {
            id: "toadstool1".to_string(),
            name: "ToadStool".to_string(),
            primal_type: "compute".to_string(),
            capabilities: vec!["compute".to_string()],
            health: "healthy".to_string(),
            load: Some(0.81),
        }],
        recent_events: None,
        preferences: None,
    };

    let suggestions = AISuggestionManager::generate_local_suggestions(&context);
    assert_eq!(suggestions.len(), 1);
    assert_eq!(
        suggestions[0].suggestion_type,
        SuggestionType::ResourceReallocation
    );
}

#[tokio::test]
async fn test_local_suggestions_load_below_threshold() {
    let context = SuggestionContext {
        assignments: HashMap::new(),
        available_devices: vec![],
        running_primals: vec![PrimalInfo {
            id: "toadstool1".to_string(),
            name: "ToadStool".to_string(),
            primal_type: "compute".to_string(),
            capabilities: vec!["compute".to_string()],
            health: "healthy".to_string(),
            load: Some(0.79),
        }],
        recent_events: None,
        preferences: None,
    };

    let suggestions = AISuggestionManager::generate_local_suggestions(&context);
    assert!(
        suggestions.is_empty(),
        "No rebalance suggestion when load < 0.8"
    );
}

#[tokio::test]
async fn test_local_suggestions_primal_no_load_info() {
    let context = SuggestionContext {
        assignments: HashMap::new(),
        available_devices: vec![],
        running_primals: vec![PrimalInfo {
            id: "toadstool1".to_string(),
            name: "ToadStool".to_string(),
            primal_type: "compute".to_string(),
            capabilities: vec!["compute".to_string()],
            health: "healthy".to_string(),
            load: None,
        }],
        recent_events: None,
        preferences: None,
    };

    let suggestions = AISuggestionManager::generate_local_suggestions(&context);
    assert!(
        suggestions.is_empty(),
        "No rebalance suggestion when load is unknown"
    );
}
