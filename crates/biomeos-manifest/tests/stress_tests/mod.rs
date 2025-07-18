//! Stress tests for biomeOS recursive BYOB functionality
//!
//! This test suite validates system behavior under extreme loads and
//! stress conditions with focus on Songbird and NestGate eco-primals.

pub mod common;
pub mod tournament_tests;
pub mod biome_tests;
pub mod scaling_tests;
pub mod replication_tests;

// Re-export common types and functions for easy access
pub use common::*;

// Re-export test modules for external access
pub use tournament_tests::*;
pub use biome_tests::*;
pub use scaling_tests::*;
pub use replication_tests::*; 