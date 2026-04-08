// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Newline-delimited JSON-RPC request/response and streaming over byte streams.

use anyhow::{Context, Result};
use biomeos_graph::StreamItem;
use biomeos_types::{JsonRpcRequest, JsonRpcResponse};
use tokio::io::{AsyncBufReadExt, AsyncRead, AsyncWrite, AsyncWriteExt, BufReader};
use tracing::trace;

/// Send one JSON-RPC request and read one newline-delimited response.
pub(crate) async fn send_jsonrpc_line<S>(stream: S, request: JsonRpcRequest) -> Result<JsonRpcResponse>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let (reader, mut writer) = tokio::io::split(stream);

    let request_str =
        serde_json::to_string(&request).context("Failed to serialize JSON-RPC request")?;

    trace!("Sending JSON-RPC request: {}", request_str);

    writer.write_all(request_str.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    let mut reader = BufReader::new(reader);
    let mut line = String::new();
    reader
        .read_line(&mut line)
        .await
        .context("Failed to read JSON-RPC response")?;

    trace!("Received JSON-RPC response: {}", line.trim());

    let response: JsonRpcResponse =
        serde_json::from_str(&line).context("Failed to parse JSON-RPC response")?;

    Ok(response)
}

/// Send a request and read multiple NDJSON response lines as `StreamItem`s.
pub(crate) async fn pump_ndjson_stream<S>(
    stream: S,
    request: JsonRpcRequest,
    tx: &tokio::sync::mpsc::Sender<StreamItem>,
) -> Result<()>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    let (reader, mut writer) = tokio::io::split(stream);

    let request_str =
        serde_json::to_string(&request).context("Failed to serialize streaming request")?;

    writer.write_all(request_str.as_bytes()).await?;
    writer.write_all(b"\n").await?;
    writer.flush().await?;

    let mut reader = BufReader::new(reader);
    let mut line = String::new();

    loop {
        line.clear();
        let n = reader.read_line(&mut line).await?;
        if n == 0 {
            let _ = tx.send(StreamItem::End).await;
            break;
        }

        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }

        if let Ok(item) = serde_json::from_str::<StreamItem>(trimmed) {
            let is_end = matches!(item, StreamItem::End);
            if tx.send(item).await.is_err() {
                break;
            }
            if is_end {
                break;
            }
        } else if let Ok(resp) = serde_json::from_str::<JsonRpcResponse>(trimmed) {
            if let Some(result) = resp.result {
                let _ = tx.send(StreamItem::Data(result)).await;
            }
            let _ = tx.send(StreamItem::End).await;
            break;
        } else {
            let _ = tx
                .send(StreamItem::Data(serde_json::Value::String(
                    trimmed.to_string(),
                )))
                .await;
        }
    }

    Ok(())
}
