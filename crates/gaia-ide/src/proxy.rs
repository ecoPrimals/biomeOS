// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Reverse proxy handlers for Forgejo and Ollama APIs.
//!
//! The browser IDE at localhost:3141 cannot directly reach Forgejo (192.168.4.103:3000)
//! or Ollama (192.168.4.147:11434) due to CORS restrictions. This module proxies
//! those requests through the local Rust server, solving CORS and providing
//! a single unified API surface.

use std::sync::Arc;

use axum::{
    body::Body,
    extract::{Path, State},
    http::{Request, StatusCode, header},
    response::{IntoResponse, Response},
};
use tracing::{debug, warn};

use crate::AppState;

/// Proxy requests to the Forgejo API
///
/// `GET/POST /api/forge/repos/search` → `GET/POST {forge_url}/api/v1/repos/search`
pub async fn forge_proxy(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    req: Request<Body>,
) -> impl IntoResponse {
    let Some(forge_url) = &state.services.forge_url else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "Forgejo not discovered on LAN",
        )
            .into_response();
    };

    let target = format!("{forge_url}/api/v1/{path}");
    debug!("Forge proxy: {} → {}", req.uri(), target);
    proxy_request(&target, req).await
}

/// Proxy requests to the Ollama API
///
/// `GET/POST /api/ollama/tags` → `GET/POST {ollama_url}/api/tags`
/// `POST /api/ollama/chat` → `POST {ollama_url}/api/chat`
pub async fn ollama_proxy(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    req: Request<Body>,
) -> impl IntoResponse {
    let Some(ollama_url) = &state.services.ollama_url else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "Ollama not discovered on LAN",
        )
            .into_response();
    };

    let target = format!("{ollama_url}/api/{path}");
    debug!("Ollama proxy: {} → {}", req.uri(), target);
    proxy_request(&target, req).await
}

/// OpenAI-compatible API proxy — THE INTERCEPTION LAYER
///
/// Any tool that speaks the OpenAI API (Cursor, Windsurf, Continue.dev, aider, etc.)
/// can point at `http://localhost:3141/v1/` and it routes to Ollama's OpenAI endpoint.
///
/// This means: Cursor → localhost:3141/v1/chat/completions → Ollama → your local models.
/// Claude's API calls get intercepted and served by YOUR models. Zero fees. Zero cloud.
///
/// Gaia captures every prompt and response for learning.
pub async fn openai_compat_proxy(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    req: Request<Body>,
) -> impl IntoResponse {
    let Some(ollama_url) = &state.services.ollama_url else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "Ollama not discovered — cannot serve OpenAI-compatible requests",
        )
            .into_response();
    };

    // Ollama serves OpenAI-compatible at /v1/*
    let target = format!("{ollama_url}/v1/{path}");
    let query = req.uri().query().map(|q| format!("?{q}")).unwrap_or_default();
    let target_with_query = format!("{target}{query}");
    tracing::info!("🧠 OpenAI intercept: /v1/{path} → {target_with_query}");
    proxy_request(&target_with_query, req).await
}

/// Spine RSS proxy — routes to local Forge RSS feeds (the woven signal layer)
///
/// RSS spine IS the signal. MCP IS the stroll. They are WOVEN, not parallel.
/// `GET /spine/rss/ecoPrimals/biomeOS.rss` → `GET {forge_url}/ecoPrimals/biomeOS.rss`
/// `GET /spine/rss/ecoPrimals/biomeOS.atom` → `GET {forge_url}/ecoPrimals/biomeOS.atom`
pub async fn spine_rss_proxy(
    State(state): State<Arc<AppState>>,
    Path(path): Path<String>,
    req: Request<Body>,
) -> impl IntoResponse {
    let Some(forge_url) = &state.services.forge_url else {
        return (
            StatusCode::SERVICE_UNAVAILABLE,
            "Forge not discovered — RSS spine unavailable",
        )
            .into_response();
    };

    let target = format!("{forge_url}/{path}");
    let query = req.uri().query().map(|q| format!("?{q}")).unwrap_or_default();
    let target_with_query = format!("{target}{query}");
    tracing::info!("📡 RSS spine: /spine/rss/{path} → {target_with_query}");
    proxy_request(&target_with_query, req).await
}

/// Forward a request to a target URL using hyper
async fn proxy_request(target_url: &str, original: Request<Body>) -> Response {
    // Use tokio's TCP + hyper for the proxy connection
    let uri: hyper::Uri = match target_url.parse() {
        Ok(u) => u,
        Err(e) => {
            warn!("Invalid proxy target URL {target_url}: {e}");
            return (StatusCode::BAD_REQUEST, format!("Bad URL: {e}")).into_response();
        }
    };

    let host = uri.host().unwrap_or("localhost");
    let port = uri.port_u16().unwrap_or(80);
    let addr = format!("{host}:{port}");

    // Connect via TCP
    let stream = match tokio::net::TcpStream::connect(&addr).await {
        Ok(s) => s,
        Err(e) => {
            warn!("Failed to connect to {addr}: {e}");
            return (
                StatusCode::BAD_GATEWAY,
                format!("Cannot reach {addr}: {e}"),
            )
                .into_response();
        }
    };

    // Build hyper HTTP/1 connection
    let io = hyper_util::rt::TokioIo::new(stream);
    let (mut sender, conn) = match hyper::client::conn::http1::handshake(io).await {
        Ok(c) => c,
        Err(e) => {
            warn!("HTTP handshake failed with {addr}: {e}");
            return (StatusCode::BAD_GATEWAY, format!("Handshake error: {e}")).into_response();
        }
    };

    // Drive the connection in background
    tokio::spawn(async move {
        if let Err(e) = conn.await {
            warn!("Proxy connection error: {e}");
        }
    });

    // Build the proxied request
    let method = original.method().clone();
    let path_and_query = uri.path_and_query().map_or("/", |pq| pq.as_str());

    let mut proxy_req = Request::builder()
        .method(method)
        .uri(path_and_query)
        .header("Host", host);

    // Forward relevant headers
    for (name, value) in original.headers() {
        if name == header::HOST || name == header::CONNECTION {
            continue;
        }
        proxy_req = proxy_req.header(name, value);
    }

    let proxy_req = match proxy_req.body(original.into_body()) {
        Ok(r) => r,
        Err(e) => {
            return (StatusCode::INTERNAL_SERVER_ERROR, format!("Request build error: {e}"))
                .into_response();
        }
    };

    // Send and get response
    match sender.send_request(proxy_req).await {
        Ok(resp) => {
            let (parts, body) = resp.into_parts();
            let mut response = Response::from_parts(parts, Body::new(body));
            // Add CORS headers
            response.headers_mut().insert(
                header::ACCESS_CONTROL_ALLOW_ORIGIN,
                "*".parse().unwrap_or_else(|_| axum::http::HeaderValue::from_static("*")),
            );
            response
        }
        Err(e) => {
            warn!("Proxy request failed: {e}");
            (StatusCode::BAD_GATEWAY, format!("Proxy error: {e}")).into_response()
        }
    }
}
