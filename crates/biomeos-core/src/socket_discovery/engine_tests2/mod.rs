// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Continuation of socket discovery engine tests (split from `engine_tests2.rs`).
//! Covers manifest/registry discovery, XDG paths, and remaining discovery flows.

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

mod cache;
mod capability;
mod env_endpoint;
mod fallback;
mod family_tmp;
mod manifest;
mod registry;
mod strategy;
mod verification;
mod xdg;
