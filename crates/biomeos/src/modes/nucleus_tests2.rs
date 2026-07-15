//! Continuation of nucleus mode tests (split from `nucleus_tests.rs`).

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

#[path = "nucleus_tests2_config.rs"]
mod config;

#[path = "nucleus_tests2_primal_command.rs"]
mod primal_command;

#[path = "nucleus_tests2_ecosystem.rs"]
mod ecosystem;

#[path = "nucleus_tests2_discovery.rs"]
mod discovery;
