// =============================================================================
// Error Types - Modern Rust Error Handling
// =============================================================================

use thiserror::Error;

/// Result type for graph operations
pub type Result<T> = std::result::Result<T, GraphError>;

/// Graph-related errors
#[derive(Error, Debug)]
pub enum GraphError {
    #[error("Graph parsing error: {0}")]
    ParseError(String),
    
    #[error("Graph validation error: {0}")]
    ValidationError(String),
    
    #[error("Graph contains cycle")]
    CyclicGraph,
    
    #[error("Node not found: {0}")]
    NodeNotFound(String),
    
    #[error("Edge references unknown node: {0}")]
    InvalidEdge(String),
    
    #[error("Duplicate node ID: {0}")]
    DuplicateNode(String),
    
    #[error("Execution error: {0}")]
    ExecutionError(String),
    
    #[error("Timeout after {0}ms")]
    Timeout(u64),
    
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("TOML parse error: {0}")]
    TomlParse(#[from] toml::de::Error),
    
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
    
    #[error("Primal capability not found: {0}")]
    CapabilityNotFound(String),
    
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

