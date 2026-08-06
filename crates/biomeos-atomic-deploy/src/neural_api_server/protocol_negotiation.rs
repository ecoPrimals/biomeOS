// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! G65 Protocol Negotiation
//!
//! Enables automatic protocol selection between JSON-RPC and tarpc at
//! connection time on a single socket. Implements the G65 standard
//! independently (no shared crate — primal violation per Wave 156m).
//!
//! ## Wire Protocol
//!
//! ```text
//! Client → Server: "PROTOCOLS: tarpc,jsonrpc\n"
//! Server → Client: "PROTOCOL: tarpc\n"
//! [Connection proceeds with selected protocol]
//! ```
//!
//! ## Backward Compatibility
//!
//! If the first bytes are NOT a `PROTOCOLS:` header, the server assumes
//! JSON-RPC (default protocol). This preserves compatibility with all
//! existing clients that don't negotiate.

#![forbid(unsafe_code)]

use anyhow::{Context, Result};
use tokio::io::{AsyncBufReadExt, AsyncWrite, AsyncWriteExt, BufReader};
use tracing::{debug, info};

/// Protocols supported by biomeOS's Neural API server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NegotiatedProtocol {
    JsonRpc,
    Tarpc,
}

impl NegotiatedProtocol {
    fn from_name(s: &str) -> Option<Self> {
        match s.trim().to_lowercase().as_str() {
            "jsonrpc" | "json-rpc" | "json_rpc" => Some(Self::JsonRpc),
            "tarpc" => Some(Self::Tarpc),
            _ => None,
        }
    }

    const fn wire_name(self) -> &'static str {
        match self {
            Self::JsonRpc => "jsonrpc",
            Self::Tarpc => "tarpc",
        }
    }
}

/// Server-side protocol negotiation result.
#[derive(Debug)]
pub enum NegotiationOutcome {
    /// Client sent `PROTOCOLS:` header and we selected a protocol.
    Negotiated(NegotiatedProtocol),
    /// First line was NOT a protocol negotiation header — it's a regular
    /// request line that should be dispatched as JSON-RPC. The line is
    /// returned so the caller can process it without re-reading.
    NotNegotiation(String),
    /// Connection closed or empty read before any data arrived.
    Closed,
}

/// Server-supported protocols in preference order.
///
/// tarpc is preferred when the client supports it (higher throughput,
/// lower latency for binary payloads).
const SERVER_PROTOCOLS: &[NegotiatedProtocol] =
    &[NegotiatedProtocol::Tarpc, NegotiatedProtocol::JsonRpc];

/// Attempt G65 protocol negotiation on an incoming connection.
///
/// Reads the first line from the buffered reader. If it starts with
/// `PROTOCOLS: `, parses the client's supported protocols, selects the
/// best match, writes the `PROTOCOL: <selected>\n` response, and returns
/// the negotiated protocol.
///
/// If the first line is NOT a negotiation header, returns it as
/// [`NegotiationOutcome::NotNegotiation`] so the caller can dispatch it
/// as a normal JSON-RPC request without data loss.
pub async fn negotiate_server<S>(reader: &mut BufReader<S>) -> Result<NegotiationOutcome>
where
    S: tokio::io::AsyncRead + AsyncWrite + Unpin,
{
    let mut first_line = String::new();

    let read_result = tokio::time::timeout(
        biomeos_types::constants::timeouts::POLL_INTERVAL_FAST,
        reader.read_line(&mut first_line),
    )
    .await;

    match read_result {
        Ok(Ok(0) | Err(_)) | Err(_) => {
            return Ok(NegotiationOutcome::Closed);
        }
        Ok(Ok(_)) => {}
    }

    let trimmed = first_line.trim();

    let Some(protocols_str) = trimmed.strip_prefix("PROTOCOLS: ") else {
        debug!("No G65 negotiation header — fallback to JSON-RPC");
        return Ok(NegotiationOutcome::NotNegotiation(first_line));
    };

    let client_supported: Vec<NegotiatedProtocol> = protocols_str
        .split(',')
        .filter_map(NegotiatedProtocol::from_name)
        .collect();

    if client_supported.is_empty() {
        debug!("G65: no recognized protocols in client request, defaulting to JSON-RPC");
        let response = format!("PROTOCOL: {}\n", NegotiatedProtocol::JsonRpc.wire_name());
        reader
            .get_mut()
            .write_all(response.as_bytes())
            .await
            .context("Failed to write G65 response")?;
        reader
            .get_mut()
            .flush()
            .await
            .context("Failed to flush G65 response")?;
        return Ok(NegotiationOutcome::Negotiated(NegotiatedProtocol::JsonRpc));
    }

    let selected = select_protocol(&client_supported);

    let response = format!("PROTOCOL: {}\n", selected.wire_name());
    reader
        .get_mut()
        .write_all(response.as_bytes())
        .await
        .context("Failed to write G65 response")?;
    reader
        .get_mut()
        .flush()
        .await
        .context("Failed to flush G65 response")?;

    info!(protocol = selected.wire_name(), "G65 protocol negotiated");
    Ok(NegotiationOutcome::Negotiated(selected))
}

/// Select the best protocol: first client protocol that the server supports.
fn select_protocol(client_supported: &[NegotiatedProtocol]) -> NegotiatedProtocol {
    for client_proto in client_supported {
        if SERVER_PROTOCOLS.contains(client_proto) {
            return *client_proto;
        }
    }
    NegotiatedProtocol::JsonRpc
}

/// Client-side protocol negotiation helper.
///
/// Sends `PROTOCOLS: tarpc,jsonrpc\n` and reads back the server's selection.
/// Used by Neural API when connecting to other primals that support G65.
#[allow(
    dead_code,
    reason = "client-side helper for future outbound G65 connections"
)]
pub async fn negotiate_client<S>(
    stream: &mut S,
    preferred: &[NegotiatedProtocol],
) -> Result<NegotiatedProtocol>
where
    S: tokio::io::AsyncRead + AsyncWrite + Unpin,
{
    let names: Vec<&str> = preferred.iter().map(|p| p.wire_name()).collect();
    let request = format!("PROTOCOLS: {}\n", names.join(","));

    stream
        .write_all(request.as_bytes())
        .await
        .context("Failed to send G65 protocol request")?;
    stream
        .flush()
        .await
        .context("Failed to flush G65 request")?;

    let mut reader = BufReader::new(stream);
    let mut response_line = String::new();
    reader
        .read_line(&mut response_line)
        .await
        .context("Failed to read G65 protocol response")?;

    let trimmed = response_line.trim();
    let proto_name = trimmed
        .strip_prefix("PROTOCOL: ")
        .ok_or_else(|| anyhow::anyhow!("Invalid G65 response: {trimmed}"))?;

    NegotiatedProtocol::from_name(proto_name)
        .ok_or_else(|| anyhow::anyhow!("Unknown protocol in G65 response: {proto_name}"))
}

#[cfg(test)]
#[allow(
    clippy::unwrap_used,
    clippy::expect_used,
    reason = "test assertions use unwrap/expect for clarity"
)]
mod tests {
    use super::*;
    use tokio::io::duplex;

    #[test]
    fn test_negotiated_protocol_from_name() {
        assert_eq!(
            NegotiatedProtocol::from_name("jsonrpc"),
            Some(NegotiatedProtocol::JsonRpc)
        );
        assert_eq!(
            NegotiatedProtocol::from_name("tarpc"),
            Some(NegotiatedProtocol::Tarpc)
        );
        assert_eq!(
            NegotiatedProtocol::from_name("JSON-RPC"),
            Some(NegotiatedProtocol::JsonRpc)
        );
        assert_eq!(NegotiatedProtocol::from_name("unknown"), None);
    }

    #[test]
    fn test_select_protocol_tarpc_preferred() {
        let client = &[NegotiatedProtocol::Tarpc, NegotiatedProtocol::JsonRpc];
        assert_eq!(select_protocol(client), NegotiatedProtocol::Tarpc);
    }

    #[test]
    fn test_select_protocol_jsonrpc_only() {
        let client = &[NegotiatedProtocol::JsonRpc];
        assert_eq!(select_protocol(client), NegotiatedProtocol::JsonRpc);
    }

    #[test]
    fn test_select_protocol_empty_fallback() {
        let client: &[NegotiatedProtocol] = &[];
        assert_eq!(select_protocol(client), NegotiatedProtocol::JsonRpc);
    }

    #[tokio::test]
    async fn test_negotiate_server_with_protocol_header() {
        let (mut client, server) = duplex(4096);
        let mut server_reader = BufReader::new(server);

        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            client
                .write_all(b"PROTOCOLS: tarpc,jsonrpc\n")
                .await
                .unwrap();
            client.flush().await.unwrap();

            let mut response = String::new();
            let mut reader = BufReader::new(&mut client);
            tokio::io::AsyncBufReadExt::read_line(&mut reader, &mut response)
                .await
                .unwrap();
            assert_eq!(response.trim(), "PROTOCOL: tarpc");
        });

        let outcome = negotiate_server(&mut server_reader).await.unwrap();
        match outcome {
            NegotiationOutcome::Negotiated(proto) => {
                assert_eq!(proto, NegotiatedProtocol::Tarpc);
            }
            _ => panic!("Expected Negotiated outcome"),
        }
    }

    #[tokio::test]
    async fn test_negotiate_server_without_header_returns_line() {
        let (mut client, server) = duplex(4096);
        let mut server_reader = BufReader::new(server);

        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            client
                .write_all(b"{\"jsonrpc\":\"2.0\",\"method\":\"health.check\",\"id\":1}\n")
                .await
                .unwrap();
            client.flush().await.unwrap();
        });

        let outcome = negotiate_server(&mut server_reader).await.unwrap();
        match outcome {
            NegotiationOutcome::NotNegotiation(line) => {
                assert!(line.contains("health.check"));
            }
            _ => panic!("Expected NotNegotiation outcome"),
        }
    }

    #[tokio::test]
    async fn test_negotiate_server_jsonrpc_only_client() {
        let (mut client, server) = duplex(4096);
        let mut server_reader = BufReader::new(server);

        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            client.write_all(b"PROTOCOLS: jsonrpc\n").await.unwrap();
            client.flush().await.unwrap();

            let mut response = String::new();
            let mut reader = BufReader::new(&mut client);
            tokio::io::AsyncBufReadExt::read_line(&mut reader, &mut response)
                .await
                .unwrap();
            assert_eq!(response.trim(), "PROTOCOL: jsonrpc");
        });

        let outcome = negotiate_server(&mut server_reader).await.unwrap();
        match outcome {
            NegotiationOutcome::Negotiated(proto) => {
                assert_eq!(proto, NegotiatedProtocol::JsonRpc);
            }
            _ => panic!("Expected Negotiated outcome"),
        }
    }

    #[tokio::test]
    async fn test_negotiate_client_server_duplex() {
        let (mut client_stream, mut server_stream) = duplex(4096);

        let server_task = tokio::spawn(async move {
            let mut reader = BufReader::new(&mut server_stream);
            negotiate_server(&mut reader).await
        });

        let client_result = negotiate_client(
            &mut client_stream,
            &[NegotiatedProtocol::Tarpc, NegotiatedProtocol::JsonRpc],
        )
        .await
        .unwrap();

        assert_eq!(client_result, NegotiatedProtocol::Tarpc);

        let server_result = server_task.await.unwrap().unwrap();
        match server_result {
            NegotiationOutcome::Negotiated(proto) => {
                assert_eq!(proto, NegotiatedProtocol::Tarpc);
            }
            _ => panic!("Expected Negotiated outcome"),
        }
    }

    #[tokio::test]
    async fn test_negotiate_server_closed_connection() {
        let (client, server) = duplex(4096);
        let mut server_reader = BufReader::new(server);

        drop(client);

        let outcome = negotiate_server(&mut server_reader).await.unwrap();
        assert!(matches!(outcome, NegotiationOutcome::Closed));
    }

    #[tokio::test]
    async fn test_negotiate_server_unknown_protocols_defaults_jsonrpc() {
        let (mut client, server) = duplex(4096);
        let mut server_reader = BufReader::new(server);

        tokio::spawn(async move {
            use tokio::io::AsyncWriteExt;
            client.write_all(b"PROTOCOLS: grpc,http2\n").await.unwrap();
            client.flush().await.unwrap();

            let mut response = String::new();
            let mut reader = BufReader::new(&mut client);
            tokio::io::AsyncBufReadExt::read_line(&mut reader, &mut response)
                .await
                .unwrap();
            assert_eq!(response.trim(), "PROTOCOL: jsonrpc");
        });

        let outcome = negotiate_server(&mut server_reader).await.unwrap();
        match outcome {
            NegotiationOutcome::Negotiated(proto) => {
                assert_eq!(proto, NegotiatedProtocol::JsonRpc);
            }
            _ => panic!("Expected Negotiated outcome"),
        }
    }
}
