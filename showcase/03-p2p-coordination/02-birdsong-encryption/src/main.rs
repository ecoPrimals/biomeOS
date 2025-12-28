//! BirdSong Encrypted Discovery Demo
//!
//! This demo shows BiomeOS coordinating BirdSong (privacy-preserving discovery)
//! in pure Rust.
//!
//! "A broadcast that is obvious to family and noise otherwise"
//!
//! Key Features:
//! - Capability-based primal discovery
//! - Pure Rust coordination (not shell scripts!)
//! - Lineage-based encryption
//! - Privacy-preserving service discovery

use anyhow::Result;
use biomeos_core::p2p_coordination::{BirdSongCoordinator, DiscoveryProvider, SecurityProvider};
use std::sync::Arc;
use std::time::SystemTime;

#[tokio::main]
async fn main() -> Result<()> {
    println!("🌱 BiomeOS P2P Coordination Demo: BirdSong Encryption");
    println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
    println!();
    println!("🎵 \"A broadcast that is obvious to family and noise otherwise\"");
    println!();

    // Step 1: Discover primals by capability
    println!("🔍 Step 1: Discovering primals by capability...");
    println!("   Looking for: security capability (BirdSong support)");
    println!("   Looking for: discovery capability (encrypted mode)");
    println!();

    println!("⚠️  Note: Using mock providers for demonstration");
    println!("   In production, BiomeOS discovers real primals by capability");
    println!();

    let security = create_mock_security_provider();
    let discovery = create_mock_discovery_provider();

    println!("✅ Found security primal: MockSecurity (demonstrates BearDog)");
    println!("✅ Found discovery primal: MockDiscovery (demonstrates Songbird)");
    println!();

    // Step 2: Create BirdSong coordinator
    println!("🔐 Step 2: Creating BirdSong coordinator...");
    let coordinator = BirdSongCoordinator::new(security, discovery);
    println!("✅ Coordinator created");
    println!();

    // Step 3: Enable encrypted discovery
    println!("🎵 Step 3: Enabling BirdSong encrypted discovery...");
    println!("   Family ID: demo-family");
    println!();

    println!("   Generating broadcast keys from security primal...");
    println!("   Configuring discovery primal for encrypted mode...");
    println!("   Testing encryption is working...");
    println!();

    match coordinator.enable_encrypted_discovery("demo-family").await {
        Ok(mode) => {
            println!("✅ BirdSong encryption enabled successfully!");
            println!();
            println!("📊 Discovery Mode:");
            println!("   Mode: {:?}", mode);
            println!("   Privacy: HIGH (encrypted broadcasts)");
            println!("   Visibility: Family-only (lineage-verified)");
            println!();

            // Step 4: Demonstrate the privacy model
            println!("📊 Step 4: Privacy Model Demonstration");
            println!();
            println!("🔒 For Family Members (verified lineage):");
            println!("   ✅ Can decrypt service broadcasts");
            println!("   ✅ Can discover services");
            println!("   ✅ Can see node details");
            println!("   ✅ Can connect to services");
            println!();
            println!("👁️  For Others (non-family):");
            println!("   ❌ See only encrypted noise");
            println!("   ❌ Cannot discover services");
            println!("   ❌ Cannot see node details");
            println!("   ❌ Cannot connect to services");
            println!();

            // Step 5: Show what discovery looks like
            println!("📡 Step 5: Discovery Examples");
            println!();
            println!("Without BirdSong (Plaintext):");
            println!("   Observer sees:");
            println!("   - Node IDs: node-a, node-b, node-c");
            println!("   - IP Addresses: 192.168.1.100, 192.168.1.101...");
            println!("   - Services: web-server, database, api");
            println!("   - Capabilities: All visible");
            println!("   Privacy Level: ❌ LOW (everything visible)");
            println!();
            println!("With BirdSong (Encrypted):");
            println!("   Observer sees:");
            println!("   - Encrypted data: [random bytes]");
            println!("   - No IP addresses visible");
            println!("   - No service names visible");
            println!("   - No capabilities visible");
            println!("   Privacy Level: ✅ HIGH (selective visibility)");
            println!();

            // Step 6: Graceful degradation
            println!("🔄 Step 6: Graceful Degradation");
            println!();
            println!("BiomeOS supports both modes:");
            println!("   • BirdSong (Encrypted): For internet/untrusted networks");
            println!("   • Plaintext: For trusted LAN (faster, zero-config)");
            println!();
            println!("Current mode: {:?}", mode);
            println!();

            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
            println!("🎉 Demo complete!");
            println!();
            println!("Key Takeaways:");
            println!("  ✅ BiomeOS coordinated BirdSong encryption in pure Rust");
            println!("  ✅ Privacy-preserving discovery (family sees, others don't)");
            println!("  ✅ Lineage-based access control");
            println!("  ✅ Graceful degradation (encrypted or plaintext)");
            println!();
            println!("Next Steps:");
            println!("  - Run demo 03: Lineage-Gated Relay");
            println!("  - Deploy with BYOB: templates/birdsong-discovery.biome.yaml");
            println!("  - Test with real BearDog + Songbird");
            println!("━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━");
        }
        Err(e) => {
            println!("❌ BirdSong encryption failed: {}", e);
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
        _node_a: &str,
        _node_b: &str,
        _proof: &biomeos_core::p2p_coordination::LineageProof,
    ) -> Result<biomeos_core::p2p_coordination::TunnelRequest> {
        unimplemented!("Not used in this demo")
    }

    async fn check_tunnel_health(
        &self,
        _tunnel_id: &str,
    ) -> Result<biomeos_core::p2p_coordination::TunnelHealth> {
        unimplemented!("Not used in this demo")
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
        unimplemented!("Not used in this demo")
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
