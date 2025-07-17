//! YAML template management

use super::types::*;
use eframe::egui;

impl YamlEditorView {
    /// Render template browser
    pub fn render_template_browser(&mut self, ui: &mut egui::Ui) {
        if !self.show_template_browser {
            return;
        }

        ui.collapsing("📋 Template Browser", |ui| {
            ui.label("Choose a template to start with:");

            // Clone the templates to avoid borrowing issues
            let templates = self.available_templates.clone();

            for template in &templates {
                ui.horizontal(|ui| {
                    if ui.button(&template.name).clicked() {
                        self.load_template(&template.name);
                        self.show_template_browser = false;
                    }

                    ui.label(&template.description);
                    ui.colored_label(egui::Color32::LIGHT_BLUE, &template.category);
                });
            }
        });
    }

    /// Get available templates
    pub fn get_available_templates(&self) -> &[YamlTemplate] {
        &self.available_templates
    }

    /// Add new template
    pub fn add_template(&mut self, template: YamlTemplate) {
        self.available_templates.push(template);
    }

    /// Remove template by name
    pub fn remove_template(&mut self, name: &str) {
        self.available_templates.retain(|t| t.name != name);
    }

    /// Get template by name
    pub fn get_template(&self, name: &str) -> Option<&YamlTemplate> {
        self.available_templates.iter().find(|t| t.name == name)
    }

    /// Create template from current YAML
    pub fn create_template_from_current(
        &mut self,
        name: String,
        description: String,
        category: String,
    ) {
        let template = YamlTemplate {
            name: name.clone(),
            description,
            file_path: format!("custom/{}.yaml", name),
            category,
            content: self.current_yaml.clone(),
            features: self.extract_features_from_yaml(),
        };

        self.add_template(template);
    }

    /// Extract features from current YAML
    fn extract_features_from_yaml(&self) -> Vec<String> {
        let mut features = Vec::new();

        // Parse YAML and extract primal names
        if let Ok(value) = serde_yaml::from_str::<serde_yaml::Value>(&self.current_yaml) {
            if let Some(map) = value.as_mapping() {
                if let Some(primals) = map.get("primals") {
                    if let Some(primals_map) = primals.as_mapping() {
                        for (primal_name, _) in primals_map {
                            if let Some(name_str) = primal_name.as_str() {
                                features.push(name_str.to_string());
                            }
                        }
                    }
                }
            }
        }

        features
    }

    /// Toggle template browser visibility
    pub fn toggle_template_browser(&mut self) {
        self.show_template_browser = !self.show_template_browser;
    }
}
