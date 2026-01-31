//! Node executor modules
//!
//! **TRUE ecoBin v2.0:** Domain-driven organization.
//!
//! Each module handles a specific domain of node types:
//! - `filesystem` - File and directory operations
//! - `crypto` - Cryptographic operations (delegates to BearDog)
//! - `primal` - Primal lifecycle management
//! - `health` - Health check operations
//! - `lineage` - Genetic lineage verification
//! - `report` - Deployment reporting

pub mod crypto;
pub mod filesystem;
pub mod health;
pub mod lineage;
pub mod primal;
pub mod report;
