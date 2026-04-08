// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use super::types::{AiSuggestion, ImpactEstimate, SuggestionType};
use crate::graph::{CoordinationPattern, Operation, PrimalGraph, PrimalNode, PrimalSelector};
use crate::modification::GraphModification;
use biomeos_types::Uuid;

#[derive(Debug, Clone)]
#[expect(dead_code, reason = "populated for future local fallback suggestions")]
pub(super) struct LocalPattern {
    pub(super) name: String,
    pub(super) description: String,
    pub(super) confidence: f64,
}

pub(super) fn initialize_local_patterns() -> Vec<LocalPattern> {
    vec![
        LocalPattern {
            name: "parallelization".to_string(),
            description: "Detect sequential graphs that could be parallel".to_string(),
            confidence: 0.85,
        },
        LocalPattern {
            name: "error_handling".to_string(),
            description: "Detect missing error handling".to_string(),
            confidence: 0.70,
        },
        LocalPattern {
            name: "coordination".to_string(),
            description: "Suggest better coordination patterns".to_string(),
            confidence: 0.60,
        },
    ]
}

impl super::AiGraphAdvisor {
    pub(super) fn get_local_suggestions(&self, graph: &PrimalGraph) -> Vec<AiSuggestion> {
        let mut suggestions = Vec::new();

        if let Some(suggestion) = self.detect_parallelization_opportunity(graph) {
            suggestions.push(suggestion);
        }

        if let Some(suggestion) = self.detect_missing_error_handling(graph) {
            suggestions.push(suggestion);
        }

        if let Some(suggestion) = self.detect_coordination_improvements(graph) {
            suggestions.push(suggestion);
        }

        suggestions
    }

    pub(super) fn detect_parallelization_opportunity(
        &self,
        graph: &PrimalGraph,
    ) -> Option<AiSuggestion> {
        let _ = self;
        if matches!(graph.coordination, CoordinationPattern::Sequential) && graph.nodes.len() > 2 {
            let has_dependencies = !graph.edges.is_empty();

            if !has_dependencies {
                return Some(AiSuggestion {
                    id: format!("local_parallel_{}", Uuid::new_v4()),
                    suggestion_type: SuggestionType::PerformanceImprovement,
                    modification: GraphModification::ChangeCoordination {
                        pattern: CoordinationPattern::Parallel,
                    },
                    reasoning: format!(
                        "Graph has {} nodes with no dependencies, could execute in parallel for better performance",
                        graph.nodes.len()
                    ),
                    confidence: 0.85,
                    evidence: vec![
                        format!("{} independent nodes", graph.nodes.len()),
                        "No edges defining dependencies".to_string(),
                        "Sequential execution not required".to_string(),
                    ],
                    impact: ImpactEstimate {
                        performance: 0.7,
                        reliability: 0.0,
                        complexity: 0.1,
                        summary: format!("Could improve performance by ~{}x", graph.nodes.len()),
                    },
                });
            }
        }

        None
    }

    pub(super) fn detect_missing_error_handling(
        &self,
        graph: &PrimalGraph,
    ) -> Option<AiSuggestion> {
        let _ = self;
        let has_retry = false;

        if !has_retry && graph.nodes.len() > 3 {
            return Some(AiSuggestion {
                id: format!("local_retry_{}", Uuid::new_v4()),
                suggestion_type: SuggestionType::BestPractice,
                modification: GraphModification::AddNode {
                    node: PrimalNode {
                        id: "error_handler".to_string(),
                        primal: PrimalSelector::ByCapability {
                            by_capability: "error-handling".to_string(),
                        },
                        operation: Operation {
                            name: "handle_errors".to_string(),
                            params: serde_json::json!({}),
                            environment: None,
                        },
                        input: None,
                        outputs: vec![],
                        constraints: None,
                    },
                },
                reasoning: "No error handling nodes detected. Consider adding error handling for better reliability.".to_string(),
                confidence: 0.70,
                evidence: vec![
                    "No nodes with retry policies".to_string(),
                    format!("Graph has {} nodes that could fail", graph.nodes.len()),
                ],
                impact: ImpactEstimate {
                    performance: -0.05,
                    reliability: 0.8,
                    complexity: 0.2,
                    summary: "Improves reliability at small performance cost".to_string(),
                },
            });
        }

        None
    }

    pub(super) fn detect_coordination_improvements(
        &self,
        graph: &PrimalGraph,
    ) -> Option<AiSuggestion> {
        let _ = self;
        if matches!(graph.coordination, CoordinationPattern::Parallel) && graph.edges.len() > 2 {
            return Some(AiSuggestion {
                id: format!("local_dag_{}", Uuid::new_v4()),
                suggestion_type: SuggestionType::Optimization,
                modification: GraphModification::ChangeCoordination {
                    pattern: CoordinationPattern::ConditionalDag,
                },
                reasoning: format!(
                    "Parallel graph has {} edges defining dependencies. Consider DAG coordination for proper dependency ordering.",
                    graph.edges.len()
                ),
                confidence: 0.75,
                evidence: vec![
                    format!("{} edges in parallel graph", graph.edges.len()),
                    "Dependencies should be respected".to_string(),
                    "DAG provides optimal parallel execution with dependencies".to_string(),
                ],
                impact: ImpactEstimate {
                    performance: 0.3,
                    reliability: 0.5,
                    complexity: 0.1,
                    summary: "Improves correctness while maintaining parallelism".to_string(),
                },
            });
        }

        if graph.nodes.len() == 1 && !graph.edges.is_empty() {
            if let Some(edge) = graph.edges.first() {
                return Some(AiSuggestion {
                    id: format!("local_simplify_{}", Uuid::new_v4()),
                    suggestion_type: SuggestionType::BestPractice,
                    modification: GraphModification::RemoveEdge {
                        from: edge.from.clone(),
                        to: edge.to.clone(),
                    },
                    reasoning: "Single-node graph has edges which are unnecessary".to_string(),
                    confidence: 0.95,
                    evidence: vec![
                        "Only one node exists".to_string(),
                        format!("{} unnecessary edges", graph.edges.len()),
                    ],
                    impact: ImpactEstimate {
                        performance: 0.1,
                        reliability: 0.1,
                        complexity: -0.3,
                        summary: "Simplifies graph structure".to_string(),
                    },
                });
            }
        }

        None
    }
}
