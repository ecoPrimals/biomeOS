// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Node configuration types for spore incubation.
//!
//! Defines the data structures persisted when a spore is incubated on a local computer,
//! including node identity, lineage, spore provenance, and federation membership.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

use crate::incubation::local_entropy::LocalEntropy;

/// Result of incubating a spore on a local computer.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IncubatedNode {
    /// Unique node identifier (format: `node-{spore_id}-{hostname}`)
    pub node_id: String,
    /// Identifier of the parent spore
    pub spore_id: String,
    /// SHA-256 hash of the deployed seed (spore seed mixed with local entropy)
    pub deployed_seed_hash: String,
    /// Path to the local configuration directory
    pub local_config_path: PathBuf,
    /// When this node was incubated
    pub incubated_at: DateTime<Utc>,
    /// Hash of the local entropy used during incubation
    pub entropy_hash: String,
    /// Path to the original spore (if available)
    pub spore_path: Option<PathBuf>,
}

/// Full node configuration persisted as `node.toml` in the deployed-nodes directory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    /// Node identity information
    pub node: NodeInfo,
    /// Genetic lineage chain
    pub lineage: LineageInfo,
    /// Parent spore provenance
    pub spore: SporeInfo,
    /// Federation membership
    pub federation: FederationInfo,
}

/// Node identity details.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// Identifier of the parent spore
    pub spore_id: String,
    /// Unique node identifier
    pub node_id: String,
    /// When the node was deployed
    pub deployed_at: DateTime<Utc>,
    /// Name of the host computer
    pub computer_name: String,
    /// Hash of the entropy collected at deployment time
    pub entropy_hash: String,
}

/// Genetic lineage information linking a node to its seed chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LineageInfo {
    /// SHA-256 hash of the parent family seed
    pub parent_seed_hash: String,
    /// SHA-256 hash of the spore seed
    pub spore_seed_hash: String,
    /// SHA-256 hash of the deployed (mixed) seed
    pub deployed_seed_hash: String,
}

/// Provenance information about the parent spore.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SporeInfo {
    /// Filesystem path to the original spore (e.g. USB mount point)
    pub original_path: Option<PathBuf>,
    /// When the spore was last observed
    pub last_seen: DateTime<Utc>,
    /// How many times this spore has been deployed
    pub deployment_count: u32,
}

/// Federation membership information for a deployed node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FederationInfo {
    /// Family identifier (genetic root)
    pub family_id: String,
    /// Sub-federation memberships
    pub sub_federations: Vec<String>,
}

/// Parameters for creating a local node configuration during incubation.
pub struct CreateLocalConfigParams<'a> {
    /// Directory where the config will be written
    pub config_path: &'a Path,
    /// Spore identifier
    pub spore_id: &'a str,
    /// Node identifier
    pub node_id: &'a str,
    /// Hash of the deployed seed
    pub deployed_seed_hash: &'a str,
    /// Hash of the local entropy
    pub entropy_hash: &'a str,
    /// Computer hostname
    pub computer_name: &'a str,
    /// Full local entropy data (persisted as `entropy.json`)
    pub local_entropy: &'a LocalEntropy,
}
