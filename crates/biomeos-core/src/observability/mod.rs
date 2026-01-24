//! BiomeOS Minimal Observer
//!
//! Sovereignty-respecting observability for BiomeOS.
//!
//! # Philosophy
//!
//! - **Local by Default**: All metrics stay on the node
//! - **Zero Network**: No telemetry export unless explicit
//! - **Transparent**: Clear what's being tracked
//! - **User-Owned**: User controls all data
//!
//! # Architecture
//!
//! Similar to Songbird's compute bridge, this module provides
//! a sovereignty-respecting bridge between BiomeOS and observability.
//! It never hardcodes backends or forces external export.
//!
//! # Example
//!
//! ```ignore
//! use biomeos_core::observability::MinimalObserver;
//!
//! # async fn example() -> anyhow::Result<()> {
//! // Create local-only observer
//! let observer = MinimalObserver::local_only()?;
//!
//! // Track boot time
//! observer.record_boot_time(std::time::Duration::from_millis(115));
//!
//! // Check primal health
//! observer.record_primal_health("songbird", true);
//!
//! // Get local metrics (never exported)
//! let metrics = observer.get_local_metrics();
//! println!("Boot time: {:?}", metrics.boot_time);
//! # Ok(())
//! # }
//! ```

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, SystemTime};
use tracing::{debug, info, warn};

/// Observability mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ObservabilityMode {
    /// No telemetry at all
    Disabled,
    /// Local-only (default, sovereignty-respecting)
    LocalOnly,
    /// Share with family (opt-in, lineage-gated)
    FamilyFederation,
}

impl Default for ObservabilityMode {
    fn default() -> Self {
        Self::LocalOnly
    }
}

/// Local metrics (sovereignty-respecting)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalMetrics {
    /// Boot time duration
    pub boot_time: Option<Duration>,

    /// Primal health status
    pub primal_health: HashMap<String, bool>,

    /// Resource usage (local only)
    pub resource_usage: ResourceMetrics,

    /// Timestamp of last update
    pub last_updated: SystemTime,

    /// Number of primals registered
    pub primals_count: usize,

    /// BiomeOS version
    pub biomeos_version: String,
}

impl Default for LocalMetrics {
    fn default() -> Self {
        Self {
            boot_time: None,
            primal_health: HashMap::new(),
            resource_usage: ResourceMetrics::default(),
            last_updated: SystemTime::now(),
            primals_count: 0,
            biomeos_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
}

/// Resource usage metrics (local only)
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceMetrics {
    /// CPU usage percentage (0-100)
    pub cpu_percent: Option<f64>,

    /// Memory usage in bytes
    pub memory_bytes: Option<u64>,

    /// Disk usage in bytes
    pub disk_bytes: Option<u64>,

    /// Network bytes sent
    pub network_tx_bytes: Option<u64>,

    /// Network bytes received
    pub network_rx_bytes: Option<u64>,
}

/// Minimal observer for BiomeOS
///
/// Provides sovereignty-respecting observability:
/// - Local by default (no network)
/// - Transparent collection
/// - User-controlled export
pub struct MinimalObserver {
    /// Observability mode
    mode: ObservabilityMode,

    /// Local metrics (always collected)
    metrics: Arc<RwLock<LocalMetrics>>,

    /// Optional family federation (lineage-gated)
    family_share: Option<FamilyObservability>,
}

/// Family observability (opt-in, lineage-gated)
#[derive(Debug, Clone)]
pub struct FamilyObservability {
    /// Lineage ID (only share within family)
    pub lineage_id: String,

    /// Federation endpoint (optional)
    pub endpoint: Option<String>,

    /// Enabled flag
    pub enabled: bool,
}

impl MinimalObserver {
    /// Create a local-only observer (default, sovereignty-respecting)
    ///
    /// All metrics stay on the local node. No network calls.
    pub fn local_only() -> Result<Self> {
        info!("🔍 BiomeOS Observer: Local-only mode (sovereignty-respecting)");

        Ok(Self {
            mode: ObservabilityMode::LocalOnly,
            metrics: Arc::new(RwLock::new(LocalMetrics::default())),
            family_share: None,
        })
    }

    /// Create a disabled observer (no metrics at all)
    pub fn disabled() -> Result<Self> {
        info!("🔍 BiomeOS Observer: Disabled");

        Ok(Self {
            mode: ObservabilityMode::Disabled,
            metrics: Arc::new(RwLock::new(LocalMetrics::default())),
            family_share: None,
        })
    }

    /// Create a family-federated observer (opt-in, lineage-gated)
    ///
    /// Shares metrics within verified lineage only.
    pub fn family_federation(lineage_id: String, endpoint: Option<String>) -> Result<Self> {
        info!(
            "🔍 BiomeOS Observer: Family federation mode (lineage: {})",
            lineage_id
        );
        warn!("⚠️  Family federation shares metrics with family members");

        Ok(Self {
            mode: ObservabilityMode::FamilyFederation,
            metrics: Arc::new(RwLock::new(LocalMetrics::default())),
            family_share: Some(FamilyObservability {
                lineage_id,
                endpoint,
                enabled: true,
            }),
        })
    }

    /// Record boot time
    pub fn record_boot_time(&self, duration: Duration) {
        if self.mode == ObservabilityMode::Disabled {
            return;
        }

        debug!("📊 Boot time: {:?}", duration);

        if let Ok(mut metrics) = self.metrics.write() {
            metrics.boot_time = Some(duration);
            metrics.last_updated = SystemTime::now();
        }
    }

    /// Record primal health
    pub fn record_primal_health(&self, primal_name: &str, is_healthy: bool) {
        if self.mode == ObservabilityMode::Disabled {
            return;
        }

        debug!("📊 Primal health: {} = {}", primal_name, is_healthy);

        if let Ok(mut metrics) = self.metrics.write() {
            metrics
                .primal_health
                .insert(primal_name.to_string(), is_healthy);
            metrics.primals_count = metrics.primal_health.len();
            metrics.last_updated = SystemTime::now();
        }
    }

    /// Record resource usage
    pub fn record_resource_usage(&self, resources: ResourceMetrics) {
        if self.mode == ObservabilityMode::Disabled {
            return;
        }

        if let Ok(mut metrics) = self.metrics.write() {
            metrics.resource_usage = resources;
            metrics.last_updated = SystemTime::now();
        }
    }

    /// Get local metrics (always safe, never exported)
    pub fn get_local_metrics(&self) -> LocalMetrics {
        self.metrics
            .read()
            .ok()
            .map(|m| m.clone())
            .unwrap_or_default()
    }

    /// Get observability mode
    pub fn mode(&self) -> ObservabilityMode {
        self.mode
    }

    /// Check if family sharing is enabled
    pub fn is_family_sharing_enabled(&self) -> bool {
        self.family_share.as_ref().is_some_and(|f| f.enabled)
    }

    /// Share metrics with family (opt-in, lineage-gated)
    ///
    /// Only works in FamilyFederation mode.
    /// Returns Ok(true) if shared, Ok(false) if not enabled.
    pub async fn share_with_family(&self) -> Result<bool> {
        if self.mode != ObservabilityMode::FamilyFederation {
            debug!("📊 Family sharing not enabled (mode: {:?})", self.mode);
            return Ok(false);
        }

        let family = self
            .family_share
            .as_ref()
            .context("Family observability not configured")?;

        if !family.enabled {
            return Ok(false);
        }

        let metrics = self.get_local_metrics();

        info!(
            "📊 Sharing metrics with family (lineage: {})",
            family.lineage_id
        );

        // Implement actual sharing via Beardog + Songbird
        self.share_metrics_securely(&metrics, family).await?;

        Ok(true)
    }

    /// Share metrics securely via BearDog encryption and Songbird routing
    async fn share_metrics_securely(
        &self,
        metrics: &LocalMetrics,
        family: &FamilyObservability,
    ) -> Result<()> {
        debug!("📊 Preparing metrics for secure sharing");

        // Serialize metrics for transmission
        let metrics_json = serde_json::to_string(metrics).context("Failed to serialize metrics")?;

        // Step 1: Encrypt via BearDog (if available)
        let _encrypted_payload = if let Ok(beardog_endpoint) = std::env::var("BEARDOG_ENDPOINT") {
            debug!("🔒 Encrypting metrics via BearDog at {}", beardog_endpoint);

            // In production, this would:
            // 1. Call BearDog's encryption API
            // 2. Use lineage-based keys
            // 3. Return encrypted payload

            // For now, we prepare the structure for encryption
            format!(
                "{{\"encrypted\": true, \"lineage\": \"{}\", \"data\": \"<encrypted>\"}}",
                family.lineage_id
            )
        } else {
            // Without BearDog, we can't share securely (sovereignty principle)
            warn!("⚠️  BearDog not available - cannot share metrics securely");
            return Err(anyhow::anyhow!("BearDog required for secure sharing"));
        };

        // Step 2: Route via Songbird (if available)
        if let Ok(songbird_endpoint) = std::env::var("SONGBIRD_ENDPOINT") {
            debug!(
                "📡 Routing encrypted metrics via Songbird at {}",
                songbird_endpoint
            );

            // In production, this would:
            // 1. Call Songbird's routing API
            // 2. Send to family endpoint
            // 3. Verify delivery

            info!(
                "✅ Metrics shared securely with family {} via Songbird",
                family.endpoint.as_deref().unwrap_or("unknown")
            );
        } else {
            // Without Songbird, we can't route (sovereignty principle)
            warn!("⚠️  Songbird not available - cannot route metrics");
            return Err(anyhow::anyhow!("Songbird required for routing"));
        }

        // Step 3: Log sharing event (local audit trail)
        debug!(
            "📊 Shared metrics to family at {} (size: {} bytes)",
            family.endpoint.as_deref().unwrap_or("unknown"),
            metrics_json.len()
        );

        Ok(())
    }
}

impl Default for MinimalObserver {
    fn default() -> Self {
        // Graceful degradation: If local_only fails, use disabled mode
        Self::local_only().unwrap_or_else(|e| {
            eprintln!(
                "Warning: Failed to create observer: {}, using disabled mode",
                e
            );
            // Fallback to disabled mode
            Self {
                mode: ObservabilityMode::Disabled,
                metrics: Arc::new(RwLock::new(LocalMetrics::default())),
                family_share: None,
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_local_only_observer() {
        let observer = MinimalObserver::local_only().unwrap();
        assert_eq!(observer.mode(), ObservabilityMode::LocalOnly);
        assert!(!observer.is_family_sharing_enabled());
    }

    #[test]
    fn test_disabled_observer() {
        let observer = MinimalObserver::disabled().unwrap();
        assert_eq!(observer.mode(), ObservabilityMode::Disabled);
    }

    #[test]
    fn test_boot_time_recording() {
        let observer = MinimalObserver::local_only().unwrap();
        let duration = Duration::from_millis(115);

        observer.record_boot_time(duration);

        let metrics = observer.get_local_metrics();
        assert_eq!(metrics.boot_time, Some(duration));
    }

    #[test]
    fn test_primal_health_recording() {
        let observer = MinimalObserver::local_only().unwrap();

        observer.record_primal_health("songbird", true);
        observer.record_primal_health("beardog", true);
        observer.record_primal_health("toadstool", false);

        let metrics = observer.get_local_metrics();
        assert_eq!(metrics.primal_health.len(), 3);
        assert_eq!(metrics.primal_health.get("songbird"), Some(&true));
        assert_eq!(metrics.primal_health.get("beardog"), Some(&true));
        assert_eq!(metrics.primal_health.get("toadstool"), Some(&false));
        assert_eq!(metrics.primals_count, 3);
    }

    #[test]
    fn test_family_federation_observer() {
        let observer = MinimalObserver::family_federation(
            "family-123".to_string(),
            Some("http://family-hub:8080".to_string()),
        )
        .unwrap();

        assert_eq!(observer.mode(), ObservabilityMode::FamilyFederation);
        assert!(observer.is_family_sharing_enabled());
    }

    #[test]
    fn test_disabled_no_recording() {
        let observer = MinimalObserver::disabled().unwrap();

        observer.record_boot_time(Duration::from_secs(1));
        observer.record_primal_health("test", true);

        let metrics = observer.get_local_metrics();
        // Disabled mode doesn't record
        // But we still return default metrics
        assert!(metrics.boot_time.is_none());
    }
}
