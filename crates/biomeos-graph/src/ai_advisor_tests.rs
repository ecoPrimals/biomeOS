// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test")]
#![expect(clippy::float_cmp, reason = "exact f64 comparison in test fixtures")]

use super::*;
use crate::{
    events::GraphEvent,
    graph::{
        CoordinationPattern, EdgeType, GraphEdge, GraphId, Operation, PrimalGraph, PrimalNode,
        PrimalSelector,
    },
};
use chrono::Utc;

fn create_test_graph() -> PrimalGraph {
    PrimalGraph {
        id: GraphId::new("test").unwrap(),
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
                constraints: None,
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
                constraints: None,
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
                constraints: None,
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

    let suggestions = advisor.get_local_suggestions(&graph);
    let suggestion = suggestions
        .iter()
        .find(|s| s.suggestion_type == SuggestionType::PerformanceImprovement);
    assert!(suggestion.is_some());

    let suggestion = suggestion.unwrap();
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
            constraints: None,
        });
    }

    let suggestions = advisor.get_local_suggestions(&graph);
    let suggestion = suggestions
        .iter()
        .find(|s| s.suggestion_type == SuggestionType::BestPractice);
    assert!(suggestion.is_some());
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
#[test]
fn test_graph_snapshot_by_id_and_by_capabilities_keys() {
    let graph = PrimalGraph {
        id: GraphId::new("g").unwrap(),
        name: "g".to_string(),
        description: String::new(),
        version: "1".to_string(),
        coordination: CoordinationPattern::Sequential,
        nodes: vec![
            PrimalNode {
                id: "a".to_string(),
                primal: PrimalSelector::ById {
                    by_id: "beardog".to_string(),
                },
                operation: Operation {
                    name: "op".to_string(),
                    params: serde_json::json!({}),
                    environment: None,
                },
                input: None,
                outputs: vec![],
                constraints: None,
            },
            PrimalNode {
                id: "b".to_string(),
                primal: PrimalSelector::ByCapabilities {
                    by_capabilities: vec!["x".to_string(), "y".to_string()],
                },
                operation: Operation {
                    name: "op2".to_string(),
                    params: serde_json::json!({}),
                    environment: None,
                },
                input: None,
                outputs: vec![],
                constraints: None,
            },
        ],
        edges: vec![],
    };
    let snap = GraphSnapshot::from_graph(&graph);
    assert_eq!(snap.node_count, 2);
    assert!(snap.capabilities.contains_key("beardog"));
    assert!(snap.capabilities.contains_key("x+y"));
}
#[test]
fn test_detect_parallelization_skipped_when_edges_present() {
    let advisor = AiGraphAdvisor::new();
    let mut graph = create_test_graph();
    graph.edges.push(GraphEdge {
        from: "node1".to_string(),
        to: "node2".to_string(),
        edge_type: EdgeType::DataFlow,
    });
    let suggestions = advisor.get_local_suggestions(&graph);
    assert!(
        !suggestions
            .iter()
            .any(|s| s.suggestion_type == SuggestionType::PerformanceImprovement)
    );
}
#[test]
fn test_detect_coordination_dag_when_parallel_has_many_edges() {
    let advisor = AiGraphAdvisor::new();
    let graph = PrimalGraph {
        id: GraphId::new("dag").unwrap(),
        name: "dag".to_string(),
        description: String::new(),
        version: "1".to_string(),
        coordination: CoordinationPattern::Parallel,
        nodes: vec![
            PrimalNode {
                id: "n1".to_string(),
                primal: PrimalSelector::ByCapability {
                    by_capability: "a".to_string(),
                },
                operation: Operation {
                    name: "op".to_string(),
                    params: serde_json::json!({}),
                    environment: None,
                },
                input: None,
                outputs: vec![],
                constraints: None,
            },
            PrimalNode {
                id: "n2".to_string(),
                primal: PrimalSelector::ByCapability {
                    by_capability: "b".to_string(),
                },
                operation: Operation {
                    name: "op".to_string(),
                    params: serde_json::json!({}),
                    environment: None,
                },
                input: None,
                outputs: vec![],
                constraints: None,
            },
        ],
        edges: vec![
            GraphEdge {
                from: "n1".to_string(),
                to: "n2".to_string(),
                edge_type: EdgeType::DataFlow,
            },
            GraphEdge {
                from: "n2".to_string(),
                to: "n1".to_string(),
                edge_type: EdgeType::DataFlow,
            },
            GraphEdge {
                from: "n1".to_string(),
                to: "n2".to_string(),
                edge_type: EdgeType::ControlFlow,
            },
        ],
    };
    let suggestions = advisor.get_local_suggestions(&graph);
    let s = suggestions
        .iter()
        .find(|s| s.suggestion_type == SuggestionType::Optimization);
    assert!(s.is_some());
}
#[test]
fn test_detect_single_node_with_edge_suggests_remove_edge() {
    let advisor = AiGraphAdvisor::new();
    let graph = PrimalGraph {
        id: GraphId::new("one").unwrap(),
        name: "one".to_string(),
        description: String::new(),
        version: "1".to_string(),
        coordination: CoordinationPattern::Sequential,
        nodes: vec![PrimalNode {
            id: "only".to_string(),
            primal: PrimalSelector::ByCapability {
                by_capability: "c".to_string(),
            },
            operation: Operation {
                name: "op".to_string(),
                params: serde_json::json!({}),
                environment: None,
            },
            input: None,
            outputs: vec![],
            constraints: None,
        }],
        edges: vec![GraphEdge {
            from: "only".to_string(),
            to: "only".to_string(),
            edge_type: EdgeType::DataFlow,
        }],
    };
    let suggestions = advisor.get_local_suggestions(&graph);
    let s = suggestions
        .iter()
        .find(|s| s.suggestion_type == SuggestionType::BestPractice);
    assert!(s.is_some());
}
#[tokio::test]
async fn test_learn_from_event_graph_events_ok() {
    let advisor = AiGraphAdvisor::new();
    let failed = GraphEvent::NodeFailed {
        graph_id: "g".to_string(),
        node_id: "n".to_string(),
        error: "e".to_string(),
        retry_attempt: 0,
        will_retry: false,
        timestamp: Utc::now(),
    };
    assert!(advisor.learn_from_event(&failed).await.is_ok());

    let decision = GraphEvent::DecisionMade {
        graph_id: "g".to_string(),
        decision_type: "t".to_string(),
        reasoning: vec!["r".to_string()],
        confidence: 0.9,
        timestamp: Utc::now(),
    };
    assert!(advisor.learn_from_event(&decision).await.is_ok());
}
#[tokio::test]
async fn test_advisor_with_timeout_constructor() {
    let advisor = AiGraphAdvisor::with_timeout(std::time::Duration::from_millis(200));
    let graph = create_test_graph();
    let r = advisor.get_suggestions(&graph).await;
    assert!(r.is_ok());
}
#[test]
fn test_suggestion_feedback_serde_roundtrip() {
    let fb = SuggestionFeedback {
        suggestion_id: "s1".to_string(),
        accepted: true,
        comments: Some("ok".to_string()),
        outcome: Some(FeedbackOutcome {
            success: true,
            performance_delta: Some(0.1),
            satisfaction: Some(5),
        }),
    };
    let json = serde_json::to_string(&fb).unwrap();
    let back: SuggestionFeedback = serde_json::from_str(&json).unwrap();
    assert_eq!(fb.suggestion_id, back.suggestion_id);
    assert!(back.accepted);
}
