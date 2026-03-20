// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! TUI Dashboard Types for Ecosystem Interface
//!
//! Comprehensive types for BiomeOS as the human/AI interface to a headless, AI-first ecosystem.
//! Supports API ingestion from all primals and deployment orchestration.

mod ai;
mod api_logs;
mod dashboard;
mod tabs;

pub use super::primal_ecosystem::{
    DeploymentEvent, DeploymentEventType, DeploymentPhase, DeploymentStatus, EcosystemHealth,
    PrimalApiState, PrimalMetadata, PrimalMetrics, PrimalServiceInfo, ResourceUsage, ServiceStatus,
};

pub use ai::{
    AiChatMessage, AiInsight, AiRole, AiSuggestion, AiSuggestionCategory, InsightSeverity,
};
pub use api_logs::{ApiEndpointStatus, ApiError, ApiStatus, LogEntry, LogFilter, LogLevel};
pub use dashboard::DashboardState;
pub use tabs::{TabId, TabInfo};

#[cfg(test)]
mod tests;
