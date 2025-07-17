//! # Cross-Primal API Contracts for biomeOS
//!
//! This module defines standardized API contracts for communication between
//! all primals in the biomeOS ecosystem, ensuring consistent interfaces,
//! error handling, and data validation across the federation.

pub mod errors;
pub mod lifecycle;
pub mod middleware;
pub mod requests;
pub mod responses;
pub mod traits;
pub mod types;
pub mod validation;

// Re-export all public types and traits
pub use errors::*;
pub use lifecycle::*;
pub use middleware::*;
pub use requests::*;
pub use responses::*;
pub use traits::*;
pub use types::*;
pub use validation::*;
