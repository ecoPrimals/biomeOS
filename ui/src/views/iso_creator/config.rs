//! Configuration management for ISO Creator
//!
//! This module handles loading, saving, and managing ISO configurations,
//! including validation and preset management.

use crate::views::iso_creator::types::*;
use std::collections::HashMap;
use std::path::Path;

/// Configuration manager for ISO Creator
pub struct ConfigManager {
    configs: Vec<IsoConfig>,
    templates: Vec<IsoTemplate>,
    default_config: IsoCreatorConfig,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        Self {
            configs: Vec::new(),
            templates: Vec::new(),
            default_config: IsoCreatorConfig::default(),
        }
    }

    /// Load configurations from storage
    pub fn load_configurations(&mut self) -> Result<(), String> {
        // In a real implementation, this would load from files
        self.configs = Self::get_default_configs();
        self.templates = Self::get_default_templates();
        Ok(())
    }

    /// Save configuration to storage
    pub fn save_configuration(&mut self, config: IsoConfig) -> Result<(), String> {
        // Validate configuration before saving
        config.validate()?;
        
        // Check for duplicate names
        if self.configs.iter().any(|c| c.name == config.name) {
            return Err(format!("Configuration '{}' already exists", config.name));
        }

        self.configs.push(config);
        // In a real implementation, persist to file system
        Ok(())
    }

    /// Update existing configuration
    pub fn update_configuration(&mut self, index: usize, config: IsoConfig) -> Result<(), String> {
        config.validate()?;
        
        if index >= self.configs.len() {
            return Err("Configuration index out of bounds".to_string());
        }

        self.configs[index] = config;
        // In a real implementation, persist to file system
        Ok(())
    }

    /// Delete configuration
    pub fn delete_configuration(&mut self, index: usize) -> Result<(), String> {
        if index >= self.configs.len() {
            return Err("Configuration index out of bounds".to_string());
        }

        self.configs.remove(index);
        // In a real implementation, update file system
        Ok(())
    }

    /// Get all configurations
    pub fn get_configurations(&self) -> &[IsoConfig] {
        &self.configs
    }

    /// Get configuration by name
    pub fn get_configuration_by_name(&self, name: &str) -> Option<&IsoConfig> {
        self.configs.iter().find(|c| c.name == name)
    }

    /// Get all templates
    pub fn get_templates(&self) -> &[IsoTemplate] {
        &self.templates
    }

    /// Create configuration from template
    pub fn create_from_template(&self, template_name: &str, config_name: String) -> Result<IsoConfig, String> {
        let template = self.templates.iter()
            .find(|t| t.name == template_name)
            .ok_or("Template not found")?;

        let mut config = IsoConfig::new(config_name, template.description.clone());
        config.included_primals = template.included_components.clone();
        config.size_estimate = template.size_estimate;
        
        // Apply template-specific settings
        match template.use_case.to_lowercase().as_str() {
            s if s.contains("gaming") => {
                config.boot_mode = BootMode::UEFI;
                config.compression_level = 7;
                config.included_niches = vec!["gaming-tournament".to_string()];
            },
            s if s.contains("research") => {
                config.boot_mode = BootMode::Hybrid;
                config.compression_level = 5;
                config.included_niches = vec!["ai-research".to_string()];
            },
            s if s.contains("development") => {
                config.boot_mode = BootMode::Hybrid;
                config.compression_level = 6;
                config.included_niches = vec!["web-development".to_string()];
            },
            s if s.contains("minimal") => {
                config.boot_mode = BootMode::Legacy;
                config.compression_level = 9;
                config.included_primals = vec!["toadstool".to_string()];
            },
            _ => {
                // Use defaults
            }
        }

        Ok(config)
    }

    /// Get default configurations
    fn get_default_configs() -> Vec<IsoConfig> {
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
            IsoConfig {
                name: "biomeOS-development".to_string(),
                description: "Full-stack development environment".to_string(),
                version: "1.1.0".to_string(),
                target_arch: "x86_64".to_string(),
                boot_mode: BootMode::Hybrid,
                included_primals: vec![
                    "toadstool".to_string(),
                    "songbird".to_string(),
                    "nestgate".to_string(),
                    "squirrel".to_string(),
                ],
                included_niches: vec!["web-development".to_string(), "mobile-development".to_string()],
                custom_components: vec!["docker-tools".to_string(), "nodejs-runtime".to_string()],
                compression_level: 6,
                size_estimate: 2800,
                created_at: "2024-01-12".to_string(),
            },
            IsoConfig {
                name: "biomeOS-minimal".to_string(),
                description: "Minimal biomeOS installation".to_string(),
                version: "1.0.0".to_string(),
                target_arch: "x86_64".to_string(),
                boot_mode: BootMode::Legacy,
                included_primals: vec!["toadstool".to_string()],
                included_niches: Vec::new(),
                custom_components: Vec::new(),
                compression_level: 9,
                size_estimate: 800,
                created_at: "2024-01-15".to_string(),
            },
        ]
    }

    /// Get default templates
    fn get_default_templates() -> Vec<IsoTemplate> {
        vec![
            IsoTemplate {
                name: "Gaming Server".to_string(),
                description: "High-performance gaming server with tournament support".to_string(),
                use_case: "gaming".to_string(),
                included_components: vec![
                    "toadstool".to_string(),
                    "songbird".to_string(),
                    "nestgate".to_string(),
                ],
                size_estimate: 2100,
                difficulty: TemplateDifficulty::Intermediate,
                tags: vec!["gaming".to_string(), "performance".to_string()],
                author: "biomeOS Gaming Team".to_string(),
                version: "1.0.0".to_string(),
            },
            IsoTemplate {
                name: "AI Research Platform".to_string(),
                description: "Complete AI research environment with GPU support".to_string(),
                use_case: "research".to_string(),
                included_components: vec![
                    "toadstool".to_string(),
                    "squirrel".to_string(),
                    "nestgate".to_string(),
                ],
                size_estimate: 3500,
                difficulty: TemplateDifficulty::Advanced,
                tags: vec!["ai".to_string(), "research".to_string(), "gpu".to_string()],
                author: "biomeOS Research Team".to_string(),
                version: "2.0.0".to_string(),
            },
            IsoTemplate {
                name: "Web Development Suite".to_string(),
                description: "Full-stack web development environment".to_string(),
                use_case: "development".to_string(),
                included_components: vec![
                    "toadstool".to_string(),
                    "songbird".to_string(),
                    "nestgate".to_string(),
                    "squirrel".to_string(),
                ],
                size_estimate: 2800,
                difficulty: TemplateDifficulty::Intermediate,
                tags: vec!["web".to_string(), "development".to_string(), "fullstack".to_string()],
                author: "biomeOS Dev Team".to_string(),
                version: "1.5.0".to_string(),
            },
            IsoTemplate {
                name: "Minimal Installation".to_string(),
                description: "Bare-bones biomeOS installation".to_string(),
                use_case: "minimal".to_string(),
                included_components: vec!["toadstool".to_string()],
                size_estimate: 800,
                difficulty: TemplateDifficulty::Beginner,
                tags: vec!["minimal".to_string(), "lightweight".to_string()],
                author: "biomeOS Core Team".to_string(),
                version: "1.0.0".to_string(),
            },
            IsoTemplate {
                name: "Desktop Environment".to_string(),
                description: "Complete desktop environment with GUI applications".to_string(),
                use_case: "desktop".to_string(),
                included_components: vec![
                    "toadstool".to_string(),
                    "songbird".to_string(),
                    "nestgate".to_string(),
                    "squirrel".to_string(),
                    "beardog".to_string(),
                ],
                size_estimate: 4200,
                difficulty: TemplateDifficulty::Advanced,
                tags: vec!["desktop".to_string(), "gui".to_string(), "applications".to_string()],
                author: "biomeOS Desktop Team".to_string(),
                version: "1.3.0".to_string(),
            },
            IsoTemplate {
                name: "Server Infrastructure".to_string(),
                description: "Enterprise server infrastructure platform".to_string(),
                use_case: "server".to_string(),
                included_components: vec![
                    "toadstool".to_string(),
                    "songbird".to_string(),
                    "nestgate".to_string(),
                    "beardog".to_string(),
                ],
                size_estimate: 3200,
                difficulty: TemplateDifficulty::Expert,
                tags: vec!["server".to_string(), "enterprise".to_string(), "infrastructure".to_string()],
                author: "biomeOS Enterprise Team".to_string(),
                version: "2.1.0".to_string(),
            },
        ]
    }

    /// Export configuration to file
    pub fn export_configuration(&self, config: &IsoConfig, path: &Path) -> Result<(), String> {
        let config_json = serde_json::to_string_pretty(config)
            .map_err(|e| format!("Failed to serialize configuration: {}", e))?;
        
        std::fs::write(path, config_json)
            .map_err(|e| format!("Failed to write configuration file: {}", e))?;
        
        Ok(())
    }

    /// Import configuration from file
    pub fn import_configuration(&mut self, path: &Path) -> Result<IsoConfig, String> {
        let config_json = std::fs::read_to_string(path)
            .map_err(|e| format!("Failed to read configuration file: {}", e))?;
        
        let config: IsoConfig = serde_json::from_str(&config_json)
            .map_err(|e| format!("Failed to parse configuration file: {}", e))?;
        
        config.validate()?;
        Ok(config)
    }

    /// Get configuration suggestions based on use case
    pub fn get_suggestions(&self, use_case: &str) -> Vec<String> {
        let mut suggestions = Vec::new();
        
        match use_case.to_lowercase().as_str() {
            "gaming" => {
                suggestions.push("Enable UEFI boot for faster startup".to_string());
                suggestions.push("Include gaming-tournament niche".to_string());
                suggestions.push("Use compression level 7 for balance".to_string());
                suggestions.push("Include GPU drivers and tools".to_string());
            },
            "research" => {
                suggestions.push("Include AI research niche".to_string());
                suggestions.push("Enable hybrid boot for compatibility".to_string());
                suggestions.push("Use lower compression for faster extraction".to_string());
                suggestions.push("Include CUDA toolkit for GPU computing".to_string());
            },
            "development" => {
                suggestions.push("Include web development niche".to_string());
                suggestions.push("Enable hybrid boot mode".to_string());
                suggestions.push("Include Docker and containers".to_string());
                suggestions.push("Add development tools and IDEs".to_string());
            },
            "minimal" => {
                suggestions.push("Include only Toadstool primal".to_string());
                suggestions.push("Use maximum compression".to_string());
                suggestions.push("Enable Legacy boot for older hardware".to_string());
                suggestions.push("Exclude documentation and demos".to_string());
            },
            _ => {
                suggestions.push("Consider your primary use case".to_string());
                suggestions.push("Include all primals for full functionality".to_string());
                suggestions.push("Use hybrid boot for maximum compatibility".to_string());
            }
        }
        
        suggestions
    }

    /// Validate configuration compatibility
    pub fn validate_compatibility(&self, config: &IsoConfig) -> Result<(), String> {
        // Check primal dependencies
        for niche in &config.included_niches {
            match niche.as_str() {
                "gaming-tournament" => {
                    if !config.included_primals.contains(&"toadstool".to_string()) ||
                       !config.included_primals.contains(&"songbird".to_string()) {
                        return Err("Gaming tournament niche requires Toadstool and Songbird primals".to_string());
                    }
                },
                "ai-research" => {
                    if !config.included_primals.contains(&"squirrel".to_string()) ||
                       !config.included_primals.contains(&"nestgate".to_string()) {
                        return Err("AI research niche requires Squirrel and NestGate primals".to_string());
                    }
                },
                "web-development" => {
                    if !config.included_primals.contains(&"songbird".to_string()) {
                        return Err("Web development niche requires Songbird primal".to_string());
                    }
                },
                _ => {}
            }
        }

        // Check architecture compatibility
        if config.target_arch != "x86_64" && config.target_arch != "aarch64" {
            return Err("Unsupported target architecture".to_string());
        }

        // Check size limits
        if config.size_estimate > 10000 { // 10GB limit
            return Err("Configuration exceeds maximum size limit (10GB)".to_string());
        }

        Ok(())
    }

    /// Get size breakdown for configuration
    pub fn get_size_breakdown(&self, config: &IsoConfig) -> HashMap<String, u64> {
        let mut breakdown = HashMap::new();
        
        breakdown.insert("Base OS".to_string(), 500);
        
        for primal in &config.included_primals {
            let size = match primal.as_str() {
                "toadstool" => 200,
                "songbird" => 150,
                "nestgate" => 300,
                "squirrel" => 100,
                "beardog" => 250,
                _ => 100,
            };
            breakdown.insert(format!("Primal: {}", primal), size);
        }
        
        for niche in &config.included_niches {
            let size = match niche.as_str() {
                "gaming-tournament" => 450,
                "ai-research" => 1200,
                "web-development" => 800,
                _ => 400,
            };
            breakdown.insert(format!("Niche: {}", niche), size);
        }
        
        for component in &config.custom_components {
            breakdown.insert(format!("Component: {}", component), 150);
        }
        
        breakdown
    }
} 