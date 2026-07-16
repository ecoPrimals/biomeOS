// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::or_fun_call,
    reason = "test assertions use unwrap/expect for clarity"
)]

mod extract_edge_cases;
mod extract_formats;
mod socket_probe;
