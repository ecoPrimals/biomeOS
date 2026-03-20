// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! AI assistant and insight types for the TUI dashboard.

use std::time::Instant;

/// AI chat message for human/AI interface
#[derive(Debug, Clone)]
pub struct AiChatMessage {
    /// When the message was sent
    pub timestamp: Instant,
    /// Who sent the message
    pub role: AiRole,
    /// Message content
    pub content: String,
    /// Optional context about what system state this relates to
    pub context: Option<String>,
}

/// AI roles in conversation
#[derive(Debug, Clone)]
pub enum AiRole {
    /// Human user
    Human,
    /// AI assistant
    Assistant,
    /// System-generated message
    System,
}

/// AI suggestions for operations
#[derive(Debug, Clone)]
pub struct AiSuggestion {
    /// Unique suggestion identifier
    pub id: String,
    /// Short title for the suggestion
    pub title: String,
    /// Detailed description of the suggestion
    pub description: String,
    /// CLI command to execute (if applicable)
    pub command: Option<String>,
    /// Confidence score (0.0–1.0)
    pub confidence: f64,
    /// Category of the suggestion
    pub category: AiSuggestionCategory,
    /// Whether the suggestion can be executed automatically
    pub can_execute: bool,
}

/// Categories of AI suggestions
#[derive(Debug, Clone)]
pub enum AiSuggestionCategory {
    /// Scaling-related suggestion
    Scaling,
    /// Performance optimization suggestion
    Performance,
    /// Security-related suggestion
    Security,
    /// Deployment-related suggestion
    Deployment,
    /// Troubleshooting suggestion
    Troubleshooting,
    /// Resource optimization suggestion
    Optimization,
}

/// AI insights from analyzing ecosystem data
#[derive(Debug, Clone)]
pub struct AiInsight {
    /// Insight title
    pub title: String,
    /// Detailed insight description
    pub insight: String,
    /// Severity level of the insight
    pub severity: InsightSeverity,
    /// Components affected by this insight
    pub affected_components: Vec<String>,
    /// Recommended actions to address the insight
    pub recommended_actions: Vec<String>,
    /// Confidence in the insight accuracy (0.0–1.0)
    pub confidence: f64,
}

/// Severity levels for insights
#[derive(Debug, Clone)]
pub enum InsightSeverity {
    /// Informational insight
    Info,
    /// Warning-level insight
    Warning,
    /// Critical issue requiring immediate attention
    Critical,
    /// Optimization opportunity
    Optimization,
}
