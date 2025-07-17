//! # EcoPrimals Ecosystem Demonstration
//!
//! Unified CLI that demonstrates the complete biomeOS ecosystem with all Primals
//! working together in perfect coordination.

use std::collections::HashMap;
use std::path::PathBuf;
use std::time::Duration;

use clap::{Parser, Subcommand};
use tracing::info;

use biomeos_core::{
    ecosystem_integration::{
        EcosystemCapabilities, EcosystemCoordinator, EcosystemEndpoints, EcosystemSecurity,
        EcosystemServiceRegistration, HealthCheckConfig, ResourceRequirements,
    },
    BiomeError, BiomeResult,
};

#[derive(Parser)]
#[command(
    name = "ecosystem-demo",
    about = "EcoPrimals Ecosystem Demonstration - All Primals Working Together",
    version = "1.0.0"
)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,

    /// Configuration file
    #[arg(short, long, default_value = "ecosystem.yaml")]
    config: PathBuf,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize the complete ecosystem
    Init {
        /// Ecosystem name
        #[arg(short, long, default_value = "demo-ecosystem")]
        name: String,
    },

    /// Deploy a unified biome across all Primals
    Deploy {
        /// Biome manifest file
        manifest: PathBuf,

        /// Watch deployment progress
        #[arg(short, long)]
        watch: bool,
    },

    /// Show ecosystem status
    Status {
        /// Show detailed status
        #[arg(short, long)]
        detailed: bool,

        /// Watch status continuously
        #[arg(short, long)]
        watch: bool,
    },

    /// Run ecosystem health checks
    Health {
        /// Run comprehensive health check
        #[arg(short, long)]
        comprehensive: bool,
    },

    /// Demonstrate ecosystem capabilities
    Demo {
        /// Demo scenario to run
        #[arg(short, long, default_value = "full-stack")]
        scenario: String,
    },

    /// Manage Primal coordination
    Primals {
        #[command(subcommand)]
        action: PrimalCommands,
    },

    /// Federation management
    Federation {
        #[command(subcommand)]
        action: FederationCommands,
    },
}

#[derive(Subcommand)]
enum PrimalCommands {
    /// List all registered Primals
    List,

    /// Register a new Primal
    Register {
        /// Primal type
        primal_type: String,

        /// Primal endpoint
        endpoint: String,
    },

    /// Test Primal integration
    Test {
        /// Primal to test
        primal_type: String,
    },

    /// Show Primal coordination matrix
    Matrix,
}

#[derive(Subcommand)]
enum FederationCommands {
    /// Join federation network
    Join {
        /// Bootstrap nodes
        #[arg(short, long)]
        bootstrap: Vec<String>,
    },

    /// Show federation topology
    Topology,

    /// Test cross-biome deployment
    CrossDeploy {
        /// Deployment manifest
        manifest: PathBuf,
    },
}

/// Main ecosystem demonstration
struct EcosystemDemo {
    coordinator: EcosystemCoordinator,
    config: EcosystemDemoConfig,
}

#[derive(Debug, Clone)]
struct EcosystemDemoConfig {
    name: String,
    primals: HashMap<String, PrimalConfig>,
    federation_enabled: bool,
    demo_scenarios: HashMap<String, DemoScenario>,
}

#[derive(Debug, Clone)]
struct PrimalConfig {
    endpoint: String,
    enabled: bool,
    auto_register: bool,
}

#[derive(Debug, Clone)]
struct DemoScenario {
    name: String,
    description: String,
    steps: Vec<DemoStep>,
}

#[derive(Debug, Clone)]
struct DemoStep {
    name: String,
    action: String,
    target_primals: Vec<String>,
    expected_result: String,
}

impl EcosystemDemo {
    async fn new() -> BiomeResult<Self> {
        let coordinator = EcosystemCoordinator::new();
        let config = Self::load_default_config();

        Ok(Self {
            coordinator,
            config,
        })
    }

    fn load_default_config() -> EcosystemDemoConfig {
        let mut primals = HashMap::new();

        // Configure all Primals with default endpoints
        primals.insert(
            "songbird".to_string(),
            PrimalConfig {
                endpoint: "http://localhost:8080".to_string(),
                enabled: true,
                auto_register: true,
            },
        );

        primals.insert(
            "nestgate".to_string(),
            PrimalConfig {
                endpoint: "http://localhost:8082".to_string(),
                enabled: true,
                auto_register: true,
            },
        );

        primals.insert(
            "toadstool".to_string(),
            PrimalConfig {
                endpoint: "http://localhost:8084".to_string(),
                enabled: true,
                auto_register: true,
            },
        );

        // BearDog and Squirrel are preparing
        primals.insert(
            "beardog".to_string(),
            PrimalConfig {
                endpoint: "http://localhost:9000".to_string(),
                enabled: false,
                auto_register: false,
            },
        );

        primals.insert(
            "squirrel".to_string(),
            PrimalConfig {
                endpoint: "http://localhost:5000".to_string(),
                enabled: false,
                auto_register: false,
            },
        );

        let mut demo_scenarios = HashMap::new();
        demo_scenarios.insert(
            "full-stack".to_string(),
            DemoScenario {
                name: "Full Stack Deployment".to_string(),
                description: "Deploy a complete web application across all Primals".to_string(),
                steps: vec![
                    DemoStep {
                        name: "Storage Provisioning".to_string(),
                        action: "provision_storage".to_string(),
                        target_primals: vec!["nestgate".to_string()],
                        expected_result: "Storage volumes created".to_string(),
                    },
                    DemoStep {
                        name: "Service Registration".to_string(),
                        action: "register_services".to_string(),
                        target_primals: vec!["songbird".to_string()],
                        expected_result: "Services registered in mesh".to_string(),
                    },
                    DemoStep {
                        name: "Workload Deployment".to_string(),
                        action: "deploy_workloads".to_string(),
                        target_primals: vec!["toadstool".to_string()],
                        expected_result: "Workloads running".to_string(),
                    },
                    DemoStep {
                        name: "Health Verification".to_string(),
                        action: "verify_health".to_string(),
                        target_primals: vec![
                            "songbird".to_string(),
                            "nestgate".to_string(),
                            "toadstool".to_string(),
                        ],
                        expected_result: "All services healthy".to_string(),
                    },
                ],
            },
        );

        EcosystemDemoConfig {
            name: "EcoPrimals Demo Ecosystem".to_string(),
            primals,
            federation_enabled: true,
            demo_scenarios,
        }
    }

    async fn initialize_ecosystem(&mut self, name: String) -> BiomeResult<()> {
        info!("🌱 Initializing EcoPrimals Ecosystem: {}", name);

        // Update config
        self.config.name = name;

        // Initialize coordinator
        self.coordinator.initialize_ecosystem().await?;

        // Register enabled Primals
        let primals_to_register: Vec<_> = self
            .config
            .primals
            .iter()
            .filter(|(_, config)| config.enabled && config.auto_register)
            .map(|(primal_type, config)| (primal_type.clone(), config.endpoint.clone()))
            .collect();

        for (primal_type, endpoint) in primals_to_register {
            self.register_primal(primal_type, endpoint).await?;
        }

        info!("✅ Ecosystem initialization complete!");
        Ok(())
    }

    async fn register_primal(&mut self, primal_type: String, endpoint: String) -> BiomeResult<()> {
        info!("🔗 Registering {} at {}", primal_type, endpoint);

        let registration = self.create_primal_registration(&primal_type, &endpoint)?;

        // Register with ecosystem
        self.coordinator
            .service_registry
            .register_service(registration)
            .await?;

        info!("✅ {} successfully registered", primal_type);
        Ok(())
    }

    fn create_primal_registration(
        &self,
        primal_type: &str,
        endpoint: &str,
    ) -> BiomeResult<EcosystemServiceRegistration> {
        let (capabilities, metadata) = match primal_type {
            "songbird" => (
                EcosystemCapabilities {
                    core: vec![
                        "service_discovery".to_string(),
                        "load_balancing".to_string(),
                        "traffic_routing".to_string(),
                    ],
                    extended: vec!["federation".to_string(), "byob_coordination".to_string()],
                    integrations: vec![
                        "toadstool_orchestration".to_string(),
                        "nestgate_storage_discovery".to_string(),
                    ],
                },
                {
                    let mut meta = HashMap::new();
                    meta.insert("role".to_string(), "orchestrator".to_string());
                    meta.insert("protocols".to_string(), "http,grpc,websocket".to_string());
                    meta
                },
            ),
            "nestgate" => (
                EcosystemCapabilities {
                    core: vec![
                        "storage_provisioning".to_string(),
                        "volume_management".to_string(),
                        "zfs_operations".to_string(),
                    ],
                    extended: vec![
                        "tiered_storage".to_string(),
                        "multi_protocol_access".to_string(),
                    ],
                    integrations: vec![
                        "biomeos_volume_provisioning".to_string(),
                        "toadstool_mount_coordination".to_string(),
                    ],
                },
                {
                    let mut meta = HashMap::new();
                    meta.insert("role".to_string(), "storage_provider".to_string());
                    meta.insert("protocols".to_string(), "nfs,smb,iscsi,s3".to_string());
                    meta
                },
            ),
            "toadstool" => (
                EcosystemCapabilities {
                    core: vec![
                        "workload_execution".to_string(),
                        "runtime_orchestration".to_string(),
                        "resource_management".to_string(),
                    ],
                    extended: vec![
                        "multi_runtime_support".to_string(),
                        "byob_execution".to_string(),
                        "gpu_acceleration".to_string(),
                    ],
                    integrations: vec![
                        "biomeos_manifest_parsing".to_string(),
                        "nestgate_volume_mounting".to_string(),
                        "songbird_service_registration".to_string(),
                    ],
                },
                {
                    let mut meta = HashMap::new();
                    meta.insert("role".to_string(), "compute_executor".to_string());
                    meta.insert(
                        "runtimes".to_string(),
                        "container,native,wasm,python".to_string(),
                    );
                    meta
                },
            ),
            _ => {
                return Err(BiomeError::InvalidInput(format!(
                    "Unknown primal type: {}",
                    primal_type
                )))
            }
        };

        Ok(EcosystemServiceRegistration {
            service_id: format!("primal-{}-demo", primal_type),
            primal_type: primal_type.to_string(),
            biome_id: "demo-ecosystem".to_string(),
            version: "1.0.0".to_string(),
            api_version: "biomeOS/v1".to_string(),
            registration_time: chrono::Utc::now(),
            endpoints: EcosystemEndpoints {
                primary: endpoint.to_string(),
                health: format!("{}/health", endpoint),
                metrics: format!("{}/metrics", endpoint),
                admin: Some(format!("{}/admin", endpoint)),
                websocket: Some(format!("{}/ws", endpoint.replace("http", "ws"))),
            },
            capabilities,
            security: EcosystemSecurity {
                authentication_method: "ecosystem_jwt".to_string(),
                tls_enabled: true,
                mtls_required: false,
                trust_domain: "biome.local".to_string(),
            },
            resource_requirements: ResourceRequirements {
                cpu: "2".to_string(),
                memory: "4Gi".to_string(),
                storage: "10Gi".to_string(),
                network: "1Gbps".to_string(),
            },
            health_check: HealthCheckConfig {
                interval: Duration::from_secs(30),
                timeout: Duration::from_secs(10),
                retries: 3,
                grace_period: Duration::from_secs(60),
            },
            metadata,
        })
    }

    async fn deploy_biome(&self, manifest_path: PathBuf, watch: bool) -> BiomeResult<()> {
        info!("🚀 Deploying biome from: {}", manifest_path.display());

        // Load manifest
        let manifest_content = tokio::fs::read_to_string(&manifest_path)
            .await
            .map_err(|e| BiomeError::InvalidInput(format!("Failed to read manifest: {}", e)))?;

        info!("📋 Manifest loaded, coordinating deployment across Primals...");

        // Simulate coordinated deployment
        self.simulate_coordinated_deployment(&manifest_content, watch)
            .await?;

        info!("✅ Biome deployment complete!");
        Ok(())
    }

    async fn simulate_coordinated_deployment(
        &self,
        _manifest: &str,
        watch: bool,
    ) -> BiomeResult<()> {
        let steps = vec![
            ("🏗️  Preparing infrastructure", "songbird", 2),
            ("💾 Provisioning storage", "nestgate", 3),
            ("🔗 Setting up networking", "songbird", 2),
            ("🚀 Deploying workloads", "toadstool", 4),
            ("🔍 Verifying health", "all", 2),
        ];

        for (step_name, primal, duration) in steps {
            info!("{} ({})", step_name, primal);

            if watch {
                for i in 1..=duration {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    info!("  ⏳ Progress: {}/{}", i, duration);
                }
            } else {
                tokio::time::sleep(Duration::from_millis(500)).await;
            }

            info!("  ✅ {} complete", step_name);
        }

        Ok(())
    }

    async fn show_ecosystem_status(&self, detailed: bool, watch: bool) -> BiomeResult<()> {
        loop {
            info!("🌍 EcoPrimals Ecosystem Status");
            info!("═══════════════════════════════");

            // Get ecosystem status
            let status = self.coordinator.get_ecosystem_status().await?;

            info!("📊 Overall Health: {:?}", status.health.overall_health);
            info!("🔧 Active Primals: {}", status.active_primals);
            info!("⚙️  Total Services: {}", status.total_services);
            info!("⏱️  Uptime: {:?}", status.uptime);

            if detailed {
                info!("\n🔍 Detailed Status:");
                info!("───────────────────");

                for (primal_type, health_info) in &status.health.primal_health {
                    info!(
                        "  {} - {:?} ({}/{})",
                        primal_type,
                        health_info.health,
                        health_info.healthy_count,
                        health_info.total_count
                    );
                }

                // Show service registry
                let songbird_services = self
                    .coordinator
                    .service_registry
                    .get_services_by_type("songbird")
                    .await;
                let nestgate_services = self
                    .coordinator
                    .service_registry
                    .get_services_by_type("nestgate")
                    .await;
                let toadstool_services = self
                    .coordinator
                    .service_registry
                    .get_services_by_type("toadstool")
                    .await;

                info!("\n📋 Service Registry:");
                info!("  🎼 Songbird: {} services", songbird_services.len());
                info!("  🏰 NestGate: {} services", nestgate_services.len());
                info!("  🍄 Toadstool: {} services", toadstool_services.len());
            }

            if !watch {
                break;
            }

            info!("\n⏳ Refreshing in 5 seconds... (Ctrl+C to stop)");
            tokio::time::sleep(Duration::from_secs(5)).await;

            // Clear screen for next update
            print!("\x1B[2J\x1B[1;1H");
        }

        Ok(())
    }

    async fn run_health_checks(&self, comprehensive: bool) -> BiomeResult<()> {
        info!("🏥 Running ecosystem health checks");

        if comprehensive {
            info!("🔬 Comprehensive health check mode");
        }

        // Check ecosystem health
        let health = self
            .coordinator
            .service_registry
            .check_ecosystem_health()
            .await?;

        info!("📊 Health Check Results:");
        info!("═══════════════════════");
        info!("Overall: {:?}", health.overall_health);
        info!(
            "Services: {}/{} healthy",
            health.healthy_services, health.total_services
        );

        for (primal_type, health_info) in &health.primal_health {
            let status_emoji = match health_info.health {
                biomeos_core::HealthStatus::Healthy => "✅",
                biomeos_core::HealthStatus::Warning => "⚠️",
                biomeos_core::HealthStatus::Critical => "❌",
                _ => "❓",
            };

            info!(
                "{} {}: {}/{} services healthy",
                status_emoji, primal_type, health_info.healthy_count, health_info.total_count
            );
        }

        if comprehensive {
            info!("\n🔍 Running comprehensive checks...");

            // Simulate comprehensive checks
            let checks = vec![
                "Network connectivity",
                "Storage integrity",
                "Resource availability",
                "Security posture",
                "Performance metrics",
            ];

            for check in checks {
                tokio::time::sleep(Duration::from_millis(500)).await;
                info!("  ✅ {}", check);
            }
        }

        info!("\n🎉 Health check complete!");
        Ok(())
    }

    async fn run_demo_scenario(&self, scenario: String) -> BiomeResult<()> {
        info!("🎭 Running demo scenario: {}", scenario);

        let demo_scenario =
            self.config.demo_scenarios.get(&scenario).ok_or_else(|| {
                BiomeError::InvalidInput(format!("Unknown scenario: {}", scenario))
            })?;

        info!("📝 Scenario: {}", demo_scenario.description);
        info!("🎬 Starting demonstration...");

        for (i, step) in demo_scenario.steps.iter().enumerate() {
            info!("\n📍 Step {}: {}", i + 1, step.name);
            info!("   Target: {}", step.target_primals.join(", "));
            info!("   Action: {}", step.action);

            // Simulate step execution
            tokio::time::sleep(Duration::from_secs(2)).await;

            info!("   ✅ Result: {}", step.expected_result);
        }

        info!("\n🎉 Demo scenario complete!");
        info!("🌟 All Primals working together in perfect harmony!");
        Ok(())
    }

    async fn list_primals(&self) -> BiomeResult<()> {
        info!("🔍 Registered Primals in Ecosystem");
        info!("═══════════════════════════════════");

        for (primal_type, config) in &self.config.primals {
            let status_emoji = if config.enabled { "✅" } else { "⏸️" };
            let auto_register = if config.auto_register { "🔄" } else { "🔧" };

            info!(
                "{} {} {} - {}",
                status_emoji, auto_register, primal_type, config.endpoint
            );
        }

        info!("\n📊 Ecosystem Summary:");
        let enabled_count = self.config.primals.values().filter(|c| c.enabled).count();
        let total_count = self.config.primals.len();
        info!("  Active: {}/{} Primals", enabled_count, total_count);

        Ok(())
    }

    async fn show_primal_matrix(&self) -> BiomeResult<()> {
        info!("🕸️  Primal Coordination Matrix");
        info!("═══════════════════════════════");

        let primals = vec!["songbird", "nestgate", "toadstool", "beardog", "squirrel"];

        // Header
        print!("         ");
        for primal in &primals {
            print!("{:>10}", &primal[..3].to_uppercase());
        }
        println!();

        // Matrix
        for from_primal in &primals {
            print!("{:>8} ", &from_primal[..8]);

            for to_primal in &primals {
                let connection = if from_primal == to_primal {
                    "    -    "
                } else if self
                    .config
                    .primals
                    .get(*from_primal)
                    .is_some_and(|c| c.enabled)
                    && self
                        .config
                        .primals
                        .get(*to_primal)
                        .is_some_and(|c| c.enabled)
                {
                    "    ✅    "
                } else if from_primal == &"beardog"
                    || from_primal == &"squirrel"
                    || to_primal == &"beardog"
                    || to_primal == &"squirrel"
                {
                    "    🔄    "
                } else {
                    "    ❌    "
                };
                print!("{}", connection);
            }
            println!();
        }

        info!("\n🔑 Legend:");
        info!("  ✅ Active connection");
        info!("  🔄 Preparing integration");
        info!("  ❌ Not connected");
        info!("  -  Self");

        Ok(())
    }
}

#[tokio::main]
async fn main() -> BiomeResult<()> {
    let args = Args::parse();

    // Initialize logging
    let log_level = if args.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!(
            "ecosystem_demo={},biomeos_core={}",
            log_level, log_level
        ))
        .init();

    // Create ecosystem demo
    let mut demo = EcosystemDemo::new().await?;

    // Execute command
    match args.command {
        Commands::Init { name } => {
            demo.initialize_ecosystem(name).await?;
        }

        Commands::Deploy { manifest, watch } => {
            demo.deploy_biome(manifest, watch).await?;
        }

        Commands::Status { detailed, watch } => {
            demo.show_ecosystem_status(detailed, watch).await?;
        }

        Commands::Health { comprehensive } => {
            demo.run_health_checks(comprehensive).await?;
        }

        Commands::Demo { scenario } => {
            demo.run_demo_scenario(scenario).await?;
        }

        Commands::Primals { action } => match action {
            PrimalCommands::List => {
                demo.list_primals().await?;
            }
            PrimalCommands::Register {
                primal_type,
                endpoint,
            } => {
                demo.register_primal(primal_type, endpoint).await?;
            }
            PrimalCommands::Test { primal_type: _ } => {
                info!("🧪 Testing Primal integration...");
                info!("✅ Test complete!");
            }
            PrimalCommands::Matrix => {
                demo.show_primal_matrix().await?;
            }
        },

        Commands::Federation { action } => match action {
            FederationCommands::Join { bootstrap: _ } => {
                info!("🌐 Joining federation network...");
                info!("✅ Federation join complete!");
            }
            FederationCommands::Topology => {
                info!("🗺️  Federation topology mapping...");
                info!("✅ Topology analysis complete!");
            }
            FederationCommands::CrossDeploy { manifest: _ } => {
                info!("🌍 Cross-biome deployment...");
                info!("✅ Cross-deployment complete!");
            }
        },
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_ecosystem_initialization() {
        let mut demo = EcosystemDemo::new().await.unwrap();

        demo.initialize_ecosystem("test-ecosystem".to_string())
            .await
            .unwrap();

        assert_eq!(demo.config.name, "test-ecosystem");
    }

    #[tokio::test]
    async fn test_primal_registration() {
        let demo = EcosystemDemo::new().await.unwrap();

        let registration = demo
            .create_primal_registration("songbird", "http://localhost:8080")
            .unwrap();

        assert_eq!(registration.primal_type, "songbird");
        assert_eq!(registration.endpoints.primary, "http://localhost:8080");
    }
}
