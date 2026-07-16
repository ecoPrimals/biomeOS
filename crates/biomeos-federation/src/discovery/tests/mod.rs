// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Discovery tests - extracted to keep discovery/mod.rs under 1000 lines

#![expect(clippy::expect_used, reason = "test assertions")]

mod endpoint_parsing;
mod primal_types;
mod peer_registration;
mod discovery_async;
