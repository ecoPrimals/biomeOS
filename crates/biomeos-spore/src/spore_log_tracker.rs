//! Spore-specific log tracking module
//!
//! This module provides spore lifecycle event tracking, recording events like:
//! - Spore creation
//! - Spore cloning
//! - Spore incubation (deployment on computers)
//! - Spore verification
//! - Spore refresh

use anyhow::{Context, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::fs;
use tracing::{debug, info};

use crate::error::SporeResult;

/// Types of spore lifecycle events
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SporeEventType {
    Creation,
    Cloning,
    Incubation,
    Verification,
    Refresh,
    Custom(String),
}

/// A spore lifecycle event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SporeLifecycleEvent {
    pub timestamp: DateTime<Utc>,
    pub event_type: SporeEventType,
    pub node_id: Option<String>,
    pub deployed_to: Option<String>,
    pub metadata: HashMap<String, String>,
}

/// Lifecycle log for a spore
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SporeLifecycleLog {
    pub spore_id: String,
    pub created_at: DateTime<Utc>,
    pub events: Vec<SporeLifecycleEvent>,
}

/// Tracks spore lifecycle events
pub struct SporeLogTracker {
    spore_root: PathBuf,
    log_file: PathBuf,
}

impl SporeLogTracker {
    /// Create a new spore log tracker
    pub fn new(spore_root: impl AsRef<Path>) -> SporeResult<Self> {
        let spore_root = spore_root.as_ref().to_path_buf();
        let log_file = spore_root.join(".spore.logs").join("lifecycle.toml");
        
        Ok(Self {
            spore_root,
            log_file,
        })
    }
    
    /// Initialize the log file if it doesn't exist
    pub async fn initialize(&self) -> SporeResult<()> {
        // Create log directory
        if let Some(parent) = self.log_file.parent() {
            fs::create_dir_all(parent).await?;
        }
        
        // If log file doesn't exist, create it with initial structure
        if !self.log_file.exists() {
            let initial_log = SporeLifecycleLog {
                spore_id: self.extract_spore_id()?,
                created_at: Utc::now(),
                events: vec![],
            };
            
            let content = toml::to_string_pretty(&initial_log)
                .context("Failed to serialize initial log")?;
            fs::write(&self.log_file, content).await?;
            
            debug!("Initialized spore lifecycle log: {}", self.log_file.display());
        }
        
        Ok(())
    }
    
    /// Record a lifecycle event
    pub async fn record_event(&self, event: SporeLifecycleEvent) -> SporeResult<()> {
        // Ensure log is initialized
        self.initialize().await?;
        
        // Read current log
        let content = fs::read_to_string(&self.log_file).await?;
        let mut log: SporeLifecycleLog = toml::from_str(&content)
            .context("Failed to parse lifecycle log")?;
        
        // Add event
        log.events.push(event.clone());
        
        // Write updated log
        let updated_content = toml::to_string_pretty(&log)
            .context("Failed to serialize updated log")?;
        fs::write(&self.log_file, updated_content).await?;
        
        info!("Recorded spore lifecycle event: {:?}", event.event_type);
        
        Ok(())
    }
    
    /// Get all events
    pub async fn get_events(&self) -> SporeResult<Vec<SporeLifecycleEvent>> {
        if !self.log_file.exists() {
            return Ok(vec![]);
        }
        
        let content = fs::read_to_string(&self.log_file).await?;
        let log: SporeLifecycleLog = toml::from_str(&content)
            .context("Failed to parse lifecycle log")?;
        
        Ok(log.events)
    }
    
    /// Extract spore ID from the spore root
    fn extract_spore_id(&self) -> Result<String> {
        // Try to read from tower.toml
        let tower_toml_path = self.spore_root.join("tower.toml");
        
        if tower_toml_path.exists() {
            let content = std::fs::read_to_string(&tower_toml_path)?;
            
            if let Ok(config) = toml::from_str::<toml::Value>(&content) {
                if let Some(meta) = config.get("meta") {
                    if let Some(node_id) = meta.get("node_id") {
                        if let Some(id) = node_id.as_str() {
                            return Ok(id.to_string());
                        }
                    }
                }
            }
        }
        
        // Fallback: use directory name
        Ok(self.spore_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string())
    }
}
