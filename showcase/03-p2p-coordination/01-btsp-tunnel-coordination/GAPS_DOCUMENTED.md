# 🔒 BTSP Tunnel Coordination - API Gap Documentation

**Date**: December 31, 2025  
**Status**: Gap Discovery Phase  
**Based on**: phase1/beardog/showcase/02-ecosystem-integration/01-songbird-btsp/  

---

## Overview

This document captures the **actual APIs** needed for BTSP tunnel coordination between Songbird (orchestration) and BearDog (encryption). These APIs are **proven working** in phase1.

---

## Architecture

```
┌──────────────────┐
│ Songbird Tower   │ ← Orchestration service
│ (mDNS/UDP)       │   Coordinates node discovery
└────────┬─────────┘
         │
    ┌────┴────┐
    │         │
┌───▼───┐ ┌──▼─────┐
│ Node  │ │  Node  │
│ Alice │ │   Bob  │
│(BTSP  │ │ (BTSP  │
│client)│ │ server)│
└───┬───┘ └───┬────┘
    └─────┬───┘
      BTSP Tunnel
     (via BearDog)
```

---

## API 1: Songbird - Node Registration

### Endpoint (Phase1 Actual)
**Note**: Songbird in phase1 uses **mDNS/UDP**, not HTTP!

### Discovery Pattern (From phase1)
```rust
// From phase1/beardog/showcase/02-ecosystem-integration/01-songbird-btsp/src/main.rs:274-302

/// Discover orchestration service via capability-based discovery
async fn discover_orchestrator(config: &DemoConfig) -> Result<DiscoveredService> {
    info!("🔍 Searching for services with 'orchestration' capability...");
    
    // Method 1: Check environment variables
    if let Some(service) = discover_from_environment("orchestration").await? {
        info!("   ✅ Found via environment variables");
        return Ok(service);
    }
    
    // Method 2: mDNS/DNS-SD (future)
    // Method 3: Service registry (future)
    
    // Fallback: Use config
    Ok(DiscoveredService {
        display_name: "orchestrator (from config)".to_string(),
        endpoint: ServiceEndpoint {
            primary_url: config.orchestrator_endpoint.clone(),
        },
        capabilities: vec!["orchestration".to_string()],
    })
}

/// Discover from environment variables
/// Example: PRIMAL_SONGBIRD_ENDPOINT="http://localhost:9090"
///          PRIMAL_SONGBIRD_CAPABILITIES="orchestration,federation"
async fn discover_from_environment(required_capability: &str) -> Result<Option<DiscoveredService>> {
    for (key, value) in std::env::vars() {
        if key.starts_with("PRIMAL_") && key.ends_with("_ENDPOINT") {
            // Extract primal name
            let name = key
                .trim_start_matches("PRIMAL_")
                .trim_end_matches("_ENDPOINT")
                .to_lowercase();
            
            // Check capabilities
            let cap_key = format!("PRIMAL_{}_CAPABILITIES", name.to_uppercase());
            if let Ok(capabilities) = std::env::var(&cap_key) {
                if capabilities.contains(required_capability) {
                    return Ok(Some(DiscoveredService {
                        display_name: format!("{} (discovered)", name),
                        endpoint: ServiceEndpoint {
                            primary_url: value,
                        },
                        capabilities: capabilities.split(',').map(String::from).collect(),
                    }));
                }
            }
        }
    }
    Ok(None)
}
```

### Registration Pattern (From phase1)
```rust
// From phase1/beardog/showcase/02-ecosystem-integration/01-songbird-btsp/src/main.rs:307-331

async fn register_with_orchestrator(
    config: &DemoConfig,
    orchestrator: &DiscoveredService,
) -> Result<(String, Arc<UpaClient>)> {
    let upa_config = UpaClientConfig {
        service_endpoint: orchestrator.endpoint.primary_url.clone(),
        ..Default::default()
    };
    
    let upa_client = Arc::new(UpaClient::new(upa_config));
    
    // Register node with orchestrator
    let register_request = RegisterNodeRequest {
        node_id: config.node_id.clone(),
        node_type: "beardog".to_string(),
        capabilities: vec!["btsp".to_string(), "encryption".to_string()],
        endpoint: config.listen_address.clone(),
    };
    
    upa_client.register_node(register_request).await?;
    
    Ok((config.node_id.clone(), upa_client))
}
```

### Key Insight
**Songbird uses mDNS/UDP, not HTTP!** The "HTTP endpoint" pattern in phase1 is actually using a UPA (Universal Primal Adapter) client that abstracts the actual protocol.

---

## API 2: Songbird - Peer Discovery

### Pattern (From phase1)
```rust
// From phase1/beardog/showcase/02-ecosystem-integration/01-songbird-btsp/src/main.rs:333-351

async fn discover_peer(
    orchestrator: &DiscoveredService,
    peer_name: &str,
) -> Result<PeerInfo> {
    let upa_config = UpaClientConfig {
        service_endpoint: orchestrator.endpoint.primary_url.clone(),
        ..Default::default()
    };
    
    let upa_client = UpaClient::new(upa_config);
    
    // Query orchestrator for peer
    let peer_info = upa_client.find_peer(peer_name).await?;
    
    Ok(PeerInfo {
        node_id: peer_info.node_id,
        endpoint: peer_info.endpoint,
        capabilities: peer_info.capabilities,
    })
}
```

---

## API 3: BearDog - BTSP Tunnel Establishment

### Pattern (From phase1)
```rust
// From phase1/beardog/showcase/02-ecosystem-integration/01-songbird-btsp/src/main.rs:353-389

async fn establish_btsp_tunnel(
    config: &DemoConfig,
    peer: &PeerInfo,
) -> Result<(TunnelHandle, Arc<BeardogBtspProvider>)> {
    // Initialize HSM manager
    let hsm_manager = Arc::new(HsmManager::new(config.hsm_config.clone()).await?);
    
    // Initialize genetic engine
    let genetic_config = GeneticEngineConfig {
        enable_evolution: true,
        ..Default::default()
    };
    let genetic_engine = Arc::new(
        EcosystemGeneticEngine::new(genetic_config, hsm_manager.clone()).await?
    );
    
    // Create BTSP provider
    let btsp_provider = Arc::new(
        BeardogBtspProvider::new(hsm_manager, genetic_engine).await?
    );
    
    // Establish tunnel to peer
    let peer_endpoint = PeerEndpoint {
        address: peer.endpoint.clone(),
        public_key: None, // Discovered during handshake
    };
    
    let tunnel_handle = btsp_provider
        .establish_tunnel(peer_endpoint, config.btsp_config.clone())
        .await?;
    
    Ok((tunnel_handle, btsp_provider))
}
```

---

## API 4: BearDog - Send Encrypted Message

### Pattern (From phase1)
```rust
// From phase1/beardog/showcase/02-ecosystem-integration/01-songbird-btsp/src/main.rs:391-407

async fn send_encrypted_message(
    btsp_provider: &BeardogBtspProvider,
    tunnel: &TunnelHandle,
    message: &str,
) -> Result<()> {
    let message_bytes = message.as_bytes();
    
    btsp_provider
        .send_data(tunnel, message_bytes)
        .await?;
    
    Ok(())
}

async fn receive_encrypted_message(
    btsp_provider: &BeardogBtspProvider,
    tunnel: &TunnelHandle,
) -> Result<Vec<u8>> {
    btsp_provider
        .receive_data(tunnel)
        .await
}
```

---

## Critical Discovery: UPA (Universal Primal Adapter)

### What is UPA?

From the phase1 code, **UPA** is a protocol abstraction layer:

```rust
use beardog_tunnel::api::upa_client::{UpaClient, UpaClientConfig};
```

**Key Insight**: UPA abstracts whether the service is:
- HTTP/REST
- mDNS/UDP (Songbird's actual protocol)
- gRPC
- Custom protocol

### UPA Client Interface

```rust
pub struct UpaClient {
    service_endpoint: String,
    // ... internal protocol handling
}

impl UpaClient {
    pub async fn register_node(&self, request: RegisterNodeRequest) -> Result<()>;
    pub async fn find_peer(&self, peer_name: &str) -> Result<PeerInfo>;
    pub async fn list_nodes(&self) -> Result<Vec<PeerInfo>>;
}
```

---

## Gap Summary

### ✅ What's Working (Phase1)
1. Environment-based discovery (`PRIMAL_*_ENDPOINT`, `PRIMAL_*_CAPABILITIES`)
2. UPA client for protocol abstraction
3. Capability-based service discovery
4. BTSP tunnel establishment via BearDog
5. Encrypted message transmission

### ❌ What's Missing (Phase2/biomeOS)
1. **UPA Client in biomeOS**
   - Location: Should be in `biomeOS/src/primal_clients/upa_client.rs`
   - Status: Not yet implemented

2. **Songbird Client Wrapper**
   - Location: Should be in `biomeOS/src/primal_clients/songbird_client.rs`
   - Status: Exists but may need UPA support

3. **BearDog Client Wrapper**
   - Location: Should be in `biomeOS/src/primal_clients/beardog_client.rs`
   - Status: Exists but needs BTSP API methods

4. **Running Primal Instances**
   - Songbird: Not running with mDNS
   - BearDog: Not running with BTSP enabled

---

## Implementation Path

### Phase 1: Study & Document ✅ (This document!)
- Study phase1 implementation
- Document actual APIs
- Understand UPA abstraction

### Phase 2: Port UPA Client
```rust
// biomeOS/src/primal_clients/upa_client.rs

pub struct UpaClient {
    endpoint: String,
    // Protocol detection/abstraction
}

impl UpaClient {
    pub async fn register_node(&self, req: RegisterRequest) -> Result<String>;
    pub async fn find_peer(&self, name: &str) -> Result<PeerInfo>;
}
```

### Phase 3: Extend BearDog Client
```rust
// biomeOS/src/primal_clients/beardog_client.rs

impl BeardogClient {
    pub async fn establish_btsp_tunnel(&self, peer: PeerEndpoint) -> Result<TunnelId>;
    pub async fn send_btsp_message(&self, tunnel_id: &str, data: &[u8]) -> Result<()>;
    pub async fn close_btsp_tunnel(&self, tunnel_id: &str) -> Result<()>;
}
```

### Phase 4: Integrate with Songbird
```rust
// biomeOS/src/primal_clients/songbird_client.rs

impl SongbirdClient {
    pub async fn register_node(&self, node: NodeInfo) -> Result<String>;
    pub async fn discover_peers(&self, capability: &str) -> Result<Vec<PeerInfo>>;
}
```

### Phase 5: Build Demo
- Start Songbird tower (mDNS/UDP)
- Start BearDog instances
- Run biomeOS demo
- Establish BTSP tunnel
- Send encrypted messages

---

## Dependencies Needed

### From phase1/beardog
```toml
beardog-tunnel = { path = "../../beardog/crates/beardog-tunnel" }
beardog-capabilities = { path = "../../beardog/crates/beardog-capabilities" }
beardog-genetics = { path = "../../beardog/crates/beardog-genetics" }
```

### Alternatives for biomeOS
1. **Link to phase1 directly** (quick, but couples to phase1)
2. **Port UPA to biomeOS** (clean, but more work)
3. **Use primalBins** (treat as external services via HTTP/UDP)

**Recommendation**: Option 3 - Use primalBins as external services
- Aligns with "primals are external" architecture
- No code dependencies on phase1
- Real integration testing
- Production-realistic

---

## Environment Setup for Demo

### Start Songbird
```bash
cd /home/eastgate/Development/ecoPrimals/primalBins/
# If songbird-orchestrator exists:
./songbird-orchestrator --port 9090

# Set environment
export PRIMAL_SONGBIRD_ENDPOINT="mdns://songbird-tower.local"
export PRIMAL_SONGBIRD_CAPABILITIES="orchestration,p2p,federation"
```

### Start BearDog
```bash
cd /home/eastgate/Development/ecoPrimals/primalBins/
# If beardog-hsm exists:
./beardog-hsm --btsp-enabled --port 9091

# Set environment
export PRIMAL_BEARDOG_ENDPOINT="http://localhost:9091"
export PRIMAL_BEARDOG_CAPABILITIES="encryption,btsp,hsm"
```

### Run biomeOS Demo
```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS/showcase/03-p2p-coordination/01-btsp-tunnel-coordination/
./demo.sh
```

---

## Next Steps

1. **Verify primal binaries** in `../../primalBins/`
2. **Port UPA client** or use HTTP wrappers
3. **Extend BearDog client** with BTSP methods
4. **Test with real primals** running
5. **Document actual behavior** (update this file)

---

## References

- `phase1/beardog/showcase/02-ecosystem-integration/01-songbird-btsp/` - Full working implementation
- `phase1/beardog/showcase/00-local-primal/06-btsp-tunnel/` - BTSP concepts
- `PRIMAL_GAPS.md` - Songbird mDNS/UDP discovery gap
- `specs/UNIVERSAL_ADAPTER_SPECIFICATION.md` - UPA design

---

**Status**: Gap Discovery Complete ✅  
**Next**: Implement UPA client or HTTP wrappers  
**Blocked By**: Running primal instances  

🔍 **APIs documented from proven phase1 implementation!**

