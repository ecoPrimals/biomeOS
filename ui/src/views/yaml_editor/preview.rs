//! YAML preview functionality

use super::types::*;
use eframe::egui;

impl YamlEditorView {
    /// Render preview editor
    pub fn render_preview_editor(&mut self, ui: &mut egui::Ui) {
        ui.heading("👁️ YAML Preview");
        ui.separator();

        // Preview controls
        ui.horizontal(|ui| {
            ui.checkbox(&mut self.show_preview, "📊 Show Structure");
            ui.checkbox(&mut self.show_validation_panel, "✅ Show Validation");

            ui.separator();

            if ui.button("🔄 Refresh").clicked() {
                self.refresh_preview();
            }
        });

        ui.add_space(10.0);

        egui::ScrollArea::vertical().show(ui, |ui| {
            if self.show_preview {
                self.render_yaml_preview(ui);
            } else {
                // Raw YAML display
                ui.add(egui::Label::new(
                    egui::RichText::new(&self.current_yaml)
                        .monospace()
                        .color(egui::Color32::LIGHT_GRAY),
                ));
            }
        });

        ui.add_space(10.0);

        // Validation results
        if self.show_validation_panel {
            self.render_validation_results(ui);
        }
    }

    /// Render YAML preview
    pub fn render_yaml_preview(&mut self, ui: &mut egui::Ui) {
        ui.heading("YAML Structure Preview");
        ui.add_space(10.0);

        // Parse and display structure
        if let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(&self.current_yaml) {
            self.render_yaml_value(ui, &value, 0);
        } else {
            ui.colored_label(
                egui::Color32::RED,
                "Invalid YAML - cannot preview structure",
            );
        }
    }

    /// Render YAML value recursively
    fn render_yaml_value(&self, ui: &mut egui::Ui, value: &serde_yaml::Value, indent: usize) {
        let indent_str = "  ".repeat(indent);

        match value {
            serde_yaml::Value::Null => {
                ui.label(format!("{}null", indent_str));
            }
            serde_yaml::Value::Bool(b) => {
                ui.label(format!("{}{}", indent_str, b));
            }
            serde_yaml::Value::Number(n) => {
                ui.label(format!("{}{}", indent_str, n));
            }
            serde_yaml::Value::String(s) => {
                ui.label(format!("{}\"{}\"", indent_str, s));
            }
            serde_yaml::Value::Sequence(seq) => {
                ui.label(format!("{}[]", indent_str));
                for (i, item) in seq.iter().enumerate() {
                    ui.label(format!("{}[{}]:", indent_str, i));
                    self.render_yaml_value(ui, item, indent + 1);
                }
            }
            serde_yaml::Value::Mapping(map) => {
                ui.label(format!("{}{{...}}", indent_str));
                for (key, val) in map {
                    if let Some(key_str) = key.as_str() {
                        ui.label(format!("{}{}:", indent_str, key_str));
                        self.render_yaml_value(ui, val, indent + 1);
                    }
                }
            }
            serde_yaml::Value::Tagged(tagged) => {
                ui.label(format!("{}!{}", indent_str, tagged.tag));
                self.render_yaml_value(ui, &tagged.value, indent + 1);
            }
        }
    }

    /// Refresh preview
    pub fn refresh_preview(&mut self) {
        self.validate_yaml();
        self.parse_yaml_into_sections();
    }

    /// Get preview statistics
    pub fn get_preview_stats(&self) -> PreviewStats {
        let line_count = self.current_yaml.lines().count();
        let char_count = self.current_yaml.chars().count();
        let section_count = self.yaml_sections.len();

        PreviewStats {
            line_count,
            char_count,
            section_count,
            has_errors: !self.validation_errors.is_empty(),
            has_warnings: !self.validation_warnings.is_empty(),
        }
    }

    /// Render preview statistics
    pub fn render_preview_stats(&self, ui: &mut egui::Ui) {
        let stats = self.get_preview_stats();

        ui.horizontal(|ui| {
            ui.label(format!("📊 Lines: {}", stats.line_count));
            ui.separator();
            ui.label(format!("🔤 Characters: {}", stats.char_count));
            ui.separator();
            ui.label(format!("📋 Sections: {}", stats.section_count));

            if stats.has_errors {
                ui.separator();
                ui.colored_label(egui::Color32::RED, "❌ Errors");
            }

            if stats.has_warnings {
                ui.separator();
                ui.colored_label(egui::Color32::YELLOW, "⚠ Warnings");
            }
        });
    }
}

/// Preview statistics
#[derive(Debug, Clone)]
pub struct PreviewStats {
    pub line_count: usize,
    pub char_count: usize,
    pub section_count: usize,
    pub has_errors: bool,
    pub has_warnings: bool,
}
