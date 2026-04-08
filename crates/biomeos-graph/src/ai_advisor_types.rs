// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use crate::graph::{PrimalGraph, PrimalSelector};
use crate::modification::GraphModification;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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
    /// Build a snapshot from a primal graph.
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
