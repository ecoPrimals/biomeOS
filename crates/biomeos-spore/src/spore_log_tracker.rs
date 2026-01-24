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

            let content =
                toml::to_string_pretty(&initial_log).context("Failed to serialize initial log")?;
            fs::write(&self.log_file, content).await?;

            debug!(
                "Initialized spore lifecycle log: {}",
                self.log_file.display()
            );
        }

        Ok(())
    }

    /// Record a lifecycle event
    pub async fn record_event(&self, event: SporeLifecycleEvent) -> SporeResult<()> {
        // Ensure log is initialized
        self.initialize().await?;

        // Read current log
        let content = fs::read_to_string(&self.log_file).await?;
        let mut log: SporeLifecycleLog =
            toml::from_str(&content).context("Failed to parse lifecycle log")?;

        // Add event
        log.events.push(event.clone());

        // Write updated log
        let updated_content =
            toml::to_string_pretty(&log).context("Failed to serialize updated log")?;
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
        let log: SporeLifecycleLog =
            toml::from_str(&content).context("Failed to parse lifecycle log")?;

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
        Ok(self
            .spore_root
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown")
            .to_string())
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    // ========== SporeEventType Tests ==========

    #[test]
    fn test_spore_event_type_creation() {
        let event = SporeEventType::Creation;
        let json = serde_json::to_string(&event).unwrap();
        assert!(json.contains("creation"));
    }

    #[test]
    fn test_spore_event_type_custom() {
        let event = SporeEventType::Custom("migration".to_string());
        match event {
            SporeEventType::Custom(name) => assert_eq!(name, "migration"),
            _ => panic!("Expected Custom variant"),
        }
    }

    #[test]
    fn test_all_spore_event_types() {
        let events = vec![
            SporeEventType::Creation,
            SporeEventType::Cloning,
            SporeEventType::Incubation,
            SporeEventType::Verification,
            SporeEventType::Refresh,
            SporeEventType::Custom("test".to_string()),
        ];

        assert_eq!(events.len(), 6);
    }

    // ========== SporeLifecycleEvent Tests ==========

    #[test]
    fn test_spore_lifecycle_event_creation() {
        let event = SporeLifecycleEvent {
            timestamp: Utc::now(),
            event_type: SporeEventType::Creation,
            node_id: Some("node-123".to_string()),
            deployed_to: None,
            metadata: HashMap::new(),
        };

        assert!(matches!(event.event_type, SporeEventType::Creation));
        assert_eq!(event.node_id, Some("node-123".to_string()));
        assert!(event.metadata.is_empty());
    }

    #[test]
    fn test_spore_lifecycle_event_with_metadata() {
        let mut metadata = HashMap::new();
        metadata.insert("version".to_string(), "1.0.0".to_string());
        metadata.insert("environment".to_string(), "production".to_string());

        let event = SporeLifecycleEvent {
            timestamp: Utc::now(),
            event_type: SporeEventType::Incubation,
            node_id: Some("node-456".to_string()),
            deployed_to: Some("/media/usb".to_string()),
            metadata,
        };

        assert_eq!(event.metadata.len(), 2);
        assert_eq!(event.metadata.get("version"), Some(&"1.0.0".to_string()));
        assert_eq!(event.deployed_to, Some("/media/usb".to_string()));
    }

    #[test]
    fn test_spore_lifecycle_event_serialization() {
        let event = SporeLifecycleEvent {
            timestamp: Utc::now(),
            event_type: SporeEventType::Verification,
            node_id: Some("test-node".to_string()),
            deployed_to: None,
            metadata: HashMap::new(),
        };

        // Test JSON serialization
        let json = serde_json::to_string(&event).unwrap();
        let deserialized: SporeLifecycleEvent = serde_json::from_str(&json).unwrap();

        assert!(matches!(
            deserialized.event_type,
            SporeEventType::Verification
        ));
        assert_eq!(deserialized.node_id, Some("test-node".to_string()));
    }

    // ========== SporeLifecycleLog Tests ==========

    #[test]
    fn test_spore_lifecycle_log_creation() {
        let log = SporeLifecycleLog {
            spore_id: "spore-789".to_string(),
            created_at: Utc::now(),
            events: vec![],
        };

        assert_eq!(log.spore_id, "spore-789");
        assert!(log.events.is_empty());
    }

    #[test]
    fn test_spore_lifecycle_log_with_events() {
        let mut log = SporeLifecycleLog {
            spore_id: "spore-abc".to_string(),
            created_at: Utc::now(),
            events: vec![],
        };

        // Add events
        log.events.push(SporeLifecycleEvent {
            timestamp: Utc::now(),
            event_type: SporeEventType::Creation,
            node_id: None,
            deployed_to: None,
            metadata: HashMap::new(),
        });

        log.events.push(SporeLifecycleEvent {
            timestamp: Utc::now(),
            event_type: SporeEventType::Incubation,
            node_id: Some("node-1".to_string()),
            deployed_to: Some("/media/usb1".to_string()),
            metadata: HashMap::new(),
        });

        assert_eq!(log.events.len(), 2);
        assert!(matches!(log.events[0].event_type, SporeEventType::Creation));
        assert!(matches!(
            log.events[1].event_type,
            SporeEventType::Incubation
        ));
    }

    // ========== SporeLogTracker Tests ==========

    #[test]
    fn test_spore_log_tracker_new() {
        let temp_dir = TempDir::new().unwrap();
        let tracker = SporeLogTracker::new(temp_dir.path()).unwrap();

        assert_eq!(tracker.spore_root, temp_dir.path());
        assert_eq!(
            tracker.log_file,
            temp_dir.path().join(".spore.logs/lifecycle.toml")
        );
    }

    #[tokio::test]
    async fn test_spore_log_tracker_initialize() {
        let temp_dir = TempDir::new().unwrap();
        let tracker = SporeLogTracker::new(temp_dir.path()).unwrap();

        tracker.initialize().await.unwrap();

        // Verify log file was created
        assert!(tracker.log_file.exists());

        // Verify content structure
        let content = fs::read_to_string(&tracker.log_file).await.unwrap();
        let log: SporeLifecycleLog = toml::from_str(&content).unwrap();

        assert!(!log.spore_id.is_empty());
        assert!(log.events.is_empty());
    }

    #[tokio::test]
    async fn test_spore_log_tracker_record_event() {
        let temp_dir = TempDir::new().unwrap();
        let tracker = SporeLogTracker::new(temp_dir.path()).unwrap();

        let event = SporeLifecycleEvent {
            timestamp: Utc::now(),
            event_type: SporeEventType::Creation,
            node_id: Some("test-node".to_string()),
            deployed_to: None,
            metadata: HashMap::new(),
        };

        tracker.record_event(event.clone()).await.unwrap();

        // Verify event was recorded
        let events = tracker.get_events().await.unwrap();
        assert_eq!(events.len(), 1);
        assert!(matches!(events[0].event_type, SporeEventType::Creation));
        assert_eq!(events[0].node_id, Some("test-node".to_string()));
    }

    #[tokio::test]
    async fn test_spore_log_tracker_multiple_events() {
        let temp_dir = TempDir::new().unwrap();
        let tracker = SporeLogTracker::new(temp_dir.path()).unwrap();

        // Record multiple events
        for i in 0..5 {
            let event = SporeLifecycleEvent {
                timestamp: Utc::now(),
                event_type: if i % 2 == 0 {
                    SporeEventType::Verification
                } else {
                    SporeEventType::Refresh
                },
                node_id: Some(format!("node-{}", i)),
                deployed_to: None,
                metadata: HashMap::new(),
            };

            tracker.record_event(event).await.unwrap();
        }

        // Verify all events were recorded
        let events = tracker.get_events().await.unwrap();
        assert_eq!(events.len(), 5);
    }

    #[tokio::test]
    async fn test_spore_log_tracker_get_events_empty() {
        let temp_dir = TempDir::new().unwrap();
        let tracker = SporeLogTracker::new(temp_dir.path()).unwrap();

        // Before initialization, should return empty
        let events = tracker.get_events().await.unwrap();
        assert!(events.is_empty());
    }

    #[tokio::test]
    async fn test_spore_log_tracker_extract_spore_id_fallback() {
        let temp_dir = TempDir::new().unwrap();
        let tracker = SporeLogTracker::new(temp_dir.path()).unwrap();

        let spore_id = tracker.extract_spore_id().unwrap();

        // Should use directory name as fallback
        assert!(!spore_id.is_empty());
    }

    #[tokio::test]
    async fn test_spore_log_tracker_extract_spore_id_from_tower_toml() {
        let temp_dir = TempDir::new().unwrap();

        // Create tower.toml with node_id
        let tower_toml = r#"
[meta]
node_id = "custom-spore-id-123"
"#;
        std::fs::write(temp_dir.path().join("tower.toml"), tower_toml).unwrap();

        let tracker = SporeLogTracker::new(temp_dir.path()).unwrap();
        let spore_id = tracker.extract_spore_id().unwrap();

        assert_eq!(spore_id, "custom-spore-id-123");
    }

    #[tokio::test]
    async fn test_spore_log_tracker_event_with_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let tracker = SporeLogTracker::new(temp_dir.path()).unwrap();

        let mut metadata = HashMap::new();
        metadata.insert("primal_count".to_string(), "5".to_string());
        metadata.insert("graph_name".to_string(), "ecosystem".to_string());

        let event = SporeLifecycleEvent {
            timestamp: Utc::now(),
            event_type: SporeEventType::Incubation,
            node_id: Some("production-node".to_string()),
            deployed_to: Some("/media/usb0".to_string()),
            metadata,
        };

        tracker.record_event(event).await.unwrap();

        // Verify metadata was preserved
        let events = tracker.get_events().await.unwrap();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].metadata.len(), 2);
        assert_eq!(
            events[0].metadata.get("primal_count"),
            Some(&"5".to_string())
        );
    }

    // ========== Integration Tests ==========

    #[tokio::test]
    async fn test_spore_log_tracker_full_lifecycle() {
        let temp_dir = TempDir::new().unwrap();
        let tracker = SporeLogTracker::new(temp_dir.path()).unwrap();

        // 1. Creation event
        tracker
            .record_event(SporeLifecycleEvent {
                timestamp: Utc::now(),
                event_type: SporeEventType::Creation,
                node_id: None,
                deployed_to: None,
                metadata: HashMap::new(),
            })
            .await
            .unwrap();

        // 2. Cloning event
        tracker
            .record_event(SporeLifecycleEvent {
                timestamp: Utc::now(),
                event_type: SporeEventType::Cloning,
                node_id: None,
                deployed_to: Some("/media/usb1".to_string()),
                metadata: HashMap::new(),
            })
            .await
            .unwrap();

        // 3. Incubation event
        let mut metadata = HashMap::new();
        metadata.insert("computer_name".to_string(), "laptop-01".to_string());

        tracker
            .record_event(SporeLifecycleEvent {
                timestamp: Utc::now(),
                event_type: SporeEventType::Incubation,
                node_id: Some("nat0-node-1".to_string()),
                deployed_to: Some("/media/usb1".to_string()),
                metadata,
            })
            .await
            .unwrap();

        // 4. Verification event
        tracker
            .record_event(SporeLifecycleEvent {
                timestamp: Utc::now(),
                event_type: SporeEventType::Verification,
                node_id: Some("nat0-node-1".to_string()),
                deployed_to: None,
                metadata: HashMap::new(),
            })
            .await
            .unwrap();

        // 5. Refresh event
        tracker
            .record_event(SporeLifecycleEvent {
                timestamp: Utc::now(),
                event_type: SporeEventType::Refresh,
                node_id: Some("nat0-node-1".to_string()),
                deployed_to: None,
                metadata: HashMap::new(),
            })
            .await
            .unwrap();

        // Verify full lifecycle
        let events = tracker.get_events().await.unwrap();
        assert_eq!(events.len(), 5);

        // Verify event order
        assert!(matches!(events[0].event_type, SporeEventType::Creation));
        assert!(matches!(events[1].event_type, SporeEventType::Cloning));
        assert!(matches!(events[2].event_type, SporeEventType::Incubation));
        assert!(matches!(events[3].event_type, SporeEventType::Verification));
        assert!(matches!(events[4].event_type, SporeEventType::Refresh));
    }

    #[test]
    fn test_spore_lifecycle_log_serialization() {
        let mut log = SporeLifecycleLog {
            spore_id: "test-spore".to_string(),
            created_at: Utc::now(),
            events: vec![],
        };

        log.events.push(SporeLifecycleEvent {
            timestamp: Utc::now(),
            event_type: SporeEventType::Creation,
            node_id: None,
            deployed_to: None,
            metadata: HashMap::new(),
        });

        // Test TOML serialization
        let toml = toml::to_string(&log).unwrap();
        let deserialized: SporeLifecycleLog = toml::from_str(&toml).unwrap();

        assert_eq!(deserialized.spore_id, "test-spore");
        assert_eq!(deserialized.events.len(), 1);
    }
}
