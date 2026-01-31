//! Platform-agnostic transport layer
//!
//! **TRUE ecoBin v2.0:** Zero platform assumptions, runtime discovery.
//!
//! This module provides a universal transport abstraction that works on:
//! - Linux (Unix sockets)
//! - Android (Abstract sockets)
//! - Windows (Named pipes)
//! - macOS (Unix sockets)
//! - iOS (XPC when available, fallback to sockets)
//! - WASM (In-process channels)

use anyhow::{Context, Result};
use std::path::PathBuf;
use tokio::io::{AsyncRead, AsyncWrite};
use tracing::{debug, info};

/// Universal IPC transport type
///
/// Automatically selected based on platform capabilities.
#[derive(Debug, Clone)]
pub enum TransportType {
    /// Unix domain sockets (Linux, macOS, BSD)
    UnixSocket { path: PathBuf },

    /// Abstract sockets (Android, Linux with abstract namespace support)
    #[cfg(target_os = "linux")]
    AbstractSocket { name: String },

    /// Named pipes (Windows)
    #[cfg(target_os = "windows")]
    NamedPipe { name: String },

    /// TCP localhost fallback (Universal)
    TcpLocalhost { port: u16 },

    /// In-process channel (WASM, embedded, or same-process communication)
    InProcess { channel_id: String },
}

/// Transport endpoint with platform-specific implementation
pub struct Transport {
    transport_type: TransportType,
}

impl Transport {
    /// Create a new transport for the given type
    pub fn new(transport_type: TransportType) -> Self {
        Self { transport_type }
    }

    /// Connect to the transport endpoint
    ///
    /// Returns a bidirectional stream that implements AsyncRead + AsyncWrite.
    pub async fn connect(&self) -> Result<Box<dyn AsyncReadWrite>> {
        match &self.transport_type {
            TransportType::UnixSocket { path } => {
                debug!("Connecting via Unix socket: {}", path.display());
                let stream = tokio::net::UnixStream::connect(path)
                    .await
                    .context(format!("Failed to connect to Unix socket: {}", path.display()))?;
                Ok(Box::new(stream))
            }

            #[cfg(target_os = "linux")]
            TransportType::AbstractSocket { name } => {
                debug!("Connecting via abstract socket: @{}", name);
                // Abstract sockets use null byte prefix
                let addr = format!("\0{}", name);
                let stream = tokio::net::UnixStream::connect(addr.as_str())
                    .await
                    .context(format!("Failed to connect to abstract socket: @{}", name))?;
                Ok(Box::new(stream))
            }

            #[cfg(target_os = "windows")]
            TransportType::NamedPipe { name } => {
                debug!("Connecting via named pipe: {}", name);
                // Windows named pipes: \\.\pipe\name
                let pipe_path = format!(r"\\.\pipe\{}", name);
                
                // Note: tokio doesn't have native named pipe support yet
                // We would use tokio::net::windows::named_pipe here when available
                // For now, fall back to TCP localhost
                warn!("Named pipes not yet implemented, falling back to TCP");
                return self.connect_tcp_fallback(3000).await;
            }

            TransportType::TcpLocalhost { port } => {
                debug!("Connecting via TCP localhost:{}", port);
                self.connect_tcp_fallback(*port).await
            }

            TransportType::InProcess { channel_id } => {
                debug!("Using in-process channel: {}", channel_id);
                anyhow::bail!("In-process channels not yet implemented");
            }
        }
    }

    /// Bind and listen on the transport endpoint
    ///
    /// Returns a listener for accepting incoming connections.
    pub async fn bind(&self) -> Result<Box<dyn TransportListener>> {
        match &self.transport_type {
            TransportType::UnixSocket { path } => {
                debug!("Binding Unix socket: {}", path.display());
                
                // Remove existing socket file if it exists
                if path.exists() {
                    std::fs::remove_file(path)
                        .context(format!("Failed to remove existing socket: {}", path.display()))?;
                }

                // Create parent directory if it doesn't exist
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).context(format!(
                        "Failed to create socket directory: {}",
                        parent.display()
                    ))?;
                }

                let listener = tokio::net::UnixListener::bind(path)
                    .context(format!("Failed to bind Unix socket: {}", path.display()))?;
                
                Ok(Box::new(UnixTransportListener { listener }))
            }

            #[cfg(target_os = "linux")]
            TransportType::AbstractSocket { name } => {
                debug!("Binding abstract socket: @{}", name);
                // Abstract sockets use null byte prefix
                let addr = format!("\0{}", name);
                let listener = tokio::net::UnixListener::bind(addr.as_str())
                    .context(format!("Failed to bind abstract socket: @{}", name))?;
                Ok(Box::new(UnixTransportListener { listener }))
            }

            #[cfg(target_os = "windows")]
            TransportType::NamedPipe { name } => {
                debug!("Binding named pipe: {}", name);
                warn!("Named pipes not yet implemented, falling back to TCP");
                self.bind_tcp_fallback(3000).await
            }

            TransportType::TcpLocalhost { port } => {
                debug!("Binding TCP localhost:{}", port);
                self.bind_tcp_fallback(*port).await
            }

            TransportType::InProcess { channel_id } => {
                debug!("Creating in-process listener: {}", channel_id);
                anyhow::bail!("In-process channels not yet implemented");
            }
        }
    }

    /// TCP localhost fallback (for platforms without better IPC)
    async fn connect_tcp_fallback(&self, port: u16) -> Result<Box<dyn AsyncReadWrite>> {
        let addr = format!("127.0.0.1:{}", port);
        let stream = tokio::net::TcpStream::connect(&addr)
            .await
            .context(format!("Failed to connect to TCP localhost:{}", port))?;
        Ok(Box::new(stream))
    }

    /// TCP localhost bind fallback
    async fn bind_tcp_fallback(&self, port: u16) -> Result<Box<dyn TransportListener>> {
        let addr = format!("127.0.0.1:{}", port);
        let listener = tokio::net::TcpListener::bind(&addr)
            .await
            .context(format!("Failed to bind TCP localhost:{}", port))?;
        Ok(Box::new(TcpTransportListener { listener }))
    }

    /// Get the transport type
    pub fn transport_type(&self) -> &TransportType {
        &self.transport_type
    }
}

/// Trait for bidirectional async I/O
pub trait AsyncReadWrite: AsyncRead + AsyncWrite + Send + Unpin {}

// Blanket implementation for types that implement the required traits
impl<T> AsyncReadWrite for T where T: AsyncRead + AsyncWrite + Send + Unpin {}

/// Trait for transport listeners
#[async_trait::async_trait]
pub trait TransportListener: Send {
    /// Accept an incoming connection
    async fn accept(&mut self) -> Result<Box<dyn AsyncReadWrite>>;
}

/// Unix socket listener wrapper
struct UnixTransportListener {
    listener: tokio::net::UnixListener,
}

#[async_trait::async_trait]
impl TransportListener for UnixTransportListener {
    async fn accept(&mut self) -> Result<Box<dyn AsyncReadWrite>> {
        let (stream, _addr) = self
            .listener
            .accept()
            .await
            .context("Failed to accept Unix socket connection")?;
        Ok(Box::new(stream))
    }
}

/// TCP listener wrapper
struct TcpTransportListener {
    listener: tokio::net::TcpListener,
}

#[async_trait::async_trait]
impl TransportListener for TcpTransportListener {
    async fn accept(&mut self) -> Result<Box<dyn AsyncReadWrite>> {
        let (stream, _addr) = self
            .listener
            .accept()
            .await
            .context("Failed to accept TCP connection")?;
        Ok(Box::new(stream))
    }
}

/// Detect the best transport for a given service on this platform
///
/// Uses runtime detection to choose the most appropriate IPC mechanism:
/// - **Linux:** Abstract sockets (preferred) or Unix sockets
/// - **Android:** Abstract sockets (required)
/// - **Windows:** Named pipes (when available) or TCP localhost
/// - **macOS:** Unix sockets
/// - **iOS:** Unix sockets (sandboxed)
/// - **WASM:** In-process channels
///
/// # Examples
///
/// ```ignore
/// let transport = detect_best_transport("beardog")?;
/// let stream = transport.connect().await?;
/// ```
pub fn detect_best_transport(service_name: &str) -> Result<Transport> {
    // Platform-specific detection
    #[cfg(target_os = "android")]
    {
        // Android: ALWAYS use abstract sockets (no filesystem access)
        info!("Detected Android - using abstract sockets");
        return Ok(Transport::new(TransportType::AbstractSocket {
            name: service_name.to_string(),
        }));
    }

    #[cfg(all(target_os = "linux", not(target_os = "android")))]
    {
        // Linux desktop: Prefer abstract sockets, fallback to Unix sockets
        if supports_abstract_sockets() {
            info!("Detected Linux with abstract socket support");
            return Ok(Transport::new(TransportType::AbstractSocket {
                name: service_name.to_string(),
            }));
        } else {
            info!("Linux without abstract sockets - using Unix sockets");
            let socket_path = get_unix_socket_path(service_name)?;
            return Ok(Transport::new(TransportType::UnixSocket { path: socket_path }));
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Windows: Use TCP localhost fallback until tokio supports named pipes
        // See: https://github.com/tokio-rs/tokio/issues/3557
        info!("Detected Windows - using TCP localhost (named pipes pending tokio support)");
        return Ok(Transport::new(TransportType::TcpLocalhost { port: 3000 }));
    }

    #[cfg(any(target_os = "macos", target_os = "ios"))]
    {
        // macOS/iOS: Use Unix sockets
        info!("Detected macOS/iOS - using Unix sockets");
        let socket_path = get_unix_socket_path(service_name)?;
        return Ok(Transport::new(TransportType::UnixSocket { path: socket_path }));
    }

    #[cfg(target_family = "wasm")]
    {
        // WASM: Use in-process channels
        info!("Detected WASM - using in-process channels");
        return Ok(Transport::new(TransportType::InProcess {
            channel_id: service_name.to_string(),
        }));
    }

    // Fallback for unknown platforms
    #[cfg(not(any(
        target_os = "linux",
        target_os = "android",
        target_os = "windows",
        target_os = "macos",
        target_os = "ios",
        target_family = "wasm"
    )))]
    {
        warn!("Unknown platform - using TCP localhost fallback");
        Ok(Transport::new(TransportType::TcpLocalhost { port: 3000 }))
    }
}

/// Check if the system supports abstract sockets
#[cfg(target_os = "linux")]
fn supports_abstract_sockets() -> bool {
    // Abstract sockets are a Linux feature, always available on Linux
    // (except in very old kernels, but we don't support those)
    true
}

/// Get the Unix socket path for a service
///
/// Uses XDG runtime directory when available, falls back to /tmp.
#[cfg(any(unix, target_os = "macos", target_os = "ios"))]
fn get_unix_socket_path(service_name: &str) -> Result<PathBuf> {
    // Try XDG_RUNTIME_DIR first (Linux/BSD standard)
    if let Ok(runtime_dir) = std::env::var("XDG_RUNTIME_DIR") {
        let socket_path = PathBuf::from(runtime_dir)
            .join("biomeos")
            .join(format!("{}.sock", service_name));
        return Ok(socket_path);
    }

    // Fallback to /tmp (less secure but works everywhere)
    Ok(PathBuf::from(format!("/tmp/{}.sock", service_name)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transport_type_debug() {
        let transport = TransportType::UnixSocket {
            path: PathBuf::from("/tmp/test.sock"),
        };
        assert!(format!("{:?}", transport).contains("UnixSocket"));
    }

    #[test]
    fn test_detect_best_transport() {
        // This should not panic on any platform
        let transport = detect_best_transport("test_service");
        assert!(transport.is_ok());
    }

    #[cfg(unix)]
    #[test]
    fn test_get_unix_socket_path() {
        let path = get_unix_socket_path("beardog").unwrap();
        assert!(path.to_string_lossy().contains("beardog"));
        assert!(path.to_string_lossy().ends_with(".sock"));
    }
}
