//! BYOB UI Rendering Functions
//!
//! This module contains all the UI rendering functions for the BYOB system.
//! The rendering is completely universal and capability-based, working with
//! any primal combination without hardcoded names.

use super::data::{get_mock_deployments, get_mock_services, get_mock_teams, get_primal_discovery};
use super::templates::get_template_loader;
use super::types::*;
use egui::{Color32, RichText, Ui};
use std::collections::HashSet;

/// Render team selection interface
pub fn render_team_selection(ui: &mut Ui, team_data: &mut TeamData) -> bool {
    ui.heading("🏗️ Team Setup");
    ui.separator();

    let mut next_clicked = false;

    ui.horizontal(|ui| {
        ui.label("Team Name:");
        ui.text_edit_singleline(&mut team_data.team_info.name);
    });

    ui.horizontal(|ui| {
        ui.label("Description:");
        ui.text_edit_multiline(&mut team_data.team_info.description);
    });

    ui.horizontal(|ui| {
        ui.label("Team Size:");
        egui::ComboBox::from_label("")
            .selected_text(format!("{:?}", team_data.team_info.size))
            .show_ui(ui, |ui| {
                ui.selectable_value(&mut team_data.team_info.size, TeamSize::Solo, "Solo");
                ui.selectable_value(
                    &mut team_data.team_info.size,
                    TeamSize::Small,
                    "Small (2-5)",
                );
                ui.selectable_value(
                    &mut team_data.team_info.size,
                    TeamSize::Medium,
                    "Medium (6-15)",
                );
                ui.selectable_value(
                    &mut team_data.team_info.size,
                    TeamSize::Large,
                    "Large (16+)",
                );
            });
    });

    ui.horizontal(|ui| {
        ui.label("Focus Area:");
        ui.text_edit_singleline(&mut team_data.team_info.focus_area);
    });

    ui.horizontal(|ui| {
        ui.label("Experience Level:");
        egui::ComboBox::from_label("")
            .selected_text(format!("{:?}", team_data.team_info.experience_level))
            .show_ui(ui, |ui| {
                ui.selectable_value(
                    &mut team_data.team_info.experience_level,
                    ExperienceLevel::Beginner,
                    "Beginner",
                );
                ui.selectable_value(
                    &mut team_data.team_info.experience_level,
                    ExperienceLevel::Intermediate,
                    "Intermediate",
                );
                ui.selectable_value(
                    &mut team_data.team_info.experience_level,
                    ExperienceLevel::Advanced,
                    "Advanced",
                );
                ui.selectable_value(
                    &mut team_data.team_info.experience_level,
                    ExperienceLevel::Expert,
                    "Expert",
                );
            });
    });

    ui.separator();

    // Capability selection
    ui.heading("Required Capabilities");
    ui.label("Select the capabilities your team needs:");

    let all_capabilities = vec![
        PrimalCapability::Compute,
        PrimalCapability::Storage,
        PrimalCapability::Networking,
        PrimalCapability::Security,
        PrimalCapability::AI,
        PrimalCapability::Orchestration,
        PrimalCapability::WebDevelopment,
        PrimalCapability::Gaming,
        PrimalCapability::MachineLearning,
        PrimalCapability::Analytics,
    ];

    egui::Grid::new("capabilities_grid")
        .num_columns(3)
        .spacing([10.0, 10.0])
        .show(ui, |ui| {
            for (i, capability) in all_capabilities.iter().enumerate() {
                let mut selected = team_data
                    .team_info
                    .required_capabilities
                    .contains(capability);
                if ui
                    .checkbox(&mut selected, capability.display_name())
                    .changed()
                {
                    if selected {
                        team_data
                            .team_info
                            .required_capabilities
                            .insert(capability.clone());
                    } else {
                        team_data.team_info.required_capabilities.remove(capability);
                    }
                }

                if (i + 1) % 3 == 0 {
                    ui.end_row();
                }
            }
        });

    ui.separator();

    // Show available primals that support selected capabilities
    if !team_data.team_info.required_capabilities.is_empty() {
        ui.heading("Available Primals");
        ui.label("Primals that support your selected capabilities:");

        let discovery = get_primal_discovery();
        let compatible_primals =
            discovery.find_primals_for_capabilities(&team_data.team_info.required_capabilities);

        for primal in compatible_primals {
            ui.horizontal(|ui| {
                ui.label(format!("• {}", primal.display_name));
                ui.label(format!("({})", primal.description));

                // Show matching capabilities
                let matching_caps: Vec<_> = primal
                    .capabilities
                    .intersection(&team_data.team_info.required_capabilities)
                    .collect();

                if !matching_caps.is_empty() {
                    ui.label("Provides:");
                    for cap in matching_caps {
                        ui.label(RichText::new(cap.display_name()).color(Color32::GREEN));
                    }
                }
            });
        }
    }

    ui.separator();

    // Show mock teams for inspiration
    ui.heading("Example Teams");
    ui.label("Get inspiration from these example teams:");

    let mock_teams = get_mock_teams();
    for team in mock_teams {
        ui.horizontal(|ui| {
            if ui.button(&team.name).clicked() {
                team_data.team_info = team.clone();
            }
            ui.label(&team.description);
        });
    }

    ui.separator();

    ui.horizontal(|ui| {
        if ui.button("Next: Choose Niche").clicked() {
            next_clicked = true;
        }
    });

    next_clicked
}

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
    customizations: &mut std::collections::HashMap<String, String>,
) -> bool {
    ui.heading("⚙️ Customize Your Niche");
    ui.separator();

    let mut next_clicked = false;

    ui.horizontal(|ui| {
        ui.label("Niche:");
        ui.label(RichText::new(&template.name).strong());
        ui.label(&template.description);
    });

    ui.separator();

    ui.heading("Customization Options");

    egui::ScrollArea::vertical()
        .max_height(400.0)
        .show(ui, |ui| {
            for option in &template.customization_options {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.horizontal(|ui| {
                            ui.label(RichText::new(&option.name).strong());
                            if option.required {
                                ui.label(RichText::new("*").color(Color32::RED));
                            }
                        });

                        ui.label(&option.description);

                        // Show capability dependency
                        if let Some(cap) = &option.depends_on_capability {
                            ui.label(
                                RichText::new(format!("Requires: {}", cap.display_name()))
                                    .color(Color32::BLUE),
                            );
                        }

                        let current_value = customizations
                            .get(&option.id)
                            .cloned()
                            .unwrap_or_else(|| option.default_value.clone());

                        match &option.option_type {
                            CustomizationType::Text => {
                                let mut value = current_value;
                                if ui.text_edit_singleline(&mut value).changed() {
                                    customizations.insert(option.id.clone(), value);
                                }
                            }
                            CustomizationType::Select(options) => {
                                egui::ComboBox::from_id_source(&option.id)
                                    .selected_text(&current_value)
                                    .show_ui(ui, |ui| {
                                        for opt in options {
                                            let mut current_selection = customizations
                                                .entry(option.id.clone())
                                                .or_insert(current_value.clone())
                                                .clone();
                                            if ui
                                                .selectable_value(
                                                    &mut current_selection,
                                                    opt.clone(),
                                                    opt,
                                                )
                                                .clicked()
                                            {
                                                customizations
                                                    .insert(option.id.clone(), current_selection);
                                                // Value updated automatically
                                            }
                                        }
                                    });
                            }
                            CustomizationType::Number { min, max } => {
                                let mut value: f64 = current_value.parse().unwrap_or(0.0);
                                let mut changed = false;

                                if let Some(min_val) = min {
                                    if let Some(max_val) = max {
                                        changed = ui
                                            .add(egui::Slider::new(
                                                &mut value,
                                                *min_val as f64..=*max_val as f64,
                                            ))
                                            .changed();
                                    } else {
                                        changed = ui
                                            .add(egui::Slider::new(
                                                &mut value,
                                                *min_val as f64..=1000.0,
                                            ))
                                            .changed();
                                    }
                                } else {
                                    changed = ui.add(egui::DragValue::new(&mut value)).changed();
                                }

                                if changed {
                                    customizations.insert(option.id.clone(), value.to_string());
                                }
                            }
                            CustomizationType::Boolean => {
                                let mut value = current_value.parse::<bool>().unwrap_or(false);
                                if ui.checkbox(&mut value, "").changed() {
                                    customizations.insert(option.id.clone(), value.to_string());
                                }
                            }
                            CustomizationType::MultiSelect(options) => {
                                ui.label("Select multiple options:");
                                let selected_values: HashSet<String> = current_value
                                    .split(',')
                                    .map(|s| s.trim().to_string())
                                    .collect();
                                let mut new_selected = selected_values.clone();

                                for opt in options {
                                    let mut selected = selected_values.contains(opt);
                                    if ui.checkbox(&mut selected, opt).changed() {
                                        if selected {
                                            new_selected.insert(opt.clone());
                                        } else {
                                            new_selected.remove(opt);
                                        }
                                    }
                                }

                                if new_selected != selected_values {
                                    let new_value =
                                        new_selected.into_iter().collect::<Vec<_>>().join(",");
                                    customizations.insert(option.id.clone(), new_value);
                                }
                            }
                            CustomizationType::Capabilities(_available_caps) => {
                                ui.label("Select capabilities:");
                                let discovery = get_primal_discovery();
                                let all_caps = discovery.get_registry().get_all_capabilities();

                                let selected_caps: HashSet<String> = current_value
                                    .split(',')
                                    .map(|s| s.trim().to_string())
                                    .filter(|s| !s.is_empty())
                                    .collect();
                                let mut new_selected = selected_caps.clone();

                                for cap in all_caps {
                                    let cap_name =
                                        cap.display_name().to_lowercase().replace(" ", "-");
                                    let mut selected = selected_caps.contains(&cap_name);
                                    if ui.checkbox(&mut selected, cap.display_name()).changed() {
                                        if selected {
                                            new_selected.insert(cap_name);
                                        } else {
                                            new_selected.remove(&cap_name);
                                        }
                                    }
                                }

                                if new_selected != selected_caps {
                                    let new_value =
                                        new_selected.into_iter().collect::<Vec<_>>().join(",");
                                    customizations.insert(option.id.clone(), new_value);
                                }
                            }
                        }
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

        if ui.button("Next: Generate Manifest").clicked() {
            next_clicked = true;
        }
    });

    next_clicked
}

/// Render manifest generation interface
pub fn render_manifest_generation(
    ui: &mut Ui,
    template: &NicheTemplate,
    customizations: &std::collections::HashMap<String, String>,
    generated_manifest: &mut String,
) -> bool {
    ui.heading("📄 Generate Manifest");
    ui.separator();

    let mut next_clicked = false;

    ui.horizontal(|ui| {
        ui.label("Niche:");
        ui.label(RichText::new(&template.name).strong());
    });

    ui.separator();

    // Generate manifest if not already done
    if generated_manifest.is_empty() {
        *generated_manifest = super::templates::generate_manifest(template, customizations);
    }

    ui.heading("Generated Manifest");
    ui.label("Preview of your biome manifest:");

    egui::ScrollArea::vertical()
        .max_height(300.0)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(generated_manifest)
                    .font(egui::TextStyle::Monospace)
                    .code_editor(),
            );
        });

    ui.separator();

    // Show capability analysis
    ui.heading("Capability Analysis");
    ui.label("Required capabilities and available primals:");

    let discovery = get_primal_discovery();
    for capability in &template.required_capabilities {
        let compatible_primals = discovery
            .get_registry()
            .find_primals_by_capability(capability);
        ui.horizontal(|ui| {
            ui.label(format!("• {}", capability.display_name()));
            if compatible_primals.is_empty() {
                ui.label(RichText::new("⚠️ No compatible primals found").color(Color32::RED));
            } else {
                ui.label(format!("✅ {} primals available", compatible_primals.len()));
                for primal in compatible_primals {
                    ui.label(RichText::new(&primal.display_name).color(Color32::GREEN));
                }
            }
        });
    }

    ui.separator();

    ui.horizontal(|ui| {
        if ui.button("Back").clicked() {
            // Handle back navigation
        }

        if ui.button("Next: Edit YAML").clicked() {
            next_clicked = true;
        }

        if ui.button("Save Manifest").clicked() {
            // Handle save functionality
        }
    });

    next_clicked
}

/// Render YAML editing interface
pub fn render_yaml_editing(ui: &mut Ui, manifest: &mut String) -> bool {
    ui.heading("📝 Edit YAML Manifest");
    ui.separator();

    let mut next_clicked = false;

    ui.label("Fine-tune your biome manifest:");

    egui::ScrollArea::vertical()
        .max_height(400.0)
        .show(ui, |ui| {
            ui.add(
                egui::TextEdit::multiline(manifest)
                    .font(egui::TextStyle::Monospace)
                    .code_editor(),
            );
        });

    ui.separator();

    // Validation
    ui.heading("Validation");
    match serde_yaml::from_str::<serde_yaml::Value>(manifest) {
        Ok(_) => {
            ui.label(RichText::new("✅ Valid YAML").color(Color32::GREEN));
        }
        Err(e) => {
            ui.label(RichText::new(format!("❌ Invalid YAML: {}", e)).color(Color32::RED));
        }
    }

    ui.separator();

    ui.horizontal(|ui| {
        if ui.button("Back").clicked() {
            // Handle back navigation
        }

        if ui.button("Validate").clicked() {
            // Handle validation
        }

        if ui.button("Next: Deploy").clicked() {
            next_clicked = true;
        }
    });

    next_clicked
}

/// Render deployment interface
pub fn render_deployment(ui: &mut Ui, deployment_data: &mut DeploymentData) -> bool {
    ui.heading("🚀 Deploy Your Biome");
    ui.separator();

    let mut completed = false;

    // Mock deployment data
    if deployment_data.deployments.is_empty() {
        deployment_data.deployments = get_mock_deployments();
        deployment_data.services = get_mock_services();
        deployment_data.active_deployments = deployment_data.deployments.len();
        deployment_data.total_resource_usage = ResourceUsage {
            cpu_percent: 45.0,
            memory_percent: 55.0,
            storage_percent: 30.0,
            network_mbps: 85.0,
            cpu_cores: 6.0,
            memory_gb: 12.0,
            storage_gb: 75.0,
        };
    }

    // Deployment status
    ui.heading("Deployment Status");
    ui.horizontal(|ui| {
        ui.label("Active Deployments:");
        ui.label(RichText::new(deployment_data.active_deployments.to_string()).strong());
    });

    // Resource usage
    ui.heading("Resource Usage");
    ui.horizontal(|ui| {
        ui.label("CPU:");
        ui.add(
            egui::ProgressBar::new(
                (deployment_data.total_resource_usage.cpu_percent / 100.0) as f32,
            )
            .text(format!(
                "{:.1}%",
                deployment_data.total_resource_usage.cpu_percent
            )),
        );
    });
    ui.horizontal(|ui| {
        ui.label("Memory:");
        ui.add(
            egui::ProgressBar::new(
                (deployment_data.total_resource_usage.memory_percent / 100.0) as f32,
            )
            .text(format!(
                "{:.1}%",
                deployment_data.total_resource_usage.memory_percent
            )),
        );
    });

    ui.separator();

    // Deployments list
    ui.heading("Active Deployments");
    for deployment in &deployment_data.deployments {
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label(RichText::new(&deployment.name).strong());
                ui.label(format!("{:?}", deployment.status));

                // Show primals used
                if !deployment.primals.is_empty() {
                    ui.label("Primals:");
                    for primal in &deployment.primals {
                        ui.label(RichText::new(primal).color(Color32::BLUE));
                    }
                }

                // Show capabilities
                if !deployment.capabilities.is_empty() {
                    ui.label("Capabilities:");
                    for cap in &deployment.capabilities {
                        ui.label(RichText::new(cap.display_name()).color(Color32::GREEN));
                    }
                }
            });
        });
    }

    ui.separator();

    // Services list
    ui.heading("Running Services");
    for service in &deployment_data.services {
        ui.horizontal(|ui| {
            ui.label(&service.name);
            ui.label(format!("{:?}", service.status));
            if let Some(port) = service.port {
                ui.label(format!("Port: {}", port));
            }
            ui.label(format!("Primal: {}", service.primal_name));

            // Show service capabilities
            if !service.capabilities.is_empty() {
                ui.label("Capabilities:");
                for cap in &service.capabilities {
                    ui.label(RichText::new(cap.display_name()).color(Color32::GREEN));
                }
            }
        });
    }

    ui.separator();

    ui.horizontal(|ui| {
        if ui.button("Back").clicked() {
            // Handle back navigation
        }

        if ui.button("Stop Deployment").clicked() {
            // Handle stop
        }

        if ui.button("Complete").clicked() {
            completed = true;
        }
    });

    completed
}

/// Render completion interface
pub fn render_completion(ui: &mut Ui) {
    ui.heading("🎉 Biome Deployment Complete!");
    ui.separator();

    ui.label("Your biome has been successfully deployed using capability-based primal selection.");
    ui.label("The system automatically matched your requirements with available primals.");

    ui.separator();

    ui.heading("What's Next?");
    ui.label("• Monitor your deployment through the dashboard");
    ui.label("• Scale services based on demand");
    ui.label("• Add more capabilities as needed");
    ui.label("• Explore other niche templates");

    ui.separator();

    ui.horizontal(|ui| {
        if ui.button("Create Another Biome").clicked() {
            // Handle restart
        }

        if ui.button("View Dashboard").clicked() {
            // Handle dashboard view
        }
    });
}
