//! Mock data for ISO Creator development and testing
//!
//! This module provides mock data for development and testing purposes,
//! including sample configurations, niches, components, and templates.

use crate::views::iso_creator::types::*;

/// Mock data provider for ISO Creator
pub struct MockDataProvider;

impl MockDataProvider {
    /// Get mock niche packages
    pub fn get_mock_niches() -> Vec<NichePackage> {
        vec![
            NichePackage {
                id: "gaming-tournament".to_string(),
                name: "Gaming Tournament Platform".to_string(),
                description: "Complete tournament management and gaming infrastructure with real-time matchmaking".to_string(),
                author: "Tournament Masters Team".to_string(),
                version: "1.5.0".to_string(),
                category: "Gaming".to_string(),
                size_mb: 450,
                features: vec![
                    "Real-time matchmaking".to_string(),
                    "Physics simulation".to_string(),
                    "Leaderboard system".to_string(),
                    "Anti-cheat integration".to_string(),
                    "Stream overlay support".to_string(),
                ],
                dependencies: vec!["toadstool".to_string(), "songbird".to_string()],
                manifest_path: "/niches/gaming-tournament/manifest.yaml".to_string(),
                icon_path: Some("/niches/gaming-tournament/icon.png".to_string()),
            },
            NichePackage {
                id: "ai-research".to_string(),
                name: "AI Research Platform".to_string(),
                description: "Machine learning research environment with GPU support and distributed training".to_string(),
                author: "Deep Learning Lab".to_string(),
                version: "2.1.0".to_string(),
                category: "Research".to_string(),
                size_mb: 1200,
                features: vec![
                    "Distributed training".to_string(),
                    "Model versioning".to_string(),
                    "Dataset management".to_string(),
                    "Experiment tracking".to_string(),
                    "GPU acceleration".to_string(),
                    "Jupyter notebooks".to_string(),
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
                description: "Full-stack web development environment with modern frameworks".to_string(),
                author: "Frontend Velocity Team".to_string(),
                version: "1.8.2".to_string(),
                category: "Development".to_string(),
                size_mb: 800,
                features: vec![
                    "React/Next.js tools".to_string(),
                    "Auto-scaling frontend".to_string(),
                    "CDN integration".to_string(),
                    "Performance monitoring".to_string(),
                    "Hot reloading".to_string(),
                    "TypeScript support".to_string(),
                ],
                dependencies: vec![
                    "toadstool".to_string(),
                    "songbird".to_string(),
                ],
                manifest_path: "/niches/web-development/manifest.yaml".to_string(),
                icon_path: Some("/niches/web-development/icon.png".to_string()),
            },
            NichePackage {
                id: "security-audit".to_string(),
                name: "Security Audit Platform".to_string(),
                description: "Comprehensive security auditing and penetration testing tools".to_string(),
                author: "CyberSec Warriors".to_string(),
                version: "3.0.1".to_string(),
                category: "Security".to_string(),
                size_mb: 950,
                features: vec![
                    "Vulnerability scanning".to_string(),
                    "Penetration testing".to_string(),
                    "Network monitoring".to_string(),
                    "Threat intelligence".to_string(),
                    "Compliance reporting".to_string(),
                ],
                dependencies: vec![
                    "beardog".to_string(),
                    "squirrel".to_string(),
                ],
                manifest_path: "/niches/security-audit/manifest.yaml".to_string(),
                icon_path: Some("/niches/security-audit/icon.png".to_string()),
            },
            NichePackage {
                id: "media-production".to_string(),
                name: "Media Production Studio".to_string(),
                description: "Professional video and audio production environment".to_string(),
                author: "Creative Studio Team".to_string(),
                version: "2.3.0".to_string(),
                category: "Media".to_string(),
                size_mb: 1500,
                features: vec![
                    "Video editing".to_string(),
                    "Audio production".to_string(),
                    "3D rendering".to_string(),
                    "Color grading".to_string(),
                    "Effects libraries".to_string(),
                ],
                dependencies: vec![
                    "toadstool".to_string(),
                    "nestgate".to_string(),
                ],
                manifest_path: "/niches/media-production/manifest.yaml".to_string(),
                icon_path: Some("/niches/media-production/icon.png".to_string()),
            },
            NichePackage {
                id: "blockchain-dev".to_string(),
                name: "Blockchain Development Kit".to_string(),
                description: "Smart contract development and blockchain integration tools".to_string(),
                author: "Blockchain Builders".to_string(),
                version: "1.4.0".to_string(),
                category: "Development".to_string(),
                size_mb: 650,
                features: vec![
                    "Smart contracts".to_string(),
                    "DeFi protocols".to_string(),
                    "NFT support".to_string(),
                    "Cross-chain tools".to_string(),
                    "Testing frameworks".to_string(),
                ],
                dependencies: vec![
                    "toadstool".to_string(),
                    "beardog".to_string(),
                ],
                manifest_path: "/niches/blockchain-dev/manifest.yaml".to_string(),
                icon_path: Some("/niches/blockchain-dev/icon.png".to_string()),
            },
            NichePackage {
                id: "iot-platform".to_string(),
                name: "IoT Management Platform".to_string(),
                description: "Internet of Things device management and data collection".to_string(),
                author: "IoT Solutions Inc".to_string(),
                version: "2.0.0".to_string(),
                category: "Network".to_string(),
                size_mb: 400,
                features: vec![
                    "Device management".to_string(),
                    "Data collection".to_string(),
                    "Real-time monitoring".to_string(),
                    "Edge computing".to_string(),
                    "Protocol support".to_string(),
                ],
                dependencies: vec![
                    "toadstool".to_string(),
                    "songbird".to_string(),
                    "squirrel".to_string(),
                ],
                manifest_path: "/niches/iot-platform/manifest.yaml".to_string(),
                icon_path: Some("/niches/iot-platform/icon.png".to_string()),
            },
            NichePackage {
                id: "database-admin".to_string(),
                name: "Database Administration Suite".to_string(),
                description: "Database management and administration tools for multiple engines".to_string(),
                author: "Database Masters".to_string(),
                version: "1.9.0".to_string(),
                category: "Database".to_string(),
                size_mb: 750,
                features: vec![
                    "Multi-engine support".to_string(),
                    "Query optimization".to_string(),
                    "Backup management".to_string(),
                    "Performance tuning".to_string(),
                    "Migration tools".to_string(),
                ],
                dependencies: vec![
                    "nestgate".to_string(),
                    "squirrel".to_string(),
                ],
                manifest_path: "/niches/database-admin/manifest.yaml".to_string(),
                icon_path: Some("/niches/database-admin/icon.png".to_string()),
            },
        ]
    }

    /// Get mock custom components
    pub fn get_mock_components() -> Vec<CustomComponent> {
        vec![
            CustomComponent {
                name: "CUDA Toolkit".to_string(),
                description: "NVIDIA CUDA development toolkit for GPU computing".to_string(),
                component_type: ComponentType::Library,
                source_path: "/opt/nvidia/cuda".to_string(),
                destination_path: "/usr/local/cuda".to_string(),
                size_mb: 2500,
                required: false,
            },
            CustomComponent {
                name: "Docker Engine".to_string(),
                description: "Container runtime and orchestration platform".to_string(),
                component_type: ComponentType::Binary,
                source_path: "/usr/bin/docker".to_string(),
                destination_path: "/usr/bin/docker".to_string(),
                size_mb: 150,
                required: false,
            },
            CustomComponent {
                name: "Game Server Tools".to_string(),
                description: "Specialized tools for game server management".to_string(),
                component_type: ComponentType::Binary,
                source_path: "/opt/gameserver".to_string(),
                destination_path: "/usr/local/gameserver".to_string(),
                size_mb: 300,
                required: false,
            },
            CustomComponent {
                name: "PyTorch Models".to_string(),
                description: "Pre-trained machine learning models for PyTorch".to_string(),
                component_type: ComponentType::Library,
                source_path: "/opt/pytorch/models".to_string(),
                destination_path: "/usr/share/pytorch/models".to_string(),
                size_mb: 800,
                required: false,
            },
            CustomComponent {
                name: "Development Config".to_string(),
                description: "Pre-configured development environment settings".to_string(),
                component_type: ComponentType::Configuration,
                source_path: "/etc/dev-config".to_string(),
                destination_path: "/etc/dev-config".to_string(),
                size_mb: 5,
                required: false,
            },
            CustomComponent {
                name: "Monitoring Scripts".to_string(),
                description: "System monitoring and alerting scripts".to_string(),
                component_type: ComponentType::Script,
                source_path: "/opt/monitoring".to_string(),
                destination_path: "/usr/local/bin/monitoring".to_string(),
                size_mb: 25,
                required: false,
            },
            CustomComponent {
                name: "API Documentation".to_string(),
                description: "Complete API documentation and examples".to_string(),
                component_type: ComponentType::Documentation,
                source_path: "/docs/api".to_string(),
                destination_path: "/usr/share/doc/api".to_string(),
                size_mb: 50,
                required: false,
            },
            CustomComponent {
                name: "Deployment Templates".to_string(),
                description: "Infrastructure deployment templates and configs".to_string(),
                component_type: ComponentType::Template,
                source_path: "/templates/deploy".to_string(),
                destination_path: "/usr/share/templates/deploy".to_string(),
                size_mb: 20,
                required: false,
            },
        ]
    }

    /// Get mock build jobs
    pub fn get_mock_build_jobs() -> Vec<BuildJob> {
        vec![
            BuildJob {
                id: "build-001".to_string(),
                config: IsoConfig {
                    name: "biomeOS-gaming-complete".to_string(),
                    description: "Complete gaming platform with all features".to_string(),
                    version: "1.0.0".to_string(),
                    target_arch: "x86_64".to_string(),
                    boot_mode: BootMode::UEFI,
                    included_primals: vec![
                        "toadstool".to_string(),
                        "songbird".to_string(),
                        "nestgate".to_string(),
                    ],
                    included_niches: vec!["gaming-tournament".to_string()],
                    custom_components: vec!["game-server-tools".to_string()],
                    compression_level: 7,
                    size_estimate: 2100,
                    created_at: "2024-01-15 10:30:00".to_string(),
                },
                status: BuildStatus::Success,
                progress: 1.0,
                started_at: Some("2024-01-15 10:30:00".to_string()),
                completed_at: Some("2024-01-15 10:45:00".to_string()),
                output_path: Some("/tmp/biomeos-isos/biomeOS-gaming-complete.iso".to_string()),
                error_message: None,
                build_log: vec![
                    "🚀 Build started".to_string(),
                    "📋 Configuration validated".to_string(),
                    "🧩 Installing primals".to_string(),
                    "🎭 Installing niches".to_string(),
                    "🗜️ Compressing filesystem".to_string(),
                    "💿 Creating ISO image".to_string(),
                    "✅ Build completed successfully".to_string(),
                ],
            },
            BuildJob {
                id: "build-002".to_string(),
                config: IsoConfig {
                    name: "biomeOS-research-ai".to_string(),
                    description: "AI research platform with GPU support".to_string(),
                    version: "1.2.0".to_string(),
                    target_arch: "x86_64".to_string(),
                    boot_mode: BootMode::Hybrid,
                    included_primals: vec![
                        "toadstool".to_string(),
                        "squirrel".to_string(),
                        "nestgate".to_string(),
                    ],
                    included_niches: vec!["ai-research".to_string()],
                    custom_components: vec![
                        "cuda-toolkit".to_string(),
                        "pytorch-models".to_string(),
                    ],
                    compression_level: 5,
                    size_estimate: 4200,
                    created_at: "2024-01-15 11:00:00".to_string(),
                },
                status: BuildStatus::Building,
                progress: 0.65,
                started_at: Some("2024-01-15 11:00:00".to_string()),
                completed_at: None,
                output_path: None,
                error_message: None,
                build_log: vec![
                    "🚀 Build started".to_string(),
                    "📋 Configuration validated".to_string(),
                    "🧩 Installing primals".to_string(),
                    "🎭 Installing niches".to_string(),
                    "⚙️ Installing CUDA toolkit".to_string(),
                    "🤖 Installing PyTorch models".to_string(),
                    "🗜️ Compressing filesystem (in progress)".to_string(),
                ],
            },
            BuildJob {
                id: "build-003".to_string(),
                config: IsoConfig {
                    name: "biomeOS-minimal-server".to_string(),
                    description: "Minimal server installation".to_string(),
                    version: "1.0.0".to_string(),
                    target_arch: "x86_64".to_string(),
                    boot_mode: BootMode::Legacy,
                    included_primals: vec!["toadstool".to_string()],
                    included_niches: vec![],
                    custom_components: vec![],
                    compression_level: 9,
                    size_estimate: 800,
                    created_at: "2024-01-15 09:45:00".to_string(),
                },
                status: BuildStatus::Failed,
                progress: 0.3,
                started_at: Some("2024-01-15 09:45:00".to_string()),
                completed_at: Some("2024-01-15 09:50:00".to_string()),
                output_path: None,
                error_message: Some("Insufficient disk space".to_string()),
                build_log: vec![
                    "🚀 Build started".to_string(),
                    "📋 Configuration validated".to_string(),
                    "🧩 Installing primals".to_string(),
                    "❌ Error: Insufficient disk space".to_string(),
                    "⏹️ Build failed".to_string(),
                ],
            },
        ]
    }

    /// Get mock ISO configurations
    pub fn get_mock_configs() -> Vec<IsoConfig> {
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
                included_niches: vec!["gaming-tournament".to_string()],
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
                included_niches: vec!["ai-research".to_string()],
                custom_components: vec!["cuda-toolkit".to_string(), "pytorch-models".to_string()],
                compression_level: 5,
                size_estimate: 4200,
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
                included_niches: vec!["web-development".to_string()],
                custom_components: vec!["docker-engine".to_string()],
                compression_level: 6,
                size_estimate: 2800,
                created_at: "2024-01-12".to_string(),
            },
            IsoConfig {
                name: "biomeOS-security".to_string(),
                description: "Security-focused distribution with audit tools".to_string(),
                version: "1.3.0".to_string(),
                target_arch: "x86_64".to_string(),
                boot_mode: BootMode::Hybrid,
                included_primals: vec![
                    "toadstool".to_string(),
                    "beardog".to_string(),
                    "squirrel".to_string(),
                ],
                included_niches: vec!["security-audit".to_string()],
                custom_components: vec!["monitoring-scripts".to_string()],
                compression_level: 6,
                size_estimate: 1950,
                created_at: "2024-01-14".to_string(),
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

    /// Get mock templates
    pub fn get_mock_templates() -> Vec<IsoTemplate> {
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
                size_estimate: 4200,
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
                tags: vec![
                    "web".to_string(),
                    "development".to_string(),
                    "fullstack".to_string(),
                ],
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
                name: "Security Audit Platform".to_string(),
                description: "Security-focused distribution with penetration testing tools"
                    .to_string(),
                use_case: "security".to_string(),
                included_components: vec![
                    "toadstool".to_string(),
                    "beardog".to_string(),
                    "squirrel".to_string(),
                ],
                size_estimate: 1950,
                difficulty: TemplateDifficulty::Expert,
                tags: vec![
                    "security".to_string(),
                    "audit".to_string(),
                    "pentest".to_string(),
                ],
                author: "biomeOS Security Team".to_string(),
                version: "1.0.0".to_string(),
            },
            IsoTemplate {
                name: "Media Production Studio".to_string(),
                description: "Professional media production environment".to_string(),
                use_case: "media".to_string(),
                included_components: vec!["toadstool".to_string(), "nestgate".to_string()],
                size_estimate: 3200,
                difficulty: TemplateDifficulty::Advanced,
                tags: vec![
                    "media".to_string(),
                    "production".to_string(),
                    "creative".to_string(),
                ],
                author: "biomeOS Media Team".to_string(),
                version: "1.0.0".to_string(),
            },
        ]
    }

    /// Get default ISO creator configuration
    pub fn get_default_creator_config() -> IsoCreatorConfig {
        IsoCreatorConfig {
            output_directory: "/tmp/biomeos-isos".to_string(),
            default_compression_level: 6,
            default_target_architecture: "x86_64".to_string(),
            default_boot_mode: BootMode::Hybrid,
            include_all_primals: true,
            include_demos: true,
            include_documentation: true,
            max_concurrent_builds: 2,
            build_timeout_minutes: 60,
        }
    }

    /// Generate random build log entries
    pub fn generate_build_log_entries(progress: f32) -> Vec<String> {
        let mut entries = vec![
            "🚀 Starting build process...".to_string(),
            "📋 Configuration validated".to_string(),
            "📦 Collecting components".to_string(),
        ];

        if progress > 0.2 {
            entries.push("🧩 Installing Toadstool primal".to_string());
        }
        if progress > 0.3 {
            entries.push("🐦 Installing Songbird primal".to_string());
        }
        if progress > 0.4 {
            entries.push("🥚 Installing NestGate primal".to_string());
        }
        if progress > 0.5 {
            entries.push("🐿️ Installing Squirrel primal".to_string());
        }
        if progress > 0.6 {
            entries.push("🎭 Installing niche packages".to_string());
        }
        if progress > 0.7 {
            entries.push("⚙️ Installing custom components".to_string());
        }
        if progress > 0.8 {
            entries.push("🗜️ Compressing filesystem".to_string());
        }
        if progress > 0.9 {
            entries.push("💿 Creating ISO image".to_string());
        }
        if progress >= 1.0 {
            entries.push("✅ Build completed successfully".to_string());
        }

        entries
    }
}
