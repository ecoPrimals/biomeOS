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
//! ```rust,no_run
//! use biomeos_graph::{DeploymentGraph, GraphLoader};
//!
//! // Load and validate graph at startup (fail-fast)
//! let graph = GraphLoader::from_file("graphs/livespore_deploy.toml")?;
//!
//! // Execute validated graph
//! let executor = GraphExecutor::new(neural_api_socket)?;
//! executor.run(&graph).await?;
//! ```

#![deny(unsafe_code)]
#![warn(missing_docs)]

pub mod error;
pub mod graph;
pub mod loader;
pub mod node;
pub mod validation;

pub use error::{GraphError, Result};
pub use graph::{DeploymentGraph, GraphMetadata};
pub use loader::GraphLoader;
pub use node::{GraphNode, NodeConfig, NodeParams};
pub use validation::GraphValidator;

#[cfg(test)]
mod tests;
