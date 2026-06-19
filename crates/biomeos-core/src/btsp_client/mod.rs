// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! BTSP Client — biomeOS-side handshake for Secure Socket Architecture.
//!
//! When biomeOS connects to a family-scoped primal socket (`{primal}-{fid}.sock`),
//! it MUST perform a BTSP handshake to prove family membership before sending
//! any JSON-RPC requests.
//!
//! This module provides:
//! - Detection of family-scoped sockets (BTSP-required vs development-mode)
//! - BTSP session state tracking
//! - The INSECURE guard (refuse to run with both `FAMILY_ID` and `BIOMEOS_INSECURE`)
//! - Phase 2 server-side handshake enforcement for UDS listeners
//! - Phase 2 client-side handshake initiation for outbound forwarding
//! - Phase 3 client-side negotiate + encrypted framing for outbound calls
//!
//! The actual cryptographic handshake is delegated to the security provider
//! via JSON-RPC (`btsp.session.create`, `btsp.session.verify`). biomeOS is a
//! family member and holds the family seed for key derivation.

mod client;
mod config;
mod provider;
mod server;
mod types;

pub use client::perform_client_handshake;
pub(crate) use client::{client_keygen, read_json_line, serialize_line, write_line_to};
pub use config::{
    btsp_enforce, extract_family_id, family_id, has_family_id, is_family_scoped_socket,
    log_security_posture, security_mode, security_provider_socket_path, validate_insecure_guard,
};
pub use server::server_handshake;
pub use types::{
    BTSP_VERSION, BtspHandshakeError, ChallengeResponse, ClientHello, HandshakeComplete,
    HandshakeError, HandshakeOutcome, SecurityMode, ServerHello,
};

#[cfg(test)]
pub(crate) use provider::{
    create_session_via_security_provider, verify_session_via_security_provider,
};

#[cfg(test)]
#[path = "../btsp_client_tests.rs"]
mod tests;
