//! Universal Primal Client (DEPRECATED - Use atomic_client!)
//!
//! ⚠️ WARNING: This module uses HTTP transport with C dependencies (reqwest->openssl-sys).
//! ⚠️ For ecoBin-compliant Pure Rust communication, use `atomic_client` with Unix sockets.
//!
//! Agnostic API negotiation layer for communicating with any primal,
//! regardless of response format, protocol, or API version.
//!
//! # Core Principle
//!
//! biomeOS adapts to primals, primals don't adapt to biomeOS.
//!
//! # Migration Path
//!
//! Old: HTTP-based universal client with format adaptation
//! New: JSON-RPC over Unix sockets with atomic_client

#![cfg(feature = "http-transport")]
//!
//! # Features
//!
//! - **Format Agnostic**: Handles wrapped, unwrapped, and custom response formats
//! - **Protocol Agnostic**: Supports HTTP, tarpc, gRPC, and more
//! - **Schema-Driven**: Parses OpenAPI, JSON Schema for automatic adaptation
//! - **Auto-Discovery**: Finds primals via mDNS, multicast, Consul, or config
//!
//! # Example
//!
//! ```rust,no_run
//! use biomeos_core::primal_client::UniversalPrimalClient;
//! use biomeos_core::discovery_http::IdentityResponse;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create client
//! let client = UniversalPrimalClient::new(Default::default());
//!
//! // Discover BearDog by capability
//! let beardog = client.discover_primal("security").await?;
//!
//! // Call endpoint - format is handled automatically
//! let identity: IdentityResponse = client
//!     .call(&beardog, "get_identity", ())
//!     .await?;
//!
//! println!("BearDog identity: {:?}", identity);
//! # Ok(())
//! # }
//! ```

pub mod adapters;
pub mod cache;
pub mod client;
pub mod config;
pub mod discovery;
pub mod error;
pub mod handle;
pub mod schema;
pub mod traits;

pub use client::UniversalPrimalClient;
pub use config::{ClientConfig, TrustPolicy};
pub use error::{ApiError, Result};
pub use handle::{Capability, Endpoint, PrimalHandle, PrimalId, PrimalMetadata};
pub use traits::{HealthStatus, PrimalClient};

// Re-export commonly used types
pub use adapters::format::FormatAdapter;
pub use adapters::protocol::ProtocolAdapter;
pub use discovery::DiscoveryClient;
pub use schema::{ApiSchema, SchemaParser};
