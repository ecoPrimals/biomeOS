//! BiomeOS UI Components & Integration
//!
//! This crate provides comprehensive UI components and backend integration 
//! for biomeOS using the unified type system from biomeos-types.
//!
//! ## Features
//! 
//! - **Live Backend Integration**: Real-time system monitoring and control
//! - **Unified Types**: Uses biomeos-types for consistency across the ecosystem
//! - **Modern Architecture**: Async-first, event-driven UI backend
//! - **Comprehensive Metrics**: System health, resource usage, and performance
//! - **YAML Management**: Create, edit, and validate biome configuration files
//! - **Error Handling**: Integrated with unified BiomeError system

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{RwLock, mpsc};
use anyhow::Result;

// Re-export unified types from biomeos-types with specific paths where needed
pub use biomeos_types::{
    // Core error and result types
    BiomeError,
    BiomeResult,
    
    // Health system - use specific path to avoid ambiguity
    Health,
    HealthReport,
    health::HealthMetrics,
    ComponentHealth,
    
    // Primal system
    PrimalType,
    PrimalCapability,
    PrimalConfiguration,
    
    // Resource and metrics
    ResourceMetrics,
    NetworkIoMetrics,
    AvailabilityMetrics,
    
    // Configuration
    BiomeOSConfig,
    SystemConfig,
    
    // Manifest types - use specific path to avoid ambiguity
    BiomeManifest,
    manifest::ServiceSpec,
    service::ServiceMetadata,
    
    // Service types
    ServiceStatus,
    
    // Health subjects
    HealthSubject,
    HealthSubjectType,
};

// Re-export backend components
pub mod backend;
pub use backend::*;

// UI Component modules
pub mod components;
pub mod metrics;
pub mod events;
pub mod validation;

// Re-export key UI types and traits
pub use components::*;
pub use metrics::*;
pub use events::*;
pub use validation::*;

/// UI Event types for the biomeos-ui backend
#[derive(Debug, Clone)]
pub enum UIEvent {
    /// System status has been updated
    SystemStatusUpdated {
        health: Health,
        metrics: ResourceMetrics,
        timestamp: chrono::DateTime<chrono::Utc>,
    },
    /// Service status has changed
    ServiceStatusChanged {
        old_status: ServiceStatus,
        new_status: ServiceStatus,
        service_id: String,
    },
    /// Configuration has been updated
    ConfigurationUpdated {
        content: String,
        is_valid: bool,
    },
}

/// Main UI Controller that orchestrates backend and frontend components
pub struct UIController {
    backend: Arc<LiveBackend>,
    event_sender: mpsc::UnboundedSender<UIEvent>,
    event_receiver: RwLock<Option<mpsc::UnboundedReceiver<UIEvent>>>,
    health_cache: RwLock<Option<HealthReport>>,
}

impl UIController {
    /// Create a new UI controller with live backend integration
    pub async fn new() -> Result<Arc<Self>> {
        let backend = LiveBackend::new().await?;
        let (event_sender, event_receiver) = mpsc::unbounded_channel();
        
        let controller = Arc::new(Self {
            backend,
            event_sender,
            event_receiver: RwLock::new(Some(event_receiver)),
            health_cache: RwLock::new(None),
        });
        
        // Start background monitoring
        let controller_clone = controller.clone();
        tokio::spawn(async move {
            controller_clone.start_monitoring_loop().await;
        });
        
        Ok(controller)
    }
    
    /// Get current system health report
    pub async fn get_health_report(&self) -> Result<HealthReport> {
        // Try cache first
        if let Some(cached) = self.health_cache.read().await.as_ref() {
            // Return cached if less than 30 seconds old
            let age = chrono::Utc::now().signed_duration_since(cached.generated_at);
            if age < chrono::Duration::seconds(30) {
                return Ok(cached.clone());
            }
        }
        
        // Fetch fresh health report
        let status = self.backend.get_system_status().await?;
        let health_report = self.convert_status_to_health_report(status).await?;
        
        // Update cache
        *self.health_cache.write().await = Some(health_report.clone());
        
        Ok(health_report)
    }
    
    /// Get current resource metrics
    pub async fn get_resource_metrics(&self) -> Result<ResourceMetrics> {
        let metrics = self.backend.refresh_and_get_metrics().await?;
        Ok(ResourceMetrics {
            cpu_usage: Some(metrics.cpu_usage),
            memory_usage: Some(metrics.memory_usage),
            disk_usage: None, // Would need additional implementation
            network_io: Some(NetworkIoMetrics {
                bytes_in_per_sec: 0.0,  // Placeholder
                bytes_out_per_sec: 0.0, // Placeholder  
                packets_in_per_sec: 0.0,
                packets_out_per_sec: 0.0,
            }),
        })
    }
    
    /// Get discovered primals
    pub async fn get_discovered_primals(&self) -> Vec<String> {
        self.backend.get_discovered_primals().await
    }
    
    /// Load and validate a biome configuration
    pub async fn load_biome_config(&self, content: &str) -> Result<BiomeManifest> {
        // Use the unified manifest system from biomeos-manifest
        let manifest = biomeos_manifest::BiomeManifestProcessor::load_from_yaml(content)?;
        Ok(manifest)
    }
    
    /// Save a biome configuration  
    pub async fn save_biome_config(&self, manifest: &BiomeManifest, file_name: &str) -> Result<()> {
        let yaml_content = biomeos_manifest::BiomeManifestProcessor::save_to_yaml(manifest)?;
        self.backend.update_yaml_content(file_name, &yaml_content).await?;
        
        // Send event
        self.event_sender.send(UIEvent::ConfigurationUpdated {
            content: yaml_content,
            is_valid: true,
        }).map_err(|_| BiomeError::internal_error("Failed to send configuration event", None::<String>))?;
        
        Ok(())
    }
    
    /// Validate YAML configuration content
    pub async fn validate_config(&self, content: &str) -> Result<ValidationResult> {
        match biomeos_manifest::BiomeManifestProcessor::load_from_yaml(content) {
            Ok(manifest) => {
                // Additional validation using ManifestValidator
                match biomeos_manifest::BiomeManifestProcessor::validate(&manifest) {
                    Ok(_) => Ok(ValidationResult {
                        is_valid: true,
                        errors: vec![],
                        warnings: vec![],
                        manifest: Some(manifest),
                    }),
                    Err(e) => Ok(ValidationResult {
                        is_valid: false,
                        errors: vec![format!("Validation error: {}", e)],
                        warnings: vec![],
                        manifest: None,
                    }),
                }
            }
            Err(e) => Ok(ValidationResult {
                is_valid: false,
                errors: vec![format!("YAML parsing error: {}", e)],
                warnings: vec![],
                manifest: None,
            }),
        }
    }
    
    /// Take the event receiver for UI event handling
    pub async fn take_event_receiver(&self) -> Result<mpsc::UnboundedReceiver<UIEvent>> {
        let mut receiver_lock = self.event_receiver.write().await;
        receiver_lock.take().ok_or_else(|| 
            anyhow::anyhow!("Event receiver already taken")
        )
    }
    
    /// Start the monitoring loop
    async fn start_monitoring_loop(&self) {
        let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(10));
        
        loop {
            interval.tick().await;
            
            // Get fresh system status
            if let Ok(status) = self.backend.get_system_status().await {
                // Convert to health report and cache
                if let Ok(health_report) = self.convert_status_to_health_report(status).await {
                    *self.health_cache.write().await = Some(health_report.clone());
                    
                    // Send UI event
                    let _ = self.event_sender.send(UIEvent::SystemStatusUpdated {
                        health: health_report.health,
                        metrics: health_report.metrics.resources.unwrap_or_else(|| ResourceMetrics {
                            cpu_usage: Some(0.0),
                            memory_usage: Some(0.0),
                            disk_usage: Some(0.0),
                            network_io: Some(NetworkIoMetrics {
                                bytes_in_per_sec: 0.0,
                                bytes_out_per_sec: 0.0,
                                packets_in_per_sec: 0.0,
                                packets_out_per_sec: 0.0,
                            }),
                        }),
                        timestamp: health_report.generated_at,
                    });
                }
            }
        }
    }
    
    /// Convert system status to health report
    async fn convert_status_to_health_report(&self, status: biomeos_core::integration::SystemStatus) -> Result<HealthReport> {
        use biomeos_types::{HealthSubject, HealthSubjectType, health::HealthMetrics};
        
        let health = if status.resource_usage.cpu_usage.unwrap_or(0.0) > 0.9 ||
                        status.resource_usage.memory_usage.unwrap_or(0.0) > 0.9 {
            Health::Critical {
                issues: vec![biomeos_types::HealthIssue {
                    id: format!("system-critical-{}", chrono::Utc::now().timestamp()),
                    category: biomeos_types::HealthIssueCategory::Resource,
                    severity: biomeos_types::HealthIssueSeverity::Critical,
                    message: "System resources critically high".to_string(),
                    detected_at: chrono::Utc::now(),
                    details: HashMap::new(),
                    remediation: vec![],
                }],
                affected_capabilities: vec!["compute".to_string()],
            }
        } else if status.resource_usage.cpu_usage.unwrap_or(0.0) > 0.7 ||
                  status.resource_usage.memory_usage.unwrap_or(0.0) > 0.7 {
            Health::Degraded {
                issues: vec![biomeos_types::HealthIssue {
                    id: format!("system-degraded-{}", chrono::Utc::now().timestamp()),
                    category: biomeos_types::HealthIssueCategory::Performance,
                    severity: biomeos_types::HealthIssueSeverity::Medium,
                    message: "System resources elevated".to_string(),
                    detected_at: chrono::Utc::now(),
                    details: HashMap::new(),
                    remediation: vec![],
                }],
                impact_score: Some(0.3),
            }
        } else {
            Health::Healthy
        };
        
        let subject = HealthSubject {
            id: "biomeos-system".to_string(),
            subject_type: HealthSubjectType::System,
            name: "BiomeOS System".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
        };
        
        let metrics = HealthMetrics {
            response_time: None,
            resources: Some(status.resource_usage),
            errors: None,
            availability: Some(AvailabilityMetrics {
                uptime_percentage: 0.999,
                uptime_seconds: status.uptime.num_seconds() as u64,
                downtime_seconds: 0,
                outage_count: 0,
                mttr_seconds: None,
            }),
            custom: HashMap::new(),
        };
        
        Ok(HealthReport {
            id: uuid::Uuid::new_v4(),
            subject,
            health,
            components: HashMap::new(), // Would be populated with component details
            metrics,
            history: vec![],
            generated_at: chrono::Utc::now(),
            next_check_at: Some(chrono::Utc::now() + chrono::Duration::minutes(5)),
        })
    }
}

/// Validation result for UI forms and configurations
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
    pub manifest: Option<BiomeManifest>,
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self {
            is_valid: false,
            errors: vec![],
            warnings: vec![],
            manifest: None,
        }
    }
}
