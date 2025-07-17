//! ISO Creator View
//!
//! Create bootable biomeOS ISO images with custom configurations,
//! niche packages, and team-specific setups.

use eframe::egui;
use std::collections::HashMap;
use std::process;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::api::BiomeOSApi;
use crate::state::AppState;
use crate::views::{BaseView, View};

/// ISO Creator view for building custom biomeOS distributions
pub struct IsoCreatorView {
    pub base: BaseView,
    pub selected_tab: IsoCreatorTab,
    pub iso_configs: Vec<IsoConfig>,
    pub current_config: IsoConfig,
    pub build_progress: f32,
    pub build_status: BuildStatus,
    pub build_log: Vec<String>,
    pub show_advanced_options: bool,
    pub available_niches: Vec<NichePackage>,
    pub selected_niches: Vec<String>,
    pub custom_components: Vec<CustomComponent>,
    pub iso_templates: Vec<IsoTemplate>,
    pub build_queue: Vec<BuildJob>,
    pub output_directory: String,
    pub iso_name: String,
    pub iso_description: String,
    pub include_all_primals: bool,
    pub include_demos: bool,
    pub include_documentation: bool,
    pub compression_level: u8,
    pub target_architecture: String,
    pub boot_mode: BootMode,
}

#[derive(Debug, Clone, PartialEq)]
pub enum IsoCreatorTab {
    Configuration,
    Niches,
    Components,
    Build,
    Queue,
}

#[derive(Debug, Clone)]
pub struct IsoConfig {
    pub name: String,
    pub description: String,
    pub version: String,
    pub target_arch: String,
    pub boot_mode: BootMode,
    pub included_primals: Vec<String>,
    pub included_niches: Vec<String>,
    pub custom_components: Vec<String>,
    pub compression_level: u8,
    pub size_estimate: u64, // in MB
    pub created_at: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BootMode {
    Legacy,
    UEFI,
    Hybrid,
}

#[derive(Debug, Clone, PartialEq)]
pub enum BuildStatus {
    Idle,
    Preparing,
    Building,
    Packaging,
    Completing,
    Success,
    Failed,
}

#[derive(Debug, Clone)]
pub struct NichePackage {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub category: String,
    pub size_mb: u64,
    pub features: Vec<String>,
    pub dependencies: Vec<String>,
    pub manifest_path: String,
    pub icon_path: Option<String>,
}

#[derive(Debug, Clone)]
pub struct CustomComponent {
    pub name: String,
    pub description: String,
    pub component_type: ComponentType,
    pub source_path: String,
    pub destination_path: String,
    pub size_mb: u64,
    pub required: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ComponentType {
    Binary,
    Library,
    Configuration,
    Documentation,
    Template,
    Script,
}

#[derive(Debug, Clone)]
pub struct IsoTemplate {
    pub name: String,
    pub description: String,
    pub use_case: String,
    pub included_components: Vec<String>,
    pub size_estimate: u64,
    pub difficulty: TemplateDifficulty,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TemplateDifficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone)]
pub struct BuildJob {
    pub id: String,
    pub config: IsoConfig,
    pub status: BuildStatus,
    pub progress: f32,
    pub started_at: Option<String>,
    pub completed_at: Option<String>,
    pub output_path: Option<String>,
    pub error_message: Option<String>,
}

impl IsoCreatorView {
    pub fn new(state: Arc<Mutex<AppState>>, api: Arc<BiomeOSApi>) -> Self {
        Self {
            base: BaseView::new(state, api),
            selected_tab: IsoCreatorTab::Configuration,
            iso_configs: Self::get_mock_configs(),
            current_config: Self::default_config(),
            build_progress: 0.0,
            build_status: BuildStatus::Idle,
            build_log: Vec::new(),
            show_advanced_options: false,
            available_niches: Self::get_mock_niches(),
            selected_niches: Vec::new(),
            custom_components: Self::get_mock_components(),
            iso_templates: Self::get_mock_templates(),
            build_queue: Vec::new(),
            output_directory: "/tmp/biomeos-isos".to_string(),
            iso_name: "biomeOS-custom".to_string(),
            iso_description: "Custom biomeOS distribution".to_string(),
            include_all_primals: true,
            include_demos: true,
            include_documentation: true,
            compression_level: 6,
            target_architecture: "x86_64".to_string(),
            boot_mode: BootMode::Hybrid,
        }
    }

    fn default_config() -> IsoConfig {
        IsoConfig {
            name: "biomeOS-custom".to_string(),
            description: "Custom biomeOS distribution".to_string(),
            version: "1.0.0".to_string(),
            target_arch: "x86_64".to_string(),
            boot_mode: BootMode::Hybrid,
            included_primals: vec![
                "toadstool".to_string(),
                "songbird".to_string(),
                "nestgate".to_string(),
                "squirrel".to_string(),
                "beardog".to_string(),
            ],
            included_niches: Vec::new(),
            custom_components: Vec::new(),
            compression_level: 6,
            size_estimate: 1500, // MB
            created_at: "2024-01-15".to_string(),
        }
    }

    fn get_mock_configs() -> Vec<IsoConfig> {
        vec![
            IsoConfig {
                name: "biomeOS-gaming".to_string(),
                description: "Gaming-optimized biomeOS with tournament support".to_string(),
                version: "1.0.0".to_string(),
                target_arch: "x86_64".to_string(),
                boot_mode: BootMode::UEFI,
                included_primals: vec![
                    "toadstool".to_string(),
                    "songbird".to_string(),
                    "nestgate".to_string(),
                ],
                included_niches: vec![
                    "gaming-tournament".to_string(),
                    "esports-platform".to_string(),
                ],
                custom_components: vec!["game-server-tools".to_string()],
                compression_level: 7,
                size_estimate: 2100,
                created_at: "2024-01-10".to_string(),
            },
            IsoConfig {
                name: "biomeOS-research".to_string(),
                description: "AI research platform with GPU acceleration".to_string(),
                version: "1.2.0".to_string(),
                target_arch: "x86_64".to_string(),
                boot_mode: BootMode::Hybrid,
                included_primals: vec![
                    "toadstool".to_string(),
                    "squirrel".to_string(),
                    "nestgate".to_string(),
                ],
                included_niches: vec!["ai-research".to_string(), "ml-training".to_string()],
                custom_components: vec!["cuda-toolkit".to_string(), "pytorch-models".to_string()],
                compression_level: 5,
                size_estimate: 3500,
                created_at: "2024-01-08".to_string(),
            },
        ]
    }

    fn get_mock_niches() -> Vec<NichePackage> {
        vec![
            NichePackage {
                id: "gaming-tournament".to_string(),
                name: "Gaming Tournament Platform".to_string(),
                description: "Complete tournament management and gaming infrastructure".to_string(),
                author: "Tournament Masters Team".to_string(),
                version: "1.5.0".to_string(),
                category: "Gaming".to_string(),
                size_mb: 450,
                features: vec![
                    "Real-time matchmaking".to_string(),
                    "Physics simulation".to_string(),
                    "Leaderboard system".to_string(),
                    "Anti-cheat integration".to_string(),
                ],
                dependencies: vec!["toadstool".to_string(), "songbird".to_string()],
                manifest_path: "/niches/gaming-tournament/manifest.yaml".to_string(),
                icon_path: Some("/niches/gaming-tournament/icon.png".to_string()),
            },
            NichePackage {
                id: "ai-research".to_string(),
                name: "AI Research Platform".to_string(),
                description: "Machine learning research environment with GPU support".to_string(),
                author: "Deep Learning Lab".to_string(),
                version: "2.1.0".to_string(),
                category: "Research".to_string(),
                size_mb: 1200,
                features: vec![
                    "Distributed training".to_string(),
                    "Model versioning".to_string(),
                    "Dataset management".to_string(),
                    "Experiment tracking".to_string(),
                ],
                dependencies: vec![
                    "toadstool".to_string(),
                    "squirrel".to_string(),
                    "nestgate".to_string(),
                ],
                manifest_path: "/niches/ai-research/manifest.yaml".to_string(),
                icon_path: Some("/niches/ai-research/icon.png".to_string()),
            },
            NichePackage {
                id: "web-development".to_string(),
                name: "Web Development Suite".to_string(),
                description: "Full-stack web development environment".to_string(),
                author: "Frontend Velocity Team".to_string(),
                version: "1.8.2".to_string(),
                category: "Development".to_string(),
                size_mb: 800,
                features: vec![
                    "React/Next.js tools".to_string(),
                    "Auto-scaling frontend".to_string(),
                    "CDN integration".to_string(),
                    "Performance monitoring".to_string(),
                ],
                dependencies: vec![
                    "toadstool".to_string(),
                    "songbird".to_string(),
                    "nestgate".to_string(),
                ],
                manifest_path: "/niches/web-development/manifest.yaml".to_string(),
                icon_path: Some("/niches/web-development/icon.png".to_string()),
            },
        ]
    }

    fn get_mock_components() -> Vec<CustomComponent> {
        vec![
            CustomComponent {
                name: "CUDA Toolkit".to_string(),
                description: "NVIDIA CUDA development toolkit for GPU computing".to_string(),
                component_type: ComponentType::Library,
                source_path: "/components/cuda-toolkit".to_string(),
                destination_path: "/opt/cuda".to_string(),
                size_mb: 2500,
                required: false,
            },
            CustomComponent {
                name: "Game Server Tools".to_string(),
                description: "Specialized tools for game server management".to_string(),
                component_type: ComponentType::Binary,
                source_path: "/components/game-tools".to_string(),
                destination_path: "/usr/local/bin".to_string(),
                size_mb: 150,
                required: false,
            },
            CustomComponent {
                name: "Monitoring Dashboard".to_string(),
                description: "Real-time system monitoring and metrics dashboard".to_string(),
                component_type: ComponentType::Binary,
                source_path: "/components/dashboard".to_string(),
                destination_path: "/opt/monitoring".to_string(),
                size_mb: 200,
                required: false,
            },
        ]
    }

    fn get_mock_templates() -> Vec<IsoTemplate> {
        vec![
            IsoTemplate {
                name: "Minimal biomeOS".to_string(),
                description: "Lightweight biomeOS with core functionality only".to_string(),
                use_case: "Edge computing, IoT, minimal installations".to_string(),
                included_components: vec!["core".to_string(), "toadstool".to_string()],
                size_estimate: 800,
                difficulty: TemplateDifficulty::Beginner,
            },
            IsoTemplate {
                name: "Developer Workstation".to_string(),
                description: "Complete development environment with all tools".to_string(),
                use_case: "Software development, testing, prototyping".to_string(),
                included_components: vec![
                    "all-primals".to_string(),
                    "dev-tools".to_string(),
                    "documentation".to_string(),
                ],
                size_estimate: 2500,
                difficulty: TemplateDifficulty::Intermediate,
            },
            IsoTemplate {
                name: "Enterprise Server".to_string(),
                description: "Production-ready server configuration with security hardening"
                    .to_string(),
                use_case: "Production deployments, enterprise environments".to_string(),
                included_components: vec![
                    "all-primals".to_string(),
                    "security-tools".to_string(),
                    "monitoring".to_string(),
                ],
                size_estimate: 3200,
                difficulty: TemplateDifficulty::Advanced,
            },
        ]
    }

    fn render_tab_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui
                .selectable_label(
                    self.selected_tab == IsoCreatorTab::Configuration,
                    "⚙️ Configuration",
                )
                .clicked()
            {
                self.selected_tab = IsoCreatorTab::Configuration;
            }
            if ui
                .selectable_label(self.selected_tab == IsoCreatorTab::Niches, "🎭 Niches")
                .clicked()
            {
                self.selected_tab = IsoCreatorTab::Niches;
            }
            if ui
                .selectable_label(
                    self.selected_tab == IsoCreatorTab::Components,
                    "🧩 Components",
                )
                .clicked()
            {
                self.selected_tab = IsoCreatorTab::Components;
            }
            if ui
                .selectable_label(self.selected_tab == IsoCreatorTab::Build, "🔨 Build")
                .clicked()
            {
                self.selected_tab = IsoCreatorTab::Build;
            }
            if ui
                .selectable_label(self.selected_tab == IsoCreatorTab::Queue, "📋 Queue")
                .clicked()
            {
                self.selected_tab = IsoCreatorTab::Queue;
            }
        });
    }

    fn render_configuration_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚙️ ISO Configuration");
        ui.add_space(10.0);

        // Quick templates
        self.base.render_card(ui, "📋 Quick Templates", |ui| {
            ui.label("Start with a pre-configured template:");
            ui.horizontal(|ui| {
                for template in &self.iso_templates {
                    if ui.button(&template.name).clicked() {
                        // Apply template configuration
                        self.iso_name = template.name.clone();
                        self.iso_description = template.description.clone();
                    }
                }
            });
        });

        ui.add_space(15.0);

        // Basic configuration
        self.base.render_card(ui, "🏷️ Basic Information", |ui| {
            ui.horizontal(|ui| {
                ui.label("ISO Name:");
                ui.text_edit_singleline(&mut self.iso_name);
            });

            ui.horizontal(|ui| {
                ui.label("Description:");
                ui.text_edit_singleline(&mut self.iso_description);
            });

            ui.horizontal(|ui| {
                ui.label("Architecture:");
                egui::ComboBox::from_label("")
                    .selected_text(&self.target_architecture)
                    .show_ui(ui, |ui| {
                        ui.selectable_value(
                            &mut self.target_architecture,
                            "x86_64".to_string(),
                            "x86_64",
                        );
                        ui.selectable_value(
                            &mut self.target_architecture,
                            "aarch64".to_string(),
                            "aarch64",
                        );
                        ui.selectable_value(
                            &mut self.target_architecture,
                            "riscv64".to_string(),
                            "riscv64",
                        );
                    });
            });

            ui.horizontal(|ui| {
                ui.label("Boot Mode:");
                egui::ComboBox::from_label("")
                    .selected_text(&format!("{:?}", self.boot_mode))
                    .show_ui(ui, |ui| {
                        ui.selectable_value(&mut self.boot_mode, BootMode::Legacy, "Legacy BIOS");
                        ui.selectable_value(&mut self.boot_mode, BootMode::UEFI, "UEFI");
                        ui.selectable_value(
                            &mut self.boot_mode,
                            BootMode::Hybrid,
                            "Hybrid (BIOS + UEFI)",
                        );
                    });
            });
        });

        ui.add_space(15.0);

        // Primal selection
        self.base.render_card(ui, "🧬 Included Primals", |ui| {
            ui.checkbox(&mut self.include_all_primals, "Include all Primals");

            if !self.include_all_primals {
                ui.separator();
                ui.label("Select specific Primals:");

                let primals = vec!["toadstool", "songbird", "nestgate", "squirrel", "beardog"];
                for primal in primals {
                    let mut included = self
                        .current_config
                        .included_primals
                        .contains(&primal.to_string());
                    if ui.checkbox(&mut included, primal).changed() {
                        if included {
                            self.current_config
                                .included_primals
                                .push(primal.to_string());
                        } else {
                            self.current_config.included_primals.retain(|p| p != primal);
                        }
                    }
                }
            }
        });

        ui.add_space(15.0);

        // Additional options
        self.base.render_card(ui, "📦 Additional Content", |ui| {
            ui.checkbox(&mut self.include_demos, "Include demo applications");
            ui.checkbox(&mut self.include_documentation, "Include documentation");

            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label("Compression Level:");
                ui.add(egui::Slider::new(&mut self.compression_level, 1..=9).text("level"));
            });

            ui.add_space(5.0);
            ui.horizontal(|ui| {
                ui.label("Output Directory:");
                ui.text_edit_singleline(&mut self.output_directory);
                if ui.button("Browse").clicked() {
                    // Open directory picker
                }
            });
        });

        ui.add_space(15.0);

        // Size estimation
        self.base.render_card(ui, "📊 Size Estimation", |ui| {
            let base_size = 1000; // Base biomeOS size
            let primal_size = if self.include_all_primals {
                800
            } else {
                self.current_config.included_primals.len() * 160
            };
            let niche_size: usize = self
                .available_niches
                .iter()
                .filter(|n| self.selected_niches.contains(&n.id))
                .map(|n| n.size_mb as usize)
                .sum();
            let component_size: usize = self
                .custom_components
                .iter()
                .map(|c| c.size_mb as usize)
                .sum();
            let demo_size = if self.include_demos { 200 } else { 0 };
            let doc_size = if self.include_documentation { 100 } else { 0 };

            let total_size =
                base_size + primal_size + niche_size + component_size + demo_size + doc_size;

            ui.label(format!("Base biomeOS: {} MB", base_size));
            ui.label(format!("Primals: {} MB", primal_size));
            ui.label(format!("Niches: {} MB", niche_size));
            ui.label(format!("Custom Components: {} MB", component_size));
            ui.label(format!("Demos: {} MB", demo_size));
            ui.label(format!("Documentation: {} MB", doc_size));
            ui.separator();
            ui.strong(format!("Total Estimated Size: {} MB", total_size));
        });
    }

    fn render_niches_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🎭 Niche Packages");
        ui.label("Select pre-configured niche packages to include in your ISO");
        ui.add_space(10.0);

        for niche in &self.available_niches {
            let mut selected = self.selected_niches.contains(&niche.id);

            self.base
                .render_card(ui, &format!("🎭 {}", niche.name), |ui| {
                    ui.horizontal(|ui| {
                        if ui.checkbox(&mut selected, "Include").changed() {
                            if selected {
                                self.selected_niches.push(niche.id.clone());
                            } else {
                                self.selected_niches.retain(|id| id != &niche.id);
                            }
                        }

                        ui.separator();
                        ui.label(format!("v{}", niche.version));
                        ui.separator();
                        ui.label(format!("by {}", niche.author));
                        ui.separator();
                        ui.label(format!("{} MB", niche.size_mb));
                    });

                    ui.label(&niche.description);
                    ui.add_space(5.0);

                    ui.label("Features:");
                    for feature in &niche.features {
                        ui.label(format!("  • {}", feature));
                    }

                    ui.add_space(5.0);
                    ui.label(format!("Dependencies: {}", niche.dependencies.join(", ")));
                });

            ui.add_space(10.0);
        }
    }

    fn render_components_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🧩 Custom Components");
        ui.label("Add custom binaries, libraries, and configurations to your ISO");
        ui.add_space(10.0);

        ui.horizontal(|ui| {
            if ui.button("➕ Add Component").clicked() {
                // Open component addition dialog
            }
            if ui.button("📁 Import from Directory").clicked() {
                // Open directory import dialog
            }
        });

        ui.add_space(15.0);

        for component in &self.custom_components {
            self.base
                .render_card(ui, &format!("🧩 {}", component.name), |ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("Type: {:?}", component.component_type));
                        ui.separator();
                        ui.label(format!("Size: {} MB", component.size_mb));
                        ui.separator();
                        if component.required {
                            ui.colored_label(egui::Color32::RED, "Required");
                        } else {
                            ui.colored_label(egui::Color32::GREEN, "Optional");
                        }
                    });

                    ui.label(&component.description);
                    ui.add_space(5.0);

                    ui.horizontal(|ui| {
                        ui.label(format!("Source: {}", component.source_path));
                        ui.separator();
                        ui.label(format!("Destination: {}", component.destination_path));
                    });

                    ui.add_space(5.0);
                    ui.horizontal(|ui| {
                        if ui.button("✏️ Edit").clicked() {
                            // Edit component
                        }
                        if ui.button("🗑️ Remove").clicked() {
                            // Remove component
                        }
                    });
                });

            ui.add_space(10.0);
        }
    }

    fn render_build_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🔨 Build ISO");

        // Build configuration summary
        ui.group(|ui| {
            ui.label("📋 Build Configuration");
            ui.separator();

            ui.columns(2, |columns| {
                columns[0].label("ISO Name:");
                columns[1].label(&self.current_config.name);

                columns[0].label("Target Architecture:");
                columns[1].label(&self.current_config.target_arch);

                columns[0].label("Boot Mode:");
                columns[1].label(format!("{:?}", self.current_config.boot_mode));

                columns[0].label("Estimated Size:");
                columns[1].label(format!("{} MB", self.current_config.size_estimate));

                columns[0].label("Included Primals:");
                columns[1].label(format!(
                    "{} primals",
                    self.current_config.included_primals.len()
                ));

                columns[0].label("Included Niches:");
                columns[1].label(format!(
                    "{} niches",
                    self.current_config.included_niches.len()
                ));
            });
        });

        ui.separator();

        // Build controls with enhanced feedback
        ui.horizontal(|ui| {
            let can_build = matches!(
                self.build_status,
                BuildStatus::Idle | BuildStatus::Success | BuildStatus::Failed
            );

            if ui
                .add_enabled(can_build, egui::Button::new("🚀 Start Build"))
                .clicked()
            {
                self.start_build();
            }

            if ui
                .add_enabled(!can_build, egui::Button::new("⏹️ Cancel Build"))
                .clicked()
            {
                self.cancel_build();
            }

            if ui.button("📋 Add to Queue").clicked() {
                self.add_to_queue();
            }

            if ui.button("💾 Save Configuration").clicked() {
                self.save_configuration();
            }

            if ui.button("📁 Open Output Directory").clicked() {
                self.open_output_directory();
            }
        });

        ui.separator();

        // Build status and progress
        match self.build_status {
            BuildStatus::Idle => {
                ui.label("Ready to build. Click 'Start Build' to begin.");
            }
            BuildStatus::Preparing => {
                ui.label("🔄 Preparing build environment...");
                ui.add(egui::ProgressBar::new(0.1).text("Preparing"));
            }
            BuildStatus::Building => {
                ui.label("🔨 Building ISO image...");
                ui.add(
                    egui::ProgressBar::new(self.build_progress)
                        .text(format!("{:.1}%", self.build_progress * 100.0)),
                );

                // Simulate build progress
                if self.build_progress < 0.9 {
                    self.build_progress += 0.01;
                }
            }
            BuildStatus::Packaging => {
                ui.label("📦 Packaging ISO...");
                ui.add(egui::ProgressBar::new(0.9).text("Packaging"));
            }
            BuildStatus::Completing => {
                ui.label("✅ Finalizing build...");
                ui.add(egui::ProgressBar::new(0.95).text("Completing"));
            }
            BuildStatus::Success => {
                ui.colored_label(
                    egui::Color32::from_rgb(100, 200, 100),
                    "✅ Build completed successfully!",
                );
                ui.label(format!(
                    "Output: {}/{}.iso",
                    self.output_directory, self.current_config.name
                ));

                ui.horizontal(|ui| {
                    if ui.button("📂 Open File").clicked() {
                        // Simulate opening file
                        self.build_log.push("Opening ISO file...".to_string());
                    }

                    if ui.button("🔄 Build Again").clicked() {
                        self.build_status = BuildStatus::Idle;
                        self.build_progress = 0.0;
                        self.build_log.clear();
                    }
                });
            }
            BuildStatus::Failed => {
                ui.colored_label(egui::Color32::from_rgb(255, 100, 100), "❌ Build failed!");

                ui.horizontal(|ui| {
                    if ui.button("🔄 Retry Build").clicked() {
                        self.start_build();
                    }

                    if ui.button("📋 View Log").clicked() {
                        // Log will be shown below
                    }
                });
            }
        }

        ui.separator();

        // Build log
        ui.collapsing("📜 Build Log", |ui| {
            egui::ScrollArea::vertical()
                .max_height(200.0)
                .show(ui, |ui| {
                    for log_entry in &self.build_log {
                        ui.label(log_entry);
                    }

                    // Auto-scroll to bottom
                    ui.scroll_to_cursor(Some(egui::Align::BOTTOM));
                });
        });

        // Build statistics
        ui.collapsing("📊 Build Statistics", |ui| {
            ui.columns(3, |columns| {
                columns[0].label("Total Builds:");
                columns[0].label(format!("{}", self.build_queue.len()));

                columns[1].label("Success Rate:");
                let success_count = self
                    .build_queue
                    .iter()
                    .filter(|job| job.status == BuildStatus::Success)
                    .count();
                let total_count = self.build_queue.len();
                let success_rate = if total_count > 0 {
                    (success_count as f32 / total_count as f32) * 100.0
                } else {
                    0.0
                };
                columns[1].label(format!("{:.1}%", success_rate));

                columns[2].label("Average Size:");
                let avg_size = if !self.iso_configs.is_empty() {
                    self.iso_configs
                        .iter()
                        .map(|c| c.size_estimate)
                        .sum::<u64>()
                        / self.iso_configs.len() as u64
                } else {
                    0
                };
                columns[2].label(format!("{} MB", avg_size));
            });
        });
    }

    fn start_build(&mut self) {
        self.build_status = BuildStatus::Preparing;
        self.build_progress = 0.0;
        self.build_log.clear();

        // Add initial log entries
        self.build_log
            .push("🚀 Starting build process...".to_string());
        self.build_log
            .push(format!("📋 Configuration: {}", self.current_config.name));
        self.build_log
            .push(format!("🎯 Target: {}", self.current_config.target_arch));
        self.build_log
            .push(format!("💾 Boot Mode: {:?}", self.current_config.boot_mode));

        // Simulate build progression
        std::thread::spawn(|| {
            std::thread::sleep(std::time::Duration::from_secs(1));
        });

        self.build_status = BuildStatus::Building;
        self.build_log
            .push("🔨 Building ISO components...".to_string());

        // Add build steps to log
        for primal in &self.current_config.included_primals {
            self.build_log
                .push(format!("📦 Including primal: {}", primal));
        }

        for niche in &self.current_config.included_niches {
            self.build_log
                .push(format!("🎭 Including niche: {}", niche));
        }
    }

    fn cancel_build(&mut self) {
        self.build_status = BuildStatus::Failed;
        self.build_progress = 0.0;
        self.build_log
            .push("⏹️ Build cancelled by user".to_string());
    }

    fn add_to_queue(&mut self) {
        let job = BuildJob {
            id: format!("build-{}", self.build_queue.len() + 1),
            config: self.current_config.clone(),
            status: BuildStatus::Idle,
            progress: 0.0,
            started_at: None,
            completed_at: None,
            output_path: None,
            error_message: None,
        };

        self.build_queue.push(job);
        self.build_log
            .push("📋 Configuration added to build queue".to_string());
    }

    fn save_configuration(&mut self) {
        self.iso_configs.push(self.current_config.clone());
        self.build_log.push(format!(
            "💾 Configuration '{}' saved",
            self.current_config.name
        ));
    }

    fn open_output_directory(&mut self) {
        self.build_log.push(format!(
            "📁 Opening output directory: {}",
            self.output_directory
        ));

        // Simulate opening directory
        if let Err(e) = process::Command::new("xdg-open")
            .arg(&self.output_directory)
            .spawn()
        {
            self.build_log
                .push(format!("❌ Failed to open directory: {}", e));
        }
    }

    fn render_queue_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("📋 Build Queue");
        ui.label("Manage queued and completed ISO builds");
        ui.add_space(10.0);

        if self.build_queue.is_empty() {
            ui.label("No builds in queue");
        } else {
            for job in &self.build_queue {
                self.base
                    .render_card(ui, &format!("🔨 {}", job.config.name), |ui| {
                        ui.horizontal(|ui| {
                            let status_color = match job.status {
                                BuildStatus::Idle => egui::Color32::GRAY,
                                BuildStatus::Building => egui::Color32::YELLOW,
                                BuildStatus::Success => egui::Color32::GREEN,
                                BuildStatus::Failed => egui::Color32::RED,
                                _ => egui::Color32::LIGHT_BLUE,
                            };

                            ui.colored_label(status_color, format!("{:?}", job.status));
                            ui.separator();
                            ui.label(format!("Progress: {:.0}%", job.progress * 100.0));
                        });

                        if let Some(started) = &job.started_at {
                            ui.label(format!("Started: {}", started));
                        }

                        if let Some(completed) = &job.completed_at {
                            ui.label(format!("Completed: {}", completed));
                        }

                        if let Some(output) = &job.output_path {
                            ui.label(format!("Output: {}", output));
                        }

                        if let Some(error) = &job.error_message {
                            ui.colored_label(egui::Color32::RED, format!("Error: {}", error));
                        }
                    });

                ui.add_space(10.0);
            }
        }
    }
}

impl View for IsoCreatorView {
    fn render(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        ui.heading("💿 ISO Creator");
        ui.label(
            "Create bootable biomeOS ISO images with custom configurations and niche packages",
        );
        ui.separator();

        self.render_tab_bar(ui);
        ui.add_space(10.0);

        match self.selected_tab {
            IsoCreatorTab::Configuration => self.render_configuration_tab(ui),
            IsoCreatorTab::Niches => self.render_niches_tab(ui),
            IsoCreatorTab::Components => self.render_components_tab(ui),
            IsoCreatorTab::Build => self.render_build_tab(ui),
            IsoCreatorTab::Queue => self.render_queue_tab(ui),
        }
    }
}
