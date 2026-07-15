// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project
//
// Test module for manifest/networking_services - included via #[path]

#![expect(clippy::unwrap_used, reason = "test")]

mod common;
mod complex_nested;
mod enum_variants;
mod http_traffic_specs;
mod spec_roundtrips;
