// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Tower orchestration tests (split from `tower_orchestration_tests.rs`).

#![expect(clippy::unwrap_used, reason = "test")]

mod common;
mod format_capabilities;
mod list_active_sockets;
mod pid_file_io;
mod pid_file_path;
mod primal_config;
mod run_tower;
mod socket_dir_path;
mod stop_tower;
mod tower_status;
