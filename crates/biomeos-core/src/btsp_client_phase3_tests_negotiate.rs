// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

#![expect(clippy::unwrap_used, clippy::expect_used, reason = "test assertions")]

use super::super::*;
use super::handshake_key_from_hex;
use base64::Engine;
use serde_json::json;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;

// ── client_negotiate ──

#[tokio::test]
async fn client_negotiate_derives_session_keys_on_success() {
    let (client, server) = UnixStream::pair().expect("pair");
    let server_nonce_hex = "0a0b0c0d0e0f101112131415161718191a1b1c1d1e1f2021222324252627";
    tokio::spawn(async move {
        let mut reader = BufReader::new(server);
        let mut line = String::new();
        reader.read_line(&mut line).await.expect("read negotiate");
        let req: serde_json::Value = serde_json::from_str(line.trim()).unwrap();
        assert_eq!(req["method"], "btsp.negotiate");
        assert_eq!(req["params"]["preferred_cipher"], "chacha20-poly1305");
        assert_eq!(req["params"]["session_id"], "sess-123");
        assert!(req["params"]["client_nonce"].as_str().is_some());
        let resp = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "cipher": "chacha20-poly1305",
                "server_nonce": server_nonce_hex,
            },
        });
        let mut stream = reader.into_inner();
        stream
            .write_all(format!("{}\n", serde_json::to_string(&resp).unwrap()).as_bytes())
            .await
            .unwrap();
    });

    let mut reader = BufReader::new(client);
    let keys = client_negotiate(&mut reader, "sess-123", &handshake_key_from_hex())
        .await
        .expect("negotiate ok");
    assert_ne!(keys.client_to_server, [0u8; 32]);
    assert_ne!(keys.server_to_client, [0u8; 32]);
    assert_ne!(keys.client_to_server, keys.server_to_client);
}

#[tokio::test]
async fn client_negotiate_accepts_base64_server_nonce() {
    let (client, server) = UnixStream::pair().expect("pair");
    let server_nonce = vec![9u8; 16];
    let server_nonce_b64 = base64::engine::general_purpose::STANDARD.encode(&server_nonce);
    tokio::spawn(async move {
        let mut reader = BufReader::new(server);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let resp = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "cipher": "ChaCha20-Poly1305",
                "server_nonce": server_nonce_b64,
            },
        });
        let mut stream = reader.into_inner();
        stream
            .write_all(format!("{}\n", serde_json::to_string(&resp).unwrap()).as_bytes())
            .await
            .unwrap();
    });

    let mut reader = BufReader::new(client);
    let keys = client_negotiate(&mut reader, "sid", &handshake_key_from_hex())
        .await
        .expect("base64 server nonce");
    assert_ne!(keys.client_to_server, keys.server_to_client);
}

#[tokio::test]
async fn client_negotiate_rejects_null_cipher() {
    let (client, server) = UnixStream::pair().expect("pair");
    tokio::spawn(async move {
        let mut reader = BufReader::new(server);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let resp = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": { "cipher": "null", "server_nonce": "0102" },
        });
        let mut stream = reader.into_inner();
        stream
            .write_all(format!("{}\n", serde_json::to_string(&resp).unwrap()).as_bytes())
            .await
            .unwrap();
    });

    let mut reader = BufReader::new(client);
    let err = client_negotiate(&mut reader, "sid", &handshake_key_from_hex())
        .await
        .expect_err("null cipher");
    assert!(matches!(err, BtspHandshakeError::Protocol(msg) if msg.contains("null cipher")));
}

#[tokio::test]
async fn client_negotiate_rejects_jsonrpc_error() {
    let (client, server) = UnixStream::pair().expect("pair");
    tokio::spawn(async move {
        let mut reader = BufReader::new(server);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let resp = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "error": { "message": "negotiate not supported" },
        });
        let mut stream = reader.into_inner();
        stream
            .write_all(format!("{}\n", serde_json::to_string(&resp).unwrap()).as_bytes())
            .await
            .unwrap();
    });

    let mut reader = BufReader::new(client);
    let err = client_negotiate(&mut reader, "sid", &handshake_key_from_hex())
        .await
        .expect_err("jsonrpc error");
    assert!(matches!(err, BtspHandshakeError::Protocol(msg) if msg.contains("negotiate rejected")));
}

#[tokio::test]
async fn client_negotiate_rejects_malformed_json() {
    let (client, server) = UnixStream::pair().expect("pair");
    tokio::spawn(async move {
        let mut reader = BufReader::new(server);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let mut stream = reader.into_inner();
        stream.write_all(b"{broken\n").await.unwrap();
    });

    let mut reader = BufReader::new(client);
    let err = client_negotiate(&mut reader, "sid", &handshake_key_from_hex())
        .await
        .expect_err("malformed json");
    assert!(matches!(err, BtspHandshakeError::Protocol(msg) if msg.contains("parse negotiate")));
}

#[tokio::test]
async fn client_negotiate_connection_closed_returns_error() {
    let (client, server) = UnixStream::pair().expect("pair");
    tokio::spawn(async move {
        let mut reader = BufReader::new(server);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        drop(reader);
    });

    let mut reader = BufReader::new(client);
    let err = client_negotiate(&mut reader, "sid", &handshake_key_from_hex())
        .await
        .expect_err("connection closed");
    assert!(matches!(err, BtspHandshakeError::ConnectionClosed));
}

#[tokio::test]
async fn client_negotiate_missing_server_nonce_is_protocol_error() {
    let (client, server) = UnixStream::pair().expect("pair");
    tokio::spawn(async move {
        let mut reader = BufReader::new(server);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let resp = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": { "cipher": "chacha20-poly1305" },
        });
        let mut stream = reader.into_inner();
        stream
            .write_all(format!("{}\n", serde_json::to_string(&resp).unwrap()).as_bytes())
            .await
            .unwrap();
    });

    let mut reader = BufReader::new(client);
    let err = client_negotiate(&mut reader, "sid", &handshake_key_from_hex())
        .await
        .expect_err("missing server_nonce");
    assert!(
        matches!(err, BtspHandshakeError::Protocol(msg) if msg.contains("missing server_nonce"))
    );
}

#[tokio::test]
async fn client_negotiate_invalid_server_nonce_encoding() {
    let (client, server) = UnixStream::pair().expect("pair");
    tokio::spawn(async move {
        let mut reader = BufReader::new(server);
        let mut line = String::new();
        reader.read_line(&mut line).await.unwrap();
        let resp = json!({
            "jsonrpc": "2.0",
            "id": 1,
            "result": {
                "cipher": "chacha20-poly1305",
                "server_nonce": "!!!",
            },
        });
        let mut stream = reader.into_inner();
        stream
            .write_all(format!("{}\n", serde_json::to_string(&resp).unwrap()).as_bytes())
            .await
            .unwrap();
    });

    let mut reader = BufReader::new(client);
    let err = client_negotiate(&mut reader, "sid", &handshake_key_from_hex())
        .await
        .expect_err("bad nonce");
    assert!(
        matches!(err, BtspHandshakeError::Protocol(msg) if msg.contains("decode server_nonce"))
    );
}
