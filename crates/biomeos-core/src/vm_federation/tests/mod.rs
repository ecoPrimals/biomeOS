// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! VM federation unit tests (split from `tests.rs`).

#![expect(clippy::unwrap_used, reason = "test")]

mod benchscale_argv;
mod collect_ips;
mod lifecycle;
mod manager;
mod parse_ip;
mod parse_vm_names;
mod ssh_validation;
mod topology;
mod validation_config;

use crate::vm_federation::{
    ValidationConfig, VmFederationManager, benchscale_create_argv, benchscale_subcommand_argv,
    collect_ips_for_vm_names, parse_ip_from_domifaddr_output, parse_vm_names_from_list,
    topology_path_for_cli, validate_ssh_probe_output, wait_for_vm_ssh_ready,
};
