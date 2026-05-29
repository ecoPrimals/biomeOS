// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Service status, phases, conditions, and replica tracking.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::health::Health;

/// Service status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceStatus {
    /// Current service phase
    pub phase: ServicePhase,

    /// Service health status
    pub health: Health,

    /// Service conditions
    pub conditions: Vec<ServiceCondition>,

    /// Replica status
    pub replicas: ReplicaStatus,

    /// Observed generation
    pub observed_generation: u64,

    /// Last update time
    pub last_update_time: DateTime<Utc>,

    /// Status message
    pub message: Option<String>,

    /// Status reason
    pub reason: Option<String>,
}

/// Service phases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ServicePhase {
    /// Service is pending
    Pending,

    /// Service is starting
    Starting,

    /// Service is running
    Running,

    /// Service is stopping
    Stopping,

    /// Service has stopped
    Stopped,

    /// Service has failed
    Failed,

    /// Service is unknown state
    Unknown,
}

/// Service condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceCondition {
    /// Condition type
    pub condition_type: String,

    /// Condition status
    pub status: ConditionStatus,

    /// Last transition time
    pub last_transition_time: DateTime<Utc>,

    /// Reason for the condition
    pub reason: Option<String>,

    /// Human readable message
    pub message: Option<String>,
}

/// Condition status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConditionStatus {
    /// Condition is met
    True,
    /// Condition is not met
    False,
    /// Condition status is unknown
    Unknown,
}

/// Replica status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReplicaStatus {
    /// Desired number of replicas
    pub desired: u32,

    /// Current number of replicas
    pub current: u32,

    /// Number of ready replicas
    pub ready: u32,

    /// Number of available replicas
    pub available: u32,

    /// Number of unavailable replicas
    pub unavailable: u32,
}
