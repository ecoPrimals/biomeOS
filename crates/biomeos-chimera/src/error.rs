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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_definition_not_found_error() {
        let err = ChimeraError::DefinitionNotFound {
            path: PathBuf::from("/path/to/chimera.yaml"),
        };
        let msg = err.to_string();
        assert!(msg.contains("not found"));
        assert!(msg.contains("/path/to/chimera.yaml"));
    }

    #[test]
    fn test_parse_error() {
        let err = ChimeraError::parse("test-chimera", "invalid syntax");
        let msg = err.to_string();
        assert!(msg.contains("test-chimera"));
        assert!(msg.contains("invalid syntax"));
    }

    #[test]
    fn test_parse_error_with_source() {
        let err =
            ChimeraError::parse_with_source("test-chimera", "missing field", "/path/to/file.yaml");
        match err {
            ChimeraError::ParseError {
                id,
                message,
                source_file,
            } => {
                assert_eq!(id, "test-chimera");
                assert_eq!(message, "missing field");
                assert!(source_file.is_some());
            }
            _ => panic!("Expected ParseError"),
        }
    }

    #[test]
    fn test_primal_not_available_error() {
        let err = ChimeraError::PrimalNotAvailable {
            chimera: "my-chimera".to_string(),
            primal: "beardog".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("beardog"));
        assert!(msg.contains("my-chimera"));
    }

    #[test]
    fn test_module_not_found_error() {
        let err = ChimeraError::ModuleNotFound {
            primal: "toadstool".to_string(),
            module: "gpu-compute".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("gpu-compute"));
        assert!(msg.contains("toadstool"));
    }

    #[test]
    fn test_fusion_error() {
        let err = ChimeraError::fusion("chimera-1", "api-fusion", "binding mismatch");
        let msg = err.to_string();
        assert!(msg.contains("chimera-1"));
        assert!(msg.contains("api-fusion"));
        assert!(msg.contains("binding mismatch"));
    }

    #[test]
    fn test_build_error() {
        let err = ChimeraError::build("gaming-mesh", "compilation failed");
        let msg = err.to_string();
        assert!(msg.contains("gaming-mesh"));
        assert!(msg.contains("compilation failed"));
    }

    #[test]
    fn test_version_mismatch_error() {
        let err = ChimeraError::VersionMismatch {
            primal: "beardog".to_string(),
            required: ">=2.0.0".to_string(),
            found: "1.5.0".to_string(),
        };
        let msg = err.to_string();
        assert!(msg.contains("beardog"));
        assert!(msg.contains(">=2.0.0"));
        assert!(msg.contains("1.5.0"));
    }

    #[test]
    fn test_registry_error() {
        let err = ChimeraError::RegistryError("connection refused".to_string());
        let msg = err.to_string();
        assert!(msg.contains("connection refused"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let err: ChimeraError = io_err.into();
        let msg = err.to_string();
        assert!(msg.contains("file not found"));
    }
}
