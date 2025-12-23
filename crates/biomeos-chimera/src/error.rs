//! Chimera-specific error types
//!
//! Provides detailed, actionable errors for chimera operations.

use std::path::PathBuf;
use thiserror::Error;

/// Result type for chimera operations
pub type ChimeraResult<T> = Result<T, ChimeraError>;

/// Errors that can occur during chimera operations
#[derive(Error, Debug)]
pub enum ChimeraError {
    /// Chimera definition file not found
    #[error("Chimera definition not found: {path}")]
    DefinitionNotFound {
        /// Path that was searched
        path: PathBuf,
    },

    /// Failed to parse chimera definition YAML
    #[error("Failed to parse chimera definition '{id}': {message}")]
    ParseError {
        /// Chimera ID
        id: String,
        /// Parse error message
        message: String,
        /// Source file
        source_file: Option<PathBuf>,
    },

    /// Required primal component not available
    #[error("Required primal '{primal}' not available for chimera '{chimera}'")]
    PrimalNotAvailable {
        /// Chimera that requires the primal
        chimera: String,
        /// Missing primal name
        primal: String,
    },

    /// Primal module not found
    #[error("Module '{module}' not found in primal '{primal}'")]
    ModuleNotFound {
        /// Primal name
        primal: String,
        /// Missing module name
        module: String,
    },

    /// Fusion configuration error
    #[error("Invalid fusion '{fusion}' in chimera '{chimera}': {message}")]
    FusionError {
        /// Chimera ID
        chimera: String,
        /// Fusion name
        fusion: String,
        /// Error message
        message: String,
    },

    /// Build error
    #[error("Failed to build chimera '{chimera}': {message}")]
    BuildError {
        /// Chimera ID
        chimera: String,
        /// Error message
        message: String,
    },

    /// Version incompatibility
    #[error("Version mismatch for primal '{primal}': requires {required}, found {found}")]
    VersionMismatch {
        /// Primal name
        primal: String,
        /// Required version constraint
        required: String,
        /// Found version
        found: String,
    },

    /// Registry error
    #[error("Registry error: {0}")]
    RegistryError(String),

    /// IO error
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    /// YAML parsing error
    #[error("YAML error: {0}")]
    Yaml(#[from] serde_yaml::Error),

    /// JSON error
    #[error("JSON error: {0}")]
    Json(#[from] serde_json::Error),
}

impl ChimeraError {
    /// Create a parse error
    pub fn parse(id: impl Into<String>, message: impl Into<String>) -> Self {
        Self::ParseError {
            id: id.into(),
            message: message.into(),
            source_file: None,
        }
    }

    /// Create a parse error with source file
    pub fn parse_with_source(
        id: impl Into<String>,
        message: impl Into<String>,
        source: impl Into<PathBuf>,
    ) -> Self {
        Self::ParseError {
            id: id.into(),
            message: message.into(),
            source_file: Some(source.into()),
        }
    }

    /// Create a fusion error
    pub fn fusion(
        chimera: impl Into<String>,
        fusion: impl Into<String>,
        message: impl Into<String>,
    ) -> Self {
        Self::FusionError {
            chimera: chimera.into(),
            fusion: fusion.into(),
            message: message.into(),
        }
    }

    /// Create a build error
    pub fn build(chimera: impl Into<String>, message: impl Into<String>) -> Self {
        Self::BuildError {
            chimera: chimera.into(),
            message: message.into(),
        }
    }
}

