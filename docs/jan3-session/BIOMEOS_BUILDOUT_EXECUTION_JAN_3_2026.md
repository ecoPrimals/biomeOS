# 🌿 biomeOS Development - Build-Out Execution Plan

**Date**: January 3, 2026  
**Status**: 🚀 **EXECUTING IN PARALLEL**  
**Focus**: Core API & USB Spore Integration while PetalTongue builds UI

---

## 🎯 Current State

### What We Have ✅

**API Server** (`biomeos-api`):
- Port: 3000
- Endpoints:
  - `GET /api/v1/health` ✅
  - `GET /api/v1/primals` ✅ (Mock + Live)
  - `GET /api/v1/topology` ✅ (Mock only)
  - `GET /api/v1/trust/identity` ✅ (Proxy to BearDog)
  - `POST /api/v1/trust/evaluate` ✅ (Proxy to BearDog)

**Crates**:
- `biomeos-api` - REST API server
- `biomeos-core` - Core types and clients
- `biomeos-types` - Shared types
- `biomeos-deploy` - Deployment utilities
- `biomeos-cli` - Command-line interface
- `biomeos-boot` - System bootstrap

**Status**: Mock mode working, live discovery partial

---

## 🎯 Build-Out Priorities

### Phase 1: Complete Live Discovery (Today)

**Goal**: Make `/api/v1/primals` and `/api/v1/topology` work in live mode

**Tasks**:

1. ✅ **Enhance Live Discovery** (2 hours)
   - Connect to running BearDog (localhost:9000)
   - Connect to running Songbird (localhost:8080)
   - Query health and capabilities
   - Return real primal data

2. ✅ **Live Topology Builder** (2 hours)
   - Query Songbird for peer list
   - Query BearDog for trust relationships
   - Build graph from real data
   - Replace mock topology

3. **Real-Time Events API** (2 hours)
   - New endpoint: `GET /api/v1/events/stream`
   - WebSocket or Server-Sent Events
   - Stream primal discoveries
   - Stream trust decisions
   - Stream health changes

---

### Phase 2: USB Spore API (Next 2 Days)

**Goal**: API endpoints for USB spore deployment

**New Endpoints**:

1. **USB Detection**:
   ```
   GET /api/v1/usb/detect
   Response: {
     "detected": true,
     "mount_point": "/media/usb",
     "spore_version": "v11.0",
     "family_id": "iidn",
     "encrypted_seed": true
   }
   ```

2. **Tower Deployment**:
   ```
   POST /api/v1/deploy/tower
   Body: {
     "tower_name": "tower1",
     "usb_path": "/media/usb/biomeOS-LAN-Deploy",
     "passphrase": "..." // for encrypted seed
   }
   Response: {
     "status": "deploying",
     "tower_id": "tower1-abc123",
     "progress": 0
   }
   ```

3. **Deployment Progress**:
   ```
   GET /api/v1/deploy/status/{tower_id}
   Response: {
     "status": "running",
     "progress": 100,
     "services": {
       "beardog": {"status": "healthy", "port": 9000},
       "songbird": {"status": "healthy", "port": 8080}
     }
   }
   ```

4. **Family Lineage**:
   ```
   GET /api/v1/family/lineage
   Response: {
     "family_id": "iidn",
     "towers": [
       {"name": "tower1", "ip": "192.168.1.144", "status": "healthy"},
       {"name": "tower2", "ip": "192.168.1.134", "status": "healthy"}
     ],
     "encryption_status": "active",
     "auto_trust": true
   }
   ```

---

### Phase 3: Primal Integration SDK (Week 2)

**Goal**: Make it easy for new primals to integrate

**Features**:

1. **Primal Registration**:
   ```
   POST /api/v1/register
   Body: {
     "primal_id": "my-primal",
     "capabilities": ["storage", "versioning"],
     "endpoint": "http://localhost:9100",
     "family_id": "iidn"
   }
   ```

2. **Capability Query**:
   ```
   POST /api/v1/query/capability
   Body: {
     "capability": "storage",
     "filters": {"family_id": "iidn"}
   }
   Response: {
     "primals": [
       {"id": "nestgate-local", "endpoint": "...", "trust_level": 3}
     ]
   }
   ```

3. **Cross-Primal RPC**:
   ```
   POST /api/v1/rpc/call
   Body: {
     "target_primal": "beardog-local",
     "method": "encrypt",
     "params": {"data": "..."}
   }
   Response: {
     "result": "..."
   }
   ```

---

## 🔧 Implementation Plan

### Task 1: Complete Live Discovery

**File**: `crates/biomeos-api/src/handlers/live_discovery.rs`

**Current Status**: Partially implemented

**Need to Add**:
```rust
// Query Songbird for peer list
pub async fn discover_from_songbird() -> Result<Vec<LivePrimalInfo>> {
    let songbird_url = env::var("SONGBIRD_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:8080".to_string());
    
    // Songbird discovery API (if available)
    // OR use UDP multicast listener
    
    Ok(peers)
}

// Query BearDog for trust info
pub async fn get_trust_info(primal_id: &str) -> Result<TrustInfo> {
    let beardog_url = env::var("BEARDOG_ENDPOINT")
        .unwrap_or_else(|_| "http://localhost:9000".to_string());
    
    let response = reqwest::Client::new()
        .post(format!("{}/api/v1/trust/evaluate", beardog_url))
        .json(&json!({
            "peer_id": primal_id,
            "peer_tags": []
        }))
        .send()
        .await?;
    
    Ok(trust_info)
}
```

### Task 2: Live Topology

**File**: `crates/biomeos-api/src/handlers/topology.rs`

**Replace Mock**:
```rust
pub async fn get_topology(
    State(state): State<Arc<AppState>>,
) -> Result<Json<TopologyResponse>, ApiError> {
    if state.mock_mode {
        // Keep mock for testing
        return Ok(mock_topology());
    }
    
    // Get primals
    let primals = live_discovery::discover_all_primals().await;
    
    // Build nodes
    let nodes: Vec<TopologyNode> = primals
        .iter()
        .map(|p| TopologyNode {
            id: p.id.clone(),
            label: p.name.clone(),
            node_type: p.primal_type.clone(),
            capabilities: p.capabilities.clone(),
            family_id: p.family_id.clone(),
        })
        .collect();
    
    // Build edges from connections
    let edges = build_edges_from_connections(&primals).await;
    
    Ok(Json(TopologyResponse {
        nodes,
        edges,
        mode: "live".to_string(),
    }))
}

async fn build_edges_from_connections(
    primals: &[LivePrimalInfo]
) -> Vec<TopologyEdge> {
    let mut edges = Vec::new();
    
    // Query each primal for its connections
    for primal in primals {
        if primal.primal_type == "orchestration" {
            // Songbird knows about connections
            let connections = query_songbird_connections(&primal.endpoint).await;
            edges.extend(connections);
        }
    }
    
    // Add trust relationships
    let trust_edges = query_trust_relationships(primals).await;
    edges.extend(trust_edges);
    
    edges
}
```

### Task 3: Events Stream

**New File**: `crates/biomeos-api/src/handlers/events.rs`

```rust
use axum::{
    extract::State,
    response::sse::{Event, KeepAlive, Sse},
};
use futures::stream::{self, Stream};
use std::time::Duration;
use tokio_stream::StreamExt;

pub async fn event_stream(
    State(state): State<Arc<AppState>>,
) -> Sse<impl Stream<Item = Result<Event, Infallible>>> {
    let stream = stream::repeat_with(|| {
        // Poll for changes
        let event = check_for_events();
        Event::default().data(serde_json::to_string(&event).unwrap())
    })
    .throttle(Duration::from_secs(1));
    
    Sse::new(stream).keep_alive(KeepAlive::default())
}

#[derive(Serialize)]
struct EcosystemEvent {
    event_type: String, // "primal_discovered", "trust_changed", etc
    timestamp: u64,
    data: serde_json::Value,
}
```

---

## 📊 Priority Matrix

| Task | Priority | Effort | Impact | Dependencies |
|------|----------|--------|--------|--------------|
| Live Discovery | P0 | 2h | High | Running primals |
| Live Topology | P0 | 2h | High | Live discovery |
| Events Stream | P1 | 2h | Medium | Live discovery |
| USB Detection | P1 | 1h | High | USB mounted |
| Tower Deployment | P1 | 4h | High | USB detection |
| Family Lineage | P2 | 2h | Medium | BearDog API |
| Primal Registration | P2 | 3h | Medium | None |
| Cross-Primal RPC | P3 | 4h | Medium | Registration |

---

## 🎯 Today's Goals

### Morning (4 hours)
1. ✅ Complete live discovery
2. ✅ Implement live topology
3. ✅ Test with running ecosystem

### Afternoon (4 hours)
4. ⏳ Add events stream
5. ⏳ Start USB detection
6. ⏳ Test PetalTongue integration

---

## 🔌 Integration Points

### With PetalTongue
- `GET /api/v1/primals` - Already working ✅
- `GET /api/v1/topology` - Needs live mode ⏳
- `GET /api/v1/events/stream` - New endpoint ⏳

### With BearDog
- `GET /api/v1/trust/identity` - Proxy working ✅
- `POST /api/v1/trust/evaluate` - Proxy working ✅
- Need: Batch trust queries ⏳

### With Songbird
- Discovery: UDP multicast listener ⏳
- Topology: Connection graph API ⏳
- Status: Health check ✅

---

## 🚀 Execution Strategy

### 1. Start with What Works
- Current API is stable
- Mock mode is good for testing
- Focus on live mode additions

### 2. Incremental Enhancement
- Don't break existing endpoints
- Add live mode alongside mock
- Graceful fallback to mock

### 3. Test as We Go
- Verify each endpoint
- Test with PetalTongue
- Document as we build

---

## 📝 Success Criteria

### Phase 1 (Today)
- [ ] `/api/v1/primals` returns live data from BearDog + Songbird
- [ ] `/api/v1/topology` builds real graph
- [ ] Events stream operational
- [ ] PetalTongue sees live ecosystem

### Phase 2 (This Week)
- [ ] USB spore detection working
- [ ] Tower deployment from API
- [ ] Family lineage visible
- [ ] Multi-tower coordination

### Phase 3 (Next Week)
- [ ] New primals can self-register
- [ ] Capability queries working
- [ ] Cross-primal RPC functional
- [ ] SDK documentation complete

---

## 🎊 Bottom Line

**Current**: Mock API serving test data  
**Target**: Live API orchestrating real ecosystem  
**Timeline**: 1 week to production-ready  

**Parallel Work**:
- biomeOS: Building API & infrastructure
- PetalTongue: Building UI & visualization
- BearDog Team: Adding PORT=0 & encrypted seeds
- Songbird Team: UDP attestations fix

**Result**: Complete ecosystem with beautiful interface! 🌿🌸

---

**Status**: 🚀 **READY TO EXECUTE**  
**Next**: Start with live discovery implementation

**Location**: `docs/jan3-session/BIOMEOS_BUILDOUT_EXECUTION_JAN_3_2026.md`

