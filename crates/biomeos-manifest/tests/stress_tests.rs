//! Stress tests for biomeOS recursive BYOB functionality
//!
//! This test suite validates system behavior under extreme loads and
//! stress conditions with focus on Songbird and NestGate eco-primals.

mod stress_tests;

// Re-export all tests from the stress_tests module
pub use stress_tests::*; 