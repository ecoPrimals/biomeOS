//! Main YAML editor implementation

use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex;

use super::types::*;
use crate::api::BiomeOSApi;
use crate::state::AppState;
use crate::views::{BaseView, View};

impl YamlEditorView {
    pub fn new(state: Arc<Mutex<AppState>>, api: Arc<BiomeOSApi>) -> Self {
        let mut view = Self {
            base: BaseView::new(state, api),
            current_yaml: String::new(),
            original_yaml: String::new(),
            file_path: None,
            is_modified: false,
            selected_template: None,
            available_templates: Vec::new(),
            validation_errors: Vec::new(),
            validation_warnings: Vec::new(),
            show_preview: false,
            show_validation_panel: true,
            show_template_browser: false,
            cursor_position: 0,
            search_query: String::new(),
            replace_query: String::new(),
            auto_save: false,
            syntax_highlighting: true,
            line_numbers: true,
            word_wrap: false,
            editor_font_size: 14.0,
            yaml_sections: std::collections::HashMap::new(),
            collapsed_sections: std::collections::HashMap::new(),
            editor_mode: EditorMode::Raw,
        };

        // Load default templates
        view.load_default_templates();

        view
    }

    /// Load default YAML templates
    fn load_default_templates(&mut self) {
        self.available_templates = vec![
            YamlTemplate {
                name: "Basic Biome".to_string(),
                description: "Simple biome with essential primals".to_string(),
                file_path: "templates/basic.yaml".to_string(),
                category: "Basic".to_string(),
                content: include_str!("../../templates/basic_biome.yaml").to_string(),
                features: vec!["toadstool".to_string(), "songbird".to_string()],
            },
            YamlTemplate {
                name: "Full Stack".to_string(),
                description: "Complete biome with all primals".to_string(),
                file_path: "templates/full.yaml".to_string(),
                category: "Advanced".to_string(),
                content: include_str!("../../templates/full_biome.yaml").to_string(),
                features: vec![
                    "toadstool".to_string(),
                    "songbird".to_string(),
                    "nestgate".to_string(),
                    "beardog".to_string(),
                    "squirrel".to_string(),
                ],
            },
            YamlTemplate {
                name: "Development".to_string(),
                description: "Development-focused biome".to_string(),
                file_path: "templates/dev.yaml".to_string(),
                category: "Development".to_string(),
                content: include_str!("../../templates/dev_biome.yaml").to_string(),
                features: vec!["toadstool".to_string(), "squirrel".to_string()],
            },
        ];
    }

    /// Load template by name
    pub fn load_template(&mut self, template_name: &str) {
        if let Some(template) = self
            .available_templates
            .iter()
            .find(|t| t.name == template_name)
        {
            self.current_yaml = template.content.clone();
            self.original_yaml = template.content.clone();
            self.selected_template = Some(template_name.to_string());
            self.is_modified = false;
            self.validate_yaml();
        }
    }

    /// Save current YAML to file
    pub fn save_yaml(&mut self) {
        if let Some(_file_path) = &self.file_path {
            // In a real implementation, this would save to the file system
            // For now, we'll just mark as saved
            self.original_yaml = self.current_yaml.clone();
            self.is_modified = false;

            // Show success message
            // Could add a notification system here
        }
    }

    /// Create new YAML file
    pub fn new_yaml(&mut self) {
        self.current_yaml = String::new();
        self.original_yaml = String::new();
        self.file_path = None;
        self.is_modified = false;
        self.selected_template = None;
        self.validation_errors.clear();
        self.validation_warnings.clear();
    }

    /// Check if YAML has been modified
    pub fn is_modified(&self) -> bool {
        self.is_modified
    }

    /// Get current YAML content
    pub fn get_yaml(&self) -> &str {
        &self.current_yaml
    }

    /// Set YAML content
    pub fn set_yaml(&mut self, content: String) {
        self.current_yaml = content;
        self.is_modified = self.current_yaml != self.original_yaml;
        self.validate_yaml();
    }

    /// Render enhanced editor with hierarchical integration
    pub fn render_enhanced_editor(&mut self, ui: &mut egui::Ui) {
        // Integration header
        ui.horizontal(|ui| {
            ui.heading("📝 YAML Editor");
            ui.separator();
            ui.label(format!("Mode: {:?}", self.editor_mode));

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🧬 Use in BYOB").clicked() {
                    self.export_to_byob();
                }

                if ui.button("🎭 Save as Niche").clicked() {
                    self.export_to_niche();
                }

                if ui.button("💿 Build ISO").clicked() {
                    self.export_to_iso();
                }
            });
        });

        ui.add_space(10.0);

        // Workflow integration indicators
        ui.horizontal(|ui| {
            ui.label("💡 Integration:");
            ui.label("BYOB → Niche → Manifest → YAML");

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.small_button("📋 Templates").clicked() {
                    self.show_template_browser = !self.show_template_browser;
                }
            });
        });

        ui.add_space(10.0);

        // Mode selection with enhanced options
        ui.horizontal(|ui| {
            ui.label("Editor Mode:");
            if ui
                .selectable_label(self.editor_mode == EditorMode::Raw, "📝 Raw YAML")
                .clicked()
            {
                self.editor_mode = EditorMode::Raw;
            }
            if ui
                .selectable_label(self.editor_mode == EditorMode::Structured, "🏗️ Structured")
                .clicked()
            {
                self.editor_mode = EditorMode::Structured;
            }
            if ui
                .selectable_label(self.editor_mode == EditorMode::Preview, "👁️ Preview")
                .clicked()
            {
                self.editor_mode = EditorMode::Preview;
            }
        });

        ui.add_space(10.0);

        // Enhanced editor content
        match self.editor_mode {
            EditorMode::Raw => self.render_raw_editor(ui),
            EditorMode::Structured => self.render_structured_editor(ui),
            EditorMode::Preview => self.render_preview_editor(ui),
        }
    }

    /// Render raw YAML editor with enhanced features
    pub fn render_raw_editor(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            // Editor controls
            if ui.button("📁 New").clicked() {
                self.new_yaml();
            }

            if ui.button("💾 Save").clicked() {
                self.save_yaml();
            }

            if ui.button("🔄 Validate").clicked() {
                self.validate_yaml();
            }

            if ui.button("🎨 Format").clicked() {
                self.format_yaml();
            }

            ui.separator();

            // Editor options
            ui.checkbox(&mut self.syntax_highlighting, "🌈 Syntax");
            ui.checkbox(&mut self.line_numbers, "📊 Lines");
            ui.checkbox(&mut self.word_wrap, "📝 Wrap");
        });

        ui.add_space(10.0);

        // Search and replace
        ui.horizontal(|ui| {
            ui.label("🔍 Search:");
            ui.text_edit_singleline(&mut self.search_query);

            ui.label("🔄 Replace:");
            ui.text_edit_singleline(&mut self.replace_query);

            if ui.button("Replace All").clicked() {
                self.replace_all();
            }
        });

        ui.add_space(10.0);

        // Main editor area
        egui::ScrollArea::vertical()
            .id_source("yaml_editor_scroll")
            .show(ui, |ui| {
                let response = ui.add(
                    egui::TextEdit::multiline(&mut self.current_yaml)
                        .font(egui::TextStyle::Monospace)
                        .code_editor()
                        .desired_rows(30)
                        .desired_width(f32::INFINITY),
                );

                if response.changed() {
                    self.is_modified = self.current_yaml != self.original_yaml;
                    if self.auto_save {
                        self.save_yaml();
                    }
                }
            });

        ui.add_space(10.0);

        // Validation panel
        if self.show_validation_panel {
            self.render_validation_results(ui);
        }
    }

    /// Replace all occurrences of search query with replace query
    fn replace_all(&mut self) {
        if !self.search_query.is_empty() {
            self.current_yaml = self
                .current_yaml
                .replace(&self.search_query, &self.replace_query);
            self.is_modified = true;
        }
    }

    /// Format YAML with proper indentation
    pub fn format_yaml(&mut self) {
        // Basic YAML formatting
        let lines: Vec<&str> = self.current_yaml.lines().collect();
        let mut formatted = String::new();

        for line in lines {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                formatted.push('\n');
                continue;
            }

            // Calculate indentation based on nesting level
            let indent_level = self.calculate_indent_level(trimmed);
            let indent = "  ".repeat(indent_level);

            formatted.push_str(&format!("{}{}\n", indent, trimmed));
        }

        self.current_yaml = formatted;
        self.is_modified = true;
    }

    /// Calculate appropriate indentation level for a line
    fn calculate_indent_level(&self, line: &str) -> usize {
        if line.starts_with("apiVersion:")
            || line.starts_with("kind:")
            || line.starts_with("metadata:")
        {
            0
        } else if line.starts_with("name:")
            || line.starts_with("version:")
            || line.starts_with("description:")
        {
            1
        } else if line.starts_with("services:")
            || line.starts_with("primals:")
            || line.starts_with("networking:")
        {
            0
        } else if line.ends_with(':') {
            1
        } else {
            2
        }
    }

    /// Render validation results
    pub fn render_validation_results(&mut self, ui: &mut egui::Ui) {
        self.base.render_card(ui, "❌ Validation Results", |ui| {
            for error in &self.validation_errors {
                ui.colored_label(egui::Color32::RED, format!("• {}", error));
            }

            for warning in &self.validation_warnings {
                ui.colored_label(egui::Color32::YELLOW, format!("⚠ {}", warning));
            }
        });
    }
}

impl View for YamlEditorView {
    fn render(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.heading("📝 biome.yaml Editor");
        ui.separator();

        // Template browser
        self.render_template_browser(ui);

        ui.add_space(10.0);

        // Editor mode tabs
        ui.horizontal(|ui| {
            if ui
                .selectable_label(self.editor_mode == EditorMode::Raw, "✏️ Raw Editor")
                .clicked()
            {
                self.editor_mode = EditorMode::Raw;
            }
            if ui
                .selectable_label(self.editor_mode == EditorMode::Structured, "🏗️ Structured")
                .clicked()
            {
                self.editor_mode = EditorMode::Structured;
            }
            if ui
                .selectable_label(self.editor_mode == EditorMode::Preview, "👁️ Preview")
                .clicked()
            {
                self.editor_mode = EditorMode::Preview;
            }
        });

        ui.add_space(10.0);

        // Main content based on mode
        match self.editor_mode {
            EditorMode::Raw => self.render_raw_editor(ui),
            EditorMode::Structured => self.render_structured_editor(ui),
            EditorMode::Preview => {
                ui.heading("👁️ Preview");
                egui::ScrollArea::vertical().show(ui, |ui| {
                    ui.add(egui::Label::new(
                        egui::RichText::new(&self.current_yaml)
                            .monospace()
                            .color(egui::Color32::LIGHT_GRAY),
                    ));
                });
            }
        }
    }
}
