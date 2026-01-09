# 🌸 petalTongue Team Handoff - biomeOS Integration

**Date**: January 9, 2026  
**From**: biomeOS Team  
**To**: petalTongue Team  
**Status**: Ready for Evolution  
**Priority**: 🟡 MEDIUM (Neural API paused until UI functional)

---

## 🎯 **Executive Summary**

**Goal**: Integrate petalTongue as the universal user interface for biomeOS, enabling visual topology rendering, multi-modal interaction, and real-time ecosystem monitoring.

**Current Status**:
- ✅ petalTongue v0.4.0 Production Ready (A+ 9.5/10)
- ✅ Binaries in `plasmidBin/primals/`
- ✅ UI niche manifest created (`niches/ui.toml`)
- ✅ Integration plan documented
- ⏳ **BLOCKED**: biomeOS topology API not yet implemented

**Decision**: Neural API evolution is **paused** until we have a functioning UI to interact with the ecosystem. petalTongue integration is now the **priority path forward**.

---

## 🚀 **What We Need from petalTongue**

### **Phase 1: HTTP Integration** (This Week)

#### **1. Topology Data Consumer**

**Current State**: petalTongue discovers providers via:
- mDNS/Multicast (zero-config)
- Environment hints (`PETALTONGUE_DISCOVERY_HINTS`)
- HTTP probing

**What We Need**:
- ✅ Verify HTTP discovery works with biomeOS
- ✅ Test with mock topology data
- ✅ Document expected topology data format

**Example**:
```bash
# Start petalTongue with biomeOS hint
export PETALTONGUE_DISCOVERY_HINTS="http://localhost:3000"
./plasmidBin/primals/petal-tongue
```

---

#### **2. Topology Data Format**

**What biomeOS Will Provide**: `/api/v1/topology`

```json
{
  "primals": [
    {
      "id": "beardog-node-alpha",
      "type": "beardog",
      "capabilities": ["security", "encryption", "identity"],
      "health": "healthy",
      "endpoints": {
        "unix_socket": "/tmp/beardog-node-alpha.sock",
        "http": null
      },
      "metadata": {
        "version": "v0.15.2",
        "family_id": "nat0",
        "node_id": "node-alpha"
      }
    },
    {
      "id": "songbird-node-alpha",
      "type": "songbird",
      "capabilities": ["discovery", "p2p", "btsp"],
      "health": "healthy",
      "endpoints": {
        "unix_socket": "/tmp/songbird-node-alpha.sock",
        "http": null
      },
      "metadata": {
        "version": "v3.19.0",
        "family_id": "nat0",
        "node_id": "node-alpha"
      }
    }
  ],
  "connections": [
    {
      "from": "songbird-node-alpha",
      "to": "beardog-node-alpha",
      "type": "capability_invocation",
      "capability": "encryption",
      "metrics": {
        "request_count": 42,
        "avg_latency_ms": 2.3
      }
    }
  ],
  "health_status": {
    "overall": "healthy",
    "primals_healthy": 2,
    "primals_total": 2
  }
}
```

**Action Items**:
1. Update petalTongue to consume this format
2. Render primals as graph nodes
3. Render connections as graph edges
4. Display health status

---

### **Phase 2: Unix Socket Evolution** (Week 2-3)

#### **1. Add Unix Socket JSON-RPC Server**

**Goal**: Port-free architecture for petalTongue

**Implementation**:
```rust
// In petalTongue
use tokio::net::UnixListener;

pub struct PetalTongueServer {
    socket_path: PathBuf,
    // ... other fields
}

impl PetalTongueServer {
    pub async fn start(&self) -> Result<()> {
        let socket_path = format!("/tmp/petaltongue-{}.sock", self.node_id);
        let listener = UnixListener::bind(&socket_path)?;
        
        loop {
            let (stream, _) = listener.accept().await?;
            // Handle JSON-RPC requests
        }
    }
}
```

**Required APIs**:
1. `get_capabilities` - Return petalTongue capabilities
2. `get_topology` - Return current topology view
3. `render_graph` - Render topology to specified format
4. `get_health` - Return petalTongue health status

---

#### **2. API Specification**

**`get_capabilities`**:
```json
{
  "jsonrpc": "2.0",
  "method": "get_capabilities",
  "params": {},
  "id": 1
}

// Response:
{
  "jsonrpc": "2.0",
  "result": {
    "capabilities": [
      "ui.desktop-interface",
      "ui.primal-interaction",
      "visualization.graph-rendering",
      "visualization.real-time-topology",
      "visualization.flow-animation",
      "ui.multi-modal",
      "ui.awakening-experience"
    ],
    "version": "v0.4.0",
    "node_id": "petaltongue-node-alpha"
  },
  "id": 1
}
```

**`render_graph`**:
```json
{
  "jsonrpc": "2.0",
  "method": "render_graph",
  "params": {
    "topology": { /* topology data */ },
    "format": "svg",  // or "png", "terminal", "json"
    "options": {
      "width": 1920,
      "height": 1080,
      "theme": "dark"
    }
  },
  "id": 2
}

// Response:
{
  "jsonrpc": "2.0",
  "result": {
    "format": "svg",
    "data": "<svg>...</svg>",  // or base64 for binary formats
    "metadata": {
      "nodes": 5,
      "edges": 8,
      "render_time_ms": 42
    }
  },
  "id": 2
}
```

---

### **Phase 3: Advanced Features** (Week 4+)

#### **1. Network to Toadstool for GPU Rendering**

**Goal**: Offload heavy rendering to GPU-accelerated Toadstool

**Implementation**:
```rust
// In petalTongue
pub async fn render_with_gpu(&self, topology: &Topology) -> Result<RenderedFrame> {
    // 1. Discover Toadstool via SPDP
    let toadstool = self.spdp.discover_by_capability("compute.gpu").await?;
    
    // 2. Create render workload
    let workload = RenderWorkload {
        scene: topology_to_3d_scene(topology),
        quality: RenderQuality::High,
        output_format: OutputFormat::RGBA8,
        shader: ShaderType::RayTracing,
    };
    
    // 3. Submit to Toadstool
    let rendered = toadstool.submit_workload(workload).await?;
    
    // 4. Display to user
    Ok(rendered)
}
```

**Use Cases**:
- High-end 3D visualizations
- VR rendering
- Ray tracing
- Complex shaders

---

#### **2. Real-Time Updates via WebSocket**

**Goal**: Live topology changes, flow animations

**Implementation**:
```rust
// In petalTongue
pub async fn subscribe_to_topology_updates(&self) -> Result<()> {
    let ws_client = WebSocketClient::connect("ws://biomeos/topology").await?;
    
    ws_client.on_message(|change: TopologyChange| {
        match change {
            TopologyChange::PrimalAdded(primal) => {
                self.add_node_to_graph(primal);
            }
            TopologyChange::PrimalRemoved(id) => {
                self.remove_node_from_graph(id);
            }
            TopologyChange::ConnectionEstablished(from, to) => {
                self.add_edge_to_graph(from, to);
            }
            // ... other changes
        }
        self.re_render();
    });
    
    Ok(())
}
```

---

## 📋 **Action Items for petalTongue Team**

### **Immediate (This Week)**

| Task | Priority | Estimated Time | Status |
|------|----------|----------------|--------|
| Test HTTP discovery with biomeOS | 🔴 HIGH | 2 hours | ⏳ Pending |
| Document topology data format | 🔴 HIGH | 1 hour | ⏳ Pending |
| Create mock topology endpoint for testing | 🔴 HIGH | 3 hours | ⏳ Pending |
| Test rendering with mock data | 🔴 HIGH | 4 hours | ⏳ Pending |

### **Short-Term (Week 2-3)**

| Task | Priority | Estimated Time | Status |
|------|----------|----------------|--------|
| Add Unix socket JSON-RPC server | 🔴 HIGH | 1 day | ⏳ Pending |
| Implement `get_capabilities` API | 🔴 HIGH | 2 hours | ⏳ Pending |
| Implement `render_graph` API | 🔴 HIGH | 4 hours | ⏳ Pending |
| Implement `get_health` API | 🟡 MEDIUM | 2 hours | ⏳ Pending |
| HTTP deprecation (port-free mode) | 🟡 MEDIUM | 4 hours | ⏳ Pending |
| Integration tests with biomeOS | 🔴 HIGH | 1 day | ⏳ Pending |

### **Future (Week 4+)**

| Task | Priority | Estimated Time | Status |
|------|----------|----------------|--------|
| Network to Toadstool for GPU rendering | 🟢 LOW | 2 days | ⏳ Future |
| Real-time updates via WebSocket | 🟢 LOW | 1 day | ⏳ Future |
| Advanced visualizations (3D, VR) | 🟢 LOW | 1 week | ⏳ Future |

---

## 🤝 **Coordination with biomeOS Team**

### **What biomeOS Will Provide**

1. **Topology API** (`/api/v1/topology`)
   - HTTP endpoint (Phase 1)
   - Unix socket (Phase 2)
   - WebSocket (Phase 3)

2. **SPDP Integration**
   - petalTongue discovered via capability: `"ui.desktop-interface"`
   - Secure discovery with BearDog verification

3. **Test Environment**
   - Mock topology data for testing
   - Local deployment for E2E testing

### **What We Need from petalTongue**

1. **Topology Consumer**
   - Parse topology JSON
   - Render as graph
   - Handle updates

2. **Unix Socket Server**
   - JSON-RPC 2.0
   - 3+ APIs (`get_capabilities`, `render_graph`, `get_health`)

3. **Documentation**
   - API reference
   - Integration guide
   - Example usage

---

## 🎯 **Success Criteria**

### **Phase 1 Complete When**:
- ✅ petalTongue discovers biomeOS via HTTP
- ✅ petalTongue fetches topology data
- ✅ petalTongue renders topology as graph
- ✅ E2E test passing (biomeOS → petalTongue)

### **Phase 2 Complete When**:
- ✅ petalTongue on Unix socket JSON-RPC
- ✅ biomeOS discovers petalTongue via SPDP
- ✅ Port-free architecture working
- ✅ Integration tests passing

### **Phase 3 Complete When**:
- ✅ petalTongue networks to Toadstool for GPU rendering
- ✅ Real-time updates via WebSocket
- ✅ Advanced visualizations working

---

## 📚 **References**

### **petalTongue**
- **Location**: `/home/eastgate/Development/ecoPrimals/phase2/petalTongue`
- **Binaries**: `plasmidBin/primals/petal-tongue`, `plasmidBin/primals/petal-tongue-headless`
- **Status**: v0.4.0 Production Ready (A+ 9.5/10)
- **README**: `petalTongue/README.md`
- **STATUS**: `petalTongue/STATUS.md`

### **biomeOS**
- **Location**: `/home/eastgate/Development/ecoPrimals/phase2/biomeOS`
- **Integration Plan**: `docs/PETALTONGUE_BIOMEOS_INTEGRATION_PLAN.md`
- **UI Niche**: `niches/ui.toml`
- **SPDP**: `specs/SECURE_PRIMAL_DISCOVERY_PROTOCOL.md`
- **Neural API**: `specs/NEURAL_API_IMPLEMENTATION_PHASES.md`

### **Toadstool** (for GPU rendering)
- **Location**: `/home/eastgate/Development/ecoPrimals/phase1/toadstool`
- **Handoff**: `docs/jan4-session/TOADSTOOL_UNIX_SOCKET_HANDOFF_JAN9.md`

---

## 🎊 **Bottom Line**

**petalTongue is production ready!**

✅ **v0.4.0**: A+ grade, 536+ tests, zero mocks, 100% safe Rust  
✅ **Multi-modal**: Visual + audio + text  
✅ **6 display backends**: All working  
✅ **TRUE PRIMAL**: Zero hardcoding, capability-based discovery  
✅ **In-code rendering**: Can network to Toadstool for GPU power  

**What We Need**:
1. Test HTTP discovery with biomeOS
2. Evolve to Unix socket JSON-RPC (port-free)
3. Integrate with SPDP (secure discovery)

**Timeline**: 3-4 weeks to full integration

**Priority**: 🟡 **MEDIUM** (Neural API paused until UI functional)

---

**The face of ecoPrimals is ready to shine! Let's make it happen!** 🌸✨

🌱 **biomeOS + petalTongue: Complete Ecosystem with Universal UI** 🎉

