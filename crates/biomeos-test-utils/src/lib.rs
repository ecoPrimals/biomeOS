//! BiomeOS Test Utilities
//!
//! Shared test infrastructure for BiomeOS crates, including:
//! - Mock primal servers
//! - VM test harnesses
//! - Network test utilities
//! - Fixture management
//!
//! This crate is only for testing - it should never be used in production code.

#![warn(missing_docs)]
#![deny(unsafe_code)]

pub mod assertions;
pub mod fixtures;
pub mod mock_primal;

pub use fixtures::{create_test_config, create_test_manifest};
/// Re-export commonly used test utilities
pub use mock_primal::{MockPrimal, MockPrimalBuilder};
