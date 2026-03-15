// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! # biomeos-graph
//!
//! Type-safe ingestion layer for Neural API deployment graphs.
//!
//! ## Architecture
//!
//! ```text
//! Human                    Ingestion                  Runtime
//! ┌──────────┐            ┌──────────────┐           ┌──────────────┐
//! │   TOML   │ ────────▶  │ Rust Types   │ ────────▶ │  JSON-RPC    │
//! │  Graphs  │   parse    │ (validated)  │  execute  │  (runtime)   │
//! └──────────┘            └──────────────┘           └──────────────┘
//!     ▲                         │
//!     │                         ▼
//!   Human edits            Compile-time
//!   On-the-fly              guarantees
//! ```
//!
//! ## Why Three Layers?
//!
//! - **TOML**: Human-friendly, easy to edit, version control friendly
//! - **Rust Types**: Type-safe validation at load time, not runtime
//! - **JSON-RPC**: Language-agnostic runtime communication
//!
//! ## Usage
//!
//! ```rust,ignore
//! use biomeos_graph::GraphLoader;
//!
//! // Load and validate graph at startup (fail-fast)
//! let graph = GraphLoader::from_file("graphs/livespore_deploy.toml")?;
//!
//! // Graph is now validated and ready for execution via Neural API
//! println!("Loaded graph: {} with {} nodes", graph.id(), graph.nodes().len());
//! ```

#![forbid(unsafe_code)]
#![warn(missing_docs)]

pub mod continuous;
pub mod error;
pub mod events;
pub mod graph;
pub mod loader;
pub mod metrics;
pub mod node;
pub mod pathway_learner;
pub mod sensor;
pub mod validation;

pub use continuous::{ContinuousExecutor, SessionState, TickClock};
pub use error::{GraphError, Result};
pub use events::{GraphEvent, GraphEventBroadcaster};
pub use graph::{CoordinationPattern, DeploymentGraph, GraphMetadata, TickConfig};
pub use loader::GraphLoader;
pub use node::{GraphNode, NodeConfig, NodeParams};
pub use pathway_learner::{
    GraphAnalysis, OptimizationSuggestion, OptimizationType, PathwayLearner,
};
pub use sensor::{SensorEvent, SensorEventBus, SensorSource};
pub use validation::GraphValidator;

#[cfg(test)]
mod tests;
