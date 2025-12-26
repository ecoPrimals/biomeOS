//! Event System Module
//!
//! This module provides a simple event system for reactive UI updates.

use crate::UIEvent;
use chrono::{DateTime, Utc};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Event priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum EventPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

/// Event context with metadata
#[derive(Debug, Clone)]
pub struct EventContext {
    /// Event unique identifier
    pub id: String,
    /// When the event was created
    pub timestamp: DateTime<Utc>,
    /// Event priority
    pub priority: EventPriority,
    /// Event source component
    pub source: String,
    /// Target component (optional)
    pub target: Option<String>,
    /// Additional metadata
    pub metadata: HashMap<String, String>,
}

/// Event error types
#[derive(Debug)]
pub enum EventError {
    ProcessingFailed { message: String },
    InvalidEvent { reason: String },
}

impl std::fmt::Display for EventError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EventError::ProcessingFailed { message } => write!(f, "Processing failed: {}", message),
            EventError::InvalidEvent { reason } => write!(f, "Invalid event: {}", reason),
        }
    }
}

impl std::error::Error for EventError {}

/// Event statistics for monitoring
#[derive(Debug, Default, Clone)]
pub struct EventStats {
    pub total_events: u64,
    pub events_by_priority: HashMap<EventPriority, u64>,
    pub average_processing_time: std::time::Duration,
}

/// Simple event dispatcher for UI events
#[derive(Clone)]
pub struct EventDispatcher {
    stats: Arc<RwLock<EventStats>>,
}

impl Default for EventDispatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl EventDispatcher {
    /// Create a new event dispatcher
    pub fn new() -> Self {
        Self {
            stats: Arc::new(RwLock::new(EventStats::default())),
        }
    }

    /// Dispatch an event with context
    pub async fn dispatch_with_context(
        &self,
        _event: UIEvent,
        context: EventContext,
    ) -> Result<(), EventError> {
        // Update metrics
        {
            let mut stats = self.stats.write().await;
            stats.total_events += 1;
            *stats
                .events_by_priority
                .entry(context.priority)
                .or_insert(0) += 1;
        }

        tracing::debug!(
            "Dispatched event {} with priority {:?}",
            context.id,
            context.priority
        );
        Ok(())
    }

    /// Get current statistics
    pub async fn get_stats(&self) -> EventStats {
        self.stats.read().await.clone()
    }
}
