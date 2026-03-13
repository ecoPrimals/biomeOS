// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Capability category for grouping

use serde::{Deserialize, Serialize};
use std::fmt;

/// Capability category for grouping
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityCategory {
    /// Security and cryptography capabilities
    Security,
    /// Communication and networking capabilities
    Communication,
    /// Compute and execution capabilities
    Compute,
    /// Storage and data capabilities
    Storage,
    /// User interface and rendering capabilities
    UserInterface,
    /// Orchestration and management capabilities
    Orchestration,
    /// AI and intelligence capabilities
    AI,
    /// Specialized and custom capabilities
    Specialized,
}

impl fmt::Display for CapabilityCategory {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = format!("{:?}", self);
        write!(f, "{}", s)
    }
}
