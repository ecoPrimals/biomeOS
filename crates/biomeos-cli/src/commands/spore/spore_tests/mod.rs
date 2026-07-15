// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Spore command tests split by domain:
//! - [`unit`] — parsing, structure inspection, refresh planning, and formatting
//! - [`handlers`] — async command handlers (list, verify, info, clone, create)
//! - [`refresh`] — plasmid discovery and nucleus refresh integration
#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

mod handlers;
mod refresh;
mod unit;
