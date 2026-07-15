// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! WebSocket graph-event tests (split from `websocket_tests.rs`).
//!
//! Domain-focused modules:
//! - [`subscription_filter`] — `SubscriptionFilter` matching and serialization
//! - [`json_rpc`] — JSON-RPC request/response/error serialization
//! - [`handle_message`] — `GraphEventWebSocketServer::handle_message` and broadcast paths
//! - [`dispatch`] — `dispatch_ws_method` routing
#![expect(
    clippy::unwrap_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
#![expect(
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]

mod common;
mod dispatch;
mod handle_message;
mod json_rpc;
mod subscription_filter;
