// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! NUCLEUS Startup Mode
//!
//! Pure Rust replacement for `start_nucleus.sh`.
//! Discovers primals, starts them in dependency order, registers capabilities.
//!
//! ## Bootstrap Detection
//!
//! Before launching, the nucleus checks if an existing ecosystem is already
//! running. This determines the startup strategy:
//!
//! - **Bootstrap Mode**: No existing BearDog socket found. biomeOS acts as the
//!   genesis orchestrator, starts all primals from scratch, and creates the
//!   initial capability registry.
//!
//! - **Coordinated Mode**: An existing BearDog socket is detected and responds
//!   to health checks. biomeOS joins the existing ecosystem, potentially
//!   starting only supplementary primals (e.g., adding Toadstool to an existing
//!   Tower).

#![expect(
    unused_imports,
    reason = "facade re-exports consumed by tests and external callers"
)]

mod local;
mod remote;
mod types;

#[path = "../nucleus_launch.rs"]
mod nucleus_launch;
#[path = "../nucleus_procs.rs"]
mod nucleus_procs;

#[cfg(test)]
use biomeos_types::primal_names::{
    BARRACUDA, BEARDOG, CORALREEF, LOAMSPINE, NESTGATE, PETALTONGUE, RHIZOCRYPT, SKUNKBAT,
    SONGBIRD, SQUIRREL, SWEETGRASS, TOADSTOOL,
};

pub use local::{NucleusRunConfig, run};
pub use remote::{run_deploy, run_start, run_status, run_stop, run_undeploy};
pub use types::NucleusMode;

pub(crate) use local::format_nucleus_summary;
pub(crate) use remote::send_lifecycle_rpc;
pub(crate) use types::{
    EcosystemState, NucleusStatusSummary, PrimalCommandConfig, SporeDeployManifest,
    SporeDeploySpec, StartupConfig, build_primal_command, build_primal_command_with,
    parse_nucleus_status, parse_spore_deploy_manifest, resolve_lifecycle_socket,
    resolve_startup_config, resolve_startup_config_with, socket_path_for_capability,
};

#[cfg(test)]
use nucleus_procs::health_check;
use nucleus_procs::{
    DEFAULT_SOCKET_POLL_INTERVAL, cleanup_stale_sockets, detect_ecosystem, discover_binaries,
    start_primal, wait_for_socket,
};
use nucleus_procs::{
    auto_register_with_discovery_provider, generate_jwt_secret, health_check_with_backoff,
    resolve_socket_dir_with, wait_for_shutdown_signal,
};

#[cfg(test)]
pub(crate) use nucleus_procs::discover_binaries_with;
#[cfg(test)]
use nucleus_procs::discover_search_path;

#[cfg(test)]
#[path = "../nucleus_tests.rs"]
mod tests;

#[cfg(test)]
#[path = "../nucleus_tests2.rs"]
mod tests2;

#[cfg(test)]
#[path = "../nucleus_tests3.rs"]
mod tests3;

#[cfg(test)]
#[path = "../nucleus_tests4.rs"]
mod tests4;
