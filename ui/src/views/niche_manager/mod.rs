//! Niche Manager Module
//!
//! This module coordinates all niche management functionality, providing
//! a unified interface for creating, editing, testing, and managing niche packages.

use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::api::BiomeOSApi;
use crate::state::AppState;
use crate::views::{BaseView, View};

// Sub-modules
pub mod editor;
pub mod marketplace;
pub mod templates;
pub mod testing;
pub mod types;

// Re-export types for convenience
pub use editor::NicheEditor;
pub use marketplace::{MarketplaceManager, MarketplaceStats};
pub use templates::TemplateManager;
pub use testing::{NicheTester, TestStatistics};
pub use types::*;

/// Main Niche Manager view
pub struct NicheManagerView {
    pub base: BaseView,
    pub selected_tab: NicheManagerTab,
    pub niches: Vec<NichePackage>,
    pub templates: Vec<NicheTemplate>,
    pub current_niche: Option<NichePackage>,
    pub show_niche_editor: bool,
    pub show_template_wizard: bool,
    pub niche_filter: String,
    pub category_filter: String,
    pub difficulty_filter: Option<NicheDifficulty>,
    pub sort_by: NicheSortBy,
    pub editor_mode: NicheEditorMode,
    pub niche_yaml: String,
    pub niche_manifest: NicheManifest,
    pub validation_errors: Vec<String>,
    pub test_results: Vec<TestResult>,
    pub publishing_status: PublishingStatus,
    pub marketplace_niches: Vec<MarketplaceNiche>,
}

impl NicheManagerView {
    /// Create a new NicheManagerView
    pub fn new(state: Arc<Mutex<AppState>>, api: Arc<BiomeOSApi>) -> Self {
        Self {
            base: BaseView::new(state, api),
            selected_tab: NicheManagerTab::Browse,
            niches: Self::get_mock_niches(),
            templates: TemplateManager::get_templates(),
            current_niche: None,
            show_niche_editor: false,
            show_template_wizard: false,
            niche_filter: String::new(),
            category_filter: "All".to_string(),
            difficulty_filter: None,
            sort_by: NicheSortBy::Name,
            editor_mode: NicheEditorMode::Visual,
            niche_yaml: TemplateManager::get_default_niche_yaml(),
            niche_manifest: NicheManifest::default(),
            validation_errors: Vec::new(),
            test_results: Vec::new(),
            publishing_status: PublishingStatus::Idle,
            marketplace_niches: MarketplaceManager::get_featured_niches(),
        }
    }

    /// Get mock niches for development
    fn get_mock_niches() -> Vec<NichePackage> {
        vec![
            NichePackage {
                id: "gaming-tournament".to_string(),
                name: "Gaming Tournament Platform".to_string(),
                description: "Complete tournament management system with real-time matchmaking and leaderboards".to_string(),
                author: "Tournament Masters".to_string(),
                version: "1.5.0".to_string(),
                category: NicheCategory::Gaming,
                difficulty: NicheDifficulty::Intermediate,
                tags: vec!["gaming".to_string(), "tournament".to_string(), "realtime".to_string()],
                features: vec![
                    "Tournament brackets".to_string(),
                    "Real-time matchmaking".to_string(),
                    "Leaderboards".to_string(),
                    "Stream integration".to_string(),
                ],
                requirements: SystemRequirements {
                    min_cpu_cores: 4,
                    min_memory_gb: 16,
                    min_storage_gb: 200,
                    required_features: vec!["low_latency_networking".to_string()],
                    supported_architectures: vec!["x86_64".to_string()],
                },
                manifest_path: "/niches/gaming-tournament/niche.yaml".to_string(),
                icon_path: Some("/niches/gaming-tournament/icon.png".to_string()),
                size_mb: 800,
                downloads: 1200,
                rating: 4.6,
                created_at: "2023-11-20".to_string(),
                updated_at: "2024-01-05".to_string(),
                status: NicheStatus::Published,
            },
            NichePackage {
                id: "web-dev-stack".to_string(),
                name: "Full Stack Web Development".to_string(),
                description: "Complete MEAN/MERN stack development environment with hot reload and debugging".to_string(),
                author: "Web Dev Community".to_string(),
                version: "2.0.5".to_string(),
                category: NicheCategory::Development,
                difficulty: NicheDifficulty::Intermediate,
                tags: vec!["web".to_string(), "fullstack".to_string(), "react".to_string(), "node".to_string()],
                features: vec![
                    "React frontend".to_string(),
                    "Node.js backend".to_string(),
                    "MongoDB database".to_string(),
                    "Hot reload".to_string(),
                ],
                requirements: SystemRequirements::default(),
                manifest_path: "/niches/web-dev-stack/niche.yaml".to_string(),
                icon_path: Some("/niches/web-dev-stack/icon.png".to_string()),
                size_mb: 600,
                downloads: 2100,
                rating: 4.4,
                created_at: "2023-09-15".to_string(),
                updated_at: "2024-01-08".to_string(),
                status: NicheStatus::Published,
            },
            NichePackage {
                id: "ml-research".to_string(),
                name: "ML Research Environment".to_string(),
                description: "Comprehensive machine learning research environment with Jupyter, TensorFlow, and PyTorch".to_string(),
                author: "AI Research Lab".to_string(),
                version: "1.8.3".to_string(),
                category: NicheCategory::Research,
                difficulty: NicheDifficulty::Advanced,
                tags: vec!["ml".to_string(), "ai".to_string(), "research".to_string(), "jupyter".to_string()],
                features: vec![
                    "Jupyter Lab".to_string(),
                    "TensorFlow/PyTorch".to_string(),
                    "GPU support".to_string(),
                    "Dataset management".to_string(),
                ],
                requirements: SystemRequirements {
                    min_cpu_cores: 8,
                    min_memory_gb: 32,
                    min_storage_gb: 500,
                    required_features: vec!["gpu".to_string()],
                    supported_architectures: vec!["x86_64".to_string()],
                },
                manifest_path: "/niches/ml-research/niche.yaml".to_string(),
                icon_path: Some("/niches/ml-research/icon.png".to_string()),
                size_mb: 2500,
                downloads: 380,
                rating: 4.9,
                created_at: "2023-10-10".to_string(),
                updated_at: "2024-01-12".to_string(),
                status: NicheStatus::Published,
            },
        ]
    }

    /// Render the niche workflow header
    fn render_niche_workflow(&mut self, ui: &mut egui::Ui) {
        self.base
            .render_card(ui, "🎯 Niche Management Workflow", |ui| {
                ui.horizontal(|ui| {
                    ui.label("📚 Browse existing niches");
                    ui.separator();
                    ui.label("➕ Create new niche");
                    ui.separator();
                    ui.label("✏️ Edit and customize");
                    ui.separator();
                    ui.label("🧪 Test and validate");
                    ui.separator();
                    ui.label("🏪 Publish to marketplace");
                });
            });
        ui.add_space(10.0);
    }

    /// Render the browse tab
    fn render_browse_tab(&mut self, ui: &mut egui::Ui) {
        // Filters
        ui.horizontal(|ui| {
            ui.label("Filter:");
            ui.text_edit_singleline(&mut self.niche_filter);

            ui.label("Category:");
            egui::ComboBox::from_label("")
                .selected_text(&self.category_filter)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.category_filter, "All".to_string(), "All");
                    ui.selectable_value(&mut self.category_filter, "Gaming".to_string(), "Gaming");
                    ui.selectable_value(
                        &mut self.category_filter,
                        "Development".to_string(),
                        "Development",
                    );
                    ui.selectable_value(
                        &mut self.category_filter,
                        "Research".to_string(),
                        "Research",
                    );
                    ui.selectable_value(
                        &mut self.category_filter,
                        "Enterprise".to_string(),
                        "Enterprise",
                    );
                });

            ui.label("Sort by:");
            egui::ComboBox::from_label("")
                .selected_text(format!("{:?}", self.sort_by))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.sort_by, NicheSortBy::Name, "Name");
                    ui.selectable_value(&mut self.sort_by, NicheSortBy::Rating, "Rating");
                    ui.selectable_value(&mut self.sort_by, NicheSortBy::Downloads, "Downloads");
                    ui.selectable_value(&mut self.sort_by, NicheSortBy::Recent, "Recent");
                });
        });

        ui.add_space(10.0);

        // Niche list
        egui::ScrollArea::vertical().show(ui, |ui| {
            for niche in &self.niches {
                // Apply filters
                if !self.niche_filter.is_empty()
                    && !niche
                        .name
                        .to_lowercase()
                        .contains(&self.niche_filter.to_lowercase())
                {
                    continue;
                }

                if self.category_filter != "All"
                    && format!("{:?}", niche.category) != self.category_filter
                {
                    continue;
                }

                self.base.render_card(ui, &niche.name, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(&niche.description);
                            ui.horizontal(|ui| {
                                ui.label(format!("Author: {}", niche.author));
                                ui.label(format!("Version: {}", niche.version));
                                ui.label(format!("Rating: {:.1}⭐", niche.rating));
                            });
                            ui.horizontal(|ui| {
                                ui.label(format!("Category: {:?}", niche.category));
                                ui.label(format!("Difficulty: {:?}", niche.difficulty));
                                ui.label(format!("Downloads: {}", niche.downloads));
                            });
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Edit").clicked() {
                                self.current_niche = Some(niche.clone());
                                self.selected_tab = NicheManagerTab::Edit;
                            }
                            if ui.button("Test").clicked() {
                                self.current_niche = Some(niche.clone());
                                self.selected_tab = NicheManagerTab::Test;
                            }
                        });
                    });
                });

                ui.add_space(5.0);
            }
        });
    }

    /// Render the create tab
    fn render_enhanced_create_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.button("📝 Start from Scratch").clicked() {
                self.niche_manifest = NicheManifest::default();
                self.niche_yaml = TemplateManager::get_default_niche_yaml();
                self.show_niche_editor = true;
            }

            if ui.button("📋 Use Template").clicked() {
                self.show_template_wizard = true;
            }
        });

        ui.add_space(10.0);

        // Template selection
        if !self.templates.is_empty() {
            self.base.render_card(ui, "📋 Quick Start Templates", |ui| {
                ui.label("Choose a template to get started quickly:");
                ui.add_space(5.0);

                for template in &self.templates {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(&template.name);
                            ui.label(&template.description);
                            ui.label(format!(
                                "Category: {:?} | Difficulty: {:?}",
                                template.category, template.difficulty
                            ));
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Use Template").clicked() {
                                self.niche_yaml = template.template_yaml.clone();
                                self.show_niche_editor = true;
                            }
                        });
                    });
                    ui.separator();
                }
            });
        }

        // Editor
        if self.show_niche_editor {
            self.render_niche_editor(ui);
        }
    }

    /// Render the edit tab
    fn render_edit_tab(&mut self, ui: &mut egui::Ui) {
        if let Some(ref niche) = self.current_niche {
            ui.heading(format!("Editing: {}", niche.name));
            self.render_niche_editor(ui);
        } else {
            ui.label("Select a niche to edit from the Browse tab");
        }
    }

    /// Render the test tab
    fn render_test_tab(&mut self, ui: &mut egui::Ui) {
        if let Some(ref niche) = self.current_niche {
            ui.heading(format!("Testing: {}", niche.name));

            ui.horizontal(|ui| {
                if ui.button("🧪 Run Quick Tests").clicked() {
                    self.test_results = NicheTester::run_quick_tests(&self.niche_manifest);
                }

                if ui.button("🔍 Run Comprehensive Tests").clicked() {
                    self.test_results =
                        NicheTester::run_comprehensive_tests(niche, &self.niche_manifest);
                }

                if ui.button("🗑️ Clear Results").clicked() {
                    self.test_results.clear();
                }
            });

            ui.add_space(10.0);

            // Test results
            if !self.test_results.is_empty() {
                let stats = NicheTester::get_test_statistics(&self.test_results);

                self.base.render_card(ui, "📊 Test Summary", |ui| {
                    ui.horizontal(|ui| {
                        ui.label(format!("Total: {}", stats.total_tests));
                        ui.label(format!("✅ Passed: {}", stats.passed_tests));
                        ui.label(format!("❌ Failed: {}", stats.failed_tests));
                        ui.label(format!("⏭️ Skipped: {}", stats.skipped_tests));
                        ui.label(format!("Success Rate: {:.1}%", stats.success_rate));
                    });
                });

                ui.add_space(5.0);

                egui::ScrollArea::vertical()
                    .max_height(400.0)
                    .show(ui, |ui| {
                        for result in &self.test_results {
                            let (icon, color) = match result.status {
                                TestStatus::Passed => ("✅", egui::Color32::GREEN),
                                TestStatus::Failed => ("❌", egui::Color32::RED),
                                TestStatus::Skipped => ("⏭️", egui::Color32::YELLOW),
                                TestStatus::Running => ("⏳", egui::Color32::BLUE),
                            };

                            ui.horizontal(|ui| {
                                ui.colored_label(color, icon);
                                ui.label(&result.test_name);
                                ui.label(format!("({}ms)", result.duration_ms));
                            });

                            if !result.message.is_empty() {
                                ui.indent("test_message", |ui| {
                                    ui.colored_label(color, &result.message);
                                });
                            }

                            ui.add_space(3.0);
                        }
                    });
            }
        } else {
            ui.label("Select a niche to test from the Browse tab");
        }
    }

    /// Render the marketplace tab
    fn render_marketplace_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🏪 Niche Marketplace");

        // Marketplace stats
        let stats = MarketplaceManager::get_marketplace_stats();
        self.base
            .render_card(ui, "📊 Marketplace Statistics", |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("Total Niches: {}", stats.total_niches));
                    ui.label(format!("Total Downloads: {}", stats.total_downloads));
                    ui.label(format!("Average Rating: {:.1}⭐", stats.average_rating));
                    ui.label(format!("Verified: {}", stats.verified_niches));
                });
            });

        ui.add_space(10.0);

        // Featured niches
        egui::ScrollArea::vertical().show(ui, |ui| {
            for marketplace_niche in &self.marketplace_niches {
                let niche = &marketplace_niche.package;

                self.base.render_card(ui, &niche.name, |ui| {
                    ui.horizontal(|ui| {
                        ui.vertical(|ui| {
                            ui.label(&niche.description);
                            ui.horizontal(|ui| {
                                ui.label(format!("Author: {}", niche.author));
                                ui.label(format!("Version: {}", niche.version));
                                ui.label(format!("Rating: {:.1}⭐", niche.rating));
                                if marketplace_niche.verified {
                                    ui.colored_label(egui::Color32::GREEN, "✓ Verified");
                                }
                                if marketplace_niche.featured {
                                    ui.colored_label(egui::Color32::GOLD, "⭐ Featured");
                                }
                            });
                            ui.horizontal(|ui| {
                                ui.label(format!("Downloads: {}", niche.downloads));
                                ui.label(format!("Size: {} MB", niche.size_mb));
                                ui.label(format!(
                                    "Security Score: {:.1}/10",
                                    marketplace_niche.security_score
                                ));
                            });
                        });

                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Install").clicked() {
                                // Install niche
                                match MarketplaceManager::install_niche(&niche.id) {
                                    Ok(message) => {
                                        // Show success message
                                        println!("{}", message);
                                    }
                                    Err(error) => {
                                        // Show error message
                                        println!("Error: {}", error);
                                    }
                                }
                            }
                            if ui.button("Download").clicked() {
                                match MarketplaceManager::download_niche(&niche.id) {
                                    Ok(message) => {
                                        println!("{}", message);
                                    }
                                    Err(error) => {
                                        println!("Error: {}", error);
                                    }
                                }
                            }
                        });
                    });
                });

                ui.add_space(5.0);
            }
        });
    }

    /// Render the niche editor
    fn render_niche_editor(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.selectable_value(&mut self.editor_mode, NicheEditorMode::Visual, "🎨 Visual");
            ui.selectable_value(&mut self.editor_mode, NicheEditorMode::YAML, "📝 YAML");
            ui.selectable_value(
                &mut self.editor_mode,
                NicheEditorMode::Preview,
                "👁️ Preview",
            );
        });

        ui.add_space(10.0);

        match self.editor_mode {
            NicheEditorMode::Visual => {
                if let Some(next_mode) =
                    NicheEditor::render_enhanced_visual_editor(&mut self.niche_manifest, ui)
                {
                    self.editor_mode = next_mode.clone();
                    if next_mode == NicheEditorMode::YAML {
                        self.niche_yaml =
                            NicheEditor::generate_yaml_from_manifest(&self.niche_manifest);
                    }
                }
            }
            NicheEditorMode::YAML => {
                NicheEditor::render_yaml_editor(
                    &mut self.niche_yaml,
                    &mut self.validation_errors,
                    ui,
                    |ui, title, content| self.base.render_card(ui, title, content),
                );
            }
            NicheEditorMode::Preview => {
                NicheEditor::render_preview_mode(&self.niche_manifest, ui, |ui, title, content| {
                    self.base.render_card(ui, title, content)
                });
            }
        }
    }
}

impl View for NicheManagerView {
    fn render(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        // Render the hierarchical workflow header
        self.render_niche_workflow(ui);

        // Tab bar
        ui.horizontal(|ui| {
            if ui
                .selectable_label(self.selected_tab == NicheManagerTab::Browse, "📚 Browse")
                .clicked()
            {
                self.selected_tab = NicheManagerTab::Browse;
            }
            if ui
                .selectable_label(self.selected_tab == NicheManagerTab::Create, "➕ Create")
                .clicked()
            {
                self.selected_tab = NicheManagerTab::Create;
            }
            if ui
                .selectable_label(self.selected_tab == NicheManagerTab::Edit, "✏️ Edit")
                .clicked()
            {
                self.selected_tab = NicheManagerTab::Edit;
            }
            if ui
                .selectable_label(self.selected_tab == NicheManagerTab::Test, "🧪 Test")
                .clicked()
            {
                self.selected_tab = NicheManagerTab::Test;
            }
            if ui
                .selectable_label(
                    self.selected_tab == NicheManagerTab::Marketplace,
                    "🏪 Marketplace",
                )
                .clicked()
            {
                self.selected_tab = NicheManagerTab::Marketplace;
            }
        });

        ui.add_space(10.0);

        // Render current tab with enhanced workflow
        match self.selected_tab {
            NicheManagerTab::Browse => self.render_browse_tab(ui),
            NicheManagerTab::Create => self.render_enhanced_create_tab(ui),
            NicheManagerTab::Edit => self.render_edit_tab(ui),
            NicheManagerTab::Test => self.render_test_tab(ui),
            NicheManagerTab::Marketplace => self.render_marketplace_tab(ui),
        }
    }
}
