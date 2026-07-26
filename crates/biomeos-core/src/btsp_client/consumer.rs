// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Consumer-side BTSP handshake for connecting to bearDog in strict mode.
//!
//! When `BEARDOG_UDS_REQUIRE_BTSP=1` is set, bearDog rejects plain JSON-RPC
//! with `-32600`. This module implements the consumer-side of the 4-step BTSP
//! handshake using LOCAL HMAC-SHA256 with the family seed — avoiding the
//! chicken-and-egg of delegating crypto to bearDog for the handshake that
//! authenticates us *to* bearDog.

use base64::Engine as _;
use base64::prelude::BASE64_STANDARD;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use tokio::io::BufReader;
use tracing::{debug, warn};

use super::types::{
    BTSP_VERSION, BtspConnection, BtspHandshakeError, ChallengeResponse, ClientHello,
    HandshakeComplete, ServerHello,
};
#[cfg(unix)]
use super::{read_json_line, serialize_line, write_line_to};

type HmacSha256 = Hmac<Sha256>;

const PREFERRED_CIPHER: &str = "chacha20_poly1305";

/// Resolve the raw family seed from environment.
fn resolve_family_seed_raw() -> Option<String> {
    std::env::var(biomeos_types::env_config::vars::FAMILY_SEED_LEGACY)
        .or_else(|_| std::env::var(biomeos_types::env_config::vars::FAMILY_SEED))
        .or_else(|_| {
            std::env::var("BEARDOG_FAMILY_SEED").inspect(|_| {
                warn!("BEARDOG_FAMILY_SEED is deprecated — use FAMILY_SEED");
            })
        })
        .ok()
        .filter(|s| !s.trim().is_empty())
}

/// Perform the consumer-side BTSP handshake over an NDJSON stream.
///
/// Authenticates to bearDog using the family seed from environment.
/// After success, the stream is ready for JSON-RPC traffic.
///
/// # Errors
///
/// Returns [`BtspHandshakeError`] if the family seed is unavailable, the server
/// rejects the handshake, or I/O fails.
#[cfg(unix)]
pub async fn perform_consumer_handshake(
    stream: BtspConnection,
) -> Result<BtspConnection, BtspHandshakeError> {
    let family_seed = resolve_family_seed_raw().ok_or(BtspHandshakeError::NoFamilySeed)?;

    let mut ephemeral_key = [0u8; 32];
    rand::Rng::fill(&mut rand::rng(), &mut ephemeral_key);

    let hello = ClientHello {
        protocol: "btsp".into(),
        version: BTSP_VERSION,
        client_ephemeral_pub: BASE64_STANDARD.encode(ephemeral_key),
    };
    let hello_line = serialize_line(&hello)?;

    let mut reader = BufReader::new(stream);
    write_line_to(&mut reader, &hello_line).await?;

    debug!("BTSP consumer: sent ClientHello");

    let server_hello: ServerHello = read_json_line(&mut reader).await?;

    debug!(
        session_id = %server_hello.session_id,
        "BTSP consumer: received ServerHello"
    );

    let challenge_bytes = BASE64_STANDARD
        .decode(&server_hello.challenge)
        .map_err(|e| BtspHandshakeError::Protocol(format!("decode challenge: {e}")))?;

    let mut mac = HmacSha256::new_from_slice(family_seed.trim().as_bytes())
        .map_err(|_| BtspHandshakeError::Hmac)?;
    mac.update(&challenge_bytes);
    let hmac_result = mac.finalize().into_bytes();

    let response = ChallengeResponse {
        response: BASE64_STANDARD.encode(hmac_result),
        preferred_cipher: PREFERRED_CIPHER.into(),
    };
    let resp_line = serialize_line(&response)?;
    write_line_to(&mut reader, &resp_line).await?;

    debug!("BTSP consumer: sent ChallengeResponse");

    let complete: HandshakeComplete = read_json_line(&mut reader).await?;

    debug!(
        session_id = %complete.session_id,
        cipher = %complete.cipher,
        "BTSP consumer: handshake COMPLETE"
    );

    Ok(reader.into_inner())
}

/// Windows stub — Unix domain sockets unavailable on this platform.
#[cfg(windows)]
pub async fn perform_consumer_handshake(
    _stream: BtspConnection,
) -> Result<BtspConnection, BtspHandshakeError> {
    Err(BtspHandshakeError::Protocol(
        "Unix domain sockets unavailable on Windows".into(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[expect(clippy::expect_used, reason = "test assertion")]
    fn hmac_computation_produces_32_bytes() {
        let key = b"test-family-seed";
        let challenge = b"random-challenge-data";
        let mut mac = HmacSha256::new_from_slice(key).expect("valid HMAC key");
        mac.update(challenge);
        let result = mac.finalize().into_bytes();
        assert_eq!(result.len(), 32);
    }

    #[test]
    #[expect(clippy::expect_used, reason = "test assertion")]
    fn client_hello_serializes_correctly() {
        let hello = ClientHello {
            protocol: "btsp".into(),
            version: 1,
            client_ephemeral_pub: String::from("AAAA"),
        };
        let json = serde_json::to_string(&hello).expect("serialize ClientHello");
        assert!(json.contains("\"protocol\":\"btsp\""));
        assert!(json.contains("\"version\":1"));
    }
}
