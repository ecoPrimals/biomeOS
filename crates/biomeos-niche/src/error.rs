// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Niche-specific error types

use std::path::PathBuf;
use thiserror::Error;

/// Result type for niche operations
pub type NicheResult<T> = Result<T, NicheError>;

/// Errors that can occur during niche operations
#[derive(Error, Debug)]
pub enum NicheError {
    /// Niche definition not found
    #[error("Niche definition not found: {path}")]
    DefinitionNotFound {
        /// Path searched
        path: PathBuf,
    },

    /// Failed to parse niche definition
    #[error("Failed to parse niche definition '{id}': {message}")]
    ParseError {
        /// Niche ID
        id: String,
        /// Error message
        message: String,
    },

    /// Required organism not available
    #[error("Required organism '{organism}' not available for niche '{niche}'")]
    OrganismNotAvailable {
        /// Niche ID
        niche: String,
        /// Missing organism
        organism: String,
    },

    /// Invalid interaction
    #[error("Invalid interaction in niche '{niche}': {message}")]
    InvalidInteraction {
        /// Niche ID
        niche: String,
        /// Error message
        message: String,
    },

    /// Deployment error
    #[error("Failed to deploy niche '{niche}': {message}")]
    DeploymentError {
        /// Niche ID
        niche: String,
        /// Error message
        message: String,
    },

    /// Chimera error
    #[error("Chimera error: {0}")]
    Chimera(#[from] biomeos_chimera::ChimeraError),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// YAML error
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),
}

impl NicheError {
    /// Create a parse error
    pub fn parse(id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ParseError {
            id: id.into(),
            message: message.into(),
        }
    }

    /// Create a deployment error
    pub fn deployment(niche: impl Into<String>, message: impl Into<String>) -> Self {
        Self::DeploymentError {
            niche: niche.into(),
            message: message.into(),
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_error_display() {
        let err = NicheError::parse("test-niche", "invalid yaml");
        assert!(err.to_string().contains("test-niche"));
        assert!(err.to_string().contains("invalid yaml"));
    }

    #[test]
    fn test_deployment_error_display() {
        let err = NicheError::deployment("my-niche", "organism failed");
        assert!(err.to_string().contains("my-niche"));
        assert!(err.to_string().contains("organism failed"));
    }

    #[test]
    fn test_definition_not_found_display() {
        let err = NicheError::DefinitionNotFound {
            path: PathBuf::from("/tmp/missing.yaml"),
        };
        assert!(err.to_string().contains("definition not found"));
        assert!(err.to_string().contains("/tmp/missing.yaml"));
    }

    #[test]
    fn test_organism_not_available_display() {
        let err = NicheError::OrganismNotAvailable {
            niche: "n1".to_string(),
            organism: "beardog".to_string(),
        };
        assert!(err.to_string().contains("beardog"));
        assert!(err.to_string().contains("n1"));
    }

    #[test]
    fn test_invalid_interaction_display() {
        let err = NicheError::InvalidInteraction {
            niche: "n1".to_string(),
            message: "unknown from".to_string(),
        };
        assert!(err.to_string().contains("Invalid interaction"));
        assert!(err.to_string().contains("unknown from"));
    }

    #[test]
    fn test_from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: NicheError = io_err.into();
        assert!(err.to_string().contains("file not found"));
    }

    #[test]
    fn test_from_yaml_error() {
        let yaml_err = serde_yaml::from_str::<String>("invalid: yaml: [").unwrap_err();
        let err: NicheError = yaml_err.into();
        assert!(err.to_string().contains("YAML"));
    }

    #[test]
    fn test_niche_result_type() {
        let ok_result: NicheResult<i32> = Ok(42);
        assert!(ok_result.is_ok());
        assert_eq!(ok_result.as_ref().unwrap(), &42);

        let err_result: NicheResult<i32> = Err(NicheError::parse("id", "msg"));
        assert!(err_result.is_err());
    }
}
