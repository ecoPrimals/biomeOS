//! ISO Monitor - Live ISO Building Data Collection

use std::fs;
use std::path::Path;
use crate::views::iso_creator::{IsoConfig, NichePackage, CustomComponent, ComponentType, IsoTemplate, TemplateDifficulty, BootMode};

#[derive(Debug, Clone)]
pub struct IsoMonitor {
    iso_configs_path: String,
    niches_path: String,
    components_path: String,
    templates_path: String,
}

impl IsoMonitor {
    pub fn new() -> Self {
        Self {
            iso_configs_path: "/etc/biomeos/iso-configs".to_string(),
            niches_path: "/var/lib/biomeos/niches".to_string(),
            components_path: "/usr/share/biomeos/components".to_string(),
            templates_path: "/usr/share/biomeos/templates".to_string(),
        }
    }

    pub fn get_live_configs(&mut self) -> Vec<IsoConfig> {
        if let Ok(configs) = self.read_iso_configs() {
            if !configs.is_empty() {
                return configs;
            }
        }
        
        // Fallback to default configs
        self.get_default_configs()
    }

    pub fn get_live_niches(&mut self) -> Vec<NichePackage> {
        if let Ok(niches) = self.scan_available_niches() {
            if !niches.is_empty() {
                return niches;
            }
        }
        
        // Fallback to detected niches
        self.detect_system_niches()
    }

    pub fn get_live_components(&mut self) -> Vec<CustomComponent> {
        if let Ok(components) = self.scan_custom_components() {
            if !components.is_empty() {
                return components;
            }
        }
        
        // Fallback to system components
        self.detect_system_components()
    }

    pub fn get_live_templates(&mut self) -> Vec<IsoTemplate> {
        if let Ok(templates) = self.read_iso_templates() {
            if !templates.is_empty() {
                return templates;
            }
        }
        
        // Fallback to default templates
        self.get_default_templates()
    }

    fn read_iso_configs(&self) -> Result<Vec<IsoConfig>, Box<dyn std::error::Error>> {
        let configs_path = Path::new(&self.iso_configs_path);
        if !configs_path.exists() {
            return Ok(Vec::new());
        }

        let mut configs = Vec::new();
        
        for entry in fs::read_dir(configs_path)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("yaml") {
                let content = fs::read_to_string(entry.path())?;
                if let Ok(config) = serde_yaml::from_str::<IsoConfig>(&content) {
                    configs.push(config);
                }
            }
        }

        Ok(configs)
    }

    fn scan_available_niches(&self) -> Result<Vec<NichePackage>, Box<dyn std::error::Error>> {
        let niches_path = Path::new(&self.niches_path);
        if !niches_path.exists() {
            return Ok(Vec::new());
        }

        let mut niches = Vec::new();
        
        for entry in fs::read_dir(niches_path)? {
            let entry = entry?;
            if entry.path().is_dir() {
                let manifest_path = entry.path().join("manifest.yaml");
                if manifest_path.exists() {
                    let content = fs::read_to_string(manifest_path)?;
                    if let Ok(niche) = serde_yaml::from_str::<NichePackage>(&content) {
                        niches.push(niche);
                    }
                }
            }
        }

        Ok(niches)
    }

    fn scan_custom_components(&self) -> Result<Vec<CustomComponent>, Box<dyn std::error::Error>> {
        let components_path = Path::new(&self.components_path);
        if !components_path.exists() {
            return Ok(Vec::new());
        }

        let mut components = Vec::new();
        
        for entry in fs::read_dir(components_path)? {
            let entry = entry?;
            if entry.path().is_dir() {
                let component_name = entry.file_name().to_string_lossy().to_string();
                let size_mb = self.calculate_directory_size(&entry.path())?;
                
                components.push(CustomComponent {
                    name: component_name.clone(),
                    description: format!("Custom component: {}", component_name),
                    component_type: ComponentType::Library,
                    source_path: entry.path().to_string_lossy().to_string(),
                    destination_path: format!("/opt/{}", component_name),
                    size_mb,
                    required: false,
                });
            }
        }

        Ok(components)
    }

    fn read_iso_templates(&self) -> Result<Vec<IsoTemplate>, Box<dyn std::error::Error>> {
        let templates_path = Path::new(&self.templates_path);
        if !templates_path.exists() {
            return Ok(Vec::new());
        }

        let mut templates = Vec::new();
        
        for entry in fs::read_dir(templates_path)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("yaml") {
                let content = fs::read_to_string(entry.path())?;
                if let Ok(template) = serde_yaml::from_str::<IsoTemplate>(&content) {
                    templates.push(template);
                }
            }
        }

        Ok(templates)
    }

    fn get_default_configs(&self) -> Vec<IsoConfig> {
        vec![
            IsoConfig {
                name: "biomeOS-current".to_string(),
                description: "Current system configuration".to_string(),
                version: "1.0.0".to_string(),
                target_arch: "x86_64".to_string(),
                boot_mode: BootMode::Hybrid,
                included_primals: self.detect_installed_primals(),
                included_niches: Vec::new(),
                custom_components: Vec::new(),
                compression_level: 6,
                size_estimate: 1800,
                created_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
            },
        ]
    }

    fn detect_system_niches(&self) -> Vec<NichePackage> {
        let mut niches = Vec::new();
        
        // Check for running services that might be niches
        if let Ok(output) = std::process::Command::new("systemctl")
            .args(&["list-units", "--type=service", "--state=running"])
            .output() {
            let services = String::from_utf8_lossy(&output.stdout);
            
            for line in services.lines() {
                if line.contains("biomeos-niche-") {
                    if let Some(service_name) = line.split_whitespace().next() {
                        let niche_name = service_name.replace("biomeos-niche-", "").replace(".service", "");
                        niches.push(NichePackage {
                            id: niche_name.clone(),
                            name: niche_name.clone(),
                            description: format!("Detected niche: {}", niche_name),
                            author: "System".to_string(),
                            version: "1.0.0".to_string(),
                            category: "System".to_string(),
                            size_mb: 100,
                            features: vec!["Active service".to_string()],
                            dependencies: vec!["toadstool".to_string()],
                            manifest_path: format!("/etc/biomeos/niches/{}/manifest.yaml", niche_name),
                            icon_path: None,
                        });
                    }
                }
            }
        }

        // If no niches detected, provide some defaults based on system
        if niches.is_empty() {
            niches.push(NichePackage {
                id: "system-core".to_string(),
                name: "System Core".to_string(),
                description: "Core biomeOS functionality".to_string(),
                author: "biomeOS Team".to_string(),
                version: "1.0.0".to_string(),
                category: "Core".to_string(),
                size_mb: 200,
                features: vec!["System management".to_string(), "Core services".to_string()],
                dependencies: vec!["toadstool".to_string(), "nestgate".to_string()],
                manifest_path: "/etc/biomeos/core/manifest.yaml".to_string(),
                icon_path: None,
            });
        }

        niches
    }

    fn detect_system_components(&self) -> Vec<CustomComponent> {
        let mut components = Vec::new();
        
        // Check for common system components
        let potential_components = vec![
            ("/usr/local/bin", "Local Binaries", ComponentType::Binary),
            ("/opt", "Optional Software", ComponentType::Library),
            ("/usr/share/biomeos", "biomeOS Shared", ComponentType::Library),
        ];

        for (path, name, comp_type) in potential_components {
            if Path::new(path).exists() {
                if let Ok(size_mb) = self.calculate_directory_size(&Path::new(path).to_path_buf()) {
                    components.push(CustomComponent {
                        name: name.to_string(),
                        description: format!("System component at {}", path),
                        component_type: comp_type,
                        source_path: path.to_string(),
                        destination_path: path.to_string(),
                        size_mb,
                        required: false,
                    });
                }
            }
        }

        components
    }

    fn get_default_templates(&self) -> Vec<IsoTemplate> {
        vec![
            IsoTemplate {
                name: "Minimal biomeOS".to_string(),
                description: "Lightweight biomeOS with core functionality".to_string(),
                use_case: "Edge computing, minimal installations".to_string(),
                included_components: vec!["core".to_string()],
                size_estimate: 800,
                difficulty: TemplateDifficulty::Beginner,
            },
            IsoTemplate {
                name: "Full biomeOS".to_string(),
                description: "Complete biomeOS with all primals".to_string(),
                use_case: "Development, full functionality".to_string(),
                included_components: vec!["all-primals".to_string()],
                size_estimate: 2500,
                difficulty: TemplateDifficulty::Intermediate,
            },
        ]
    }

    fn detect_installed_primals(&self) -> Vec<String> {
        let mut primals = Vec::new();
        
        // Check for primal binaries
        let potential_primals = vec!["toadstool", "songbird", "nestgate", "squirrel", "beardog"];
        
        for primal in potential_primals {
            if std::process::Command::new("which").arg(primal).output().is_ok() {
                primals.push(primal.to_string());
            }
        }

        // Always include toadstool as it's core
        if !primals.contains(&"toadstool".to_string()) {
            primals.push("toadstool".to_string());
        }

        primals
    }

    fn calculate_directory_size(&self, path: &Path) -> Result<u32, Box<dyn std::error::Error>> {
        let output = std::process::Command::new("du")
            .args(&["-sm", &path.to_string_lossy()])
            .output()?;
        
        let size_str = String::from_utf8_lossy(&output.stdout);
        let size_mb = size_str
            .split_whitespace()
            .next()
            .unwrap_or("0")
            .parse::<u32>()
            .unwrap_or(0);
        
        Ok(size_mb)
    }
} 