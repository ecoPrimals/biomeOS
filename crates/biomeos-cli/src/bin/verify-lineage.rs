// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Verify genetic lineage between spores using BearDog.
//!
//! Thin binary entry point that delegates to
//! [`biomeos_cli::commands::verify::run_verify_lineage`].

use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    biomeos_cli::commands::verify::run_verify_lineage().await
}
