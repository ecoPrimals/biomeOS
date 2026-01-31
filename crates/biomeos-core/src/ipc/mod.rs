//! Platform-Agnostic IPC Module
//!
//! **TRUE ecoBin v2.0:** Universal IPC that works on ANY platform.
//!
//! This module provides a unified IPC abstraction that automatically selects
//! the best transport mechanism for each platform:
//!
//! - **Linux/macOS:** Unix domain sockets
//! - **Android:** Abstract sockets (no filesystem required)
//! - **Windows:** Named pipes
//! - **iOS:** XPC (when available)
//! - **WASM:** In-process channels
//!
//! ## Philosophy
//!
//! > "If it can't run on the arch/platform, it's not a true ecoBin"
//!
//! ## Usage
//!
//! ```ignore
//! use biomeos_core::ipc::{Transport, detect_best_transport};
//!
//! // Automatic platform detection
//! let transport = detect_best_transport("beardog")?;
//!
//! // Connect using platform-appropriate mechanism
//! let stream = transport.connect().await?;
//! ```

pub mod transport;

pub use transport::{detect_best_transport, Transport, TransportType};
