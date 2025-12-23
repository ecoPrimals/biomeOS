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

