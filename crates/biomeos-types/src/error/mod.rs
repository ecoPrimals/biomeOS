// SPDX-License-Identifier: AGPL-3.0-or-later
// Copyright 2025-2026 ecoPrimals Project

//! Unified Error Handling System
//!
//! This module consolidates all error types that were previously scattered
//! across biomeos-core (`BiomeError`), biomeos-primal-sdk (`PrimalError`),
//! and `ai_first_api` (`AIFirstError`) into a comprehensive, AI-first error system.
//!
//! The module is split into:
//! - `core`: Main `BiomeError` enum and core error types
//! - `ai_context`: AI-specific error context and automation features
//! - `operations`: Operation types for error context
//! - `conversions`: Error constructor methods and From implementations

pub mod ai_context;
pub mod conversions;
pub mod core;
pub mod operations;

// Re-export core types
pub use core::{BiomeError, IpcError, ValidationError};

// Re-export AI context types
pub use ai_context::{
    AIErrorCategory, AIErrorContext, ActionRiskLevel, ActionType, BackoffType, ErrorSeverity,
    RetryStrategy, SuggestedAction,
};

// Re-export operation types
pub use operations::{DataOperation, NetworkOperation, ResourceOperation, SecurityViolationType};

// Re-export conversions
pub use conversions::BiomeResult;
