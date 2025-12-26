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

use crate::clients::base::PrimalHttpClient;
use crate::primal_client::{HealthStatus, PrimalClient};
use anyhow::Result;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// Songbird discovery and coordination client
///
/// # Example
/// ```no_run
/// use biomeos_core::clients::songbird::SongbirdClient;
///
/// #[tokio::main]
/// async fn main() -> anyhow::Result<()> {
///     let songbird = SongbirdClient::new("http://localhost:3000");
///
///     // Discover compute services
///     let services = songbird.discover_by_capability("compute").await?;
///     for service in services {
///         println!("Found: {} at {}", service.service_name, service.endpoint);
///     }
///
///     Ok(())
/// }
/// ```
#[derive(Debug, Clone)]
pub struct SongbirdClient {
    http: PrimalHttpClient,
    endpoint: String,
}

impl SongbirdClient {
    /// Create a new Songbird client
    ///
    /// # Arguments
    /// * `endpoint` - Songbird endpoint URL (e.g., `http://localhost:3000`)
    pub fn new(endpoint: impl Into<String>) -> Self {
        let endpoint = endpoint.into();
        Self {
            http: PrimalHttpClient::new(&endpoint),
            endpoint,
        }
    }

    /// Discover services by capability
    ///
    /// Query Songbird for all services that provide a specific capability.
    ///
    /// # Arguments
    /// * `capability` - Capability to search for (e.g., "compute", "storage", "ai")
    ///
    /// # Errors
    /// Returns an error if the request fails or the response cannot be parsed.
    ///
    /// # Example
    /// ```no_run
    /// # use biomeos_core::clients::songbird::SongbirdClient;
    /// # #[tokio::main]
    /// # async fn main() -> anyhow::Result<()> {
    /// let songbird = SongbirdClient::new("http://localhost:3000");
    /// let compute_services = songbird.discover_by_capability("compute").await?;
    /// # Ok(())
    /// # }
    /// ```
    pub async fn discover_by_capability(&self, capability: &str) -> Result<Vec<ServiceInfo>> {
        let response = self
            .http
            .get(&format!("/api/v1/services/query/{}", capability))
            .await?;

        serde_json::from_value(response)
            .map_err(|e| anyhow::anyhow!("Failed to parse service list: {}", e))
    }

    /// Register a service with Songbird
    ///
    /// # Arguments
    /// * `service` - Service registration information
    ///
    /// # Returns
    /// The service ID assigned by Songbird
    ///
    /// # Errors
    /// Returns an error if registration fails.
    pub async fn register_service(&self, service: &ServiceRegistration) -> Result<String> {
        let response = self
            .http
            .post("/api/v1/services/register", serde_json::to_value(service)?)
            .await?;

        response["service_id"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| anyhow::anyhow!("No service_id in registration response"))
    }

    /// Get health status for a specific service
    ///
    /// # Arguments
    /// * `service_id` - Service ID to check
    ///
    /// # Errors
    /// Returns an error if the request fails.
    pub async fn get_service_health(&self, service_id: &str) -> Result<HealthStatus> {
        let response = self
            .http
            .get(&format!("/api/health/{}", service_id))
            .await?;

        Ok(HealthStatus {
            healthy: response["status"] == "healthy",
            message: response["message"]
                .as_str()
                .unwrap_or("Unknown")
                .to_string(),
            details: Some(response),
        })
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
    /// # Arguments
    /// * `latitude` - Latitude coordinate
    /// * `longitude` - Longitude coordinate
    /// * `radius_km` - Search radius in kilometers
    ///
    /// # Errors
    /// Returns an error if the discovery request fails.
    pub async fn discover_by_location(
        &self,
        latitude: f64,
        longitude: f64,
        radius_km: f64,
    ) -> Result<Vec<ServiceInfo>> {
        // Get all services
        let response = self.http.get("/api/v1/services/all").await?;
        let all_services: Vec<ServiceInfo> = serde_json::from_value(response)
            .map_err(|e| anyhow::anyhow!("Failed to parse service list: {}", e))?;

        // Filter by location
        Ok(all_services
            .into_iter()
            .filter(|s| {
                if let Some(loc) = &s.metadata.location {
                    loc.distance_to(latitude, longitude) <= radius_km
                } else {
                    false
                }
            })
            .collect())
    }
}

#[async_trait]
impl PrimalClient for SongbirdClient {
    fn name(&self) -> &str {
        "songbird"
    }

    fn endpoint(&self) -> &str {
        &self.endpoint
    }

    async fn is_available(&self) -> bool {
        self.health_check().await.is_ok()
    }

    async fn health_check(&self) -> Result<HealthStatus> {
        let response = self.http.get("/health").await?;
        Ok(HealthStatus {
            healthy: response["status"] == "healthy",
            message: response["message"]
                .as_str()
                .unwrap_or("Unknown")
                .to_string(),
            details: Some(response),
        })
    }

    async fn request(&self, method: &str, path: &str, body: Option<Value>) -> Result<Value> {
        match method {
            "GET" => self.http.get(path).await,
            "POST" => self.http.post(path, body.unwrap_or(Value::Null)).await,
            _ => anyhow::bail!("Unsupported method: {}", method),
        }
    }
}

/// Service information from Songbird
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

    #[test]
    fn test_songbird_client_creation() {
        let client = SongbirdClient::new("http://localhost:3000");
        assert_eq!(client.name(), "songbird");
        assert_eq!(client.endpoint(), "http://localhost:3000");
    }
}
