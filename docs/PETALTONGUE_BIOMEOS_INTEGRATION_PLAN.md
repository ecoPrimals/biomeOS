# 🌸 petalTongue + biomeOS Integration Plan

**Date**: January 9, 2026  
**Status**: Planning → Implementation  
**Version**: v1.0.0

---

## 🎯 **Overview**

**petalTongue** is a mature Phase 1 primal (v0.4.0 - Production Ready) that provides the **Universal User Interface (UUI)** for the ecoPrimals ecosystem. It's the "face" of ecoPrimals, enabling ANY user, anywhere, with any capability to interact with the ecosystem.

**Current Status**:
- ✅ v0.4.0 Production Ready (A+ 9.5/10)
- ✅ 536+ tests passing
- ✅ Zero production mocks
- ✅ 100% safe Rust
- ✅ Zero hardcoding (TRUE PRIMAL)
- ✅ Multi-modal rendering (visual, audio, text)
- ✅ 6 display backends working
- ✅ In-code render capabilities (can network to Toadstool for high-end rendering!)

---

## 🏗️ **Integration Architecture**

### **Current State: petalTongue as Standalone Primal**

```
┌─────────────────────────────────────────────────────────────────┐
│                      petalTongue Primal                          │
│                  (Universal User Interface)                      │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  Capabilities:                                                  │
│  • "visualization.graph-rendering"                              │
│  • "visualization.real-time-topology"                           │
│  • "visualization.flow-animation"                               │
│  • "ui.desktop-interface"                                       │
│  • "ui.primal-interaction"                                      │
│  • "ui.multi-modal" (visual + audio + text)                     │
│                                                                 │
│  Discovery Methods:                                             │
│  • mDNS/Multicast (zero-config, preferred)                      │
│  • Environment hints (PETALTONGUE_DISCOVERY_HINTS)              │
│  • HTTP probing (capability-based)                              │
│                                                                 │
│  Communication:                                                 │
│  • HTTP (current)                                               │
│  • Unix socket (future evolution)                               │
│  • WebSocket (real-time updates)                                │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
                              │
                  ┌───────────┴───────────┐
                  │                       │
                  ▼                       ▼
         ┌────────────────┐      ┌────────────────┐
         │    biomeOS     │      │  Any Primal    │
         │  (via SPDP)    │      │ (via Songbird) │
         └────────────────┘      └────────────────┘
```

---

## 🎨 **petalTongue Features for biomeOS**

### **1. Multi-Modal Rendering** ✅ Complete
- **Visual**: Terminal, SVG, PNG, native GUI (egui)
- **Audio**: Signature tones, music, soundscape, chimes
- **Text**: JSON, descriptions, logs
- **Use Case**: Accessible UI for all users (including blind users via audio)

### **2. Display Backends** ✅ All 6 Working
1. **ToadStool HTTP**: GPU-accelerated rendering via Toadstool
2. **VNC**: File-based protocol (no TCP needed yet)
3. **WebSocket**: JSON streaming for real-time updates
4. **Software Window**: Pure Rust rendering (works everywhere)
5. **Framebuffer**: Direct Linux console rendering
6. **External**: X11/Wayland/Windows/macOS native

### **3. Awakening Experience** ✅ Complete
- 4-stage journey (12 seconds)
- Multi-modal coordination (visual + audio + text)
- Signature flower opening animation
- Tutorial mode (works standalone)

### **4. TRUE PRIMAL Architecture** ✅ Validated
- Zero hardcoded primal dependencies
- Infant Discovery Pattern (zero knowledge at start)
- Capability-based discovery (not name-based)
- Runtime service discovery
- Graceful degradation

### **5. In-Code Rendering** ✅ Ready
- Can render locally (Pure Rust)
- Can delegate to Toadstool for GPU acceleration
- Adaptive rendering based on available compute

---

## 🔌 **Integration Patterns**

### **Pattern 1: Standalone Primal** (Recommended for Now)

**Use Case**: petalTongue as independent UI service

```
biomeOS → SPDP → Discover petalTongue → Use UI APIs
```

**Advantages**:
- ✅ Separation of concerns
- ✅ Independent scaling
- ✅ Can serve multiple consumers
- ✅ Easier to evolve independently

**Implementation**:
1. petalTongue runs as standalone primal
2. biomeOS discovers via SPDP (capability: "ui.desktop-interface")
3. biomeOS provides topology data via HTTP/Unix socket
4. petalTongue renders and displays

---

### **Pattern 2: Embedded in biomeOS** (Future Chimera)

**Use Case**: Tightly coupled UI for specific biomeOS deployments

```
biomeOS (with embedded petalTongue) → Chimera pattern
```

**Advantages**:
- ✅ Tighter coupling for specific use cases
- ✅ Reduced network overhead
- ✅ Simplified deployment

**Implementation**:
1. biomeOS embeds petalTongue as a library
2. Direct function calls (no network)
3. Shared memory for topology data

**Note**: This is a "chimera" pattern (future evolution)

---

### **Pattern 3: Network to Toadstool** (High-End Rendering)

**Use Case**: Offload heavy rendering to GPU-accelerated Toadstool

```
petalTongue → Toadstool → GPU rendering → Return result
```

**Advantages**:
- ✅ High-end graphics (ray tracing, complex shaders)
- ✅ Offload compute from UI primal
- ✅ Leverage Toadstool's multi-runtime (GPU/CPU/WASM)

**Implementation**:
1. petalTongue discovers Toadstool via SPDP
2. Sends rendering workload to Toadstool
3. Receives rendered frames back
4. Displays to user

**Future**: This enables VR, 3D visualizations, advanced animations

---

## 🚀 **Phase 1: Standalone Integration** (This Week)

### **Goal**: Get petalTongue running as a standalone primal, discovered by biomeOS

### **Tasks**

#### **biomeOS Side**

| Task | Owner | Priority | Status |
|------|-------|----------|--------|
| Add petalTongue to `PrimalRegistry` | biomeOS | 🔴 HIGH | ⏳ Pending |
| Discover petalTongue via SPDP | biomeOS | 🔴 HIGH | ⏳ Pending |
| Provide topology data API | biomeOS | 🔴 HIGH | ⏳ Pending |
| Test with real petalTongue binary | biomeOS | 🔴 HIGH | ⏳ Pending |

#### **petalTongue Side**

| Task | Owner | Priority | Status |
|------|-------|----------|--------|
| Verify Unix socket evolution | petalTongue | 🟡 MEDIUM | ⏳ Pending |
| Add JSON-RPC server | petalTongue | 🟡 MEDIUM | ⏳ Pending |
| Test with biomeOS topology data | petalTongue | 🟡 MEDIUM | ⏳ Pending |

---

### **Step 1: Run petalTongue Standalone**

```bash
# From biomeOS directory
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS

# Run petalTongue (GUI mode)
./nucleusBins/petal-tongue

# OR run headless (terminal mode)
./nucleusBins/petal-tongue-headless --mode terminal
```

**Expected Behavior**:
- ✅ petalTongue starts
- ✅ Attempts mDNS discovery
- ✅ Falls back to environment hints
- ✅ Shows awakening experience
- ✅ Displays UI (or terminal output)

---

### **Step 2: biomeOS Provides Topology Data**

**Option A: HTTP Endpoint** (Current)

```rust
// In biomeOS HTTP server
#[get("/api/v1/topology")]
async fn get_topology() -> Json<TopologyData> {
    let primals = discover_all_primals().await?;
    Json(TopologyData {
        primals,
        connections,
        health_status,
    })
}
```

**Option B: Unix Socket JSON-RPC** (Future)

```rust
// In biomeOS Unix socket server
async fn handle_get_topology() -> Result<TopologyData> {
    let primals = discover_all_primals().await?;
    Ok(TopologyData {
        primals,
        connections,
        health_status,
    })
}
```

---

### **Step 3: petalTongue Discovers biomeOS**

**Current Method**: HTTP probing

```rust
// In petalTongue discovery
let hints = std::env::var("PETALTONGUE_DISCOVERY_HINTS")
    .unwrap_or_else(|_| "http://localhost:3000".to_string());

for hint in hints.split(',') {
    match try_connect_http(hint).await {
        Ok(provider) => {
            // Found biomeOS!
            providers.push(provider);
        }
        Err(e) => {
            tracing::error!("Failed to connect: {}", e);
        }
    }
}
```

**Future Method**: SPDP

```rust
// In petalTongue discovery
let spdp = SecurePrimalDiscovery::new(songbird, beardog);
let biomeos = spdp.discover_by_capability("topology.provider").await?;
```

---

### **Step 4: Test Integration**

```bash
# Terminal 1: Start biomeOS (with topology API)
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
cargo run --bin biomeos -- serve --port 3000

# Terminal 2: Start petalTongue (with discovery hint)
export PETALTONGUE_DISCOVERY_HINTS="http://localhost:3000"
./nucleusBins/petal-tongue

# Expected: petalTongue discovers biomeOS, fetches topology, displays UI
```

---

## 🔮 **Phase 2: Unix Socket Evolution** (Week 2-3)

### **Goal**: Evolve petalTongue to Unix socket JSON-RPC (port-free architecture)

### **Tasks**

| Task | Owner | Priority | Status |
|------|-------|----------|--------|
| Add Unix socket JSON-RPC server | petalTongue | 🔴 HIGH | ⏳ Pending |
| Implement `get_capabilities` API | petalTongue | 🔴 HIGH | ⏳ Pending |
| Implement `get_topology` API | petalTongue | 🔴 HIGH | ⏳ Pending |
| Implement `render_graph` API | petalTongue | 🟡 MEDIUM | ⏳ Pending |
| HTTP deprecation (port-free mode) | petalTongue | 🟡 MEDIUM | ⏳ Pending |
| Integration tests with biomeOS | Both Teams | 🔴 HIGH | ⏳ Pending |

**Deliverables**:
- ✅ `/tmp/petaltongue-${NODE_ID}.sock` (Unix socket)
- ✅ JSON-RPC 2.0 server
- ✅ 3+ APIs functional
- ✅ Port-free architecture

---

## 🌟 **Phase 3: Advanced Features** (Week 4+)

### **1. Network to Toadstool for GPU Rendering**

**Use Case**: High-end 3D visualizations, VR, ray tracing

```rust
// In petalTongue
let toadstool = spdp.discover_by_capability("compute.gpu").await?;
let workload = RenderWorkload {
    scene: topology_to_3d_scene(),
    quality: RenderQuality::High,
    output_format: OutputFormat::RGBA8,
};
let rendered = toadstool.submit_workload(workload).await?;
display_to_user(rendered);
```

**Benefits**:
- ✅ Offload heavy rendering
- ✅ Leverage GPU acceleration
- ✅ Enable advanced visualizations

---

### **2. Real-Time Updates via WebSocket**

**Use Case**: Live topology changes, flow animations

```rust
// In biomeOS
let ws_server = WebSocketServer::new();
ws_server.on_topology_change(|change| {
    broadcast_to_all_clients(change);
});

// In petalTongue
let ws_client = WebSocketClient::connect("ws://biomeos/topology").await?;
ws_client.on_message(|change| {
    update_topology(change);
    re_render();
});
```

---

### **3. Multi-Modal Accessibility**

**Use Case**: Blind users, audio-only environments

```rust
// In petalTongue
let modalities = vec!["audio", "text", "visual"];
for modality in modalities {
    if is_available(modality) {
        render_in_modality(modality, topology);
    }
}
```

**Example**:
- Visual: Graph with nodes and edges
- Audio: Sonification of primal health (tones, chimes)
- Text: JSON output for screen readers

---

## 📊 **Success Metrics**

### **Phase 1 Success Criteria**
- ✅ petalTongue runs as standalone primal
- ✅ biomeOS provides topology data via HTTP
- ✅ petalTongue discovers biomeOS via environment hints
- ✅ petalTongue displays topology in GUI
- ✅ E2E integration test passing

### **Phase 2 Success Criteria**
- ✅ petalTongue on Unix socket JSON-RPC
- ✅ biomeOS discovers petalTongue via SPDP
- ✅ Port-free architecture working
- ✅ Integration tests passing

### **Phase 3 Success Criteria**
- ✅ petalTongue networks to Toadstool for GPU rendering
- ✅ Real-time updates via WebSocket
- ✅ Multi-modal rendering working
- ✅ Advanced visualizations (3D, VR)

---

## 🤝 **Team Coordination**

### **biomeOS Team**
**Focus**: Provide topology data, integrate with SPDP  
**Current**: Need to implement topology API  
**Blockers**: None

### **petalTongue Team**
**Focus**: Unix socket evolution, SPDP integration  
**Current**: Production ready (HTTP mode)  
**Blockers**: 🔴 Unix socket handoff needed

---

## 🎯 **Immediate Next Steps**

### **This Week (biomeOS)**
1. 🔴 Add topology data API to biomeOS
2. 🔴 Test with petalTongue binary
3. 🔴 Document topology data format

### **Next 2 Weeks (petalTongue)**
1. 🔴 Unix socket JSON-RPC server
2. 🔴 SPDP integration
3. 🔴 Integration tests with biomeOS

### **Week 4 (Both Teams)**
1. 🟡 E2E testing (biomeOS + petalTongue)
2. 🟡 Performance benchmarks
3. 🟡 Advanced features (Toadstool GPU, WebSocket)

---

## 📚 **References**

### **petalTongue**
- Location: `/home/eastgate/Development/ecoPrimals/phase2/petalTongue`
- Binaries: `nucleusBins/petal-tongue`, `nucleusBins/petal-tongue-headless`
- Spec: `specs/PETALTONGUE_UI_AND_VISUALIZATION_SPECIFICATION.md`
- Status: v0.4.0 Production Ready (A+ 9.5/10)

### **biomeOS**
- Location: `/home/eastgate/Development/ecoPrimals/phase2/biomeOS`
- SPDP: `specs/SECURE_PRIMAL_DISCOVERY_PROTOCOL.md`
- Neural API: `specs/NEURAL_API_IMPLEMENTATION_PHASES.md`
- Roadmap: `ROADMAP.md`

### **Integration**
- This document: `docs/PETALTONGUE_BIOMEOS_INTEGRATION_PLAN.md`
- Toadstool handoff: `docs/jan4-session/TOADSTOOL_UNIX_SOCKET_HANDOFF_JAN9.md`

---

## 🎊 **Bottom Line**

**petalTongue is production ready!**

✅ **v0.4.0**: A+ grade, 536+ tests, zero mocks, 100% safe Rust  
✅ **Multi-modal**: Visual + audio + text  
✅ **6 display backends**: All working  
✅ **TRUE PRIMAL**: Zero hardcoding, capability-based discovery  
✅ **In-code rendering**: Can network to Toadstool for GPU power  

**Next Steps**:
1. Add topology API to biomeOS
2. Test with petalTongue binary
3. Evolve to Unix socket (Phase 2)
4. Network to Toadstool for GPU rendering (Phase 3)

**Timeline**: 4 weeks to full integration

---

**The face of ecoPrimals is ready to shine!** 🌸✨

🌱 **biomeOS + petalTongue: Complete Ecosystem UI** 🎉

