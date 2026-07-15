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
use super::{
    BtspServerConfig, NegotiateBehavior, Phase2Behavior, VALID_SHARED_SECRET_HEX,
    run_btsp_server, with_security_provider,
};
use tokio::net::UnixStream;

// ── perform_client_handshake_phase3 (integration) ──

#[tokio::test]
async fn perform_client_handshake_phase3_security_provider_not_found() {
    let dir = tempfile::tempdir().expect("tempdir");
    let iso = dir.path().to_str().unwrap();
    temp_env::async_with_vars(
        [
            ("BIOMEOS_SECURITY_SOCKET", None::<&str>),
            ("SECURITY_PROVIDER_SOCKET", None::<&str>),
            ("FAMILY_ID", None::<&str>),
            ("BIOMEOS_FAMILY_ID", None::<&str>),
            ("BIOMEOS_SOCKET_DIR", Some(iso)),
            ("XDG_RUNTIME_DIR", Some(iso)),
        ],
        async {
            let (client, _server) = UnixStream::pair().expect("pair");
            assert!(matches!(
                perform_client_handshake_phase3(client).await,
                Err(BtspHandshakeError::SecurityProviderNotFound)
            ));
        },
    )
    .await;
}

#[tokio::test]
async fn perform_client_handshake_phase3_establishes_encrypted_session() {
    with_security_provider(VALID_SHARED_SECRET_HEX, |_provider_path| async move {
        let (client, server) = UnixStream::pair().expect("pair");
        let server_nonce_hex = "aabbccddeeff00112233445566778899aabbccddeeff00112233445566778899";
        let server_task = tokio::spawn(async move {
            run_btsp_server(
                server,
                BtspServerConfig {
                    phase2: Phase2Behavior::Success {
                        session_id: "phase3-session".to_owned(),
                    },
                    negotiate: NegotiateBehavior::Encrypted {
                        server_nonce_hex: server_nonce_hex.to_owned(),
                    },
                },
            )
            .await;
        });

        let outcome = perform_client_handshake_phase3(client)
            .await
            .expect("handshake ok");
        match outcome {
            ClientPhase3Outcome::Encrypted { keys, stream: _ } => {
                assert_ne!(keys.client_to_server, [0u8; 32]);
                assert_ne!(keys.server_to_client, [0u8; 32]);
                assert_ne!(keys.client_to_server, keys.server_to_client);
            }
            ClientPhase3Outcome::Plaintext { .. } => panic!("expected encrypted outcome"),
        }
        server_task.await.expect("server");
    })
    .await;
}

#[tokio::test]
async fn perform_client_handshake_phase3_plaintext_when_shared_secret_not_hex_key() {
    with_security_provider(
        "not-valid-32-byte-hex-key-material!!!",
        |provider_path| async move {
            let (client, server) = UnixStream::pair().expect("pair");
            let _server_task = tokio::spawn(async move {
                run_btsp_server(
                    server,
                    BtspServerConfig {
                        phase2: Phase2Behavior::Success {
                            session_id: "plain-session".to_owned(),
                        },
                        negotiate: NegotiateBehavior::CloseWithoutResponse,
                    },
                )
                .await;
            });

            let outcome = perform_client_handshake_phase3(client)
                .await
                .expect("handshake ok");
            assert!(matches!(outcome, ClientPhase3Outcome::Plaintext { .. }));
            let _ = provider_path;
        },
    )
    .await;
}

#[tokio::test]
async fn perform_client_handshake_phase3_plaintext_when_negotiate_returns_null_cipher() {
    with_security_provider(VALID_SHARED_SECRET_HEX, |provider_path| async move {
        let (client, server) = UnixStream::pair().expect("pair");
        let server_task = tokio::spawn(async move {
            run_btsp_server(
                server,
                BtspServerConfig {
                    phase2: Phase2Behavior::Success {
                        session_id: "null-cipher".to_owned(),
                    },
                    negotiate: NegotiateBehavior::NullCipher,
                },
            )
            .await;
        });

        let outcome = perform_client_handshake_phase3(client)
            .await
            .expect("handshake ok");
        assert!(matches!(outcome, ClientPhase3Outcome::Plaintext { .. }));
        server_task.await.expect("server");
        let _ = provider_path;
    })
    .await;
}

#[tokio::test]
async fn perform_client_handshake_phase3_plaintext_when_negotiate_has_invalid_server_nonce() {
    with_security_provider(VALID_SHARED_SECRET_HEX, |provider_path| async move {
        let (client, server) = UnixStream::pair().expect("pair");
        let server_task = tokio::spawn(async move {
            run_btsp_server(
                server,
                BtspServerConfig {
                    phase2: Phase2Behavior::Success {
                        session_id: "bad-server-nonce".to_owned(),
                    },
                    negotiate: NegotiateBehavior::InvalidServerNonce,
                },
            )
            .await;
        });

        let outcome = perform_client_handshake_phase3(client)
            .await
            .expect("handshake ok");
        assert!(matches!(outcome, ClientPhase3Outcome::Plaintext { .. }));
        server_task.await.expect("server");
        let _ = provider_path;
    })
    .await;
}

#[tokio::test]
async fn perform_client_handshake_phase3_plaintext_when_negotiate_rejected() {
    with_security_provider(VALID_SHARED_SECRET_HEX, |provider_path| async move {
        let (client, server) = UnixStream::pair().expect("pair");
        let server_task = tokio::spawn(async move {
            run_btsp_server(
                server,
                BtspServerConfig {
                    phase2: Phase2Behavior::Success {
                        session_id: "reject-negotiate".to_owned(),
                    },
                    negotiate: NegotiateBehavior::JsonRpcError {
                        message: "phase 3 unavailable".to_owned(),
                    },
                },
            )
            .await;
        });

        let outcome = perform_client_handshake_phase3(client)
            .await
            .expect("handshake ok");
        assert!(matches!(outcome, ClientPhase3Outcome::Plaintext { .. }));
        server_task.await.expect("server");
        let _ = provider_path;
    })
    .await;
}

#[tokio::test]
async fn perform_client_handshake_phase3_plaintext_on_malformed_negotiate_response() {
    with_security_provider(VALID_SHARED_SECRET_HEX, |provider_path| async move {
        let (client, server) = UnixStream::pair().expect("pair");
        let server_task = tokio::spawn(async move {
            run_btsp_server(
                server,
                BtspServerConfig {
                    phase2: Phase2Behavior::Success {
                        session_id: "bad-negotiate-json".to_owned(),
                    },
                    negotiate: NegotiateBehavior::MalformedJson,
                },
            )
            .await;
        });

        let outcome = perform_client_handshake_phase3(client)
            .await
            .expect("handshake ok");
        assert!(matches!(outcome, ClientPhase3Outcome::Plaintext { .. }));
        server_task.await.expect("server");
        let _ = provider_path;
    })
    .await;
}

#[tokio::test]
async fn perform_client_handshake_phase3_rejects_phase2_handshake_failure() {
    with_security_provider(VALID_SHARED_SECRET_HEX, |provider_path| async move {
        let (client, server) = UnixStream::pair().expect("pair");
        let server_task = tokio::spawn(async move {
            run_btsp_server(
                server,
                BtspServerConfig {
                    phase2: Phase2Behavior::Reject {
                        reason: "family_verification".to_owned(),
                    },
                    negotiate: NegotiateBehavior::CloseWithoutResponse,
                },
            )
            .await;
        });

        assert!(matches!(
            perform_client_handshake_phase3(client).await,
            Err(BtspHandshakeError::Protocol(msg)) if msg.contains("family_verification")
        ));
        server_task.await.expect("server");
        let _ = provider_path;
    })
    .await;
}

#[tokio::test]
async fn perform_client_handshake_phase3_connection_closed_during_phase2() {
    with_security_provider(VALID_SHARED_SECRET_HEX, |provider_path| async move {
        let (client, server) = UnixStream::pair().expect("pair");
        let server_task = tokio::spawn(async move {
            run_btsp_server(
                server,
                BtspServerConfig {
                    phase2: Phase2Behavior::CloseAfterHello,
                    negotiate: NegotiateBehavior::CloseWithoutResponse,
                },
            )
            .await;
        });

        assert!(matches!(
            perform_client_handshake_phase3(client).await,
            Err(BtspHandshakeError::ConnectionClosed | BtspHandshakeError::Timeout)
        ));
        server_task.await.expect("server");
        let _ = provider_path;
    })
    .await;
}

#[tokio::test]
async fn client_phase3_outcome_variants_are_constructible() {
    let (s1, _s2) = UnixStream::pair().expect("pair");
    let keys = crate::btsp_crypto::derive_session_keys(&[1u8; 32], &[2u8; 8], &[3u8; 8]);
    let encrypted = ClientPhase3Outcome::Encrypted { keys, stream: s1 };
    let (s3, _s4) = UnixStream::pair().expect("pair");
    let plaintext = ClientPhase3Outcome::Plaintext { stream: s3 };
    assert!(matches!(encrypted, ClientPhase3Outcome::Encrypted { .. }));
    assert!(matches!(plaintext, ClientPhase3Outcome::Plaintext { .. }));
}
