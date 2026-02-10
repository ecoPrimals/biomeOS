//! Strong-typed identifiers for BiomeOS domain concepts
//!
//! This module provides NewType wrappers for domain identifiers,
//! ensuring type safety and preventing ID confusion at compile time.
//!
//! # Examples
//!
//! ```
//! use biomeos_types::identifiers::{PrimalId, FamilyId, Endpoint};
//!
//! // Type-safe primal identifier
//! let primal = PrimalId::new("beardog-local").unwrap();
//!
//! // Genetic lineage identifier
//! let family = FamilyId::new("iidn");
//!
//! // Validated endpoint
//! let endpoint = Endpoint::new("http://localhost:9000").unwrap();
//! let health_url = endpoint.join("health").unwrap();
//! ```
//!
//! # Philosophy
//!
//! Make invalid states unrepresentable through the type system.
//! Validation happens at construction time, so downstream code
//! can assume all identifiers are valid.

use serde::{Deserialize, Serialize};
use std::fmt;

/// Primal identifier (strong type)
///
/// A unique identifier for a primal in the ecosystem.
/// Enforces alphanumeric characters, dashes, and underscores only.
///
/// # Examples
///
/// ```
/// use biomeos_types::identifiers::PrimalId;
///
/// let id = PrimalId::new("beardog-local").unwrap();
/// assert_eq!(id.as_str(), "beardog-local");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct PrimalId(String);

impl PrimalId {
    /// Create a new primal ID with validation
    ///
    /// # Errors
    ///
    /// Returns `IdError::Empty` if the ID is empty.
    /// Returns `IdError::InvalidCharacters` if the ID contains invalid characters.
    pub fn new(id: impl Into<String>) -> Result<Self, IdError> {
        let id = id.into();

        if id.is_empty() {
            return Err(IdError::Empty);
        }

        if !id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(IdError::InvalidCharacters);
        }

        Ok(Self(id))
    }

    /// Create unchecked ID (for trusted sources like database)
    ///
    /// # Safety
    ///
    /// Only use this when you know the ID is valid (e.g., from database).
    pub fn new_unchecked(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the inner string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert into owned String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for PrimalId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for PrimalId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Family identifier (genetic lineage)
///
/// Represents a family in the genetic lineage system.
/// Used for auto-trust decisions based on shared genetics.
///
/// # Examples
///
/// ```
/// use biomeos_types::identifiers::FamilyId;
///
/// let family = FamilyId::new("iidn");
/// assert_eq!(family.as_str(), "iidn");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FamilyId(String);

impl FamilyId {
    /// Create a new family ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get family ID from environment variable
    ///
    /// Checks `BIOMEOS_FAMILY_ID` environment variable
    pub fn from_env() -> Option<Self> {
        std::env::var("BIOMEOS_FAMILY_ID").ok().map(Self::new)
    }

    /// Discover local family ID from config
    ///
    /// Checks for existing family configuration in:
    /// - `$XDG_CONFIG_HOME/biomeos/family.txt`
    /// - `~/.config/biomeos/family.txt`
    pub fn discover_local() -> Option<Self> {
        use crate::paths::SystemPaths;

        let paths = SystemPaths::new().ok()?;
        let family_file = paths.config_dir().join("family.txt");

        std::fs::read_to_string(family_file)
            .ok()
            .map(|s| Self::new(s.trim()))
    }

    /// Generate a new random family ID
    ///
    /// Uses a memorable name generator for human-friendly IDs
    pub fn generate() -> Self {
        use uuid::Uuid;
        // Generate memorable ID: first 8 chars of UUID
        let id = Uuid::new_v4().to_string();
        Self::new(&id[..8])
    }

    /// Get or create family ID with fallback chain
    ///
    /// Priority:
    /// 1. Environment variable (`BIOMEOS_FAMILY_ID`)
    /// 2. Local config file
    /// 3. Generate new ID
    pub fn get_or_create() -> Self {
        Self::from_env()
            .or_else(Self::discover_local)
            .unwrap_or_else(Self::generate)
    }

    /// For tests only - deterministic family ID
    #[cfg(test)]
    pub fn new_for_test() -> Self {
        Self::new("test-family")
    }

    /// Get the inner string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert into owned String
    pub fn into_string(self) -> String {
        self.0
    }
}

impl fmt::Display for FamilyId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for FamilyId {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

/// Endpoint URL (strong type with validation)
///
/// A validated HTTP(S) endpoint for primal communication.
/// Ensures URL is well-formed at construction time.
///
/// # Examples
///
/// ```
/// use biomeos_types::identifiers::Endpoint;
///
/// let endpoint = Endpoint::new("http://localhost:9000").unwrap();
/// assert_eq!(endpoint.as_str(), "http://localhost:9000/");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Endpoint(url::Url);

impl Endpoint {
    /// Create a new endpoint with URL validation
    ///
    /// # Errors
    ///
    /// Returns error if the URL is malformed.
    pub fn new(url: impl AsRef<str>) -> Result<Self, url::ParseError> {
        Ok(Self(url::Url::parse(url.as_ref())?))
    }

    /// Get the underlying URL
    pub fn url(&self) -> &url::Url {
        &self.0
    }

    /// Get the URL as a string
    pub fn as_str(&self) -> &str {
        self.0.as_str()
    }

    /// Join a path to the endpoint
    ///
    /// # Examples
    ///
    /// ```
    /// use biomeos_types::identifiers::Endpoint;
    ///
    /// let base = Endpoint::new("http://localhost:9000").unwrap();
    /// let api = base.join("api/v1/health").unwrap();
    /// assert_eq!(api.as_str(), "http://localhost:9000/api/v1/health");
    /// ```
    pub fn join(&self, path: &str) -> Result<Self, url::ParseError> {
        Ok(Self(self.0.join(path)?))
    }
}

impl fmt::Display for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl AsRef<str> for Endpoint {
    fn as_ref(&self) -> &str {
        self.0.as_str()
    }
}

/// Tower identifier (for multi-tower deployments)
///
/// Identifies a specific tower in a distributed deployment.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct TowerId(String);

impl TowerId {
    /// Create a new tower ID
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }

    /// Get the inner string reference
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TowerId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Session identifier (for stateful operations)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct SessionId(uuid::Uuid);

impl SessionId {
    /// Create a new random session ID
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    /// Create from existing UUID
    pub fn from_uuid(id: uuid::Uuid) -> Self {
        Self(id)
    }

    /// Get the underlying UUID
    pub fn uuid(&self) -> &uuid::Uuid {
        &self.0
    }
}

impl Default for SessionId {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for SessionId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Identifier errors
#[derive(Debug, thiserror::Error)]
pub enum IdError {
    /// ID cannot be empty
    #[error("ID cannot be empty")]
    Empty,

    /// ID contains invalid characters
    #[error("ID contains invalid characters (use alphanumeric, dash, underscore only)")]
    InvalidCharacters,

    /// Invalid URL format
    #[error("Invalid URL format: {0}")]
    InvalidUrl(#[from] url::ParseError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn primal_id_valid() {
        assert!(PrimalId::new("beardog-local").is_ok());
        assert!(PrimalId::new("songbird_v2").is_ok());
        assert!(PrimalId::new("tower123").is_ok());
    }

    #[test]
    fn primal_id_invalid() {
        assert!(PrimalId::new("").is_err());
        assert!(PrimalId::new("has spaces").is_err());
        assert!(PrimalId::new("has/slash").is_err());
    }

    #[test]
    fn endpoint_valid() {
        assert!(Endpoint::new("http://localhost:9000").is_ok());
        assert!(Endpoint::new("https://192.168.1.144:8080").is_ok());
    }

    #[test]
    fn endpoint_join() {
        let base = Endpoint::new("http://localhost:9000").unwrap();
        let api = base.join("api/v1/health").unwrap();
        assert_eq!(api.as_str(), "http://localhost:9000/api/v1/health");
    }

    #[test]
    fn family_id_display() {
        let family = FamilyId::new("iidn");
        assert_eq!(format!("{}", family), "iidn");
    }

    #[test]
    fn session_id_unique() {
        let id1 = SessionId::new();
        let id2 = SessionId::new();
        assert_ne!(id1, id2);
    }
}
