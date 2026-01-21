//! AI-powered graph advisor using Squirrel integration
//!
//! This module provides AI-driven suggestions and learning for graph modifications.
//! It learns from user modifications and provides intelligent recommendations
//! through Squirrel integration.
//!
//! Deep Debt Principles:
//! - Capability-based Squirrel discovery (no hardcoding)
//! - Graceful degradation without Squirrel
//! - Modern async Rust
//! - No unsafe code

use crate::events::GraphEvent;
use crate::graph::{Operation, PrimalGraph, PrimalNode, PrimalSelector};
use crate::modification::GraphModification;
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::time::{timeout, Duration};

/// AI suggestion from Squirrel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AiSuggestion {
    /// Unique ID for this suggestion
    pub id: String,

    /// Type of suggestion
    pub suggestion_type: SuggestionType,

    /// The suggested modification
    pub modification: GraphModification,

    /// Human-readable reasoning
    pub reasoning: String,

    /// Confidence level (0.0 - 1.0)
    pub confidence: f64,

    /// Supporting evidence
    pub evidence: Vec<String>,

    /// Estimated impact
    pub impact: ImpactEstimate,
}

/// Type of AI suggestion
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SuggestionType {
    /// Optimization suggestion
    Optimization,

    /// Error prevention
    ErrorPrevention,

    /// Performance improvement
    PerformanceImprovement,

    /// Best practice recommendation
    BestPractice,

    /// Pattern-based suggestion
    PatternBased,

    /// Learning-based suggestion
    LearningBased,
}

/// Estimated impact of a suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImpactEstimate {
    /// Performance impact (-1.0 to 1.0, positive is better)
    pub performance: f64,

    /// Reliability impact (-1.0 to 1.0, positive is better)
    pub reliability: f64,

    /// Complexity impact (-1.0 to 1.0, negative means simpler)
    pub complexity: f64,

    /// Human-readable summary
    pub summary: String,
}

/// User feedback on a suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionFeedback {
    /// ID of the suggestion
    pub suggestion_id: String,

    /// Whether the user accepted the suggestion
    pub accepted: bool,

    /// Optional user comments
    pub comments: Option<String>,

    /// Outcome after applying (if accepted)
    pub outcome: Option<FeedbackOutcome>,
}

/// Outcome of applying a suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackOutcome {
    /// Whether it worked as expected
    pub success: bool,

    /// Performance change observed
    pub performance_delta: Option<f64>,

    /// User satisfaction (1-5)
    pub satisfaction: Option<u8>,
}

/// Learning event for Squirrel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEvent {
    /// Type of event
    pub event_type: String,

    /// Graph state before
    pub before: GraphSnapshot,

    /// Graph state after
    pub after: GraphSnapshot,

    /// User action taken
    pub action: GraphModification,

    /// Context about why the user made this change
    pub context: HashMap<String, String>,
}

/// Snapshot of graph state for learning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSnapshot {
    /// Number of nodes
    pub node_count: usize,

    /// Number of edges
    pub edge_count: usize,

    /// Coordination pattern
    pub coordination: String,

    /// Capability distribution
    pub capabilities: HashMap<String, usize>,
}

impl GraphSnapshot {
    pub fn from_graph(graph: &PrimalGraph) -> Self {
        let mut capabilities: HashMap<String, usize> = HashMap::new();

        for node in &graph.nodes {
            let cap = match &node.primal {
                PrimalSelector::ByCapability { by_capability } => by_capability.clone(),
                PrimalSelector::ByCapabilities { by_capabilities } => by_capabilities.join("+"),
                PrimalSelector::ById { by_id } => by_id.clone(),
            };
            *capabilities.entry(cap).or_insert(0) += 1;
        }

        Self {
            node_count: graph.nodes.len(),
            edge_count: graph.edges.len(),
            coordination: format!("{:?}", graph.coordination),
            capabilities,
        }
    }
}

/// AI-powered graph advisor
pub struct AiGraphAdvisor {
    /// Whether Squirrel is available
    squirrel_available: bool,

    /// Timeout for Squirrel requests
    squirrel_timeout: Duration,

    /// Local suggestion cache (fallback when Squirrel unavailable)
    local_patterns: Vec<LocalPattern>,
}

/// Local pattern recognition (fallback when Squirrel unavailable)
#[derive(Debug, Clone)]
struct LocalPattern {
    name: String,
    description: String,
    confidence: f64,
}

impl AiGraphAdvisor {
    /// Create a new AI advisor
    pub fn new() -> Self {
        Self {
            squirrel_available: false,
            squirrel_timeout: Duration::from_secs(5),
            local_patterns: Self::initialize_local_patterns(),
        }
    }

    /// Create advisor with custom timeout
    pub fn with_timeout(timeout: Duration) -> Self {
        Self {
            squirrel_available: false,
            squirrel_timeout: timeout,
            local_patterns: Self::initialize_local_patterns(),
        }
    }

    /// Check if Squirrel is available
    pub async fn check_squirrel_availability(&mut self) -> Result<bool> {
        // TODO: Implement actual Squirrel discovery via Songbird
        // For now, we'll check if Squirrel is reachable

        // This would use the biomeos-core SquirrelClient when available
        // let squirrel = SquirrelClient::discover().await?;
        // self.squirrel_available = squirrel.health_check().await.is_ok();

        // For now, gracefully degrade
        self.squirrel_available = false;
        Ok(self.squirrel_available)
    }

    /// Get AI suggestions for a graph
    pub async fn get_suggestions(&self, graph: &PrimalGraph) -> Result<Vec<AiSuggestion>> {
        if self.squirrel_available {
            self.get_squirrel_suggestions(graph).await
        } else {
            Ok(self.get_local_suggestions(graph))
        }
    }

    /// Get suggestions from Squirrel
    async fn get_squirrel_suggestions(&self, graph: &PrimalGraph) -> Result<Vec<AiSuggestion>> {
        // TODO: Implement actual Squirrel integration
        // This would call Squirrel's analyze_graph method

        let result = timeout(self.squirrel_timeout, async {
            // Placeholder for Squirrel call
            // let squirrel = SquirrelClient::discover().await?;
            // squirrel.analyze_graph(graph).await
            Ok::<Vec<AiSuggestion>, anyhow::Error>(Vec::new())
        })
        .await;

        match result {
            Ok(Ok(suggestions)) => Ok(suggestions),
            Ok(Err(e)) => {
                // Squirrel failed, fall back to local
                eprintln!("Squirrel request failed: {}, using local patterns", e);
                Ok(self.get_local_suggestions(graph))
            }
            Err(_) => {
                // Timeout, fall back to local
                eprintln!("Squirrel request timed out, using local patterns");
                Ok(self.get_local_suggestions(graph))
            }
        }
    }

    /// Get suggestions using local pattern matching (fallback)
    fn get_local_suggestions(&self, graph: &PrimalGraph) -> Vec<AiSuggestion> {
        let mut suggestions = Vec::new();

        // Pattern 1: Detect sequential execution that could be parallel
        if let Some(suggestion) = self.detect_parallelization_opportunity(graph) {
            suggestions.push(suggestion);
        }

        // Pattern 2: Detect missing error handling
        if let Some(suggestion) = self.detect_missing_error_handling(graph) {
            suggestions.push(suggestion);
        }

        // Pattern 3: Detect inefficient coordination patterns
        if let Some(suggestion) = self.detect_coordination_improvements(graph) {
            suggestions.push(suggestion);
        }

        suggestions
    }

    /// Detect opportunities for parallelization
    fn detect_parallelization_opportunity(&self, graph: &PrimalGraph) -> Option<AiSuggestion> {
        use crate::graph::CoordinationPattern;

        // If graph is sequential but has independent nodes, suggest parallel
        if matches!(graph.coordination, CoordinationPattern::Sequential) && graph.nodes.len() > 2 {
            // Check if nodes have dependencies
            let has_dependencies = !graph.edges.is_empty();

            if !has_dependencies {
                return Some(AiSuggestion {
                    id: format!("local_parallel_{}", uuid::Uuid::new_v4()),
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

    /// Detect missing error handling nodes
    fn detect_missing_error_handling(&self, graph: &PrimalGraph) -> Option<AiSuggestion> {
        // Check if any nodes have retry policies
        // Note: Retry policies would be tracked separately in execution context
        let has_retry = false; // Simplified for now - can be enhanced via execution metadata

        if !has_retry && graph.nodes.len() > 3 {
            // Suggest adding retry to critical nodes
            return Some(AiSuggestion {
                id: format!("local_retry_{}", uuid::Uuid::new_v4()),
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

    /// Detect coordination pattern improvements
    fn detect_coordination_improvements(&self, _graph: &PrimalGraph) -> Option<AiSuggestion> {
        // TODO: Implement more sophisticated pattern detection
        None
    }

    /// Send learning event to Squirrel
    pub async fn send_learning_event(&self, event: LearningEvent) -> Result<()> {
        if !self.squirrel_available {
            // Log locally for future batch sending
            return Ok(());
        }

        // TODO: Implement actual Squirrel learning
        // let squirrel = SquirrelClient::discover().await?;
        // squirrel.learn_from_event(event).await?;

        Ok(())
    }

    /// Send feedback on a suggestion
    pub async fn send_feedback(&self, _feedback: SuggestionFeedback) -> Result<()> {
        if !self.squirrel_available {
            return Ok(());
        }

        // TODO: Implement actual Squirrel feedback
        // let squirrel = SquirrelClient::discover().await?;
        // squirrel.record_feedback(feedback).await?;

        Ok(())
    }

    /// Learn from graph events
    pub async fn learn_from_event(&self, event: &GraphEvent) -> Result<()> {
        // Extract learning signals from events
        match event {
            GraphEvent::NodeFailed { node_id, error, .. } => {
                // Learn about failure patterns
                let context = HashMap::from([
                    ("node_id".to_string(), node_id.clone()),
                    ("error".to_string(), error.clone()),
                ]);

                // Would send to Squirrel for learning
                let _ = context; // Use context when Squirrel integration is complete
            }
            GraphEvent::DecisionMade { reasoning, .. } => {
                // Learn from AI decisions and their outcomes
                let _ = reasoning; // Use reasoning when Squirrel integration is complete
            }
            _ => {}
        }

        Ok(())
    }

    /// Initialize local pattern recognition
    fn initialize_local_patterns() -> Vec<LocalPattern> {
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
}

impl Default for AiGraphAdvisor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
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
}
