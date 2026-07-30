// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Main binary integration tests (split into domain-focused submodules).

#[path = "main_tests_common.rs"]
mod common;

#[path = "main_tests_cli_core.rs"]
mod cli_core;

#[path = "main_tests_cli_runtime.rs"]
mod cli_runtime;

#[path = "main_tests_cli_nucleus.rs"]
mod cli_nucleus;

#[path = "main_tests_cli_subsystems.rs"]
mod cli_subsystems;

#[path = "main_tests_cli_errors.rs"]
mod cli_errors;

#[path = "main_tests_dispatch.rs"]
mod dispatch;
