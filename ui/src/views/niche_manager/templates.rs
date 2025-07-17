//! Template management functionality for the Niche Manager
//!
//! This module handles niche templates, providing pre-built configurations
//! and examples for common niche patterns.

use crate::views::niche_manager::types::*;
use std::collections::HashMap;

/// Template management functionality
pub struct TemplateManager;

impl TemplateManager {
    /// Get all available niche templates
    pub fn get_templates() -> Vec<NicheTemplate> {
        vec![
            NicheTemplate {
                id: "basic-web-app".to_string(),
                name: "Basic Web Application".to_string(),
                description: "Simple web application template with frontend and backend"
                    .to_string(),
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
                        param_type: ParameterType::Choice(vec![
                            "react".to_string(),
                            "vue".to_string(),
                            "angular".to_string(),
                        ]),
                        required: true,
                        default_value: Some("react".to_string()),
                        validation: None,
                    },
                ],
                examples: vec![TemplateExample {
                    name: "React E-commerce".to_string(),
                    description: "E-commerce site with React frontend".to_string(),
                    parameters: {
                        let mut params = HashMap::new();
                        params.insert("app_name".to_string(), "ecommerce-store".to_string());
                        params.insert("framework".to_string(), "react".to_string());
                        params
                    },
                }],
            },
            NicheTemplate {
                id: "gaming-server".to_string(),
                name: "Gaming Server".to_string(),
                description: "Dedicated gaming server with matchmaking and statistics".to_string(),
                category: NicheCategory::Gaming,
                difficulty: NicheDifficulty::Intermediate,
                template_yaml: Self::get_gaming_server_template(),
                parameters: vec![
                    TemplateParameter {
                        name: "game_type".to_string(),
                        description: "Type of game server".to_string(),
                        param_type: ParameterType::Choice(vec![
                            "fps".to_string(),
                            "mmo".to_string(),
                            "rts".to_string(),
                        ]),
                        required: true,
                        default_value: Some("fps".to_string()),
                        validation: None,
                    },
                    TemplateParameter {
                        name: "max_players".to_string(),
                        description: "Maximum number of players".to_string(),
                        param_type: ParameterType::Number,
                        required: true,
                        default_value: Some("32".to_string()),
                        validation: Some("^[1-9][0-9]*$".to_string()),
                    },
                ],
                examples: vec![TemplateExample {
                    name: "FPS Tournament".to_string(),
                    description: "Tournament-ready FPS server".to_string(),
                    parameters: {
                        let mut params = HashMap::new();
                        params.insert("game_type".to_string(), "fps".to_string());
                        params.insert("max_players".to_string(), "64".to_string());
                        params
                    },
                }],
            },
            NicheTemplate {
                id: "data-pipeline".to_string(),
                name: "Data Processing Pipeline".to_string(),
                description: "ETL pipeline for data processing and analytics".to_string(),
                category: NicheCategory::Research,
                difficulty: NicheDifficulty::Advanced,
                template_yaml: Self::get_data_pipeline_template(),
                parameters: vec![
                    TemplateParameter {
                        name: "data_source".to_string(),
                        description: "Primary data source type".to_string(),
                        param_type: ParameterType::Choice(vec![
                            "database".to_string(),
                            "api".to_string(),
                            "files".to_string(),
                        ]),
                        required: true,
                        default_value: Some("database".to_string()),
                        validation: None,
                    },
                    TemplateParameter {
                        name: "processing_framework".to_string(),
                        description: "Data processing framework".to_string(),
                        param_type: ParameterType::Choice(vec![
                            "spark".to_string(),
                            "airflow".to_string(),
                            "kafka".to_string(),
                        ]),
                        required: true,
                        default_value: Some("spark".to_string()),
                        validation: None,
                    },
                ],
                examples: vec![TemplateExample {
                    name: "Real-time Analytics".to_string(),
                    description: "Real-time data processing with Kafka".to_string(),
                    parameters: {
                        let mut params = HashMap::new();
                        params.insert("data_source".to_string(), "api".to_string());
                        params.insert("processing_framework".to_string(), "kafka".to_string());
                        params
                    },
                }],
            },
        ]
    }

    /// Get web application template YAML
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
    resources:
      cpu: 2.0
      memory: 2GB
      storage: 20GB
    
  backend:
    primal: toadstool
    runtime: container
    image: "node:18-alpine"
    ports:
      - "8080:8080"
    resources:
      cpu: 2.0
      memory: 4GB
      storage: 30GB
      
  database:
    primal: nestgate
    runtime: container
    image: "postgres:15"
    ports:
      - "5432:5432"
    resources:
      cpu: 2.0
      memory: 8GB
      storage: 100GB
      
networking:
  load_balancing: true
  service_discovery: true
  ingress_enabled: true
  
security:
  network_policies: true
  resource_quotas: true
  encryption_in_transit: true
"#
        .to_string()
    }

    /// Get gaming server template YAML
    fn get_gaming_server_template() -> String {
        r#"# Gaming Server Niche Template
apiVersion: biomeOS/v1
kind: Niche
metadata:
  name: "{{game_type}}-server"
  description: "{{game_type}} gaming server for {{max_players}} players"
  
services:
  game-server:
    primal: toadstool
    runtime: container
    image: "gameserver:latest"
    ports:
      - "7777:7777"
      - "27015:27015"
    environment:
      MAX_PLAYERS: "{{max_players}}"
      GAME_TYPE: "{{game_type}}"
    resources:
      cpu: 4.0
      memory: 16GB
      storage: 200GB
      
  matchmaking:
    primal: squirrel
    runtime: container
    image: "matchmaking:latest"
    ports:
      - "8080:8080"
    resources:
      cpu: 2.0
      memory: 4GB
      storage: 50GB
      
  statistics:
    primal: nestgate
    runtime: container
    image: "postgres:15"
    ports:
      - "5432:5432"
    resources:
      cpu: 2.0
      memory: 8GB
      storage: 500GB
      
networking:
  high_bandwidth: true
  service_discovery: true
  
security:
  network_policies: true
  resource_quotas: true
"#
        .to_string()
    }

    /// Get data pipeline template YAML
    fn get_data_pipeline_template() -> String {
        r#"# Data Processing Pipeline Niche Template
apiVersion: biomeOS/v1
kind: Niche
metadata:
  name: "data-pipeline-{{processing_framework}}"
  description: "Data processing pipeline using {{processing_framework}}"
  
services:
  data-processor:
    primal: squirrel
    runtime: container
    image: "{{processing_framework}}:latest"
    ports:
      - "8080:8080"
    environment:
      DATA_SOURCE: "{{data_source}}"
      PROCESSING_FRAMEWORK: "{{processing_framework}}"
    resources:
      cpu: 8.0
      memory: 32GB
      storage: 1TB
      
  data-storage:
    primal: nestgate
    runtime: container
    image: "postgres:15"
    ports:
      - "5432:5432"
    resources:
      cpu: 4.0
      memory: 16GB
      storage: 2TB
      
  monitoring:
    primal: songbird
    runtime: container
    image: "prometheus:latest"
    ports:
      - "9090:9090"
    resources:
      cpu: 2.0
      memory: 4GB
      storage: 100GB
      
networking:
  high_bandwidth: true
  service_discovery: true
  
security:
  network_policies: true
  resource_quotas: true
  encryption_at_rest: true
"#
        .to_string()
    }

    /// Get default niche YAML for new niches
    pub fn get_default_niche_yaml() -> String {
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
      storage: 10GB
      
networking:
  load_balancing: true
  service_discovery: true
  
security:
  network_policies: true
  resource_quotas: true
"#
        .to_string()
    }

    /// Generate a niche from a template with parameters
    pub fn generate_from_template(
        template: &NicheTemplate,
        parameters: &HashMap<String, String>,
    ) -> Result<String, String> {
        let mut yaml = template.template_yaml.clone();

        // Replace template parameters
        for (key, value) in parameters {
            let placeholder = format!("{{{{{}}}}}", key);
            yaml = yaml.replace(&placeholder, value);
        }

        // Validate that all required parameters were provided
        for param in &template.parameters {
            if param.required {
                let placeholder = format!("{{{{{}}}}}", param.name);
                if yaml.contains(&placeholder) {
                    return Err(format!("Required parameter '{}' not provided", param.name));
                }
            }
        }

        Ok(yaml)
    }

    /// Validate template parameters
    pub fn validate_parameters(
        template: &NicheTemplate,
        parameters: &HashMap<String, String>,
    ) -> Vec<String> {
        let mut errors = Vec::new();

        for param in &template.parameters {
            if param.required && !parameters.contains_key(&param.name) {
                errors.push(format!("Required parameter '{}' is missing", param.name));
                continue;
            }

            if let Some(value) = parameters.get(&param.name) {
                if let Some(validation) = &param.validation {
                    // Basic regex validation (in a real implementation, use a proper regex crate)
                    if validation.contains("^[a-z][a-z0-9-]*$")
                        && !value
                            .chars()
                            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-')
                    {
                        errors.push(format!("Parameter '{}' must contain only lowercase letters, numbers, and hyphens", param.name));
                    }
                    if validation.contains("^[1-9][0-9]*$") && value.parse::<u32>().is_err() {
                        errors.push(format!(
                            "Parameter '{}' must be a positive integer",
                            param.name
                        ));
                    }
                }

                // Validate choice parameters
                if let ParameterType::Choice(choices) = &param.param_type {
                    if !choices.contains(value) {
                        errors.push(format!(
                            "Parameter '{}' must be one of: {}",
                            param.name,
                            choices.join(", ")
                        ));
                    }
                }
            }
        }

        errors
    }
}
