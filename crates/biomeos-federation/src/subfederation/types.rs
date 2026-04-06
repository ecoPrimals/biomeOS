// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Sub-federation type definitions

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::capability::{Capability, CapabilitySet};

/// Isolation level for sub-federations
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum IsolationLevel {
    /// No isolation - full federation
    None,

    /// Low isolation - limited capabilities
    Low,

    /// Medium isolation - specific primals only
    Medium,

    /// High isolation - compute-only, no data access
    High,

    /// Critical isolation - air-gapped, manual approval required
    Critical,
}

impl IsolationLevel {
    /// Check if this isolation level allows auto-approval
    #[must_use]
    pub const fn allows_auto_approval(&self) -> bool {
        matches!(self, Self::None | Self::Low | Self::Medium)
    }
}

/// Node ID type
pub type NodeId = String;

/// A sub-federation within the family trust network
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubFederation {
    /// Sub-federation name
    pub name: String,

    /// Parent family ID (genetic lineage)
    pub parent_family: String,

    /// Member node IDs (supports wildcards like "node-*")
    pub members: Vec<String>,

    /// Capabilities granted to members
    pub capabilities: CapabilitySet,

    /// Isolation level
    pub isolation_level: IsolationLevel,

    /// Created timestamp
    pub created_at: DateTime<Utc>,

    /// Metadata
    pub metadata: HashMap<String, String>,

    /// `BearDog` encryption key ID (managed by `BearDog`, not stored here)
    pub encryption_key_ref: Option<String>,
}

impl SubFederation {
    /// Create a new sub-federation
    pub fn new(
        name: String,
        parent_family: String,
        members: Vec<String>,
        capabilities: CapabilitySet,
        isolation_level: IsolationLevel,
    ) -> Self {
        tracing::info!(
            "Creating sub-federation '{}' for family '{}' with {} members",
            name,
            parent_family,
            members.len()
        );

        Self {
            name,
            parent_family,
            members,
            capabilities,
            isolation_level,
            created_at: Utc::now(),
            metadata: HashMap::new(),
            encryption_key_ref: None,
        }
    }

    /// Check if a node is a member of this sub-federation
    #[must_use]
    pub fn is_member(&self, node_id: &str) -> bool {
        self.members.iter().any(|pattern| {
            if pattern.contains('*') {
                let prefix = pattern.trim_end_matches('*');
                node_id.starts_with(prefix)
            } else {
                pattern == node_id
            }
        })
    }

    /// Check if a node has access to a specific capability
    pub fn has_capability(&self, node_id: &str, capability: &Capability) -> bool {
        if !self.is_member(node_id) {
            tracing::debug!(
                "Node {} is not a member of sub-federation {}",
                node_id,
                self.name
            );
            return false;
        }

        if !self.capabilities.has(capability) {
            tracing::debug!(
                "Capability {} not granted in sub-federation {}",
                capability,
                self.name
            );
            return false;
        }

        if !self.isolation_level.allows_auto_approval() {
            tracing::warn!(
                "Sub-federation {} requires manual approval (isolation: {:?})",
                self.name,
                self.isolation_level
            );
            return false;
        }

        true
    }

    /// Add a member to this sub-federation
    pub fn add_member(&mut self, node_id: String) {
        if !self.members.contains(&node_id) {
            self.members.push(node_id);
        }
    }

    /// Remove a member from this sub-federation
    pub fn remove_member(&mut self, node_id: &str) {
        self.members.retain(|id| id != node_id);
    }

    /// Set `BearDog` encryption key reference
    pub fn set_encryption_key_ref(&mut self, key_ref: String) {
        self.encryption_key_ref = Some(key_ref);
    }
}
