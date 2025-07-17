use biomeos_core::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> BiomeResult<()> {
    println!("🧬 biomeOS Universal Platform - Fully Universalized with Crypto Locks");
    println!("═══════════════════════════════════════════════════════════════════════");
    println!();

    // Initialize universal biomeOS with grandma-safe defaults
    let config = create_grandma_safe_config();
    let mut biome_manager = UniversalBiomeManager::new(config);

    println!("🔧 Initializing biomeOS with sovereignty-first defaults...");
    biome_manager.initialize_grandma_safe().await?;
    println!("✅ biomeOS initialized with grandma-safe AI cat door and crypto locks");
    println!();

    // Demonstrate universal platform capabilities
    demonstrate_universal_platform(&biome_manager).await?;

    // Demonstrate crypto lock system
    demonstrate_crypto_locks(&biome_manager).await?;

    // Demonstrate AI cat door for basic users
    demonstrate_ai_cat_door(&biome_manager).await?;

    // Demonstrate vendor lock elimination
    demonstrate_vendor_lock_elimination(&biome_manager).await?;

    // Demonstrate sovereignty compliance
    demonstrate_sovereignty_compliance(&biome_manager).await?;

    // Deploy universally across all platforms
    println!("🚀 Deploying biomeOS universally across all platforms...");
    biome_manager.deploy_universal().await?;
    println!("✅ biomeOS deployed successfully with full vendor independence");

    Ok(())
}

/// Create grandma-safe configuration with crypto locks and AI cat door
fn create_grandma_safe_config() -> UniversalBiomeConfig {
    let mut config = UniversalBiomeConfig::default();

    // Enable crypto locks with AI cat door for basic users
    config.crypto_locks.enabled = true;
    config.crypto_locks.ai_cat_door.enabled = true;
    config
        .crypto_locks
        .ai_cat_door
        .cost_protection
        .max_monthly_cost = 20.0;
    config
        .crypto_locks
        .ai_cat_door
        .cost_protection
        .auto_disable_on_limit = true;

    // Set medium sovereignty level (good balance)
    config.platform.sovereignty_level = SovereigntyLevel::Medium;

    // Configure universal providers with fallback chains
    config.providers.container_providers = vec![
        ContainerProviderConfig {
            name: "podman".to_string(),
            provider_type: "podman".to_string(),
            enabled: true,
            priority: 1, // Prefer open source
            config: HashMap::new(),
        },
        ContainerProviderConfig {
            name: "docker".to_string(),
            provider_type: "docker".to_string(),
            enabled: true,
            priority: 2, // Fallback to Docker
            config: HashMap::new(),
        },
        ContainerProviderConfig {
            name: "native".to_string(),
            provider_type: "native".to_string(),
            enabled: true,
            priority: 3, // Ultimate fallback
            config: HashMap::new(),
        },
    ];

    // Configure crypto providers with quantum-resistant options
    config.providers.crypto_providers = vec![
        CryptoProviderConfig {
            name: "rustls".to_string(),
            provider_type: CryptoProvider::Rustls {
                version: "0.21".to_string(),
            },
            enabled: true,
            quantum_resistant: false,
            fallback_priority: 1,
        },
        CryptoProviderConfig {
            name: "ring".to_string(),
            provider_type: CryptoProvider::Ring {
                version: "0.16".to_string(),
            },
            enabled: true,
            quantum_resistant: false,
            fallback_priority: 2,
        },
        CryptoProviderConfig {
            name: "quantum-resistant".to_string(),
            provider_type: CryptoProvider::QuantumResistant {
                provider: "pqcrypto".to_string(),
            },
            enabled: true,
            quantum_resistant: true,
            fallback_priority: 3,
        },
    ];

    config
}

/// Demonstrate universal platform capabilities
async fn demonstrate_universal_platform(manager: &UniversalBiomeManager) -> BiomeResult<()> {
    println!("🌍 Universal Platform Capabilities");
    println!("─────────────────────────────────");

    // Detect current platform
    let platform_info = manager.platform.detect_platform().await?;
    println!("📱 Platform detected: {:?}", platform_info.os_type);
    println!("🏗️ Architecture: {}", platform_info.architecture);
    println!("💾 Memory: {} MB", platform_info.resources.memory_mb);
    println!("🖥️ CPU cores: {}", platform_info.resources.cpu_cores);

    if let Some(gpu) = &platform_info.resources.gpu_info {
        println!(
            "🎮 GPU: {} {} ({} MB)",
            gpu.vendor, gpu.model, gpu.memory_mb
        );
    } else {
        println!("🎮 GPU: CPU-only mode (universal fallback)");
    }

    println!("✨ Capabilities:");
    for capability in &platform_info.capabilities {
        match capability {
            PlatformCapability::Containers { runtime } => {
                println!("  📦 Container runtime: {}", runtime);
            }
            PlatformCapability::Virtualization { technology } => {
                println!("  🔧 Virtualization: {}", technology);
            }
            PlatformCapability::GpuCompute { technology } => {
                println!("  🚀 GPU compute: {}", technology);
            }
            _ => {
                println!("  ⚡ Additional capabilities available");
            }
        }
    }
    println!();

    Ok(())
}

/// Demonstrate crypto lock system in action
async fn demonstrate_crypto_locks(manager: &UniversalBiomeManager) -> BiomeResult<()> {
    println!("🔐 Crypto Lock System");
    println!("────────────────────");

    // Simulate external dependency access requests
    let dependencies = vec![
        (
            "AWS S3",
            DependencyType::CloudProvider {
                services: vec!["s3".to_string()],
            },
        ),
        (
            "Docker Hub",
            DependencyType::PackageRegistry {
                package_types: vec!["docker".to_string()],
            },
        ),
        (
            "OpenAI API",
            DependencyType::AiService {
                model_types: vec!["gpt-4".to_string()],
            },
        ),
        (
            "Kubernetes",
            DependencyType::Orchestrator {
                api_version: "v1.28".to_string(),
            },
        ),
    ];

    for (name, dep_type) in dependencies {
        let dependency = ExternalDependency {
            id: name.to_lowercase().replace(" ", "_"),
            name: name.to_string(),
            dependency_type: dep_type,
            vendor: "External".to_string(),
            access_requirements: AccessRequirements {
                crypto_lock_required: true,
                sovereign_key_required: name != "OpenAI API", // AI cat door exception
                compliance_level: if name == "OpenAI API" {
                    ComplianceLevel::Personal
                } else {
                    ComplianceLevel::Commercial
                },
                usage_restrictions: vec![
                    UsageRestriction::RateLimit {
                        requests_per_hour: 1000,
                    },
                    UsageRestriction::DataLimit {
                        mb_per_month: 10000,
                    },
                ],
                cat_door_allowed: name == "OpenAI API", // AI cat door for personal AI use
            },
            sovereignty_impact: SovereigntyImpact {
                impact_level: if name.contains("AWS") {
                    SovereigntyImpactLevel::High
                } else {
                    SovereigntyImpactLevel::Moderate
                },
                data_residency_requirements: vec!["US".to_string()],
                vendor_lock_risk: VendorLockRisk {
                    risk_level: if name.contains("AWS") {
                        RiskLevel::High
                    } else {
                        RiskLevel::Medium
                    },
                    lock_factors: vec![LockFactor::ProprietaryApi, LockFactor::NetworkEffects],
                    migration_difficulty: MigrationDifficulty::Moderate,
                    cost_to_exit: Some(10000.0),
                },
                exit_strategy: ExitStrategy {
                    data_portability: DataPortability::MostlyPortable {
                        limitations: vec!["Vendor-specific metadata".to_string()],
                    },
                    code_portability: CodePortability::PartiallyPortable {
                        major_refactoring_needed: vec!["API calls".to_string()],
                    },
                    estimated_migration_time_weeks: 8,
                    migration_checklist: vec![
                        "Data export".to_string(),
                        "API refactoring".to_string(),
                        "Testing".to_string(),
                    ],
                },
                alternatives_available: true,
            },
            licensing: LicensingInfo {
                license_type: LicenseType::Commercial {
                    pricing_model: PricingModel::PerRequest,
                },
                commercial_terms: None,
                attribution_required: false,
                source_disclosure_required: false,
                patent_grant: false,
                copyleft_requirements: vec![],
            },
            api_signatures: vec![],
            alternatives: vec![AlternativeDependency {
                name: "MinIO".to_string(),
                vendor: "Open Source".to_string(),
                compatibility_score: 0.95,
                migration_effort: MigrationDifficulty::Easy,
                sovereignty_improvement: 0.9,
                cost_comparison: CostComparison::Cheaper {
                    savings_percent: 60.0,
                },
            }],
        };

        // Simulate access context for individual user
        let context = AccessContext {
            user_type: UserType::Individual { verified: true },
            usage_pattern: UsagePattern {
                usage_type: UsageType::Personal,
                scale: UsageScale::Individual,
                frequency: UsageFrequency::Occasional,
                data_sensitivity: DataSensitivity::Internal,
                commercial_purpose: false,
                revenue_generating: false,
            },
            geographic_location: Some("US".to_string()),
            current_usage: CurrentUsage {
                daily_requests: 50,
                monthly_data_gb: 1.0,
                concurrent_connections: 1,
                peak_usage_time: chrono::Utc::now(),
                cost_current_month: 5.0,
            },
            biome_configuration: BiomeConfiguration {
                energy_flow_state: EnergyFlowState::Closed,
                sovereignty_level: locks::SovereigntyLevel::Medium,
                ai_cat_door_enabled: true,
                compliance_frameworks: vec!["personal_use".to_string()],
                geographic_restrictions: vec![],
            },
        };

        // Validate access through crypto lock system
        let decision = manager
            .crypto_locks
            .validate_access(&dependency, &context)
            .await?;

        println!("🔍 Dependency: {}", name);
        match decision.decision {
            AccessVerdict::Granted => {
                if dependency.access_requirements.cat_door_allowed {
                    println!("  🐱 Access: GRANTED via AI Cat Door (grandma-safe)");
                } else {
                    println!("  ✅ Access: GRANTED");
                }
            }
            AccessVerdict::ConditionalGrant => {
                println!("  ⚠️ Access: CONDITIONAL");
                for condition in &decision.conditions {
                    match condition {
                        AccessCondition::RateLimit {
                            max_requests_per_hour,
                        } => {
                            println!("    📊 Rate limit: {} requests/hour", max_requests_per_hour);
                        }
                        AccessCondition::PaymentRequired { amount, currency } => {
                            println!("    💳 Payment required: {} {}", amount, currency);
                        }
                        _ => {
                            println!("    📋 Additional conditions apply");
                        }
                    }
                }
            }
            AccessVerdict::Denied => {
                println!("  ❌ Access: DENIED");
            }
            _ => {
                println!("  🔄 Access: PENDING REVIEW");
            }
        }
        println!("  💡 Reasoning: {}", decision.reasoning);

        if !decision.alternatives_suggested.is_empty() {
            println!("  🔄 Suggested alternatives:");
            for alt in &decision.alternatives_suggested {
                println!(
                    "    • {} ({}% compatibility, {}% more sovereign)",
                    alt.name,
                    (alt.compatibility_score * 100.0) as u32,
                    (alt.sovereignty_improvement * 100.0) as u32
                );
            }
        }
        println!();
    }

    Ok(())
}

/// Demonstrate AI cat door for basic users
async fn demonstrate_ai_cat_door(manager: &UniversalBiomeManager) -> BiomeResult<()> {
    println!("🐱 AI Cat Door (Grandma-Safe AI Access)");
    println!("──────────────────────────────────────");

    if !manager.crypto_locks.ai_cat_door.enabled {
        println!("❌ AI Cat Door is disabled");
        return Ok(());
    }

    println!("✅ AI Cat Door is ENABLED for grandma-safe AI access");
    println!();

    println!("📋 Allowed AI Services:");
    for service in &manager.crypto_locks.ai_cat_door.allowed_ai_services {
        println!("  🤖 {}", service.service_name);
        println!("    📊 Max requests/day: {}", service.max_requests_per_day);
        println!(
            "    🎫 Max tokens/request: {}",
            service.max_tokens_per_request
        );
        println!(
            "    💰 Cost limit/month: ${:.2}",
            service.cost_limit_per_month
        );
    }
    println!();

    println!("🛡️ Safety Limits:");
    let limits = &manager.crypto_locks.ai_cat_door.usage_limits;
    println!("  📈 Daily requests: {}", limits.daily_request_limit);
    println!("  🎫 Monthly tokens: {}", limits.monthly_token_limit);
    println!("  💸 Monthly cost: ${:.2}", limits.monthly_cost_limit);
    println!(
        "  🔄 Concurrent requests: {}",
        limits.concurrent_request_limit
    );
    println!();

    println!("🔒 Cost Protection:");
    println!(
        "  💰 Maximum monthly cost: ${:.2}",
        manager
            .config
            .crypto_locks
            .ai_cat_door
            .cost_protection
            .max_monthly_cost
    );
    println!(
        "  📢 Alert thresholds: ${:.0}, ${:.0}, ${:.0}",
        manager
            .config
            .crypto_locks
            .ai_cat_door
            .cost_protection
            .alert_thresholds[0],
        manager
            .config
            .crypto_locks
            .ai_cat_door
            .cost_protection
            .alert_thresholds[1],
        manager
            .config
            .crypto_locks
            .ai_cat_door
            .cost_protection
            .alert_thresholds[2]
    );
    println!(
        "  🚨 Auto-disable on limit: {}",
        if manager
            .config
            .crypto_locks
            .ai_cat_door
            .cost_protection
            .auto_disable_on_limit
        {
            "YES (prevents surprise bills)"
        } else {
            "NO"
        }
    );
    println!();

    println!("💡 AI Cat Door Benefits:");
    println!("  🎯 Grandma can use AI safely without complex setup");
    println!("  💰 Cost protection prevents surprise bills");
    println!("  🔒 Personal use stays within biome sovereignty");
    println!("  🚀 No crypto locks needed for basic AI access");
    println!("  📚 AI helps with learning and productivity");
    println!();

    Ok(())
}

/// Demonstrate vendor lock elimination
async fn demonstrate_vendor_lock_elimination(manager: &UniversalBiomeManager) -> BiomeResult<()> {
    println!("🔓 Vendor Lock Elimination");
    println!("─────────────────────────");

    // Container runtime vendor lock elimination
    println!("📦 Container Runtime Universality:");
    for provider in &manager.config.providers.container_providers {
        let status = if provider.enabled {
            "✅ ACTIVE"
        } else {
            "⏸️ INACTIVE"
        };
        println!(
            "  {} {} (priority: {})",
            status, provider.name, provider.priority
        );
    }
    println!("  🔄 Fallback chain: Podman → Docker → Native processes");
    println!("  🎯 Result: Zero container runtime vendor lock");
    println!();

    // Cloud provider vendor lock elimination
    println!("☁️ Cloud Provider Universality:");
    if manager.config.providers.cloud_providers.is_empty() {
        println!("  🏠 Self-hosted deployment (maximum sovereignty)");
        println!("  🔄 Can deploy to any cloud provider when needed");
        println!("  🎯 Result: Zero cloud vendor lock");
    } else {
        for provider in &manager.config.providers.cloud_providers {
            println!(
                "  {} {} (sovereign: {})",
                if provider.enabled { "✅" } else { "❌" },
                provider.name,
                if provider.sovereignty_compliant {
                    "YES"
                } else {
                    "NO"
                }
            );
        }
    }
    println!();

    // Compute provider vendor lock elimination
    println!("🖥️ Compute Provider Universality:");
    for provider in &manager.config.providers.compute_providers {
        println!(
            "  {} {} (sovereignty impact: {})",
            if provider.enabled { "✅" } else { "❌" },
            provider.name,
            provider.sovereignty_impact
        );
    }
    println!("  🔄 Fallback chain: Open drivers → Proprietary drivers → CPU-only");
    println!("  🎯 Result: Zero compute vendor lock");
    println!();

    // Orchestration vendor lock elimination
    println!("🎼 Orchestration Universality:");
    for provider in &manager.config.providers.orchestration_providers {
        println!(
            "  {} {} (self-hosted: {})",
            if provider.enabled { "✅" } else { "❌" },
            provider.name,
            if provider.self_hosted { "YES" } else { "NO" }
        );
    }
    println!("  🔄 Fallback chain: Direct deployment → K3s → Nomad → Kubernetes");
    println!("  🎯 Result: Zero orchestration vendor lock");
    println!();

    // Crypto provider vendor lock elimination
    println!("🔐 Cryptography Universality:");
    for provider in &manager.config.providers.crypto_providers {
        println!(
            "  {} {} (quantum-resistant: {}, priority: {})",
            if provider.enabled { "✅" } else { "❌" },
            provider.name,
            if provider.quantum_resistant {
                "YES"
            } else {
                "NO"
            },
            provider.fallback_priority
        );
    }
    println!("  🔄 Fallback chain: rustls → ring → sodium → openssl");
    println!("  🎯 Result: Zero crypto library vendor lock");
    println!();

    println!("🎉 Universal Vendor Independence Achieved!");
    println!("  🔓 No single vendor can hold biomeOS hostage");
    println!("  🔄 Seamless migration between providers");
    println!("  💰 Cost optimization through competition");
    println!("  🌍 True platform universality");
    println!();

    Ok(())
}

/// Demonstrate sovereignty compliance
async fn demonstrate_sovereignty_compliance(manager: &UniversalBiomeManager) -> BiomeResult<()> {
    println!("👑 Sovereignty Compliance");
    println!("────────────────────────");

    // Check current sovereignty level
    match manager.config.platform.sovereignty_level {
        SovereigntyLevel::Maximum => {
            println!("🏰 Sovereignty Level: MAXIMUM (Air-gapped)");
            println!("  🚫 Zero external dependencies");
            println!("  🔒 Complete data sovereignty");
            println!("  🏠 100% self-hosted");
        }
        SovereigntyLevel::High => {
            println!("🏛️ Sovereignty Level: HIGH");
            println!("  🔐 All external deps crypto-locked");
            println!("  🗝️ Sovereign keys required");
            println!("  📊 Minimal external services");
        }
        SovereigntyLevel::Medium => {
            println!("🏢 Sovereignty Level: MEDIUM (Recommended)");
            println!("  ⚖️ Balanced sovereignty and functionality");
            println!("  🔄 Exit strategies for all dependencies");
            println!("  🛡️ Crypto locks for critical services");
        }
        SovereigntyLevel::Low => {
            println!("🏪 Sovereignty Level: LOW");
            println!("  📱 Standard external dependencies allowed");
            println!("  📊 Basic vendor lock protection");
        }
        SovereigntyLevel::Minimal => {
            println!("🌐 Sovereignty Level: MINIMAL");
            println!("  🔓 Any external dependencies allowed");
            println!("  ⚠️ Maximum convenience, minimal sovereignty");
        }
    }
    println!();

    // MYCORRHIZA energy flow state
    match manager.config.platform.mycorrhiza.system_state {
        EnergyFlowState::Closed => {
            println!("🔒 MYCORRHIZA State: CLOSED (Grandma-safe default)");
            println!("  🏠 Foundation locked to external access");
            println!("  🔐 All Primals locked to external APIs");
            println!("  🐱 Personal AI cat door enabled");
            println!("  🛡️ Zero external dependencies for core functions");
            println!("  👑 Full sovereignty maintained");
        }
        EnergyFlowState::PrivateOpen => {
            println!("🔓 MYCORRHIZA State: PRIVATE OPEN");
            println!("  🤝 Trust-based external access");
            println!("  🗝️ Sovereign keys for partners");
            println!("  🔬 Research collaborations enabled");
        }
        EnergyFlowState::CommercialOpen => {
            println!("💼 MYCORRHIZA State: COMMERCIAL OPEN");
            println!("  💰 Pay-to-play enterprise integrations");
            println!("  ☁️ Cloud providers pay for access");
            println!("  💵 Revenue funds biomeOS development");
        }
    }
    println!();

    // AI sovereignty features
    if manager.config.platform.mycorrhiza.personal_ai.enabled {
        println!("🤖 AI Sovereignty Features:");
        println!("  🏠 Local AI models available");
        println!("  🔑 Personal API keys (encrypted)");
        println!("  🐱 Cat door for basic users");
        println!("  🛡️ AI stays within biome boundaries");
        println!();
    }

    // Voluntary partnership model for sustainability
    println!("🤝 Voluntary Partnership Model (Not Extraction):");
    if manager
        .config
        .crypto_locks
        .licensing
        .partnership
        .partnership_enabled
    {
        if let Some(percentage) = manager
            .config
            .crypto_locks
            .licensing
            .partnership
            .contribution_percent
        {
            println!("  📊 Partnership contribution: {}% (voluntary)", percentage);
        } else {
            println!("  📊 Partnership contribution: User-defined (voluntary)");
        }
        if let Some(wallet) = &manager
            .config
            .crypto_locks
            .licensing
            .partnership
            .sovereign_wallet
        {
            println!("  🏦 Sovereign wallet: {} (sweetgrass/rhizoCrypt)", wallet);
        } else {
            println!(
                "  🏦 Sovereign wallet: Not configured (uses sweetgrass/rhizoCrypt when available)"
            );
        }
        println!("  🎁 Partnership benefits:");
        for benefit in &manager
            .config
            .crypto_locks
            .licensing
            .partnership
            .partnership_benefits
        {
            println!("    • {}", benefit);
        }
    } else {
        println!("  🚫 Partnership disabled (fully sovereign mode)");
        println!("  ⚡ Users choose: licensing OR partnership OR fully open");
    }
    println!("  👑 True sovereignty: no compulsory extraction");
    println!();

    println!("🎯 True Sovereignty Benefits:");
    println!("  🔓 No vendor can hold your data hostage");
    println!("  💰 Cost optimization through competition");
    println!("  🌍 Deploy anywhere, migrate anytime");
    println!("  🛡️ Privacy and security by default");
    println!("  🚀 Innovation without vendor constraints");
    println!("  👑 No compulsory extraction or hidden fees");
    println!("  🤝 Partnership is voluntary, not mandatory");
    println!("  💎 Economic sovereignty through sweetgrass/rhizoCrypt");
    println!();

    // sweetgrass/rhizoCrypt sovereignty layer
    println!("🌾 sweetgrass/rhizoCrypt Integration:");
    println!("  🔐 Sovereign payment infrastructure");
    println!("  🌱 Built-in crypto layer for economic independence");
    println!("  🕸️ Rhizome network for decentralized transactions");
    println!("  💫 No dependency on traditional payment processors");
    println!("  🏛️ Users control their own economic relationships");
    println!("  🔮 Ready for partnership contributions when users choose");
    println!();

    Ok(())
}

// Helper types for demonstration
use biomeos_core::universal::PlatformCapability;
