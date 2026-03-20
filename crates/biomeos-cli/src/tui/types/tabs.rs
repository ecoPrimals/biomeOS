// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! Tab identifiers and metadata for the ecosystem dashboard.

/// Tab information for the comprehensive ecosystem interface
#[derive(Clone)]
pub struct TabInfo {
    /// Tab display title
    pub title: &'static str,
    /// Tab identifier
    pub id: TabId,
    /// Tab icon (emoji)
    pub icon: &'static str,
}

/// Available tabs in the ecosystem dashboard
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum TabId {
    /// Overview of the entire ecosystem
    EcosystemOverview,
    /// Status of individual primals
    PrimalStatus,
    /// Deployment orchestration view
    DeploymentOrchestration,

    /// Running services view
    Services,
    /// Health monitoring view
    Health,

    /// AI assistant chat interface
    AiAssistant,
    /// AI-generated insights view
    AiInsights,

    /// API data ingestion status
    ApiIngestion,
    /// Metrics and performance view
    Metrics,
    /// Log streaming view
    Logs,
}

impl TabId {
    /// Get all available tabs for the ecosystem interface
    pub fn all_tabs() -> Vec<TabInfo> {
        vec![
            TabInfo {
                title: "Ecosystem",
                id: TabId::EcosystemOverview,
                icon: "🌍",
            },
            TabInfo {
                title: "Primals",
                id: TabId::PrimalStatus,
                icon: "🎯",
            },
            TabInfo {
                title: "Deploy",
                id: TabId::DeploymentOrchestration,
                icon: "🚀",
            },
            TabInfo {
                title: "Services",
                id: TabId::Services,
                icon: "⚙️",
            },
            TabInfo {
                title: "Health",
                id: TabId::Health,
                icon: "💊",
            },
            TabInfo {
                title: "AI Assistant",
                id: TabId::AiAssistant,
                icon: "🤖",
            },
            TabInfo {
                title: "AI Insights",
                id: TabId::AiInsights,
                icon: "🧠",
            },
            TabInfo {
                title: "API Data",
                id: TabId::ApiIngestion,
                icon: "📡",
            },
            TabInfo {
                title: "Metrics",
                id: TabId::Metrics,
                icon: "📊",
            },
            TabInfo {
                title: "Logs",
                id: TabId::Logs,
                icon: "📜",
            },
        ]
    }
}
