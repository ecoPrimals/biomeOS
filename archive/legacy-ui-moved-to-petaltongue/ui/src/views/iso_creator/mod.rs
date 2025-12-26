//! ISO Creator Module
//!
//! This module provides a modular ISO Creator implementation, replacing the
//! original monolithic iso_creator.rs file with a well-organized structure.
//!
//! # Architecture
//!
//! The ISO Creator is organized into several modules:
//! - `types`: Data structures and type definitions
//! - `config`: Configuration management
//! - `build`: Build process management
//! - `queue`: Build queue management
//! - `ui`: UI rendering components
//! - `mock_data`: Mock data for development/testing
//!
//! # Usage
//!
//! ```rust
//! use crate::views::iso_creator::IsoCreatorView;
//! use crate::api::BiomeOSApi;
//! use crate::state::AppState;
//!
//! let view = IsoCreatorView::new(state, api);
//! ```

use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::api::BiomeOSApi;
use crate::state::AppState;
use crate::views::{BaseView, View};

// Module declarations
pub mod build;
pub mod config;
pub mod mock_data;
pub mod queue;
pub mod types;
pub mod ui;

// Re-export commonly used types
pub use build::BuildManager;
pub use config::ConfigManager;
pub use mock_data::MockDataProvider;
pub use queue::BuildQueue;
pub use types::*;
pub use ui::IsoCreatorUI;

/// Main ISO Creator view implementation
pub struct IsoCreatorView {
    pub base: BaseView,
    pub ui: IsoCreatorUI,
    pub config_manager: ConfigManager,
    pub build_manager: BuildManager,
    pub build_queue: BuildQueue,
    pub current_config: IsoConfig,
    pub creator_config: IsoCreatorConfig,
    pub available_niches: Vec<NichePackage>,
    pub selected_niches: Vec<String>,
    pub custom_components: Vec<CustomComponent>,
    pub iso_templates: Vec<IsoTemplate>,
    pub build_log: Vec<String>,
    pub build_progress: f32,
    pub build_status: BuildStatus,
}

impl IsoCreatorView {
    /// Create a new ISO Creator view
    pub fn new(state: Arc<Mutex<AppState>>, api: Arc<BiomeOSApi>) -> Self {
        let mut config_manager = ConfigManager::new();
        let creator_config = MockDataProvider::get_default_creator_config();
        let build_manager = BuildManager::new(creator_config.clone());
        let build_queue = BuildQueue::new(creator_config.max_concurrent_builds);

        // Load initial data
        config_manager.load_configurations().ok();

        Self {
            base: BaseView::new(state, api),
            ui: IsoCreatorUI::new(),
            config_manager,
            build_manager,
            build_queue,
            current_config: IsoConfig::new(
                "biomeOS-custom".to_string(),
                "Custom biomeOS distribution".to_string(),
            ),
            creator_config,
            available_niches: MockDataProvider::get_mock_niches(),
            selected_niches: Vec::new(),
            custom_components: MockDataProvider::get_mock_components(),
            iso_templates: MockDataProvider::get_mock_templates(),
            build_log: Vec::new(),
            build_progress: 0.0,
            build_status: BuildStatus::Idle,
        }
    }

    /// Get available configurations
    pub fn get_configurations(&self) -> &[IsoConfig] {
        self.config_manager.get_configurations()
    }

    /// Save current configuration
    pub fn save_current_configuration(&mut self) -> Result<(), String> {
        self.config_manager
            .save_configuration(self.current_config.clone())
    }

    /// Load configuration by name
    pub fn load_configuration(&mut self, name: &str) -> Result<(), String> {
        if let Some(config) = self.config_manager.get_configuration_by_name(name) {
            self.current_config = config.clone();
            Ok(())
        } else {
            Err(format!("Configuration '{}' not found", name))
        }
    }

    /// Create configuration from template
    pub fn create_from_template(&mut self, template_name: &str) -> Result<(), String> {
        let config = self.config_manager.create_from_template(
            template_name,
            format!("biomeOS-{}", template_name.to_lowercase().replace(" ", "-")),
        )?;
        self.current_config = config;
        Ok(())
    }

    /// Start build process
    pub fn start_build(&mut self) -> Result<(), String> {
        // Validate current configuration
        self.current_config.validate()?;

        // Update configuration with selected niches
        self.current_config.included_niches = self.selected_niches.clone();

        // Start the build
        let build_id = self
            .build_manager
            .start_build(self.current_config.clone())?;
        self.build_status = BuildStatus::Building;
        self.build_progress = 0.0;
        self.build_log.clear();
        self.build_log
            .push(format!("🚀 Started build: {}", build_id));

        // Simulate build process (in a real implementation, this would be async)
        self.simulate_build_progress();

        Ok(())
    }

    /// Cancel current build
    pub fn cancel_build(&mut self) -> Result<(), String> {
        self.build_manager.cancel_build()?;
        self.build_status = BuildStatus::Failed;
        self.build_log
            .push("⏹️ Build cancelled by user".to_string());
        Ok(())
    }

    /// Add current configuration to build queue
    pub fn add_to_queue(&mut self, priority: queue::JobPriority) -> Result<String, String> {
        self.current_config.included_niches = self.selected_niches.clone();
        let job_id = self
            .build_queue
            .add_job(self.current_config.clone(), priority);
        self.build_log
            .push(format!("📋 Added to queue: {}", job_id));
        Ok(job_id)
    }

    /// Get build queue statistics
    pub fn get_queue_statistics(&self) -> queue::QueueStatistics {
        self.build_queue.get_statistics()
    }

    /// Update build progress (for simulation)
    fn simulate_build_progress(&mut self) {
        // In a real implementation, this would be handled by a background thread
        // For now, we'll just update the progress based on the current state

        if self.build_status == BuildStatus::Building {
            self.build_progress += 0.1;

            if self.build_progress >= 1.0 {
                self.build_progress = 1.0;
                self.build_status = BuildStatus::Success;
                self.build_log
                    .push("✅ Build completed successfully".to_string());

                // Complete the build
                let output_path = format!(
                    "{}/{}.iso",
                    self.creator_config.output_directory, self.current_config.name
                );
                self.build_manager.complete_build(output_path).ok();
            } else {
                // Add progress log entries
                let entries = MockDataProvider::generate_build_log_entries(self.build_progress);
                if let Some(last_entry) = entries.last() {
                    if !self.build_log.contains(last_entry) {
                        self.build_log.push(last_entry.clone());
                    }
                }
            }
        }
    }

    /// Get estimated build time
    pub fn get_estimated_build_time(&self) -> std::time::Duration {
        let mut duration = std::time::Duration::from_secs(300); // 5 minutes base

        // Add time for each primal
        duration +=
            std::time::Duration::from_secs(60 * self.current_config.included_primals.len() as u64);

        // Add time for each niche
        duration +=
            std::time::Duration::from_secs(120 * self.current_config.included_niches.len() as u64);

        // Add time for each custom component
        duration +=
            std::time::Duration::from_secs(30 * self.current_config.custom_components.len() as u64);

        // Adjust for compression level
        match self.current_config.compression_level {
            0..=3 => duration += std::time::Duration::from_secs(60),
            4..=6 => duration += std::time::Duration::from_secs(120),
            7..=9 => duration += std::time::Duration::from_secs(180),
            _ => duration += std::time::Duration::from_secs(240),
        }

        duration
    }

    /// Get size breakdown for current configuration
    pub fn get_size_breakdown(&self) -> std::collections::HashMap<String, u64> {
        self.config_manager.get_size_breakdown(&self.current_config)
    }

    /// Validate current configuration
    pub fn validate_current_config(&self) -> Result<(), String> {
        self.current_config.validate()?;
        self.config_manager
            .validate_compatibility(&self.current_config)?;
        Ok(())
    }

    /// Get configuration suggestions
    pub fn get_suggestions(&self, use_case: &str) -> Vec<String> {
        self.config_manager.get_suggestions(use_case)
    }

    /// Export current configuration
    pub fn export_configuration(&self, path: &std::path::Path) -> Result<(), String> {
        self.config_manager
            .export_configuration(&self.current_config, path)
    }

    /// Import configuration from file
    pub fn import_configuration(&mut self, path: &std::path::Path) -> Result<(), String> {
        let config = self.config_manager.import_configuration(path)?;
        self.current_config = config;
        Ok(())
    }

    /// Reset to default configuration
    pub fn reset_to_default(&mut self) {
        self.current_config = IsoConfig::new(
            "biomeOS-custom".to_string(),
            "Custom biomeOS distribution".to_string(),
        );
        self.selected_niches.clear();
        self.build_log.clear();
        self.build_progress = 0.0;
        self.build_status = BuildStatus::Idle;
    }

    /// Get build history
    pub fn get_build_history(&self) -> &[BuildJob] {
        self.build_manager.get_build_history()
    }

    /// Clear build history
    pub fn clear_build_history(&mut self) {
        self.build_manager.clear_history();
    }

    /// Get current build status
    pub fn get_build_status(&self) -> &BuildStatus {
        &self.build_status
    }

    /// Get current build progress
    pub fn get_build_progress(&self) -> f32 {
        self.build_progress
    }

    /// Get build log
    pub fn get_build_log(&self) -> &[String] {
        &self.build_log
    }

    /// Update selected niches
    pub fn update_selected_niches(&mut self, niches: Vec<String>) {
        self.selected_niches = niches;
        // Update current config size estimate
        self.current_config.included_niches = self.selected_niches.clone();
        self.current_config.size_estimate = self.current_config.calculate_size_estimate();
    }

    /// Add custom component
    pub fn add_custom_component(&mut self, component: CustomComponent) {
        self.custom_components.push(component);
    }

    /// Remove custom component
    pub fn remove_custom_component(&mut self, index: usize) -> Result<(), String> {
        if index < self.custom_components.len() {
            self.custom_components.remove(index);
            Ok(())
        } else {
            Err("Component index out of bounds".to_string())
        }
    }

    /// Get available templates
    pub fn get_templates(&self) -> &[IsoTemplate] {
        &self.iso_templates
    }

    /// Get available niches
    pub fn get_niches(&self) -> &[NichePackage] {
        &self.available_niches
    }

    /// Get custom components
    pub fn get_custom_components(&self) -> &[CustomComponent] {
        &self.custom_components
    }

    /// Get selected niches
    pub fn get_selected_niches(&self) -> &[String] {
        &self.selected_niches
    }

    /// Get current configuration
    pub fn get_current_config(&self) -> &IsoConfig {
        &self.current_config
    }

    /// Get mutable current configuration
    pub fn get_current_config_mut(&mut self) -> &mut IsoConfig {
        &mut self.current_config
    }

    /// Get creator configuration
    pub fn get_creator_config(&self) -> &IsoCreatorConfig {
        &self.creator_config
    }

    /// Update creator configuration
    pub fn update_creator_config(&mut self, config: IsoCreatorConfig) {
        self.creator_config = config;
    }
}

impl View for IsoCreatorView {
    fn render(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.heading("💿 ISO Creator");
        ui.label(
            "Create bootable biomeOS ISO images with custom configurations and niche packages",
        );
        ui.separator();

        // Render tab bar
        self.ui.render_tab_bar(ui);
        ui.add_space(10.0);

        // Render current tab
        match self.ui.get_selected_tab() {
            IsoCreatorTab::Configuration => {
                self.ui
                    .render_configuration_tab(ui, &mut self.current_config, &self.iso_templates);
            }
            IsoCreatorTab::Niches => {
                self.ui
                    .render_niches_tab(ui, &self.available_niches, &mut self.selected_niches);
            }
            IsoCreatorTab::Components => {
                self.ui
                    .render_components_tab(ui, &mut self.custom_components);
            }
            IsoCreatorTab::Build => {
                self.ui.render_build_tab(
                    ui,
                    &self.build_status,
                    self.build_progress,
                    &self.build_log,
                );

                // Handle build actions
                if self.build_status == BuildStatus::Idle {
                    ui.horizontal(|ui| {
                        if ui.button("🚀 Start Build").clicked() {
                            if let Err(e) = self.start_build() {
                                self.build_log
                                    .push(format!("❌ Build failed to start: {}", e));
                            }
                        }

                        if ui.button("📋 Add to Queue").clicked() {
                            if let Err(e) = self.add_to_queue(queue::JobPriority::Normal) {
                                self.build_log
                                    .push(format!("❌ Failed to add to queue: {}", e));
                            }
                        }

                        if ui.button("💾 Save Config").clicked() {
                            if let Err(e) = self.save_current_configuration() {
                                self.build_log
                                    .push(format!("❌ Failed to save config: {}", e));
                            } else {
                                self.build_log.push("✅ Configuration saved".to_string());
                            }
                        }
                    });
                } else if self.build_status == BuildStatus::Building {
                    if ui.button("⏹️ Cancel Build").clicked() {
                        if let Err(e) = self.cancel_build() {
                            self.build_log
                                .push(format!("❌ Failed to cancel build: {}", e));
                        }
                    }
                }
            }
            IsoCreatorTab::Queue => {
                // Convert build manager history to build jobs for display
                let build_jobs: Vec<BuildJob> = self.build_manager.get_build_history().to_vec();
                self.ui.render_queue_tab(ui, &build_jobs);
            }
        }

        // Update build progress if building
        if self.build_status == BuildStatus::Building {
            self.simulate_build_progress();
        }
    }
}

/// Helper functions for ISO Creator
impl IsoCreatorView {
    /// Check if build can be started
    pub fn can_start_build(&self) -> bool {
        self.build_status == BuildStatus::Idle && self.current_config.validate().is_ok()
    }

    /// Get estimated completion time
    pub fn get_estimated_completion(&self) -> Option<std::time::SystemTime> {
        if self.build_status == BuildStatus::Building {
            let remaining_time = self.get_estimated_build_time();
            let remaining_factor = 1.0 - self.build_progress;
            let adjusted_time = std::time::Duration::from_secs(
                (remaining_time.as_secs() as f32 * remaining_factor) as u64,
            );
            Some(std::time::SystemTime::now() + adjusted_time)
        } else {
            None
        }
    }

    /// Get total size of selected components
    pub fn get_total_selected_size(&self) -> u64 {
        self.current_config.calculate_size_estimate()
    }

    /// Check if configuration is valid
    pub fn is_configuration_valid(&self) -> bool {
        self.validate_current_config().is_ok()
    }

    /// Get validation errors
    pub fn get_validation_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();

        if let Err(e) = self.current_config.validate() {
            errors.push(e);
        }

        if let Err(e) = self
            .config_manager
            .validate_compatibility(&self.current_config)
        {
            errors.push(e);
        }

        errors
    }
}
