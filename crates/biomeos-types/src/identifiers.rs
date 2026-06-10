// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Strong-typed identifiers for `BiomeOS` domain concepts
//!
//! This module provides `NewType` wrappers for domain identifiers,
//! ensuring type safety and preventing ID confusion at compile time.
//!
//! # Zero-copy design
//!
//! Identifiers use `Arc<str>` internally for cheap cloning in hot paths
//! (discovery, registry lookups, channel passing).
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

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::fmt;
use std::sync::Arc;

/// Primal identifier (strong type)
///
/// A unique identifier for a primal in the ecosystem.
/// Enforces alphanumeric characters, dashes, and underscores only.
/// Uses `Arc<str>` for zero-copy cloning in hot paths.
///
/// # Examples
///
/// ```
/// # fn main() -> Result<(), Box<dyn std::error::Error>> {
/// use biomeos_types::identifiers::{IdError, PrimalId};
///
/// let id = PrimalId::new("beardog-local")?;
/// assert_eq!(id.as_str(), "beardog-local");
/// assert!(PrimalId::new("bad id").is_err());
/// assert!(matches!(PrimalId::new(""), Err(IdError::Empty)));
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct PrimalId(Arc<str>);

impl PrimalId {
    /// Create a new primal ID with validation.
    ///
    /// # Errors
    ///
    /// Returns `IdError::Empty` if the ID is empty.
    /// Returns `IdError::InvalidCharacters` if the ID contains invalid characters.
    ///
    /// # Examples
    ///
    /// ```
    /// use biomeos_types::PrimalId;
    /// let id = PrimalId::new("beardog-local").unwrap();
    /// assert_eq!(id.as_str(), "beardog-local");
    /// ```
    ///
    /// Invalid IDs are rejected:
    ///
    /// ```
    /// use biomeos_types::PrimalId;
    /// assert!(PrimalId::new("").is_err());
    /// assert!(PrimalId::new("has spaces").is_err());
    /// ```
    pub fn new(id: impl AsRef<str>) -> Result<Self, IdError> {
        let id = id.as_ref();

        if id.is_empty() {
            return Err(IdError::Empty);
        }

        if !id
            .chars()
            .all(|c| c.is_alphanumeric() || c == '-' || c == '_')
        {
            return Err(IdError::InvalidCharacters);
        }

        Ok(Self(Arc::from(id)))
    }

    /// Create unchecked ID (for trusted sources like database)
    ///
    /// # Safety
    ///
    /// Only use this when you know the ID is valid (e.g., from database).
    pub fn new_unchecked(id: impl AsRef<str>) -> Self {
        Self(Arc::from(id.as_ref()))
    }

    /// Get the inner string reference
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert into owned String
    #[must_use]
    pub fn into_string(self) -> String {
        self.0.to_string()
    }
}

impl Serialize for PrimalId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for PrimalId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Self::new(s).map_err(serde::de::Error::custom)
    }
}

impl From<PrimalId> for String {
    fn from(id: PrimalId) -> Self {
        id.into_string()
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
/// Uses `Arc<str>` for zero-copy cloning in hot paths.
///
/// # Examples
///
/// ```
/// use biomeos_types::identifiers::FamilyId;
///
/// let family = FamilyId::new("iidn");
/// assert_eq!(family.as_str(), "iidn");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct FamilyId(Arc<str>);

impl FamilyId {
    /// Create a new family ID.
    ///
    /// # Examples
    ///
    /// ```
    /// use biomeos_types::FamilyId;
    /// let family = FamilyId::new("iidn");
    /// assert_eq!(family.as_str(), "iidn");
    /// ```
    pub fn new(id: impl AsRef<str>) -> Self {
        Self(Arc::from(id.as_ref()))
    }

    /// Get family ID from environment variable
    ///
    /// Checks `BIOMEOS_FAMILY_ID` environment variable
    pub fn from_env() -> Option<Self> {
        Self::from_env_or_override(None)
    }

    /// Resolve family ID from an explicit value or `BIOMEOS_FAMILY_ID` (no env mutation needed in tests).
    #[must_use]
    pub fn from_env_or_override(env_value: Option<&str>) -> Option<Self> {
        env_value.map(Self::new).or_else(|| {
            std::env::var(crate::env_config::vars::FAMILY_ID)
                .ok()
                .map(Self::new)
        })
    }

    /// Discover local family ID from config
    ///
    /// Checks for existing family configuration in:
    /// - `$XDG_CONFIG_HOME/biomeos/family.txt`
    /// - `~/.config/biomeos/family.txt`
    #[must_use]
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
    #[must_use]
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
    #[must_use]
    pub fn get_or_create() -> Self {
        Self::get_or_create_with(None)
    }

    /// Get or create family ID with explicit env override (for testing)
    pub fn get_or_create_with(env_value: Option<&str>) -> Self {
        env_value
            .map(Self::new)
            .or_else(Self::discover_local)
            .unwrap_or_else(Self::generate)
    }

    /// For tests only - deterministic family ID
    #[cfg(test)]
    pub fn new_for_test() -> Self {
        Self::new("test-family")
    }

    /// Get the inner string reference
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Convert into owned String
    #[must_use]
    pub fn into_string(self) -> String {
        self.0.to_string()
    }
}

impl Serialize for FamilyId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for FamilyId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::new(s))
    }
}

impl From<FamilyId> for String {
    fn from(id: FamilyId) -> Self {
        id.into_string()
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
    #[must_use]
    pub const fn url(&self) -> &url::Url {
        &self.0
    }

    /// Get the URL as a string
    #[must_use]
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
/// Uses `Arc<str>` for zero-copy cloning in hot paths.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct TowerId(Arc<str>);

impl TowerId {
    /// Create a new tower ID
    pub fn new(id: impl AsRef<str>) -> Self {
        Self(Arc::from(id.as_ref()))
    }

    /// Get the inner string reference
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Serialize for TowerId {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(self.as_str())
    }
}

impl<'de> Deserialize<'de> for TowerId {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Ok(Self::new(s))
    }
}

impl From<TowerId> for String {
    fn from(id: TowerId) -> Self {
        id.0.to_string()
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
    #[must_use]
    pub fn new() -> Self {
        Self(uuid::Uuid::new_v4())
    }

    /// Create from existing UUID
    #[must_use]
    pub const fn from_uuid(id: uuid::Uuid) -> Self {
        Self(id)
    }

    /// Get the underlying UUID
    #[must_use]
    pub const fn uuid(&self) -> &uuid::Uuid {
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
#[path = "identifiers_tests.rs"]
mod tests;
