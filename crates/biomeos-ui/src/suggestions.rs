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

use anyhow::{Result, Context};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::{info, debug, warn};

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
        device_id: String,
        primal_id: String,
        reason: String,
    },
    
    /// Remove a device assignment
    RemoveAssignment {
        device_id: String,
        primal_id: String,
        reason: String,
    },
    
    /// Reallocate resources
    ReallocateResources {
        from_primal: String,
        to_primal: String,
        resource_type: String,
        amount: String,
    },
    
    /// Add more capacity
    AddCapacity {
        primal_type: String,
        estimated_need: String,
    },
    
    /// Optimize configuration
    OptimizeConfig {
        primal_id: String,
        config_key: String,
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
    pub id: String,
    pub device_type: String,
    pub capabilities: Vec<String>,
    pub current_assignment: Option<String>,
}

/// Primal information for context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PrimalInfo {
    pub id: String,
    pub name: String,
    pub primal_type: String,
    pub capabilities: Vec<String>,
    pub health: String,
    pub load: Option<f32>,
}

/// User feedback on a suggestion
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SuggestionFeedback {
    /// User accepted and applied the suggestion
    Accepted,
    
    /// User rejected the suggestion with reason
    Rejected { reason: String },
    
    /// User dismissed without action
    Dismissed,
    
    /// User modified the suggestion
    Modified { changes: String },
}

/// AI Suggestion Manager
///
/// Interfaces with Squirrel AI primal to get intelligent suggestions
/// for device assignments and optimizations.
pub struct AISuggestionManager {
    /// Squirrel client (discovered via capabilities)
    squirrel_client: Option<SquirrelClientPlaceholder>,
    
    /// Family ID
    family_id: String,
    
    /// Active suggestions
    active_suggestions: HashMap<String, AISuggestion>,
}

// Placeholder for Squirrel client
// Will be replaced with actual client import
type SquirrelClientPlaceholder = ();

impl AISuggestionManager {
    /// Create a new AI suggestion manager
    pub fn new(family_id: String) -> Self {
        Self {
            squirrel_client: None,
            family_id,
            active_suggestions: HashMap::new(),
        }
    }
    
    /// Discover and connect to Squirrel
    pub async fn discover_squirrel(&mut self) -> Result<()> {
        info!("🔍 Discovering Squirrel AI primal...");
        
        // TODO: Use actual capability-based discovery
        // For now, set placeholder
        self.squirrel_client = Some(());
        
        info!("✅ Squirrel AI connected (placeholder)");
        Ok(())
    }
    
    /// Request suggestions based on current context
    pub async fn request_suggestions(&mut self, context: SuggestionContext) -> Result<Vec<AISuggestion>> {
        info!("🤖 Requesting AI suggestions from Squirrel...");
        
        if self.squirrel_client.is_none() {
            warn!("Squirrel not available, using local heuristics");
            return Ok(self.generate_local_suggestions(&context));
        }
        
        // TODO: Call actual Squirrel API
        // For now, generate local suggestions
        let suggestions = self.generate_local_suggestions(&context);
        
        // Store active suggestions
        for suggestion in &suggestions {
            self.active_suggestions.insert(suggestion.id.clone(), suggestion.clone());
        }
        
        info!("✅ Generated {} suggestions", suggestions.len());
        Ok(suggestions)
    }
    
    /// Send feedback on a suggestion to Squirrel for learning
    pub async fn send_feedback(&mut self, suggestion_id: &str, feedback: SuggestionFeedback) -> Result<()> {
        info!("📨 Sending feedback for suggestion {}: {:?}", suggestion_id, feedback);
        
        // Send to Squirrel if available
        if let Some(_squirrel) = &self.squirrel_client {
            // TODO: Send feedback to Squirrel when client method is available
            debug!("Would send feedback to Squirrel (method not yet implemented)");
        } else {
            warn!("Squirrel not available, feedback recorded locally only");
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
    fn find_compatible_primal<'a>(&self, device: &DeviceInfo, context: &'a SuggestionContext) -> Option<&'a PrimalInfo> {
        // Simple heuristic: find primal with overlapping capabilities
        context.running_primals.iter()
            .find(move |primal| {
                device.capabilities.iter()
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
        assert!(manager.squirrel_client.is_none());
    }
    
    #[tokio::test]
    async fn test_local_suggestions_unassigned_device() {
        let manager = AISuggestionManager::new("test_family".to_string());
        
        let context = SuggestionContext {
            assignments: HashMap::new(),
            available_devices: vec![
                DeviceInfo {
                    id: "device1".to_string(),
                    device_type: "gpu".to_string(),
                    capabilities: vec!["compute".to_string()],
                    current_assignment: None,
                },
            ],
            running_primals: vec![
                PrimalInfo {
                    id: "toadstool1".to_string(),
                    name: "ToadStool".to_string(),
                    primal_type: "compute".to_string(),
                    capabilities: vec!["compute".to_string()],
                    health: "healthy".to_string(),
                    load: Some(0.5),
                },
            ],
            recent_events: None,
            preferences: None,
        };
        
        let suggestions = manager.generate_local_suggestions(&context);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].suggestion_type, SuggestionType::DeviceAssignment);
    }
    
    #[tokio::test]
    async fn test_local_suggestions_overloaded_primal() {
        let manager = AISuggestionManager::new("test_family".to_string());
        
        let context = SuggestionContext {
            assignments: HashMap::new(),
            available_devices: vec![],
            running_primals: vec![
                PrimalInfo {
                    id: "toadstool1".to_string(),
                    name: "ToadStool".to_string(),
                    primal_type: "compute".to_string(),
                    capabilities: vec!["compute".to_string()],
                    health: "healthy".to_string(),
                    load: Some(0.9), // 90% load
                },
            ],
            recent_events: None,
            preferences: None,
        };
        
        let suggestions = manager.generate_local_suggestions(&context);
        assert_eq!(suggestions.len(), 1);
        assert_eq!(suggestions[0].suggestion_type, SuggestionType::ResourceReallocation);
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
        
        manager.active_suggestions.insert(suggestion.id.clone(), suggestion.clone());
        assert_eq!(manager.active_suggestions.len(), 1);
        
        // Send accepted feedback
        let result = manager.send_feedback(&suggestion.id, SuggestionFeedback::Accepted).await;
        assert!(result.is_ok());
        
        // Should be removed from active suggestions
        assert_eq!(manager.active_suggestions.len(), 0);
    }
}

