//! Niche Monitor - Live Niche Package Data Collection

use crate::views::niche_manager::{
    MarketplaceNiche, NicheCategory, NicheDifficulty, NichePackage, NicheStatus, NicheTemplate,
};
use std::fs;
use std::path::Path;

#[derive(Debug, Clone)]
pub struct NicheMonitor {
    local_niches_path: String,
    templates_path: String,
    marketplace_cache_path: String,
}

impl NicheMonitor {
    pub fn new() -> Self {
        Self {
            local_niches_path: "/var/lib/biomeos/niches".to_string(),
            templates_path: "/usr/share/biomeos/niche-templates".to_string(),
            marketplace_cache_path: "/tmp/biomeos-marketplace.json".to_string(),
        }
    }

    pub fn get_live_niches(&mut self) -> Vec<NichePackage> {
        if let Ok(niches) = self.scan_local_niches() {
            if !niches.is_empty() {
                return niches;
            }
        }

        // Fallback to system detection
        self.detect_running_niches()
    }

    pub fn get_live_templates(&mut self) -> Vec<NicheTemplate> {
        if let Ok(templates) = self.read_niche_templates() {
            if !templates.is_empty() {
                return templates;
            }
        }

        // Fallback to default templates
        self.get_default_templates()
    }

    pub fn get_live_marketplace(&mut self) -> Vec<MarketplaceNiche> {
        if let Ok(marketplace) = self.read_marketplace_cache() {
            if !marketplace.is_empty() {
                return marketplace;
            }
        }

        // Fallback to simulated marketplace
        self.get_simulated_marketplace()
    }

    fn scan_local_niches(&self) -> Result<Vec<NichePackage>, Box<dyn std::error::Error>> {
        let niches_path = Path::new(&self.local_niches_path);
        if !niches_path.exists() {
            return Ok(Vec::new());
        }

        let mut niches = Vec::new();

        for entry in fs::read_dir(niches_path)? {
            let entry = entry?;
            if entry.path().is_dir() {
                let manifest_path = entry.path().join("niche.yaml");
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

    fn read_niche_templates(&self) -> Result<Vec<NicheTemplate>, Box<dyn std::error::Error>> {
        let templates_path = Path::new(&self.templates_path);
        if !templates_path.exists() {
            return Ok(Vec::new());
        }

        let mut templates = Vec::new();

        for entry in fs::read_dir(templates_path)? {
            let entry = entry?;
            if entry.path().extension().and_then(|s| s.to_str()) == Some("yaml") {
                let content = fs::read_to_string(entry.path())?;
                if let Ok(template) = serde_yaml::from_str::<NicheTemplate>(&content) {
                    templates.push(template);
                }
            }
        }

        Ok(templates)
    }

    fn read_marketplace_cache(&self) -> Result<Vec<MarketplaceNiche>, Box<dyn std::error::Error>> {
        let cache_path = Path::new(&self.marketplace_cache_path);
        if !cache_path.exists() {
            return Ok(Vec::new());
        }

        let content = fs::read_to_string(cache_path)?;
        let marketplace: Vec<MarketplaceNiche> = serde_json::from_str(&content)?;

        Ok(marketplace)
    }

    fn detect_running_niches(&self) -> Vec<NichePackage> {
        let mut niches = Vec::new();

        // Check for running containers that might be niches
        if let Ok(output) = std::process::Command::new("docker")
            .args(&["ps", "--format", "{{.Names}}:{{.Image}}"])
            .output()
        {
            let containers = String::from_utf8_lossy(&output.stdout);

            for line in containers.lines() {
                if line.contains("biomeos-niche") {
                    let parts: Vec<&str> = line.split(':').collect();
                    if parts.len() >= 2 {
                        let niche_name = parts[0].replace("biomeos-niche-", "");
                        niches.push(NichePackage {
                            id: niche_name.clone(),
                            name: niche_name.clone(),
                            description: format!("Running niche: {}", niche_name),
                            author: "System".to_string(),
                            version: "1.0.0".to_string(),
                            category: NicheCategory::Development,
                            difficulty: NicheDifficulty::Intermediate,
                            tags: vec!["running".to_string(), "container".to_string()],
                            features: vec!["Active deployment".to_string()],
                            requirements: Default::default(),
                            manifest_path: format!(
                                "/var/lib/biomeos/niches/{}/niche.yaml",
                                niche_name
                            ),
                            icon_path: None,
                            size_mb: 500,
                            downloads: 0,
                            rating: 4.0,
                            created_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                            updated_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                            status: NicheStatus::Published,
                        });
                    }
                }
            }
        }

        // Check for systemd services
        if let Ok(output) = std::process::Command::new("systemctl")
            .args(&["list-units", "--type=service", "--state=running"])
            .output()
        {
            let services = String::from_utf8_lossy(&output.stdout);

            for line in services.lines() {
                if line.contains("biomeos-niche-") {
                    if let Some(service_name) = line.split_whitespace().next() {
                        let niche_name = service_name
                            .replace("biomeos-niche-", "")
                            .replace(".service", "");
                        niches.push(NichePackage {
                            id: niche_name.clone(),
                            name: niche_name.clone(),
                            description: format!("System service niche: {}", niche_name),
                            author: "System".to_string(),
                            version: "1.0.0".to_string(),
                            category: NicheCategory::Development,
                            difficulty: NicheDifficulty::Advanced,
                            tags: vec!["service".to_string(), "system".to_string()],
                            features: vec!["System integration".to_string()],
                            requirements: Default::default(),
                            manifest_path: format!(
                                "/etc/systemd/system/biomeos-niche-{}.service",
                                niche_name
                            ),
                            icon_path: None,
                            size_mb: 100,
                            downloads: 0,
                            rating: 4.5,
                            created_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                            updated_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                            status: NicheStatus::Published,
                        });
                    }
                }
            }
        }

        // If no niches detected, provide a default based on current system
        if niches.is_empty() {
            niches.push(NichePackage {
                id: "system-ui".to_string(),
                name: "biomeOS UI".to_string(),
                description: "Current biomeOS user interface".to_string(),
                author: "biomeOS Team".to_string(),
                version: "1.0.0".to_string(),
                category: NicheCategory::Development,
                difficulty: NicheDifficulty::Beginner,
                tags: vec!["ui".to_string(), "system".to_string()],
                features: vec![
                    "Dashboard".to_string(),
                    "BYOB management".to_string(),
                    "ISO creation".to_string(),
                ],
                requirements: Default::default(),
                manifest_path: "/etc/biomeos/ui/niche.yaml".to_string(),
                icon_path: None,
                size_mb: 50,
                downloads: 1,
                rating: 5.0,
                created_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                updated_at: chrono::Utc::now().format("%Y-%m-%d").to_string(),
                status: NicheStatus::Published,
            });
        }

        niches
    }

    fn get_default_templates(&self) -> Vec<NicheTemplate> {
        vec![
            // Templates would be defined here based on actual system templates
        ]
    }

    fn get_simulated_marketplace(&self) -> Vec<MarketplaceNiche> {
        // Return empty for now - would connect to real marketplace
        Vec::new()
    }
}
