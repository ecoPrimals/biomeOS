// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! genomeBin Deployment Library
//!
//! Rust implementation of genomeBin deployment, replacing POSIX shell wrappers
//! with type-safe, cross-platform deployment infrastructure.
//!
//! ## Safe I/O Architecture
//!
//! Uses `std::fs::read()` for genomeBin extraction — 100% safe Rust with
//! zero `unsafe` blocks. For a one-shot deployment tool, the allocation
//! cost of reading the file is negligible compared to the disk I/O and
//! extraction time. This allows `#![forbid(unsafe_code)]` across the entire crate.

#![warn(missing_docs)]
#![forbid(unsafe_code)]

mod deployer;
mod types;

pub use deployer::GenomeDeployer;
pub use types::{Architecture, GenomeMetadata, Platform};

#[cfg(test)]
#[expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#[expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests;
