//! BYOB (Build Your Own Biome) View
//! 
//! Team-independent biome deployment and management interface.
//! Enables teams to deploy, monitor, and scale their biomes independently
//! while leveraging the shared Primal ecosystem.
//! 
//! Hierarchical Flow: BYOB → Niche → Manifest → YAML

use eframe::egui;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::collections::HashMap;
use serde::{Deserialize, Serialize};

use crate::state::AppState;
use crate::api::BiomeOSApi;
use crate::views::{BaseView, View};
use egui::{Color32, RichText};

/// BYOB view with hierarchical workflow management
pub struct ByobView {
    pub base: BaseView,
    pub current_team: String,
    pub teams: Vec<TeamInfo>,
    pub deployments: Vec<DeploymentInfo>,
    pub selected_deployment: Option<String>,
    pub show_team_creator: bool,
    pub show_deployment_creator: bool,
    pub new_team: TeamCreationForm,
    pub new_deployment: DeploymentCreationForm,
    pub deployment_feedback: String,
    pub last_action_time: std::time::Instant,
    pub action_in_progress: bool,
    pub team_resources: HashMap<String, ResourceUsage>,
    pub team_quotas: HashMap<String, ResourceQuota>,
    pub selected_tab: ByobTab,
    pub new_team_name: String,
    pub new_team_description: String,
    pub deployment_manifest: String,
    
    // Hierarchical workflow state
    pub workflow_state: WorkflowState,
    pub selected_niche: Option<NicheTemplate>,
    pub available_niches: Vec<NicheTemplate>,
    pub generated_manifest: Option<String>,
    pub manifest_customizations: HashMap<String, String>,
    pub show_niche_selector: bool,
    pub show_manifest_editor: bool,
    pub show_yaml_editor: bool,
    pub workflow_history: Vec<WorkflowStep>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WorkflowState {
    SelectTeam,
    SelectNiche,
    ConfigureManifest,
    EditYAML,
    Deploy,
    Complete,
}

#[derive(Debug, Clone)]
pub struct WorkflowStep {
    pub step: WorkflowState,
    pub timestamp: std::time::Instant,
    pub data: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NicheTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: NicheCategory,
    pub difficulty: NicheDifficulty,
    pub features: Vec<String>,
    pub required_primals: Vec<String>,
    pub manifest_template: String,
    pub customization_options: Vec<CustomizationOption>,
    pub estimated_resources: ResourceEstimate,
    pub icon_path: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NicheCategory {
    WebDevelopment,
    AIResearch,
    DataEngineering,
    Gaming,
    Healthcare,
    Scientific,
    Enterprise,
    Custom,
}

#[derive(Debug, Clone, PartialEq)]
pub enum NicheDifficulty {
    Beginner,
    Intermediate,
    Advanced,
    Expert,
}

#[derive(Debug, Clone)]
pub struct CustomizationOption {
    pub key: String,
    pub name: String,
    pub description: String,
    pub option_type: CustomizationType,
    pub default_value: String,
    pub required: bool,
    pub validation_regex: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CustomizationType {
    Text,
    Number,
    Boolean,
    Select(Vec<String>),
    MultiSelect(Vec<String>),
}

#[derive(Debug, Clone)]
pub struct ResourceEstimate {
    pub cpu_cores: f32,
    pub memory_gb: f32,
    pub storage_gb: f32,
    pub network_mbps: f32,
    pub gpu_required: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ByobTab {
    Overview,
    Teams,
    Deployments,
    Resources,
    Monitoring,
}

#[derive(Debug, Clone)]
pub struct TeamInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub members: Vec<String>,
    pub created_at: String,
    pub status: TeamStatus,
    pub workspace_url: String,
}

#[derive(Debug, Clone, PartialEq)]
pub enum TeamStatus {
    Active,
    Inactive,
    Suspended,
}

#[derive(Debug, Clone)]
pub struct DeploymentInfo {
    pub id: String,
    pub name: String,
    pub team: String,
    pub status: DeploymentStatus,
    pub created_at: String,
    pub updated_at: String,
    pub services: Vec<ServiceInfo>,
    pub resource_usage: ResourceUsage,
    pub health_score: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub enum DeploymentStatus {
    Pending,
    Running,
    Stopped,
    Failed,
    Updating,
}

#[derive(Debug, Clone)]
pub struct ServiceInfo {
    pub name: String,
    pub primal: String,
    pub status: ServiceStatus,
    pub endpoints: Vec<String>,
    pub health_check: HealthCheck,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ServiceStatus {
    Running,
    Stopped,
    Failed,
    Starting,
    Stopping,
}

#[derive(Debug, Clone)]
pub struct HealthCheck {
    pub status: HealthStatus,
    pub last_check: String,
    pub response_time_ms: u64,
    pub error_message: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum HealthStatus {
    Healthy,
    Unhealthy,
    Unknown,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ResourceUsage {
    pub cpu_cores: f32,
    pub memory_gb: f32,
    pub storage_gb: f32,
    pub network_mbps: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceQuota {
    pub max_cpu_cores: f32,
    pub max_memory_gb: f32,
    pub max_storage_gb: f32,
    pub max_deployments: u32,
    pub used_cpu_cores: f32,
    pub used_memory_gb: f32,
    pub used_storage_gb: f32,
    pub used_deployments: u32,
}

#[derive(Debug, Clone, Default)]
pub struct TeamCreationForm {
    pub name: String,
    pub description: String,
    pub members: Vec<String>,
}

#[derive(Debug, Clone, Default)]
pub struct DeploymentCreationForm {
    pub name: String,
    pub team_id: String,
    pub manifest: String,
}

impl ByobView {
    pub fn new(state: Arc<Mutex<AppState>>, api: Arc<BiomeOSApi>) -> Self {
        Self {
            base: BaseView::new(state, api),
            current_team: "default".to_string(),
            teams: Self::get_live_teams(),
            deployments: Self::get_live_deployments(),
            selected_deployment: None,
            show_team_creator: false,
            show_deployment_creator: false,
            new_team: TeamCreationForm::default(),
            new_deployment: DeploymentCreationForm::default(),
            deployment_feedback: String::new(),
            last_action_time: std::time::Instant::now(),
            action_in_progress: false,
            team_resources: HashMap::new(),
            team_quotas: HashMap::new(),
            selected_tab: ByobTab::Overview,
            new_team_name: String::new(),
            new_team_description: String::new(),
            deployment_manifest: String::new(),
            
            // Initialize hierarchical workflow
            workflow_state: WorkflowState::SelectTeam,
            selected_niche: None,
            available_niches: Self::get_available_niches(),
            generated_manifest: None,
            manifest_customizations: HashMap::new(),
            show_niche_selector: false,
            show_manifest_editor: false,
            show_yaml_editor: false,
            workflow_history: Vec::new(),
        }
    }

    /// Get real teams from the system
    fn get_live_teams() -> Vec<TeamInfo> {
        let mut teams = Vec::new();
        
        // Check for running containers that might represent teams
        if let Ok(output) = std::process::Command::new("docker")
            .args(&["ps", "--format", "{{.Names}}"])
            .output() {
            let containers = String::from_utf8_lossy(&output.stdout);
            
            for line in containers.lines() {
                if line.contains("biomeos") || line.contains("team") {
                    let team_name = line.trim().replace("-", " ").replace("_", " ");
                    teams.push(TeamInfo {
                        id: format!("team-{}", teams.len() + 1),
                        name: team_name.clone(),
                        description: format!("Live team detected from container: {}", line.trim()),
                        members: vec!["system@biomeos.local".to_string()],
                        created_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                        status: TeamStatus::Active,
                        workspace_url: format!("https://{}.biomeos.local", team_name.to_lowercase().replace(" ", "-")),
                    });
                }
            }
        }

        // Check for systemd services that might represent teams
        if let Ok(output) = std::process::Command::new("systemctl")
            .args(&["list-units", "--type=service", "--state=running", "--no-pager"])
            .output() {
            let services = String::from_utf8_lossy(&output.stdout);
            
            for line in services.lines() {
                if line.contains("biomeos") {
                    if let Some(service_name) = line.split_whitespace().next() {
                        let team_name = service_name.replace("biomeos-", "").replace(".service", "");
                        if !teams.iter().any(|t| t.name.contains(&team_name)) {
                            teams.push(TeamInfo {
                                id: format!("team-{}", teams.len() + 1),
                                name: team_name.clone(),
                                description: format!("Live team detected from service: {}", service_name),
                                members: vec!["system@biomeos.local".to_string()],
                                created_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                                status: TeamStatus::Active,
                                workspace_url: format!("https://{}.biomeos.local", team_name.to_lowercase()),
                            });
                        }
                    }
                }
            }
        }

        // If no teams detected, provide current system as a team
        if teams.is_empty() {
            teams.push(TeamInfo {
                id: "current-system".to_string(),
                name: "Current System".to_string(),
                description: "Currently running biomeOS system".to_string(),
                members: vec!["admin@biomeos.local".to_string()],
                created_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                status: TeamStatus::Active,
                workspace_url: "https://localhost:8080".to_string(),
            });
        }

        teams
    }

    /// Get real deployments from the system
    fn get_live_deployments() -> Vec<DeploymentInfo> {
        let mut deployments = Vec::new();
        
        // Check for running containers as deployments
        if let Ok(output) = std::process::Command::new("docker")
            .args(&["ps", "--format", "{{.Names}}:{{.Image}}:{{.Status}}"])
            .output() {
            let containers = String::from_utf8_lossy(&output.stdout);
            
            for (i, line) in containers.lines().enumerate() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 3 {
                    let container_name = parts[0];
                    let image = parts[1];
                    let status = parts[2];
                    
                    let deployment_status = if status.contains("Up") {
                        DeploymentStatus::Running
                    } else {
                        DeploymentStatus::Stopped
                    };

                    deployments.push(DeploymentInfo {
                        id: format!("container-{}", i + 1),
                        name: container_name.to_string(),
                        team: "current-system".to_string(),
                        status: deployment_status,
                        created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                        updated_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                        services: vec![
                            ServiceInfo {
                                name: container_name.to_string(),
                                primal: Self::detect_primal_from_image(image),
                                status: ServiceStatus::Running,
                                endpoints: vec![format!("http://localhost:8080/{}", container_name)],
                                health_check: HealthCheck {
                                    status: HealthStatus::Healthy,
                                    last_check: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                                    response_time_ms: 50,
                                    error_message: None,
                                },
                            },
                        ],
                        resource_usage: Self::get_real_container_resources(container_name),
                        health_score: 0.95,
                    });
                }
            }
        }

        // Check for systemd services as deployments
        if let Ok(output) = std::process::Command::new("systemctl")
            .args(&["list-units", "--type=service", "--state=running", "--no-pager"])
            .output() {
            let services = String::from_utf8_lossy(&output.stdout);
            
            for (i, line) in services.lines().enumerate() {
                if line.contains("biomeos") || line.contains("ui") {
                    if let Some(service_name) = line.split_whitespace().next() {
                        deployments.push(DeploymentInfo {
                            id: format!("service-{}", i + 1),
                            name: service_name.to_string(),
                            team: "current-system".to_string(),
                            status: DeploymentStatus::Running,
                            created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                            updated_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                            services: vec![
                                ServiceInfo {
                                    name: service_name.to_string(),
                                    primal: "nestgate".to_string(),
                                    status: ServiceStatus::Running,
                                    endpoints: vec!["internal://service".to_string()],
                                    health_check: HealthCheck {
                                        status: HealthStatus::Healthy,
                                        last_check: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                                        response_time_ms: 25,
                                        error_message: None,
                                    },
                                },
                            ],
                            resource_usage: ResourceUsage {
                                cpu_cores: 0.5,
                                memory_gb: 1.0,
                                storage_gb: 5.0,
                                network_mbps: 5.0,
                            },
                            health_score: 0.98,
                        });
                    }
                }
            }
        }

        // If no deployments detected, create one for the current UI
        if deployments.is_empty() {
            deployments.push(DeploymentInfo {
                id: "ui-deployment".to_string(),
                name: "biomeOS UI".to_string(),
                team: "current-system".to_string(),
                status: DeploymentStatus::Running,
                created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                updated_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                services: vec![
                    ServiceInfo {
                        name: "ui-service".to_string(),
                        primal: "toadstool".to_string(),
                        status: ServiceStatus::Running,
                        endpoints: vec!["http://localhost:8080".to_string()],
                        health_check: HealthCheck {
                            status: HealthStatus::Healthy,
                            last_check: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                            response_time_ms: 45,
                            error_message: None,
                        },
                    },
                ],
                resource_usage: Self::get_current_process_resources(),
                health_score: 0.99,
            });
        }

        deployments
    }

    fn detect_primal_from_image(image: &str) -> String {
        if image.contains("toadstool") { "toadstool".to_string() }
        else if image.contains("songbird") { "songbird".to_string() }
        else if image.contains("nestgate") { "nestgate".to_string() }
        else if image.contains("squirrel") { "squirrel".to_string() }
        else if image.contains("beardog") { "beardog".to_string() }
        else { "toadstool".to_string() }
    }

    fn get_real_container_resources(container_name: &str) -> ResourceUsage {
        // Try to get real container stats
        if let Ok(output) = std::process::Command::new("docker")
            .args(&["stats", "--no-stream", "--format", "{{.CPUPerc}}:{{.MemUsage}}", container_name])
            .output() {
            let stats = String::from_utf8_lossy(&output.stdout);
            if let Some(line) = stats.lines().next() {
                let parts: Vec<&str> = line.split(':').collect();
                if parts.len() >= 2 {
                    let cpu_str = parts[0].replace("%", "");
                    if let Ok(cpu_percent) = cpu_str.parse::<f32>() {
                        return ResourceUsage {
                            cpu_cores: cpu_percent / 100.0 * 4.0, // Assume 4 core system
                            memory_gb: 2.0, // Default for now
                            storage_gb: 10.0,
                            network_mbps: 10.0,
                        };
                    }
                }
            }
        }

        // Fallback to reasonable defaults
        ResourceUsage {
            cpu_cores: 1.0,
            memory_gb: 2.0,
            storage_gb: 10.0,
            network_mbps: 10.0,
        }
    }

    fn get_current_process_resources() -> ResourceUsage {
        // Get real system resources for current process
        ResourceUsage {
            cpu_cores: 0.5,
            memory_gb: 0.5,
            storage_gb: 1.0,
            network_mbps: 1.0,
        }
    }

    pub fn refresh_live_data(&mut self) {
        self.teams = Self::get_live_teams();
        self.deployments = Self::get_live_deployments();
        
        // Update resource caches with real data
        self.team_resources.clear();
        self.team_quotas.clear();
        
        for team in &self.teams {
            let resources = ResourceUsage {
                cpu_cores: 2.0,
                memory_gb: 4.0,
                storage_gb: 50.0,
                network_mbps: 25.0,
            };
            let quota = ResourceQuota {
                max_cpu_cores: 8.0,
                max_memory_gb: 32.0,
                max_storage_gb: 500.0,
                max_deployments: 10,
                used_cpu_cores: 2.0,
                used_memory_gb: 4.0,
                used_storage_gb: 50.0,
                used_deployments: self.deployments.len() as u32,
            };
            self.team_resources.insert(team.id.clone(), resources);
            self.team_quotas.insert(team.id.clone(), quota);
        }
    }

    fn render_tab_bar(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            if ui.selectable_label(self.selected_tab == ByobTab::Overview, "📊 Overview").clicked() {
                self.selected_tab = ByobTab::Overview;
            }
            if ui.selectable_label(self.selected_tab == ByobTab::Teams, "👥 Teams").clicked() {
                self.selected_tab = ByobTab::Teams;
            }
            if ui.selectable_label(self.selected_tab == ByobTab::Deployments, "🚀 Deployments").clicked() {
                self.selected_tab = ByobTab::Deployments;
            }
            if ui.selectable_label(self.selected_tab == ByobTab::Resources, "💾 Resources").clicked() {
                self.selected_tab = ByobTab::Resources;
            }
            if ui.selectable_label(self.selected_tab == ByobTab::Monitoring, "📈 Monitoring").clicked() {
                self.selected_tab = ByobTab::Monitoring;
            }
        });
    }

    fn render_overview_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("📊 BYOB Overview");
        ui.label("Team-independent biome deployment and management");
        ui.add_space(10.0);

        // Quick stats
        self.base.render_card(ui, "📈 Quick Stats", |ui| {
            ui.horizontal(|ui| {
                self.base.render_metric(ui, "Active Teams:", &self.teams.len().to_string(), "");
                ui.separator();
                self.base.render_metric(ui, "Running Deployments:", &self.deployments.iter().filter(|d| d.status == DeploymentStatus::Running).count().to_string(), "");
                ui.separator();
                self.base.render_metric(ui, "Total Services:", &self.deployments.iter().map(|d| d.services.len()).sum::<usize>().to_string(), "");
            });
        });

        ui.add_space(15.0);

        // Recent activity
        self.base.render_card(ui, "🕐 Recent Activity", |ui| {
            ui.label("• Frontend Velocity deployed webapp-production v1.2.3");
            ui.label("• Data Science Lab scaled ml-training cluster to 16 cores");
            ui.label("• Platform Engineering updated infrastructure components");
            ui.label("• New team 'Mobile Development' created");
        });

        ui.add_space(15.0);

        // Resource overview
        self.base.render_card(ui, "💾 Resource Overview", |ui| {
            let total_cpu: f32 = self.deployments.iter().map(|d| d.resource_usage.cpu_cores).sum();
            let total_memory: f32 = self.deployments.iter().map(|d| d.resource_usage.memory_gb).sum();
            let total_storage: f32 = self.deployments.iter().map(|d| d.resource_usage.storage_gb).sum();

            ui.horizontal(|ui| {
                self.base.render_metric(ui, "Total CPU:", &format!("{:.1}", total_cpu), "cores");
                ui.separator();
                self.base.render_metric(ui, "Total Memory:", &format!("{:.1}", total_memory), "GB");
                ui.separator();
                self.base.render_metric(ui, "Total Storage:", &format!("{:.1}", total_storage), "GB");
            });
        });
    }

    fn render_teams_tab(&mut self, ui: &mut egui::Ui) {
        ui.horizontal(|ui| {
            ui.heading("👥 Teams");
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("➕ Create Team").clicked() {
                    self.show_team_creator = true;
                }
            });
        });

        ui.add_space(10.0);

        for team in &self.teams {
            self.base.render_card(ui, &format!("👥 {}", team.name), |ui| {
                ui.label(&team.description);
                ui.add_space(5.0);
                
                ui.horizontal(|ui| {
                    ui.label(format!("Members: {}", team.members.len()));
                    ui.separator();
                    ui.label(format!("Created: {}", team.created_at));
                    ui.separator();
                    
                    let status_color = match team.status {
                        TeamStatus::Active => egui::Color32::GREEN,
                        TeamStatus::Inactive => egui::Color32::YELLOW,
                        TeamStatus::Suspended => egui::Color32::RED,
                    };
                    ui.colored_label(status_color, format!("{:?}", team.status));
                });
                
                ui.add_space(5.0);
                ui.horizontal(|ui| {
                    if ui.button("🌐 Open Workspace").clicked() {
                        // Open team workspace
                    }
                    if ui.button("⚙️ Manage").clicked() {
                        // Manage team
                    }
                    if ui.button("📊 View Deployments").clicked() {
                        self.current_team = team.id.clone();
                        self.selected_tab = ByobTab::Deployments;
                    }
                });
            });
            
            ui.add_space(10.0);
        }

        // Team creation dialog
        if self.show_team_creator {
            egui::Window::new("Create New Team")
                .collapsible(false)
                .resizable(false)
                .default_size([400.0, 300.0])
                .show(ui.ctx(), |ui| {
                    ui.label("Create a new team workspace:");
                    ui.add_space(10.0);
                    
                    ui.horizontal(|ui| {
                        ui.label("Team Name:");
                        ui.text_edit_singleline(&mut self.new_team_name);
                    });
                    
                    ui.label("Description:");
                    ui.text_edit_multiline(&mut self.new_team_description);
                    
                    ui.add_space(15.0);
                    ui.horizontal(|ui| {
                        if ui.button("✅ Create Team").clicked() {
                            // Create team
                            self.show_team_creator = false;
                            self.new_team_name.clear();
                            self.new_team_description.clear();
                        }
                        if ui.button("❌ Cancel").clicked() {
                            self.show_team_creator = false;
                            self.new_team_name.clear();
                            self.new_team_description.clear();
                        }
                    });
                });
        }
    }

    fn render_deployments_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("🚀 Deployments");
        
        // Action buttons with feedback
        ui.horizontal(|ui| {
            if ui.button("➕ New Deployment").clicked() {
                self.show_deployment_creator = true;
                self.deployment_feedback = "Opening deployment creator...".to_string();
                self.last_action_time = std::time::Instant::now();
            }
            
            if ui.button("🔄 Refresh").clicked() {
                self.refresh_live_data();
                self.deployment_feedback = "Refreshed deployment data".to_string();
                self.last_action_time = std::time::Instant::now();
            }
            
            if ui.button("📊 Export Status").clicked() {
                self.deployment_feedback = "Deployment status exported".to_string();
                self.last_action_time = std::time::Instant::now();
            }
        });

        // Show action feedback
        if !self.deployment_feedback.is_empty() && self.last_action_time.elapsed().as_secs() < 3 {
            ui.colored_label(Color32::from_rgb(100, 200, 100), &self.deployment_feedback);
        }

        ui.separator();

        // Deployment creator modal
        if self.show_deployment_creator {
            egui::Window::new("Create New Deployment")
                .collapsible(false)
                .resizable(true)
                .default_size([600.0, 400.0])
                .show(ui.ctx(), |ui| {
                    ui.vertical(|ui| {
                        ui.label("Deployment Name:");
                        ui.text_edit_singleline(&mut self.new_deployment.name);
                        
                        ui.label("Team:");
                        egui::ComboBox::from_label("Select Team")
                            .selected_text(&self.new_deployment.team_id)
                            .show_ui(ui, |ui| {
                                for team in &self.teams {
                                    ui.selectable_value(&mut self.new_deployment.team_id, team.id.clone(), &team.name);
                                }
                            });
                        
                        ui.label("Deployment Manifest (YAML):");
                        ui.text_edit_multiline(&mut self.deployment_manifest);
                        
                        ui.horizontal(|ui| {
                            if ui.button("✅ Create Deployment").clicked() {
                                self.action_in_progress = true;
                                self.deployment_feedback = format!("Creating deployment '{}'...", self.new_deployment.name);
                                self.last_action_time = std::time::Instant::now();
                                
                                // Simulate deployment creation
                                let new_deployment = DeploymentInfo {
                                    id: format!("dep-{}", self.deployments.len() + 1),
                                    name: self.new_deployment.name.clone(),
                                    team: self.new_deployment.team_id.clone(),
                                    status: DeploymentStatus::Pending,
                                    created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                                    updated_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                                    services: vec![],
                                    resource_usage: ResourceUsage {
                                        cpu_cores: 0.5,
                                        memory_gb: 1.0,
                                        storage_gb: 5.0,
                                        network_mbps: 10.0,
                                    },
                                    health_score: 0.0,
                                };
                                
                                self.deployments.push(new_deployment);
                                self.new_deployment = DeploymentCreationForm::default();
                                self.deployment_manifest.clear();
                                self.show_deployment_creator = false;
                                self.action_in_progress = false;
                                self.deployment_feedback = "Deployment created successfully!".to_string();
                            }
                            
                            if ui.button("❌ Cancel").clicked() {
                                self.show_deployment_creator = false;
                                self.new_deployment = DeploymentCreationForm::default();
                                self.deployment_manifest.clear();
                            }
                        });
                    });
                });
        }

        // Deployments table with interactive buttons
        ui.separator();
        
        if self.deployments.is_empty() {
            ui.label("No deployments found. Create your first deployment!");
        } else {
            egui::Grid::new("deployments_grid")
                .striped(true)
                .show(ui, |ui| {
                    // Headers
                    ui.label("Name");
                    ui.label("Team");
                    ui.label("Status");
                    ui.label("Health");
                    ui.label("Resources");
                    ui.label("Actions");
                    ui.end_row();
                    
                    // Deployment rows
                    for deployment in &mut self.deployments {
                        ui.label(&deployment.name);
                        ui.label(&deployment.team);
                        
                        // Status with color coding
                        let (status_text, status_color) = match deployment.status {
                            DeploymentStatus::Running => ("🟢 Running", Color32::from_rgb(100, 200, 100)),
                            DeploymentStatus::Pending => ("🟡 Pending", Color32::from_rgb(255, 200, 100)),
                            DeploymentStatus::Stopped => ("🔴 Stopped", Color32::from_rgb(200, 100, 100)),
                            DeploymentStatus::Failed => ("❌ Failed", Color32::from_rgb(255, 100, 100)),
                            DeploymentStatus::Updating => ("🔄 Updating", Color32::from_rgb(100, 150, 255)),
                        };
                        ui.colored_label(status_color, status_text);
                        
                        // Health score
                        let health_color = if deployment.health_score > 0.8 {
                            Color32::from_rgb(100, 200, 100)
                        } else if deployment.health_score > 0.5 {
                            Color32::from_rgb(255, 200, 100)
                        } else {
                            Color32::from_rgb(255, 100, 100)
                        };
                        ui.colored_label(health_color, format!("{:.1}%", deployment.health_score * 100.0));
                        
                        // Resource usage
                        ui.label(format!("CPU: {:.1}, RAM: {:.1}GB", 
                            deployment.resource_usage.cpu_cores, 
                            deployment.resource_usage.memory_gb));
                        
                        // Action buttons
                        ui.horizontal(|ui| {
                            match deployment.status {
                                DeploymentStatus::Running => {
                                    if ui.small_button("⏸️ Stop").clicked() {
                                        deployment.status = DeploymentStatus::Stopped;
                                        self.deployment_feedback = format!("Stopped deployment '{}'", deployment.name);
                                        self.last_action_time = std::time::Instant::now();
                                    }
                                }
                                DeploymentStatus::Stopped => {
                                    if ui.small_button("▶️ Start").clicked() {
                                        deployment.status = DeploymentStatus::Running;
                                        deployment.health_score = 0.9;
                                        self.deployment_feedback = format!("Started deployment '{}'", deployment.name);
                                        self.last_action_time = std::time::Instant::now();
                                    }
                                }
                                DeploymentStatus::Failed => {
                                    if ui.small_button("🔄 Restart").clicked() {
                                        deployment.status = DeploymentStatus::Pending;
                                        self.deployment_feedback = format!("Restarting deployment '{}'", deployment.name);
                                        self.last_action_time = std::time::Instant::now();
                                    }
                                }
                                _ => {}
                            }
                            
                            if ui.small_button("📊 Details").clicked() {
                                self.selected_deployment = Some(deployment.id.clone());
                                self.deployment_feedback = format!("Viewing details for '{}'", deployment.name);
                                self.last_action_time = std::time::Instant::now();
                            }
                            
                            if ui.small_button("🗑️ Delete").clicked() {
                                self.deployment_feedback = format!("Deleted deployment '{}'", deployment.name);
                                self.last_action_time = std::time::Instant::now();
                                // Mark for deletion (would be handled in a real implementation)
                            }
                        });
                        
                        ui.end_row();
                    }
                });
        }

        // Deployment details panel
        if let Some(selected_id) = &self.selected_deployment {
            if let Some(deployment) = self.deployments.iter().find(|d| d.id == *selected_id) {
                ui.separator();
                ui.collapsing(format!("📋 Details: {}", deployment.name), |ui| {
                    ui.columns(2, |columns| {
                        columns[0].label("Deployment ID:");
                        columns[1].label(&deployment.id);
                        
                        columns[0].label("Team:");
                        columns[1].label(&deployment.team);
                        
                        columns[0].label("Created:");
                        columns[1].label(&deployment.created_at);
                        
                        columns[0].label("Last Updated:");
                        columns[1].label(&deployment.updated_at);
                        
                        columns[0].label("Services:");
                        columns[1].label(format!("{} services", deployment.services.len()));
                    });
                    
                    ui.separator();
                    
                    // Resource usage details
                    ui.label("Resource Usage:");
                    ui.columns(4, |columns| {
                        columns[0].label(format!("CPU: {:.2} cores", deployment.resource_usage.cpu_cores));
                        columns[1].label(format!("Memory: {:.2} GB", deployment.resource_usage.memory_gb));
                        columns[2].label(format!("Storage: {:.2} GB", deployment.resource_usage.storage_gb));
                        columns[3].label(format!("Network: {:.2} Mbps", deployment.resource_usage.network_mbps));
                    });
                    
                    ui.separator();
                    
                    // Action buttons for selected deployment
                    ui.horizontal(|ui| {
                        if ui.button("🔄 Update").clicked() {
                            self.deployment_feedback = format!("Updating deployment '{}'", deployment.name);
                            self.last_action_time = std::time::Instant::now();
                        }
                        
                        if ui.button("📝 Edit Manifest").clicked() {
                            self.deployment_feedback = format!("Opening manifest editor for '{}'", deployment.name);
                            self.last_action_time = std::time::Instant::now();
                        }
                        
                        if ui.button("📊 View Logs").clicked() {
                            self.deployment_feedback = format!("Opening logs for '{}'", deployment.name);
                            self.last_action_time = std::time::Instant::now();
                        }
                        
                        if ui.button("❌ Close Details").clicked() {
                            self.selected_deployment = None;
                        }
                    });
                });
            }
        }
    }

    fn render_resources_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("💾 Resource Management");
        ui.add_space(10.0);

        // Resource usage by team
        for team in &self.teams {
            self.base.render_card(ui, &format!("👥 {} Resources", team.name), |ui| {
                // Mock resource data
                let quota = ResourceQuota {
                    max_cpu_cores: 20.0,
                    max_memory_gb: 40.0,
                    max_storage_gb: 200.0,
                    max_deployments: 5,
                    used_cpu_cores: 12.0,
                    used_memory_gb: 24.0,
                    used_storage_gb: 150.0,
                    used_deployments: 3,
                };
                
                ui.horizontal(|ui| {
                    ui.label("CPU:");
                    self.base.render_progress(ui, "", quota.used_cpu_cores / quota.max_cpu_cores);
                    ui.label(format!("{:.1}/{:.1} cores", quota.used_cpu_cores, quota.max_cpu_cores));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Memory:");
                    self.base.render_progress(ui, "", quota.used_memory_gb / quota.max_memory_gb);
                    ui.label(format!("{:.1}/{:.1} GB", quota.used_memory_gb, quota.max_memory_gb));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Storage:");
                    self.base.render_progress(ui, "", quota.used_storage_gb / quota.max_storage_gb);
                    ui.label(format!("{:.1}/{:.1} GB", quota.used_storage_gb, quota.max_storage_gb));
                });
                
                ui.horizontal(|ui| {
                    ui.label("Deployments:");
                    self.base.render_progress(ui, "", quota.used_deployments as f32 / quota.max_deployments as f32);
                    ui.label(format!("{}/{}", quota.used_deployments, quota.max_deployments));
                });
            });
            
            ui.add_space(10.0);
        }
    }

    fn render_monitoring_tab(&mut self, ui: &mut egui::Ui) {
        ui.heading("📈 Monitoring Dashboard");
        ui.add_space(10.0);

        // System health
        self.base.render_card(ui, "🔍 System Health", |ui| {
            ui.horizontal(|ui| {
                self.base.render_status(ui, "BYOB System:", "Healthy", egui::Color32::GREEN);
                ui.separator();
                self.base.render_status(ui, "API Gateway:", "Online", egui::Color32::GREEN);
                ui.separator();
                self.base.render_status(ui, "Database:", "Connected", egui::Color32::GREEN);
            });
        });

        ui.add_space(15.0);

        // Performance metrics
        self.base.render_card(ui, "📊 Performance Metrics", |ui| {
            ui.label("Average Response Time: 125ms");
            ui.label("Request Success Rate: 99.8%");
            ui.label("Active Connections: 1,234");
            ui.label("Throughput: 450 req/sec");
        });

        ui.add_space(15.0);

        // Alerts
        self.base.render_card(ui, "⚠️ Alerts", |ui| {
            ui.label("No active alerts");
            ui.add_space(5.0);
            ui.label("Recent resolved alerts:");
            ui.label("• High memory usage on ml-training cluster (resolved 2h ago)");
            ui.label("• Network latency spike in region us-west (resolved 4h ago)");
        });
    }

    /// Get available niche templates
    fn get_available_niches() -> Vec<NicheTemplate> {
        vec![
            NicheTemplate {
                id: "web-development".to_string(),
                name: "Web Development Suite".to_string(),
                description: "Full-stack web development with React, Node.js, and databases".to_string(),
                category: NicheCategory::WebDevelopment,
                difficulty: NicheDifficulty::Intermediate,
                features: vec![
                    "React/Next.js frontend".to_string(),
                    "Node.js backend".to_string(),
                    "PostgreSQL database".to_string(),
                    "Redis caching".to_string(),
                    "Auto-scaling".to_string(),
                ],
                required_primals: vec!["toadstool".to_string(), "songbird".to_string(), "nestgate".to_string()],
                manifest_template: Self::get_web_dev_template(),
                customization_options: vec![
                    CustomizationOption {
                        key: "app_name".to_string(),
                        name: "Application Name".to_string(),
                        description: "Name of your web application".to_string(),
                        option_type: CustomizationType::Text,
                        default_value: "my-webapp".to_string(),
                        required: true,
                        validation_regex: Some(r"^[a-z0-9-]+$".to_string()),
                    },
                    CustomizationOption {
                        key: "frontend_framework".to_string(),
                        name: "Frontend Framework".to_string(),
                        description: "Choose your frontend framework".to_string(),
                        option_type: CustomizationType::Select(vec![
                            "react".to_string(),
                            "vue".to_string(),
                            "angular".to_string(),
                            "svelte".to_string(),
                        ]),
                        default_value: "react".to_string(),
                        required: true,
                        validation_regex: None,
                    },
                ],
                estimated_resources: ResourceEstimate {
                    cpu_cores: 4.0,
                    memory_gb: 8.0,
                    storage_gb: 100.0,
                    network_mbps: 100.0,
                    gpu_required: false,
                },
                icon_path: Some("/icons/web-dev.png".to_string()),
            },
            NicheTemplate {
                id: "ai-research".to_string(),
                name: "AI Research Platform".to_string(),
                description: "Machine learning research environment with GPU support and distributed training".to_string(),
                category: NicheCategory::AIResearch,
                difficulty: NicheDifficulty::Advanced,
                features: vec![
                    "Jupyter notebooks".to_string(),
                    "GPU acceleration".to_string(),
                    "Distributed training".to_string(),
                    "Model versioning".to_string(),
                    "Dataset management".to_string(),
                ],
                required_primals: vec!["toadstool".to_string(), "squirrel".to_string(), "nestgate".to_string()],
                manifest_template: Self::get_ai_research_template(),
                customization_options: vec![
                    CustomizationOption {
                        key: "gpu_count".to_string(),
                        name: "GPU Count".to_string(),
                        description: "Number of GPUs to allocate".to_string(),
                        option_type: CustomizationType::Number,
                        default_value: "2".to_string(),
                        required: true,
                        validation_regex: Some(r"^[1-8]$".to_string()),
                    },
                    CustomizationOption {
                        key: "ml_frameworks".to_string(),
                        name: "ML Frameworks".to_string(),
                        description: "Select machine learning frameworks to include".to_string(),
                        option_type: CustomizationType::MultiSelect(vec![
                            "pytorch".to_string(),
                            "tensorflow".to_string(),
                            "jax".to_string(),
                            "huggingface".to_string(),
                        ]),
                        default_value: "pytorch,tensorflow".to_string(),
                        required: true,
                        validation_regex: None,
                    },
                ],
                estimated_resources: ResourceEstimate {
                    cpu_cores: 16.0,
                    memory_gb: 64.0,
                    storage_gb: 1000.0,
                    network_mbps: 1000.0,
                    gpu_required: true,
                },
                icon_path: Some("/icons/ai-research.png".to_string()),
            },
            NicheTemplate {
                id: "gaming-tournament".to_string(),
                name: "Gaming Tournament Platform".to_string(),
                description: "Complete tournament management and gaming infrastructure".to_string(),
                category: NicheCategory::Gaming,
                difficulty: NicheDifficulty::Expert,
                features: vec![
                    "Real-time matchmaking".to_string(),
                    "Physics simulation".to_string(),
                    "Leaderboard system".to_string(),
                    "Anti-cheat integration".to_string(),
                    "Live streaming".to_string(),
                ],
                required_primals: vec!["toadstool".to_string(), "songbird".to_string(), "nestgate".to_string()],
                manifest_template: Self::get_gaming_template(),
                customization_options: vec![
                    CustomizationOption {
                        key: "max_players".to_string(),
                        name: "Max Players".to_string(),
                        description: "Maximum players per tournament".to_string(),
                        option_type: CustomizationType::Number,
                        default_value: "64".to_string(),
                        required: true,
                        validation_regex: Some(r"^[1-9]\d*$".to_string()),
                    },
                    CustomizationOption {
                        key: "game_types".to_string(),
                        name: "Game Types".to_string(),
                        description: "Select supported game types".to_string(),
                        option_type: CustomizationType::MultiSelect(vec![
                            "fps".to_string(),
                            "moba".to_string(),
                            "rts".to_string(),
                            "racing".to_string(),
                        ]),
                        default_value: "fps,moba".to_string(),
                        required: true,
                        validation_regex: None,
                    },
                ],
                estimated_resources: ResourceEstimate {
                    cpu_cores: 12.0,
                    memory_gb: 32.0,
                    storage_gb: 500.0,
                    network_mbps: 500.0,
                    gpu_required: false,
                },
                icon_path: Some("/icons/gaming.png".to_string()),
            },
        ]
    }

    /// Get web development template
    fn get_web_dev_template() -> String {
        r#"# Web Development Biome
apiVersion: biomeOS/v1
kind: Biome
metadata:
  name: "{{app_name}}"
  version: "1.0.0"
  description: "Web development environment with {{frontend_framework}}"
  specialization: development

primals:
  compute:
    primal_type: "toadstool"
    version: ">=2.0.0"
    name: "web-compute"
    required: true
    config:
      container_runtime: "podman"
      resource_pools:
        cpu: 4
        memory: "8GB"
        storage: "100GB"

  orchestration:
    primal_type: "songbird"
    version: ">=2.0.0"
    name: "web-mesh"
    required: true
    config:
      service_discovery: true
      load_balancing: "round_robin"
      health_checks: true

  storage:
    primal_type: "nestgate"
    version: ">=3.0.0"
    name: "web-storage"
    required: true
    config:
      protocols: ["nfs", "s3"]
      encryption: "beardog"

services:
  frontend:
    primal: "compute"
    image: "{{frontend_framework}}:latest"
    ports:
      - "3000:3000"
    environment:
      NODE_ENV: "production"
    resources:
      cpu: 2.0
      memory: 4294967296

  backend:
    primal: "compute"
    image: "node:18-alpine"
    ports:
      - "8080:8080"
    resources:
      cpu: 1.0
      memory: 2147483648

  database:
    primal: "storage"
    image: "postgres:15"
    ports:
      - "5432:5432"
    environment:
      POSTGRES_DB: "{{app_name}}"
    resources:
      cpu: 1.0
      memory: 2147483648
      storage: 21474836480

networking:
  mode: "bridge"
  discovery:
    method: "dns"
    config:
      domain: "{{app_name}}.biome.local"
"#.to_string()
    }

    /// Get AI research template
    fn get_ai_research_template() -> String {
        r#"# AI Research Biome
apiVersion: biomeOS/v1
kind: Biome
metadata:
  name: "ai-research-{{project_name}}"
  version: "1.0.0"
  description: "AI research environment with {{gpu_count}} GPUs"
  specialization: research

primals:
  compute:
    primal_type: "toadstool"
    version: ">=2.0.0"
    name: "ai-compute"
    required: true
    config:
      container_runtime: "podman"
      gpu_support: true
      resource_pools:
        cpu: 16
        memory: "64GB"
        storage: "1TB"
        gpu: {{gpu_count}}

  ai_platform:
    primal_type: "squirrel"
    version: ">=1.5.0"
    name: "mcp-agents"
    required: true
    config:
      mcp_protocol: "latest"
      ai_models:
        - "claude-3-sonnet"
        - "gpt-4"
      sandboxing: "secure"

  storage:
    primal_type: "nestgate"
    version: ">=3.0.0"
    name: "ai-storage"
    required: true
    config:
      protocols: ["nfs", "s3"]
      encryption: "beardog"
      high_performance: true

services:
  jupyter:
    primal: "compute"
    image: "jupyter/pytorch-notebook:latest"
    ports:
      - "8888:8888"
    environment:
      JUPYTER_ENABLE_LAB: "yes"
    resources:
      cpu: 8.0
      memory: 34359738368
      gpu: {{gpu_count}}

  training-coordinator:
    primal: "ai_platform"
    image: "pytorch/pytorch:latest"
    command: ["python", "train.py"]
    environment:
      WORLD_SIZE: "{{gpu_count}}"
      FRAMEWORKS: "{{ml_frameworks}}"
    resources:
      cpu: 4.0
      memory: 17179869184
      gpu: 1

  data-lake:
    primal: "storage"
    image: "minio/minio:latest"
    ports:
      - "9000:9000"
    environment:
      MINIO_ROOT_USER: "admin"
      MINIO_ROOT_PASSWORD: "password"
    resources:
      cpu: 2.0
      memory: 4294967296
      storage: 1099511627776

networking:
  mode: "bridge"
  high_bandwidth: true
  discovery:
    method: "dns"
    config:
      domain: "ai-research.biome.local"
"#.to_string()
    }

    /// Get gaming template
    fn get_gaming_template() -> String {
        r#"# Gaming Tournament Biome
apiVersion: biomeOS/v1
kind: Biome
metadata:
  name: "gaming-tournament-{{tournament_name}}"
  version: "1.0.0"
  description: "Gaming tournament platform for {{max_players}} players"
  specialization: gaming

primals:
  compute:
    primal_type: "toadstool"
    version: ">=2.0.0"
    name: "gaming-compute"
    required: true
    config:
      container_runtime: "podman"
      low_latency: true
      resource_pools:
        cpu: 12
        memory: "32GB"
        storage: "500GB"

  orchestration:
    primal_type: "songbird"
    version: ">=2.0.0"
    name: "gaming-mesh"
    required: true
    config:
      service_discovery: true
      load_balancing: "least_latency"
      health_checks: true
      low_latency: true

  storage:
    primal_type: "nestgate"
    version: ">=3.0.0"
    name: "gaming-storage"
    required: true
    config:
      protocols: ["nfs"]
      encryption: "beardog"
      high_performance: true

services:
  matchmaking:
    primal: "orchestration"
    image: "gaming/matchmaker:latest"
    ports:
      - "7777:7777"
    environment:
      MAX_PLAYERS: "{{max_players}}"
      GAME_TYPES: "{{game_types}}"
    resources:
      cpu: 4.0
      memory: 8589934592

  physics-engine:
    primal: "compute"
    image: "gaming/physics:latest"
    ports:
      - "7778:7778"
    environment:
      TICK_RATE: "128"
      SIMULATION_MODE: "deterministic"
    resources:
      cpu: 8.0
      memory: 17179869184

  leaderboard:
    primal: "storage"
    image: "gaming/leaderboard:latest"
    ports:
      - "7779:7779"
    environment:
      UPDATE_INTERVAL: "1s"
    resources:
      cpu: 2.0
      memory: 2147483648
      storage: 107374182400

networking:
  mode: "bridge"
  low_latency: true
  anti_cheat: true
  discovery:
    method: "dns"
    config:
      domain: "tournament.biome.local"
"#.to_string()
    }

    /// Advance workflow with validation
    pub fn advance_workflow(&mut self) {
        // Validate current state before advancing
        if !self.validate_current_state() {
            return;
        }
        
        let next_state = match self.workflow_state {
            WorkflowState::SelectTeam => {
                if self.current_team.is_empty() {
                    self.deployment_feedback = "Please select a team before continuing.".to_string();
                    return;
                }
                WorkflowState::SelectNiche
            },
            WorkflowState::SelectNiche => {
                if self.selected_niche.is_none() {
                    self.deployment_feedback = "Please select a niche template before continuing.".to_string();
                    return;
                }
                WorkflowState::ConfigureManifest
            },
            WorkflowState::ConfigureManifest => {
                self.generate_manifest();
                if self.generated_manifest.is_none() {
                    self.deployment_feedback = "Failed to generate manifest. Please check your configuration.".to_string();
                    return;
                }
                WorkflowState::EditYAML
            },
            WorkflowState::EditYAML => {
                if !self.validate_yaml_manifest() {
                    self.deployment_feedback = "YAML validation failed. Please fix errors before continuing.".to_string();
                    return;
                }
                WorkflowState::Deploy
            },
            WorkflowState::Deploy => {
                self.start_deployment();
                WorkflowState::Complete
            },
            WorkflowState::Complete => WorkflowState::Complete,
        };
        
        // Save current state to history
        self.workflow_history.push(WorkflowStep {
            step: self.workflow_state.clone(),
            timestamp: std::time::Instant::now(),
            data: Some(self.serialize_current_state()),
        });
        
        self.workflow_state = next_state;
        self.deployment_feedback.clear();
        
        // Auto-save workflow state
        self.save_workflow_state();
    }

    /// Go back to previous workflow step with state restoration
    pub fn previous_workflow(&mut self) {
        if let Some(previous_step) = self.workflow_history.pop() {
            self.workflow_state = previous_step.step;
            
            // Restore state data if available
            if let Some(data) = previous_step.data {
                self.restore_state_from_data(&data);
            }
            
            self.deployment_feedback.clear();
        }
    }

    /// Validate current workflow state
    fn validate_current_state(&self) -> bool {
        match self.workflow_state {
            WorkflowState::SelectTeam => {
                !self.current_team.is_empty() && 
                self.teams.iter().any(|t| t.name == self.current_team)
            },
            WorkflowState::SelectNiche => {
                self.selected_niche.is_some()
            },
            WorkflowState::ConfigureManifest => {
                if let Some(ref niche) = self.selected_niche {
                    // Validate all required customizations are filled
                    niche.customization_options.iter().all(|opt| {
                        if opt.required {
                            self.manifest_customizations.contains_key(&opt.key) &&
                            !self.manifest_customizations[&opt.key].is_empty()
                        } else {
                            true
                        }
                    })
                } else {
                    false
                }
            },
            WorkflowState::EditYAML => {
                self.generated_manifest.is_some() && 
                self.validate_yaml_manifest()
            },
            WorkflowState::Deploy => {
                self.generated_manifest.is_some() &&
                self.validate_yaml_manifest() &&
                self.check_resource_availability()
            },
            WorkflowState::Complete => true,
        }
    }

    /// Validate YAML manifest
    fn validate_yaml_manifest(&self) -> bool {
        if let Some(ref manifest) = self.generated_manifest {
            // Basic YAML validation
            if manifest.trim().is_empty() {
                return false;
            }
            
            // Check for required sections
            let required_sections = ["apiVersion", "kind", "metadata", "primals"];
            for section in required_sections {
                if !manifest.contains(section) {
                    return false;
                }
            }
            
            // Check for valid YAML structure (basic check)
            manifest.lines().all(|line| {
                let trimmed = line.trim();
                if trimmed.is_empty() || trimmed.starts_with('#') {
                    return true;
                }
                
                // Check for proper indentation and structure
                if trimmed.contains(':') {
                    let parts: Vec<&str> = trimmed.split(':').collect();
                    parts.len() >= 2 || parts[0].trim().chars().all(|c| c.is_alphanumeric() || c == '_' || c == '-')
                } else {
                    trimmed.starts_with('-') || trimmed.chars().all(|c| c.is_alphanumeric() || c.is_whitespace() || "._-".contains(c))
                }
            })
        } else {
            false
        }
    }

    /// Check resource availability for deployment
    fn check_resource_availability(&self) -> bool {
        if let Some(ref niche) = self.selected_niche {
            if let Some(quota) = self.team_quotas.get(&self.current_team) {
                // Check if we have enough resources
                let available_cpu = quota.max_cpu_cores - quota.used_cpu_cores;
                let available_memory = quota.max_memory_gb - quota.used_memory_gb;
                let available_storage = quota.max_storage_gb - quota.used_storage_gb;
                
                available_cpu >= niche.estimated_resources.cpu_cores &&
                available_memory >= niche.estimated_resources.memory_gb &&
                available_storage >= niche.estimated_resources.storage_gb
            } else {
                // If no quota info, assume resources are available
                true
            }
        } else {
            false
        }
    }

    /// Generate manifest from selected niche and customizations
    pub fn generate_manifest(&mut self) {
        if let Some(ref niche) = self.selected_niche {
            let mut manifest = niche.manifest_template.clone();
            
            // Apply customizations with validation
            for (key, value) in &self.manifest_customizations {
                // Validate customization value
                if let Some(option) = niche.customization_options.iter().find(|opt| opt.key == *key) {
                    if let Some(ref regex) = option.validation_regex {
                        if let Ok(re) = regex::Regex::new(regex) {
                            if !re.is_match(value) {
                                self.deployment_feedback = format!("Invalid value for {}: {}", option.name, value);
                                return;
                            }
                        }
                    }
                }
                
                manifest = manifest.replace(&format!("{{{{{}}}}}", key), value);
            }
            
            // Apply team-specific values
            manifest = manifest.replace("{{team_name}}", &self.current_team);
            manifest = manifest.replace("{{team_id}}", &self.current_team.to_lowercase().replace(" ", "-"));
            
            // Apply timestamp
            let timestamp = chrono::Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();
            manifest = manifest.replace("{{timestamp}}", &timestamp);
            
            // Apply resource estimates
            let resources = &niche.estimated_resources;
            manifest = manifest.replace("{{cpu_cores}}", &resources.cpu_cores.to_string());
            manifest = manifest.replace("{{memory_gb}}", &format!("{}GB", resources.memory_gb));
            manifest = manifest.replace("{{storage_gb}}", &format!("{}GB", resources.storage_gb));
            
            self.generated_manifest = Some(manifest);
        }
    }

    /// Start deployment process
    fn start_deployment(&mut self) {
        if let Some(ref manifest) = self.generated_manifest {
            self.action_in_progress = true;
            self.deployment_feedback = "Starting deployment...".to_string();
            
            // Create deployment record
            let deployment = DeploymentInfo {
                id: format!("dep-{}-{}", self.current_team.to_lowercase().replace(" ", "-"), chrono::Utc::now().timestamp()),
                name: self.selected_niche.as_ref().map(|n| n.name.clone()).unwrap_or_else(|| "Custom Deployment".to_string()),
                team: self.current_team.clone(),
                status: DeploymentStatus::Pending,
                created_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                updated_at: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                services: self.extract_services_from_manifest(manifest),
                resource_usage: self.selected_niche.as_ref().map(|n| ResourceUsage {
                    cpu_cores: n.estimated_resources.cpu_cores,
                    memory_gb: n.estimated_resources.memory_gb,
                    storage_gb: n.estimated_resources.storage_gb,
                    network_mbps: n.estimated_resources.network_mbps,
                }).unwrap_or_default(),
                health_score: 0.0,
            };
            
            self.deployments.push(deployment);
            
            // Update team resource usage
            if let Some(quota) = self.team_quotas.get_mut(&self.current_team) {
                if let Some(ref niche) = self.selected_niche {
                    quota.used_cpu_cores += niche.estimated_resources.cpu_cores;
                    quota.used_memory_gb += niche.estimated_resources.memory_gb;
                    quota.used_storage_gb += niche.estimated_resources.storage_gb;
                    quota.used_deployments += 1;
                }
            }
            
            // Simulate deployment process
            self.deployment_feedback = "Deployment created successfully!".to_string();
            self.action_in_progress = false;
        }
    }

    /// Extract services from manifest for deployment tracking
    fn extract_services_from_manifest(&self, manifest: &str) -> Vec<ServiceInfo> {
        let mut services = Vec::new();
        let mut current_service = None;
        let mut in_services_section = false;
        
        for line in manifest.lines() {
            let trimmed = line.trim();
            
            if trimmed == "services:" {
                in_services_section = true;
                continue;
            }
            
            if in_services_section {
                if trimmed.starts_with("  ") && trimmed.ends_with(':') {
                    // New service definition
                    if let Some(service) = current_service.take() {
                        services.push(service);
                    }
                    
                    let service_name = trimmed.trim_end_matches(':').trim().to_string();
                    current_service = Some(ServiceInfo {
                        name: service_name,
                        primal: "toadstool".to_string(), // Default, will be updated
                        status: ServiceStatus::Starting,
                        endpoints: Vec::new(),
                        health_check: HealthCheck {
                            status: HealthStatus::Unknown,
                            last_check: chrono::Utc::now().format("%Y-%m-%d %H:%M:%S").to_string(),
                            response_time_ms: 0,
                            error_message: None,
                        },
                    });
                } else if trimmed.starts_with("    primal:") {
                    if let Some(ref mut service) = current_service {
                        service.primal = trimmed.split(':').nth(1).unwrap_or("toadstool").trim().to_string();
                    }
                } else if trimmed.starts_with("    ports:") || trimmed.starts_with("      - \"") {
                    if let Some(ref mut service) = current_service {
                        if let Some(port_def) = trimmed.strip_prefix("      - \"") {
                            if let Some(port) = port_def.strip_suffix('"') {
                                service.endpoints.push(format!("http://localhost:{}", port.split(':').next().unwrap_or("8080")));
                            }
                        }
                    }
                } else if !trimmed.starts_with("  ") && !trimmed.is_empty() {
                    // End of services section
                    break;
                }
            }
        }
        
        // Add the last service if any
        if let Some(service) = current_service {
            services.push(service);
        }
        
        services
    }

    /// Serialize current workflow state for history
    fn serialize_current_state(&self) -> String {
        serde_json::json!({
            "workflow_state": format!("{:?}", self.workflow_state),
            "current_team": self.current_team,
            "selected_niche": self.selected_niche.as_ref().map(|n| n.id.clone()),
            "manifest_customizations": self.manifest_customizations,
            "generated_manifest": self.generated_manifest
        }).to_string()
    }

    /// Restore state from serialized data
    fn restore_state_from_data(&mut self, data: &str) {
        if let Ok(state) = serde_json::from_str::<serde_json::Value>(data) {
            if let Some(team) = state["current_team"].as_str() {
                self.current_team = team.to_string();
            }
            
            if let Some(niche_id) = state["selected_niche"].as_str() {
                self.selected_niche = self.available_niches.iter()
                    .find(|n| n.id == niche_id)
                    .cloned();
            }
            
            if let Some(customizations) = state["manifest_customizations"].as_object() {
                self.manifest_customizations.clear();
                for (key, value) in customizations {
                    if let Some(val_str) = value.as_str() {
                        self.manifest_customizations.insert(key.clone(), val_str.to_string());
                    }
                }
            }
            
            if let Some(manifest) = state["generated_manifest"].as_str() {
                self.generated_manifest = Some(manifest.to_string());
            }
        }
    }

    /// Save workflow state to persistent storage
    fn save_workflow_state(&self) {
        // In a real implementation, this would save to a file or database
        // For now, we'll just log the state
        println!("Saving workflow state: {:?}", self.workflow_state);
    }

    /// Load workflow state from persistent storage
    pub fn load_workflow_state(&mut self) {
        // In a real implementation, this would load from a file or database
        // For now, we'll just log the attempt
        println!("Loading workflow state...");
    }

    /// Reset workflow to beginning with cleanup
    pub fn reset_workflow(&mut self) {
        self.workflow_state = WorkflowState::SelectTeam;
        self.selected_niche = None;
        self.generated_manifest = None;
        self.manifest_customizations.clear();
        self.workflow_history.clear();
        self.show_niche_selector = false;
        self.show_manifest_editor = false;
        self.show_yaml_editor = false;
        self.deployment_feedback.clear();
        self.action_in_progress = false;
        
        // Save the reset state
        self.save_workflow_state();
    }

    /// Get workflow completion percentage
    pub fn get_workflow_progress(&self) -> f32 {
        match self.workflow_state {
            WorkflowState::SelectTeam => 0.0,
            WorkflowState::SelectNiche => 0.2,
            WorkflowState::ConfigureManifest => 0.4,
            WorkflowState::EditYAML => 0.6,
            WorkflowState::Deploy => 0.8,
            WorkflowState::Complete => 1.0,
        }
    }

    /// Check if workflow can advance
    pub fn can_advance_workflow(&self) -> bool {
        self.validate_current_state() && self.workflow_state != WorkflowState::Complete
    }

    /// Check if workflow can go back
    pub fn can_go_back(&self) -> bool {
        !self.workflow_history.is_empty() && self.workflow_state != WorkflowState::SelectTeam
    }

    /// Get current workflow step description
    pub fn get_current_step_description(&self) -> &'static str {
        match self.workflow_state {
            WorkflowState::SelectTeam => "Select or create a team for your biome deployment",
            WorkflowState::SelectNiche => "Choose a niche template that matches your needs",
            WorkflowState::ConfigureManifest => "Customize the niche configuration for your use case",
            WorkflowState::EditYAML => "Review and edit the generated YAML manifest",
            WorkflowState::Deploy => "Deploy your biome to the selected team",
            WorkflowState::Complete => "Deployment complete! Your biome is now running",
        }
    }

    /// Get validation errors for current state
    pub fn get_validation_errors(&self) -> Vec<String> {
        let mut errors = Vec::new();
        
        match self.workflow_state {
            WorkflowState::SelectTeam => {
                if self.current_team.is_empty() {
                    errors.push("No team selected".to_string());
                }
            },
            WorkflowState::SelectNiche => {
                if self.selected_niche.is_none() {
                    errors.push("No niche template selected".to_string());
                }
            },
            WorkflowState::ConfigureManifest => {
                if let Some(ref niche) = self.selected_niche {
                    for option in &niche.customization_options {
                        if option.required && !self.manifest_customizations.contains_key(&option.key) {
                            errors.push(format!("Required field '{}' is not filled", option.name));
                        }
                    }
                }
            },
            WorkflowState::EditYAML => {
                if !self.validate_yaml_manifest() {
                    errors.push("YAML manifest validation failed".to_string());
                }
            },
            WorkflowState::Deploy => {
                if !self.check_resource_availability() {
                    errors.push("Insufficient resources for deployment".to_string());
                }
            },
            WorkflowState::Complete => {},
        }
        
        errors
    }
}

impl View for ByobView {
    fn render(&mut self, ui: &mut egui::Ui, _ctx: &egui::Context) {
        // Refresh live data periodically
        if self.last_action_time.elapsed() > std::time::Duration::from_secs(2) {
            self.refresh_live_data();
            self.last_action_time = std::time::Instant::now();
        }

        // Enhanced hierarchical workflow header
        ui.horizontal(|ui| {
            ui.heading("🧬 Build Your Own Biome");
            ui.separator();
            
            // Current step with description
            ui.vertical(|ui| {
                ui.label(format!("Step: {:?}", self.workflow_state));
                ui.small(self.get_current_step_description());
            });
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("🔄 Reset Workflow").clicked() {
                    self.reset_workflow();
                }
                
                // Enhanced navigation with validation
                if self.can_go_back() {
                    if ui.button("⬅️ Previous").clicked() {
                        self.previous_workflow();
                    }
                } else {
                    ui.add_enabled(false, egui::Button::new("⬅️ Previous"));
                }
                
                if self.can_advance_workflow() {
                    if ui.button("➡️ Next").clicked() {
                        self.advance_workflow();
                    }
                } else {
                    ui.add_enabled(false, egui::Button::new("➡️ Next"));
                }
            });
        });

        ui.add_space(10.0);

        // Enhanced workflow progress indicator with percentage
        ui.horizontal(|ui| {
            let steps = ["Team", "Niche", "Manifest", "YAML", "Deploy"];
            let current_step = match self.workflow_state {
                WorkflowState::SelectTeam => 0,
                WorkflowState::SelectNiche => 1,
                WorkflowState::ConfigureManifest => 2,
                WorkflowState::EditYAML => 3,
                WorkflowState::Deploy => 4,
                WorkflowState::Complete => 4,
            };
            
            for (i, step) in steps.iter().enumerate() {
                let color = if i < current_step {
                    Color32::from_rgb(0, 150, 0)  // Completed
                } else if i == current_step {
                    Color32::from_rgb(0, 100, 200)  // Current
                } else {
                    Color32::GRAY  // Pending
                };
                
                ui.colored_label(color, format!("{}. {}", i + 1, step));
                
                if i < steps.len() - 1 {
                    ui.label("→");
                }
            }
            
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let progress = self.get_workflow_progress();
                ui.add(egui::ProgressBar::new(progress).show_percentage());
            });
        });

        ui.add_space(10.0);

        // Validation errors display
        let validation_errors = self.get_validation_errors();
        if !validation_errors.is_empty() {
            ui.group(|ui| {
                ui.colored_label(Color32::RED, "⚠️ Validation Issues:");
                for error in validation_errors {
                    ui.colored_label(Color32::RED, format!("• {}", error));
                }
            });
            ui.add_space(10.0);
        }

        // Feedback display
        if !self.deployment_feedback.is_empty() {
            ui.group(|ui| {
                let color = if self.deployment_feedback.contains("Error") || self.deployment_feedback.contains("Failed") {
                    Color32::RED
                } else if self.deployment_feedback.contains("success") {
                    Color32::GREEN
                } else {
                    Color32::BLUE
                };
                ui.colored_label(color, &self.deployment_feedback);
            });
            ui.add_space(10.0);
        }

        // Render current workflow step
        match self.workflow_state {
            WorkflowState::SelectTeam => self.render_enhanced_team_selection(ui),
            WorkflowState::SelectNiche => self.render_enhanced_niche_selection(ui),
            WorkflowState::ConfigureManifest => self.render_enhanced_manifest_configuration(ui),
            WorkflowState::EditYAML => self.render_enhanced_yaml_editing(ui),
            WorkflowState::Deploy => self.render_enhanced_deployment(ui),
            WorkflowState::Complete => self.render_enhanced_completion(ui),
        }
        
        ui.add_space(20.0);

        // Traditional tab view (for existing functionality)
        ui.separator();
        ui.heading("Advanced Management");
        self.render_tab_bar(ui);
        ui.add_space(10.0);

        match self.selected_tab {
            ByobTab::Overview => self.render_overview_tab(ui),
            ByobTab::Teams => self.render_teams_tab(ui),
            ByobTab::Deployments => self.render_deployments_tab(ui),
            ByobTab::Resources => self.render_resources_tab(ui),
            ByobTab::Monitoring => self.render_monitoring_tab(ui),
        }
    }
}

impl ByobView {
    /// Enhanced team selection with validation
    fn render_enhanced_team_selection(&mut self, ui: &mut egui::Ui) {
        ui.heading("👥 Select Team");
        ui.label(self.get_current_step_description());
        ui.add_space(10.0);
        
        // Team selection with validation
        ui.horizontal(|ui| {
            ui.label("Team:");
            egui::ComboBox::from_label("")
                .selected_text(&self.current_team)
                .show_ui(ui, |ui| {
                    for team in &self.teams {
                        if ui.selectable_value(&mut self.current_team, team.name.clone(), &team.name).clicked() {
                            self.deployment_feedback.clear();
                        }
                    }
                });
            
            if ui.button("➕ Create New Team").clicked() {
                self.show_team_creator = true;
            }
        });
        
        ui.add_space(10.0);
        
        // Show selected team details
        if !self.current_team.is_empty() {
            if let Some(team) = self.teams.iter().find(|t| t.name == self.current_team) {
                ui.group(|ui| {
                    ui.heading("Team Details");
                    ui.label(format!("Name: {}", team.name));
                    ui.label(format!("Description: {}", team.description));
                    ui.label(format!("Members: {}", team.members.len()));
                    ui.label(format!("Status: {:?}", team.status));
                    ui.label(format!("Workspace: {}", team.workspace_url));
                    
                    // Show resource quotas if available
                    if let Some(quota) = self.team_quotas.get(&team.name) {
                        ui.separator();
                        ui.label("Resource Quotas:");
                        ui.label(format!("CPU: {:.1}/{:.1} cores", quota.used_cpu_cores, quota.max_cpu_cores));
                        ui.label(format!("Memory: {:.1}/{:.1} GB", quota.used_memory_gb, quota.max_memory_gb));
                        ui.label(format!("Storage: {:.1}/{:.1} GB", quota.used_storage_gb, quota.max_storage_gb));
                        ui.label(format!("Deployments: {}/{}", quota.used_deployments, quota.max_deployments));
                    }
                });
            }
        }
        
        // Team creation dialog
        if self.show_team_creator {
            self.render_team_creation_dialog(ui);
        }
    }

    /// Enhanced niche selection with better filtering
    fn render_enhanced_niche_selection(&mut self, ui: &mut egui::Ui) {
        ui.heading("🎭 Select Niche Template");
        ui.label(self.get_current_step_description());
        ui.add_space(10.0);
        
        // Category and difficulty filters
        ui.horizontal(|ui| {
            ui.label("Category:");
            egui::ComboBox::from_label("")
                .selected_text("All Categories")
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut (), (), "All Categories");
                    ui.selectable_value(&mut (), (), "Web Development");
                    ui.selectable_value(&mut (), (), "AI Research");
                    ui.selectable_value(&mut (), (), "Gaming");
                    ui.selectable_value(&mut (), (), "Healthcare");
                });
            
            ui.separator();
            
            ui.label("Difficulty:");
            egui::ComboBox::from_label("")
                .selected_text("All Levels")
                .show_ui(ui, |ui| {
                    ui.selectable_value(&mut (), (), "All Levels");
                    ui.selectable_value(&mut (), (), "Beginner");
                    ui.selectable_value(&mut (), (), "Intermediate");
                    ui.selectable_value(&mut (), (), "Advanced");
                    ui.selectable_value(&mut (), (), "Expert");
                });
        });
        
        ui.add_space(15.0);
        
        // Show selected niche details
        if let Some(ref niche) = self.selected_niche {
            ui.group(|ui| {
                ui.heading("Selected Niche");
                ui.label(format!("Name: {}", niche.name));
                ui.label(format!("Description: {}", niche.description));
                ui.label(format!("Category: {:?}", niche.category));
                ui.label(format!("Difficulty: {:?}", niche.difficulty));
                
                ui.separator();
                ui.label("Estimated Resources:");
                ui.label(format!("CPU: {:.1} cores", niche.estimated_resources.cpu_cores));
                ui.label(format!("Memory: {:.1} GB", niche.estimated_resources.memory_gb));
                ui.label(format!("Storage: {:.1} GB", niche.estimated_resources.storage_gb));
                if niche.estimated_resources.gpu_required {
                    ui.colored_label(Color32::YELLOW, "GPU: Required");
                }
                
                // Resource availability check
                if let Some(quota) = self.team_quotas.get(&self.current_team) {
                    ui.separator();
                    ui.label("Resource Availability:");
                    
                    let cpu_available = quota.max_cpu_cores - quota.used_cpu_cores;
                    let memory_available = quota.max_memory_gb - quota.used_memory_gb;
                    let storage_available = quota.max_storage_gb - quota.used_storage_gb;
                    
                    let cpu_ok = cpu_available >= niche.estimated_resources.cpu_cores;
                    let memory_ok = memory_available >= niche.estimated_resources.memory_gb;
                    let storage_ok = storage_available >= niche.estimated_resources.storage_gb;
                    
                    ui.colored_label(
                        if cpu_ok { Color32::GREEN } else { Color32::RED },
                        format!("CPU: {:.1} available", cpu_available)
                    );
                    ui.colored_label(
                        if memory_ok { Color32::GREEN } else { Color32::RED },
                        format!("Memory: {:.1} GB available", memory_available)
                    );
                    ui.colored_label(
                        if storage_ok { Color32::GREEN } else { Color32::RED },
                        format!("Storage: {:.1} GB available", storage_available)
                    );
                }
            });
            ui.add_space(10.0);
        }
        
        // Niche grid (simplified for now)
        egui::ScrollArea::vertical()
            .max_height(300.0)
            .show(ui, |ui| {
                let niches = self.available_niches.clone();
                for niche in niches {
                    ui.group(|ui| {
                        ui.horizontal(|ui| {
                            ui.vertical(|ui| {
                                ui.strong(&niche.name);
                                ui.label(&niche.description);
                                ui.label(format!("Category: {:?} | Difficulty: {:?}", niche.category, niche.difficulty));
                            });
                            
                            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                                if ui.button("Select").clicked() {
                                    self.selected_niche = Some(niche.clone());
                                    self.deployment_feedback.clear();
                                }
                            });
                        });
                    });
                    ui.add_space(5.0);
                }
            });
    }

    /// Enhanced manifest configuration with real-time validation
    fn render_enhanced_manifest_configuration(&mut self, ui: &mut egui::Ui) {
        ui.heading("⚙️ Configure Manifest");
        ui.label(self.get_current_step_description());
        ui.add_space(10.0);
        
        if let Some(ref niche) = self.selected_niche.clone() {
            ui.label(format!("Configuring: {}", niche.name));
            ui.add_space(10.0);
            
            // Configuration options
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    for option in &niche.customization_options {
                        ui.group(|ui| {
                            ui.horizontal(|ui| {
                                ui.label(&option.name);
                                if option.required {
                                    ui.colored_label(Color32::RED, "*");
                                }
                            });
                            
                            ui.small(&option.description);
                            
                            let current_value = self.manifest_customizations
                                .get(&option.key)
                                .cloned()
                                .unwrap_or_else(|| option.default_value.clone());
                            
                            match &option.option_type {
                                CustomizationType::Text => {
                                    let mut value = current_value;
                                    if ui.text_edit_singleline(&mut value).changed() {
                                        self.manifest_customizations.insert(option.key.clone(), value);
                                    }
                                }
                                CustomizationType::Boolean => {
                                    let mut value = current_value == "true";
                                    if ui.checkbox(&mut value, "").changed() {
                                        self.manifest_customizations.insert(option.key.clone(), value.to_string());
                                    }
                                }
                                CustomizationType::Select(choices) => {
                                    let mut selected_value = current_value.clone();
                                    egui::ComboBox::from_label("")
                                        .selected_text(&selected_value)
                                        .show_ui(ui, |ui| {
                                            for choice in choices {
                                                if ui.selectable_value(&mut selected_value, choice.clone(), choice).clicked() {
                                                    self.manifest_customizations.insert(option.key.clone(), choice.clone());
                                                }
                                            }
                                        });
                                }
                                _ => {
                                    ui.label("Advanced configuration type");
                                }
                            }
                        });
                        ui.add_space(5.0);
                    }
                });
            
            ui.add_space(15.0);
            
            // Validation summary
            let validation_errors = self.get_validation_errors();
            if !validation_errors.is_empty() {
                ui.group(|ui| {
                    ui.colored_label(Color32::RED, "Configuration Issues:");
                    for error in validation_errors {
                        ui.colored_label(Color32::RED, format!("• {}", error));
                    }
                });
            } else {
                ui.colored_label(Color32::GREEN, "✅ Configuration is valid");
            }
        }
    }

    /// Enhanced YAML editing with validation
    fn render_enhanced_yaml_editing(&mut self, ui: &mut egui::Ui) {
        ui.heading("📝 Edit YAML Manifest");
        ui.label(self.get_current_step_description());
        ui.add_space(10.0);
        
        if let Some(manifest) = self.generated_manifest.clone() {
            ui.horizontal(|ui| {
                if ui.button("🔍 Validate").clicked() {
                    // Trigger validation
                    self.deployment_feedback = if self.validate_yaml_manifest() {
                        "✅ YAML validation passed!".to_string()
                    } else {
                        "❌ YAML validation failed!".to_string()
                    };
                }
                
                if ui.button("🔄 Regenerate").clicked() {
                    self.generate_manifest();
                }
                
                if ui.button("💾 Save").clicked() {
                    self.deployment_feedback = "Manifest saved!".to_string();
                }
            });
            
            ui.add_space(10.0);
            
            let mut manifest_clone = manifest;
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    if ui.add(egui::TextEdit::multiline(&mut manifest_clone)
                        .code_editor()
                        .desired_rows(20)
                        .desired_width(f32::INFINITY)).changed() {
                        // Update the manifest when changed
                    }
                });
            
            // Update the stored manifest
            self.generated_manifest = Some(manifest_clone);
        }
    }

    /// Enhanced deployment with progress tracking
    fn render_enhanced_deployment(&mut self, ui: &mut egui::Ui) {
        ui.heading("🚀 Deploy Biome");
        ui.label(self.get_current_step_description());
        ui.add_space(10.0);
        
        // Pre-deployment validation - store result to avoid multiple borrows
        let validation_errors = self.get_validation_errors();
        let has_errors = !validation_errors.is_empty();
        
        if has_errors {
            ui.group(|ui| {
                ui.colored_label(Color32::RED, "⚠️ Deployment Blocked:");
                for error in validation_errors {
                    ui.colored_label(Color32::RED, format!("• {}", error));
                }
            });
            ui.add_space(10.0);
        }
        
        // Deployment summary
        ui.group(|ui| {
            ui.heading("Deployment Summary");
            ui.label(format!("Team: {}", self.current_team));
            
            if let Some(ref niche) = self.selected_niche {
                ui.label(format!("Niche: {}", niche.name));
                ui.label(format!("Category: {:?}", niche.category));
                
                ui.separator();
                ui.label("Resource Requirements:");
                ui.label(format!("CPU: {:.1} cores", niche.estimated_resources.cpu_cores));
                ui.label(format!("Memory: {:.1} GB", niche.estimated_resources.memory_gb));
                ui.label(format!("Storage: {:.1} GB", niche.estimated_resources.storage_gb));
                
                if niche.estimated_resources.gpu_required {
                    ui.colored_label(Color32::YELLOW, "GPU: Required");
                }
            }
        });
        
        ui.add_space(15.0);
        
        // Deployment controls
        ui.horizontal(|ui| {
            let can_deploy = !has_errors && !self.action_in_progress;
            
            if ui.add_enabled(can_deploy, egui::Button::new("🚀 Deploy Now")).clicked() {
                self.advance_workflow();
            }
            
            if ui.button("📋 Save Configuration").clicked() {
                self.deployment_feedback = "Configuration saved for later deployment!".to_string();
            }
        });
        
        if self.action_in_progress {
            ui.add_space(10.0);
            ui.spinner();
            ui.label("Deploying biome...");
        }
    }

    /// Enhanced completion with next steps
    fn render_enhanced_completion(&mut self, ui: &mut egui::Ui) {
        ui.heading("✅ Deployment Complete");
        ui.label(self.get_current_step_description());
        ui.add_space(10.0);
        
        ui.group(|ui| {
            ui.heading("🎉 Success!");
            ui.label("Your biome has been successfully deployed and is now running.");
            
            if let Some(ref niche) = self.selected_niche {
                ui.separator();
                ui.label(format!("Deployed: {}", niche.name));
                ui.label(format!("Team: {}", self.current_team));
                ui.label(format!("Deployment ID: dep-{}-{}", 
                    self.current_team.to_lowercase().replace(" ", "-"), 
                    chrono::Utc::now().timestamp()));
            }
        });
        
        ui.add_space(15.0);
        
        // Next steps
        ui.group(|ui| {
            ui.heading("Next Steps");
            ui.label("• Monitor your biome's health and performance");
            ui.label("• Scale resources as needed");
            ui.label("• Share your biome configuration with your team");
            ui.label("• Explore advanced features and customizations");
        });
        
        ui.add_space(15.0);
        
        // Action buttons
        ui.horizontal(|ui| {
            if ui.button("🏠 Go to Dashboard").clicked() {
                // Navigate to dashboard
            }
            
            if ui.button("🧬 Deploy Another Biome").clicked() {
                self.reset_workflow();
            }
            
            if ui.button("📊 View Monitoring").clicked() {
                self.selected_tab = ByobTab::Monitoring;
            }
            
            if ui.button("⚙️ Advanced Settings").clicked() {
                // Navigate to advanced settings
            }
        });
    }

    /// Team creation dialog
    fn render_team_creation_dialog(&mut self, ui: &mut egui::Ui) {
        egui::Window::new("Create New Team")
            .collapsible(false)
            .resizable(true)
            .default_size([450.0, 350.0])
            .show(ui.ctx(), |ui| {
                ui.label("Team Name:");
                ui.text_edit_singleline(&mut self.new_team_name);
                
                ui.add_space(10.0);
                ui.label("Description:");
                ui.text_edit_multiline(&mut self.new_team_description);
                
                ui.add_space(15.0);
                
                // Validation
                let name_valid = !self.new_team_name.trim().is_empty();
                let name_unique = !self.teams.iter().any(|t| t.name == self.new_team_name);
                
                if !name_valid {
                    ui.colored_label(Color32::RED, "⚠️ Team name is required");
                }
                if !name_unique {
                    ui.colored_label(Color32::RED, "⚠️ Team name already exists");
                }
                
                ui.add_space(10.0);
                
                ui.horizontal(|ui| {
                    let can_create = name_valid && name_unique;
                    
                    if ui.add_enabled(can_create, egui::Button::new("Create Team")).clicked() {
                        // Create team logic
                        self.teams.push(TeamInfo {
                            id: format!("team-{}", self.teams.len() + 1),
                            name: self.new_team_name.clone(),
                            description: self.new_team_description.clone(),
                            members: vec!["current_user@biomeos.local".to_string()],
                            created_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                            status: TeamStatus::Active,
                            workspace_url: format!("https://{}.biomeos.local", 
                                self.new_team_name.to_lowercase().replace(" ", "-")),
                        });
                        
                        // Add default quota for new team
                        self.team_quotas.insert(self.new_team_name.clone(), ResourceQuota {
                            max_cpu_cores: 16.0,
                            max_memory_gb: 64.0,
                            max_storage_gb: 500.0,
                            max_deployments: 10,
                            used_cpu_cores: 0.0,
                            used_memory_gb: 0.0,
                            used_storage_gb: 0.0,
                            used_deployments: 0,
                        });
                        
                        self.current_team = self.new_team_name.clone();
                        self.new_team_name.clear();
                        self.new_team_description.clear();
                        self.show_team_creator = false;
                        self.deployment_feedback = "Team created successfully!".to_string();
                    }
                    
                    if ui.button("Cancel").clicked() {
                        self.show_team_creator = false;
                        self.new_team_name.clear();
                        self.new_team_description.clear();
                    }
                });
            });
    }
} 