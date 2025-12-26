use crate::views::niche_manager::types::*;
use egui;

pub struct NicheEditor;

impl NicheEditor {
    /// Render the visual niche editor
    pub fn render_visual_editor(
        niche_manifest: &mut NicheManifest,
        ui: &mut egui::Ui,
        _render_card: impl Fn(&mut egui::Ui, &str, &mut dyn FnMut(&mut egui::Ui)),
    ) {
        ui.label("Visual Editor - Temporarily Disabled");
        ui.label("This feature is being refactored for compatibility.");

        // Basic metadata editing
        ui.horizontal(|ui| {
            ui.label("Name:");
            ui.text_edit_singleline(&mut niche_manifest.metadata.name);
        });

        ui.horizontal(|ui| {
            ui.label("Version:");
            ui.text_edit_singleline(&mut niche_manifest.metadata.version);
        });

        ui.horizontal(|ui| {
            ui.label("Author:");
            ui.text_edit_singleline(&mut niche_manifest.metadata.author);
        });

        ui.label("Description:");
        ui.text_edit_multiline(&mut niche_manifest.metadata.description);
    }

    /// Render the enhanced visual editor
    pub fn render_enhanced_visual_editor(
        niche_manifest: &mut NicheManifest,
        ui: &mut egui::Ui,
    ) -> Option<NicheEditorMode> {
        ui.heading("🎨 Enhanced Visual Niche Editor");
        ui.separator();

        ui.label("Configure your niche with the unified BiomeOS architecture:");

        Self::render_visual_editor(niche_manifest, ui, |ui, title, _content| {
            ui.collapsing(title, |ui| {
                // Text editing temporarily disabled during refactoring
                ui.label("Text editing is being refactored for the unified architecture");

                // Add helpful guidance for unified architecture
                ui.separator();
                ui.label("💡 Universal Adapter Guidelines:");
                ui.label("• Delegate parsing to Toadstool");
                ui.label("• Use Songbird for service discovery");
                ui.label("• Leverage capability-based routing");
            });
        });

        if ui.button("🔄 Switch to YAML Editor").clicked() {
            return Some(NicheEditorMode::YAML);
        }

        None
    }

    /// Render the YAML editor
    pub fn render_yaml_editor(
        niche_yaml: &mut String,
        validation_errors: &mut Vec<String>,
        ui: &mut egui::Ui,
        _render_card: impl Fn(&mut egui::Ui, &str, &mut dyn FnMut(&mut egui::Ui)),
    ) {
        ui.label("YAML Editor");
        ui.add_space(5.0);

        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                ui.add(
                    egui::TextEdit::multiline(niche_yaml)
                        .code_editor()
                        .desired_rows(20)
                        .desired_width(f32::INFINITY),
                );
            });

        ui.add_space(10.0);
        ui.horizontal(|ui| {
            if ui.button("✅ Validate").clicked() {
                validation_errors.clear();
                Self::validate_yaml(niche_yaml, validation_errors);
            }
            if ui.button("💾 Save").clicked() {
                // Save niche - would integrate with actual save functionality
            }
            if ui.button("🔄 Reset").clicked() {
                *niche_yaml = Self::get_default_niche_yaml();
            }
        });

        // Validation results
        if !validation_errors.is_empty() {
            ui.add_space(10.0);
            ui.separator();
            ui.label("❌ Validation Errors:");
            for error in validation_errors {
                ui.colored_label(egui::Color32::RED, format!("• {}", error));
            }
        }
    }

    /// Render the preview mode
    pub fn render_preview_mode(
        niche_manifest: &NicheManifest,
        ui: &mut egui::Ui,
        _render_card: impl Fn(&mut egui::Ui, &str, &mut dyn FnMut(&mut egui::Ui)),
    ) {
        ui.heading(&niche_manifest.metadata.name);
        ui.label(format!("Version: {}", niche_manifest.metadata.version));
        ui.label(format!("Author: {}", niche_manifest.metadata.author));
        ui.add_space(5.0);
        ui.label(&niche_manifest.metadata.description);

        ui.add_space(10.0);
        ui.separator();
        ui.heading("Services");

        for service in &niche_manifest.services {
            ui.label(format!("• {}", service.name));
        }

        ui.add_space(10.0);
        ui.separator();
        ui.heading("Resources");
        ui.label(format!(
            "CPU: {:.1} cores",
            niche_manifest.resources.total_cpu
        ));
        ui.label(format!(
            "Memory: {:.1} GB",
            niche_manifest.resources.total_memory_gb
        ));
        ui.label(format!(
            "Storage: {:.1} GB",
            niche_manifest.resources.total_storage_gb
        ));
    }

    /// Generate YAML from the visual manifest
    pub fn generate_yaml_from_manifest(niche_manifest: &NicheManifest) -> String {
        format!(
            r#"# Generated Niche Package
name: {}
version: {}
author: {}
description: {}

services:
{}

resources:
  total_cpu: {}
  total_memory_gb: {}
  total_storage_gb: {}
"#,
            niche_manifest.metadata.name,
            niche_manifest.metadata.version,
            niche_manifest.metadata.author,
            niche_manifest.metadata.description,
            niche_manifest
                .services
                .iter()
                .map(|s| format!("  - name: {}", s.name))
                .collect::<Vec<_>>()
                .join("\n"),
            niche_manifest.resources.total_cpu,
            niche_manifest.resources.total_memory_gb,
            niche_manifest.resources.total_storage_gb
        )
    }

    /// Validate YAML content
    pub fn validate_yaml(niche_yaml: &str, validation_errors: &mut Vec<String>) {
        validation_errors.clear();

        if niche_yaml.trim().is_empty() {
            validation_errors.push("YAML content cannot be empty".to_string());
            return;
        }

        // Basic YAML validation
        if !niche_yaml.contains("name:") {
            validation_errors.push("Missing 'name' field".to_string());
        }

        if !niche_yaml.contains("version:") {
            validation_errors.push("Missing 'version' field".to_string());
        }

        if !niche_yaml.contains("author:") {
            validation_errors.push("Missing 'author' field".to_string());
        }

        if !niche_yaml.contains("description:") {
            validation_errors.push("Missing 'description' field".to_string());
        }
    }

    /// Get default niche YAML
    pub fn get_default_niche_yaml() -> String {
        r#"# Default Niche Package
name: my-niche
version: 1.0.0
author: Your Name
description: A description of your niche

services:
  - name: web-server
    image: nginx:latest
    ports:
      - 80:80

resources:
  total_cpu: 2.0
  total_memory_gb: 4.0
  total_storage_gb: 20.0
"#
        .to_string()
    }
}
