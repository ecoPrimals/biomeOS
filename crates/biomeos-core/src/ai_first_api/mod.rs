// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! AI-First Citizen API Standard Implementation
//!
//! ✅ MIGRATION NOTICE: Core AI types have been moved to biomeos-types
//! This module now re-exports the unified AI-first system and provides
//! backward compatibility for the biomeOS ecosystem.
//!
//! Implements the AI-First response format and human-AI collaboration context
//! as defined in handOff/AI_FIRST_CITIZEN_API_STANDARD.md

mod response;
mod types;

#[cfg(test)]
mod tests;

// Re-export unified AI-first types from biomeos-types
pub use biomeos_types::{
    BiomeError, BiomeOSConfig, BiomeResult, Environment, error::AIErrorCategory,
    error::ErrorSeverity,
};

// Re-export our types
pub use types::{
    AIFirstResponse, AIResourceLimits, AIResourceUsage, AIResponseMetadata, AIUserPreferences,
    ActionHistory, ActionResult, BackoffType, CacheInfo, ContactType, DataSharingLevel,
    EscalationConfig, EscalationContact, HumanInteractionContext, InteractionMode,
    NotificationPreferences, NotificationUrgency, PerformanceMetrics, PriorityLevel,
    QualityMetrics, QuietHours, RateLimitStatus, RetryStrategy, RiskLevel, RiskTolerance,
    SessionContext, SuggestedAction,
};
