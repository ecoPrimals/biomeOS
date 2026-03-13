// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Tracing configuration types

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::logging::LogLevel;

/// Tracing configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingConfig {
    /// Enable tracing
    pub enabled: bool,
    /// Tracing level
    pub level: LogLevel,
    /// Tracing exporter
    pub exporter: TracingExporter,
    /// Tracing sampling
    pub sampling: TracingSamplingConfig,
    /// Span limits
    pub span_limits: SpanLimitsConfig,
    /// Resource configuration
    pub resource: TracingResourceConfig,
}

/// Tracing exporters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TracingExporter {
    /// Console output
    Console,
    /// Jaeger distributed tracing
    Jaeger(JaegerConfig),
    /// Zipkin distributed tracing
    Zipkin(ZipkinConfig),
    /// OpenTelemetry Protocol (OTLP)
    Otlp(OtlpConfig),
    /// Custom exporter
    Custom(String),
}

/// Jaeger configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JaegerConfig {
    /// Jaeger endpoint
    pub endpoint: String,
    /// Service name
    pub service_name: String,
    /// Authentication
    pub auth: Option<TracingAuth>,
}

/// Zipkin configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZipkinConfig {
    /// Zipkin endpoint
    pub endpoint: String,
    /// Service name
    pub service_name: String,
}

/// OTLP configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtlpConfig {
    /// OTLP endpoint
    pub endpoint: String,
    /// Protocol
    pub protocol: OtlpProtocol,
    /// Headers
    pub headers: HashMap<String, String>,
    /// Compression
    pub compression: Option<OtlpCompression>,
}

/// OTLP protocols
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OtlpProtocol {
    /// gRPC transport
    Grpc,
    /// HTTP transport
    Http,
}

/// OTLP compression
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OtlpCompression {
    /// Gzip compression
    Gzip,
    /// No compression
    None,
}

/// Tracing authentication
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TracingAuth {
    /// Bearer token
    Bearer(String),
    /// HTTP basic auth
    Basic {
        /// Username
        username: String,
        /// Password
        password: String,
    },
    /// API key auth
    ApiKey {
        /// API key value
        key: String,
        /// Header name
        header: String,
    },
}

/// Tracing sampling configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingSamplingConfig {
    /// Sampling rate (0.0-1.0)
    pub rate: f64,
    /// Sampling strategy
    pub strategy: TracingSamplingStrategy,
    /// Parent-based sampling
    pub parent_based: bool,
}

/// Tracing sampling strategies
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TracingSamplingStrategy {
    /// Always sample
    Always,
    /// Never sample
    Never,
    /// Sample based on trace ID ratio
    TraceIdRatio,
    /// Rate-limited sampling
    RateLimited {
        /// Maximum traces per second
        rate: u32,
    },
    /// Custom sampling strategy
    Custom(String),
}

/// Span limits configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpanLimitsConfig {
    /// Max attributes per span
    pub max_attributes: Option<u32>,
    /// Max events per span
    pub max_events: Option<u32>,
    /// Max links per span
    pub max_links: Option<u32>,
    /// Max attribute value length
    pub max_attribute_value_length: Option<u32>,
}

/// Tracing resource configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TracingResourceConfig {
    /// Service name
    pub service_name: String,
    /// Service version
    pub service_version: Option<String>,
    /// Service namespace
    pub service_namespace: Option<String>,
    /// Additional attributes
    pub attributes: HashMap<String, String>,
}

impl Default for TracingConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            level: LogLevel::Info,
            exporter: TracingExporter::Console,
            sampling: TracingSamplingConfig::default(),
            span_limits: SpanLimitsConfig::default(),
            resource: TracingResourceConfig::default(),
        }
    }
}

impl Default for TracingSamplingConfig {
    fn default() -> Self {
        Self {
            rate: 1.0,
            strategy: TracingSamplingStrategy::Always,
            parent_based: true,
        }
    }
}

impl Default for SpanLimitsConfig {
    fn default() -> Self {
        Self {
            max_attributes: Some(128),
            max_events: Some(128),
            max_links: Some(128),
            max_attribute_value_length: Some(4096),
        }
    }
}

impl Default for TracingResourceConfig {
    fn default() -> Self {
        Self {
            service_name: "biomeos".to_string(),
            service_version: None,
            service_namespace: None,
            attributes: HashMap::new(),
        }
    }
}
