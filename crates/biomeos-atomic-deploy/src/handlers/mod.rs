//! Handler modules for Neural API Server.
//!
//! Smart decomposition - each handler focuses on one domain:
//! - `graph`: Graph CRUD and execution
//! - `capability`: Capability routing and discovery
//! - `topology`: System topology and metrics
//! - `niche`: Niche template deployment
//!
//! # Architecture
//!
//! ```text
//! NeuralApiServer
//!   ├── GraphHandler      (CRUD, execute)
//!   ├── CapabilityHandler (routing, discovery)
//!   ├── TopologyHandler   (primals, metrics)
//!   └── NicheHandler      (templates)
//! ```
//!
//! # Capability-Based Design
//!
//! All handlers use capability-based discovery instead of hardcoded primal names.
//! This enables TRUE PRIMAL architecture where primals are discovered by what
//! they can do, not what they're named.

pub mod capability;
pub mod graph;
pub mod niche;
pub mod topology;

pub use capability::CapabilityHandler;
pub use graph::{ExecutionStatus, GraphHandler};
pub use niche::NicheHandler;
pub use topology::TopologyHandler;
