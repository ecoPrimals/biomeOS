// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::or_fun_call,
    clippy::future_not_send,
    reason = "test assertions"
)]

use super::super::*;
use base64::Engine as _;
use base64::prelude::BASE64_STANDARD;
use hmac::{Hmac, Mac};
use sha2::Sha256;
use std::path::Path;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::{UnixListener, UnixStream};

type HmacSha256 = Hmac<Sha256>;

const TEST_FAMILY_SEED: &str = "test-family-seed";
const TEST_CHALLENGE_B64: &str = "dGVzdC1jaGFsbGVuZ2U=";

async fn run_consumer_btsp_server(stream: UnixStream) {
    let mut reader = BufReader::new(stream);
    let mut line = String::new();
    reader.read_line(&mut line).await.expect("read hello");

    let hello: ClientHello = serde_json::from_str(line.trim()).expect("parse ClientHello");
    assert_eq!(hello.protocol, "btsp");
    assert_eq!(hello.version, BTSP_VERSION);

    let server_hello = ServerHello {
        version: BTSP_VERSION,
        server_ephemeral_pub: "dGVzdC1zZXJ2ZXItcHVi".to_owned(),
        challenge: TEST_CHALLENGE_B64.to_owned(),
        session_id: "consumer-test-session".to_owned(),
    };
    let body = format!("{}\n", serde_json::to_string(&server_hello).unwrap());
    reader.get_mut().write_all(body.as_bytes()).await.unwrap();
    reader.get_mut().flush().await.unwrap();

    line.clear();
    reader
        .read_line(&mut line)
        .await
        .expect("read challenge response");

    let response: ChallengeResponse =
        serde_json::from_str(line.trim()).expect("parse ChallengeResponse");
    assert_eq!(response.preferred_cipher, "chacha20_poly1305");

    let challenge_bytes = BASE64_STANDARD.decode(TEST_CHALLENGE_B64).unwrap();
    let mut mac = HmacSha256::new_from_slice(TEST_FAMILY_SEED.as_bytes()).unwrap();
    mac.update(&challenge_bytes);
    let expected = BASE64_STANDARD.encode(mac.finalize().into_bytes());
    assert_eq!(response.response, expected);

    let complete = HandshakeComplete {
        cipher: "chacha20_poly1305".to_owned(),
        session_id: "consumer-test-session".to_owned(),
    };
    let body = format!("{}\n", serde_json::to_string(&complete).unwrap());
    reader.get_mut().write_all(body.as_bytes()).await.unwrap();
    reader.get_mut().flush().await.unwrap();
}

#[test]
fn btsp_strict_mode_default_off() {
    assert!(!btsp_strict_mode_expected());
}

#[test]
fn is_security_provider_socket_detects_beardog_filename() {
    assert!(is_security_provider_socket(Path::new(
        "/run/user/1000/biomeos/beardog.sock"
    )));
    assert!(is_security_provider_socket(Path::new(
        "/run/user/1000/biomeos/beardog-abc123.sock"
    )));
    assert!(!is_security_provider_socket(Path::new(
        "/run/user/1000/biomeos/neural-api.sock"
    )));
}

#[tokio::test]
async fn perform_consumer_handshake_completes_with_local_hmac() {
    let dir = tempfile::tempdir().expect("tempdir");
    let sock_path = dir.path().join("beardog.sock");
    let listener = UnixListener::bind(&sock_path).expect("bind listener");

    let server = tokio::spawn(async move {
        let (stream, _) = listener.accept().await.expect("accept");
        run_consumer_btsp_server(stream).await;
    });

    temp_env::async_with_vars([("FAMILY_SEED", Some(TEST_FAMILY_SEED))], async {
        let client = UnixStream::connect(&sock_path)
            .await
            .expect("connect client");
        let stream = perform_consumer_handshake(client)
            .await
            .expect("consumer handshake");
        drop(stream);
    })
    .await;

    server.await.expect("server task");
}

#[tokio::test]
async fn perform_consumer_handshake_fails_without_family_seed() {
    let (client, server) = UnixStream::pair().expect("pair");
    drop(server);

    temp_env::async_with_vars([("FAMILY_SEED", None::<&str>)], async {
        let err = perform_consumer_handshake(client)
            .await
            .expect_err("missing seed");
        assert!(matches!(err, BtspHandshakeError::NoFamilySeed));
    })
    .await;
}
