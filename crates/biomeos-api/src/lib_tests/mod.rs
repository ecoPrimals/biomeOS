// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! API library tests split by domain:
//! - [`router`] — HTTP routing, health, CORS, and sovereign gate behavior
//! - [`websocket`] — events WebSocket JSON-RPC roundtrips
//! - [`jsonrpc_errors`] — JSON-RPC types, ApiError, and app construction
#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

mod common;
mod jsonrpc_errors;
mod router;
mod websocket;
