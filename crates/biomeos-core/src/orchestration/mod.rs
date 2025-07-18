//! Universal Orchestration Interface
//!
//! This module provides a universal orchestration interface that eliminates
//! vendor lock-in across different orchestration platforms like Kubernetes,
//! Nomad, Docker Swarm, and others.

pub mod types;
pub mod interface;
pub mod workload;
pub mod service;
pub mod cluster;
pub mod manager;

// Re-export main types and traits
pub use types::*;
pub use interface::{UniversalOrchestrationInterface, ExtendedOrchestrationInterface};
pub use workload::*;
pub use service::*;
pub use cluster::*;
pub use manager::{UniversalOrchestrationManager, OrchestrationSovereigntyRequirements};

// Re-export commonly used types
pub use interface::{
    WorkloadMetrics, LogStream, PortForwardSession, NetworkPolicySpec, ResourceQuotaSpec,
    PersistentVolumeSpec, BackupSpec, RestoreSpec, RetentionPolicy,
};

pub use manager::{ValidationResult, ManagerStatistics}; 