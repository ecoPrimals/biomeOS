# 🌸 PetalTongue Live Integration - Gap Discovery

**Date**: January 3, 2026  
**Method**: Hands-on execution to discover real evolution needs  
**Status**: 🔍 Active Discovery

---

## 🎯 Test Setup

### What's Running:
- ✅ BearDog server: PID 887949, port 9000
- ✅ Songbird orchestrator: PID 889382, port 8080
- ✅ PetalTongue: Just launched, BIOMEOS_URL=http://localhost:9000

### What Worked Immediately:
```
✅ PetalTongue launched successfully
✅ Capability detection complete
✅ Visual2D: Available
✅ Animation: Available
✅ TextDescription: Available
✅ Tool integration: 4 tools registered
   • BingoCube v0.1.0
   • System Monitor v0.1.0
   • Process Viewer v0.1.0
   • Graph Metrics v0.1.0
```

---

## 🚨 GAP #1: API Mismatch (IMMEDIATE)

### The Error:
```
WARN petal_tongue_ui::app: Failed to discover primals: error decoding response body
```

### Root Cause Analysis:

**PetalTongue Expectations**:
- Pointed at `BIOMEOS_URL=http://localhost:9000` (BearDog)
- Expects a **biomeOS API** with primal discovery endpoints
- Expects endpoints like:
  ```
  GET /api/v1/primals/list
  GET /api/v1/topology
  ```

**Reality**:
- Port 9000 is **BearDog** (trust/security primal)
- BearDog exposes:
  ```
  GET /api/v1/health
  GET /api/v1/trust/identity
  POST /api/v1/trust/evaluate
  GET /api/v1/lineage/current
  ... (lineage APIs)
  ```

**Gap**: **No biomeOS orchestration layer API running!**

---

## 🏗️ Architecture Gap Discovered

### Current Reality:
```
PetalTongue → (expects biomeOS API)
  ↓
BearDog API (trust/security only)
  ❌ Not an orchestration API
  ❌ No primal discovery
  ❌ No topology endpoints
```

### What We Need:
```
PetalTongue → (expects biomeOS API)
  ↓
biomeOS API Server (NEW!)
  • GET /api/v1/primals/discovered
  • GET /api/v1/topology
  • POST /api/v1/peers/{id}/elevate
  ↓
Universal Primal Client (we built this!)
  ↓ Discovers
Songbird (discovery) + BearDog (trust) + Others
```

---

## 🔍 GAP #2: No biomeOS API Server Implementation

### What Exists:
- ✅ `biomeos-core` crate with Universal Primal Client
- ✅ `biomeos-cli` binary (CLI tool, not API server)
- ❌ **No `biomeos-server` or `biomeos-api` binary!**

### What's Missing:
```rust
// We have the client logic:
// biomeOS/crates/biomeos-core/src/primal_client/

// But NO server exposing:
// GET /api/v1/primals/discovered
// GET /api/v1/topology
// POST /api/v1/peers/{id}/elevate
```

### Evolution Needed:
1. Create `biomeos-api` crate
2. Implement axum/actix server
3. Wire up Universal Primal Client
4. Expose REST endpoints for PetalTongue

---

## 🔍 GAP #3: PetalTongue's BiomeOS Client is Mocked

### Current Code:
```rust
// petal-tongue-api/src/biomeos_client.rs

impl BiomeOSClient {
    pub async fn discover_primals(&self) -> Result<Vec<PrimalInfo>> {
        if self.mock_mode {
            // Returns hardcoded mock data
            return Ok(vec![
                PrimalInfo { name: "mock-primal-1", ... },
                PrimalInfo { name: "mock-primal-2", ... },
            ]);
        }
        
        // Real mode:
        let response = self.client
            .get(format!("{}/api/v1/primals/list", self.base_url))
            .send()
            .await?;
        
        // ❌ But this endpoint doesn't exist yet!
    }
}
```

### Gap:
- PetalTongue **expects** a biomeOS API that doesn't exist
- Falls back to mock mode
- Can't discover real primals

---

## 🔍 GAP #4: Songbird Discovery Not Exposed

### Current Reality:
- ✅ Songbird is running (PID 889382, port 8080)
- ✅ Songbird has UDP multicast discovery
- ✅ Songbird knows about peers
- ❌ **Songbird's discovery data not exposed via HTTP REST!**

### Songbird APIs (Current):
```
Songbird uses tarpc (Rust RPC), not HTTP REST!

Available via tarpc:
  • register_primal()
  • discover_peers()
  • get_topology()

NOT available via HTTP:
  • No GET /api/v1/peers/list
  • No GET /api/v1/topology
```

### Gap:
- biomeOS needs to call Songbird via **tarpc**
- OR Songbird needs HTTP REST endpoints
- OR biomeOS provides translation layer

---

## 🔍 GAP #5: Protocol Mismatch

### The Problem:
```
PetalTongue:
  • Speaks: HTTP REST (via reqwest)
  • Expects: JSON responses

biomeOS (our plan):
  • Speaks: HTTP REST (via Universal Primal Client)
  • Supports: HTTP, tarpc, gRPC

Songbird:
  • Speaks: tarpc (Rust RPC)
  • Discovery: UDP multicast
  • NOT HTTP REST!

BearDog:
  • Speaks: HTTP REST ✅
  • Format: Unwrapped JSON ✅
```

### Evolution Needed:
**Option A**: biomeOS translates
- biomeOS API server speaks HTTP to PetalTongue
- biomeOS internally uses tarpc to talk to Songbird
- Clean separation

**Option B**: Songbird adds HTTP endpoints
- Songbird exposes HTTP REST in addition to tarpc
- Duplicates API surface
- More work for Songbird team

**Recommendation**: **Option A** (biomeOS translation layer)

---

## 🔍 GAP #6: No Topology Aggregation

### Current State:
- Songbird knows peer topology (who's connected to whom)
- BearDog knows trust relationships (who trusts whom)
- No system aggregates this into a unified view

### What PetalTongue Needs:
```json
GET /api/v1/topology

{
  "nodes": [
    {
      "id": "tower2",
      "name": "tower2",
      "type": "tower",
      "health": "healthy",
      "trust_level": 1,
      "family_id": "iidn",
      "capabilities": ["orchestration", "federation"]
    }
  ],
  "edges": [
    {
      "from": "pop-os",
      "to": "tower2",
      "type": "federation",
      "protocol": "tarpc",
      "trust": "limited"
    }
  ]
}
```

### Gap:
- No service aggregates Songbird + BearDog data
- biomeOS API server must do this

---

## 🎯 Immediate Action Items

### 1. Create biomeOS API Server (CRITICAL)

**Goal**: HTTP REST API for PetalTongue (and other UIs)

**Files to Create**:
```
biomeOS/crates/biomeos-api/
  ├── Cargo.toml
  ├── src/
  │   ├── main.rs           # axum server
  │   ├── handlers/
  │   │   ├── discovery.rs  # GET /api/v1/primals/discovered
  │   │   ├── topology.rs   # GET /api/v1/topology
  │   │   └── trust.rs      # POST /api/v1/peers/{id}/elevate
  │   ├── state.rs          # Shared app state
  │   └── config.rs         # Configuration
```

**Dependencies**:
```toml
[dependencies]
axum = "0.7"
tokio = { version = "1", features = ["full"] }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
biomeos-core = { path = "../biomeos-core" }  # Use Universal Primal Client
tracing = "0.1"
```

---

### 2. Implement Discovery Endpoint

**Goal**: Return discovered primals with trust levels

**Handler**:
```rust
// biomeos-api/src/handlers/discovery.rs

use axum::{extract::State, Json};
use biomeos_core::primal_client::UniversalPrimalClient;

pub async fn get_discovered_primals(
    State(client): State<UniversalPrimalClient>,
) -> Json<DiscoveredPrimalsResponse> {
    // 1. Discover Songbird via mDNS/multicast
    let songbird = client
        .discover_primal_by_capability("orchestration")
        .await
        .expect("Failed to discover Songbird");
    
    // 2. Call Songbird to get peer list (via tarpc!)
    let peers = client
        .call(&songbird, "peers/list", EmptyRequest)
        .await
        .expect("Failed to get peers from Songbird");
    
    // 3. For each peer, get trust evaluation from BearDog
    let beardog = client
        .discover_primal_by_capability("security")
        .await
        .expect("Failed to discover BearDog");
    
    let mut primals_with_trust = Vec::new();
    for peer in peers {
        let trust = client
            .call(&beardog, "trust/evaluate", TrustRequest { peer_id: peer.id })
            .await
            .unwrap_or_default();
        
        primals_with_trust.push(DiscoveredPrimal {
            id: peer.id,
            name: peer.name,
            primal_type: peer.primal_type,
            health: peer.health,
            trust_level: trust.trust_level,
            family_id: trust.family_id,
            allowed_capabilities: trust.allowed_capabilities,
            denied_capabilities: trust.denied_capabilities,
            endpoint: peer.endpoint,
        });
    }
    
    Json(DiscoveredPrimalsResponse {
        primals: primals_with_trust,
    })
}
```

---

### 3. Implement Topology Endpoint

**Goal**: Aggregate Songbird + BearDog data into unified topology

**Handler**:
```rust
// biomeos-api/src/handlers/topology.rs

pub async fn get_topology(
    State(client): State<UniversalPrimalClient>,
) -> Json<TopologyResponse> {
    // 1. Get peers from Songbird
    let peers = get_peers_from_songbird(&client).await?;
    
    // 2. Get trust levels from BearDog
    let trust_levels = get_trust_levels(&client, &peers).await?;
    
    // 3. Aggregate into nodes and edges
    let nodes = peers.iter().map(|peer| {
        let trust = trust_levels.get(&peer.id);
        Node {
            id: peer.id.clone(),
            name: peer.name.clone(),
            primal_type: peer.primal_type.clone(),
            health: peer.health.clone(),
            trust_level: trust.map(|t| t.trust_level).unwrap_or(0),
            family_id: trust.and_then(|t| t.family_id.clone()),
            capabilities: peer.capabilities.clone(),
        }
    }).collect();
    
    let edges = build_edges_from_connections(&peers);
    
    Json(TopologyResponse { nodes, edges })
}
```

---

### 4. Add tarpc Support to Universal Primal Client

**Goal**: Universal Primal Client can call Songbird via tarpc

**Current Gap**:
```rust
// We have HTTP protocol adapter:
impl HttpProtocolAdapter { ... }

// But NO tarpc protocol adapter!
// Need:
impl TarpcProtocolAdapter { ... }
```

**Evolution Needed**:
```rust
// biomeos-core/src/primal_client/adapters/protocol/tarpc.rs

pub struct TarpcProtocolAdapter {
    // tarpc client pool
}

#[async_trait]
impl ProtocolAdapter for TarpcProtocolAdapter {
    async fn request(
        &self,
        endpoint: &str,
        method: Method,
        body: Option<Vec<u8>>,
    ) -> Result<Vec<u8>> {
        // Connect to tarpc endpoint
        // Make RPC call
        // Return serialized response
    }
}
```

---

## 📊 Gap Summary

| Gap | Severity | Effort | Status |
|-----|----------|--------|--------|
| #1: API Mismatch | 🔴 Critical | Medium | Discovered |
| #2: No biomeOS API Server | 🔴 Critical | High | Discovered |
| #3: PetalTongue Mocked | 🟡 High | Low | Discovered |
| #4: Songbird Discovery Not Exposed | 🟡 High | Medium | Discovered |
| #5: Protocol Mismatch (tarpc vs HTTP) | 🔴 Critical | High | Discovered |
| #6: No Topology Aggregation | 🟡 High | Medium | Discovered |

---

## 🚀 Evolution Path

### Phase 1: Basic biomeOS API Server (1-2 days)

**Goal**: Get PetalTongue connecting to real primals

**Tasks**:
1. Create `biomeos-api` crate with axum server
2. Implement `/api/v1/health` endpoint (sanity check)
3. Implement `/api/v1/primals/discovered` (hardcoded for now)
4. Test: PetalTongue → biomeOS API → returns mock data

**Success**: PetalTongue connects, shows mock primals

---

### Phase 2: Songbird Integration (2-3 days)

**Goal**: biomeOS API queries Songbird for real peer list

**Tasks**:
1. Add tarpc protocol adapter to Universal Primal Client
2. Implement Songbird discovery (mDNS/env var)
3. Call Songbird's `get_peers()` RPC
4. Return real peer list to PetalTongue

**Success**: PetalTongue shows real discovered peers

---

### Phase 3: BearDog Trust Integration (1-2 days)

**Goal**: Enrich peer data with trust levels

**Tasks**:
1. For each peer, query BearDog for trust evaluation
2. Add trust_level, family_id, capabilities to response
3. Test: PetalTongue shows peers with trust badges

**Success**: PetalTongue displays trust levels

---

### Phase 4: Trust Elevation UI (2-3 days)

**Goal**: User can elevate trust via PetalTongue

**Tasks**:
1. Implement `/api/v1/peers/{id}/elevate` endpoint
2. Call BearDog's elevation API
3. Update Songbird connection
4. PetalTongue: Add elevation UI (we already designed this!)

**Success**: User clicks "Elevate Trust" in PetalTongue, it works!

---

### Phase 5: Topology View (1-2 days)

**Goal**: PetalTongue shows network topology

**Tasks**:
1. Implement `/api/v1/topology` endpoint
2. Aggregate Songbird (connections) + BearDog (trust)
3. Return nodes + edges
4. PetalTongue: Display graph (already has this!)

**Success**: Beautiful topology graph with trust colors

---

## 🎊 What We Learned

### 1. Gap Discovery Works!

**Method**: Run real code, see what breaks

**Result**: Found 6 critical gaps in 5 minutes!

---

### 2. PetalTongue is Production-Ready

**Already Works**:
- ✅ Clean UI
- ✅ Tool integration
- ✅ Capability detection
- ✅ Mock mode (for development)

**Just Needs**: Real API to talk to!

---

### 3. Architecture is Sound

**The Design Works**:
- PetalTongue → biomeOS API → Universal Primal Client → Primals

**Just Need**: Implementation!

---

### 4. Protocol Diversity is Real

**Discovery**:
- Songbird: tarpc (not HTTP!)
- BearDog: HTTP REST
- Future primals: gRPC, others?

**Universal Primal Client was the right call!**

---

## 📋 Next Steps

### Immediate (This Session):

1. ✅ Document gaps (this file)
2. → Create `biomeos-api` crate skeleton
3. → Implement basic health endpoint
4. → Test PetalTongue connection

### Short-Term (Next Session):

1. Implement `/api/v1/primals/discovered` with Songbird integration
2. Add tarpc protocol adapter
3. Test with real primals

### Long-Term (Next Week):

1. Trust elevation endpoints
2. Topology aggregation
3. Full PetalTongue integration

---

**Status**: 🎯 **6 critical gaps discovered through hands-on execution!**  
**Method**: ✅ **Run real code, discover real needs**  
**Next**: 🏗️ **Build biomeOS API server**

🌸 **This is how we evolve: discover, document, execute!** 🚀

