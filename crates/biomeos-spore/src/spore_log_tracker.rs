//! Spore log tracking - Integrates fossil record into spore lifecycle
//!
//! This module tracks deployment, usage, and lifecycle events for each spore
//! directly on the USB drive, enabling forensic tracking and diagnostics.

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{info, warn};

/// Spore lifecycle event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SporeLifecycleEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: SporeEventType,
    pub node_id: Option<String>,
    pub deployed_to: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Types of spore lifecycle events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SporeEventType {
    /// Spore was created
    Created,
    /// Spore was deployed to a system
    Deployed,
    /// Spore deployment completed successfully
    DeploymentSuccess,
    /// Spore deployment failed
    DeploymentFailure,
    /// Spore was verified
    Verified,
    /// Spore was refreshed (binaries updated)
    Refreshed,
    /// Spore was archived
    Archived,
    /// Spore was cloned to create sibling
    Cloned,
    /// Custom event
    Custom(String),
}

/// Spore log tracker - Manages lifecycle events on spore
#[derive(Debug)]
pub struct SporeLogTracker {
    spore_path: PathBuf,
    log_dir: PathBuf,
    lifecycle_log: PathBuf,
}

impl SporeLogTracker {
    /// Create a new log tracker for a spore
    pub fn new(spore_path: impl AsRef<Path>) -> Result<Self> {
        let spore_path = spore_path.as_ref().to_path_buf();
        let log_dir = spore_path.join(".spore.logs");
        let lifecycle_log = log_dir.join("lifecycle.toml");
        
        Ok(Self {
            spore_path,
            log_dir,
            lifecycle_log,
        })
    }
    
    /// Initialize log tracking on spore
    pub async fn initialize(&self) -> Result<()> {
        info!("Initializing log tracking for spore: {}", self.spore_path.display());
        
        // Create log directory
        fs::create_dir_all(&self.log_dir)
            .await
            .context("Failed to create spore log directory")?;
        
        // Create initial lifecycle log if it doesn't exist
        if !self.lifecycle_log.exists() {
            let initial_log = SporeLifecycleLog {
                spore_path: self.spore_path.to_string_lossy().to_string(),
                created_at: Utc::now(),
                events: vec![],
            };
            
            self.save_lifecycle_log(&initial_log).await?;
        }
        
        // Create README explaining the logs
        let readme_path = self.log_dir.join("README.md");
        if !readme_path.exists() {
            let readme_content = r#"# Spore Log Tracking

This directory contains lifecycle and diagnostic logs for this spore.

## Files

- `lifecycle.toml` - Complete lifecycle history (creation, deployments, refreshes)
- `deployments/*.toml` - Individual deployment session logs
- `diagnostics/*.log` - Diagnostic snapshots from deployments

## Purpose

These logs enable:
- Forensic tracking of spore usage
- Deployment success/failure analysis
- Lineage and provenance validation
- Security audit trails

## Future Features

- **BearDog Encryption**: Logs will be encrypted with parent seed
- **Distributed Forensics**: Cross-spore lineage tracking
- **Auto-cleanup**: Old logs automatically archived to fossil record

---

**Note**: This is a self-tracking system. The spore records its own history.
"#;
            fs::write(&readme_path, readme_content)
                .await
                .context("Failed to write log README")?;
        }
        
        info!("✅ Log tracking initialized");
        Ok(())
    }
    
    /// Record a lifecycle event
    pub async fn record_event(&self, event: SporeLifecycleEvent) -> Result<()> {
        info!("Recording spore event: {:?}", event.event_type);
        
        // Load existing log
        let mut log = self.load_lifecycle_log().await?;
        
        // Add new event
        log.events.push(event);
        
        // Save updated log
        self.save_lifecycle_log(&log).await?;
        
        Ok(())
    }
    
    /// Record spore creation
    pub async fn record_creation(&self, node_id: &str) -> Result<()> {
        let event = SporeLifecycleEvent {
            timestamp: Utc::now(),
            event_type: SporeEventType::Created,
            node_id: Some(node_id.to_string()),
            deployed_to: None,
            metadata: HashMap::new(),
        };
        
        self.record_event(event).await
    }
    
    /// Record spore deployment
    pub async fn record_deployment(&self, node_id: &str, deployed_to: &str) -> Result<()> {
        let event = SporeLifecycleEvent {
            timestamp: Utc::now(),
            event_type: SporeEventType::Deployed,
            node_id: Some(node_id.to_string()),
            deployed_to: Some(deployed_to.to_string()),
            metadata: HashMap::new(),
        };
        
        self.record_event(event).await
    }
    
    /// Record deployment success
    pub async fn record_deployment_success(&self, node_id: &str, deployed_to: &str, metadata: HashMap<String, String>) -> Result<()> {
        let event = SporeLifecycleEvent {
            timestamp: Utc::now(),
            event_type: SporeEventType::DeploymentSuccess,
            node_id: Some(node_id.to_string()),
            deployed_to: Some(deployed_to.to_string()),
            metadata,
        };
        
        self.record_event(event).await
    }
    
    /// Record deployment failure
    pub async fn record_deployment_failure(&self, node_id: &str, deployed_to: &str, error: &str) -> Result<()> {
        let mut metadata = HashMap::new();
        metadata.insert("error".to_string(), error.to_string());
        
        let event = SporeLifecycleEvent {
            timestamp: Utc::now(),
            event_type: SporeEventType::DeploymentFailure,
            node_id: Some(node_id.to_string()),
            deployed_to: Some(deployed_to.to_string()),
            metadata,
        };
        
        self.record_event(event).await
    }
    
    /// Record spore verification
    pub async fn record_verification(&self, is_fresh: bool, stale_binaries: Vec<String>) -> Result<()> {
        let mut metadata = HashMap::new();
        metadata.insert("is_fresh".to_string(), is_fresh.to_string());
        if !stale_binaries.is_empty() {
            metadata.insert("stale_binaries".to_string(), stale_binaries.join(","));
        }
        
        let event = SporeLifecycleEvent {
            timestamp: Utc::now(),
            event_type: SporeEventType::Verified,
            node_id: None,
            deployed_to: None,
            metadata,
        };
        
        self.record_event(event).await
    }
    
    /// Record spore refresh
    pub async fn record_refresh(&self, refreshed_count: usize, failed_count: usize) -> Result<()> {
        let mut metadata = HashMap::new();
        metadata.insert("refreshed_count".to_string(), refreshed_count.to_string());
        metadata.insert("failed_count".to_string(), failed_count.to_string());
        
        let event = SporeLifecycleEvent {
            timestamp: Utc::now(),
            event_type: SporeEventType::Refreshed,
            node_id: None,
            deployed_to: None,
            metadata,
        };
        
        self.record_event(event).await
    }
    
    /// Record spore cloning
    pub async fn record_cloning(&self, parent_node: &str, sibling_node: &str) -> Result<()> {
        let mut metadata = HashMap::new();
        metadata.insert("parent_node".to_string(), parent_node.to_string());
        metadata.insert("sibling_node".to_string(), sibling_node.to_string());
        
        let event = SporeLifecycleEvent {
            timestamp: Utc::now(),
            event_type: SporeEventType::Cloned,
            node_id: Some(parent_node.to_string()),
            deployed_to: None,
            metadata,
        };
        
        self.record_event(event).await
    }
    
    /// Get lifecycle history
    pub async fn get_history(&self) -> Result<SporeLifecycleLog> {
        self.load_lifecycle_log().await
    }
    
    /// Get deployment count
    pub async fn get_deployment_count(&self) -> Result<usize> {
        let log = self.load_lifecycle_log().await?;
        let count = log.events.iter()
            .filter(|e| matches!(e.event_type, SporeEventType::Deployed | SporeEventType::DeploymentSuccess))
            .count();
        Ok(count)
    }
    
    /// Check if spore has been deployed
    pub async fn has_been_deployed(&self) -> bool {
        match self.get_deployment_count().await {
            Ok(count) => count > 0,
            Err(_) => false,
        }
    }
    
    /// Load lifecycle log from disk
    async fn load_lifecycle_log(&self) -> Result<SporeLifecycleLog> {
        if !self.lifecycle_log.exists() {
            // Return empty log if not yet created
            return Ok(SporeLifecycleLog {
                spore_path: self.spore_path.to_string_lossy().to_string(),
                created_at: Utc::now(),
                events: vec![],
            });
        }
        
        let content = fs::read_to_string(&self.lifecycle_log)
            .await
            .context("Failed to read lifecycle log")?;
        
        let log: SporeLifecycleLog = toml::from_str(&content)
            .context("Failed to parse lifecycle log")?;
        
        Ok(log)
    }
    
    /// Save lifecycle log to disk
    async fn save_lifecycle_log(&self, log: &SporeLifecycleLog) -> Result<()> {
        let content = toml::to_string_pretty(log)
            .context("Failed to serialize lifecycle log")?;
        
        fs::write(&self.lifecycle_log, content)
            .await
            .context("Failed to write lifecycle log")?;
        
        Ok(())
    }
}

/// Complete lifecycle log for a spore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SporeLifecycleLog {
    pub spore_path: String,
    pub created_at: DateTime<Utc>,
    pub events: Vec<SporeLifecycleEvent>,
}

