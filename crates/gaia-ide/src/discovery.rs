// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Service discovery for LAN services (Forgejo, Ollama, etc.)
//!
//! Probes known LAN addresses to find running services.
//! In the future, this will integrate with NUCLEUS for proper
//! capability-based discovery via songBird UDP multicast.

use std::time::Duration;
use tokio::net::TcpStream;
use tracing::{debug, info, warn};

/// Registry of discovered services on the LAN
#[derive(Debug, Clone)]
pub struct ServiceRegistry {
    /// Forgejo git forge URL (e.g., `http://192.168.4.103:3000`)
    pub forge_url: Option<String>,
    /// Ollama AI API URL (e.g., `http://192.168.4.147:11434`)
    pub ollama_url: Option<String>,
    /// Detected primals on the network
    pub primals: Vec<String>,
}

/// Known LAN hosts to probe for services
const FORGE_CANDIDATES: &[(&str, u16)] = &[
    ("192.168.4.103", 3000), // eastgate — primary forge
    ("192.168.4.194", 3000), // eastgate secondary
    ("127.0.0.1", 3000),     // local
];

const OLLAMA_CANDIDATES: &[(&str, u16)] = &[
    ("192.168.4.147", 11434), // irongate/northgate — primary AI
    ("192.168.4.103", 11434), // eastgate
    ("127.0.0.1", 11434),     // local
];

/// Discover services on the LAN by probing known addresses.
///
/// Runs all probes concurrently with 3-second timeouts.
pub async fn discover_services() -> ServiceRegistry {
    info!("🔍 Discovering services on LAN...");

    let (forge, ollama) = tokio::join!(discover_forge(), discover_ollama());

    let mut primals = Vec::new();
    if forge.is_some() {
        primals.push("forgejo".to_string());
    }
    if ollama.is_some() {
        primals.push("ollama".to_string());
    }

    let registry = ServiceRegistry {
        forge_url: forge,
        ollama_url: ollama,
        primals,
    };

    info!("Discovery complete: {} services found", registry.primals.len());
    registry
}

/// Probe for a Forgejo instance
async fn discover_forge() -> Option<String> {
    for &(host, port) in FORGE_CANDIDATES {
        let url = format!("http://{host}:{port}");
        if probe_tcp(host, port).await {
            // Verify it's actually Forgejo by checking the API version endpoint
            if probe_http(&format!("{url}/api/v1/version")).await {
                info!("✅ Forge discovered: {url}");
                return Some(url);
            }
        }
    }
    warn!("⚠️ No Forgejo instance found on LAN");
    None
}

/// Probe for an Ollama instance
async fn discover_ollama() -> Option<String> {
    for &(host, port) in OLLAMA_CANDIDATES {
        let url = format!("http://{host}:{port}");
        if probe_tcp(host, port).await {
            // Verify it's Ollama by checking the tags endpoint
            if probe_http(&format!("{url}/api/tags")).await {
                info!("✅ Ollama discovered: {url}");
                return Some(url);
            }
        }
    }
    warn!("⚠️ No Ollama instance found on LAN");
    None
}

/// TCP connection probe with timeout
async fn probe_tcp(host: &str, port: u16) -> bool {
    let addr = format!("{host}:{port}");
    debug!("Probing TCP {addr}...");
    tokio::time::timeout(Duration::from_secs(3), TcpStream::connect(&addr))
        .await
        .map_or(false, |r| r.is_ok())
}

/// HTTP GET probe with timeout — checks if the endpoint returns a success status
async fn probe_http(url: &str) -> bool {
    debug!("Probing HTTP {url}...");
    // Simple HTTP/1.1 GET using raw TCP + minimal request
    let uri: hyper::Uri = match url.parse() {
        Ok(u) => u,
        Err(_) => return false,
    };

    let host = uri.host().unwrap_or("localhost");
    let port = uri.port_u16().unwrap_or(80);
    let path = uri.path();

    let stream = match tokio::time::timeout(
        Duration::from_secs(3),
        TcpStream::connect(format!("{host}:{port}")),
    )
    .await
    {
        Ok(Ok(s)) => s,
        _ => return false,
    };

    // Send minimal HTTP request
    let request = format!("GET {path} HTTP/1.1\r\nHost: {host}\r\nConnection: close\r\n\r\n");
    use tokio::io::{AsyncReadExt, AsyncWriteExt};
    let mut stream = stream;
    if stream.write_all(request.as_bytes()).await.is_err() {
        return false;
    }

    let mut buf = vec![0u8; 1024];
    match tokio::time::timeout(Duration::from_secs(3), stream.read(&mut buf)).await {
        Ok(Ok(n)) if n > 0 => {
            let response = String::from_utf8_lossy(&buf[..n]);
            // Check for HTTP 200 OK
            response.starts_with("HTTP/1.1 200") || response.starts_with("HTTP/1.0 200")
        }
        _ => false,
    }
}
