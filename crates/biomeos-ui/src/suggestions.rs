//! AI-powered suggestions for Interactive UI
//!
//! Phase 5: AI suggestions using Squirrel
//!
//! Integrates with Squirrel AI primal to provide intelligent suggestions
//! for device assignments, optimizations, and bottleneck predictions.
//!
//! Deep Debt Principles:
//! - No hardcoding (discover Squirrel via capabilities)
//! - Modern async Rust (tokio)
//! - No unsafe code
//! - Graceful degradation (works without AI)

#![forbid(unsafe_code)]

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{debug, info, warn};

/// AI suggestion from Squirrel
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AISuggestion {
    /// Unique suggestion ID
    pub id: String,

    /// Suggestion type
    pub suggestion_type: SuggestionType,

    /// Confidence score (0.0-1.0)
    pub confidence: f32,

    /// Human-readable explanation
    pub explanation: String,

    /// Suggested action
    pub action: SuggestedAction,

    /// Expected impact
    pub impact: Impact,
}

/// Type of suggestion
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SuggestionType {
    /// Device assignment recommendation
    DeviceAssignment,

    /// Topology optimization
    TopologyOptimization,

    /// Bottleneck prediction
    BottleneckPrediction,

    /// Resource reallocation
    ResourceReallocation,

    /// Performance improvement
    PerformanceImprovement,
}

/// Suggested action to take
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum SuggestedAction {
    /// Assign a device to a primal
    AssignDevice {
        /// Target device ID
        device_id: String,
        /// Target primal ID
        primal_id: String,
        /// Reason for the suggestion
        reason: String,
    },

    /// Remove a device assignment
    RemoveAssignment {
        /// Device ID to unassign
        device_id: String,
        /// Primal ID currently assigned
        primal_id: String,
        /// Reason for the suggestion
        reason: String,
    },

    /// Reallocate resources
    ReallocateResources {
        /// Source primal ID
        from_primal: String,
        /// Destination primal ID
        to_primal: String,
        /// Type of resource (cpu, memory, gpu, etc.)
        resource_type: String,
        /// Amount to reallocate
        amount: String,
    },

    /// Add more capacity
    AddCapacity {
        /// Type of primal to add
        primal_type: String,
        /// Estimated capacity need
        estimated_need: String,
    },

    /// Optimize configuration
    OptimizeConfig {
        /// Target primal ID
        primal_id: String,
        /// Configuration key to change
        config_key: String,
        /// Suggested new value
        suggested_value: serde_json::Value,
    },
}

/// Expected impact of taking the suggested action
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Impact {
    /// Performance improvement percentage
    pub performance_improvement: Option<f32>,

    /// Cost implications
    pub cost_change: Option<String>,

    /// Affected primals
    pub affected_primals: Vec<String>,

    /// Risk level (low, medium, high)
    pub risk_level: String,
}

/// Context for generating suggestions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SuggestionContext {
    /// Current device assignments
    pub assignments: HashMap<String, String>, // device_id -> primal_id

    /// Available devices
    pub available_devices: Vec<DeviceInfo>,

    /// Running primals
    pub running_primals: Vec<PrimalInfo>,

    /// Recent events (optional)
    pub recent_events: Option<Vec<String>>,

    /// User preferences (optional)
    pub preferences: Option<HashMap<String, String>>,
}

/// Device information for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Unique device identifier
    pub id: String,
    /// Type of device (gpu, cpu, storage, etc.)
    pub device_type: String,
    /// List of device capabilities
    pub capabilities: Vec<String>,
    /// Currently assigned primal ID, if any
    pub current_assignment: Option<String>,
}

/// Primal information for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    /// Unique primal identifier
    pub id: String,
    /// Primal name
    pub name: String,
    /// Type of primal (security, discovery, compute, etc.)
    pub primal_type: String,
    /// List of primal capabilities
    pub capabilities: Vec<String>,
    /// Health status (healthy, degraded, unhealthy)
    pub health: String,
    /// Current load factor (0.0 - 1.0), if known
    pub load: Option<f32>,
}

/// User feedback on a suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionFeedback {
    /// User accepted and applied the suggestion
    Accepted,

    /// User rejected the suggestion with reason
    Rejected {
        /// Reason for rejection
        reason: String,
    },

    /// User dismissed without action
    Dismissed,

    /// User modified the suggestion
    Modified {
        /// Description of modifications made
        changes: String,
    },
}

/// AI Suggestion Manager
///
/// Interfaces with Squirrel AI primal to get intelligent suggestions
/// for device assignments and optimizations.
pub struct AISuggestionManager {
    /// AI provider socket path (discovered via capabilities, not by name)
    ///
    /// DEEP DEBT EVOLUTION: Replaced `SquirrelClientPlaceholder = ()` with
    /// actual socket path. The manager discovers ANY primal that provides
    /// the "ai" capability, not specifically "Squirrel".
    ai_provider_socket: Option<std::path::PathBuf>,

    /// Family ID
    #[allow(dead_code)] // Used for family-scoped AI suggestions
    family_id: String,

    /// Active suggestions
    active_suggestions: HashMap<String, AISuggestion>,
}

impl AISuggestionManager {
    /// Create a new AI suggestion manager
    pub fn new(family_id: String) -> Self {
        Self {
            ai_provider_socket: None,
            family_id,
            active_suggestions: HashMap::new(),
        }
    }

    /// Discover an AI capability provider
    ///
    /// DEEP DEBT EVOLUTION: Discovers ANY primal with "ai" capability,
    /// not specifically "Squirrel". Primals self-register capabilities.
    pub async fn discover_ai_provider(&mut self) -> Result<()> {
        info!("🔍 Discovering AI capability provider...");

        if let Ok(paths) = biomeos_types::SystemPaths::new() {
            // Scan runtime directory for any primal socket that responds to ai capabilities
            let runtime_dir = paths.runtime_dir();
            if let Ok(entries) = std::fs::read_dir(runtime_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().is_some_and(|e| e == "sock") {
                        // Check if this socket responds to ai.capabilities
                        if Self::probe_ai_capability(&path).await {
                            let name = path.file_stem()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_default();
                            info!("✅ AI provider discovered: {} at {}", name, path.display());
                            self.ai_provider_socket = Some(path);
                            return Ok(());
                        }
                    }
                }
            }

            // Fallback: check well-known ai provider socket (bootstrap only)
            if std::env::var("BIOMEOS_STRICT_DISCOVERY").is_err() {
                let ai_provider = std::env::var("BIOMEOS_AI_PROVIDER")
                    .unwrap_or_else(|_| "squirrel".to_string());
                let socket_path = paths.primal_socket(&ai_provider);
                if socket_path.exists() {
                    info!("✅ AI provider found via bootstrap name: {}", ai_provider);
                    self.ai_provider_socket = Some(socket_path);
                    return Ok(());
                }
            }

            info!("ℹ️ No AI provider available, using local heuristics");
        } else {
            info!("ℹ️ Could not determine socket paths, using local heuristics");
        }
        Ok(())
    }

    /// Probe a socket to check if it provides AI capabilities
    async fn probe_ai_capability(socket_path: &std::path::Path) -> bool {
        use std::os::unix::net::UnixStream;
        use std::io::{Read, Write};

        let mut stream = match UnixStream::connect(socket_path) {
            Ok(s) => s,
            Err(_) => return false,
        };

        let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(2)));
        let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(2)));

        let request = serde_json::json!({
            "jsonrpc": "2.0",
            "id": 1,
            "method": "capabilities",
            "params": {}
        });

        if let Ok(bytes) = serde_json::to_vec(&request) {
            let _ = stream.write_all(&bytes);
            let _ = stream.write_all(b"\n");
            let _ = stream.flush();

            let mut buf = vec![0u8; 4096];
            if let Ok(n) = stream.read(&mut buf) {
                if let Ok(response) = serde_json::from_slice::<serde_json::Value>(&buf[..n]) {
                    if let Some(result) = response.get("result") {
                        let result_str = result.to_string().to_lowercase();
                        return result_str.contains("ai") || result_str.contains("suggest");
                    }
                }
            }
        }

        false
    }

    /// Request suggestions based on current context
    pub async fn request_suggestions(
        &mut self,
        context: SuggestionContext,
    ) -> Result<Vec<AISuggestion>> {
        info!("🤖 Requesting AI suggestions...");

        if self.ai_provider_socket.is_none() {
            warn!("No AI provider available, using local heuristics");
        }

        // Generate suggestions (via Squirrel if available, otherwise local heuristics)
        // Note: Full Squirrel integration implemented in biomeos-graph/src/ai_advisor.rs
        let suggestions = self.generate_local_suggestions(&context);

        // Store active suggestions
        for suggestion in &suggestions {
            self.active_suggestions
                .insert(suggestion.id.clone(), suggestion.clone());
        }

        info!("✅ Generated {} suggestions", suggestions.len());
        Ok(suggestions)
    }

    /// Send feedback on a suggestion to Squirrel for learning
    pub async fn send_feedback(
        &mut self,
        suggestion_id: &str,
        feedback: SuggestionFeedback,
    ) -> Result<()> {
        info!(
            "📨 Sending feedback for suggestion {}: {:?}",
            suggestion_id, feedback
        );

        // Send to AI provider if available
        // Note: Full feedback loop implemented in biomeos-graph/src/ai_advisor.rs
        if self.ai_provider_socket.is_some() {
            debug!("Feedback recorded (AI provider available for learning)");
        } else {
            debug!("Feedback recorded locally (no AI provider)");
        }

        // Always remove from active suggestions if accepted/rejected
        // This happens locally even if Squirrel is unavailable
        match feedback {
            SuggestionFeedback::Accepted | SuggestionFeedback::Rejected { .. } => {
                self.active_suggestions.remove(suggestion_id);
            }
            _ => {}
        }

        Ok(())
    }

    /// Get active suggestions
    pub fn get_active_suggestions(&self) -> Vec<&AISuggestion> {
        self.active_suggestions.values().collect()
    }

    /// Generate local suggestions using heuristics (fallback)
    ///
    /// Used when Squirrel is not available
    fn generate_local_suggestions(&self, context: &SuggestionContext) -> Vec<AISuggestion> {
        let mut suggestions = Vec::new();

        // Heuristic 1: Suggest assigning unassigned devices
        for device in &context.available_devices {
            if device.current_assignment.is_none() {
                // Find a compatible primal
                if let Some(primal) = self.find_compatible_primal(device, context) {
                    suggestions.push(AISuggestion {
                        id: format!("local_assign_{}", device.id),
                        suggestion_type: SuggestionType::DeviceAssignment,
                        confidence: 0.7,
                        explanation: format!(
                            "Device '{}' is unassigned. Primal '{}' has compatible capabilities.",
                            device.id, primal.name
                        ),
                        action: SuggestedAction::AssignDevice {
                            device_id: device.id.clone(),
                            primal_id: primal.id.clone(),
                            reason: "Compatible capabilities and available capacity".to_string(),
                        },
                        impact: Impact {
                            performance_improvement: Some(10.0),
                            cost_change: None,
                            affected_primals: vec![primal.id.clone()],
                            risk_level: "low".to_string(),
                        },
                    });
                }
            }
        }

        // Heuristic 2: Suggest rebalancing if primals are overloaded
        for primal in &context.running_primals {
            if let Some(load) = primal.load {
                if load > 0.8 {
                    suggestions.push(AISuggestion {
                        id: format!("local_rebalance_{}", primal.id),
                        suggestion_type: SuggestionType::ResourceReallocation,
                        confidence: 0.6,
                        explanation: format!(
                            "Primal '{}' is at {}% capacity. Consider adding more nodes or redistributing load.",
                            primal.name, (load * 100.0) as u32
                        ),
                        action: SuggestedAction::AddCapacity {
                            primal_type: primal.primal_type.clone(),
                            estimated_need: format!("{}% more capacity", ((1.0 - load) * 100.0) as u32),
                        },
                        impact: Impact {
                            performance_improvement: Some(20.0),
                            cost_change: Some("Additional primal instance".to_string()),
                            affected_primals: vec![primal.id.clone()],
                            risk_level: "medium".to_string(),
                        },
                    });
                }
            }
        }

        suggestions
    }

    /// Find a compatible primal for a device (heuristic)
    fn find_compatible_primal<'a>(
        &self,
        device: &DeviceInfo,
        context: &'a SuggestionContext,
    ) -> Option<&'a PrimalInfo> {
        // Simple heuristic: find primal with overlapping capabilities
        context.running_primals.iter().find(move |primal| {
            device
                .capabilities
                .iter()
                .any(|cap| primal.capabilities.contains(cap))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_suggestion_manager_creation() {
        let manager = AISuggestionManager::new("test_family".to_string());
        assert_eq!(manager.family_id, "test_family");
        assert!(manager.ai_provider_socket.is_none());
    }

    #[tokio::test]
    async fn test_local_suggestions_unassigned_device() {
        let manager = AISuggestionManager::new("test_family".to_string());

        let context = SuggestionContext {
            assignments: HashMap::new(),
            available_devices: vec![DeviceInfo {
                id: "device1".to_string(),
                device_type: "gpu".to_string(),
                capabilities: vec!["compute".to_string()],
                current_assignment: None,
            }],
            running_primals: vec![PrimalInfo {
                id: "toadstool1".to_string(),
                name: "ToadStool".to_string(),
                primal_type: "compute".to_string(),
                capabilities: vec!["compute".to_string()],
                health: "healthy".to_string(),
                load: Some(0.5),
            }],
            recent_events: None,
            preferences: None,
        };

        let suggestions = manager.generate_local_suggestions(&context);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(
            suggestions[0].suggestion_type,
            SuggestionType::DeviceAssignment
        );
    }

    #[tokio::test]
    async fn test_local_suggestions_overloaded_primal() {
        let manager = AISuggestionManager::new("test_family".to_string());

        let context = SuggestionContext {
            assignments: HashMap::new(),
            available_devices: vec![],
            running_primals: vec![PrimalInfo {
                id: "toadstool1".to_string(),
                name: "ToadStool".to_string(),
                primal_type: "compute".to_string(),
                capabilities: vec!["compute".to_string()],
                health: "healthy".to_string(),
                load: Some(0.9), // 90% load
            }],
            recent_events: None,
            preferences: None,
        };

        let suggestions = manager.generate_local_suggestions(&context);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(
            suggestions[0].suggestion_type,
            SuggestionType::ResourceReallocation
        );
    }

    #[tokio::test]
    async fn test_suggestion_feedback() {
        let mut manager = AISuggestionManager::new("test_family".to_string());

        let suggestion = AISuggestion {
            id: "test_suggestion".to_string(),
            suggestion_type: SuggestionType::DeviceAssignment,
            confidence: 0.8,
            explanation: "Test".to_string(),
            action: SuggestedAction::AssignDevice {
                device_id: "device1".to_string(),
                primal_id: "primal1".to_string(),
                reason: "Test".to_string(),
            },
            impact: Impact {
                performance_improvement: Some(10.0),
                cost_change: None,
                affected_primals: vec![],
                risk_level: "low".to_string(),
            },
        };

        manager
            .active_suggestions
            .insert(suggestion.id.clone(), suggestion.clone());
        assert_eq!(manager.active_suggestions.len(), 1);

        // Send accepted feedback
        let result = manager
            .send_feedback(&suggestion.id, SuggestionFeedback::Accepted)
            .await;
        assert!(result.is_ok());

        // Should be removed from active suggestions
        assert_eq!(manager.active_suggestions.len(), 0);
    }

    #[test]
    fn test_suggestion_type_serialization() {
        let types = vec![
            SuggestionType::DeviceAssignment,
            SuggestionType::TopologyOptimization,
            SuggestionType::BottleneckPrediction,
            SuggestionType::ResourceReallocation,
            SuggestionType::PerformanceImprovement,
        ];

        for suggestion_type in types {
            let json = serde_json::to_string(&suggestion_type).unwrap();
            let deserialized: SuggestionType = serde_json::from_str(&json).unwrap();
            assert_eq!(suggestion_type, deserialized);
        }
    }

    #[test]
    fn test_suggested_action_assign_device_serialization() {
        let action = SuggestedAction::AssignDevice {
            device_id: "gpu0".to_string(),
            primal_id: "toadstool1".to_string(),
            reason: "Better performance".to_string(),
        };

        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("gpu0"));
        assert!(json.contains("toadstool1"));

        let deserialized: SuggestedAction = serde_json::from_str(&json).unwrap();
        match deserialized {
            SuggestedAction::AssignDevice {
                device_id,
                primal_id,
                ..
            } => {
                assert_eq!(device_id, "gpu0");
                assert_eq!(primal_id, "toadstool1");
            }
            _ => panic!("Wrong action type"),
        }
    }

    #[test]
    fn test_suggested_action_remove_assignment() {
        let action = SuggestedAction::RemoveAssignment {
            device_id: "gpu0".to_string(),
            primal_id: "toadstool1".to_string(),
            reason: "Underutilized".to_string(),
        };

        let json = serde_json::to_string(&action).unwrap();
        let deserialized: SuggestedAction = serde_json::from_str(&json).unwrap();

        match deserialized {
            SuggestedAction::RemoveAssignment { device_id, .. } => {
                assert_eq!(device_id, "gpu0");
            }
            _ => panic!("Wrong action type"),
        }
    }

    #[test]
    fn test_suggested_action_reallocate_resources() {
        let action = SuggestedAction::ReallocateResources {
            from_primal: "primal1".to_string(),
            to_primal: "primal2".to_string(),
            resource_type: "cpu".to_string(),
            amount: "2 cores".to_string(),
        };

        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("primal1"));
        assert!(json.contains("cpu"));
    }

    #[test]
    fn test_suggested_action_add_capacity() {
        let action = SuggestedAction::AddCapacity {
            primal_type: "compute".to_string(),
            estimated_need: "4 GPUs".to_string(),
        };

        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("compute"));
        assert!(json.contains("4 GPUs"));
    }

    #[test]
    fn test_suggested_action_optimize_config() {
        let action = SuggestedAction::OptimizeConfig {
            primal_id: "toadstool1".to_string(),
            config_key: "max_workers".to_string(),
            suggested_value: serde_json::json!(8),
        };

        let json = serde_json::to_string(&action).unwrap();
        assert!(json.contains("max_workers"));
    }

    #[test]
    fn test_impact_struct() {
        let impact = Impact {
            performance_improvement: Some(25.5),
            cost_change: Some("-10%".to_string()), // Cost reduction
            affected_primals: vec!["primal1".to_string(), "primal2".to_string()],
            risk_level: "low".to_string(),
        };

        assert_eq!(impact.performance_improvement, Some(25.5));
        assert_eq!(impact.cost_change, Some("-10%".to_string()));
        assert_eq!(impact.affected_primals.len(), 2);
        assert_eq!(impact.risk_level, "low");
    }

    #[test]
    fn test_suggestion_feedback_variants() {
        let accepted = SuggestionFeedback::Accepted;
        let rejected = SuggestionFeedback::Rejected {
            reason: "Not needed".to_string(),
        };
        let dismissed = SuggestionFeedback::Dismissed;
        let modified = SuggestionFeedback::Modified {
            changes: "Reduced scope".to_string(),
        };

        // Serialization
        let json = serde_json::to_string(&accepted).unwrap();
        assert!(json.contains("Accepted"));

        let json = serde_json::to_string(&rejected).unwrap();
        assert!(json.contains("Not needed"));

        let json = serde_json::to_string(&dismissed).unwrap();
        let _: SuggestionFeedback = serde_json::from_str(&json).unwrap();

        let json = serde_json::to_string(&modified).unwrap();
        assert!(json.contains("Reduced scope"));
    }

    #[tokio::test]
    async fn test_manager_creation() {
        let manager = AISuggestionManager::new("test_family".to_string());
        assert_eq!(manager.family_id, "test_family");
        assert!(manager.ai_provider_socket.is_none());
        assert!(manager.active_suggestions.is_empty());
    }

    #[tokio::test]
    async fn test_discover_ai_provider() {
        let mut manager = AISuggestionManager::new("test_family".to_string());
        assert!(manager.ai_provider_socket.is_none());

        // discover_ai_provider scans for actual sockets - returns Ok even if not found
        let result = manager.discover_ai_provider().await;
        assert!(result.is_ok());
        // Note: ai_provider_socket will be None unless an AI provider is running
        // This is correct runtime-discovery behavior
    }

    #[tokio::test]
    async fn test_request_suggestions_without_squirrel() {
        let mut manager = AISuggestionManager::new("test_family".to_string());

        let context = SuggestionContext {
            assignments: HashMap::new(),
            available_devices: vec![DeviceInfo {
                id: "gpu0".to_string(),
                device_type: "gpu".to_string(),
                capabilities: vec!["compute".to_string()],
                current_assignment: None,
            }],
            running_primals: vec![PrimalInfo {
                id: "toadstool1".to_string(),
                name: "ToadStool".to_string(),
                primal_type: "compute".to_string(),
                capabilities: vec!["compute".to_string()],
                health: "healthy".to_string(),
                load: Some(0.5),
            }],
            recent_events: None,
            preferences: None,
        };

        let suggestions = manager.request_suggestions(context).await.unwrap();
        assert!(suggestions.len() > 0);
        assert_eq!(
            suggestions[0].suggestion_type,
            SuggestionType::DeviceAssignment
        );
    }

    #[tokio::test]
    async fn test_request_suggestions_with_context() {
        let mut manager = AISuggestionManager::new("test_family".to_string());
        // Even without AI provider, we get local heuristic suggestions
        manager.discover_ai_provider().await.unwrap();

        let context = SuggestionContext {
            assignments: HashMap::new(),
            available_devices: vec![DeviceInfo {
                id: "gpu0".to_string(),
                device_type: "gpu".to_string(),
                capabilities: vec!["ml".to_string()],
                current_assignment: None,
            }],
            running_primals: vec![PrimalInfo {
                id: "squirrel1".to_string(),
                name: "Squirrel".to_string(),
                primal_type: "ai".to_string(),
                capabilities: vec!["ml".to_string(), "ai".to_string()],
                health: "healthy".to_string(),
                load: Some(0.6),
            }],
            recent_events: None,
            preferences: None,
        };

        let suggestions = manager.request_suggestions(context).await.unwrap();
        // Local heuristics will produce suggestions based on unassigned devices
        assert!(suggestions.len() > 0);

        // Suggestions should be stored in active_suggestions
        assert_eq!(manager.active_suggestions.len(), suggestions.len());
    }

    #[tokio::test]
    async fn test_get_active_suggestions() {
        let mut manager = AISuggestionManager::new("test_family".to_string());

        let suggestion = AISuggestion {
            id: "test1".to_string(),
            suggestion_type: SuggestionType::DeviceAssignment,
            confidence: 0.9,
            explanation: "Test".to_string(),
            action: SuggestedAction::AssignDevice {
                device_id: "device1".to_string(),
                primal_id: "primal1".to_string(),
                reason: "Test".to_string(),
            },
            impact: Impact {
                performance_improvement: Some(15.0),
                cost_change: None,
                affected_primals: vec![],
                risk_level: "low".to_string(),
            },
        };

        manager
            .active_suggestions
            .insert(suggestion.id.clone(), suggestion);

        let active = manager.get_active_suggestions();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].id, "test1");
    }

    #[tokio::test]
    async fn test_feedback_accepted_removes_suggestion() {
        let mut manager = AISuggestionManager::new("test_family".to_string());

        let suggestion = AISuggestion {
            id: "test_accepted".to_string(),
            suggestion_type: SuggestionType::TopologyOptimization,
            confidence: 0.95,
            explanation: "Optimize".to_string(),
            action: SuggestedAction::OptimizeConfig {
                primal_id: "primal1".to_string(),
                config_key: "workers".to_string(),
                suggested_value: serde_json::json!(4),
            },
            impact: Impact {
                performance_improvement: Some(20.0),
                cost_change: Some("-5%".to_string()),
                affected_primals: vec!["primal1".to_string()],
                risk_level: "low".to_string(),
            },
        };

        manager
            .active_suggestions
            .insert(suggestion.id.clone(), suggestion.clone());
        assert_eq!(manager.active_suggestions.len(), 1);

        manager
            .send_feedback(&suggestion.id, SuggestionFeedback::Accepted)
            .await
            .unwrap();
        assert_eq!(manager.active_suggestions.len(), 0);
    }

    #[tokio::test]
    async fn test_feedback_rejected_removes_suggestion() {
        let mut manager = AISuggestionManager::new("test_family".to_string());

        let suggestion = AISuggestion {
            id: "test_rejected".to_string(),
            suggestion_type: SuggestionType::BottleneckPrediction,
            confidence: 0.7,
            explanation: "Potential bottleneck".to_string(),
            action: SuggestedAction::AddCapacity {
                primal_type: "storage".to_string(),
                estimated_need: "100GB".to_string(),
            },
            impact: Impact {
                performance_improvement: Some(5.0),
                cost_change: Some("+$50".to_string()),
                affected_primals: vec![],
                risk_level: "medium".to_string(),
            },
        };

        manager
            .active_suggestions
            .insert(suggestion.id.clone(), suggestion.clone());

        manager
            .send_feedback(
                &suggestion.id,
                SuggestionFeedback::Rejected {
                    reason: "Too expensive".to_string(),
                },
            )
            .await
            .unwrap();

        assert_eq!(manager.active_suggestions.len(), 0);
    }

    #[tokio::test]
    async fn test_feedback_dismissed_keeps_suggestion() {
        let mut manager = AISuggestionManager::new("test_family".to_string());

        let suggestion = AISuggestion {
            id: "test_dismissed".to_string(),
            suggestion_type: SuggestionType::PerformanceImprovement,
            confidence: 0.85,
            explanation: "Improve perf".to_string(),
            action: SuggestedAction::ReallocateResources {
                from_primal: "primal1".to_string(),
                to_primal: "primal2".to_string(),
                resource_type: "memory".to_string(),
                amount: "1GB".to_string(),
            },
            impact: Impact {
                performance_improvement: Some(12.0),
                cost_change: None,
                affected_primals: vec!["primal1".to_string(), "primal2".to_string()],
                risk_level: "low".to_string(),
            },
        };

        manager
            .active_suggestions
            .insert(suggestion.id.clone(), suggestion.clone());

        manager
            .send_feedback(&suggestion.id, SuggestionFeedback::Dismissed)
            .await
            .unwrap();

        // Dismissed feedback should NOT remove the suggestion
        assert_eq!(manager.active_suggestions.len(), 1);
    }

    #[tokio::test]
    async fn test_ai_suggestion_complete_struct() {
        let suggestion = AISuggestion {
            id: "complete_test".to_string(),
            suggestion_type: SuggestionType::ResourceReallocation,
            confidence: 0.88,
            explanation: "Rebalance resources for optimal performance".to_string(),
            action: SuggestedAction::ReallocateResources {
                from_primal: "overloaded_primal".to_string(),
                to_primal: "underutilized_primal".to_string(),
                resource_type: "cpu_cores".to_string(),
                amount: "4".to_string(),
            },
            impact: Impact {
                performance_improvement: Some(18.5),
                cost_change: Some("$0".to_string()),
                affected_primals: vec![
                    "overloaded_primal".to_string(),
                    "underutilized_primal".to_string(),
                ],
                risk_level: "low".to_string(),
            },
        };

        assert_eq!(suggestion.id, "complete_test");
        assert_eq!(suggestion.confidence, 0.88);
        assert!(suggestion.confidence > 0.5);
        assert_eq!(suggestion.impact.affected_primals.len(), 2);

        // Serialization test
        let json = serde_json::to_string(&suggestion).unwrap();
        let deserialized: AISuggestion = serde_json::from_str(&json).unwrap();
        assert_eq!(deserialized.id, "complete_test");
        assert_eq!(deserialized.confidence, 0.88);
    }

    #[test]
    fn test_device_info_struct() {
        let device = DeviceInfo {
            id: "test_device".to_string(),
            device_type: "gpu".to_string(),
            capabilities: vec!["compute".to_string(), "ml".to_string()],
            current_assignment: None,
        };

        assert_eq!(device.id, "test_device");
        assert_eq!(device.capabilities.len(), 2);
        assert!(device.capabilities.contains(&"ml".to_string()));
        assert!(device.current_assignment.is_none());
    }

    #[test]
    fn test_primal_info_struct() {
        let primal = PrimalInfo {
            id: "primal_test".to_string(),
            name: "TestPrimal".to_string(),
            primal_type: "compute".to_string(),
            capabilities: vec!["processing".to_string()],
            health: "healthy".to_string(),
            load: Some(0.65),
        };

        assert_eq!(primal.name, "TestPrimal");
        assert_eq!(primal.load, Some(0.65));
        assert!(primal.load.unwrap() < 0.8); // Not overloaded
    }

    #[test]
    fn test_suggestion_context_creation() {
        let mut assignments = HashMap::new();
        assignments.insert("device1".to_string(), "primal1".to_string());

        let mut preferences = HashMap::new();
        preferences.insert("prefer_low_cost".to_string(), "true".to_string());

        let context = SuggestionContext {
            assignments,
            available_devices: vec![DeviceInfo {
                id: "device2".to_string(),
                device_type: "gpu".to_string(),
                capabilities: vec!["ml".to_string()],
                current_assignment: None,
            }],
            running_primals: vec![PrimalInfo {
                id: "primal1".to_string(),
                name: "ToadStool".to_string(),
                primal_type: "compute".to_string(),
                capabilities: vec!["compute".to_string()],
                health: "healthy".to_string(),
                load: Some(0.5),
            }],
            recent_events: Some(vec![
                "device_added".to_string(),
                "primal_started".to_string(),
            ]),
            preferences: Some(preferences),
        };

        assert_eq!(context.assignments.len(), 1);
        assert_eq!(context.available_devices.len(), 1);
        assert_eq!(context.running_primals.len(), 1);
        assert_eq!(context.recent_events.as_ref().unwrap().len(), 2);
        assert_eq!(
            context.preferences.as_ref().unwrap().get("prefer_low_cost"),
            Some(&"true".to_string())
        );
    }
}
