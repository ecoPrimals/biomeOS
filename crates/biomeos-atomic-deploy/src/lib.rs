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

pub mod orchestrator;
pub mod primal_launcher;
pub mod health_check;
pub mod deployment_graph;

// Neural API graph execution (TOML-based deterministic deployment)
pub mod neural_graph;
pub mod neural_executor;

pub use orchestrator::{DeploymentOrchestrator, DeploymentConfig, AtomicType};
pub use primal_launcher::{PrimalLauncher, PrimalInstance};
pub use health_check::{HealthChecker, HealthStatus};
pub use deployment_graph::{AtomicDeploymentGraph, DeploymentResult};

// Neural API exports
pub use neural_graph::{Graph as NeuralGraph, GraphNode as NeuralGraphNode, GraphConfig};
pub use neural_executor::{
    GraphExecutor as NeuralGraphExecutor, 
    ExecutionContext as NeuralExecutionContext,
    ExecutionReport, PhaseResult, NodeStatus
};

