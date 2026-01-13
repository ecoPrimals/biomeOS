// =============================================================================
// biomeos-graph - Graph-Based Orchestration
// =============================================================================
//
// Adaptive primal coordination through graph execution.
//
// Deep Debt Principles:
// - Modern idiomatic Rust (no unsafe, Result<T,E>, async/await)
// - Capability-based (discover primals, don't hardcode)
// - Self-knowledge only (no assumptions about other primals)
// - Mocks isolated to #[cfg(test)]
//
// =============================================================================

pub mod ai_advisor;
pub mod events;
pub mod executor;
pub mod graph;
pub mod modification;
pub mod parser;
pub mod templates;
pub mod validation;
pub mod validator;
// pub mod nucleus_executor; // TODO: Re-enable after Wave 2 evolution to use CapabilityTaxonomy
pub mod context;
pub mod error;
pub mod metrics;

// Neural API modules moved to biomeos-atomic-deploy to avoid circular dependency

// Re-export core types
pub use graph::{
    CoordinationPattern, EdgeType, Graph, GraphConfig, GraphEdge, GraphId, GraphNode, GraphResult,
    NodeConstraints, NodeMetrics, NodeOutput, Operation, PrimalGraph, PrimalNode, PrimalSelector,
    RetryPolicy,
};

pub use ai_advisor::{
    AiGraphAdvisor, AiSuggestion, FeedbackOutcome, GraphSnapshot, ImpactEstimate, LearningEvent,
    SuggestionFeedback, SuggestionType,
};
pub use events::{EventCollector, GraphEvent, GraphEventBroadcaster};
pub use executor::{GraphExecutor, PrimalOperationExecutor};
pub use modification::{GraphModification, GraphModificationHandler, ModificationResult};
pub use parser::GraphParser;
pub use templates::{GraphTemplate, GraphTemplateManager, ParameterType, TemplateParameter};
pub use validation::{
    EnhancedGraphValidator, PrimalAvailability, ValidationError, ValidationReport,
    ValidationSuggestion, ValidationWarning,
};
pub use validator::GraphValidator;
// pub use nucleus_executor::NucleusPrimalExecutor; // TODO: Re-enable after Wave 2 evolution
pub use context::ExecutionContext;
pub use error::{GraphError, Result};
pub use metrics::{ExecutionRecord, GraphMetrics, MetricsCollector, NodeMetricsAggregate};

// Neural API moved to biomeos-atomic-deploy crate
