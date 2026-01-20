//! Atomic deployment orchestration with genetic lineage
//!
//! This module provides modern, idiomatic Rust for deploying Tower, Node, and Nest
//! atomics from a USB seed with genetic lineage mixing.
//!
//! # Design Principles
//!
//! - **No Jelly Strings**: Pure Rust, no bash scripts
//! - **Neural API Integration**: Deterministic orchestration
//! - **Genetic Lineage**: Cryptographic family trust
//! - **Verifiable Deployment**: Every step logged and checkpointed
//!
//! # Architecture
//!
//! ```text
//! DeploymentOrchestrator
//!     ↓
//!     ├─→ USB Seed Reader (FamilySeed)
//!     ├─→ Child Seed Derivation (per atomic)
//!     ├─→ Primal Launcher (tokio process management)
//!     ├─→ Health Checker (socket + JSON-RPC)
//!     └─→ Neural API Reporter (graph execution)
//! ```

pub mod beardog_jwt_client; // NEW: BearDog JWT integration for orchestrator
pub mod deployment_graph;
pub mod health_check;
pub mod orchestrator;
pub mod primal_coordinator; // NEW: Discovery-based coordination (TRUE PRIMAL)
pub mod primal_discovery; // NEW: Socket scanning discovery
pub mod primal_launcher; // EVOLVING: Legacy launcher → coordinator

// Neural API graph execution (TOML-based deterministic deployment)
pub mod neural_api_server;
pub mod neural_executor;
pub mod neural_graph;
pub mod neural_router; // NEW: Capability-based routing layer

pub use deployment_graph::{AtomicDeploymentGraph, DeploymentResult};
pub use health_check::{HealthChecker, HealthStatus};
pub use orchestrator::{AtomicType, DeploymentConfig, DeploymentOrchestrator};
pub use primal_coordinator::{CoordinationStatus, DeploymentGuide, PrimalCoordinator}; // NEW
pub use primal_discovery::{DiscoveredPrimal, PrimalDiscovery}; // NEW
pub use primal_launcher::{PrimalInstance, PrimalLauncher}; // LEGACY

// Neural API exports
pub use neural_executor::{
    ExecutionContext as NeuralExecutionContext, ExecutionReport,
    GraphExecutor as NeuralGraphExecutor, NodeStatus, PhaseResult,
};
pub use neural_graph::{Graph as NeuralGraph, GraphConfig, GraphNode as NeuralGraphNode};
pub use neural_router::{
    AtomicType as RouterAtomicType, DiscoveredAtomic, DiscoveredPrimal as RouterDiscoveredPrimal,
    NeuralRouter, RoutingMetrics,
};
