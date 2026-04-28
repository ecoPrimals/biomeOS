// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Error types for graph operations.

use thiserror::Error;

/// Result type for graph operations.
pub type Result<T> = std::result::Result<T, GraphError>;

/// Errors that can occur during graph operations.
#[derive(Debug, Error)]
pub enum GraphError {
    /// IO error (file not found, permission denied, etc.)
    #[error("IO error: {0}")]
    Io(String),

    /// TOML parsing error
    #[error("Parse error: {0}")]
    Parse(String),

    /// Validation error (structural issues)
    #[error("Validation error: {0}")]
    Validation(String),

    /// Cyclic dependency detected
    #[error("Cyclic dependency: {0}")]
    CyclicDependency(String),

    /// Missing dependency
    #[error("Missing dependency: {0}")]
    MissingDependency(String),

    /// Execution error
    #[error("Execution error: {0}")]
    Execution(String),

    /// Capability not found
    #[error("Capability not found: {0}")]
    CapabilityNotFound(String),

    /// Node not found in graph
    #[error("Node not found: {0}")]
    NodeNotFound(String),

    /// Graph integrity check failed (hash mismatch or invalid signature)
    #[error("Integrity error: {0}")]
    Integrity(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_io_error_display() {
        let err = GraphError::Io("file not found".to_string());
        assert_eq!(err.to_string(), "IO error: file not found");
    }

    #[test]
    fn test_parse_error_display() {
        let err = GraphError::Parse("invalid TOML".to_string());
        assert_eq!(err.to_string(), "Parse error: invalid TOML");
    }

    #[test]
    fn test_validation_error_display() {
        let err = GraphError::Validation("missing required field".to_string());
        assert_eq!(err.to_string(), "Validation error: missing required field");
    }

    #[test]
    fn test_cyclic_dependency_display() {
        let err = GraphError::CyclicDependency("A -> B -> A".to_string());
        assert_eq!(err.to_string(), "Cyclic dependency: A -> B -> A");
    }

    #[test]
    fn test_missing_dependency_display() {
        let err = GraphError::MissingDependency("node-x".to_string());
        assert_eq!(err.to_string(), "Missing dependency: node-x");
    }

    #[test]
    fn test_execution_error_display() {
        let err = GraphError::Execution("timeout".to_string());
        assert_eq!(err.to_string(), "Execution error: timeout");
    }

    #[test]
    fn test_capability_not_found_display() {
        let err = GraphError::CapabilityNotFound("crypto.encrypt".to_string());
        assert_eq!(err.to_string(), "Capability not found: crypto.encrypt");
    }

    #[test]
    fn test_node_not_found_display() {
        let err = GraphError::NodeNotFound("missing-node".to_string());
        assert_eq!(err.to_string(), "Node not found: missing-node");
    }

    #[test]
    fn test_integrity_error_display() {
        let err = GraphError::Integrity("hash mismatch".to_string());
        assert_eq!(err.to_string(), "Integrity error: hash mismatch");
    }

    #[test]
    fn test_result_type() {
        let ok_result: Result<i32> = Ok(42);
        match ok_result {
            Ok(val) => assert_eq!(val, 42),
            Err(e) => panic!("Expected Ok(42), got Err({e})"),
        }

        let err_result: Result<i32> = Err(GraphError::Io("test".to_string()));
        assert!(err_result.is_err());
    }
}
