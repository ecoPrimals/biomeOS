// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Integration tests for [`super::forwarding`] routing: `should_use_tarpc`,
//! `forward_request`, `forward_via_tarpc`, and `primal_label_for_endpoint`.

#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

mod common;
mod endpoint_label;
mod forward_request;
mod forward_via_tarpc;
mod should_use_tarpc;
