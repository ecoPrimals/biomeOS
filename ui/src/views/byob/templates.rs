//! BYOB Template Management
//!
//! This module handles niche template loading, parsing, and fallback generation
//! for the BYOB system.
//!
//! The system is completely universal and uses capability-based matching
//! instead of hardcoded primal names.

use super::data::get_primal_discovery;
use super::types::*;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;

/// Template loader for niche definitions
pub struct TemplateLoader {
    templates: Vec<NicheTemplate>,
    template_path: String,
}

impl TemplateLoader {
    pub fn new(template_path: &str) -> Self {
        let mut loader = Self {
            templates: Vec::new(),
            template_path: template_path.to_string(),
        };

        // Load templates from YAML files
        loader.load_templates_from_directory();

        // If no templates found, use fallback system
        if loader.templates.is_empty() {
            loader.load_fallback_templates();
        }

        loader
    }

    fn load_templates_from_directory(&mut self) {
        let template_dirs = vec![
            format!("{}/niches", self.template_path),
            format!("{}/templates/niches", self.template_path),
            "templates/niches".to_string(),
            "niches".to_string(),
        ];

        for dir in template_dirs {
            if let Ok(templates) = self.load_templates_from_path(&dir) {
                self.templates.extend(templates);
                return;
            }
        }
    }

    fn load_templates_from_path(
        &self,
        path: &str,
    ) -> Result<Vec<NicheTemplate>, Box<dyn std::error::Error>> {
        let mut templates = Vec::new();

        if !Path::new(path).exists() {
            return Err("Path does not exist".into());
        }

        for entry in fs::read_dir(path)? {
            let entry = entry?;
            let file_path = entry.path();

            if let Some(extension) = file_path.extension() {
                if extension == "yaml" || extension == "yml" {
                    if let Ok(template) = self.load_template_from_file(&file_path) {
                        templates.push(template);
                    }
                }
            }
        }

        Ok(templates)
    }

    fn load_template_from_file(
        &self,
        file_path: &Path,
    ) -> Result<NicheTemplate, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(file_path)?;
        let template: NicheTemplate = serde_yaml::from_str(&content)?;
        Ok(template)
    }

    fn load_fallback_templates(&mut self) {
        // Universal fallback templates using capability-based matching
        let fallback_templates = vec![
            NicheTemplate {
                id: "web-development".to_string(),
                name: "Web Development".to_string(),
                description: "Modern web application development with compute and networking"
                    .to_string(),
                category: NicheCategory::WebDevelopment,
                difficulty: NicheDifficulty::Intermediate,
                features: vec![
                    "React/Vue/Angular frontend".to_string(),
                    "Node.js/Python backend".to_string(),
                    "Database integration".to_string(),
                    "API development".to_string(),
                    "Load balancing".to_string(),
                ],
                required_capabilities: [
                    PrimalCapability::Compute,
                    PrimalCapability::WebDevelopment,
                    PrimalCapability::Networking,
                ]
                .into_iter()
                .collect(),
                preferred_primals: vec![], // No hardcoded names
                manifest_template: r#"
name: "{{project_name}}"
description: "{{project_description}}"
version: "1.0.0"

capabilities:
  - compute
  - web-development
  - networking

services:
  frontend:
    image: "node:18-alpine"
    ports:
      - "3000:3000"
    capabilities: ["compute", "web-development"]
    
  backend:
    image: "python:3.11-slim"
    ports:
      - "8000:8000"
    capabilities: ["compute", "web-development"]
    
  proxy:
    image: "nginx:alpine"
    ports:
      - "80:80"
    capabilities: ["networking"]

deployment:
  strategy: capability-based
  auto_scale: true
  health_checks: true
"#
                .to_string(),
                customization_options: vec![
                    CustomizationOption {
                        id: "project_name".to_string(),
                        name: "Project Name".to_string(),
                        description: "Name of your web project".to_string(),
                        option_type: CustomizationType::Text,
                        default_value: "my-web-app".to_string(),
                        required: true,
                        validation_regex: Some(r"^[a-z0-9-]+$".to_string()),
                        depends_on_capability: None,
                    },
                    CustomizationOption {
                        id: "project_description".to_string(),
                        name: "Project Description".to_string(),
                        description: "Brief description of your project".to_string(),
                        option_type: CustomizationType::Text,
                        default_value: "A modern web application".to_string(),
                        required: false,
                        validation_regex: None,
                        depends_on_capability: None,
                    },
                    CustomizationOption {
                        id: "frontend_framework".to_string(),
                        name: "Frontend Framework".to_string(),
                        description: "Choose your frontend framework".to_string(),
                        option_type: CustomizationType::Select(vec![
                            "React".to_string(),
                            "Vue".to_string(),
                            "Angular".to_string(),
                            "Svelte".to_string(),
                        ]),
                        default_value: "React".to_string(),
                        required: true,
                        validation_regex: None,
                        depends_on_capability: Some(PrimalCapability::WebDevelopment),
                    },
                ],
                metadata: [
                    ("template_version".to_string(), "2.0".to_string()),
                    ("compatibility".to_string(), "universal".to_string()),
                ]
                .into_iter()
                .collect(),
            },
            NicheTemplate {
                id: "ai-research".to_string(),
                name: "AI Research".to_string(),
                description: "Machine learning and AI research platform".to_string(),
                category: NicheCategory::AIResearch,
                difficulty: NicheDifficulty::Advanced,
                features: vec![
                    "Jupyter notebooks".to_string(),
                    "GPU acceleration".to_string(),
                    "Model training".to_string(),
                    "Data processing".to_string(),
                    "Experiment tracking".to_string(),
                ],
                required_capabilities: [
                    PrimalCapability::AI,
                    PrimalCapability::MachineLearning,
                    PrimalCapability::Compute,
                    PrimalCapability::Analytics,
                ]
                .into_iter()
                .collect(),
                preferred_primals: vec![], // No hardcoded names
                manifest_template: r#"
name: "{{project_name}}"
description: "{{project_description}}"
version: "1.0.0"

capabilities:
  - ai
  - machine-learning
  - compute
  - analytics

services:
  jupyter:
    image: "jupyter/tensorflow-notebook:latest"
    ports:
      - "8888:8888"
    capabilities: ["ai", "machine-learning"]
    resources:
      gpu: true
      memory: "8Gi"
      
  mlflow:
    image: "python:3.11-slim"
    ports:
      - "5000:5000"
    capabilities: ["analytics"]
    
  data-processor:
    image: "python:3.11-slim"
    capabilities: ["compute", "analytics"]

deployment:
  strategy: capability-based
  gpu_required: true
  auto_scale: false
  health_checks: true
"#
                .to_string(),
                customization_options: vec![
                    CustomizationOption {
                        id: "project_name".to_string(),
                        name: "Research Project Name".to_string(),
                        description: "Name of your AI research project".to_string(),
                        option_type: CustomizationType::Text,
                        default_value: "ai-research".to_string(),
                        required: true,
                        validation_regex: Some(r"^[a-z0-9-]+$".to_string()),
                        depends_on_capability: None,
                    },
                    CustomizationOption {
                        id: "ml_framework".to_string(),
                        name: "ML Framework".to_string(),
                        description: "Choose your machine learning framework".to_string(),
                        option_type: CustomizationType::Select(vec![
                            "TensorFlow".to_string(),
                            "PyTorch".to_string(),
                            "Scikit-learn".to_string(),
                            "JAX".to_string(),
                        ]),
                        default_value: "TensorFlow".to_string(),
                        required: true,
                        validation_regex: None,
                        depends_on_capability: Some(PrimalCapability::MachineLearning),
                    },
                    CustomizationOption {
                        id: "gpu_enabled".to_string(),
                        name: "GPU Acceleration".to_string(),
                        description: "Enable GPU acceleration for training".to_string(),
                        option_type: CustomizationType::Boolean,
                        default_value: "true".to_string(),
                        required: false,
                        validation_regex: None,
                        depends_on_capability: Some(PrimalCapability::Compute),
                    },
                ],
                metadata: [
                    ("template_version".to_string(), "2.0".to_string()),
                    ("gpu_required".to_string(), "true".to_string()),
                    ("compatibility".to_string(), "universal".to_string()),
                ]
                .into_iter()
                .collect(),
            },
            NicheTemplate {
                id: "gaming-platform".to_string(),
                name: "Gaming Platform".to_string(),
                description: "Game development and hosting platform".to_string(),
                category: NicheCategory::Gaming,
                difficulty: NicheDifficulty::Advanced,
                features: vec![
                    "Game server hosting".to_string(),
                    "Real-time multiplayer".to_string(),
                    "Matchmaking".to_string(),
                    "Player analytics".to_string(),
                    "Anti-cheat systems".to_string(),
                ],
                required_capabilities: [
                    PrimalCapability::Gaming,
                    PrimalCapability::Compute,
                    PrimalCapability::Networking,
                    PrimalCapability::Security,
                ]
                .into_iter()
                .collect(),
                preferred_primals: vec![], // No hardcoded names
                manifest_template: r#"
name: "{{project_name}}"
description: "{{project_description}}"
version: "1.0.0"

capabilities:
  - gaming
  - compute
  - networking
  - security

services:
  game-server:
    image: "ubuntu:22.04"
    ports:
      - "{{game_port}}:{{game_port}}"
    capabilities: ["gaming", "compute"]
    
  matchmaking:
    image: "node:18-alpine"
    ports:
      - "3001:3001"
    capabilities: ["networking", "gaming"]
    
  anti-cheat:
    image: "python:3.11-slim"
    capabilities: ["security", "gaming"]

deployment:
  strategy: capability-based
  auto_scale: true
  health_checks: true
  security_enhanced: true
"#
                .to_string(),
                customization_options: vec![
                    CustomizationOption {
                        id: "project_name".to_string(),
                        name: "Game Project Name".to_string(),
                        description: "Name of your gaming project".to_string(),
                        option_type: CustomizationType::Text,
                        default_value: "my-game".to_string(),
                        required: true,
                        validation_regex: Some(r"^[a-z0-9-]+$".to_string()),
                        depends_on_capability: None,
                    },
                    CustomizationOption {
                        id: "game_port".to_string(),
                        name: "Game Port".to_string(),
                        description: "Port for game server".to_string(),
                        option_type: CustomizationType::Number {
                            min: Some(1024),
                            max: Some(65535),
                        },
                        default_value: "7777".to_string(),
                        required: true,
                        validation_regex: None,
                        depends_on_capability: Some(PrimalCapability::Networking),
                    },
                    CustomizationOption {
                        id: "max_players".to_string(),
                        name: "Max Players".to_string(),
                        description: "Maximum number of players".to_string(),
                        option_type: CustomizationType::Number {
                            min: Some(1),
                            max: Some(1000),
                        },
                        default_value: "32".to_string(),
                        required: true,
                        validation_regex: None,
                        depends_on_capability: Some(PrimalCapability::Gaming),
                    },
                ],
                metadata: [
                    ("template_version".to_string(), "2.0".to_string()),
                    ("real_time".to_string(), "true".to_string()),
                    ("compatibility".to_string(), "universal".to_string()),
                ]
                .into_iter()
                .collect(),
            },
            NicheTemplate {
                id: "custom-generic".to_string(),
                name: "Custom Generic".to_string(),
                description: "Fully customizable biome for any use case".to_string(),
                category: NicheCategory::Custom("Generic".to_string()),
                difficulty: NicheDifficulty::Expert,
                features: vec![
                    "Fully customizable".to_string(),
                    "Any capability combination".to_string(),
                    "Dynamic service configuration".to_string(),
                    "Advanced deployment options".to_string(),
                ],
                required_capabilities: HashSet::new(), // No requirements - fully flexible
                preferred_primals: vec![],             // No hardcoded names
                manifest_template: r#"
name: "{{project_name}}"
description: "{{project_description}}"
version: "1.0.0"

capabilities: {{capabilities}}

services: {{services}}

deployment:
  strategy: capability-based
  auto_scale: {{auto_scale}}
  health_checks: {{health_checks}}
"#
                .to_string(),
                customization_options: vec![
                    CustomizationOption {
                        id: "project_name".to_string(),
                        name: "Project Name".to_string(),
                        description: "Name of your custom project".to_string(),
                        option_type: CustomizationType::Text,
                        default_value: "custom-biome".to_string(),
                        required: true,
                        validation_regex: Some(r"^[a-z0-9-]+$".to_string()),
                        depends_on_capability: None,
                    },
                    CustomizationOption {
                        id: "capabilities".to_string(),
                        name: "Required Capabilities".to_string(),
                        description: "Select the capabilities you need".to_string(),
                        option_type: CustomizationType::Capabilities(HashSet::new()),
                        default_value: "compute,networking".to_string(),
                        required: true,
                        validation_regex: None,
                        depends_on_capability: None,
                    },
                    CustomizationOption {
                        id: "auto_scale".to_string(),
                        name: "Auto Scaling".to_string(),
                        description: "Enable automatic scaling".to_string(),
                        option_type: CustomizationType::Boolean,
                        default_value: "true".to_string(),
                        required: false,
                        validation_regex: None,
                        depends_on_capability: None,
                    },
                ],
                metadata: [
                    ("template_version".to_string(), "2.0".to_string()),
                    ("flexibility".to_string(), "maximum".to_string()),
                    ("compatibility".to_string(), "universal".to_string()),
                ]
                .into_iter()
                .collect(),
            },
        ];

        self.templates.extend(fallback_templates);
    }

    pub fn get_templates(&self) -> &[NicheTemplate] {
        &self.templates
    }

    pub fn get_template(&self, id: &str) -> Option<&NicheTemplate> {
        self.templates.iter().find(|t| t.id == id)
    }

    pub fn get_templates_by_category(&self, category: &NicheCategory) -> Vec<&NicheTemplate> {
        self.templates
            .iter()
            .filter(|t| &t.category == category)
            .collect()
    }

    pub fn get_templates_by_difficulty(&self, difficulty: &NicheDifficulty) -> Vec<&NicheTemplate> {
        self.templates
            .iter()
            .filter(|t| &t.difficulty == difficulty)
            .collect()
    }

    pub fn get_templates_by_capability(
        &self,
        capability: &PrimalCapability,
    ) -> Vec<&NicheTemplate> {
        self.templates
            .iter()
            .filter(|t| t.required_capabilities.contains(capability))
            .collect()
    }

    pub fn get_templates_by_capabilities(
        &self,
        capabilities: &HashSet<PrimalCapability>,
    ) -> Vec<&NicheTemplate> {
        self.templates
            .iter()
            .filter(|t| {
                capabilities
                    .iter()
                    .any(|cap| t.required_capabilities.contains(cap))
            })
            .collect()
    }

    pub fn find_compatible_templates(
        &self,
        _team_capabilities: &HashSet<PrimalCapability>,
    ) -> Vec<&NicheTemplate> {
        let discovery = get_primal_discovery();
        let available_primals = discovery.get_all_primals();

        self.templates
            .iter()
            .filter(|template| {
                // Check if we have primals that support the required capabilities
                template.required_capabilities.iter().all(|required_cap| {
                    available_primals
                        .iter()
                        .any(|primal| primal.capabilities.contains(required_cap))
                })
            })
            .collect()
    }
}

/// Get template loader instance
pub fn get_template_loader() -> TemplateLoader {
    TemplateLoader::new("templates")
}

/// Get all available niche templates
pub fn get_niche_templates() -> Vec<NicheTemplate> {
    let loader = get_template_loader();
    loader.get_templates().to_vec()
}

/// Get a specific niche template by ID
pub fn get_niche_template(id: &str) -> Option<NicheTemplate> {
    let loader = get_template_loader();
    loader.get_template(id).cloned()
}

/// Get templates compatible with team capabilities
pub fn get_compatible_templates(
    team_capabilities: &HashSet<PrimalCapability>,
) -> Vec<NicheTemplate> {
    let loader = get_template_loader();
    loader
        .find_compatible_templates(team_capabilities)
        .into_iter()
        .cloned()
        .collect()
}

/// Generate manifest for a template with customizations
pub fn generate_manifest(
    template: &NicheTemplate,
    customizations: &HashMap<String, String>,
) -> String {
    let mut manifest = template.manifest_template.clone();

    // Replace template variables
    for (key, value) in customizations {
        let placeholder = format!("{{{{{}}}}}", key);
        manifest = manifest.replace(&placeholder, value);
    }

    manifest
}

/// Validate template compatibility with available primals
pub fn validate_template_compatibility(template: &NicheTemplate) -> bool {
    let discovery = get_primal_discovery();
    let available_primals = discovery.get_all_primals();

    // Check if we have primals that support all required capabilities
    template.required_capabilities.iter().all(|required_cap| {
        available_primals
            .iter()
            .any(|primal| primal.capabilities.contains(required_cap))
    })
}
