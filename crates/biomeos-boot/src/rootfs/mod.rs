// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025-2026 ecoPrimals Project

//! Root filesystem builder for BiomeOS bootable images
//!
//! Creates qcow2 disk images with BiomeOS binaries, kernel modules,
//! and configuration for standalone boot from USB or network.

mod builder;
mod cli;
mod config;
mod dns;
mod nbd;

#[cfg(test)]
mod tests;

pub use builder::RootFsBuilder;
pub use cli::RootFsCli;
pub use config::RootFsConfig;
