// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Unit tests for real-time event streaming (WebSocket/SSE).

#![expect(clippy::unwrap_used, reason = "test assertions use unwrap for clarity")]

mod events;
mod handler_errors;
mod handler_loops;
mod parse_extended;
mod sse_parse;
mod subscriber;
mod transport_discovery;
