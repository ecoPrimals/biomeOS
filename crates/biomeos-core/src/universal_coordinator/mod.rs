//! Universal Biome Coordinator
//!
//! This module provides the universal coordinator that can bootstrap and manage
//! biome ecosystems using any Primal implementations through capability-based
//! discovery and routing.

pub mod types;
pub mod client;
pub mod config;
pub mod router;
pub mod matcher;
pub mod coordinator;

// Re-export the main types and traits
pub use types::{
    UniversalBiomeCoordinator, CoordinatorConfig, EcosystemInstance, EcosystemStatus,
    DeployedPrimal, PrimalStatus, DeploymentPlan, PrimalAssignment, ResourcePlan,
    ResourcePool, ResourceAllocation, CapabilityRouter, RequirementMatcher,
    MatchingConfig, ScoringWeights, RoutingStrategy, MatchingAlgorithm,
    MatchResult, MatchDetails, HttpPrimalClient,
};

pub use client::PrimalClient;

pub use router::RouterStats;

pub use coordinator::CoordinatorStats; 