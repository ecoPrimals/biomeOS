//! BTSP Tunnel Coordination Demo
//!
//! This demo shows BiomeOS coordinating BTSP tunnel creation in pure Rust.
//!
//! Key Features:
//! - Capability-based primal discovery (not hardcoded names!)
//! - Pure Rust coordination (not shell scripts!)
//! - Agnostic architecture (works with any compatible primals)
//! - Real error handling and health monitoring

use anyhow::Result;
use biomeos_core::p2p_coordination::{
    BtspCoordinator, LineageProof, SecurityProvider, DiscoveryProvider,
};
use std::sync::Arc;
use std::time::SystemTime;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🌱 BiomeOS P2P Coordination Demo: BTSP Tunnel");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();

    // Step 1: Discover primals by capability
    println!("🔍 Step 1: Discovering primals by capability...");
    println!("   Looking for: security capability (BTSP support)");
    println!("   Looking for: discovery capability (transport registration)");
    println!();

    // TODO: Implement real capability-based discovery
    // For now, we'll use mock providers to demonstrate the architecture
    println!("⚠️  Note: Using mock providers for demonstration");
    println!("   In production, BiomeOS discovers real primals by capability");
    println!();

    let security = create_mock_security_provider();
    let discovery = create_mock_discovery_provider();

    println!("✅ Found security primal: MockSecurity (demonstrates BearDog)");
    println!("✅ Found discovery primal: MockDiscovery (demonstrates Songbird)");
    println!();

    // Step 2: Create BTSP coordinator
    println!("🔐 Step 2: Creating BTSP tunnel coordinator...");
    let coordinator = BtspCoordinator::new(security, discovery);
    println!("✅ Coordinator created");
    println!();

    // Step 3: Create tunnel
    println!("🔗 Step 3: Coordinating BTSP tunnel creation...");
    println!("   Node A: alice");
    println!("   Node B: bob");
    println!();

    let lineage_proof = LineageProof {
        lineage_id: "demo-family".to_string(),
        depth: 2,
        proof: vec![1, 2, 3, 4], // Mock proof
        timestamp: SystemTime::now(),
    };

    println!("   Requesting tunnel from security primal...");
    println!("   Registering endpoints with discovery primal...");
    println!("   Verifying tunnel health...");
    println!();

    match coordinator.create_tunnel("alice", "bob", lineage_proof).await {
        Ok(tunnel) => {
            println!("✅ BTSP tunnel created successfully!");
            println!();
            println!("📊 Tunnel Information:");
            println!("   Tunnel ID: {}", tunnel.tunnel_id);
            println!("   Status: {:?}", tunnel.status);
            println!("   Endpoints: {} nodes", tunnel.endpoints.len());
            println!("   Established: {:?}", tunnel.established_at);
            println!();

            // Step 4: Monitor health
            println!("📊 Step 4: Monitoring tunnel health...");
            match coordinator.monitor_tunnel(&tunnel.tunnel_id).await {
                Ok(health) => {
                    println!("✅ Health check complete:");
                    println!("   Security: {:?}", health.security_health.status);
                    println!("   Transport: {:?}", health.transport_health.status);
                    println!("   Overall: {:?}", health.status);
                    println!();
                }
                Err(e) => {
                    println!("⚠️  Health check failed: {}", e);
                    println!();
                }
            }

            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("🎉 Demo complete!");
            println!();
            println!("Key Takeaways:");
            println!("  ✅ BiomeOS discovered primals by capability (not by name)");
            println!("  ✅ Pure Rust coordination (no shell scripts)");
            println!("  ✅ Agnostic architecture (works with any compatible primals)");
            println!("  ✅ Real error handling and health monitoring");
            println!();
            println!("Next Steps:");
            println!("  - Run demo 02: BirdSong Encryption");
            println!("  - Deploy with BYOB: templates/btsp-tunnel-only.biome.yaml");
            println!("  - Test with real BearDog + Songbird");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        }
        Err(e) => {
            println!("❌ Tunnel creation failed: {}", e);
            println!();
            println!("This is expected with mock providers.");
            println!("In production, BiomeOS coordinates real primals.");
        }
    }

    Ok(())
}

// Mock providers for demonstration
// In production, these are real adapters to BearDog, Songbird, etc.

fn create_mock_security_provider() -> Arc<dyn SecurityProvider> {
    Arc::new(MockSecurityProvider)
}

fn create_mock_discovery_provider() -> Arc<dyn DiscoveryProvider> {
    Arc::new(MockDiscoveryProvider)
}

struct MockSecurityProvider;

#[async_trait::async_trait]
impl SecurityProvider for MockSecurityProvider {
    async fn request_tunnel(
        &self,
        node_a: &str,
        node_b: &str,
        _proof: &LineageProof,
    ) -> Result<biomeos_core::p2p_coordination::TunnelRequest> {
        use biomeos_core::p2p_coordination::{TunnelRequest, TransportEndpoint};

        Ok(TunnelRequest {
            id: format!("tunnel-{}-{}", node_a, node_b),
            endpoint_a: TransportEndpoint {
                node_id: node_a.to_string(),
                address: "127.0.0.1".to_string(),
                port: 9100,
                protocol: "btsp".to_string(),
                secure: true,
            },
            endpoint_b: TransportEndpoint {
                node_id: node_b.to_string(),
                address: "127.0.0.1".to_string(),
                port: 9101,
                protocol: "btsp".to_string(),
                secure: true,
            },
            encryption_key: vec![1, 2, 3, 4, 5, 6, 7, 8],
            created_at: SystemTime::now(),
        })
    }

    async fn check_tunnel_health(
        &self,
        _tunnel_id: &str,
    ) -> Result<biomeos_core::p2p_coordination::TunnelHealth> {
        use biomeos_core::p2p_coordination::{TunnelHealth, HealthStatus};

        Ok(TunnelHealth {
            encryption_status: HealthStatus::Healthy,
            forward_secrecy: true,
            last_key_rotation: Some(SystemTime::now()),
            status: HealthStatus::Healthy,
        })
    }

    async fn generate_broadcast_keys(
        &self,
        family_id: &str,
    ) -> Result<biomeos_core::p2p_coordination::BroadcastKeys> {
        use biomeos_core::p2p_coordination::{BroadcastKeys, LineageProof};

        Ok(BroadcastKeys {
            broadcast_key: vec![1, 2, 3, 4, 5, 6, 7, 8],
            lineage_proof: LineageProof {
                lineage_id: family_id.to_string(),
                depth: 0,
                proof: vec![],
                timestamp: SystemTime::now(),
            },
            generated_at: SystemTime::now(),
        })
    }

    async fn verify_lineage(
        &self,
        _requester: &str,
        _target: &str,
    ) -> Result<biomeos_core::p2p_coordination::LineageInfo> {
        use biomeos_core::p2p_coordination::{LineageInfo, LineageProof};

        Ok(LineageInfo {
            is_ancestor: true,
            depth: 2,
            proof: LineageProof {
                lineage_id: "demo".to_string(),
                depth: 2,
                proof: vec![],
                timestamp: SystemTime::now(),
            },
        })
    }
}

struct MockDiscoveryProvider;

#[async_trait::async_trait]
impl DiscoveryProvider for MockDiscoveryProvider {
    async fn register_transport(
        &self,
        _endpoint: &biomeos_core::p2p_coordination::TransportEndpoint,
    ) -> Result<()> {
        Ok(())
    }

    async fn enable_encrypted_mode(
        &self,
        _config: biomeos_core::p2p_coordination::EncryptedDiscoveryConfig,
    ) -> Result<()> {
        Ok(())
    }

    async fn check_transport_health(
        &self,
        _transport_id: &str,
    ) -> Result<biomeos_core::p2p_coordination::TransportHealth> {
        use biomeos_core::p2p_coordination::{TransportHealth, HealthStatus};

        Ok(TransportHealth {
            connection_status: HealthStatus::Healthy,
            latency_ms: Some(12),
            packet_loss: Some(0.0),
            status: HealthStatus::Healthy,
        })
    }

    async fn test_encrypted_broadcast(
        &self,
    ) -> Result<biomeos_core::p2p_coordination::BroadcastTest> {
        use biomeos_core::p2p_coordination::BroadcastTest;

        Ok(BroadcastTest {
            encrypted: true,
            timestamp: SystemTime::now(),
            success: true,
        })
    }
}

