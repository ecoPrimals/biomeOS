// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Virtual service and HTTP/TCP/TLS routing types.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Virtual service specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualServiceSpec {
    /// Service name
    pub name: String,

    /// Hosts
    pub hosts: Vec<String>,

    /// Gateways
    pub gateways: Vec<String>,

    /// HTTP routes
    pub http: Vec<HttpRouteSpec>,

    /// TCP routes
    pub tcp: Vec<TcpRouteSpec>,

    /// TLS routes
    pub tls: Vec<TlsRouteSpec>,
}

// HTTP routing types for VirtualService
/// HTTP route specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRouteSpec {
    /// Match conditions
    pub match_conditions: Vec<HttpMatchCondition>,

    /// Route destinations
    pub route: Vec<HttpRouteDestination>,

    /// Redirect
    pub redirect: Option<HttpRedirect>,

    /// Rewrite
    pub rewrite: Option<HttpRewrite>,

    /// Timeout
    pub timeout: Option<u32>,

    /// Retries
    pub retries: Option<HttpRetry>,
}

/// HTTP match condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpMatchCondition {
    /// URI
    pub uri: Option<StringMatch>,

    /// Scheme
    pub scheme: Option<StringMatch>,

    /// Method
    pub method: Option<StringMatch>,

    /// Authority
    pub authority: Option<StringMatch>,

    /// Headers
    pub headers: HashMap<String, StringMatch>,

    /// Query parameters
    pub query_params: HashMap<String, StringMatch>,
}

/// String match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StringMatch {
    /// Exact string match
    Exact(String),
    /// String prefix match
    Prefix(String),
    /// Regular expression match
    Regex(String),
}

/// HTTP route destination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRouteDestination {
    /// Destination
    pub destination: DestinationSpec,

    /// Weight
    pub weight: Option<u32>,

    /// Headers
    pub headers: Option<HeadersSpec>,
}

/// Destination specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DestinationSpec {
    /// Host
    pub host: String,

    /// Subset
    pub subset: Option<String>,

    /// Port
    pub port: Option<PortSelector>,
}

/// Port selector
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PortSelector {
    /// Port number
    Number(u16),
    /// Port name
    Name(String),
}

/// Headers specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeadersSpec {
    /// Request headers
    pub request: Option<HeaderOperations>,

    /// Response headers
    pub response: Option<HeaderOperations>,
}

/// Header operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderOperations {
    /// Set headers
    pub set: HashMap<String, String>,

    /// Add headers
    pub add: HashMap<String, String>,

    /// Remove headers
    pub remove: Vec<String>,
}

/// HTTP redirect
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRedirect {
    /// URI
    pub uri: Option<String>,

    /// Authority
    pub authority: Option<String>,

    /// Redirect code
    pub redirect_code: Option<u16>,
}

/// HTTP rewrite
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRewrite {
    /// URI
    pub uri: Option<String>,

    /// Authority
    pub authority: Option<String>,
}

/// HTTP retry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpRetry {
    /// Attempts
    pub attempts: u32,

    /// Per try timeout
    pub per_try_timeout: Option<u32>,

    /// Retry on
    pub retry_on: Option<String>,
}

/// TCP route specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpRouteSpec {
    /// Match conditions
    pub match_conditions: Vec<TcpMatchCondition>,

    /// Route destinations
    pub route: Vec<TcpRouteDestination>,
}

/// TCP match condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpMatchCondition {
    /// Destination subnets
    pub destination_subnets: Vec<String>,

    /// Port
    pub port: Option<u16>,

    /// Source labels
    pub source_labels: HashMap<String, String>,

    /// Gateways
    pub gateways: Vec<String>,
}

/// TCP route destination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpRouteDestination {
    /// Destination
    pub destination: DestinationSpec,

    /// Weight
    pub weight: Option<u32>,
}

/// TLS route specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsRouteSpec {
    /// Match conditions
    pub match_conditions: Vec<TlsMatchCondition>,

    /// Route destinations
    pub route: Vec<TlsRouteDestination>,
}

/// TLS match condition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsMatchCondition {
    /// SNI hosts
    pub sni_hosts: Vec<String>,

    /// Destination subnets
    pub destination_subnets: Vec<String>,

    /// Port
    pub port: Option<u16>,

    /// Source labels
    pub source_labels: HashMap<String, String>,

    /// Gateways
    pub gateways: Vec<String>,
}

/// TLS route destination
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TlsRouteDestination {
    /// Destination
    pub destination: DestinationSpec,

    /// Weight
    pub weight: Option<u32>,
}
