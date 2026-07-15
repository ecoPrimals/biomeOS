// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Dark Forest beacon unit tests (split from `beacon_tests.rs`).

#![expect(clippy::expect_used, reason = "test assertions")]

mod common;
mod decryption;
mod errors;
mod generation;
mod lineage;
mod pure_noise;
mod serde;
