// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! AI-powered suggestions for Interactive UI
//!
//! Integrates with AI-capable primals to provide intelligent suggestions
//! for device assignments, optimizations, and bottleneck predictions.
//!
//! Deep Debt Principles:
//! - No hardcoding (discover AI provider via capabilities)
//! - Modern async Rust (tokio)
//! - No unsafe code
//! - Graceful degradation (works without AI)

mod manager;
pub mod types;

pub use manager::AISuggestionManager;
pub use types::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

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
                load: Some(0.9), // 90% load
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
                load: Some(0.81), // Just over 0.8 threshold
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
                load: Some(0.79), // Below 0.8
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

        // Send accepted feedback
        let result = manager.send_feedback(&suggestion.id, &SuggestionFeedback::Accepted);
        assert!(result.is_ok());

        // Should be removed from active suggestions
        assert_eq!(manager.active_suggestions.len(), 0);
    }

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
    async fn test_discover_ai_provider() {
        let mut manager = AISuggestionManager::new("test_family".to_string());

        // discover_ai_provider scans for actual sockets - returns Ok even if not found
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

        // Dismissed feedback should NOT remove the suggestion
        assert_eq!(manager.active_suggestions.len(), 1);
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

        // Serialization test
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
        assert!(primal.load.unwrap() < 0.8); // Not overloaded
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
}
