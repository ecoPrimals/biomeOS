//! Niche Selection and Customization Rendering
//!
//! This module contains UI rendering functions for the niche selection and
//! customization phases of the BYOB (Build Your Own Biome) process.

use super::super::data::get_primal_discovery;
use super::super::templates::get_template_loader;
use super::super::types::*;
use egui::{Color32, RichText, Ui};
use std::collections::HashMap;

/// Render niche selection interface
pub fn render_niche_selection(
    ui: &mut Ui,
    team_data: &TeamData,
    selected_niche: &mut Option<String>,
) -> bool {
    ui.heading("🎯 Choose Your Niche");
    ui.separator();

    let mut next_clicked = false;

    // Show team info summary
    ui.horizontal(|ui| {
        ui.label("Team:");
        ui.label(RichText::new(&team_data.team_info.name).strong());
        ui.label("Focus:");
        ui.label(&team_data.team_info.focus_area);
    });

    if !team_data.team_info.required_capabilities.is_empty() {
        ui.horizontal(|ui| {
            ui.label("Required Capabilities:");
            for cap in &team_data.team_info.required_capabilities {
                ui.label(RichText::new(cap.display_name()).color(Color32::BLUE));
            }
        });
    }

    ui.separator();

    // Get compatible templates
    let loader = get_template_loader();
    let compatible_templates =
        loader.find_compatible_templates(&team_data.team_info.required_capabilities);

    if compatible_templates.is_empty() {
        ui.label(
            RichText::new("No compatible niches found for your capabilities.").color(Color32::RED),
        );
        ui.label("Try selecting different capabilities or check if primals are available.");
        return false;
    }

    ui.heading("Compatible Niches");
    ui.label(format!(
        "Found {} compatible niches:",
        compatible_templates.len()
    ));

    egui::ScrollArea::vertical()
        .max_height(400.0)
        .show(ui, |ui| {
            for template in compatible_templates {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        // Selection radio button
                        let is_selected = selected_niche.as_ref() == Some(&template.id);
                        if ui.radio(is_selected, "").clicked() {
                            *selected_niche = Some(template.id.clone());
                        }

                        ui.vertical(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(RichText::new(&template.name).strong().size(16.0));
                                ui.label(format!("({:?})", template.difficulty));
                                ui.label(template.category.display_name());
                            });

                            ui.label(&template.description);

                            // Show required capabilities
                            if !template.required_capabilities.is_empty() {
                                ui.horizontal(|ui| {
                                    ui.label("Requires:");
                                    for cap in &template.required_capabilities {
                                        ui.label(
                                            RichText::new(cap.display_name()).color(Color32::GREEN),
                                        );
                                    }
                                });
                            }

                            // Show features
                            if !template.features.is_empty() {
                                ui.horizontal(|ui| {
                                    ui.label("Features:");
                                    for feature in &template.features {
                                        ui.label(RichText::new(feature).color(Color32::GRAY));
                                    }
                                });
                            }
                        });
                    });
                });
                ui.add_space(5.0);
            }
        });

    ui.separator();

    ui.horizontal(|ui| {
        if ui.button("Back").clicked() {
            // Handle back navigation
        }

        if selected_niche.is_some() {
            if ui.button("Next: Customize").clicked() {
                next_clicked = true;
            }
        }
    });

    next_clicked
}

/// Render niche customization interface
pub fn render_niche_customization(
    ui: &mut Ui,
    template: &NicheTemplate,
    customizations: &mut HashMap<String, String>,
) -> bool {
    ui.heading("⚙️ Customize Your Niche");
    ui.separator();

    let mut next_clicked = false;

    // Show template info
    ui.horizontal(|ui| {
        ui.label("Niche:");
        ui.label(RichText::new(&template.name).strong());
    });

    ui.label(&template.description);
    ui.separator();

    // Show customization options
    if template.customization_options.is_empty() {
        ui.label("This niche has no customization options.");
    } else {
        ui.heading("Customization Options");

        egui::ScrollArea::vertical()
            .max_height(400.0)
            .show(ui, |ui| {
                for option in &template.customization_options {
                    ui.group(|ui| {
                        ui.vertical(|ui| {
                            ui.label(RichText::new(&option.name).strong());
                            ui.label(&option.description);

                            // Get current value or default
                            let mut current_value = customizations
                                .get(&option.id)
                                .cloned()
                                .unwrap_or_else(|| option.default_value.clone());

                            match &option.option_type {
                                CustomizationType::Text => {
                                    if ui.text_edit_singleline(&mut current_value).changed() {
                                        customizations.insert(option.id.clone(), current_value.clone());
                                    }
                                }
                                CustomizationType::Select(options) => {
                                    let response = egui::ComboBox::from_id_source(&option.id)
                                        .selected_text(&current_value)
                                        .show_ui(ui, |ui| {
                                            for opt in options {
                                                ui.selectable_value(&mut current_value, opt.clone(), opt);
                                            }
                                        });
                                    if response.response.changed() {
                                        customizations.insert(option.id.clone(), current_value.clone());
                                    }
                                }
                                CustomizationType::Number { min, max } => {
                                    let mut value: i32 = current_value.parse().unwrap_or(0);
                                    let mut changed = false;

                                    if let Some(min_val) = min {
                                        if let Some(max_val) = max {
                                            changed = ui
                                                .add(egui::Slider::new(&mut value, *min_val..=*max_val))
                                                .changed();
                                        } else {
                                            changed = ui
                                                .add(egui::Slider::new(&mut value, *min_val..=1000))
                                                .changed();
                                        }
                                    } else {
                                        changed = ui
                                            .add(egui::DragValue::new(&mut value))
                                            .changed();
                                    }

                                    if changed {
                                        customizations.insert(option.id.clone(), value.to_string());
                                    }
                                }
                                CustomizationType::Boolean => {
                                    let mut value: bool = current_value.parse().unwrap_or(false);
                                    if ui.checkbox(&mut value, "").changed() {
                                        customizations.insert(option.id.clone(), value.to_string());
                                    }
                                }
                                CustomizationType::MultiSelect(options) => {
                                    ui.label("Multi-select:");
                                    let current_selections: Vec<String> = current_value
                                        .split(',')
                                        .map(|s| s.trim().to_string())
                                        .filter(|s| !s.is_empty())
                                        .collect();

                                    let mut selections = current_selections.clone();
                                    let mut changed = false;

                                    for opt in options {
                                        let mut selected = selections.contains(opt);
                                        if ui.checkbox(&mut selected, opt).changed() {
                                            changed = true;
                                            if selected {
                                                if !selections.contains(opt) {
                                                    selections.push(opt.clone());
                                                }
                                            } else {
                                                selections.retain(|s| s != opt);
                                            }
                                        }
                                    }

                                    if changed {
                                        let value = selections.join(", ");
                                        customizations.insert(option.id.clone(), value);
                                    }
                                }
                                CustomizationType::Capabilities(_) => {
                                    ui.label("Capability selection not implemented yet");
                                }
                            }

                            if option.required {
                                ui.label(RichText::new("Required").color(Color32::RED));
                            }
                        });
                    });
                    ui.add_space(5.0);
                }
            });
    }

    ui.separator();

    // Show primal compatibility for current customizations
    ui.heading("Primal Compatibility");
    ui.label("Primals available for your customized niche:");

    let discovery = get_primal_discovery();
    let compatible_primals = discovery
        .get_registry()
        .find_primals_by_capabilities(&template.required_capabilities);

    if compatible_primals.is_empty() {
        ui.label(RichText::new("⚠️ No compatible primals found").color(Color32::RED));
    } else {
        for primal in compatible_primals {
            ui.horizontal(|ui| {
                ui.label(format!("• {}", primal.display_name));
                ui.label(format!("({})", primal.description));
            });
        }
    }

    ui.separator();

    ui.horizontal(|ui| {
        if ui.button("Back").clicked() {
            // Handle back navigation
        }

        if ui.button("Next: Generate Manifest").clicked() {
            next_clicked = true;
        }
    });

    next_clicked
} 