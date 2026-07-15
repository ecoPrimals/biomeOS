// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Discovery handler tests (split from `tests.rs`).
//!
//! Domain-focused modules:
//! - [`serialization`] — `DiscoveredPrimal` and `DiscoveredPrimalsResponse` JSON
//! - [`socket_dir`] — socket directory resolution and env overrides
//! - [`socket_probe`] — live socket probing and filename parsing
//! - [`handler_live`] — live discovery mode success, failure, and health conversion
//! - [`handler_modes`] — standalone mode and trust-level mapping

mod handler_live;
mod handler_modes;
mod serialization;
mod socket_dir;
mod socket_probe;
