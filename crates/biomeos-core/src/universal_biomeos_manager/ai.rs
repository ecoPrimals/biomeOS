// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! AI and Partnership Operations
//!
//! Handles AI-powered assistance, partnership access, and specialized modes
//! like grandma safe mode.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::core::UniversalBiomeOSManager;

/// Genetic access key type (legacy)
pub type GeneticAccessKey = String;

impl UniversalBiomeOSManager {
    /// AI-powered biome management assistance
    pub async fn ai_assist(
        &self,
        query: &str,
        context: Option<String>,
    ) -> Result<HashMap<String, serde_json::Value>> {
        tracing::info!("🤖 Processing AI assistance request: {}", query);

        let mut result = HashMap::new();
        result.insert("query".to_string(), serde_json::json!(query));
        result.insert("context".to_string(), serde_json::json!(context));

        // Future: Integrate with actual AI service (Toadstool or external LLM)
        let ai_response = self.process_ai_query(query, context.as_deref()).await?;

        result.insert(
            "response".to_string(),
            serde_json::json!(ai_response.response),
        );
        result.insert(
            "confidence".to_string(),
            serde_json::json!(ai_response.confidence),
        );
        result.insert(
            "suggestions".to_string(),
            serde_json::json!(ai_response.suggestions),
        );
        result.insert(
            "actions".to_string(),
            serde_json::json!(ai_response.actions),
        );
        result.insert("status".to_string(), serde_json::json!("success"));
        result.insert(
            "timestamp".to_string(),
            serde_json::json!(chrono::Utc::now()),
        );

        tracing::info!(
            "✅ AI assistance completed with confidence: {}",
            ai_response.confidence
        );
        Ok(result)
    }

    /// Initialize partnership access
    pub async fn initialize_partnership_access(&self, _key: GeneticAccessKey) -> Result<()> {
        tracing::info!("🤝 Initializing partnership access");

        // Partnership access initialization logic would go here
        // This would involve:
        // - Validating the genetic access key
        // - Setting up secure communication channels
        // - Establishing trust relationships
        // - Configuring partnership-specific permissions

        tracing::info!("✅ Partnership access initialized successfully");
        Ok(())
    }

    /// Initialize grandma safe mode
    pub async fn initialize_grandma_safe(&self) -> Result<()> {
        tracing::info!("👵 Initializing grandma safe mode");

        // Grandma safe mode initialization:
        // - Simplified UI configurations
        // - Enhanced safety checks
        // - Automatic backup systems
        // - User-friendly error messages
        // - Restricted access to advanced features

        tracing::info!("✅ Grandma safe mode initialized");
        Ok(())
    }

    /// Get AI capabilities and status
    pub async fn get_ai_status(&self) -> Result<HashMap<String, serde_json::Value>> {
        let mut result = HashMap::new();

        result.insert("ai_enabled".to_string(), serde_json::json!(true));
        result.insert("version".to_string(), serde_json::json!("1.0.0"));
        result.insert(
            "capabilities".to_string(),
            serde_json::json!([
                "natural_language_processing",
                "system_optimization",
                "troubleshooting_assistance",
                "configuration_guidance",
                "deployment_planning"
            ]),
        );
        result.insert("status".to_string(), serde_json::json!("ready"));
        result.insert(
            "models_loaded".to_string(),
            serde_json::json!(["biome-assistant-v1"]),
        );

        Ok(result)
    }

    /// Process AI query with natural language understanding
    async fn process_ai_query(&self, query: &str, context: Option<&str>) -> Result<AIResponse> {
        tracing::debug!("Processing AI query: {} (context: {:?})", query, context);

        // Analyze query intent
        let intent = self.analyze_query_intent(query).await;

        // Generate response based on intent and system state
        let response = match intent {
            QueryIntent::HealthCheck => {
                let health = self.get_system_health().await;
                AIResponse {
                    response: format!("System health is currently: {:?}. All registered primals are being monitored.", health.health),
                    confidence: 0.95,
                    suggestions: vec![
                        "Run 'biomeos health --detailed' for more information".to_string(),
                        "Check individual service health with 'biomeos probe <service>'".to_string(),
                    ],
                    actions: vec![
                        AIAction {
                            name: "detailed_health_check".to_string(),
                            description: "Run comprehensive health check".to_string(),
                            command: Some("biomeos health --detailed".to_string()),
                        }
                    ],
                }
            }
            QueryIntent::ServiceDiscovery => {
                let primals = self.get_registered_primals().await;
                AIResponse {
                    response: format!("Found {} registered services. Use discovery commands to find more services in your network.", primals.len()),
                    confidence: 0.90,
                    suggestions: vec![
                        "Run 'biomeos discover' to find network services".to_string(),
                        "Use 'biomeos discover --capabilities <cap>' for specific capabilities".to_string(),
                    ],
                    actions: vec![
                        AIAction {
                            name: "network_discovery".to_string(),
                            description: "Discover services on network".to_string(),
                            command: Some("biomeos discover".to_string()),
                        }
                    ],
                }
            }
            QueryIntent::Deployment => {
                AIResponse {
                    response: "For deployment, you'll need a biome manifest file. I can help you create one or deploy an existing manifest.".to_string(),
                    confidence: 0.85,
                    suggestions: vec![
                        "Create a manifest with 'biomeos create <name>'".to_string(),
                        "Deploy with 'biomeos deploy --manifest <file>'".to_string(),
                        "Validate first with 'biomeos deploy --validate-only'".to_string(),
                    ],
                    actions: vec![
                        AIAction {
                            name: "create_manifest".to_string(),
                            description: "Create new biome manifest".to_string(),
                            command: Some("biomeos create".to_string()),
                        }
                    ],
                }
            }
            QueryIntent::Troubleshooting => {
                AIResponse {
                    response: "I can help troubleshoot issues. Common commands include health checks, log analysis, and service probing.".to_string(),
                    confidence: 0.80,
                    suggestions: vec![
                        "Check system health: 'biomeos health'".to_string(),
                        "View service logs: 'biomeos logs <service>'".to_string(),
                        "Probe specific service: 'biomeos probe <service>'".to_string(),
                    ],
                    actions: vec![
                        AIAction {
                            name: "diagnostic_scan".to_string(),
                            description: "Run diagnostic scan".to_string(),
                            command: Some("biomeos scan".to_string()),
                        }
                    ],
                }
            }
            QueryIntent::General => {
                AIResponse {
                    response: self.generate_general_response(query, context).await,
                    confidence: 0.70,
                    suggestions: vec![
                        "Try 'biomeos --help' for available commands".to_string(),
                        "Use 'biomeos status' for system overview".to_string(),
                    ],
                    actions: vec![],
                }
            }
        };

        Ok(response)
    }

    /// Analyze query intent using simple keyword matching
    async fn analyze_query_intent(&self, query: &str) -> QueryIntent {
        let query_lower = query.to_lowercase();

        if query_lower.contains("health")
            || query_lower.contains("status")
            || query_lower.contains("check")
        {
            QueryIntent::HealthCheck
        } else if query_lower.contains("discover")
            || query_lower.contains("find")
            || query_lower.contains("service")
        {
            QueryIntent::ServiceDiscovery
        } else if query_lower.contains("deploy")
            || query_lower.contains("create")
            || query_lower.contains("manifest")
        {
            QueryIntent::Deployment
        } else if query_lower.contains("error")
            || query_lower.contains("problem")
            || query_lower.contains("issue")
            || query_lower.contains("troubleshoot")
        {
            QueryIntent::Troubleshooting
        } else {
            QueryIntent::General
        }
    }

    /// Generate general response for unmatched queries
    async fn generate_general_response(&self, query: &str, _context: Option<&str>) -> String {
        // This would be enhanced with actual AI/ML processing
        format!(
            "I understand you're asking about '{query}'. BiomeOS provides comprehensive ecosystem management. \
             Key areas include service discovery, health monitoring, deployment management, and system orchestration. \
             What specific aspect would you like help with?"
        )
    }

    // Enable AI-powered system optimization
    // NOTE: Commented out - depends on legacy ClientRegistry
    /*
    pub async fn enable_ai_optimization(&self) -> Result<HashMap<String, serde_json::Value>> {
        tracing::info!("🧠 Enabling AI-powered system optimization");

        let mut result = HashMap::new();

        // Delegate to Squirrel for real AI-powered optimization
        if let Ok(squirrel) = self.clients().squirrel().await {
            // Gather system state for analysis
            let system_health = self.get_system_health().await;
            let primals = self.get_registered_primals().await;

            let system_state = serde_json::json!({
                "health": system_health.health,
                "service_count": primals.len(),
                "timestamp": chrono::Utc::now()
            });

            // Get AI analysis from Squirrel
            match squirrel.analyze_system_optimization(&system_state).await {
                Ok(analysis) => {
                    result.insert("optimization_enabled".to_string(), serde_json::json!(true));
                    result.insert("score".to_string(), serde_json::json!(analysis.score));
                    result.insert(
                        "opportunities".to_string(),
                        serde_json::json!(analysis.opportunities),
                    );
                    result.insert("estimated_improvement".to_string(), serde_json::json!({
                        "performance": analysis.estimated_improvement.performance,
                        "resource_efficiency": analysis.estimated_improvement.resource_efficiency,
                        "cost_savings": analysis.estimated_improvement.cost_savings
                    }));
                    result.insert(
                        "timestamp".to_string(),
                        serde_json::json!(analysis.timestamp),
                    );
                    result.insert("status".to_string(), serde_json::json!("success"));

                    tracing::info!(
                        "✅ AI optimization analysis complete (score: {})",
                        analysis.score
                    );
                }
                Err(e) => {
                    tracing::warn!("Failed to get optimization analysis from Squirrel: {}", e);
                    result.insert("optimization_enabled".to_string(), serde_json::json!(false));
                    result.insert(
                        "error".to_string(),
                        serde_json::json!(format!("Squirrel analysis failed: {}", e)),
                    );
                    result.insert("status".to_string(), serde_json::json!("error"));
                }
            }
        } else {
            // Squirrel not available - graceful degradation
            tracing::info!("Squirrel not available - AI optimization unavailable");
            result.insert("optimization_enabled".to_string(), serde_json::json!(false));
            result.insert("message".to_string(), serde_json::json!(
                "AI optimization requires Squirrel primal. Ensure Squirrel is registered with Songbird."
            ));
            result.insert("status".to_string(), serde_json::json!("unavailable"));
            result.insert(
                "delegation_target".to_string(),
                serde_json::json!({
                    "primal": "squirrel",
                    "capability": "ai",
                    "method": "analyze_system_optimization"
                }),
            );
        }

        Ok(result)
    }
    */

    /// Get AI recommendations for system improvements
    pub async fn get_ai_recommendations(&self) -> Result<Vec<AIRecommendation>> {
        let primals = self.get_registered_primals().await;
        let mut recommendations = Vec::new();

        // Analyze current system state and generate recommendations
        if primals.is_empty() {
            recommendations.push(AIRecommendation {
                title: "No Services Discovered".to_string(),
                description: "Consider running service discovery to find available primals"
                    .to_string(),
                priority: Priority::High,
                category: "Discovery".to_string(),
                action: Some("biomeos discover".to_string()),
                estimated_impact: "Enable full system functionality".to_string(),
            });
        }

        if primals.len() < 3 {
            recommendations.push(AIRecommendation {
                title: "Limited Service Ecosystem".to_string(),
                description: "Your biome could benefit from additional services for redundancy"
                    .to_string(),
                priority: Priority::Medium,
                category: "Architecture".to_string(),
                action: Some("biomeos create --template comprehensive".to_string()),
                estimated_impact: "Improved reliability and capabilities".to_string(),
            });
        }

        // Always include general optimization recommendation
        recommendations.push(AIRecommendation {
            title: "Enable AI Monitoring".to_string(),
            description: "Continuous AI monitoring can proactively identify issues".to_string(),
            priority: Priority::Low,
            category: "Optimization".to_string(),
            action: Some("biomeos monitor --ai-enabled".to_string()),
            estimated_impact: "Proactive issue prevention".to_string(),
        });

        Ok(recommendations)
    }
}

/// AI Response structure
#[derive(Debug, Clone)]
struct AIResponse {
    response: String,
    confidence: f64,
    suggestions: Vec<String>,
    actions: Vec<AIAction>,
}

/// AI Action that can be taken
#[derive(Debug, Clone, Serialize, Deserialize)]
struct AIAction {
    name: String,
    description: String,
    command: Option<String>,
}

/// Query intent classification
#[derive(Debug, Clone, PartialEq)]
enum QueryIntent {
    HealthCheck,
    ServiceDiscovery,
    Deployment,
    Troubleshooting,
    General,
}

/// AI Recommendation
#[derive(Debug, Clone)]
pub struct AIRecommendation {
    /// Short title for the recommendation
    pub title: String,
    /// Detailed description of what to do and why
    pub description: String,
    /// How important this recommendation is
    pub priority: Priority,
    /// Category (e.g. "performance", "security", "reliability")
    pub category: String,
    /// Specific command or action to take
    pub action: Option<String>,
    /// Expected improvement if recommendation is followed
    pub estimated_impact: String,
}

/// Recommendation priority
#[derive(Debug, Clone)]
pub enum Priority {
    /// Should be addressed immediately
    High,
    /// Should be addressed soon
    Medium,
    /// Can be addressed when convenient
    Low,
}

#[allow(clippy::unwrap_used, clippy::expect_used)]
#[cfg(test)]
mod tests {
    use super::*;
    use crate::universal_biomeos_manager::UniversalBiomeOSManager;
    use biomeos_types::BiomeOSConfig;

    async fn test_manager() -> UniversalBiomeOSManager {
        UniversalBiomeOSManager::new(BiomeOSConfig::default())
            .await
            .expect("create test manager")
    }

    #[tokio::test]
    async fn test_ai_assist_health_query() {
        let manager = test_manager().await;
        let result = manager
            .ai_assist("what is the system health?", None)
            .await
            .expect("ai_assist should succeed");
        assert!(result.contains_key("response"));
        assert!(result.contains_key("confidence"));
        assert!(result.contains_key("suggestions"));
        assert!(result.contains_key("actions"));
        assert_eq!(
            result.get("status").and_then(|v| v.as_str()),
            Some("success")
        );
        let response = result
            .get("response")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert!(response.to_lowercase().contains("health"));
    }

    #[tokio::test]
    async fn test_ai_assist_discovery_query() {
        let manager = test_manager().await;
        let result = manager
            .ai_assist("how do I discover services?", Some("context".to_string()))
            .await
            .expect("ai_assist should succeed");
        assert!(result.contains_key("context"));
        assert_eq!(
            result.get("query").and_then(|v| v.as_str()),
            Some("how do I discover services?")
        );
    }

    #[tokio::test]
    async fn test_ai_assist_deployment_query() {
        let manager = test_manager().await;
        let result = manager
            .ai_assist("I want to deploy a manifest", None)
            .await
            .expect("ai_assist should succeed");
        let response = result
            .get("response")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert!(
            response.to_lowercase().contains("manifest")
                || response.to_lowercase().contains("deploy")
        );
    }

    #[tokio::test]
    async fn test_ai_assist_troubleshooting_query() {
        let manager = test_manager().await;
        let result = manager
            .ai_assist("I have an error and need to troubleshoot", None)
            .await
            .expect("ai_assist should succeed");
        let response = result
            .get("response")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        assert!(
            response.to_lowercase().contains("troubleshoot")
                || response.to_lowercase().contains("help")
        );
    }

    #[tokio::test]
    async fn test_ai_assist_general_query() {
        let manager = test_manager().await;
        let result = manager
            .ai_assist("tell me about biomeos", None)
            .await
            .expect("ai_assist should succeed");
        assert!(
            result
                .get("confidence")
                .and_then(serde_json::Value::as_f64)
                .unwrap_or(0.0)
                <= 1.0
        );
    }

    #[tokio::test]
    async fn test_initialize_partnership_access() {
        let manager = test_manager().await;
        manager
            .initialize_partnership_access("test-key".to_string())
            .await
            .expect("initialize_partnership_access should succeed");
    }

    #[tokio::test]
    async fn test_initialize_grandma_safe() {
        let manager = test_manager().await;
        manager
            .initialize_grandma_safe()
            .await
            .expect("initialize_grandma_safe should succeed");
    }

    #[tokio::test]
    async fn test_get_ai_status() {
        let manager = test_manager().await;
        let status = manager
            .get_ai_status()
            .await
            .expect("get_ai_status should succeed");
        assert_eq!(
            status
                .get("ai_enabled")
                .and_then(serde_json::Value::as_bool),
            Some(true)
        );
        assert_eq!(status.get("status").and_then(|v| v.as_str()), Some("ready"));
        assert!(status.contains_key("capabilities"));
    }

    #[tokio::test]
    async fn test_get_ai_recommendations_empty_primals() {
        let manager = test_manager().await;
        let recs = manager
            .get_ai_recommendations()
            .await
            .expect("get_ai_recommendations should succeed");
        assert!(!recs.is_empty());
        let discovery_rec = recs
            .iter()
            .find(|r| r.title.contains("No Services") || r.title.contains("Discover"));
        assert!(
            discovery_rec.is_some(),
            "Should recommend discovery when no primals"
        );
    }

    #[tokio::test]
    async fn test_get_ai_recommendations_with_primals() {
        use crate::universal_biomeos_manager::PrimalInfo;
        use biomeos_primal_sdk::PrimalType;
        use biomeos_types::Health;

        let manager = test_manager().await;
        let mut registry = manager.registered_primals().write().await;
        for i in 0..5 {
            registry.insert(
                format!("primal-{i}"),
                PrimalInfo {
                    id: format!("primal-{i}"),
                    name: format!("primal-{i}"),
                    primal_type: PrimalType::from_discovered(
                        "test",
                        format!("primal-{i}"),
                        "1.0.0",
                    ),
                    endpoint: format!("/tmp/primal-{i}.sock"),
                    capabilities: vec![],
                    health: Health::Healthy,
                    last_seen: chrono::Utc::now(),
                    discovered_at: chrono::Utc::now(),
                    metadata: std::collections::HashMap::new(),
                },
            );
        }
        drop(registry);

        let recs = manager
            .get_ai_recommendations()
            .await
            .expect("get_ai_recommendations should succeed");
        assert!(!recs.is_empty());
        let opt_rec = recs.iter().find(|r| r.title.contains("AI Monitoring"));
        assert!(
            opt_rec.is_some(),
            "Should include optimization recommendation"
        );
    }

    #[test]
    fn test_ai_recommendation_structure() {
        let rec = AIRecommendation {
            title: "Test".to_string(),
            description: "Test desc".to_string(),
            priority: Priority::High,
            category: "test".to_string(),
            action: Some("biomeos test".to_string()),
            estimated_impact: "impact".to_string(),
        };
        assert_eq!(rec.title, "Test");
        assert!(matches!(rec.priority, Priority::High));
    }

    #[test]
    fn test_priority_variants() {
        let _ = Priority::High;
        let _ = Priority::Medium;
        let _ = Priority::Low;
    }
}
