//! ISO Creator Mock Data Provider
//!
//! This module provides mock data specifically for the ISO Creator system,
//! including niche packages, custom components, build jobs, and templates.

use crate::views::iso_creator::types::*;
use super::common::UnifiedNichePackage;

// Mock-specific types for ISO creator
#[derive(Debug, Clone)]
pub struct AdvancedSettings {
    pub kernel_params: Vec<String>,
    pub custom_scripts: Vec<String>,
}

/// ISO Creator-specific mock data provider
pub struct IsoCreatorMockProvider;

/// Static instance of the ISO creator mock provider
pub static ISO_CREATOR_MOCK_PROVIDER: IsoCreatorMockProvider = IsoCreatorMockProvider;

impl IsoCreatorMockProvider {
    /// Get mock niche packages converted from unified format
    pub fn get_niches(&self) -> Vec<NichePackage> {
        super::common::get_unified_niches()
            .into_iter()
            .map(|unified| self.convert_unified_niche(unified))
            .collect()
    }

    /// Get mock custom components
    pub fn get_custom_components(&self) -> Vec<CustomComponent> {
        vec![
            CustomComponent {
                id: "custom-security-scanner".to_string(),
                name: "Security Scanner".to_string(),
                description: "Advanced security vulnerability scanner with real-time monitoring".to_string(),
                component_type: ComponentType::Security,
                version: "2.1.0".to_string(),
                author: "SecureTeam".to_string(),
                size_mb: 125,
                dependencies: vec!["beardog".to_string()],
                config_schema: Some(r#"
                {
                    "scan_frequency": {
                        "type": "string",
                        "enum": ["hourly", "daily", "weekly"],
                        "default": "daily"
                    },
                    "severity_threshold": {
                        "type": "string", 
                        "enum": ["low", "medium", "high", "critical"],
                        "default": "medium"
                    }
                }
                "#.to_string()),
                installation_script: Some("install-security-scanner.sh".to_string()),
            },
            CustomComponent {
                id: "custom-monitoring-dashboard".to_string(),
                name: "Monitoring Dashboard".to_string(),
                description: "Real-time system monitoring and alerting dashboard".to_string(),
                component_type: ComponentType::Monitoring,
                version: "1.8.3".to_string(),
                author: "MonitoringPro".to_string(),
                size_mb: 89,
                dependencies: vec!["songbird".to_string(), "nestgate".to_string()],
                config_schema: Some(r#"
                {
                    "refresh_interval": {
                        "type": "integer",
                        "minimum": 5,
                        "maximum": 300,
                        "default": 30
                    },
                    "alert_channels": {
                        "type": "array",
                        "items": {"type": "string"},
                        "default": ["email", "slack"]
                    }
                }
                "#.to_string()),
                installation_script: None,
            },
        ]
    }

    /// Get mock build jobs
    pub fn get_build_jobs(&self) -> Vec<BuildJob> {
        vec![
            BuildJob {
                id: "build-gaming-iso".to_string(),
                name: "Gaming Tournament ISO".to_string(),
                description: "Building ISO for gaming tournament platform".to_string(),
                status: BuildStatus::InProgress,
                progress: 67.5,
                created_at: chrono::Utc::now() - chrono::Duration::minutes(45),
                started_at: Some(chrono::Utc::now() - chrono::Duration::minutes(42)),
                estimated_completion: Some(chrono::Utc::now() + chrono::Duration::minutes(15)),
                config: IsoConfig {
                    name: "gaming-tournament-iso".to_string(),
                    version: "1.5.0".to_string(),
                    description: Some("Tournament gaming platform".to_string()),
                    boot_mode: BootMode::UEFI,
                    architecture: Architecture::X86_64,
                    base_image: "ubuntu-22.04-minimal".to_string(),
                    size_limit_mb: Some(4096),
                    selected_niches: vec!["gaming-tournament".to_string()],
                    custom_components: vec!["custom-monitoring-dashboard".to_string()],
                    advanced_settings: AdvancedSettings {
                        compression_level: CompressionLevel::High,
                        include_source_packages: false,
                        enable_secure_boot: true,
                        custom_kernel_params: vec!["quiet".to_string(), "splash".to_string()],
                        post_install_scripts: vec![],
                    },
                },
                build_log: vec![
                    "Starting ISO build process...".to_string(),
                    "Downloading base image ubuntu-22.04-minimal...".to_string(),
                    "Installing gaming-tournament niche...".to_string(),
                    "Configuring toadstool compute services...".to_string(),
                    "Setting up songbird networking...".to_string(),
                    "Building custom components...".to_string(),
                    "Compressing filesystem...".to_string(),
                ],
            },
            BuildJob {
                id: "build-ai-research-iso".to_string(),
                name: "AI Research ISO".to_string(),
                description: "Building ISO for AI research platform".to_string(),
                status: BuildStatus::Completed,
                progress: 100.0,
                created_at: chrono::Utc::now() - chrono::Duration::hours(2),
                started_at: Some(chrono::Utc::now() - chrono::Duration::hours(1) - chrono::Duration::minutes(45)),
                estimated_completion: None,
                config: IsoConfig {
                    name: "ai-research-iso".to_string(),
                    version: "2.1.0".to_string(),
                    description: Some("AI research and ML platform".to_string()),
                    boot_mode: BootMode::UEFI,
                    architecture: Architecture::X86_64,
                    base_image: "ubuntu-22.04-server".to_string(),
                    size_limit_mb: Some(8192),
                    selected_niches: vec!["ai-research".to_string()],
                    custom_components: vec!["custom-security-scanner".to_string()],
                    advanced_settings: AdvancedSettings {
                        compression_level: CompressionLevel::Medium,
                        include_source_packages: true,
                        enable_secure_boot: true,
                        custom_kernel_params: vec!["nvidia-modeset=1".to_string()],
                        post_install_scripts: vec!["setup-cuda.sh".to_string()],
                    },
                },
                build_log: vec![
                    "Build completed successfully!".to_string(),
                    "ISO size: 7.2GB".to_string(),
                    "Output: /builds/ai-research-iso-2.1.0.iso".to_string(),
                ],
            },
        ]
    }

    /// Get mock ISO templates
    pub fn get_templates(&self) -> Vec<IsoTemplate> {
        vec![
            IsoTemplate {
                id: "gaming-optimized".to_string(),
                name: "Gaming Optimized".to_string(),
                description: "Pre-configured template for gaming environments with low-latency networking and GPU support".to_string(),
                category: "Gaming".to_string(),
                author: "BiomeOS Team".to_string(),
                version: "1.0.0".to_string(),
                base_config: IsoConfig {
                    name: "gaming-template".to_string(),
                    version: "1.0.0".to_string(),
                    description: Some("Gaming optimized configuration".to_string()),
                    boot_mode: BootMode::UEFI,
                    architecture: Architecture::X86_64,
                    base_image: "ubuntu-22.04-desktop".to_string(),
                    size_limit_mb: Some(6144),
                    selected_niches: vec!["gaming-tournament".to_string()],
                    custom_components: vec!["custom-monitoring-dashboard".to_string()],
                    advanced_settings: AdvancedSettings {
                        compression_level: CompressionLevel::Medium,
                        include_source_packages: false,
                        enable_secure_boot: false, // Gaming might need unsigned drivers
                        custom_kernel_params: vec![
                            "preempt=voluntary".to_string(),
                            "processor.max_cstate=1".to_string(),
                            "intel_idle.max_cstate=0".to_string(),
                        ],
                        post_install_scripts: vec![
                            "optimize-gaming.sh".to_string(),
                            "install-steam.sh".to_string(),
                        ],
                    },
                },
                tags: vec!["gaming".to_string(), "performance".to_string(), "desktop".to_string()],
            },
            IsoTemplate {
                id: "research-server".to_string(),
                name: "Research Server".to_string(),
                description: "High-performance server template for research and computational workloads".to_string(),
                category: "Research".to_string(),
                author: "Research Labs Consortium".to_string(),
                version: "2.0.0".to_string(),
                base_config: IsoConfig {
                    name: "research-server-template".to_string(),
                    version: "2.0.0".to_string(),
                    description: Some("Research server configuration".to_string()),
                    boot_mode: BootMode::UEFI,
                    architecture: Architecture::X86_64,
                    base_image: "ubuntu-22.04-server".to_string(),
                    size_limit_mb: Some(10240),
                    selected_niches: vec!["ai-research".to_string()],
                    custom_components: vec![
                        "custom-security-scanner".to_string(),
                        "custom-monitoring-dashboard".to_string(),
                    ],
                    advanced_settings: AdvancedSettings {
                        compression_level: CompressionLevel::Low, // Fast decompression for servers
                        include_source_packages: true,
                        enable_secure_boot: true,
                        custom_kernel_params: vec![
                            "transparent_hugepage=always".to_string(),
                            "numa_balancing=enable".to_string(),
                        ],
                        post_install_scripts: vec![
                            "setup-research-env.sh".to_string(),
                            "configure-clusters.sh".to_string(),
                        ],
                    },
                },
                tags: vec!["research".to_string(), "server".to_string(), "hpc".to_string()],
            },
        ]
    }

    /// Get default creator configuration
    pub fn get_default_creator_config(&self) -> IsoCreatorConfig {
        IsoCreatorConfig {
            build_directory: "/tmp/biomeos-builds".to_string(),
            cache_directory: "/tmp/biomeos-cache".to_string(),
            max_concurrent_builds: 2,
            cleanup_after_build: true,
            compression_threads: num_cpus::get(),
            default_timeout_minutes: 120,
        }
    }

    /// Convert unified niche to ISO creator format
    fn convert_unified_niche(&self, unified: UnifiedNichePackage) -> NichePackage {
        NichePackage {
            id: unified.id,
            name: unified.name,
            description: unified.description,
            author: unified.author,
            version: unified.version,
            category: unified.category,
            size_mb: unified.size_mb,
            features: unified.features,
            dependencies: unified.dependencies,
            manifest_path: unified.manifest_path,
            icon_path: unified.icon_path,
        }
    }
} 