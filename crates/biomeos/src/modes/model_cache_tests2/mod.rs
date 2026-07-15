// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Continuation of model-cache CLI tests (split from `model_cache_tests.rs`).

#![expect(clippy::expect_used, reason = "test assertions use expect for clarity")]

mod cli_errors;
mod common;
mod hf_import;
mod output;
mod production;
mod register_resolve;
