// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]
#![expect(clippy::expect_used, reason = "test assertions use expect for clarity")]

mod debug;
mod execute;
mod handlers;
mod helpers;
