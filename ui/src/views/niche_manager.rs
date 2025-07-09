//! Niche Manager View
//! 
//! Create, edit, and manage niche packages for specialized biomeOS environments.
//! Enables users to build custom niche packages and share them with the community.

use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;

use crate::state::AppState;
use crate::api::BiomeOSApi;
use crate::views::{BaseView, View};

/// Niche Manager view for niche package management
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

#[derive(Debug, Clone, PartialEq)]
pub enum NicheManagerTab {
    Browse,
    Create,
    Edit,
    Test,
    Marketplace,
}

#[derive(Debug, Clone)]
pub struct NichePackage {
    pub id: String,
    pub name: String,
    pub description: String,
    pub author: String,
    pub version: String,
    pub category: NicheCategory,
    pub difficulty: NicheDifficulty,
    pub tags: Vec<String>,
    pub features: Vec<String>,
    pub requirements: SystemRequirements,
    pub manifest_path: String,
    pub icon_path: Option<String>,
    pub size_mb: u64,
    pub downloads: u64,
    pub rating: f32,
    pub created_at: String,
    pub updated_at: String,
    pub status: NicheStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NicheCategory {
    Gaming,
    Research,
    Development,
    Enterprise,
    IoT,
    Education,
    Healthcare,
    Finance,
    Media,
    Custom,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NicheDifficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NicheStatus {
    Draft,
    Testing,
    Published,
    Deprecated,
    Private,
}

#[derive(Debug, Clone)]
pub struct SystemRequirements {
    pub min_cpu_cores: u32,
    pub min_memory_gb: u32,
    pub min_storage_gb: u64,
    pub required_features: Vec<String>,
    pub supported_architectures: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct NicheTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: NicheCategory,
    pub difficulty: NicheDifficulty,
    pub template_yaml: String,
    pub parameters: Vec<TemplateParameter>,
    pub examples: Vec<TemplateExample>,
}

#[derive(Debug, Clone)]
pub struct TemplateParameter {
    pub name: String,
    pub description: String,
    pub param_type: ParameterType,
    pub required: bool,
    pub default_value: Option<String>,
    pub validation: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParameterType {
    String,
    Number,
    Boolean,
    Choice(Vec<String>),
    Array,
}

#[derive(Debug, Clone)]
pub struct TemplateExample {
    pub name: String,
    pub description: String,
    pub parameters: HashMap<String, String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NicheSortBy {
    Name,
    Category,
    Rating,
    Downloads,
    Recent,
    Size,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NicheEditorMode {
    Visual,
    YAML,
    Preview,
}

#[derive(Debug, Clone)]
pub struct NicheManifest {
    pub metadata: NicheMetadata,
    pub services: Vec<ServiceDefinition>,
    pub resources: ResourceRequirements,
    pub networking: NetworkingConfig,
    pub security: SecurityConfig,
    pub dependencies: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct NicheMetadata {
    pub name: String,
    pub version: String,
    pub description: String,
    pub author: String,
    pub license: String,
    pub homepage: Option<String>,
    pub repository: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ServiceDefinition {
    pub name: String,
    pub primal: String,
    pub runtime: String,
    pub image: Option<String>,
    pub command: Vec<String>,
    pub environment: HashMap<String, String>,
    pub ports: Vec<String>,
    pub volumes: Vec<String>,
    pub resources: ServiceResources,
}

#[derive(Debug, Clone)]
pub struct ServiceResources {
    pub cpu: f32,
    pub memory_gb: f32,
    pub storage_gb: f32,
    pub gpu: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct ResourceRequirements {
    pub total_cpu: f32,
    pub total_memory_gb: f32,
    pub total_storage_gb: f32,
    pub gpu_required: bool,
    pub network_bandwidth_mbps: Option<u32>,
}

#[derive(Debug, Clone)]
pub struct NetworkingConfig {
    pub load_balancing: bool,
    pub service_discovery: bool,
    pub ingress_enabled: bool,
    pub mesh_enabled: bool,
}

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub network_policies: bool,
    pub resource_quotas: bool,
    pub encryption_at_rest: bool,
    pub encryption_in_transit: bool,
    pub access_control: String,
}

#[derive(Debug, Clone)]
pub struct TestResult {
    pub test_name: String,
    pub status: TestStatus,
    pub message: String,
    pub duration_ms: u64,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TestStatus {
    Passed,
    Failed,
    Skipped,
    Running,
}

#[derive(Debug, Clone, PartialEq)]
pub enum PublishingStatus {
    Idle,
    Validating,
    Uploading,
    Published,
    Failed,
}

#[derive(Debug, Clone)]
pub struct MarketplaceNiche {
    pub package: NichePackage,
    pub verified: bool,
    pub featured: bool,
    pub security_score: f32,
    pub community_rating: f32,
    pub last_updated: String,
}

impl NicheManagerView {
    pub fn new(state: Arc<Mutex<AppState>>, api: Arc<BiomeOSApi>) -> Self {
        Self {
            base: BaseView::new(state, api),
            selected_tab: NicheManagerTab::Browse,
            niches: Self::get_mock_niches(),
            templates: Self::get_mock_templates(),
            current_niche: None,
            show_niche_editor: false,
            show_template_wizard: false,
            niche_filter: String::new(),
            category_filter: "All".to_string(),
            difficulty_filter: None,
            sort_by: NicheSortBy::Name,
            editor_mode: NicheEditorMode::Visual,
            niche_yaml: Self::default_niche_yaml(),
            niche_manifest: Self::default_manifest(),
            validation_errors: Vec::new(),
            test_results: Vec::new(),
            publishing_status: PublishingStatus::Idle,
            marketplace_niches: Self::get_mock_marketplace(),
        }
    }

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
                    "Real-time matchmaking".to_string(),
                    "Physics simulation".to_string(),
                    "Leaderboard system".to_string(),
                    "Anti-cheat integration".to_string(),
                ],
                requirements: SystemRequirements {
                    min_cpu_cores: 8,
                    min_memory_gb: 16,
                    min_storage_gb: 100,
                    required_features: vec!["gpu".to_string(), "network".to_string()],
                    supported_architectures: vec!["x86_64".to_string()],
                },
                manifest_path: "/niches/gaming-tournament/niche.yaml".to_string(),
                icon_path: Some("/niches/gaming-tournament/icon.png".to_string()),
                size_mb: 450,
                downloads: 1250,
                rating: 4.7,
                created_at: "2024-01-10".to_string(),
                updated_at: "2024-01-15".to_string(),
                status: NicheStatus::Published,
            },
            NichePackage {
                id: "ai-research".to_string(),
                name: "AI Research Laboratory".to_string(),
                description: "Comprehensive machine learning research environment with distributed training capabilities".to_string(),
                author: "Deep Learning Lab".to_string(),
                version: "2.1.0".to_string(),
                category: NicheCategory::Research,
                difficulty: NicheDifficulty::Advanced,
                tags: vec!["ai".to_string(), "ml".to_string(), "research".to_string(), "gpu".to_string()],
                features: vec![
                    "Distributed training".to_string(),
                    "Model versioning".to_string(),
                    "Dataset management".to_string(),
                    "Experiment tracking".to_string(),
                    "Jupyter notebooks".to_string(),
                ],
                requirements: SystemRequirements {
                    min_cpu_cores: 16,
                    min_memory_gb: 64,
                    min_storage_gb: 1000,
                    required_features: vec!["gpu".to_string(), "cuda".to_string()],
                    supported_architectures: vec!["x86_64".to_string()],
                },
                manifest_path: "/niches/ai-research/niche.yaml".to_string(),
                icon_path: Some("/niches/ai-research/icon.png".to_string()),
                size_mb: 1200,
                downloads: 890,
                rating: 4.9,
                created_at: "2024-01-05".to_string(),
                updated_at: "2024-01-12".to_string(),
                status: NicheStatus::Published,
            },
            NichePackage {
                id: "web-dev-suite".to_string(),
                name: "Web Development Suite".to_string(),
                description: "Full-stack web development environment with modern frameworks and tools".to_string(),
                author: "Frontend Velocity".to_string(),
                version: "1.8.2".to_string(),
                category: NicheCategory::Development,
                difficulty: NicheDifficulty::Beginner,
                tags: vec!["web".to_string(), "frontend".to_string(), "fullstack".to_string()],
                features: vec![
                    "React/Next.js development".to_string(),
                    "Auto-scaling frontend".to_string(),
                    "CDN integration".to_string(),
                    "Performance monitoring".to_string(),
                    "Hot reloading".to_string(),
                ],
                requirements: SystemRequirements {
                    min_cpu_cores: 4,
                    min_memory_gb: 8,
                    min_storage_gb: 50,
                    required_features: vec!["network".to_string()],
                    supported_architectures: vec!["x86_64".to_string(), "aarch64".to_string()],
                },
                manifest_path: "/niches/web-dev-suite/niche.yaml".to_string(),
                icon_path: Some("/niches/web-dev-suite/icon.png".to_string()),
                size_mb: 800,
                downloads: 2100,
                rating: 4.5,
                created_at: "2024-01-01".to_string(),
                updated_at: "2024-01-14".to_string(),
                status: NicheStatus::Published,
            },
        ]
    }

    fn get_mock_templates() -> Vec<NicheTemplate> {
        vec![
            NicheTemplate {
                id: "basic-web-app".to_string(),
                name: "Basic Web Application".to_string(),
                description: "Simple web application template with frontend and backend".to_string(),
                category: NicheCategory::Development,
                difficulty: NicheDifficulty::Beginner,
                template_yaml: Self::get_web_app_template(),
                parameters: vec![
                    TemplateParameter {
                        name: "app_name".to_string(),
                        description: "Name of the web application".to_string(),
                        param_type: ParameterType::String,
                        required: true,
                        default_value: Some("my-web-app".to_string()),
                        validation: Some("^[a-z][a-z0-9-]*$".to_string()),
                    },
                    TemplateParameter {
                        name: "framework".to_string(),
                        description: "Frontend framework to use".to_string(),
                        param_type: ParameterType::Choice(vec!["react".to_string(), "vue".to_string(), "angular".to_string()]),
                        required: true,
                        default_value: Some("react".to_string()),
                        validation: None,
                    },
                ],
                examples: vec![
                    TemplateExample {
                        name: "React E-commerce".to_string(),
                        description: "E-commerce site with React frontend".to_string(),
                        parameters: {
                            let mut params = HashMap::new();
                            params.insert("app_name".to_string(), "ecommerce-store".to_string());
                            params.insert("framework".to_string(), "react".to_string());
                            params
                        },
                    },
                ],
            },
        ]
    }

    fn get_web_app_template() -> String {
        r#"# Web Application Niche Template
apiVersion: biomeOS/v1
kind: Niche
metadata:
  name: "{{app_name}}"
  description: "Web application using {{framework}}"
  
services:
  frontend:
    primal: toadstool
    runtime: container
    image: "{{framework}}:latest"
    ports:
      - "3000:3000"
    
  backend:
    primal: toadstool
    runtime: container
    image: "node:18-alpine"
    ports:
      - "8080:8080"
      
  database:
    primal: nestgate
    runtime: container
    image: "postgres:15"
"#.to_string()
    }

    fn default_niche_yaml() -> String {
        r#"# New Niche Package
apiVersion: biomeOS/v1
kind: Niche
metadata:
  name: "my-niche"
  version: "1.0.0"
  description: "My custom niche package"
  author: "Your Name"
  
services:
  main-service:
    primal: toadstool
    runtime: container
    image: "nginx:alpine"
    ports:
      - "80:80"
    resources:
      cpu: 1.0
      memory: 1GB
      
networking:
  load_balancing: true
  service_discovery: true
"#.to_string()
    }

    fn default_manifest() -> NicheManifest {
        NicheManifest {
            metadata: NicheMetadata {
                name: "my-niche".to_string(),
                version: "1.0.0".to_string(),
                description: "My custom niche package".to_string(),
                author: "Your Name".to_string(),
                license: "MIT".to_string(),
                homepage: None,
                repository: None,
            },
            services: vec![
                ServiceDefinition {
                    name: "main-service".to_string(),
                    primal: "toadstool".to_string(),
                    runtime: "container".to_string(),
                    image: Some("nginx:alpine".to_string()),
                    command: vec![],
                    environment: HashMap::new(),
                    ports: vec!["80:80".to_string()],
                    volumes: vec![],
                    resources: ServiceResources {
                        cpu: 1.0,
                        memory_gb: 1.0,
                        storage_gb: 10.0,
                        gpu: None,
                    },
                }
            ],
            resources: ResourceRequirements {
                total_cpu: 1.0,
                total_memory_gb: 1.0,
                total_storage_gb: 10.0,
                gpu_required: false,
                network_bandwidth_mbps: None,
            },
            networking: NetworkingConfig {
                load_balancing: true,
                service_discovery: true,
                ingress_enabled: false,
                mesh_enabled: false,
            },
            security: SecurityConfig {
                network_policies: false,
                resource_quotas: true,
                encryption_at_rest: false,
                encryption_in_transit: false,
                access_control: "none".to_string(),
            },
            dependencies: vec![],
        }
    }

    fn get_mock_marketplace() -> Vec<MarketplaceNiche> {
        vec![
            MarketplaceNiche {
                package: NichePackage {
                    id: "enterprise-crm".to_string(),
                    name: "Enterprise CRM Suite".to_string(),
                    description: "Complete customer relationship management system".to_string(),
                    author: "Enterprise Solutions Inc.".to_string(),
                    version: "3.2.1".to_string(),
                    category: NicheCategory::Enterprise,
                    difficulty: NicheDifficulty::Advanced,
                    tags: vec!["crm".to_string(), "enterprise".to_string(), "business".to_string()],
                    features: vec!["Customer management".to_string(), "Sales pipeline".to_string()],
                    requirements: SystemRequirements {
                        min_cpu_cores: 8,
                        min_memory_gb: 32,
                        min_storage_gb: 500,
                        required_features: vec!["database".to_string()],
                        supported_architectures: vec!["x86_64".to_string()],
                    },
                    manifest_path: "/marketplace/enterprise-crm/niche.yaml".to_string(),
                    icon_path: None,
                    size_mb: 1500,
                    downloads: 450,
                    rating: 4.8,
                    created_at: "2023-12-15".to_string(),
                    updated_at: "2024-01-10".to_string(),
                    status: NicheStatus::Published,
                },
                verified: true,
                featured: true,
                security_score: 9.2,
                community_rating: 4.8,
                last_updated: "2024-01-10".to_string(),
            },
        ]
    }

    fn render_tab_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.selectable_label(self.selected_tab == NicheManagerTab::Browse, "📚 Browse").clicked() {
                self.selected_tab = NicheManagerTab::Browse;
            }
            if ui.selectable_label(self.selected_tab == NicheManagerTab::Create, "➕ Create").clicked() {
                self.selected_tab = NicheManagerTab::Create;
            }
            if ui.selectable_label(self.selected_tab == NicheManagerTab::Edit, "✏️ Edit").clicked() {
                self.selected_tab = NicheManagerTab::Edit;
            }
            if ui.selectable_label(self.selected_tab == NicheManagerTab::Test, "🧪 Test").clicked() {
                self.selected_tab = NicheManagerTab::Test;
            }
            if ui.selectable_label(self.selected_tab == NicheManagerTab::Marketplace, "🏪 Marketplace").clicked() {
                self.selected_tab = NicheManagerTab::Marketplace;
            }
        });
    }

    fn render_browse_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("📚 Browse Niches");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🔄 Refresh").clicked() {
                    // Refresh niche list
                }
            });
        });

        ui.add_space(10.0);

        // Filters and search
        ui.horizontal(|ui| {
            ui.label("Search:");
            ui.text_edit_singleline(&mut self.niche_filter);
            
            ui.separator();
            ui.label("Category:");
            egui::ComboBox::from_label("")
                .selected_text(&self.category_filter)
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.category_filter, "All".to_string(), "All");
                    ui.selectable_value(&mut self.category_filter, "Gaming".to_string(), "Gaming");
                    ui.selectable_value(&mut self.category_filter, "Research".to_string(), "Research");
                    ui.selectable_value(&mut self.category_filter, "Development".to_string(), "Development");
                    ui.selectable_value(&mut self.category_filter, "Enterprise".to_string(), "Enterprise");
                });
            
            ui.separator();
            ui.label("Sort by:");
            egui::ComboBox::from_label("")
                .selected_text(&format!("{:?}", self.sort_by))
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut self.sort_by, NicheSortBy::Name, "Name");
                    ui.selectable_value(&mut self.sort_by, NicheSortBy::Rating, "Rating");
                    ui.selectable_value(&mut self.sort_by, NicheSortBy::Downloads, "Downloads");
                    ui.selectable_value(&mut self.sort_by, NicheSortBy::Recent, "Recent");
                });
        });

        ui.add_space(15.0);

        // Niches grid
        for niche in &self.niches {
            // Apply filters
            if !self.niche_filter.is_empty() && 
               !niche.name.to_lowercase().contains(&self.niche_filter.to_lowercase()) {
                continue;
            }
            
            if self.category_filter != "All" && 
               format!("{:?}", niche.category) != self.category_filter {
                continue;
            }

            self.base.render_card(ui, &format!("🎭 {}", niche.name), |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!("v{}", niche.version));
                    ui.separator();
                    ui.label(format!("by {}", niche.author));
                    ui.separator();
                    ui.label(format!("{:?}", niche.category));
                    ui.separator();
                    
                    let difficulty_color = match niche.difficulty {
                        NicheDifficulty::Beginner => egui::Color32::GREEN,
                        NicheDifficulty::Intermediate => egui::Color32::YELLOW,
                        NicheDifficulty::Advanced => egui::Color32::from_rgb(255, 165, 0),
                        NicheDifficulty::Expert => egui::Color32::RED,
                    };
                    ui.colored_label(difficulty_color, format!("{:?}", niche.difficulty));
                });
                
                ui.label(&niche.description);
                ui.add_space(5.0);
                
                // Rating and stats
                ui.horizontal(|ui| {
                    ui.label(format!("⭐ {:.1}", niche.rating));
                    ui.separator();
                    ui.label(format!("📥 {} downloads", niche.downloads));
                    ui.separator();
                    ui.label(format!("💾 {} MB", niche.size_mb));
                });
                
                ui.add_space(5.0);
                
                // Features
                ui.label("Features:");
                for feature in niche.features.iter().take(3) {
                    ui.label(format!("  • {}", feature));
                }
                if niche.features.len() > 3 {
                    ui.label(format!("  ... and {} more", niche.features.len() - 3));
                }
                
                ui.add_space(5.0);
                
                // Actions
                ui.horizontal(|ui| {
                    if ui.button("👁️ View Details").clicked() {
                        self.current_niche = Some(niche.clone());
                    }
                    if ui.button("✏️ Edit").clicked() {
                        self.current_niche = Some(niche.clone());
                        self.selected_tab = NicheManagerTab::Edit;
                    }
                    if ui.button("🧪 Test").clicked() {
                        self.current_niche = Some(niche.clone());
                        self.selected_tab = NicheManagerTab::Test;
                    }
                    if ui.button("📦 Use in ISO").clicked() {
                        // Navigate to ISO creator with this niche selected
                    }
                });
            });
            
            ui.add_space(10.0);
        }
    }

    fn render_create_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("➕ Create New Niche");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🎯 Use Template").clicked() {
                    self.show_template_wizard = true;
                }
            });
        });

        ui.add_space(10.0);

        // Quick start options
        self.base.render_card(ui, "🚀 Quick Start", |ui| {
            ui.label("Choose a starting point for your niche:");
            ui.horizontal(|ui| {
                if ui.button("🌐 Web Application").clicked() {
                    self.niche_yaml = Self::get_web_app_template();
                }
                if ui.button("🤖 AI/ML Research").clicked() {
                    self.niche_yaml = "# AI/ML Research Niche\n# TODO: Add AI research template".to_string();
                }
                if ui.button("🎮 Gaming Platform").clicked() {
                    self.niche_yaml = "# Gaming Platform Niche\n# TODO: Add gaming template".to_string();
                }
                if ui.button("📊 Data Analytics").clicked() {
                    self.niche_yaml = "# Data Analytics Niche\n# TODO: Add analytics template".to_string();
                }
                if ui.button("📝 Blank Niche").clicked() {
                    self.niche_yaml = Self::default_niche_yaml();
                }
            });
        });

        ui.add_space(15.0);

        // Editor mode selection
        ui.horizontal(|ui| {
            ui.label("Editor Mode:");
            if ui.selectable_label(self.editor_mode == NicheEditorMode::Visual, "🎨 Visual").clicked() {
                self.editor_mode = NicheEditorMode::Visual;
            }
            if ui.selectable_label(self.editor_mode == NicheEditorMode::YAML, "📝 YAML").clicked() {
                self.editor_mode = NicheEditorMode::YAML;
            }
            if ui.selectable_label(self.editor_mode == NicheEditorMode::Preview, "👁️ Preview").clicked() {
                self.editor_mode = NicheEditorMode::Preview;
            }
        });

        ui.add_space(10.0);

        // Editor content
        match self.editor_mode {
            NicheEditorMode::Visual => self.render_visual_editor(ui),
            NicheEditorMode::YAML => self.render_yaml_editor(ui),
            NicheEditorMode::Preview => self.render_preview_mode(ui),
        }

        // Template wizard
        if self.show_template_wizard {
            egui::Window::new("Template Wizard")
                .collapsible(false)
                .resizable(true)
                .default_size([600.0, 400.0])
                .show(ui.ctx(), |ui| {
                    ui.heading("Choose a Template");
                    ui.separator();
                    
                    for template in &self.templates {
                        self.base.render_card(ui, &template.name, |ui| {
                            ui.label(&template.description);
                            ui.add_space(5.0);
                            ui.horizontal(|ui| {
                                ui.label(format!("Category: {:?}", template.category));
                                ui.separator();
                                ui.label(format!("Difficulty: {:?}", template.difficulty));
                            });
                            ui.add_space(5.0);
                            if ui.button("Use This Template").clicked() {
                                self.niche_yaml = template.template_yaml.clone();
                                self.show_template_wizard = false;
                            }
                        });
                        ui.add_space(5.0);
                    }
                    
                    ui.add_space(10.0);
                    if ui.button("Cancel").clicked() {
                        self.show_template_wizard = false;
                    }
                });
        }
    }

    fn render_visual_editor(&mut self, ui: &mut egui::Ui) {
        self.base.render_card(ui, "📋 Niche Metadata", |ui| {
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut self.niche_manifest.metadata.name);
            });
            
            ui.horizontal(|ui| {
                ui.label("Version:");
                ui.text_edit_singleline(&mut self.niche_manifest.metadata.version);
            });
            
            ui.horizontal(|ui| {
                ui.label("Author:");
                ui.text_edit_singleline(&mut self.niche_manifest.metadata.author);
            });
            
            ui.label("Description:");
            ui.text_edit_multiline(&mut self.niche_manifest.metadata.description);
        });

        ui.add_space(10.0);

        self.base.render_card(ui, "🔧 Services", |ui| {
            ui.label("Configure the services in your niche:");
            
            for (i, service) in self.niche_manifest.services.iter_mut().enumerate() {
                ui.collapsing(format!("Service {}: {}", i + 1, service.name), |ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut service.name);
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Primal:");
                        egui::ComboBox::from_label("")
                            .selected_text(&service.primal)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut service.primal, "toadstool".to_string(), "toadstool");
                                ui.selectable_value(&mut service.primal, "songbird".to_string(), "songbird");
                                ui.selectable_value(&mut service.primal, "nestgate".to_string(), "nestgate");
                                ui.selectable_value(&mut service.primal, "squirrel".to_string(), "squirrel");
                            });
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Runtime:");
                        egui::ComboBox::from_label("")
                            .selected_text(&service.runtime)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut service.runtime, "container".to_string(), "container");
                                ui.selectable_value(&mut service.runtime, "native".to_string(), "native");
                                ui.selectable_value(&mut service.runtime, "wasm".to_string(), "wasm");
                            });
                    });
                    
                    if let Some(ref mut image) = service.image {
                        ui.horizontal(|ui| {
                            ui.label("Image:");
                            ui.text_edit_singleline(image);
                        });
                    }
                });
            }
            
            if ui.button("➕ Add Service").clicked() {
                self.niche_manifest.services.push(ServiceDefinition {
                    name: format!("service-{}", self.niche_manifest.services.len() + 1),
                    primal: "toadstool".to_string(),
                    runtime: "container".to_string(),
                    image: Some("nginx:alpine".to_string()),
                    command: vec![],
                    environment: HashMap::new(),
                    ports: vec![],
                    volumes: vec![],
                    resources: ServiceResources {
                        cpu: 1.0,
                        memory_gb: 1.0,
                        storage_gb: 10.0,
                        gpu: None,
                    },
                });
            }
        });
    }

    fn render_yaml_editor(&mut self, ui: &mut egui::Ui) {
        self.base.render_card(ui, "📝 YAML Editor", |ui| {
            ui.label("Edit the niche YAML directly:");
            ui.add_space(5.0);
            
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    ui.add(egui::TextEdit::multiline(&mut self.niche_yaml)
                        .code_editor()
                        .desired_rows(20)
                        .desired_width(f32::INFINITY));
                });
            
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("✅ Validate").clicked() {
                    // Validate YAML
                    self.validation_errors.clear();
                    // Mock validation
                    if self.niche_yaml.contains("invalid") {
                        self.validation_errors.push("Invalid YAML syntax".to_string());
                    }
                }
                if ui.button("💾 Save").clicked() {
                    // Save niche
                }
                if ui.button("🔄 Reset").clicked() {
                    self.niche_yaml = Self::default_niche_yaml();
                }
            });
        });

        // Validation results
        if !self.validation_errors.is_empty() {
            ui.add_space(10.0);
            self.base.render_card(ui, "❌ Validation Errors", |ui| {
                for error in &self.validation_errors {
                    ui.colored_label(egui::Color32::RED, format!("• {}", error));
                }
            });
        }
    }

    fn render_preview_mode(&mut self, ui: &mut egui::Ui) {
        self.base.render_card(ui, "👁️ Niche Preview", |ui| {
            ui.heading(&self.niche_manifest.metadata.name);
            ui.label(format!("Version: {}", self.niche_manifest.metadata.version));
            ui.label(format!("Author: {}", self.niche_manifest.metadata.author));
            ui.add_space(5.0);
            ui.label(&self.niche_manifest.metadata.description);
            
            ui.add_space(10.0);
            ui.separator();
            ui.heading("Services");
            
            for service in &self.niche_manifest.services {
                ui.horizontal(|ui| {
                    ui.label(format!("• {} ({})", service.name, service.primal));
                    if let Some(ref image) = service.image {
                        ui.label(format!("- {}", image));
                    }
                });
            }
            
            ui.add_space(10.0);
            ui.separator();
            ui.heading("Resource Requirements");
            ui.label(format!("CPU: {:.1} cores", self.niche_manifest.resources.total_cpu));
            ui.label(format!("Memory: {:.1} GB", self.niche_manifest.resources.total_memory_gb));
            ui.label(format!("Storage: {:.1} GB", self.niche_manifest.resources.total_storage_gb));
        });
    }

    fn render_edit_tab(&mut self, ui: &mut egui::Ui) {
        if let Some(ref niche) = self.current_niche {
            ui.heading(&format!("✏️ Editing: {}", niche.name));
            ui.add_space(10.0);
            
            // Similar to create tab but with existing niche data
            self.render_yaml_editor(ui);
        } else {
            ui.label("Select a niche to edit from the Browse tab");
        }
    }

    fn render_test_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🧪 Test Niche");
        ui.add_space(10.0);

        if let Some(ref niche) = self.current_niche {
            ui.label(format!("Testing: {}", niche.name));
            ui.add_space(10.0);
            
            self.base.render_card(ui, "🔬 Test Suite", |ui| {
                ui.horizontal(|ui| {
                    if ui.button("▶️ Run All Tests").clicked() {
                        // Run tests
                        self.test_results = vec![
                            TestResult {
                                test_name: "YAML Validation".to_string(),
                                status: TestStatus::Passed,
                                message: "Niche YAML is valid".to_string(),
                                duration_ms: 45,
                            },
                            TestResult {
                                test_name: "Resource Requirements".to_string(),
                                status: TestStatus::Passed,
                                message: "Resource requirements are reasonable".to_string(),
                                duration_ms: 12,
                            },
                            TestResult {
                                test_name: "Service Dependencies".to_string(),
                                status: TestStatus::Passed,
                                message: "All service dependencies are available".to_string(),
                                duration_ms: 89,
                            },
                        ];
                    }
                    if ui.button("🔄 Clear Results").clicked() {
                        self.test_results.clear();
                    }
                });
            });

            ui.add_space(15.0);

            // Test results
            if !self.test_results.is_empty() {
                self.base.render_card(ui, "📊 Test Results", |ui| {
                    for result in &self.test_results {
                        ui.horizontal(|ui| {
                            let status_color = match result.status {
                                TestStatus::Passed => egui::Color32::GREEN,
                                TestStatus::Failed => egui::Color32::RED,
                                TestStatus::Skipped => egui::Color32::YELLOW,
                                TestStatus::Running => egui::Color32::BLUE,
                            };
                            
                            ui.colored_label(status_color, format!("{:?}", result.status));
                            ui.label(&result.test_name);
                            ui.label(format!("({}ms)", result.duration_ms));
                        });
                        ui.label(format!("  {}", result.message));
                        ui.add_space(3.0);
                    }
                });
            }
        } else {
            ui.label("Select a niche to test from the Browse tab");
        }
    }

    fn render_marketplace_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🏪 Niche Marketplace");
        ui.label("Discover and share niches with the community");
        ui.add_space(10.0);

        // Publishing section
        self.base.render_card(ui, "📤 Publish Your Niche", |ui| {
            ui.label("Share your niche with the biomeOS community:");
            ui.add_space(5.0);
            
            ui.horizontal(|ui| {
                if ui.button("📦 Package & Publish").clicked() {
                    self.publishing_status = PublishingStatus::Validating;
                }
                
                let status_text = match self.publishing_status {
                    PublishingStatus::Idle => "Ready to publish",
                    PublishingStatus::Validating => "Validating...",
                    PublishingStatus::Uploading => "Uploading...",
                    PublishingStatus::Published => "Published successfully!",
                    PublishingStatus::Failed => "Publishing failed",
                };
                
                let status_color = match self.publishing_status {
                    PublishingStatus::Idle => egui::Color32::GRAY,
                    PublishingStatus::Validating | PublishingStatus::Uploading => egui::Color32::YELLOW,
                    PublishingStatus::Published => egui::Color32::GREEN,
                    PublishingStatus::Failed => egui::Color32::RED,
                };
                
                ui.colored_label(status_color, status_text);
            });
        });

        ui.add_space(15.0);

        // Featured niches
        self.base.render_card(ui, "⭐ Featured Niches", |ui| {
            for marketplace_niche in &self.marketplace_niches {
                let niche = &marketplace_niche.package;
                
                ui.horizontal(|ui| {
                    ui.label(&niche.name);
                    if marketplace_niche.verified {
                        ui.colored_label(egui::Color32::BLUE, "✓ Verified");
                    }
                    if marketplace_niche.featured {
                        ui.colored_label(egui::Color32::GOLD, "⭐ Featured");
                    }
                });
                
                ui.label(&niche.description);
                ui.horizontal(|ui| {
                    ui.label(format!("⭐ {:.1}", marketplace_niche.community_rating));
                    ui.separator();
                    ui.label(format!("🔒 Security: {:.1}/10", marketplace_niche.security_score));
                    ui.separator();
                    ui.label(format!("📥 {} downloads", niche.downloads));
                });
                
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    if ui.button("📥 Download").clicked() {
                        // Download niche
                    }
                    if ui.button("👁️ View Details").clicked() {
                        // View marketplace details
                    }
                });
                
                ui.separator();
                ui.add_space(5.0);
            }
        });
    }

    /// Render the main niche manager interface with hierarchical workflow
    fn render_niche_workflow(&mut self, ui: &mut egui::Ui) {
        // Workflow header
        ui.horizontal(|ui| {
            ui.heading("🎭 Niche Package Manager");
            ui.separator();
            
            // Workflow mode indicator
            let mode_text = match self.editor_mode {
                NicheEditorMode::Visual => "Visual Editor",
                NicheEditorMode::YAML => "YAML Editor", 
                NicheEditorMode::Preview => "Preview Mode",
            };
            ui.label(format!("Mode: {}", mode_text));
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🔄 Reset").clicked() {
                    self.reset_niche_workflow();
                }
            });
        });

        ui.add_space(10.0);

        // Workflow progress for creation
        if self.selected_tab == NicheManagerTab::Create {
            ui.horizontal(|ui| {
                let steps = ["Template", "Configure", "Manifest", "YAML", "Test"];
                let current_step = match self.editor_mode {
                    NicheEditorMode::Visual => 1,
                    NicheEditorMode::YAML => 3,
                    NicheEditorMode::Preview => 2,
                };
                
                for (i, step) in steps.iter().enumerate() {
                    let color = if i <= current_step {
                        egui::Color32::from_rgb(0, 150, 0)
                    } else {
                        egui::Color32::GRAY
                    };
                    
                    ui.colored_label(color, format!("{}. {}", i + 1, step));
                    
                    if i < steps.len() - 1 {
                        ui.label("→");
                    }
                }
            });
            
            ui.add_space(15.0);
        }

        // Integration with BYOB workflow
        if self.selected_tab == NicheManagerTab::Browse {
            ui.horizontal(|ui| {
                ui.label("💡 Tip: Select a niche to use in");
                if ui.button("🧬 BYOB Workflow").clicked() {
                    // Navigate to BYOB with selected niche
                    // This would be handled by the parent app
                }
            });
            ui.add_space(10.0);
        }
    }

    /// Reset the niche workflow to initial state
    fn reset_niche_workflow(&mut self) {
        self.editor_mode = NicheEditorMode::Visual;
        self.niche_yaml = Self::default_niche_yaml();
        self.niche_manifest = Self::default_manifest();
        self.validation_errors.clear();
        self.test_results.clear();
        self.current_niche = None;
        self.show_niche_editor = false;
        self.show_template_wizard = false;
    }

    /// Enhanced create tab with better workflow
    fn render_enhanced_create_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("➕ Create New Niche");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("📋 Use Template").clicked() {
                    self.show_template_wizard = true;
                }
                
                if ui.button("🧬 Export to BYOB").clicked() {
                    self.export_to_byob();
                }
            });
        });

        ui.add_space(10.0);

        // Enhanced quick start with better categorization
        ui.heading("🚀 Quick Start Templates");
        ui.label("Choose a starting point for your niche:");
        ui.add_space(5.0);
        
        // Categorized templates
        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("🌐 Web & API");
                if ui.button("React/Next.js").clicked() {
                    self.load_template("web-react");
                }
                if ui.button("Node.js API").clicked() {
                    self.load_template("api-nodejs");
                }
                if ui.button("Full Stack").clicked() {
                    self.load_template("fullstack");
                }
            });
            
            ui.separator();
            
            ui.vertical(|ui| {
                ui.label("🤖 AI & ML");
                if ui.button("PyTorch Research").clicked() {
                    self.load_template("ai-pytorch");
                }
                if ui.button("Jupyter Lab").clicked() {
                    self.load_template("jupyter");
                }
                if ui.button("Model Training").clicked() {
                    self.load_template("ml-training");
                }
            });
            
            ui.separator();
            
            ui.vertical(|ui| {
                ui.label("🎮 Gaming");
                if ui.button("Game Server").clicked() {
                    self.load_template("game-server");
                }
                if ui.button("Tournament").clicked() {
                    self.load_template("tournament");
                }
                if ui.button("Matchmaking").clicked() {
                    self.load_template("matchmaking");
                }
            });
            
            ui.separator();
            
            ui.vertical(|ui| {
                ui.label("📊 Data & Analytics");
                if ui.button("Data Pipeline").clicked() {
                    self.load_template("data-pipeline");
                }
                if ui.button("Analytics Dashboard").clicked() {
                    self.load_template("analytics");
                }
                if ui.button("ETL Workflow").clicked() {
                    self.load_template("etl");
                }
            });
        });
        
        ui.add_space(10.0);
        
        ui.horizontal(|ui| {
            if ui.button("📝 Blank Niche").clicked() {
                self.niche_yaml = Self::default_niche_yaml();
                self.editor_mode = NicheEditorMode::YAML;
            }
            
            if ui.button("🔍 Browse Community").clicked() {
                self.selected_tab = NicheManagerTab::Marketplace;
            }
        });

        ui.add_space(15.0);

        // Enhanced editor mode selection
        ui.horizontal(|ui| {
            ui.label("Editor Mode:");
            if ui.selectable_label(self.editor_mode == NicheEditorMode::Visual, "🎨 Visual Builder").clicked() {
                self.editor_mode = NicheEditorMode::Visual;
            }
            if ui.selectable_label(self.editor_mode == NicheEditorMode::YAML, "📝 YAML Editor").clicked() {
                self.editor_mode = NicheEditorMode::YAML;
            }
            if ui.selectable_label(self.editor_mode == NicheEditorMode::Preview, "👁️ Preview").clicked() {
                self.editor_mode = NicheEditorMode::Preview;
            }
        });

        ui.add_space(10.0);

        // Enhanced editor content
        match self.editor_mode {
            NicheEditorMode::Visual => self.render_enhanced_visual_editor(ui),
            NicheEditorMode::YAML => self.render_yaml_editor(ui),
            NicheEditorMode::Preview => self.render_preview_mode(ui),
        }

        // Enhanced template wizard
        if self.show_template_wizard {
            self.render_enhanced_template_wizard(ui);
        }
    }

    /// Load a template by name
    fn load_template(&mut self, template_name: &str) {
        self.niche_yaml = match template_name {
            "web-react" => self.get_react_template(),
            "api-nodejs" => self.get_nodejs_api_template(),
            "fullstack" => self.get_fullstack_template(),
            "ai-pytorch" => self.get_pytorch_template(),
            "jupyter" => self.get_jupyter_template(),
            "ml-training" => self.get_ml_training_template(),
            "game-server" => self.get_game_server_template(),
            "tournament" => self.get_tournament_template(),
            "matchmaking" => self.get_matchmaking_template(),
            "data-pipeline" => self.get_data_pipeline_template(),
            "analytics" => self.get_analytics_template(),
            "etl" => self.get_etl_template(),
            _ => Self::default_niche_yaml(),
        };
        
        self.editor_mode = NicheEditorMode::YAML;
    }

    /// Export current niche to BYOB workflow
    fn export_to_byob(&mut self) {
        // This would be implemented to pass the niche to BYOB
        // For now, we'll just show a message
        println!("Exporting niche to BYOB workflow...");
    }

    /// Enhanced visual editor with better UX
    fn render_enhanced_visual_editor(&mut self, ui: &mut egui::Ui) {
        ui.heading("🎨 Visual Niche Builder");
        ui.label("Build your niche visually with guided forms:");
        ui.add_space(10.0);
        
        // Metadata section
        ui.collapsing("📋 Metadata", |ui| {
            ui.horizontal(|ui| {
                ui.label("Name:");
                ui.text_edit_singleline(&mut self.niche_manifest.metadata.name);
            });
            
            ui.horizontal(|ui| {
                ui.label("Version:");
                ui.text_edit_singleline(&mut self.niche_manifest.metadata.version);
            });
            
            ui.horizontal(|ui| {
                ui.label("Author:");
                ui.text_edit_singleline(&mut self.niche_manifest.metadata.author);
            });
            
            ui.label("Description:");
            ui.text_edit_multiline(&mut self.niche_manifest.metadata.description);
        });
        
        ui.add_space(10.0);
        
        // Services section
        ui.collapsing("🔧 Services", |ui| {
            ui.label("Define the services in your niche:");
            ui.add_space(5.0);
            
            let service_count = self.niche_manifest.services.len();
            for i in 0..service_count {
                ui.group(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Name:");
                        ui.text_edit_singleline(&mut self.niche_manifest.services[i].name);
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Primal:");
                        egui::ComboBox::from_label("")
                            .selected_text(&self.niche_manifest.services[i].primal)
                            .show_ui(ui, |ui| {
                                ui.selectable_value(&mut self.niche_manifest.services[i].primal, "toadstool".to_string(), "toadstool");
                                ui.selectable_value(&mut self.niche_manifest.services[i].primal, "songbird".to_string(), "songbird");
                                ui.selectable_value(&mut self.niche_manifest.services[i].primal, "nestgate".to_string(), "nestgate");
                                ui.selectable_value(&mut self.niche_manifest.services[i].primal, "squirrel".to_string(), "squirrel");
                                ui.selectable_value(&mut self.niche_manifest.services[i].primal, "beardog".to_string(), "beardog");
                            });
                    });
                    
                    if let Some(ref mut image) = self.niche_manifest.services[i].image {
                        ui.horizontal(|ui| {
                            ui.label("Image:");
                            ui.text_edit_singleline(image);
                        });
                    }
                    
                    ui.horizontal(|ui| {
                        ui.label("CPU:");
                        ui.add(egui::Slider::new(&mut self.niche_manifest.services[i].resources.cpu, 0.1..=16.0).suffix(" cores"));
                    });
                    
                    ui.horizontal(|ui| {
                        ui.label("Memory:");
                        ui.add(egui::Slider::new(&mut self.niche_manifest.services[i].resources.memory_gb, 0.5..=64.0).suffix(" GB"));
                    });
                });
                
                ui.add_space(5.0);
            }
            
            if ui.button("➕ Add Service").clicked() {
                self.niche_manifest.services.push(ServiceDefinition {
                    name: "new-service".to_string(),
                    primal: "toadstool".to_string(),
                    runtime: "container".to_string(),
                    image: Some("nginx:alpine".to_string()),
                    command: vec![],
                    environment: HashMap::new(),
                    ports: vec!["80:80".to_string()],
                    volumes: vec![],
                    resources: ServiceResources {
                        cpu: 1.0,
                        memory_gb: 1.0,
                        storage_gb: 10.0,
                        gpu: None,
                    },
                });
            }
        });
        
        ui.add_space(15.0);
        
        ui.horizontal(|ui| {
            if ui.button("🔄 Generate YAML").clicked() {
                self.generate_yaml_from_manifest();
                self.editor_mode = NicheEditorMode::YAML;
            }
            
            if ui.button("👁️ Preview").clicked() {
                self.editor_mode = NicheEditorMode::Preview;
            }
        });
    }

    /// Generate YAML from the visual manifest
    fn generate_yaml_from_manifest(&mut self) {
        // This would generate YAML from the manifest structure
        // For now, we'll use a simple template
        self.niche_yaml = format!(r#"# Generated Niche Package
apiVersion: biomeOS/v1
kind: Niche
metadata:
  name: "{}"
  version: "{}"
  description: "{}"
  author: "{}"
  
services:
"#, 
            self.niche_manifest.metadata.name,
            self.niche_manifest.metadata.version,
            self.niche_manifest.metadata.description,
            self.niche_manifest.metadata.author
        );
        
        for service in &self.niche_manifest.services {
            self.niche_yaml.push_str(&format!(r#"  {}:
    primal: {}
    runtime: {}
    image: "{}"
    resources:
      cpu: {}
      memory: {}GB
      
"#,
                service.name,
                service.primal,
                service.runtime,
                service.image.as_deref().unwrap_or("nginx:alpine"),
                service.resources.cpu,
                service.resources.memory_gb
            ));
        }
    }

    fn render_enhanced_yaml_editor(&mut self, ui: &mut egui::Ui) {
        self.base.render_card(ui, "📝 YAML Editor", |ui| {
            ui.label("Edit the niche YAML directly:");
            ui.add_space(5.0);
            
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    ui.add(egui::TextEdit::multiline(&mut self.niche_yaml)
                        .code_editor()
                        .desired_rows(20)
                        .desired_width(f32::INFINITY));
                });
            
            ui.add_space(10.0);
            ui.horizontal(|ui| {
                if ui.button("✅ Validate").clicked() {
                    // Validate YAML
                    self.validation_errors.clear();
                    // Mock validation
                    if self.niche_yaml.contains("invalid") {
                        self.validation_errors.push("Invalid YAML syntax".to_string());
                    }
                }
                if ui.button("💾 Save").clicked() {
                    // Save niche
                }
                if ui.button("🔄 Reset").clicked() {
                    self.niche_yaml = Self::default_niche_yaml();
                }
            });
        });

        // Validation results
        if !self.validation_errors.is_empty() {
            ui.add_space(10.0);
            self.base.render_card(ui, "❌ Validation Errors", |ui| {
                for error in &self.validation_errors {
                    ui.colored_label(egui::Color32::RED, format!("• {}", error));
                }
            });
        }
    }

    fn render_enhanced_preview_mode(&mut self, ui: &mut egui::Ui) {
        self.base.render_card(ui, "👁️ Niche Preview", |ui| {
            ui.heading(&self.niche_manifest.metadata.name);
            ui.label(format!("Version: {}", self.niche_manifest.metadata.version));
            ui.label(format!("Author: {}", self.niche_manifest.metadata.author));
            ui.add_space(5.0);
            ui.label(&self.niche_manifest.metadata.description);
            
            ui.add_space(10.0);
            ui.separator();
            ui.heading("Services");
            
            for service in &self.niche_manifest.services {
                ui.horizontal(|ui| {
                    ui.label(format!("• {} ({})", service.name, service.primal));
                    if let Some(ref image) = service.image {
                        ui.label(format!("- {}", image));
                    }
                });
            }
            
            ui.add_space(10.0);
            ui.separator();
            ui.heading("Resource Requirements");
            ui.label(format!("CPU: {:.1} cores", self.niche_manifest.resources.total_cpu));
            ui.label(format!("Memory: {:.1} GB", self.niche_manifest.resources.total_memory_gb));
            ui.label(format!("Storage: {:.1} GB", self.niche_manifest.resources.total_storage_gb));
        });
    }

    fn render_enhanced_template_wizard(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("Enhanced Template Wizard")
            .collapsible(false)
            .resizable(true)
            .default_size([700.0, 500.0])
            .show(ui.ctx(), |ui| {
                ui.heading("Choose a Template");
                ui.separator();
                
                // Template categories
                ui.horizontal(|ui| {
                    ui.label("Category:");
                    egui::ComboBox::from_label("")
                        .selected_text("All")
                        .show_ui(ui, |ui| {
                            ui.selectable_value(&mut (), (), "All");
                            ui.selectable_value(&mut (), (), "Web Development");
                            ui.selectable_value(&mut (), (), "AI/ML");
                            ui.selectable_value(&mut (), (), "Gaming");
                            ui.selectable_value(&mut (), (), "Data");
                        });
                });
                
                ui.add_space(10.0);
                
                egui::ScrollArea::vertical()
                    .max_height(350.0)
                    .show(ui, |ui| {
                        for template in &self.templates {
                            ui.group(|ui| {
                                ui.heading(&template.name);
                                ui.label(&template.description);
                                ui.add_space(5.0);
                                ui.horizontal(|ui| {
                                    ui.label(format!("Category: {:?}", template.category));
                                    ui.separator();
                                    ui.label(format!("Difficulty: {:?}", template.difficulty));
                                });
                                ui.add_space(5.0);
                                if ui.button("Use This Template").clicked() {
                                    self.niche_yaml = template.template_yaml.clone();
                                    self.show_template_wizard = false;
                                }
                            });
                            ui.add_space(5.0);
                        }
                    });
                
                ui.add_space(10.0);
                if ui.button("Cancel").clicked() {
                    self.show_template_wizard = false;
                }
            });
    }

    // Template methods for different niche types
    
    fn get_react_template(&self) -> String {
        r#"# React Web Application Niche
apiVersion: biomeOS/v1
kind: Niche
metadata:
  name: "react-webapp"
  version: "1.0.0"
  description: "React web application with modern tooling"
  author: "Your Name"
  
services:
  frontend:
    primal: toadstool
    runtime: container
    image: "node:18-alpine"
    command: ["npm", "start"]
    ports:
      - "3000:3000"
    environment:
      NODE_ENV: "development"
      REACT_APP_API_URL: "http://localhost:8080"
    resources:
      cpu: 2.0
      memory: 4GB
      storage: 20GB
      
  backend:
    primal: toadstool
    runtime: container
    image: "node:18-alpine"
    command: ["npm", "run", "server"]
    ports:
      - "8080:8080"
    resources:
      cpu: 1.0
      memory: 2GB
      storage: 10GB
      
networking:
  load_balancing: true
  service_discovery: true
"#.to_string()
    }

    fn get_pytorch_template(&self) -> String {
        r#"# PyTorch AI Research Niche
apiVersion: biomeOS/v1
kind: Niche
metadata:
  name: "pytorch-research"
  version: "1.0.0"
  description: "PyTorch research environment with GPU support"
  author: "Your Name"
  
services:
  jupyter:
    primal: toadstool
    runtime: container
    image: "pytorch/pytorch:latest"
    command: ["jupyter", "lab", "--allow-root", "--ip=0.0.0.0"]
    ports:
      - "8888:8888"
    environment:
      JUPYTER_ENABLE_LAB: "yes"
    resources:
      cpu: 8.0
      memory: 32GB
      storage: 100GB
      gpu: 2
      
  tensorboard:
    primal: toadstool
    runtime: container
    image: "tensorflow/tensorflow:latest"
    command: ["tensorboard", "--logdir=/logs", "--host=0.0.0.0"]
    ports:
      - "6006:6006"
    resources:
      cpu: 2.0
      memory: 4GB
      storage: 50GB
      
  data-storage:
    primal: nestgate
    runtime: container
    image: "minio/minio:latest"
    command: ["server", "/data"]
    ports:
      - "9000:9000"
    environment:
      MINIO_ROOT_USER: "admin"
      MINIO_ROOT_PASSWORD: "password"
    resources:
      cpu: 1.0
      memory: 2GB
      storage: 1TB
      
networking:
  high_bandwidth: true
  service_discovery: true
"#.to_string()
    }

    fn get_game_server_template(&self) -> String {
        r#"# Game Server Niche
apiVersion: biomeOS/v1
kind: Niche
metadata:
  name: "game-server"
  version: "1.0.0"
  description: "Dedicated game server with matchmaking"
  author: "Your Name"
  
services:
  game-server:
    primal: toadstool
    runtime: container
    image: "gameserver:latest"
    ports:
      - "7777:7777/udp"
      - "7778:7778/tcp"
    environment:
      MAX_PLAYERS: "64"
      TICK_RATE: "128"
      MAP_ROTATION: "true"
    resources:
      cpu: 8.0
      memory: 16GB
      storage: 50GB
      
  matchmaking:
    primal: songbird
    runtime: container
    image: "matchmaker:latest"
    ports:
      - "8080:8080"
    environment:
      REDIS_URL: "redis://redis:6379"
    resources:
      cpu: 2.0
      memory: 4GB
      storage: 10GB
      
  redis:
    primal: nestgate
    runtime: container
    image: "redis:7-alpine"
    ports:
      - "6379:6379"
    resources:
      cpu: 1.0
      memory: 2GB
      storage: 5GB
      
networking:
  low_latency: true
  anti_cheat: true
"#.to_string()
    }

    fn get_data_pipeline_template(&self) -> String {
        r#"# Data Pipeline Niche
apiVersion: biomeOS/v1
kind: Niche
metadata:
  name: "data-pipeline"
  version: "1.0.0"
  description: "ETL data pipeline with analytics"
  author: "Your Name"
  
services:
  airflow:
    primal: toadstool
    runtime: container
    image: "apache/airflow:latest"
    ports:
      - "8080:8080"
    environment:
      AIRFLOW__CORE__EXECUTOR: "LocalExecutor"
      AIRFLOW__DATABASE__SQL_ALCHEMY_CONN: "postgresql://airflow:airflow@postgres:5432/airflow"
    resources:
      cpu: 4.0
      memory: 8GB
      storage: 20GB
      
  postgres:
    primal: nestgate
    runtime: container
    image: "postgres:15"
    ports:
      - "5432:5432"
    environment:
      POSTGRES_USER: "airflow"
      POSTGRES_PASSWORD: "airflow"
      POSTGRES_DB: "airflow"
    resources:
      cpu: 2.0
      memory: 4GB
      storage: 100GB
      
  spark:
    primal: toadstool
    runtime: container
    image: "bitnami/spark:latest"
    ports:
      - "8081:8080"
    environment:
      SPARK_MODE: "master"
    resources:
      cpu: 8.0
      memory: 32GB
      storage: 200GB
      
networking:
  high_bandwidth: true
  service_discovery: true
"#.to_string()
    }

    // Additional template methods for other niche types
    fn get_nodejs_api_template(&self) -> String {
        r#"# Node.js API Niche
apiVersion: biomeOS/v1
kind: Niche
metadata:
  name: "nodejs-api"
  version: "1.0.0"
  description: "Node.js REST API with database"
  author: "Your Name"
  
services:
  api:
    primal: toadstool
    runtime: container
    image: "node:18-alpine"
    command: ["npm", "start"]
    ports:
      - "3000:3000"
    environment:
      NODE_ENV: "production"
      DATABASE_URL: "postgresql://api:password@postgres:5432/api"
    resources:
      cpu: 2.0
      memory: 4GB
      storage: 20GB
      
  postgres:
    primal: nestgate
    runtime: container
    image: "postgres:15"
    ports:
      - "5432:5432"
    environment:
      POSTGRES_USER: "api"
      POSTGRES_PASSWORD: "password"
      POSTGRES_DB: "api"
    resources:
      cpu: 1.0
      memory: 2GB
      storage: 50GB
      
networking:
  load_balancing: true
  service_discovery: true
"#.to_string()
    }

    fn get_fullstack_template(&self) -> String {
        r#"# Full Stack Application Niche
apiVersion: biomeOS/v1
kind: Niche
metadata:
  name: "fullstack-app"
  version: "1.0.0"
  description: "Complete full-stack application with frontend, backend, and database"
  author: "Your Name"
  
services:
  frontend:
    primal: toadstool
    runtime: container
    image: "node:18-alpine"
    command: ["npm", "start"]
    ports:
      - "3000:3000"
    environment:
      REACT_APP_API_URL: "http://localhost:8080"
    resources:
      cpu: 2.0
      memory: 4GB
      storage: 20GB
      
  backend:
    primal: toadstool
    runtime: container
    image: "node:18-alpine"
    command: ["npm", "run", "server"]
    ports:
      - "8080:8080"
    environment:
      DATABASE_URL: "postgresql://app:password@postgres:5432/app"
      REDIS_URL: "redis://redis:6379"
    resources:
      cpu: 2.0
      memory: 4GB
      storage: 20GB
      
  postgres:
    primal: nestgate
    runtime: container
    image: "postgres:15"
    ports:
      - "5432:5432"
    environment:
      POSTGRES_USER: "app"
      POSTGRES_PASSWORD: "password"
      POSTGRES_DB: "app"
    resources:
      cpu: 2.0
      memory: 4GB
      storage: 100GB
      
  redis:
    primal: nestgate
    runtime: container
    image: "redis:7-alpine"
    ports:
      - "6379:6379"
    resources:
      cpu: 1.0
      memory: 1GB
      storage: 5GB
      
networking:
  load_balancing: true
  service_discovery: true
"#.to_string()
    }

    fn get_jupyter_template(&self) -> String {
        r#"# Jupyter Lab Research Niche
apiVersion: biomeOS/v1
kind: Niche
metadata:
  name: "jupyter-lab"
  version: "1.0.0"
  description: "Jupyter Lab environment for data science"
  author: "Your Name"
  
services:
  jupyter:
    primal: toadstool
    runtime: container
    image: "jupyter/datascience-notebook:latest"
    command: ["start-notebook.sh", "--NotebookApp.token=''", "--NotebookApp.password=''"]
    ports:
      - "8888:8888"
    environment:
      JUPYTER_ENABLE_LAB: "yes"
    resources:
      cpu: 4.0
      memory: 16GB
      storage: 100GB
      
networking:
  service_discovery: true
"#.to_string()
    }

    fn get_ml_training_template(&self) -> String {
        r#"# ML Training Niche
apiVersion: biomeOS/v1
kind: Niche
metadata:
  name: "ml-training"
  version: "1.0.0"
  description: "Machine learning training environment"
  author: "Your Name"
  
services:
  training:
    primal: toadstool
    runtime: container
    image: "tensorflow/tensorflow:latest-gpu"
    command: ["python", "train.py"]
    environment:
      TF_CPP_MIN_LOG_LEVEL: "2"
      CUDA_VISIBLE_DEVICES: "0,1"
    resources:
      cpu: 16.0
      memory: 64GB
      storage: 500GB
      gpu: 2
      
  mlflow:
    primal: toadstool
    runtime: container
    image: "mlflow/mlflow:latest"
    command: ["mlflow", "server", "--host", "0.0.0.0"]
    ports:
      - "5000:5000"
    resources:
      cpu: 2.0
      memory: 4GB
      storage: 50GB
      
networking:
  high_bandwidth: true
  service_discovery: true
"#.to_string()
    }

    fn get_tournament_template(&self) -> String {
        r#"# Tournament Platform Niche
apiVersion: biomeOS/v1
kind: Niche
metadata:
  name: "tournament-platform"
  version: "1.0.0"
  description: "Complete tournament management platform"
  author: "Your Name"
  
services:
  tournament-manager:
    primal: songbird
    runtime: container
    image: "tournament/manager:latest"
    ports:
      - "8080:8080"
    environment:
      MAX_TOURNAMENTS: "10"
      BRACKET_TYPE: "double_elimination"
    resources:
      cpu: 4.0
      memory: 8GB
      storage: 50GB
      
  leaderboard:
    primal: nestgate
    runtime: container
    image: "tournament/leaderboard:latest"
    ports:
      - "8081:8080"
    environment:
      REDIS_URL: "redis://redis:6379"
    resources:
      cpu: 2.0
      memory: 4GB
      storage: 20GB
      
  redis:
    primal: nestgate
    runtime: container
    image: "redis:7-alpine"
    ports:
      - "6379:6379"
    resources:
      cpu: 1.0
      memory: 2GB
      storage: 10GB
      
networking:
  low_latency: true
  service_discovery: true
"#.to_string()
    }

    fn get_matchmaking_template(&self) -> String {
        r#"# Matchmaking Service Niche
apiVersion: biomeOS/v1
kind: Niche
metadata:
  name: "matchmaking-service"
  version: "1.0.0"
  description: "Dedicated matchmaking service"
  author: "Your Name"
  
services:
  matchmaker:
    primal: songbird
    runtime: container
    image: "matchmaking/service:latest"
    ports:
      - "8080:8080"
    environment:
      ALGORITHM: "skill_based"
      MAX_WAIT_TIME: "30"
    resources:
      cpu: 4.0
      memory: 8GB
      storage: 20GB
      
networking:
  low_latency: true
  service_discovery: true
"#.to_string()
    }

    fn get_analytics_template(&self) -> String {
        r#"# Analytics Dashboard Niche
apiVersion: biomeOS/v1
kind: Niche
metadata:
  name: "analytics-dashboard"
  version: "1.0.0"
  description: "Real-time analytics dashboard"
  author: "Your Name"
  
services:
  dashboard:
    primal: toadstool
    runtime: container
    image: "grafana/grafana:latest"
    ports:
      - "3000:3000"
    environment:
      GF_SECURITY_ADMIN_PASSWORD: "admin"
    resources:
      cpu: 2.0
      memory: 4GB
      storage: 20GB
      
  prometheus:
    primal: toadstool
    runtime: container
    image: "prom/prometheus:latest"
    ports:
      - "9090:9090"
    resources:
      cpu: 2.0
      memory: 4GB
      storage: 100GB
      
networking:
  service_discovery: true
"#.to_string()
    }

    fn get_etl_template(&self) -> String {
        r#"# ETL Workflow Niche
apiVersion: biomeOS/v1
kind: Niche
metadata:
  name: "etl-workflow"
  version: "1.0.0"
  description: "Extract, Transform, Load workflow"
  author: "Your Name"
  
services:
  etl-runner:
    primal: toadstool
    runtime: container
    image: "apache/airflow:latest"
    ports:
      - "8080:8080"
    environment:
      AIRFLOW__CORE__EXECUTOR: "LocalExecutor"
    resources:
      cpu: 4.0
      memory: 8GB
      storage: 50GB
      
  data-warehouse:
    primal: nestgate
    runtime: container
    image: "postgres:15"
    ports:
      - "5432:5432"
    environment:
      POSTGRES_USER: "etl"
      POSTGRES_PASSWORD: "password"
      POSTGRES_DB: "warehouse"
    resources:
      cpu: 4.0
      memory: 16GB
      storage: 1TB
      
networking:
  high_bandwidth: true
  service_discovery: true
"#.to_string()
    }
}

impl View for NicheManagerView {
    fn render(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        // Render the hierarchical workflow header
        self.render_niche_workflow(ui);
        
        // Tab bar
        ui.horizontal(|ui| {
            if ui.selectable_label(self.selected_tab == NicheManagerTab::Browse, "📚 Browse").clicked() {
                self.selected_tab = NicheManagerTab::Browse;
            }
            if ui.selectable_label(self.selected_tab == NicheManagerTab::Create, "➕ Create").clicked() {
                self.selected_tab = NicheManagerTab::Create;
            }
            if ui.selectable_label(self.selected_tab == NicheManagerTab::Edit, "✏️ Edit").clicked() {
                self.selected_tab = NicheManagerTab::Edit;
            }
            if ui.selectable_label(self.selected_tab == NicheManagerTab::Test, "🧪 Test").clicked() {
                self.selected_tab = NicheManagerTab::Test;
            }
            if ui.selectable_label(self.selected_tab == NicheManagerTab::Marketplace, "🏪 Marketplace").clicked() {
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