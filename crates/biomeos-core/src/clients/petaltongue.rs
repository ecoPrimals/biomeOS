//! # petalTongue Client - Universal User Interface
//!
//! Client for interacting with petalTongue, the Universal User Interface primal.
//! petalTongue provides multi-modal visualization and interaction capabilities:
//! - Visual: Graphs, charts, topology maps, 3D rendering
//! - Audio: Sonification, alerts, ambient feedback
//! - Terminal: ASCII art, dashboards, status displays
//! - Framebuffer: Direct pixel rendering for embedded systems
//!
//! ## Transport Protocol
//! Uses PrimalClient abstraction with JSON-RPC 2.0 over Unix sockets.
//! Socket path: `/run/user/<uid>/petaltongue-<family>.sock`
//!
//! ## Capabilities
//! - `visualization` - Data visualization and rendering
//! - `graph-compute` - Graph layout and analysis
//! - `multi-modal` - Multiple output modalities (visual, audio, terminal)
//! - `gpu-rendering` - GPU-accelerated rendering (when available)
//!
//! ## Discovery
//! petalTongue is discovered via Songbird by capability:
//! ```ignore
//! let songbird = SongbirdClient::discover().await?;
//! let petaltongue_services = songbird
//!     .discover_by_capability("visualization")
//!     .await?;
//! ```

use crate::clients::transport::{TransportClient, TransportError};
use crate::primal_client::{HealthStatus, PrimalClient};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

/// petalTongue client for Universal User Interface operations
///
/// Provides methods for interacting with petalTongue's visualization
/// and multi-modal rendering capabilities.
///
/// # Examples
///
/// ```ignore
/// use biomeos_core::clients::petaltongue::PetalTongueClient;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     // Discover and connect to petalTongue
///     let ui = PetalTongueClient::discover().await?;
///    
///     // Check health
///     let health = ui.health().await?;
///     println!("petalTongue status: {:?}", health.status);
///    
///     // Get capabilities
///     let caps = ui.get_capabilities().await?;
///     println!("Available capabilities: {:?}", caps);
///    
///     Ok(())
/// }
/// ```ignore
#[derive(Debug, Clone)]
pub struct PetalTongueClient {
    transport: TransportClient,
}

#[async_trait]
impl PrimalClient for PetalTongueClient {
    fn name(&self) -> &str {
        "petaltongue"
    }

    fn endpoint(&self) -> String {
        self.transport.endpoint().to_string()
    }

    async fn is_available(&self) -> bool {
        self.health_check().await.is_ok()
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        self.transport.health_check().await
    }

    async fn request(&self, method: &str, _path: &str, body: Option<Value>) -> Result<Value> {
        self.transport.call(method, body).await
    }
}

impl PetalTongueClient {
    /// Discover petalTongue via Unix socket
    ///
    /// # Returns
    /// A connected petalTongue client
    ///
    /// # Errors
    /// Returns error if:
    /// - petalTongue is not running
    /// - Transport connection fails
    pub async fn discover(family_id: &str) -> Result<Self, TransportError> {
        let transport = TransportClient::discover("petaltongue", family_id)
            .await
            .map_err(|e| TransportError::Other(e))?;
        Ok(Self { transport })
    }

    /// Check health status
    ///
    /// Returns current health and operational status of petalTongue.
    ///
    /// # Returns
    /// Health status with version, uptime, and metrics
    pub async fn health(&self) -> Result<PetalTongueHealthResponse> {
        let v = self
            .transport
            .call("health", Some(serde_json::json!({})))
            .await?;
        Ok(serde_json::from_value(v)?)
    }

    /// Get primal capabilities
    ///
    /// Returns the list of capabilities this petalTongue instance provides.
    ///
    /// # Returns
    /// List of capability strings (e.g., "visualization", "gpu-rendering")
    pub async fn get_capabilities(&self) -> Result<Vec<String>> {
        let v = self
            .transport
            .call("get_capabilities", Some(serde_json::json!({})))
            .await?;
        Ok(serde_json::from_value(v)?)
    }

    /// Render a graph or visualization
    ///
    /// Requests petalTongue to render data using specified modality.
    ///
    /// # Arguments
    /// * `request` - Rendering request with data and output mode
    ///
    /// # Returns
    /// Rendering response with result data or file path
    ///
    /// # Example
    /// ```ignore
    /// # use biomeos_core::clients::petaltongue::{PetalTongueClient, RenderRequest};
    /// # async fn example(ui: &PetalTongueClient) -> Result<(), Box<dyn std::error::Error>> {
    /// let request = RenderRequest {
    ///     mode: "svg".to_string(),
    ///     data: serde_json::json!({"nodes": [{"id": "a"}], "edges": []}),
    ///     width: Some(1920),
    ///     height: Some(1080),
    ///     output_path: Some("/tmp/graph.svg".to_string()),
    /// };
    ///
    /// let response = ui.render(request).await?;
    /// println!("Rendered to: {:?}", response.output_path);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn render(&self, request: RenderRequest) -> Result<RenderResponse> {
        let v = self
            .transport
            .call("render", Some(serde_json::json!(request)))
            .await?;
        Ok(serde_json::from_value(v)?)
    }

    /// Query graph metrics
    ///
    /// Get topology and performance metrics about the current graph.
    ///
    /// # Returns
    /// Graph metrics including node count, edge count, layout time
    pub async fn graph_metrics(&self) -> Result<GraphMetrics> {
        let v = self
            .transport
            .call("graph_metrics", Some(serde_json::json!({})))
            .await?;
        Ok(serde_json::from_value(v)?)
    }

    /// List available output modalities
    ///
    /// Returns the rendering modalities supported by this instance.
    ///
    /// # Returns
    /// List of supported modalities (e.g., "terminal", "svg", "png", "audio")
    pub async fn list_modalities(&self) -> Result<Vec<String>> {
        let v = self
            .transport
            .call("list_modalities", Some(serde_json::json!({})))
            .await?;
        Ok(serde_json::from_value(v)?)
    }

    /// Discover primal by capability
    ///
    /// Query petalTongue's discovery system for primals with specific capability.
    ///
    /// # Arguments
    /// * `capability` - Required capability to search for
    ///
    /// # Returns
    /// List of primal endpoints that provide the capability
    pub async fn discover_capability(&self, capability: &str) -> Result<Vec<PrimalEndpoint>> {
        let v = self
            .transport
            .call(
                "discover_capability",
                Some(serde_json::json!({ "capability": capability })),
            )
            .await?;
        Ok(serde_json::from_value(v)?)
    }
}

// ============================================================================
// Types
// ============================================================================

/// petalTongue health status response (detailed)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PetalTongueHealthResponse {
    /// Current status: "healthy", "degraded", "unhealthy"
    pub status: String,

    /// Service version
    pub version: String,

    /// Uptime in seconds
    pub uptime_secs: u64,

    /// Active connections
    pub connections: usize,

    /// Optional metrics
    #[serde(default)]
    pub metrics: HashMap<String, serde_json::Value>,
}

/// Render request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderRequest {
    /// Output mode: "terminal", "svg", "png", "json", "dot"
    pub mode: String,

    /// Data to render (nodes, edges, or arbitrary visualization data)
    pub data: serde_json::Value,

    /// Output width in pixels (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,

    /// Output height in pixels (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,

    /// Output file path (for export modes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_path: Option<String>,
}

/// Render response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderResponse {
    /// Success flag
    pub success: bool,

    /// Result data (for inline modes like JSON)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<serde_json::Value>,

    /// Output file path (for export modes)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output_path: Option<String>,

    /// Rendering time in milliseconds
    pub render_time_ms: u64,
}

/// Graph metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetrics {
    /// Number of nodes in the graph
    pub node_count: usize,

    /// Number of edges in the graph
    pub edge_count: usize,

    /// Last layout computation time in milliseconds
    pub layout_time_ms: u64,

    /// Graph density (0.0 to 1.0)
    pub density: f64,

    /// Optional additional metrics
    #[serde(default)]
    pub additional: HashMap<String, serde_json::Value>,
}

/// Primal endpoint information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalEndpoint {
    /// Primal name or identifier
    pub name: String,

    /// Capabilities provided
    pub capabilities: Vec<String>,

    /// Socket path for Unix socket connections
    #[serde(skip_serializing_if = "Option::is_none")]
    pub socket_path: Option<String>,

    /// HTTP endpoint (deprecated, for fallback only)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_endpoint: Option<String>,

    /// Protocol support: "tarpc", "jsonrpc", "http"
    pub protocols: Vec<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_request_serialization() {
        let request = RenderRequest {
            mode: "svg".to_string(),
            data: serde_json::json!({"nodes": [], "edges": []}),
            width: Some(1920),
            height: Some(1080),
            output_path: Some("/tmp/output.svg".to_string()),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"mode\":\"svg\""));
        assert!(json.contains("\"width\":1920"));
    }

    #[test]
    fn test_health_status_deserialization() {
        let json = r#"{
            "status": "healthy",
            "version": "1.3.0",
            "uptime_secs": 3600,
            "connections": 5,
            "metrics": {}
        }"#;

        let health: PetalTongueHealthResponse = serde_json::from_str(json).unwrap();
        assert_eq!(health.status, "healthy");
        assert_eq!(health.version, "1.3.0");
        assert_eq!(health.uptime_secs, 3600);
        assert_eq!(health.connections, 5);
    }

    #[test]
    fn test_primal_endpoint_deserialization() {
        let json = r#"{
            "name": "petaltongue-main",
            "capabilities": ["visualization", "graph-compute"],
            "socket_path": "/run/user/1000/petaltongue-main.sock",
            "protocols": ["tarpc", "jsonrpc"]
        }"#;

        let endpoint: PrimalEndpoint = serde_json::from_str(json).unwrap();
        assert_eq!(endpoint.name, "petaltongue-main");
        assert_eq!(endpoint.capabilities.len(), 2);
        assert!(endpoint.protocols.contains(&"tarpc".to_string()));
    }

    // Additional unit tests for PetalTongueClient types

    #[test]
    fn test_render_request_structure() {
        let request = RenderRequest {
            mode: "terminal".to_string(),
            data: serde_json::json!({"nodes": [], "edges": []}),
            width: None,
            height: None,
            output_path: None,
        };

        assert_eq!(request.mode, "terminal");
        assert!(request.width.is_none());
    }

    #[test]
    fn test_render_response_types() {
        // Success case
        let success = RenderResponse {
            success: true,
            data: Some(serde_json::json!("rendered")),
            output_path: Some("/tmp/out.png".to_string()),
            render_time_ms: 42,
        };
        assert!(success.success);

        // Error case
        let error = RenderResponse {
            success: false,
            data: None,
            output_path: None,
            render_time_ms: 0,
        };
        assert!(!error.success);
    }

    #[test]
    fn test_all_modalities() {
        let modalities = vec!["terminal", "svg", "png", "json", "dot"];
        for modality in modalities {
            let req = RenderRequest {
                mode: modality.to_string(),
                data: serde_json::json!({}),
                width: None,
                height: None,
                output_path: None,
            };
            assert!(!req.mode.is_empty());
        }
    }
}
