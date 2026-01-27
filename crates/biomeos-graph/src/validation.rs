//! Enhanced graph validation with primal availability checking
//!
//! This module extends basic graph validation with runtime primal discovery
//! and capability verification through Songbird and BearDog integration.
//!
//! Deep Debt Principles:
//! - Capability-based validation (discover primals, don't hardcode)
//! - Graceful degradation (works without Songbird/BearDog)
//! - Modern async Rust
//! - No unsafe code

use crate::graph::{PrimalGraph, PrimalSelector};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};

/// Validation report for a graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationReport {
    /// Whether the graph is valid
    pub valid: bool,

    /// Critical errors that prevent execution
    pub errors: Vec<ValidationError>,

    /// Warnings that may affect execution
    pub warnings: Vec<ValidationWarning>,

    /// Suggestions for improvement
    pub suggestions: Vec<ValidationSuggestion>,

    /// Primal availability status
    pub primal_availability: HashMap<String, PrimalAvailability>,
}

impl ValidationReport {
    pub fn new() -> Self {
        Self {
            valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
            suggestions: Vec::new(),
            primal_availability: HashMap::new(),
        }
    }

    pub fn add_error(&mut self, error: ValidationError) {
        self.valid = false;
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: ValidationWarning) {
        self.warnings.push(warning);
    }

    pub fn add_suggestion(&mut self, suggestion: ValidationSuggestion) {
        self.suggestions.push(suggestion);
    }

    /// Check if the report has any issues
    pub fn has_issues(&self) -> bool {
        !self.errors.is_empty() || !self.warnings.is_empty()
    }
}

impl Default for ValidationReport {
    fn default() -> Self {
        Self::new()
    }
}

/// Validation error
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationError {
    pub code: String,
    pub message: String,
    pub node_id: Option<String>,
}

impl ValidationError {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            node_id: None,
        }
    }

    pub fn with_node(mut self, node_id: impl Into<String>) -> Self {
        self.node_id = Some(node_id.into());
        self
    }
}

/// Validation warning
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationWarning {
    pub code: String,
    pub message: String,
    pub node_id: Option<String>,
}

impl ValidationWarning {
    pub fn new(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            message: message.into(),
            node_id: None,
        }
    }

    pub fn with_node(mut self, node_id: impl Into<String>) -> Self {
        self.node_id = Some(node_id.into());
        self
    }
}

/// Validation suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationSuggestion {
    pub category: String,
    pub message: String,
    pub node_id: Option<String>,
}

impl ValidationSuggestion {
    pub fn new(category: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            category: category.into(),
            message: message.into(),
            node_id: None,
        }
    }

    pub fn with_node(mut self, node_id: impl Into<String>) -> Self {
        self.node_id = Some(node_id.into());
        self
    }
}

/// Primal availability status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrimalAvailability {
    /// Primal is available and healthy
    Available { capabilities: Vec<String> },

    /// Primal is unavailable
    Unavailable { reason: String },

    /// Availability check was skipped
    Skipped { reason: String },

    /// Availability is unknown
    Unknown,
}

/// Enhanced graph validator
pub struct EnhancedGraphValidator {
    check_primal_availability: bool,
}

impl EnhancedGraphValidator {
    /// Create a new enhanced validator
    pub fn new() -> Self {
        Self {
            check_primal_availability: true,
        }
    }

    /// Create a validator that skips primal availability checks
    pub fn without_availability_checks() -> Self {
        Self {
            check_primal_availability: false,
        }
    }

    /// Validate a graph
    pub fn validate(&self, graph: &PrimalGraph) -> Result<ValidationReport> {
        let mut report = ValidationReport::new();

        // Basic structural validation
        self.validate_structure(graph, &mut report)?;

        // Node validation
        self.validate_nodes(graph, &mut report)?;

        // Edge validation
        self.validate_edges(graph, &mut report)?;

        // Cycle detection
        self.validate_no_cycles(graph, &mut report)?;

        // Performance suggestions
        self.add_performance_suggestions(graph, &mut report);

        // Primal availability (if enabled)
        if self.check_primal_availability {
            self.check_primal_availability_sync(graph, &mut report);
        }

        Ok(report)
    }

    fn validate_structure(&self, graph: &PrimalGraph, report: &mut ValidationReport) -> Result<()> {
        // Check for empty graph
        if graph.nodes.is_empty() {
            report.add_error(ValidationError::new("EMPTY_GRAPH", "Graph has no nodes"));
        }

        // Check for duplicate node IDs
        let mut seen_ids = HashSet::new();
        for node in &graph.nodes {
            if !seen_ids.insert(&node.id) {
                report.add_error(
                    ValidationError::new(
                        "DUPLICATE_NODE_ID",
                        format!("Duplicate node ID: {}", node.id),
                    )
                    .with_node(&node.id),
                );
            }
        }

        // Check for reasonable graph size
        if graph.nodes.len() > 1000 {
            report.add_warning(ValidationWarning::new(
                "LARGE_GRAPH",
                format!(
                    "Graph has {} nodes, consider breaking into sub-graphs",
                    graph.nodes.len()
                ),
            ));
        }

        Ok(())
    }

    fn validate_nodes(&self, graph: &PrimalGraph, report: &mut ValidationReport) -> Result<()> {
        for node in &graph.nodes {
            // Validate node ID
            if node.id.is_empty() {
                report.add_error(ValidationError::new("EMPTY_NODE_ID", "Node has empty ID"));
            }

            // Validate operation
            if node.operation.name.is_empty() {
                report.add_error(
                    ValidationError::new(
                        "EMPTY_OPERATION",
                        format!("Node '{}' has empty operation name", node.id),
                    )
                    .with_node(&node.id),
                );
            }

            // Validate primal selector
            self.validate_primal_selector(&node.primal, &node.id, report);
        }

        Ok(())
    }

    fn validate_primal_selector(
        &self,
        selector: &PrimalSelector,
        node_id: &str,
        report: &mut ValidationReport,
    ) {
        match selector {
            PrimalSelector::ById { by_id } => {
                if by_id.is_empty() {
                    report.add_error(
                        ValidationError::new(
                            "EMPTY_PRIMAL_ID",
                            format!("Node '{}' has empty primal ID", node_id),
                        )
                        .with_node(node_id),
                    );
                }
            }
            PrimalSelector::ByCapability { by_capability } => {
                if by_capability.is_empty() {
                    report.add_error(
                        ValidationError::new(
                            "EMPTY_CAPABILITY",
                            format!("Node '{}' has empty capability selector", node_id),
                        )
                        .with_node(node_id),
                    );
                }
            }
            PrimalSelector::ByCapabilities { by_capabilities } => {
                if by_capabilities.is_empty() {
                    report.add_error(
                        ValidationError::new(
                            "EMPTY_CAPABILITIES",
                            format!("Node '{}' has empty capabilities list", node_id),
                        )
                        .with_node(node_id),
                    );
                }
            }
        }
    }

    fn validate_edges(&self, graph: &PrimalGraph, report: &mut ValidationReport) -> Result<()> {
        let node_ids: HashSet<_> = graph.nodes.iter().map(|n| n.id.as_str()).collect();

        for edge in &graph.edges {
            // Check source node exists
            if !node_ids.contains(edge.from.as_str()) {
                report.add_error(ValidationError::new(
                    "INVALID_EDGE_SOURCE",
                    format!("Edge references non-existent source node '{}'", edge.from),
                ));
            }

            // Check target node exists
            if !node_ids.contains(edge.to.as_str()) {
                report.add_error(ValidationError::new(
                    "INVALID_EDGE_TARGET",
                    format!("Edge references non-existent target node '{}'", edge.to),
                ));
            }

            // Check for self-loops
            if edge.from == edge.to {
                report.add_warning(
                    ValidationWarning::new(
                        "SELF_LOOP",
                        format!("Edge from '{}' to itself", edge.from),
                    )
                    .with_node(&edge.from),
                );
            }
        }

        Ok(())
    }

    fn validate_no_cycles(&self, graph: &PrimalGraph, report: &mut ValidationReport) -> Result<()> {
        // Build adjacency list from dependency edges
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();

        for node in &graph.nodes {
            adj.insert(node.id.clone(), Vec::new());
        }

        for edge in &graph.edges {
            // Only check dependency edges for cycles
            if matches!(edge.edge_type, crate::graph::EdgeType::Dependency) {
                adj.entry(edge.from.clone())
                    .or_default()
                    .push(edge.to.clone());
            }
        }

        // Use topological sort to detect cycles (Kahn's algorithm)
        let mut in_degree: HashMap<String, usize> = HashMap::new();

        // Initialize in-degrees
        for node in &graph.nodes {
            in_degree.insert(node.id.clone(), 0);
        }

        // Calculate in-degrees
        for neighbors in adj.values() {
            for neighbor in neighbors {
                if let Some(degree) = in_degree.get_mut(neighbor) {
                    *degree += 1;
                }
                // If neighbor doesn't exist in in_degree, it's an invalid edge
                // This will be caught by validate_edges
            }
        }

        // Find all nodes with in-degree 0
        let mut queue: Vec<String> = in_degree
            .iter()
            .filter(|(_, &degree)| degree == 0)
            .map(|(id, _)| id.clone())
            .collect();

        let mut processed = 0;

        while let Some(node) = queue.pop() {
            processed += 1;

            if let Some(neighbors) = adj.get(&node) {
                for neighbor in neighbors {
                    if let Some(degree) = in_degree.get_mut(neighbor) {
                        *degree -= 1;
                        if *degree == 0 {
                            queue.push(neighbor.clone());
                        }
                    }
                    // If neighbor doesn't exist, it's an invalid edge (caught by validate_edges)
                }
            }
        }

        // If we couldn't process all nodes, there's a cycle
        if processed != graph.nodes.len() {
            report.add_error(ValidationError::new(
                "DEPENDENCY_CYCLE",
                "Graph contains a dependency cycle",
            ));
        }

        Ok(())
    }

    fn add_performance_suggestions(&self, graph: &PrimalGraph, report: &mut ValidationReport) {
        // Check for nodes with no edges (potential parallelization)
        let nodes_with_deps: HashSet<_> = graph.edges.iter().map(|e| e.to.as_str()).collect();

        let independent_nodes: Vec<_> = graph
            .nodes
            .iter()
            .filter(|n| !nodes_with_deps.contains(n.id.as_str()))
            .collect();

        if independent_nodes.len() > 1 {
            report.add_suggestion(ValidationSuggestion::new(
                "PARALLELIZATION",
                format!(
                    "{} nodes have no dependencies and could be executed in parallel",
                    independent_nodes.len()
                ),
            ));
        }

        // Check for long chains (potential pipeline pattern)
        let max_chain_length = self.find_longest_chain(graph);
        if max_chain_length > 10 {
            report.add_suggestion(ValidationSuggestion::new(
                "PIPELINE",
                format!(
                    "Graph has a chain of {} nodes, consider pipeline pattern",
                    max_chain_length
                ),
            ));
        }
    }

    fn find_longest_chain(&self, graph: &PrimalGraph) -> usize {
        let mut adj: HashMap<String, Vec<String>> = HashMap::new();

        for node in &graph.nodes {
            adj.insert(node.id.clone(), Vec::new());
        }

        for edge in &graph.edges {
            if matches!(edge.edge_type, crate::graph::EdgeType::Dependency) {
                adj.entry(edge.from.clone())
                    .or_default()
                    .push(edge.to.clone());
            }
        }

        let mut max_length = 0;
        let mut memo = HashMap::new();

        for node in &graph.nodes {
            let length = self.dfs_chain_length(&node.id, &adj, &mut memo);
            max_length = max_length.max(length);
        }

        max_length
    }

    fn dfs_chain_length(
        &self,
        node: &str,
        adj: &HashMap<String, Vec<String>>,
        memo: &mut HashMap<String, usize>,
    ) -> usize {
        // Check memo first
        if let Some(&length) = memo.get(node) {
            return length;
        }

        // Mark as being computed (prevents infinite recursion on cycles)
        memo.insert(node.to_string(), 0);

        let mut max_child = 0;
        if let Some(neighbors) = adj.get(node) {
            for neighbor in neighbors {
                let child_length = self.dfs_chain_length(neighbor, adj, memo);
                max_child = max_child.max(child_length);
            }
        }

        let length = 1 + max_child;
        memo.insert(node.to_string(), length);
        length
    }

    fn check_primal_availability_sync(&self, graph: &PrimalGraph, report: &mut ValidationReport) {
        // Extract required capabilities from graph
        let mut required_capabilities: HashSet<String> = HashSet::new();

        for node in &graph.nodes {
            match &node.primal {
                PrimalSelector::ById { by_id } => {
                    required_capabilities.insert(by_id.clone());
                }
                PrimalSelector::ByCapability { by_capability } => {
                    required_capabilities.insert(by_capability.clone());
                }
                PrimalSelector::ByCapabilities { by_capabilities } => {
                    required_capabilities.extend(by_capabilities.iter().cloned());
                }
            }
        }

        // For now, mark all as unknown (requires async Songbird integration)
        for capability in required_capabilities {
            report.primal_availability.insert(
                capability.clone(),
                PrimalAvailability::Skipped {
                    reason: "Primal availability checking requires async context".to_string(),
                },
            );
        }

        report.add_suggestion(ValidationSuggestion::new(
            "PRIMAL_DISCOVERY",
            "Use async validate_with_discovery() for runtime primal availability checking",
        ));
    }
}

impl Default for EnhancedGraphValidator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::{CoordinationPattern, EdgeType, GraphEdge, GraphId, Operation, PrimalNode};

    fn create_test_node(id: &str, capability: &str) -> PrimalNode {
        PrimalNode {
            id: id.to_string(),
            primal: PrimalSelector::ByCapability {
                by_capability: capability.to_string(),
            },
            operation: Operation {
                name: "test_op".to_string(),
                params: serde_json::Value::Null,
                environment: None,
            },
            input: None,
            outputs: vec![],
        }
    }

    fn create_test_graph() -> PrimalGraph {
        PrimalGraph {
            id: GraphId::new("test"),
            name: "test".to_string(),
            description: "Test graph".to_string(),
            version: "1.0.0".to_string(),
            coordination: CoordinationPattern::Sequential,
            nodes: vec![
                create_test_node("node1", "capability1"),
                create_test_node("node2", "capability2"),
            ],
            edges: vec![GraphEdge {
                from: "node1".to_string(),
                to: "node2".to_string(),
                edge_type: EdgeType::Dependency,
            }],
        }
    }

    #[test]
    fn test_valid_graph() {
        let graph = create_test_graph();
        let validator = EnhancedGraphValidator::new();
        let report = validator.validate(&graph).unwrap();

        assert!(report.valid);
        assert!(report.errors.is_empty());
    }

    #[test]
    fn test_empty_graph() {
        let mut graph = create_test_graph();
        graph.nodes.clear();
        graph.edges.clear();

        let validator = EnhancedGraphValidator::new();
        let report = validator.validate(&graph).unwrap();

        assert!(!report.valid);
        assert!(!report.errors.is_empty());
        assert!(report.errors.iter().any(|e| e.code == "EMPTY_GRAPH"));
    }

    #[test]
    fn test_duplicate_node_ids() {
        let mut graph = create_test_graph();
        graph.nodes.push(create_test_node("node1", "capability3")); // Duplicate ID

        let validator = EnhancedGraphValidator::new();
        let report = validator.validate(&graph).unwrap();

        assert!(!report.valid);
        assert!(report.errors.iter().any(|e| e.code == "DUPLICATE_NODE_ID"));
    }

    #[test]
    fn test_invalid_edge_reference() {
        let mut graph = create_test_graph();
        graph.edges.push(GraphEdge {
            from: "node1".to_string(),
            to: "nonexistent".to_string(),
            edge_type: EdgeType::Dependency,
        });

        let validator = EnhancedGraphValidator::new();
        let report = validator.validate(&graph).unwrap();

        assert!(!report.valid);
        assert!(report
            .errors
            .iter()
            .any(|e| e.code == "INVALID_EDGE_TARGET"));
    }

    #[test]
    fn test_cycle_detection() {
        let mut graph = create_test_graph();
        // Create cycle: node1 -> node2 -> node1
        graph.edges.push(GraphEdge {
            from: "node2".to_string(),
            to: "node1".to_string(),
            edge_type: EdgeType::Dependency,
        });

        let validator = EnhancedGraphValidator::new();
        let report = validator.validate(&graph).unwrap();

        assert!(!report.valid);
        assert!(report.errors.iter().any(|e| e.code == "DEPENDENCY_CYCLE"));
    }

    #[test]
    fn test_self_loop_warning() {
        let mut graph = create_test_graph();
        graph.edges.push(GraphEdge {
            from: "node1".to_string(),
            to: "node1".to_string(),
            edge_type: EdgeType::Dependency,
        });

        let validator = EnhancedGraphValidator::new();
        let report = validator.validate(&graph).unwrap();

        assert!(report.warnings.iter().any(|w| w.code == "SELF_LOOP"));
    }

    #[test]
    fn test_empty_capability_error() {
        let mut graph = create_test_graph();
        graph.nodes[0].primal = PrimalSelector::ByCapability {
            by_capability: "".to_string(),
        };

        let validator = EnhancedGraphValidator::new();
        let report = validator.validate(&graph).unwrap();

        assert!(report.errors.iter().any(|e| e.code == "EMPTY_CAPABILITY"));
    }

    #[test]
    fn test_parallelization_suggestion() {
        let mut graph = create_test_graph();
        // Add third node with no dependencies
        graph.nodes.push(create_test_node("node3", "capability3"));

        let validator = EnhancedGraphValidator::new();
        let report = validator.validate(&graph).unwrap();

        assert!(report
            .suggestions
            .iter()
            .any(|s| s.category == "PARALLELIZATION"));
    }

    #[test]
    fn test_validation_report_methods() {
        let mut report = ValidationReport::new();
        assert!(report.valid);
        assert!(!report.has_issues());

        report.add_error(ValidationError::new("TEST_ERROR", "Test error"));
        assert!(!report.valid);
        assert!(report.has_issues());

        report.add_warning(ValidationWarning::new("TEST_WARNING", "Test warning"));
        assert!(report.has_issues());
    }
}
