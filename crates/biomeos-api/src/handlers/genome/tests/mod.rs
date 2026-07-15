// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Genome handler tests

#![expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
#![expect(clippy::expect_used, reason = "test assertions use expect for clarity")]

mod build_handlers;
mod retrieval;
mod serialization;
mod state;
mod validation;
