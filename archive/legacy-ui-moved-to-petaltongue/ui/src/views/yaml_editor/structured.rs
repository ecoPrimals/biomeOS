//! Structured YAML editor

use super::types::*;
use eframe::egui;

impl YamlEditorView {
    /// Render structured editor
    pub fn render_structured_editor(&mut self, ui: &mut egui::Ui) {
        ui.heading("🏗️ Structured Editor");
        ui.separator();

        egui::ScrollArea::vertical().show(ui, |ui| {
            // Metadata section
            ui.collapsing("📋 Metadata", |ui| {
                self.render_metadata_section(ui);
            });

            ui.add_space(10.0);

            // Primals section
            ui.collapsing("🧬 Primals", |ui| {
                self.render_primals_section(ui);
            });

            ui.add_space(10.0);

            // Services section
            ui.collapsing("🔧 Services", |ui| {
                self.render_services_section(ui);
            });

            ui.add_space(10.0);

            // Networking section
            ui.collapsing("🌐 Networking", |ui| {
                self.render_networking_section(ui);
            });

            ui.add_space(10.0);

            // Security section
            ui.collapsing("🔒 Security", |ui| {
                self.render_security_section(ui);
            });

            ui.add_space(10.0);

            // Resources section
            ui.collapsing("📊 Resources", |ui| {
                self.render_resources_section(ui);
            });
        });

        ui.add_space(10.0);

        // Update button
        if ui.button("🔄 Update YAML").clicked() {
            self.update_yaml_from_sections();
        }
    }

    /// Render metadata section
    pub fn render_metadata_section(&mut self, ui: &mut egui::Ui) {
        ui.label("Basic biome information:");
        ui.text_edit_singleline(&mut "my-biome".to_string());
    }

    /// Render primals section
    pub fn render_primals_section(&mut self, ui: &mut egui::Ui) {
        ui.label("Primal configurations:");
        ui.label("• toadstool (compute)");
        ui.label("• songbird (orchestration)");
        ui.label("• nestgate (storage)");
        ui.label("• beardog (security)");
        ui.label("• squirrel (AI/MCP)");
    }

    /// Render services section
    pub fn render_services_section(&mut self, ui: &mut egui::Ui) {
        ui.label("Service definitions:");
        ui.label("Configure your services here");
    }

    /// Render networking section
    pub fn render_networking_section(&mut self, ui: &mut egui::Ui) {
        ui.label("Networking configuration:");
        ui.label("Configure networking here");
    }

    /// Render security section
    pub fn render_security_section(&mut self, ui: &mut egui::Ui) {
        ui.label("Security configuration:");
        ui.label("Configure security here");
    }

    /// Render resources section
    pub fn render_resources_section(&mut self, ui: &mut egui::Ui) {
        ui.label("Resource limits:");
        ui.label("Configure resources here");
    }

    /// Update YAML from sections
    pub fn update_yaml_from_sections(&mut self) {
        // Update the YAML from the structured sections
        self.current_yaml = "# Updated from structured editor\n".to_string();
        self.is_modified = true;
    }

    /// Parse YAML into sections
    pub fn parse_yaml_into_sections(&mut self) {
        self.yaml_sections.clear();

        if let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(&self.current_yaml) {
            if let Some(map) = value.as_mapping() {
                for (key, value) in map {
                    if let Some(key_str) = key.as_str() {
                        let section_type = match key_str {
                            "metadata" => YamlSectionType::Metadata,
                            "primals" => YamlSectionType::Primals,
                            "services" => YamlSectionType::Services,
                            "resources" => YamlSectionType::Resources,
                            "security" => YamlSectionType::Security,
                            "networking" => YamlSectionType::Networking,
                            "agents" => YamlSectionType::Agents,
                            "extensions" => YamlSectionType::Extensions,
                            _ => YamlSectionType::Extensions,
                        };

                        let section = YamlSection {
                            name: key_str.to_string(),
                            start_line: 0, // Would need proper line tracking
                            end_line: 0,
                            content: serde_yaml::to_string(value).unwrap_or_default(),
                            section_type,
                        };

                        self.yaml_sections.insert(key_str.to_string(), section);
                    }
                }
            }
        }
    }

    /// Get section by name
    pub fn get_section(&self, name: &str) -> Option<&YamlSection> {
        self.yaml_sections.get(name)
    }

    /// Set section content
    pub fn set_section_content(&mut self, name: &str, content: String) {
        if let Some(section) = self.yaml_sections.get_mut(name) {
            section.content = content;
            self.is_modified = true;
        }
    }

    /// Toggle section collapse state
    pub fn toggle_section_collapse(&mut self, name: &str) {
        let current_state = self.collapsed_sections.get(name).unwrap_or(&false);
        self.collapsed_sections
            .insert(name.to_string(), !current_state);
    }

    /// Check if section is collapsed
    pub fn is_section_collapsed(&self, name: &str) -> bool {
        self.collapsed_sections.get(name).unwrap_or(&false).clone()
    }
}
