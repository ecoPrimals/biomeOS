// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! AI advisor tests (extracted from ai_advisor.rs).

#![expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]

use super::*;
use crate::graph::{CoordinationPattern, EdgeType, GraphEdge, GraphId};

fn create_test_graph() -> PrimalGraph {
    PrimalGraph {
        id: GraphId::new("test"),
        name: "test".to_string(),
        description: "Test graph".to_string(),
        version: "1.0.0".to_string(),
        coordination: CoordinationPattern::Sequential,
        nodes: vec![
            PrimalNode {
                id: "node1".to_string(),
                primal: PrimalSelector::ByCapability {
                    by_capability: "compute".to_string(),
                },
                operation: Operation {
                    name: "process".to_string(),
                    params: serde_json::json!({}),
                    environment: None,
                },
                input: None,
                outputs: vec![],
            },
            PrimalNode {
                id: "node2".to_string(),
                primal: PrimalSelector::ByCapability {
                    by_capability: "storage".to_string(),
                },
                operation: Operation {
                    name: "store".to_string(),
                    params: serde_json::json!({}),
                    environment: None,
                },
                input: None,
                outputs: vec![],
            },
            PrimalNode {
                id: "node3".to_string(),
                primal: PrimalSelector::ByCapability {
                    by_capability: "network".to_string(),
                },
                operation: Operation {
                    name: "send".to_string(),
                    params: serde_json::json!({}),
                    environment: None,
                },
                input: None,
                outputs: vec![],
            },
        ],
        edges: vec![],
    }
}

#[test]
fn test_advisor_creation() {
    let advisor = AiGraphAdvisor::new();
    assert!(!advisor.squirrel_available);
    assert_eq!(advisor.local_patterns.len(), 3);
}

#[test]
fn test_graph_snapshot() {
    let graph = create_test_graph();
    let snapshot = GraphSnapshot::from_graph(&graph);

    assert_eq!(snapshot.node_count, 3);
    assert_eq!(snapshot.edge_count, 0);
    assert_eq!(snapshot.capabilities.len(), 3);
    assert_eq!(snapshot.capabilities.get("compute"), Some(&1));
    assert_eq!(snapshot.capabilities.get("storage"), Some(&1));
    assert_eq!(snapshot.capabilities.get("network"), Some(&1));
}

#[test]
fn test_detect_parallelization() {
    let advisor = AiGraphAdvisor::new();
    let graph = create_test_graph();

    let suggestion = advisor.detect_parallelization_opportunity(&graph);
    assert!(suggestion.is_some());

    let suggestion = suggestion.unwrap();
    assert_eq!(
        suggestion.suggestion_type,
        SuggestionType::PerformanceImprovement
    );
    assert!(suggestion.confidence > 0.8);
    assert!(!suggestion.evidence.is_empty());
}

#[test]
fn test_detect_missing_error_handling() {
    let advisor = AiGraphAdvisor::new();
    let mut graph = create_test_graph();

    // Add more nodes to trigger suggestion
    for i in 3..=5 {
        graph.nodes.push(PrimalNode {
            id: format!("node{}", i),
            primal: PrimalSelector::ByCapability {
                by_capability: "compute".to_string(),
            },
            operation: Operation {
                name: "process".to_string(),
                params: serde_json::json!({}),
                environment: None,
            },
            input: None,
            outputs: vec![],
        });
    }

    let suggestion = advisor.detect_missing_error_handling(&graph);
    assert!(suggestion.is_some());

    let suggestion = suggestion.unwrap();
    assert_eq!(suggestion.suggestion_type, SuggestionType::BestPractice);
}

#[tokio::test]
async fn test_get_local_suggestions() {
    let advisor = AiGraphAdvisor::new();
    let graph = create_test_graph();

    let suggestions = advisor.get_local_suggestions(&graph);
    assert!(!suggestions.is_empty());
}

#[tokio::test]
async fn test_check_availability_graceful_failure() {
    let mut advisor = AiGraphAdvisor::new();
    let result = advisor.check_squirrel_availability().await;

    assert!(result.is_ok());
    assert!(!advisor.squirrel_available);
}

#[tokio::test]
async fn test_get_suggestions_without_squirrel() {
    let advisor = AiGraphAdvisor::new();
    let graph = create_test_graph();

    let result = advisor.get_suggestions(&graph).await;
    assert!(result.is_ok());

    let suggestions = result.unwrap();
    assert!(!suggestions.is_empty());
}

#[test]
fn test_ai_advisor_default() {
    let advisor = AiGraphAdvisor::default();
    assert!(!advisor.squirrel_available);
}

#[test]
fn test_suggestion_type_serde_roundtrip() {
    for st in [
        SuggestionType::Optimization,
        SuggestionType::ErrorPrevention,
        SuggestionType::PerformanceImprovement,
        SuggestionType::BestPractice,
        SuggestionType::PatternBased,
        SuggestionType::LearningBased,
    ] {
        let json = serde_json::to_string(&st).unwrap();
        let restored: SuggestionType = serde_json::from_str(&json).unwrap();
        assert_eq!(st, restored);
    }
}

#[test]
fn test_impact_estimate_serde_roundtrip() {
    let impact = ImpactEstimate {
        performance: 0.5,
        reliability: 0.8,
        complexity: -0.2,
        summary: "test".to_string(),
    };
    let json = serde_json::to_string(&impact).unwrap();
    let restored: ImpactEstimate = serde_json::from_str(&json).unwrap();
    assert_eq!(impact.performance, restored.performance);
    assert_eq!(impact.summary, restored.summary);
}
