// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Tests for `SystemPaths` XDG path resolution (extracted from `paths.rs`).

#![expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]

use super::*;
use crate::primal_names;
use tempfile::tempdir;

mod errors;
mod resolution;
mod runtime;
mod xdg;
