// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

use serde::{Deserialize, Serialize};

use crate::error::ParseNodeIdError;

/// Node identifier - validated to be alphanumeric with hyphens.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(try_from = "String", into = "String")]
pub struct NodeId(String);

impl NodeId {
    /// Create a new node ID, validating format.
    pub fn new(id: impl Into<String>) -> Result<Self, ParseNodeIdError> {
        let id = id.into();
        if id.is_empty() {
            return Err(ParseNodeIdError::Empty);
        }
        if !id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        {
            return Err(ParseNodeIdError::InvalidChars(id));
        }
        Ok(Self(id))
    }

    /// Get the ID as a string slice.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for NodeId {
    type Error = ParseNodeIdError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}

impl From<NodeId> for String {
    fn from(id: NodeId) -> Self {
        id.0
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
