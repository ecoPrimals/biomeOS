// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Platform-agnostic IPC transport layer.
//!
//! Provides [`TransportStream`] (client) and [`TransportListener`] (server) as
//! unified abstractions over Unix domain sockets, abstract sockets, and TCP.
//! Callers dispatch through [`connect_transport`] / [`bind_transport`] instead
//! of platform-gating with `#[cfg(unix)]` / `#[cfg(windows)]`.
//!
//! # Architecture
//!
//! ```text
//! TransportEndpoint (descriptor)
//!         │
//!    connect_transport()          bind_transport()
//!         │                            │
//!         ▼                            ▼
//!   TransportStream            TransportListener
//!   (AsyncRead+AsyncWrite)     (accept → TransportStream)
//! ```
//!
//! # Platform behavior
//!
//! - **Unix/macOS**: Uses UDS natively for `UnixSocket` and `AbstractSocket`.
//! - **Windows**: `UnixSocket` and `AbstractSocket` automatically fall back to
//!   TCP via a port-file convention (`{path}.port` contains the TCP port).
//! - **TCP**: Works everywhere.

mod stream;
pub use stream::TransportStream;

mod connect;
pub use connect::{connect_transport, connect_transport_timed};

mod listener;
pub use listener::TransportListener;

mod jsonrpc;
pub use jsonrpc::{send_jsonrpc_over_stream, send_jsonrpc_request};
