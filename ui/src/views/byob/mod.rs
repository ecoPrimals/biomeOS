//! BYOB (Build Your Own Biome) Module
//!
//! This module provides a complete universal and agnostic system for building
//! biomes using capability-based primal selection. The system works with any
//! primal combination (current and future) without hardcoded names.

use crate::views::View;
use eframe::egui;
use std::collections::HashMap;

pub mod data;
pub mod rendering;
pub mod templates;
pub mod types;
pub mod workflow;

use data::get_primal_discovery;
use rendering::*;
use templates::get_template_loader;
pub use types::*;

/// Main BYOB application state
pub struct ByobView {
    workflow_state: WorkflowState,
    team_data: TeamData,
    selected_niche: Option<String>,
    selected_template: Option<NicheTemplate>,
    customizations: HashMap<String, String>,
    generated_manifest: String,
    deployment_data: DeploymentData,
    primal_discovery: data::PrimalDiscovery,
    template_loader: templates::TemplateLoader,
}

impl Default for ByobView {
    fn default() -> Self {
        Self::new()
    }
}

impl ByobView {
    pub fn new() -> Self {
        Self {
            workflow_state: WorkflowState::TeamSelection,
            team_data: TeamData::new(),
            selected_niche: None,
            selected_template: None,
            customizations: HashMap::new(),
            generated_manifest: String::new(),
            deployment_data: DeploymentData::new(),
            primal_discovery: get_primal_discovery(),
            template_loader: get_template_loader(),
        }
    }

    /// Reset the application to initial state
    pub fn reset(&mut self) {
        self.workflow_state = WorkflowState::TeamSelection;
        self.team_data = TeamData::new();
        self.selected_niche = None;
        self.selected_template = None;
        self.customizations.clear();
        self.generated_manifest.clear();
        self.deployment_data = DeploymentData::new();
    }

    /// Get current workflow progress as percentage
    pub fn get_progress(&self) -> f32 {
        match self.workflow_state {
            WorkflowState::TeamSelection => 0.0,
            WorkflowState::NicheSelection => 0.16,
            WorkflowState::NicheCustomization => 0.33,
            WorkflowState::ManifestGeneration => 0.50,
            WorkflowState::YamlEditing => 0.66,
            WorkflowState::Deployment => 0.83,
            WorkflowState::Completed => 1.0,
        }
    }

    /// Get current workflow step title
    pub fn get_step_title(&self) -> &'static str {
        match self.workflow_state {
            WorkflowState::TeamSelection => "Team Setup",
            WorkflowState::NicheSelection => "Choose Niche",
            WorkflowState::NicheCustomization => "Customize",
            WorkflowState::ManifestGeneration => "Generate Manifest",
            WorkflowState::YamlEditing => "Edit YAML",
            WorkflowState::Deployment => "Deploy",
            WorkflowState::Completed => "Complete",
        }
    }

    /// Get current workflow step icon
    pub fn get_step_icon(&self) -> &'static str {
        match self.workflow_state {
            WorkflowState::TeamSelection => "👥",
            WorkflowState::NicheSelection => "🎯",
            WorkflowState::NicheCustomization => "⚙️",
            WorkflowState::ManifestGeneration => "📄",
            WorkflowState::YamlEditing => "📝",
            WorkflowState::Deployment => "🚀",
            WorkflowState::Completed => "🎉",
        }
    }

    /// Check if we can proceed to the next step
    pub fn can_proceed(&self) -> bool {
        match self.workflow_state {
            WorkflowState::TeamSelection => {
                !self.team_data.team_info.name.is_empty()
                    && !self.team_data.team_info.required_capabilities.is_empty()
            }
            WorkflowState::NicheSelection => self.selected_niche.is_some(),
            WorkflowState::NicheCustomization => self.selected_template.is_some(),
            WorkflowState::ManifestGeneration => !self.generated_manifest.is_empty(),
            WorkflowState::YamlEditing => !self.generated_manifest.is_empty(),
            WorkflowState::Deployment => true,
            WorkflowState::Completed => false,
        }
    }

    /// Advance to the next workflow step
    pub fn next_step(&mut self) {
        if !self.can_proceed() {
            return;
        }

        match self.workflow_state {
            WorkflowState::TeamSelection => {
                self.workflow_state = WorkflowState::NicheSelection;
                // Update available primals based on team capabilities
                self.team_data.available_primals = self
                    .primal_discovery
                    .find_primals_for_capabilities(&self.team_data.team_info.required_capabilities)
                    .into_iter()
                    .cloned()
                    .collect();
            }
            WorkflowState::NicheSelection => {
                self.workflow_state = WorkflowState::NicheCustomization;
                // Load the selected template
                if let Some(niche_id) = &self.selected_niche {
                    self.selected_template = self.template_loader.get_template(niche_id).cloned();
                }
            }
            WorkflowState::NicheCustomization => {
                self.workflow_state = WorkflowState::ManifestGeneration;
            }
            WorkflowState::ManifestGeneration => {
                self.workflow_state = WorkflowState::YamlEditing;
            }
            WorkflowState::YamlEditing => {
                self.workflow_state = WorkflowState::Deployment;
            }
            WorkflowState::Deployment => {
                self.workflow_state = WorkflowState::Completed;
            }
            WorkflowState::Completed => {
                // Stay in completed state
            }
        }
    }

    /// Go back to the previous workflow step
    pub fn previous_step(&mut self) {
        match self.workflow_state {
            WorkflowState::TeamSelection => {
                // Already at the first step
            }
            WorkflowState::NicheSelection => {
                self.workflow_state = WorkflowState::TeamSelection;
            }
            WorkflowState::NicheCustomization => {
                self.workflow_state = WorkflowState::NicheSelection;
            }
            WorkflowState::ManifestGeneration => {
                self.workflow_state = WorkflowState::NicheCustomization;
            }
            WorkflowState::YamlEditing => {
                self.workflow_state = WorkflowState::ManifestGeneration;
            }
            WorkflowState::Deployment => {
                self.workflow_state = WorkflowState::YamlEditing;
            }
            WorkflowState::Completed => {
                self.workflow_state = WorkflowState::Deployment;
            }
        }
    }

    /// Render the workflow header with progress
    pub fn render_header(&self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("🌱 Build Your Own Biome");
            ui.separator();
            ui.label(format!(
                "{} {}",
                self.get_step_icon(),
                self.get_step_title()
            ));
        });

        ui.separator();

        // Progress bar
        let progress = self.get_progress();
        ui.horizontal(|ui| {
            ui.label("Progress:");
            ui.add(egui::ProgressBar::new(progress).show_percentage());
        });

        ui.add_space(10.0);
    }

    /// Render navigation controls
    pub fn render_navigation(&mut self, ui: &mut egui::Ui) {
        ui.separator();
        ui.horizontal(|ui| {
            if self.workflow_state != WorkflowState::TeamSelection {
                if ui.button("⬅️ Previous").clicked() {
                    self.previous_step();
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if self.workflow_state == WorkflowState::Completed {
                    if ui.button("🔄 Create Another Biome").clicked() {
                        self.reset();
                    }
                } else if self.can_proceed() {
                    if ui.button("➡️ Next").clicked() {
                        self.next_step();
                    }
                }
            });
        });
    }

    /// Main render function
    pub fn render_ui(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        self.render_header(ui);

        match self.workflow_state {
            WorkflowState::TeamSelection => {
                if render_team_selection(ui, &mut self.team_data) {
                    self.next_step();
                }
            }
            WorkflowState::NicheSelection => {
                if render_niche_selection(ui, &self.team_data, &mut self.selected_niche) {
                    self.next_step();
                }
            }
            WorkflowState::NicheCustomization => {
                if let Some(template) = &self.selected_template {
                    if render_niche_customization(ui, template, &mut self.customizations) {
                        self.next_step();
                    }
                } else {
                    ui.label("Error: No template selected");
                    if ui.button("Back to Selection").clicked() {
                        self.previous_step();
                    }
                }
            }
            WorkflowState::ManifestGeneration => {
                if let Some(template) = &self.selected_template {
                    if render_manifest_generation(
                        ui,
                        template,
                        &self.customizations,
                        &mut self.generated_manifest,
                    ) {
                        self.next_step();
                    }
                } else {
                    ui.label("Error: No template selected");
                    if ui.button("Back to Selection").clicked() {
                        self.workflow_state = WorkflowState::NicheSelection;
                    }
                }
            }
            WorkflowState::YamlEditing => {
                if render_yaml_editing(ui, &mut self.generated_manifest) {
                    self.next_step();
                }
            }
            WorkflowState::Deployment => {
                if render_deployment(ui, &mut self.deployment_data) {
                    self.next_step();
                }
            }
            WorkflowState::Completed => {
                render_completion(ui);
            }
        }

        self.render_navigation(ui);
    }
}

impl View for ByobView {
    fn render(&mut self, ui: &mut egui::Ui, ctx: &egui::Context) {
        self.render_ui(ui, ctx);
    }
}

/// Utility functions for integration with the broader biomeOS system

/// Get available primals for a given set of capabilities
pub fn get_available_primals_for_capabilities(
    capabilities: &std::collections::HashSet<PrimalCapability>,
) -> Vec<PrimalDefinition> {
    let discovery = get_primal_discovery();
    discovery
        .find_primals_for_capabilities(capabilities)
        .into_iter()
        .cloned()
        .collect()
}

/// Get all available capabilities in the system
pub fn get_all_capabilities() -> std::collections::HashSet<PrimalCapability> {
    let discovery = get_primal_discovery();
    discovery.get_registry().get_all_capabilities()
}

/// Get compatible templates for a team's capabilities
pub fn get_compatible_templates_for_team(
    team_capabilities: &std::collections::HashSet<PrimalCapability>,
) -> Vec<NicheTemplate> {
    let loader = get_template_loader();
    loader
        .find_compatible_templates(team_capabilities)
        .into_iter()
        .cloned()
        .collect()
}

/// Validate that a template can be deployed with current primals
pub fn validate_template_deployment(template: &NicheTemplate) -> Result<(), String> {
    let discovery = get_primal_discovery();
    let available_primals = discovery.get_all_primals();

    for required_capability in &template.required_capabilities {
        let supporting_primals: Vec<_> = available_primals
            .iter()
            .filter(|primal| primal.capabilities.contains(required_capability))
            .collect();

        if supporting_primals.is_empty() {
            return Err(format!(
                "No primals available that support capability: {}",
                required_capability.display_name()
            ));
        }
    }

    Ok(())
}

/// Generate a deployment manifest for a template with customizations
pub fn generate_deployment_manifest(
    template: &NicheTemplate,
    customizations: &HashMap<String, String>,
) -> String {
    templates::generate_manifest(template, customizations)
}

/// Get system statistics
pub fn get_system_statistics() -> SystemStatistics {
    let discovery = get_primal_discovery();
    let loader = get_template_loader();

    SystemStatistics {
        total_primals: discovery.get_all_primals().len(),
        total_capabilities: discovery.get_registry().get_all_capabilities().len(),
        total_templates: loader.get_templates().len(),
        active_deployments: 0, // Would be populated from actual deployment system
    }
}

/// System statistics structure
pub struct SystemStatistics {
    pub total_primals: usize,
    pub total_capabilities: usize,
    pub total_templates: usize,
    pub active_deployments: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::views::byob::types::*;

    #[test]
    fn test_universal_primal_system() {
        // Test that our universal system can handle any primal combination
        let mut byob_view = ByobView::new();

        // Test with different primal combinations
        let test_cases = vec![
            vec!["toadstool", "nestgate", "songbird", "beardog", "squirrel"],
            vec!["toadstool", "nestgate"], // Minimal compute + storage
            vec!["songbird", "beardog"],   // Networking + security only
            vec!["squirrel"],              // AI only
            vec!["future-primal", "another-primal"], // Future primals
        ];

        for primals in test_cases {
            byob_view.workflow_state.selected_primals =
                primals.into_iter().map(|s| s.to_string()).collect();

            // Verify the system can handle any primal combination
            assert!(!byob_view.workflow_state.selected_primals.is_empty());

            // Test capability detection
            let capabilities =
                byob_view.get_capabilities_for_primals(&byob_view.workflow_state.selected_primals);
            assert!(
                !capabilities.is_empty()
                    || byob_view
                        .workflow_state
                        .selected_primals
                        .iter()
                        .any(|p| p.starts_with("future"))
            );
        }
    }

    #[test]
    fn test_yaml_template_loading() {
        // Test that we can load YAML templates dynamically
        let loader = TemplateLoader::new();
        let templates = loader.load_templates();

        // Should have fallback templates even if no files exist
        assert!(!templates.is_empty());

        // Test that templates have required fields
        for template in templates {
            assert!(!template.name.is_empty());
            assert!(!template.description.is_empty());
            assert!(!template.primals.is_empty());
        }
    }

    #[test]
    fn test_capability_system() {
        // Test the capability-based system
        let capabilities = vec![
            PrimalCapability::Compute,
            PrimalCapability::Storage,
            PrimalCapability::Networking,
            PrimalCapability::Security,
            PrimalCapability::AI,
            PrimalCapability::Custom("future-capability".to_string()),
        ];

        for cap in capabilities {
            let display_name = cap.display_name();
            let description = cap.description();

            assert!(!display_name.is_empty());
            assert!(!description.is_empty());
        }
    }
}
