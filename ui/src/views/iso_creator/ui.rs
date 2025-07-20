//! UI rendering components for ISO Creator
//!
//! This module contains all the UI rendering logic for the ISO Creator,
//! including tab management, forms, and progress displays.

use crate::views::iso_creator::types::*;
use crate::views::BaseView;
use eframe::egui;
use std::collections::HashMap;

/// UI renderer for ISO Creator
pub struct IsoCreatorUI {
    selected_tab: IsoCreatorTab,
    show_advanced_options: bool,
    show_help: bool,
    filter_text: String,
    selected_template: Option<String>,
    custom_iso_name: String,
    custom_iso_description: String,
}

impl IsoCreatorUI {
    /// Create a new UI renderer
    pub fn new() -> Self {
        Self {
            selected_tab: IsoCreatorTab::Configuration,
            show_advanced_options: false,
            show_help: false,
            filter_text: String::new(),
            selected_template: None,
            custom_iso_name: String::new(),
            custom_iso_description: String::new(),
        }
    }

    /// Render the main tab bar
    pub fn render_tab_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            let tabs = vec![
                (IsoCreatorTab::Configuration, "⚙️ Configuration"),
                (IsoCreatorTab::Niches, "🎭 Niches"),
                (IsoCreatorTab::Components, "📦 Components"),
                (IsoCreatorTab::Build, "🔨 Build"),
                (IsoCreatorTab::Queue, "📋 Queue"),
            ];

            for (tab, label) in tabs {
                let selected = self.selected_tab == tab;
                if ui.selectable_label(selected, label).clicked() {
                    self.selected_tab = tab;
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("❓ Help").clicked() {
                    self.show_help = !self.show_help;
                }
            });
        });

        // Show help panel if enabled
        if self.show_help {
            self.render_help_panel(ui);
        }
    }

    /// Render help panel
    fn render_help_panel(&self, ui: &mut egui::Ui) {
        ui.collapsing("💡 Help", |ui| match self.selected_tab {
            IsoCreatorTab::Configuration => {
                ui.label("Configuration tab allows you to:");
                ui.label("• Set basic ISO properties (name, description, version)");
                ui.label("• Choose target architecture and boot mode");
                ui.label("• Select which primals to include");
                ui.label("• Apply templates for quick setup");
                ui.label("• Configure compression settings");
            }
            IsoCreatorTab::Niches => {
                ui.label("Niches tab allows you to:");
                ui.label("• Browse available niche packages");
                ui.label("• Filter by category and features");
                ui.label("• Select niches for your ISO");
                ui.label("• View dependencies and size impact");
            }
            IsoCreatorTab::Components => {
                ui.label("Components tab allows you to:");
                ui.label("• Add custom components to your ISO");
                ui.label("• Configure component types and paths");
                ui.label("• Set component requirements");
                ui.label("• Manage component dependencies");
            }
            IsoCreatorTab::Build => {
                ui.label("Build tab allows you to:");
                ui.label("• Start the ISO build process");
                ui.label("• Monitor build progress");
                ui.label("• View build logs and errors");
                ui.label("• Access completed ISO files");
            }
            IsoCreatorTab::Queue => {
                ui.label("Queue tab allows you to:");
                ui.label("• View all queued builds");
                ui.label("• Manage build priorities");
                ui.label("• Monitor running builds");
                ui.label("• Review build history");
            }
        });
    }

    /// Render configuration tab
    pub fn render_configuration_tab(
        &mut self,
        ui: &mut egui::Ui,
        config: &mut IsoConfig,
        templates: &[IsoTemplate],
    ) {
        ui.heading("⚙️ Configuration");
        ui.label("Configure your custom biomeOS ISO");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.set_width(300.0);

                // Basic settings
                ui.group(|ui| {
                    ui.label("📋 Basic Settings");

                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut config.name);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Description:");
                        ui.text_edit_multiline(&mut config.description);
                    });

                    ui.horizontal(|ui| {
                        ui.label("Version:");
                        ui.text_edit_singleline(&mut config.version);
                    });
                });

                ui.add_space(10.0);

                // Architecture and boot settings
                ui.group(|ui| {
                    ui.label("🏗️ Architecture & Boot");

                    ui.horizontal(|ui| {
                        ui.label("Architecture:");
                        egui::ComboBox::from_id_source("arch_combo")
                            .selected_text(&config.target_arch)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(
                                    &mut config.target_arch,
                                    "x86_64".to_string(),
                                    "x86_64",
                                );
                                ui.selectable_value(
                                    &mut config.target_arch,
                                    "aarch64".to_string(),
                                    "aarch64",
                                );
                                ui.selectable_value(
                                    &mut config.target_arch,
                                    "i686".to_string(),
                                    "i686",
                                );
                            });
                    });

                    ui.horizontal(|ui| {
                        ui.label("Boot Mode:");
                        egui::ComboBox::from_id_source("boot_combo")
                            .selected_text(config.boot_mode.display_name())
                            .show_ui(ui, |ui| {
                                for mode in BootMode::all() {
                                    ui.selectable_value(
                                        &mut config.boot_mode,
                                        mode.clone(),
                                        mode.display_name(),
                                    );
                                }
                            });
                    });
                });

                ui.add_space(10.0);

                // Compression settings
                ui.group(|ui| {
                    ui.label("🗜️ Compression");

                    ui.horizontal(|ui| {
                        ui.label("Level:");
                        ui.add(egui::Slider::new(&mut config.compression_level, 0..=9).text(""));
                        ui.label(match config.compression_level {
                            0..=3 => "Fast",
                            4..=6 => "Balanced",
                            7..=9 => "Maximum",
                            _ => "Unknown",
                        });
                    });
                });
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.set_width(300.0);

                // Templates
                ui.group(|ui| {
                    ui.label("📋 Templates");
                    ui.label("Quick start with pre-configured templates");

                    for template in templates {
                        let selected = self.selected_template.as_ref() == Some(&template.name);
                        if ui
                            .selectable_label(
                                selected,
                                &format!("{} {}", template.use_case_icon(), template.name),
                            )
                            .clicked()
                        {
                            self.selected_template = Some(template.name.clone());
                            // Apply template to config
                            self.apply_template_to_config(template, config);
                        }

                        if selected {
                            ui.label(&format!("  {}", template.description));
                            ui.label(&format!(
                                "  {} {}",
                                template.difficulty_icon(),
                                template.difficulty.display_name()
                            ));
                            ui.label(&format!("  📏 ~{} MB", template.size_estimate));
                        }
                    }
                });

                ui.add_space(10.0);

                // Advanced options
                ui.collapsing("🔧 Advanced Options", |ui| {
                    ui.checkbox(&mut self.show_advanced_options, "Show advanced options");

                    if self.show_advanced_options {
                        ui.label("Additional configuration options would go here");
                    }
                });
            });
        });

        ui.add_space(20.0);

        // Primals selection
        ui.group(|ui| {
            ui.label("🧩 Included Primals");
            ui.label("Select which primals to include in your ISO");

            ui.horizontal_wrapped(|ui| {
                let all_primals = vec![
                    (
                        "toadstool",
                        "🍄 Toadstool",
                        "Cooperative networking and communication",
                    ),
                    ("songbird", "🐦 Songbird", "Service mesh and orchestration"),
                    (
                        "nestgate",
                        "🥚 NestGate",
                        "Distributed storage and data management",
                    ),
                    ("squirrel", "🐿️ Squirrel", "Analytics and intelligence"),
                    ("beardog", "🐻 BearDog", "Security and authentication"),
                ];

                for (id, name, description) in all_primals {
                    let mut selected = config.included_primals.contains(&id.to_string());
                    if ui.checkbox(&mut selected, name).clicked() {
                        if selected {
                            if !config.included_primals.contains(&id.to_string()) {
                                config.included_primals.push(id.to_string());
                            }
                        } else {
                            config.included_primals.retain(|p| p != id);
                        }
                    }

                    if ui.is_rect_visible(ui.min_rect()) {
                        ui.label(&format!("  {}", description));
                    }
                }
            });
        });

        ui.add_space(10.0);

        // Size estimation
        ui.group(|ui| {
            ui.label("📏 Size Estimation");
            let estimated_size = config.calculate_size_estimate();
            ui.label(&format!("Estimated size: {} MB", estimated_size));

            ui.horizontal(|ui| {
                ui.label("Breakdown:");
                ui.label(&format!("Base: 500 MB"));
                ui.label(&format!(
                    "Primals: {} MB",
                    config.included_primals.len() * 150
                ));
                ui.label(&format!(
                    "Niches: {} MB",
                    config.included_niches.len() * 400
                ));
                ui.label(&format!(
                    "Components: {} MB",
                    config.custom_components.len() * 150
                ));
            });
        });
    }

    /// Apply template to configuration
    fn apply_template_to_config(&self, template: &IsoTemplate, config: &mut IsoConfig) {
        config.description = template.description.clone();
        config.included_primals = template.included_components.clone();
        config.size_estimate = template.size_estimate;

        // Apply template-specific settings
        match template.use_case.to_lowercase().as_str() {
            s if s.contains("gaming") => {
                config.boot_mode = BootMode::UEFI;
                config.compression_level = 7;
            }
            s if s.contains("research") => {
                config.boot_mode = BootMode::Hybrid;
                config.compression_level = 5;
            }
            s if s.contains("minimal") => {
                config.boot_mode = BootMode::Legacy;
                config.compression_level = 9;
                config.included_primals = vec!["toadstool".to_string()];
            }
            _ => {
                config.boot_mode = BootMode::Hybrid;
                config.compression_level = 6;
            }
        }
    }

    /// Render niches tab
    pub fn render_niches_tab(
        &mut self,
        ui: &mut egui::Ui,
        niches: &[NichePackage],
        selected_niches: &mut Vec<String>,
    ) {
        ui.heading("🎭 Niches");
        ui.label("Browse and select niche packages for your ISO");
        ui.add_space(10.0);

        // Filter bar
        ui.horizontal(|ui| {
            ui.label("🔍 Filter:");
            ui.text_edit_singleline(&mut self.filter_text);

            if ui.button("Clear").clicked() {
                self.filter_text.clear();
            }
        });

        ui.add_space(10.0);

        // Niche categories
        let categories: Vec<String> = niches
            .iter()
            .map(|n| n.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        for category in categories {
            ui.collapsing(
                &format!(
                    "{} {}",
                    match category.as_str() {
                        "Gaming" => "🎮",
                        "Research" => "🔬",
                        "Development" => "💻",
                        "Security" => "🔒",
                        "Network" => "🌐",
                        "Database" => "🗄️",
                        "AI" => "🤖",
                        _ => "📦",
                    },
                    category
                ),
                |ui| {
                    let category_niches: Vec<_> = niches
                        .iter()
                        .filter(|n| n.category == category)
                        .filter(|n| {
                            if self.filter_text.is_empty() {
                                true
                            } else {
                                n.name
                                    .to_lowercase()
                                    .contains(&self.filter_text.to_lowercase())
                                    || n.description
                                        .to_lowercase()
                                        .contains(&self.filter_text.to_lowercase())
                            }
                        })
                        .collect();

                    for niche in category_niches {
                        let selected = selected_niches.contains(&niche.id);

                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                let mut checked = selected;
                                if ui.checkbox(&mut checked, "").clicked() {
                                    if checked {
                                        selected_niches.push(niche.id.clone());
                                    } else {
                                        selected_niches.retain(|id| id != &niche.id);
                                    }
                                }

                                ui.vertical(|ui| {
                                    ui.label(&format!("{} {}", niche.category_icon(), niche.name));
                                    ui.label(&niche.description);
                                    ui.horizontal(|ui| {
                                        ui.label(&format!("👤 {}", niche.author));
                                        ui.label(&format!("📦 v{}", niche.version));
                                        ui.label(&format!("📏 {} MB", niche.size_mb));
                                    });

                                    if !niche.features.is_empty() {
                                        ui.label(&format!(
                                            "✨ Features: {}",
                                            niche.features.join(", ")
                                        ));
                                    }

                                    if !niche.dependencies.is_empty() {
                                        ui.label(&format!(
                                            "🔗 Dependencies: {}",
                                            niche.dependencies.join(", ")
                                        ));
                                    }
                                });
                            });
                        });

                        ui.add_space(5.0);
                    }
                },
            );
        }
    }

    /// Render components tab
    pub fn render_components_tab(
        &mut self,
        ui: &mut egui::Ui,
        components: &mut Vec<CustomComponent>,
    ) {
        ui.heading("📦 Components");
        ui.label("Manage custom components for your ISO");
        ui.add_space(10.0);

        // Add new component button
        if ui.button("➕ Add Component").clicked() {
            components.push(CustomComponent::new(
                "New Component".to_string(),
                "Custom component description".to_string(),
                ComponentType::Binary,
            ));
        }

        ui.add_space(10.0);

        // Component list
        let mut to_remove = Vec::new();

        for (index, component) in components.iter_mut().enumerate() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    ui.label(&format!("{} {}", component.type_icon(), component.name));

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("🗑️").clicked() {
                            to_remove.push(index);
                        }

                        if ui.button("✏️").clicked() {
                            // Edit component (in a real implementation, this would open a dialog)
                        }
                    });
                });

                ui.label(&component.description);
                ui.label(&format!(
                    "Type: {}",
                    component.component_type.display_name()
                ));
                ui.label(&format!("Size: {} MB", component.size_mb));

                if component.required {
                    ui.label("⚠️ Required component");
                }
            });

            ui.add_space(5.0);
        }

        // Remove components marked for deletion
        for &index in to_remove.iter().rev() {
            components.remove(index);
        }
    }

    /// Render build tab
    pub fn render_build_tab(
        &mut self,
        ui: &mut egui::Ui,
        build_status: &BuildStatus,
        build_progress: f32,
        build_log: &[String],
    ) {
        ui.heading("🔨 Build");
        ui.label("Build your custom biomeOS ISO");
        ui.add_space(10.0);

        // Build status
        ui.group(|ui| {
            ui.horizontal(|ui| {
                ui.label("Status:");
                ui.colored_label(
                    build_status.color(),
                    &format!("{} {:?}", build_status.icon(), build_status),
                );
            });

            if *build_status == BuildStatus::Building {
                ui.add(egui::ProgressBar::new(build_progress).show_percentage());
            }
        });

        ui.add_space(10.0);

        // Build controls
        ui.horizontal(|ui| {
            match build_status {
                BuildStatus::Idle => {
                    if ui.button("🚀 Start Build").clicked() {
                        // Start build logic would go here
                    }
                }
                BuildStatus::Building => {
                    if ui.button("⏹️ Cancel Build").clicked() {
                        // Cancel build logic would go here
                    }
                }
                BuildStatus::Success => {
                    if ui.button("🔄 Build Again").clicked() {
                        // Restart build logic would go here
                    }

                    if ui.button("📁 Open Output").clicked() {
                        // Open output directory logic would go here
                    }
                }
                BuildStatus::Failed => {
                    if ui.button("🔄 Retry Build").clicked() {
                        // Retry build logic would go here
                    }
                }
                _ => {}
            }
        });

        ui.add_space(10.0);

        // Build log
        ui.group(|ui| {
            ui.label("📜 Build Log");

            egui::ScrollArea::vertical()
                .max_height(300.0)
                .show(ui, |ui| {
                    for entry in build_log {
                        ui.label(entry);
                    }
                });
        });
    }

    /// Render queue tab
    pub fn render_queue_tab(&mut self, ui: &mut egui::Ui, queue_jobs: &[BuildJob]) {
        ui.heading("📋 Queue");
        ui.label("Manage build queue and job history");
        ui.add_space(10.0);

        if queue_jobs.is_empty() {
            ui.label("No builds in queue");
        } else {
            for job in queue_jobs {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label(&format!("🔨 {}", job.config.name));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            ui.colored_label(
                                job.status.color(),
                                &format!("{} {:?}", job.status.icon(), job.status),
                            );
                        });
                    });

                    ui.label(&job.config.description);
                    ui.label(&format!("Progress: {:.0}%", job.progress * 100.0));

                    if let Some(started) = &job.started_at {
                        ui.label(&format!("Started: {}", started));
                    }

                    if let Some(completed) = &job.completed_at {
                        ui.label(&format!("Completed: {}", completed));
                    }

                    if let Some(output) = &job.output_path {
                        ui.label(&format!("Output: {}", output));
                    }

                    if let Some(error) = &job.error_message {
                        ui.colored_label(egui::Color32::RED, &format!("Error: {}", error));
                    }
                });

                ui.add_space(5.0);
            }
        }
    }

    /// Get current selected tab
    pub fn get_selected_tab(&self) -> &IsoCreatorTab {
        &self.selected_tab
    }

    /// Set selected tab
    pub fn set_selected_tab(&mut self, tab: IsoCreatorTab) {
        self.selected_tab = tab;
    }
}
