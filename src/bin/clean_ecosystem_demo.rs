//! Clean Ecosystem Demo
//!
//! Demonstrates proper usage of the biomeOS Primal SDK with correct API patterns.
//! This replaces the broken demos with a robust, well-designed example.

use anyhow::Result;
use biomeos_primal_sdk::*;
use serde_json::json;
use tracing::{info, warn};

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("🌱 Starting Clean BiomeOS Ecosystem Demo");

    // Demonstrate proper primal creation and usage
    let primals = create_example_primals().await?;

    // Demonstrate ecosystem interaction
    demo_primal_interactions(&primals).await?;

    // Demonstrate health checking
    demo_health_monitoring(&primals).await?;

    info!("✅ Clean Ecosystem Demo completed successfully!");
    Ok(())
}

/// Create example primals with proper API usage
async fn create_example_primals() -> Result<Vec<Box<dyn EcoPrimal>>> {
    info!("📦 Creating example primals with correct API patterns");

    // Create all primals using vec![] macro for better performance
    let primals: Vec<Box<dyn EcoPrimal>> = vec![
        // Create compute primal (like Toadstool)
        Box::new(ComputePrimal::new("compute-service")?),
        // Create orchestration primal (like Songbird)
        Box::new(OrchestrationPrimal::new("orchestration-service")?),
        // Create security primal (like BearDog)
        Box::new(SecurityPrimal::new("security-service")?),
        // Create community gaming primal
        Box::new(CommunityPrimal::new("gaming-engine", "gaming")?),
    ];

    info!("✅ Created {} primals successfully", primals.len());
    Ok(primals)
}

/// Demonstrate proper primal interactions
async fn demo_primal_interactions(primals: &[Box<dyn EcoPrimal>]) -> Result<()> {
    info!("🔄 Demonstrating primal interactions");

    for primal in primals {
        let metadata = primal.metadata();
        info!(
            "🔍 Primal: {} ({})",
            metadata.name, metadata.primal_type.category
        );

        // Test ping request
        let request = PrimalRequest::new("ping", json!({"message": "hello"}));

        match primal.handle_request(request).await {
            Ok(response) => info!("  ✅ Ping successful: {:?}", response.status),
            Err(e) => warn!("  ⚠️  Ping failed: {}", e),
        }
    }

    Ok(())
}

/// Demonstrate health monitoring
async fn demo_health_monitoring(primals: &[Box<dyn EcoPrimal>]) -> Result<()> {
    info!("🏥 Demonstrating health monitoring");

    for primal in primals {
        let metadata = primal.metadata();
        match primal.health_check().await {
            Ok(health) => info!("  ✅ {}: {:?}", metadata.name, health),
            Err(e) => warn!("  ⚠️  {} health check failed: {}", metadata.name, e),
        }
    }

    Ok(())
}

// PROPER IMPLEMENTATIONS - NO MORE BROKEN PATTERNS

/// Compute primal implementation
struct ComputePrimal {
    metadata: PrimalMetadata,
    capabilities: Vec<PrimalCapability>,
}

impl ComputePrimal {
    fn new(name: &str) -> Result<Self> {
        let primal_type = PrimalType::new("compute", name, "1.0.0");
        let capabilities = vec![
            PrimalCapability::compute_provider(),
            PrimalCapability::system_management(),
        ];

        let metadata = PrimalMetadata::new(
            name,
            "1.0.0",
            "Compute and system management primal",
            primal_type.clone(),
            capabilities.clone(),
        )
        .with_author("BiomeOS Team <team@biomeos.org>")
        .with_license("MIT");

        Ok(Self {
            metadata,
            capabilities,
        })
    }
}

#[async_trait::async_trait]
impl EcoPrimal for ComputePrimal {
    fn metadata(&self) -> &PrimalMetadata {
        &self.metadata
    }

    fn capabilities(&self) -> &[PrimalCapability] {
        &self.capabilities
    }

    async fn initialize(&self, _config: &PrimalConfig) -> PrimalResult<()> {
        info!("🍄 Initializing compute primal: {}", self.metadata.name);
        Ok(())
    }

    async fn handle_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse> {
        match request.method.as_str() {
            "ping" => Ok(PrimalResponse::success(
                request.request_id,
                json!({"message": "pong", "service": "compute"}),
            )),
            "compute" => Ok(PrimalResponse::success(
                request.request_id,
                json!({"result": "computation complete", "cpu_usage": 75}),
            )),
            _ => Err(PrimalError::invalid_request(format!(
                "Unknown method: {}",
                request.method
            ))),
        }
    }

    async fn health_check(&self) -> PrimalResult<PrimalHealth> {
        Ok(PrimalHealth::Healthy)
    }

    async fn shutdown(&self) -> PrimalResult<()> {
        info!("🛑 Shutting down compute primal: {}", self.metadata.name);
        Ok(())
    }
}

/// Orchestration primal implementation
struct OrchestrationPrimal {
    metadata: PrimalMetadata,
    capabilities: Vec<PrimalCapability>,
}

impl OrchestrationPrimal {
    fn new(name: &str) -> Result<Self> {
        let primal_type = PrimalType::new("orchestration", name, "1.0.0");
        let capabilities = vec![
            PrimalCapability::orchestration_provider(),
            PrimalCapability::service_discovery(),
            PrimalCapability::load_balancing(),
        ];

        let metadata = PrimalMetadata::new(
            name,
            "1.0.0",
            "Service orchestration and discovery primal",
            primal_type.clone(),
            capabilities.clone(),
        )
        .with_author("BiomeOS Team <team@biomeos.org>")
        .with_license("MIT");

        Ok(Self {
            metadata,
            capabilities,
        })
    }
}

#[async_trait::async_trait]
impl EcoPrimal for OrchestrationPrimal {
    fn metadata(&self) -> &PrimalMetadata {
        &self.metadata
    }

    fn capabilities(&self) -> &[PrimalCapability] {
        &self.capabilities
    }

    async fn initialize(&self, _config: &PrimalConfig) -> PrimalResult<()> {
        info!(
            "🎼 Initializing orchestration primal: {}",
            self.metadata.name
        );
        Ok(())
    }

    async fn handle_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse> {
        match request.method.as_str() {
            "ping" => Ok(PrimalResponse::success(
                request.request_id,
                json!({"message": "pong", "service": "orchestration"}),
            )),
            "discover" => Ok(PrimalResponse::success(
                request.request_id,
                json!({"services": ["compute", "security"], "count": 2}),
            )),
            "route" => Ok(PrimalResponse::success(
                request.request_id,
                json!({"routed_to": "best_service", "latency_ms": 15}),
            )),
            _ => Err(PrimalError::invalid_request(format!(
                "Unknown method: {}",
                request.method
            ))),
        }
    }

    async fn health_check(&self) -> PrimalResult<PrimalHealth> {
        Ok(PrimalHealth::Healthy)
    }

    async fn shutdown(&self) -> PrimalResult<()> {
        info!(
            "🛑 Shutting down orchestration primal: {}",
            self.metadata.name
        );
        Ok(())
    }
}

/// Security primal implementation
struct SecurityPrimal {
    metadata: PrimalMetadata,
    capabilities: Vec<PrimalCapability>,
}

impl SecurityPrimal {
    fn new(name: &str) -> Result<Self> {
        let primal_type = PrimalType::new("security", name, "1.0.0");
        let capabilities = vec![
            PrimalCapability::security_provider(),
            PrimalCapability::encryption(),
            PrimalCapability::authentication(),
        ];

        let metadata = PrimalMetadata::new(
            name,
            "1.0.0",
            "Security and encryption primal",
            primal_type.clone(),
            capabilities.clone(),
        )
        .with_author("BiomeOS Team <team@biomeos.org>")
        .with_license("MIT");

        Ok(Self {
            metadata,
            capabilities,
        })
    }
}

#[async_trait::async_trait]
impl EcoPrimal for SecurityPrimal {
    fn metadata(&self) -> &PrimalMetadata {
        &self.metadata
    }

    fn capabilities(&self) -> &[PrimalCapability] {
        &self.capabilities
    }

    async fn initialize(&self, _config: &PrimalConfig) -> PrimalResult<()> {
        info!("🐻 Initializing security primal: {}", self.metadata.name);
        Ok(())
    }

    async fn handle_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse> {
        match request.method.as_str() {
            "ping" => Ok(PrimalResponse::success(
                request.request_id,
                json!({"message": "pong", "service": "security"}),
            )),
            "encrypt" => Ok(PrimalResponse::success(
                request.request_id,
                json!({"encrypted": "abc123def456", "algorithm": "AES-256"}),
            )),
            "authenticate" => Ok(PrimalResponse::success(
                request.request_id,
                json!({"authenticated": true, "user": "demo_user"}),
            )),
            _ => Err(PrimalError::invalid_request(format!(
                "Unknown method: {}",
                request.method
            ))),
        }
    }

    async fn health_check(&self) -> PrimalResult<PrimalHealth> {
        Ok(PrimalHealth::Healthy)
    }

    async fn shutdown(&self) -> PrimalResult<()> {
        info!("🛑 Shutting down security primal: {}", self.metadata.name);
        Ok(())
    }
}

/// Community primal implementation
struct CommunityPrimal {
    metadata: PrimalMetadata,
    capabilities: Vec<PrimalCapability>,
}

impl CommunityPrimal {
    fn new(name: &str, category: &str) -> Result<Self> {
        let primal_type = PrimalType::community(name, category);
        let capabilities = vec![
            PrimalCapability::gaming(),
            PrimalCapability::custom("game_logic", "Game logic processing"),
        ];

        let metadata = PrimalMetadata::new(
            name,
            "1.0.0",
            "Community-developed gaming primal",
            primal_type.clone(),
            capabilities.clone(),
        )
        .with_author("Community Developer <dev@community.org>")
        .with_license("MIT")
        .with_keywords(vec!["gaming".to_string(), "community".to_string()]);

        Ok(Self {
            metadata,
            capabilities,
        })
    }
}

#[async_trait::async_trait]
impl EcoPrimal for CommunityPrimal {
    fn metadata(&self) -> &PrimalMetadata {
        &self.metadata
    }

    fn capabilities(&self) -> &[PrimalCapability] {
        &self.capabilities
    }

    async fn initialize(&self, _config: &PrimalConfig) -> PrimalResult<()> {
        info!("🎮 Initializing community primal: {}", self.metadata.name);
        Ok(())
    }

    async fn handle_request(&self, request: PrimalRequest) -> PrimalResult<PrimalResponse> {
        match request.method.as_str() {
            "ping" => Ok(PrimalResponse::success(
                request.request_id,
                json!({"message": "pong", "service": "gaming"}),
            )),
            "start_game" => Ok(PrimalResponse::success(
                request.request_id,
                json!({"game_started": true, "level": 1, "players": 4}),
            )),
            "get_leaderboard" => Ok(PrimalResponse::success(
                request.request_id,
                json!({"leaderboard": [{"name": "Player1", "score": 9001}]}),
            )),
            _ => Err(PrimalError::invalid_request(format!(
                "Unknown method: {}",
                request.method
            ))),
        }
    }

    async fn health_check(&self) -> PrimalResult<PrimalHealth> {
        Ok(PrimalHealth::Healthy)
    }

    async fn shutdown(&self) -> PrimalResult<()> {
        info!("🛑 Shutting down community primal: {}", self.metadata.name);
        Ok(())
    }
}
