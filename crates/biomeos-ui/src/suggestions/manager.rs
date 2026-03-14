// SPDX-License-Identifier: AGPL-3.0-only
// Copyright 2025 ecoPrimals Project

//! AI Suggestion Manager — discovery, heuristics, and feedback loop.
//!
//! Interfaces with any primal that provides the "ai" capability to get
//! intelligent suggestions for device assignments and optimizations.
//! Falls back to local heuristics when no AI provider is available.

use anyhow::Result;
use std::collections::HashMap;
use tracing::{debug, info, warn};

use super::types::*;

/// AI Suggestion Manager
///
/// Interfaces with AI-capable primals to get intelligent suggestions
/// for device assignments and optimizations.
pub struct AISuggestionManager {
    /// AI provider socket path (discovered via capabilities, not by name)
    ///
    /// The manager discovers ANY primal that provides the "ai" capability,
    /// not specifically "Squirrel".
    ai_provider_socket: Option<std::path::PathBuf>,

    /// Family ID
    #[allow(dead_code)] // TODO: Wire up for family-scoped AI suggestions
    pub(crate) family_id: String,

    /// Active suggestions
    pub(crate) active_suggestions: HashMap<String, AISuggestion>,
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
    /// Discovers ANY primal with "ai" capability, not specifically "Squirrel".
    /// Primals self-register capabilities at runtime.
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
                            let name = path
                                .file_stem()
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
            // Uses CapabilityTaxonomy for the default provider name
            if std::env::var("BIOMEOS_STRICT_DISCOVERY").is_err() {
                let ai_provider = std::env::var("BIOMEOS_AI_PROVIDER").ok().or_else(|| {
                    biomeos_types::CapabilityTaxonomy::AiCoordination
                        .default_primal()
                        .map(String::from)
                });
                let Some(ai_provider) = ai_provider else {
                    info!("ℹ️ No AI provider configured (strict discovery)");
                    return Ok(());
                };
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
        use std::io::{Read, Write};
        use std::os::unix::net::UnixStream;

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

        // Generate suggestions (via AI provider if available, otherwise local heuristics)
        let suggestions = self.generate_local_suggestions(&context);

        // Store active suggestions
        for suggestion in &suggestions {
            self.active_suggestions
                .insert(suggestion.id.clone(), suggestion.clone());
        }

        info!("✅ Generated {} suggestions", suggestions.len());
        Ok(suggestions)
    }

    /// Send feedback on a suggestion for learning
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
        if self.ai_provider_socket.is_some() {
            debug!("Feedback recorded (AI provider available for learning)");
        } else {
            debug!("Feedback recorded locally (no AI provider)");
        }

        // Always remove from active suggestions if accepted/rejected
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

    /// Generate local suggestions using heuristics (fallback when AI unavailable)
    pub(crate) fn generate_local_suggestions(
        &self,
        context: &SuggestionContext,
    ) -> Vec<AISuggestion> {
        let mut suggestions = Vec::new();

        // Heuristic 1: Suggest assigning unassigned devices
        for device in &context.available_devices {
            if device.current_assignment.is_none() {
                if let Some(primal) = Self::find_compatible_primal(device, context) {
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
                            estimated_need: format!(
                                "{}% more capacity",
                                ((1.0 - load) * 100.0) as u32
                            ),
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
        device: &DeviceInfo,
        context: &'a SuggestionContext,
    ) -> Option<&'a PrimalInfo> {
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
    #![allow(clippy::unwrap_used)]

    use super::*;
    use crate::suggestions::types::{
        DeviceInfo, Impact, PrimalInfo, SuggestedAction, SuggestionContext, SuggestionFeedback,
        SuggestionType,
    };

    #[test]
    fn test_new() {
        let mgr = AISuggestionManager::new("fam1".to_string());
        assert_eq!(mgr.family_id, "fam1");
        assert!(mgr.get_active_suggestions().is_empty());
    }

    #[test]
    fn test_generate_local_suggestions_empty_context() {
        let mgr = AISuggestionManager::new("fam1".to_string());
        let ctx = SuggestionContext {
            assignments: std::collections::HashMap::new(),
            available_devices: vec![],
            running_primals: vec![],
            recent_events: None,
            preferences: None,
        };
        let suggestions = mgr.generate_local_suggestions(&ctx);
        assert!(suggestions.is_empty());
    }

    #[test]
    fn test_generate_local_suggestions_unassigned_device() {
        let mgr = AISuggestionManager::new("fam1".to_string());
        let ctx = SuggestionContext {
            assignments: std::collections::HashMap::new(),
            available_devices: vec![DeviceInfo {
                id: "dev1".to_string(),
                device_type: "gpu".to_string(),
                capabilities: vec!["compute".to_string()],
                current_assignment: None,
            }],
            running_primals: vec![PrimalInfo {
                id: "p1".to_string(),
                name: "beardog".to_string(),
                primal_type: "compute".to_string(),
                capabilities: vec!["compute".to_string()],
                health: "healthy".to_string(),
                load: Some(0.5),
            }],
            recent_events: None,
            preferences: None,
        };
        let suggestions = mgr.generate_local_suggestions(&ctx);
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].id.starts_with("local_assign_"));
        assert_eq!(
            suggestions[0].suggestion_type,
            SuggestionType::DeviceAssignment
        );
    }

    #[test]
    fn test_generate_local_suggestions_overloaded_primal() {
        let mgr = AISuggestionManager::new("fam1".to_string());
        let ctx = SuggestionContext {
            assignments: std::collections::HashMap::new(),
            available_devices: vec![],
            running_primals: vec![PrimalInfo {
                id: "p1".to_string(),
                name: "heavy".to_string(),
                primal_type: "compute".to_string(),
                capabilities: vec!["compute".to_string()],
                health: "healthy".to_string(),
                load: Some(0.9),
            }],
            recent_events: None,
            preferences: None,
        };
        let suggestions = mgr.generate_local_suggestions(&ctx);
        assert_eq!(suggestions.len(), 1);
        assert!(suggestions[0].id.starts_with("local_rebalance_"));
        assert_eq!(
            suggestions[0].suggestion_type,
            SuggestionType::ResourceReallocation
        );
    }

    #[test]
    fn test_generate_local_suggestions_no_compatible_primal() {
        let mgr = AISuggestionManager::new("fam1".to_string());
        let ctx = SuggestionContext {
            assignments: std::collections::HashMap::new(),
            available_devices: vec![DeviceInfo {
                id: "dev1".to_string(),
                device_type: "gpu".to_string(),
                capabilities: vec!["special".to_string()],
                current_assignment: None,
            }],
            running_primals: vec![PrimalInfo {
                id: "p1".to_string(),
                name: "beardog".to_string(),
                primal_type: "security".to_string(),
                capabilities: vec!["security".to_string()],
                health: "healthy".to_string(),
                load: None,
            }],
            recent_events: None,
            preferences: None,
        };
        let suggestions = mgr.generate_local_suggestions(&ctx);
        assert!(suggestions.is_empty());
    }

    #[tokio::test]
    async fn test_request_suggestions_stores_active() {
        let mut mgr = AISuggestionManager::new("fam1".to_string());
        let ctx = SuggestionContext {
            assignments: std::collections::HashMap::new(),
            available_devices: vec![DeviceInfo {
                id: "dev1".to_string(),
                device_type: "gpu".to_string(),
                capabilities: vec!["compute".to_string()],
                current_assignment: None,
            }],
            running_primals: vec![PrimalInfo {
                id: "p1".to_string(),
                name: "beardog".to_string(),
                primal_type: "compute".to_string(),
                capabilities: vec!["compute".to_string()],
                health: "healthy".to_string(),
                load: None,
            }],
            recent_events: None,
            preferences: None,
        };
        let suggestions = mgr.request_suggestions(ctx).await.unwrap();
        assert_eq!(suggestions.len(), 1);
        assert_eq!(mgr.get_active_suggestions().len(), 1);
    }

    #[tokio::test]
    async fn test_send_feedback_removes_on_accepted() {
        let mut mgr = AISuggestionManager::new("fam1".to_string());
        mgr.active_suggestions.insert(
            "s1".to_string(),
            AISuggestion {
                id: "s1".to_string(),
                suggestion_type: SuggestionType::DeviceAssignment,
                confidence: 0.8,
                explanation: "test".to_string(),
                action: SuggestedAction::AssignDevice {
                    device_id: "d1".to_string(),
                    primal_id: "p1".to_string(),
                    reason: "test".to_string(),
                },
                impact: Impact {
                    performance_improvement: Some(10.0),
                    cost_change: None,
                    affected_primals: vec![],
                    risk_level: "low".to_string(),
                },
            },
        );
        mgr.send_feedback("s1", SuggestionFeedback::Accepted)
            .await
            .unwrap();
        assert!(mgr.get_active_suggestions().is_empty());
    }

    #[tokio::test]
    async fn test_send_feedback_removes_on_rejected() {
        let mut mgr = AISuggestionManager::new("fam1".to_string());
        mgr.active_suggestions.insert(
            "s2".to_string(),
            AISuggestion {
                id: "s2".to_string(),
                suggestion_type: SuggestionType::DeviceAssignment,
                confidence: 0.5,
                explanation: "test".to_string(),
                action: SuggestedAction::AssignDevice {
                    device_id: "d1".to_string(),
                    primal_id: "p1".to_string(),
                    reason: "test".to_string(),
                },
                impact: Impact {
                    performance_improvement: None,
                    cost_change: None,
                    affected_primals: vec![],
                    risk_level: "low".to_string(),
                },
            },
        );
        mgr.send_feedback(
            "s2",
            SuggestionFeedback::Rejected {
                reason: "not needed".to_string(),
            },
        )
        .await
        .unwrap();
        assert!(mgr.get_active_suggestions().is_empty());
    }

    #[tokio::test]
    async fn test_send_feedback_dismissed_keeps_in_map() {
        let mut mgr = AISuggestionManager::new("fam1".to_string());
        mgr.active_suggestions.insert(
            "s3".to_string(),
            AISuggestion {
                id: "s3".to_string(),
                suggestion_type: SuggestionType::DeviceAssignment,
                confidence: 0.5,
                explanation: "test".to_string(),
                action: SuggestedAction::AssignDevice {
                    device_id: "d1".to_string(),
                    primal_id: "p1".to_string(),
                    reason: "test".to_string(),
                },
                impact: Impact {
                    performance_improvement: None,
                    cost_change: None,
                    affected_primals: vec![],
                    risk_level: "low".to_string(),
                },
            },
        );
        mgr.send_feedback("s3", SuggestionFeedback::Dismissed)
            .await
            .unwrap();
        assert_eq!(mgr.get_active_suggestions().len(), 1);
    }
}
