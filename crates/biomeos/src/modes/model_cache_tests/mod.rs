// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Model-cache CLI tests (part 1 of 2; see `model_cache_tests2/`).

#![expect(clippy::expect_used, reason = "test assertions use expect for clarity")]

use super::{format_size_gb, format_size_mb, hf_dir_to_model_id, run_with, ModelCacheCommand};

mod commands;
mod formatting;
