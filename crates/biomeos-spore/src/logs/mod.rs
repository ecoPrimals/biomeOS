// Log Management Module - Smart Decomposition
//!
//! Provides comprehensive log management for biomeOS deployments:
//! - Active log tracking by node ID
//! - Automatic archival to fossil records
//! - Issue detection and forensic analysis
//!
//! # Architecture
//!
//! ```text
//! logs/
//! ├── config.rs   - Configuration types
//! ├── session.rs  - Active session and file management
//! ├── fossil.rs   - Fossil record archival
//! ├── metrics.rs  - Issues and metrics tracking
//! └── manager.rs  - LogManager and SporeLogManager
//! ```

mod config;
mod fossil;
mod manager;
mod metrics;
mod session;

pub use config::LogConfig;
pub use fossil::{ArchivalReason, FossilIndex, FossilIndexEntry, FossilRecord};
pub use manager::{LogManager, SporeLogManager};
pub use metrics::{IssueSeverity, LogIssue, LogMetrics};
pub use session::{ActiveLogSession, LogFile};
