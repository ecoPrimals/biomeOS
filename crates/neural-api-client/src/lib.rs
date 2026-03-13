// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Neural API Client - Pure Rust capability-based routing client
//!
//! This library enables primals to communicate with external services
//! and other primals **without** direct HTTP dependencies or knowledge
//! of other primals' existence.
//!
#![warn(missing_docs)]
#![forbid(unsafe_code)]
//!
//! # TRUE PRIMAL Pattern
//!
//! Primals using this client have **zero knowledge** of:
//! - Other primals (Songbird, BearDog, etc.)
//! - HTTP/TLS implementation details
//! - Crypto implementation
//! - Socket paths of other services
//!
//! They only know:
//! - "I need a capability" (e.g., "secure_http")
//! - "Neural API is at this socket"
//!
//! # Example
//!
//! ```no_run
//! use neural_api_client::NeuralApiClient;
//! use std::collections::HashMap;
//! use serde_json::json;
//!
//! # async fn example() -> anyhow::Result<()> {
//! let client = NeuralApiClient::discover("1894e909e454")?;
//!
//! let response = client.proxy_http(
//!     "POST",
//!     "https://api.anthropic.com/v1/messages",
//!     Some(HashMap::from([
//!         ("x-api-key".to_string(), "sk-...".to_string()),
//!     ])),
//!     Some(json!({
//!         "model": "claude-3-opus-20240229",
//!         "messages": [{"role": "user", "content": "Hello!"}]
//!     }))
//! ).await?;
//!
//! println!("Response: {}", response.body_str().unwrap_or("<invalid utf8>"));
//! # Ok(())
//! # }
//! ```

mod client;
mod connection;
mod error;
mod types;

pub use client::NeuralApiClient;
pub use error::NeuralApiError;
pub use types::{CapabilityInfo, HttpResponse, PrimalInfo, RoutingMetric, RoutingMetrics};

#[cfg(test)]
mod tests;
