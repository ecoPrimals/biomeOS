// SPDX-License-Identifier: APGL-3.0-or-later WITH Sovran-Exemption-1.0
//
// Copyright 2025 ecoPrimals Project
// Licensed under the Affero General Public License v3.0 or later with Sovran Exemption 1.0.
// See LICENSE file in the project root or visit https://www.gnu.org/licenses/agpl-3.0.html

//! Songbird client for service discovery and coordination
//!
//! Songbird is the discovery and service mesh primal. It provides:
//! - Service registration and discovery
//! - Capability-based service queries
//! - Health monitoring
//! - Service metadata management
//!
//! # Transport Evolution
//!
//! **NEW**: Auto-discovery via Unix socket (JSON-RPC 2.0)
//! - **PRIMARY**: JSON-RPC over Unix socket (100x faster, secure)
//! - **FALLBACK**: HTTP REST API (deprecated, legacy only)
//!
//! # Quick Start
//!
//! ```ignore
//! use biomeos_core::clients::songbird::SongbirdClient;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     // Auto-discover via Unix socket
//!     let songbird = SongbirdClient::discover("nat0").await?;
//!
//!     // Discover compute services
//!     let services = songbird.discover_by_capability("compute").await?;
//!     for service in services {
//!         println!("Found: {} at {}", service.service_name, service.endpoint);
//!     }
//!
//!     Ok(())
//! }
//! ```ignore

use crate::clients::transport::{TransportClient, TransportPreference};
use crate::primal_client::{HealthStatus, PrimalClient};
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Songbird discovery and coordination client
///
/// Uses JSON-RPC 2.0 over Unix sockets for fast, secure communication.
///
/// # Example
/// ```ignore
/// use biomeos_core::clients::songbird::SongbirdClient;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     // Auto-discover via Unix socket
///     let songbird = SongbirdClient::discover("nat0").await?;
///
///     // Discover compute services
///     let services = songbird.discover_by_capability("compute").await?;
///     for service in services {
///         println!("Found: {} at {}", service.service_name, service.endpoint);
///     }
///
///     Ok(())
/// }
/// ```ignore
#[derive(Debug, Clone)]
pub struct SongbirdClient {
    transport: TransportClient,
    family_id: String,
}

impl SongbirdClient {
    /// Auto-discover Songbird via Unix socket
    ///
    /// Searches for Songbird's Unix socket in XDG runtime directory.
    /// Falls back to HTTP if Unix socket not available.
    ///
    /// # Arguments
    /// * `family_id` - Genetic family ID (e.g., "nat0")
    ///
    /// # Returns
    /// SongbirdClient configured with JSON-RPC over Unix socket (primary)
    /// or HTTP (fallback)
    ///
    /// # Example
    /// ```no_run
    /// use biomeos_core::clients::songbird::SongbirdClient;
    ///
    /// #[tokio::main]
    /// async fn main() -> anyhow::Result<()> {
    ///     let songbird = SongbirdClient::discover("nat0").await?;
    ///     let services = songbird.discover_by_capability("compute").await?;
    ///     Ok(())
    /// }
    /// ```
    pub async fn discover(family_id: &str) -> Result<Self> {
        let transport = TransportClient::discover_with_preference(
            "songbird",
            family_id,
            TransportPreference::UnixSocket,
        )
        .await
        .context("Failed to discover Songbird. Is it running?")?;

        Ok(Self {
            transport,
            family_id: family_id.to_string(),
        })
    }

    /// Create from explicit endpoint (HTTP fallback)
    ///
    /// **DEPRECATED**: Use `discover()` for Unix socket support (100x faster)
    ///
    /// # Arguments
    /// * `endpoint` - HTTP endpoint URL (e.g., "http://localhost:8080")
    /// * `family_id` - Genetic family ID
    #[deprecated(note = "Use SongbirdClient::discover() for Unix socket support")]
    pub async fn from_endpoint(endpoint: impl Into<String>, family_id: &str) -> Result<Self> {
        let _endpoint = endpoint.into();
        let transport = TransportClient::discover_with_preference(
            "songbird",
            family_id,
            TransportPreference::Auto, // ✅ Evolved: Auto-discover secure transport
        )
        .await
        .context("Failed to discover Songbird via secure transport")?;

        Ok(Self {
            transport,
            family_id: family_id.to_string(),
        })
    }

    /// Legacy constructor (DEPRECATED)
    ///
    /// **BREAKING**: This method is now async. Use `discover()` instead.
    #[deprecated(note = "Use SongbirdClient::discover() instead")]
    pub fn new(_endpoint: impl Into<String>) -> Self {
        panic!("SongbirdClient::new() is deprecated. Use SongbirdClient::discover() instead.");
    }

    /// Discover services by capability (v3.20.0)
    ///
    /// Query Songbird for all services that provide a specific capability.
    /// Uses Songbird's JSON-RPC API: `discover_by_capability`
    ///
    /// # Arguments
    /// * `capability` - Capability to search for (e.g., "compute", "storage", "ai", "encryption")
    ///
    /// # Errors
    /// Returns an error if the request fails or the response cannot be parsed.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::songbird::SongbirdClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let songbird = SongbirdClient::discover("nat0").await?;
    /// let compute_services = songbird.discover_by_capability("compute").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn discover_by_capability(&self, capability: &str) -> Result<Vec<ServiceInfo>> {
        let response = self
            .transport
            .call(
                "discover_by_capability", // NEW: Direct method name
                Some(serde_json::json!({
                    "capability": capability,
                    "protocol": "json-rpc"  // Optional protocol filter
                })),
            )
            .await
            .context("Failed to call discover_by_capability")?;

        // v3.20.0 returns {"primals": [...]}
        if let Some(primals) = response.get("primals") {
            let endpoints: Vec<PrimalEndpoint> = serde_json::from_value(primals.clone())
                .context("Failed to parse primal endpoints")?;

            // Convert PrimalEndpoint to ServiceInfo for backward compatibility
            Ok(endpoints
                .into_iter()
                .map(|p| ServiceInfo {
                    service_id: p.service_id,
                    service_name: p.primal_name,
                    endpoint: p.endpoint,
                    capabilities: p.capabilities,
                    metadata: ServiceMetadata {
                        version: "unknown".to_string(),
                        location: None,
                        tags: vec![],
                    },
                })
                .collect())
        } else {
            // Fallback for old format
            if let Some(peers) = response.get("peers") {
                serde_json::from_value(peers.clone())
                    .context("Failed to parse peer list from response")
            } else {
                Ok(vec![])
            }
        }
    }

    /// Register a service with Songbird (v3.20.0)
    ///
    /// Uses Songbird's JSON-RPC API: `register_service`
    ///
    /// # Arguments
    /// * `service` - Service registration information
    ///
    /// # Returns
    /// The service ID assigned by Songbird
    ///
    /// # Errors
    /// Returns an error if registration fails.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::songbird::{SongbirdClient, ServiceRegistration, ServiceMetadata};
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let songbird = SongbirdClient::discover("nat0").await?;
    /// let service_id = songbird.register_service(&ServiceRegistration {
    ///     service_name: "my-service".to_string(),
    ///     capabilities: vec!["compute".to_string()],
    ///     endpoint: "http://localhost:8081".to_string(),
    ///     metadata: ServiceMetadata {
    ///         version: "1.0.0".to_string(),
    ///         location: None,
    ///         tags: vec![],
    ///     },
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn register_service(&self, service: &ServiceRegistration) -> Result<String> {
        let response = self
            .transport
            .call(
                "register_service", // NEW: Direct method name
                Some(serde_json::json!({
                    "primal_name": service.service_name,
                    "capabilities": service.capabilities,
                    "endpoint": service.endpoint,
                    "protocol": "json-rpc",
                    "health_check_interval": 30
                })),
            )
            .await
            .context("Failed to call register_service")?;

        // v3.20.0 returns {"service_id": "...", "status": "...", "registered_at": "..."}
        response["service_id"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("No service_id in registration response"))
    }

    /// Get health status for a specific service (v3.20.0)
    ///
    /// Uses Songbird's JSON-RPC API: `get_service_health`
    ///
    /// # Arguments
    /// * `service_id` - Service ID to check
    ///
    /// # Errors
    /// Returns an error if the request fails.
    ///
    /// # Example
    /// ```ignore
    /// let songbird = SongbirdClient::discover("nat0").await?;
    /// let health = songbird.get_service_health("service-123").await?;
    /// println!("Service healthy: {}", health.healthy);
    /// ```
    pub async fn get_service_health(&self, service_id: &str) -> Result<HealthStatus> {
        let response = self
            .transport
            .call(
                "get_service_health", // NEW: Direct method name
                Some(serde_json::json!({
                    "service_id": service_id
                })),
            )
            .await
            .context("Failed to call get_service_health")?;

        // v3.20.0 returns {"health": {"service_id": "...", "status": "...", ...}}
        if let Some(health) = response.get("health") {
            let status = health["status"].as_str().unwrap_or("unknown");
            match status {
                "healthy" => Ok(HealthStatus::Healthy),
                "degraded" => Ok(HealthStatus::Degraded),
                "unhealthy" => Ok(HealthStatus::Unhealthy),
                _ => Ok(HealthStatus::Unknown),
            }
        } else {
            // Fallback for old format
            let status = response["status"].as_str().unwrap_or("unknown");
            match status {
                "healthy" => Ok(HealthStatus::Healthy),
                _ => Ok(HealthStatus::Unhealthy),
            }
        }
    }

    /// Query services with metadata filter
    ///
    /// Discover services by capability and then filter by metadata.
    ///
    /// # Arguments
    /// * `capability` - Capability to search for
    /// * `filter` - Function to filter services by metadata
    ///
    /// # Errors
    /// Returns an error if the discovery request fails.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::songbird::SongbirdClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let songbird = SongbirdClient::new("http://localhost:3000");
    ///
    /// // Find compute services with version 2.x
    /// let services = songbird.query_with_metadata("compute", |meta| {
    ///     meta.version.starts_with("2.")
    /// }).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn query_with_metadata<F>(
        &self,
        capability: &str,
        filter: F,
    ) -> Result<Vec<ServiceInfo>>
    where
        F: Fn(&ServiceMetadata) -> bool,
    {
        let all = self.discover_by_capability(capability).await?;
        Ok(all.into_iter().filter(|s| filter(&s.metadata)).collect())
    }

    /// Find services near a geographic location
    ///
    /// Uses Songbird's JSON-RPC API: `discovery.find_by_location`
    ///
    /// # Arguments
    /// * `latitude` - Latitude coordinate
    /// * `longitude` - Longitude coordinate
    /// * `radius_km` - Search radius in kilometers
    ///
    /// # Errors
    /// Returns an error if the discovery request fails.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::songbird::SongbirdClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let songbird = SongbirdClient::discover("nat0").await?;
    /// let nearby = songbird.discover_by_location(40.7128, -74.0060, 100.0).await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn discover_by_location(
        &self,
        latitude: f64,
        longitude: f64,
        radius_km: f64,
    ) -> Result<Vec<ServiceInfo>> {
        let response = self
            .transport
            .call(
                "discovery.find_by_location",
                Some(serde_json::json!({
                    "latitude": latitude,
                    "longitude": longitude,
                    "radius_km": radius_km,
                    "family_id": self.family_id
                })),
            )
            .await
            .context("Failed to call discovery.find_by_location")?;

        serde_json::from_value(response).context("Failed to parse service list from response")
    }
}

#[async_trait]
impl PrimalClient for SongbirdClient {
    fn name(&self) -> &str {
        "songbird"
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
        // For JSON-RPC, method becomes the RPC method name, path is ignored
        self.transport.call(method, body).await
    }
}

/// Primal endpoint information (v3.20.0)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct PrimalEndpoint {
    /// Unique service identifier
    pub service_id: String,

    /// Primal name (e.g., "BearDog")
    pub primal_name: String,

    /// Capabilities provided
    pub capabilities: Vec<String>,

    /// Endpoint (Unix socket or URL)
    pub endpoint: String,

    /// Protocol (json-rpc, tarpc, http)
    pub protocol: String,

    /// Last health check timestamp (ISO 8601)
    pub last_health_check: String,

    /// Health status (healthy, unhealthy, unknown)
    pub health_status: String,
}

/// Service information from Songbird (legacy)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceInfo {
    /// Unique service identifier
    pub service_id: String,

    /// Human-readable service name
    pub service_name: String,

    /// Service endpoint URL
    pub endpoint: String,

    /// Capabilities provided by this service
    pub capabilities: Vec<String>,

    /// Service metadata
    pub metadata: ServiceMetadata,
}

/// Response from discover_by_capability (v3.20.0)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct DiscoverByCapabilityResponse {
    /// Array of discovered primals
    pub primals: Vec<PrimalEndpoint>,
}

/// Response from register_service (v3.20.0)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct RegisterServiceResponse {
    /// Assigned service ID
    pub service_id: String,

    /// Status (registered or updated)
    pub status: String,

    /// Registration timestamp (ISO 8601)
    pub registered_at: String,
}

/// Response from get_service_health (v3.20.0)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceHealthResponse {
    /// Health status information
    pub health: ServiceHealthStatus,
}

/// Health status information (v3.20.0)
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceHealthStatus {
    /// Service identifier
    pub service_id: String,

    /// Status (healthy, unhealthy, unknown)
    pub status: String,

    /// Optional status message
    pub message: Option<String>,

    /// Timestamp (ISO 8601)
    pub timestamp: String,
}

/// Service metadata
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct ServiceMetadata {
    /// Service version
    pub version: String,

    /// Geographic location (if applicable)
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub location: Option<Location>,

    /// Service tags for filtering
    #[serde(default)]
    pub tags: Vec<String>,
}

/// Geographic location
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct Location {
    /// Latitude coordinate
    pub latitude: f64,

    /// Longitude coordinate
    pub longitude: f64,
}

impl Location {
    /// Calculate distance to another location using Haversine formula
    ///
    /// # Arguments
    /// * `lat` - Target latitude
    /// * `lon` - Target longitude
    ///
    /// # Returns
    /// Distance in kilometers
    pub fn distance_to(&self, lat: f64, lon: f64) -> f64 {
        const EARTH_RADIUS_KM: f64 = 6371.0;

        let d_lat = (lat - self.latitude).to_radians();
        let d_lon = (lon - self.longitude).to_radians();

        let a = (d_lat / 2.0).sin().powi(2)
            + self.latitude.to_radians().cos()
                * lat.to_radians().cos()
                * (d_lon / 2.0).sin().powi(2);

        let c = 2.0 * a.sqrt().atan2((1.0 - a).sqrt());
        EARTH_RADIUS_KM * c
    }
}

/// Service registration request
#[derive(Debug, Clone, Serialize)]
pub struct ServiceRegistration {
    /// Service name
    pub service_name: String,

    /// Capabilities provided
    pub capabilities: Vec<String>,

    /// Service endpoint
    pub endpoint: String,

    /// Service metadata
    pub metadata: ServiceMetadata,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_location_distance() {
        let loc1 = Location {
            latitude: 40.7128,
            longitude: -74.0060,
        }; // New York
        let loc2_lat = 51.5074;
        let loc2_lon = -0.1278; // London

        let distance = loc1.distance_to(loc2_lat, loc2_lon);
        // Distance should be approximately 5570 km
        assert!(distance > 5500.0 && distance < 5600.0);
    }

    /// Integration test using harvested binary from plasmidBin/
    ///
    /// Start Songbird manually:
    /// ```bash
    /// ./plasmidBin/primals/songbird-orchestrator --family nat0
    /// ```
    #[ignore = "Requires running Songbird from plasmidBin/primals/songbird-orchestrator"]
    #[tokio::test]
    async fn test_songbird_client_creation() {
        let client = SongbirdClient::discover("nat0").await.unwrap();
        assert_eq!(client.name(), "songbird");
    }
}
