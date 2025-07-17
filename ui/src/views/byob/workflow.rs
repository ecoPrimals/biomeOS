//! BYOB Workflow Management
//!
//! This module handles workflow state management, validation, and transitions
//! for the BYOB system. The workflow is completely universal and capability-based.

use super::types::*;
use std::collections::HashMap;

/// Workflow state manager
pub struct WorkflowManager {
    state: WorkflowState,
    validation_errors: Vec<String>,
}

impl WorkflowManager {
    pub fn new() -> Self {
        Self {
            state: WorkflowState::TeamSelection,
            validation_errors: Vec::new(),
        }
    }

    pub fn get_state(&self) -> &WorkflowState {
        &self.state
    }

    pub fn set_state(&mut self, state: WorkflowState) {
        self.state = state;
        self.validation_errors.clear();
    }

    pub fn get_validation_errors(&self) -> &[String] {
        &self.validation_errors
    }

    /// Validate current state and return whether we can proceed
    pub fn validate_current_state(
        &mut self,
        team_data: &TeamData,
        selected_niche: &Option<String>,
        customizations: &HashMap<String, String>,
        manifest: &str,
    ) -> bool {
        self.validation_errors.clear();

        match self.state {
            WorkflowState::TeamSelection => {
                if team_data.team_info.name.trim().is_empty() {
                    self.validation_errors
                        .push("Team name is required".to_string());
                }
                if team_data.team_info.required_capabilities.is_empty() {
                    self.validation_errors
                        .push("At least one capability must be selected".to_string());
                }
                if team_data.team_info.focus_area.trim().is_empty() {
                    self.validation_errors
                        .push("Focus area is required".to_string());
                }
            }
            WorkflowState::NicheSelection => {
                if selected_niche.is_none() {
                    self.validation_errors
                        .push("A niche must be selected".to_string());
                }
            }
            WorkflowState::NicheCustomization => {
                // Validate customizations based on template requirements
                if let Some(niche_id) = selected_niche {
                    let loader = super::templates::get_template_loader();
                    if let Some(template) = loader.get_template(niche_id) {
                        for option in &template.customization_options {
                            if option.required {
                                if let Some(value) = customizations.get(&option.id) {
                                    if value.trim().is_empty() {
                                        self.validation_errors.push(format!(
                                            "Required field '{}' cannot be empty",
                                            option.name
                                        ));
                                    }

                                    // Validate against regex if provided
                                    if let Some(regex) = &option.validation_regex {
                                        if let Ok(re) = regex::Regex::new(regex) {
                                            if !re.is_match(value) {
                                                self.validation_errors.push(format!(
                                                    "Field '{}' does not match required format",
                                                    option.name
                                                ));
                                            }
                                        }
                                    }
                                } else {
                                    self.validation_errors.push(format!(
                                        "Required field '{}' is missing",
                                        option.name
                                    ));
                                }
                            }
                        }
                    }
                }
            }
            WorkflowState::ManifestGeneration => {
                if manifest.trim().is_empty() {
                    self.validation_errors
                        .push("Manifest has not been generated".to_string());
                }
            }
            WorkflowState::YamlEditing => {
                if manifest.trim().is_empty() {
                    self.validation_errors
                        .push("Manifest cannot be empty".to_string());
                } else {
                    // Validate YAML syntax
                    if let Err(e) = serde_yaml::from_str::<serde_yaml::Value>(manifest) {
                        self.validation_errors.push(format!("Invalid YAML: {}", e));
                    }
                }
            }
            WorkflowState::Deployment => {
                // Always valid for deployment
            }
            WorkflowState::Completed => {
                // Always valid when completed
            }
        }

        self.validation_errors.is_empty()
    }

    /// Attempt to advance to the next workflow state
    pub fn advance(
        &mut self,
        team_data: &TeamData,
        selected_niche: &Option<String>,
        customizations: &HashMap<String, String>,
        manifest: &str,
    ) -> bool {
        if !self.validate_current_state(team_data, selected_niche, customizations, manifest) {
            return false;
        }

        let next_state = match self.state {
            WorkflowState::TeamSelection => WorkflowState::NicheSelection,
            WorkflowState::NicheSelection => WorkflowState::NicheCustomization,
            WorkflowState::NicheCustomization => WorkflowState::ManifestGeneration,
            WorkflowState::ManifestGeneration => WorkflowState::YamlEditing,
            WorkflowState::YamlEditing => WorkflowState::Deployment,
            WorkflowState::Deployment => WorkflowState::Completed,
            WorkflowState::Completed => return false, // Cannot advance from completed
        };

        self.state = next_state;
        true
    }

    /// Go back to the previous workflow state
    pub fn go_back(&mut self) -> bool {
        let previous_state = match self.state {
            WorkflowState::TeamSelection => return false, // Cannot go back from first state
            WorkflowState::NicheSelection => WorkflowState::TeamSelection,
            WorkflowState::NicheCustomization => WorkflowState::NicheSelection,
            WorkflowState::ManifestGeneration => WorkflowState::NicheCustomization,
            WorkflowState::YamlEditing => WorkflowState::ManifestGeneration,
            WorkflowState::Deployment => WorkflowState::YamlEditing,
            WorkflowState::Completed => WorkflowState::Deployment,
        };

        self.state = previous_state;
        self.validation_errors.clear();
        true
    }

    /// Reset workflow to initial state
    pub fn reset(&mut self) {
        self.state = WorkflowState::TeamSelection;
        self.validation_errors.clear();
    }

    /// Get progress percentage (0.0 to 1.0)
    pub fn get_progress(&self) -> f32 {
        match self.state {
            WorkflowState::TeamSelection => 0.0,
            WorkflowState::NicheSelection => 0.16,
            WorkflowState::NicheCustomization => 0.33,
            WorkflowState::ManifestGeneration => 0.50,
            WorkflowState::YamlEditing => 0.66,
            WorkflowState::Deployment => 0.83,
            WorkflowState::Completed => 1.0,
        }
    }

    /// Get human-readable state title
    pub fn get_state_title(&self) -> &'static str {
        match self.state {
            WorkflowState::TeamSelection => "Team Setup",
            WorkflowState::NicheSelection => "Choose Niche",
            WorkflowState::NicheCustomization => "Customize",
            WorkflowState::ManifestGeneration => "Generate Manifest",
            WorkflowState::YamlEditing => "Edit YAML",
            WorkflowState::Deployment => "Deploy",
            WorkflowState::Completed => "Complete",
        }
    }

    /// Get state icon
    pub fn get_state_icon(&self) -> &'static str {
        match self.state {
            WorkflowState::TeamSelection => "👥",
            WorkflowState::NicheSelection => "🎯",
            WorkflowState::NicheCustomization => "⚙️",
            WorkflowState::ManifestGeneration => "📄",
            WorkflowState::YamlEditing => "📝",
            WorkflowState::Deployment => "🚀",
            WorkflowState::Completed => "🎉",
        }
    }

    /// Check if the current state allows going back
    pub fn can_go_back(&self) -> bool {
        !matches!(self.state, WorkflowState::TeamSelection)
    }

    /// Check if the current state allows advancing
    pub fn can_advance(&self) -> bool {
        !matches!(self.state, WorkflowState::Completed)
    }
}

impl Default for WorkflowManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Workflow step metadata
#[derive(Debug, Clone)]
pub struct WorkflowStep {
    pub state: WorkflowState,
    pub title: String,
    pub description: String,
    pub icon: String,
    pub progress: f32,
}

/// Get all workflow steps for UI display
pub fn get_workflow_steps() -> Vec<WorkflowStep> {
    vec![
        WorkflowStep {
            state: WorkflowState::TeamSelection,
            title: "Team Setup".to_string(),
            description: "Configure your team and select required capabilities".to_string(),
            icon: "👥".to_string(),
            progress: 0.0,
        },
        WorkflowStep {
            state: WorkflowState::NicheSelection,
            title: "Choose Niche".to_string(),
            description: "Select a niche template that matches your needs".to_string(),
            icon: "🎯".to_string(),
            progress: 0.16,
        },
        WorkflowStep {
            state: WorkflowState::NicheCustomization,
            title: "Customize".to_string(),
            description: "Customize the selected niche template".to_string(),
            icon: "⚙️".to_string(),
            progress: 0.33,
        },
        WorkflowStep {
            state: WorkflowState::ManifestGeneration,
            title: "Generate Manifest".to_string(),
            description: "Generate the biome manifest from your configuration".to_string(),
            icon: "📄".to_string(),
            progress: 0.50,
        },
        WorkflowStep {
            state: WorkflowState::YamlEditing,
            title: "Edit YAML".to_string(),
            description: "Fine-tune the generated YAML manifest".to_string(),
            icon: "📝".to_string(),
            progress: 0.66,
        },
        WorkflowStep {
            state: WorkflowState::Deployment,
            title: "Deploy".to_string(),
            description: "Deploy your biome to the selected primals".to_string(),
            icon: "🚀".to_string(),
            progress: 0.83,
        },
        WorkflowStep {
            state: WorkflowState::Completed,
            title: "Complete".to_string(),
            description: "Your biome has been successfully deployed".to_string(),
            icon: "🎉".to_string(),
            progress: 1.0,
        },
    ]
}

/// Workflow transition validation
pub fn validate_workflow_transition(from: &WorkflowState, to: &WorkflowState) -> bool {
    match (from, to) {
        // Forward transitions
        (WorkflowState::TeamSelection, WorkflowState::NicheSelection) => true,
        (WorkflowState::NicheSelection, WorkflowState::NicheCustomization) => true,
        (WorkflowState::NicheCustomization, WorkflowState::ManifestGeneration) => true,
        (WorkflowState::ManifestGeneration, WorkflowState::YamlEditing) => true,
        (WorkflowState::YamlEditing, WorkflowState::Deployment) => true,
        (WorkflowState::Deployment, WorkflowState::Completed) => true,

        // Backward transitions
        (WorkflowState::NicheSelection, WorkflowState::TeamSelection) => true,
        (WorkflowState::NicheCustomization, WorkflowState::NicheSelection) => true,
        (WorkflowState::ManifestGeneration, WorkflowState::NicheCustomization) => true,
        (WorkflowState::YamlEditing, WorkflowState::ManifestGeneration) => true,
        (WorkflowState::Deployment, WorkflowState::YamlEditing) => true,
        (WorkflowState::Completed, WorkflowState::Deployment) => true,

        // Reset transition
        (_, WorkflowState::TeamSelection) => true,

        // Same state (no transition)
        (a, b) if a == b => true,

        // All other transitions are invalid
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[test]
    fn test_workflow_manager_creation() {
        let manager = WorkflowManager::new();
        assert_eq!(manager.get_state(), &WorkflowState::TeamSelection);
        assert!(manager.get_validation_errors().is_empty());
    }

    #[test]
    fn test_workflow_progression() {
        let mut manager = WorkflowManager::new();
        let mut team_data = TeamData::new();
        team_data.team_info.name = "Test Team".to_string();
        team_data.team_info.focus_area = "Testing".to_string();
        team_data
            .team_info
            .required_capabilities
            .insert(PrimalCapability::Compute);

        let selected_niche = Some("web-development".to_string());
        let customizations = HashMap::new();
        let manifest = "test: manifest".to_string();

        // Should be able to advance from team selection
        assert!(manager.advance(&team_data, &None, &customizations, &manifest));
        assert_eq!(manager.get_state(), &WorkflowState::NicheSelection);

        // Should be able to advance from niche selection
        assert!(manager.advance(&team_data, &selected_niche, &customizations, &manifest));
        assert_eq!(manager.get_state(), &WorkflowState::NicheCustomization);
    }

    #[test]
    fn test_workflow_validation() {
        let mut manager = WorkflowManager::new();
        let team_data = TeamData::new(); // Empty team data
        let selected_niche = None;
        let customizations = HashMap::new();
        let manifest = "";

        // Should not be able to advance with empty team data
        assert!(!manager.validate_current_state(
            &team_data,
            &selected_niche,
            &customizations,
            manifest
        ));
        assert!(!manager.get_validation_errors().is_empty());
    }

    #[test]
    fn test_workflow_transitions() {
        assert!(validate_workflow_transition(
            &WorkflowState::TeamSelection,
            &WorkflowState::NicheSelection
        ));
        assert!(validate_workflow_transition(
            &WorkflowState::NicheSelection,
            &WorkflowState::TeamSelection
        ));
        assert!(!validate_workflow_transition(
            &WorkflowState::TeamSelection,
            &WorkflowState::Deployment
        ));
    }

    #[test]
    fn test_workflow_steps() {
        let steps = get_workflow_steps();
        assert_eq!(steps.len(), 7);
        assert_eq!(steps[0].state, WorkflowState::TeamSelection);
        assert_eq!(steps[6].state, WorkflowState::Completed);
    }
}
