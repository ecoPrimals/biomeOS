//! Nucleus mode tests (split into domain-focused submodules).

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

#[path = "nucleus_tests_config.rs"]
mod config;

#[path = "nucleus_tests_startup.rs"]
mod startup;

#[path = "nucleus_tests_lifecycle.rs"]
mod lifecycle;
