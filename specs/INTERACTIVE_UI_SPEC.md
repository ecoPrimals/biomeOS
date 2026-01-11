# 🎨 Interactive UI Specification - Network Effect Feature

**Version**: 1.0  
**Date**: January 11, 2026  
**Type**: Inter-Primal Network Effect  
**Status**: Ready for Implementation

---

## 🎯 **Overview**

**What This Is:**
A Discord-like interactive UI for runtime device and primal management that emerges from the **network effect** of multiple primals working together.

**What This Is NOT:**
- ❌ Not a single primal's feature
- ❌ Not a niche (deployment pattern)
- ❌ Not hardcoded coordination

**What This IS:**
- ✅ Network effect of 7 primals cooperating
- ✅ Emergent capability from TRUE PRIMAL architecture
- ✅ Dynamic, capability-based collaboration
- ✅ Example of inter-primal behavior patterns

---

## 🤝 **The Network Effect**

### **Participating Primals**

```
       Network Effect: Interactive UI
              /           |           \
             /            |            \
    petalTongue      biomeOS       Songbird
    (rendering)   (orchestration)  (discovery)
         \              |              /
          \             |             /
       BearDog     NestGate     ToadStool    Squirrel
      (security)  (storage)    (resources)    (AI)
```

**Key Insight**: No single primal "owns" this feature. It emerges from their cooperation!

---

## 📋 **Functional Requirements**

### **FR-1: Device Visualization**

**User Story**: As a user, I want to see all available devices in my ecosystem so I can understand what hardware is available.

**Participating Primals:**
1. **Songbird**: Discovers and registers devices
2. **petalTongue**: Renders device tree view
3. **biomeOS**: Coordinates the data flow

**JSON-RPC Flow:**
```json
# 1. biomeOS → Songbird
{
  "jsonrpc": "2.0",
  "method": "songbird.discover_devices",
  "params": {},
  "id": 1
}

# 2. Songbird → biomeOS (response)
{
  "jsonrpc": "2.0",
  "result": {
    "devices": [
      {
        "id": "gpu-rtx-4090",
        "type": "gpu",
        "name": "NVIDIA RTX 4090",
        "capabilities": ["cuda", "opencl", "vulkan"],
        "resources": {"vram": "24GB", "cores": 16384}
      }
    ]
  },
  "id": 1
}

# 3. biomeOS → petalTongue
{
  "jsonrpc": "2.0",
  "method": "ui.render_device_list",
  "params": {
    "devices": [/* ... */]
  },
  "id": 2
}
```

**Acceptance Criteria:**
- [ ] Songbird can discover and return device list
- [ ] petalTongue can render device tree with icons
- [ ] biomeOS coordinates the data flow
- [ ] UI updates in <100ms

---

### **FR-2: Primal Status Display**

**User Story**: As a user, I want to see the status of all running primals so I can monitor system health.

**Participating Primals:**
1. **Songbird**: Maintains primal registry and health
2. **petalTongue**: Renders primal status panels
3. **biomeOS**: Aggregates and formats data

**JSON-RPC Flow:**
```json
# biomeOS → Songbird
{
  "jsonrpc": "2.0",
  "method": "songbird.get_all_primals",
  "params": {},
  "id": 3
}

# Songbird → biomeOS (response)
{
  "jsonrpc": "2.0",
  "result": {
    "primals": [
      {
        "id": "toadstool-1",
        "name": "toadstool",
        "status": "running",
        "capabilities": ["compute", "gpu", "workload"],
        "health": {
          "status": "healthy",
          "uptime": 3600,
          "cpu_usage": 45.2,
          "memory_usage": 2048
        }
      }
    ]
  },
  "id": 3
}

# biomeOS → petalTongue
{
  "jsonrpc": "2.0",
  "method": "ui.render_primal_list",
  "params": {
    "primals": [/* ... */]
  },
  "id": 4
}
```

**Acceptance Criteria:**
- [ ] Songbird provides primal list with health data
- [ ] petalTongue renders status with color coding
- [ ] biomeOS polls for updates every 5 seconds
- [ ] UI shows real-time metrics

---

### **FR-3: Device Assignment (Drag & Drop)**

**User Story**: As a user, I want to drag a device to a primal to assign it for use.

**Participating Primals:**
1. **petalTongue**: Captures drag-and-drop event
2. **biomeOS**: Orchestrates validation and assignment
3. **BearDog**: Authorizes the action
4. **Songbird**: Validates device/primal compatibility
5. **ToadStool**: Checks resource availability
6. **NestGate**: Persists the assignment

**JSON-RPC Flow:**
```json
# 1. petalTongue → biomeOS (user drags GPU to ToadStool)
{
  "jsonrpc": "2.0",
  "method": "biomeos.ui.assign_device",
  "params": {
    "device_id": "gpu-rtx-4090",
    "primal_id": "toadstool-1",
    "user_id": "alice"
  },
  "id": 5
}

# 2. biomeOS → BearDog (authorize)
{
  "jsonrpc": "2.0",
  "method": "beardog.authorize",
  "params": {
    "user_id": "alice",
    "action": "assign_device",
    "resource": "gpu-rtx-4090"
  },
  "id": 6
}

# 3. biomeOS → Songbird (validate)
{
  "jsonrpc": "2.0",
  "method": "songbird.validate_assignment",
  "params": {
    "device_id": "gpu-rtx-4090",
    "primal_id": "toadstool-1"
  },
  "id": 7
}

# 4. biomeOS → ToadStool (check resources)
{
  "jsonrpc": "2.0",
  "method": "toadstool.check_resources",
  "params": {
    "device_id": "gpu-rtx-4090"
  },
  "id": 8
}

# 5. biomeOS → Songbird (register assignment)
{
  "jsonrpc": "2.0",
  "method": "songbird.register_assignment",
  "params": {
    "device_id": "gpu-rtx-4090",
    "primal_id": "toadstool-1",
    "timestamp": "2026-01-11T10:34:00Z"
  },
  "id": 9
}

# 6. biomeOS → NestGate (persist)
{
  "jsonrpc": "2.0",
  "method": "storage.store",
  "params": {
    "key": "assignment:gpu-rtx-4090",
    "data": {
      "primal_id": "toadstool-1",
      "assigned_at": "2026-01-11T10:34:00Z"
    },
    "family_id": "nat0"
  },
  "id": 10
}

# 7. biomeOS → petalTongue (update UI)
{
  "jsonrpc": "2.0",
  "method": "ui.update_assignment",
  "params": {
    "device_id": "gpu-rtx-4090",
    "primal_id": "toadstool-1",
    "status": "assigned"
  },
  "id": 11
}
```

**Acceptance Criteria:**
- [ ] User can drag device to primal
- [ ] Authorization check passes/fails appropriately
- [ ] Validation prevents incompatible assignments
- [ ] Assignment persists across restarts
- [ ] UI updates immediately on success
- [ ] Clear error message on failure

---

### **FR-4: AI-Powered Suggestions**

**User Story**: As a user, I want AI suggestions for optimal device assignments so I can make informed decisions.

**Participating Primals:**
1. **Squirrel**: Analyzes topology and suggests optimizations
2. **petalTongue**: Renders suggestions in UI
3. **biomeOS**: Coordinates the analysis

**JSON-RPC Flow:**
```json
# biomeOS → Squirrel
{
  "jsonrpc": "2.0",
  "method": "squirrel.suggest_assignment",
  "params": {
    "device": {
      "id": "gpu-rtx-4090",
      "type": "gpu",
      "capabilities": ["cuda", "opencl"]
    },
    "available_primals": [
      {"id": "toadstool-1", "capabilities": ["compute", "gpu"]},
      {"id": "nestgate-1", "capabilities": ["storage"]}
    ]
  },
  "id": 12
}

# Squirrel → biomeOS (response)
{
  "jsonrpc": "2.0",
  "result": {
    "suggestions": [
      {
        "primal_id": "toadstool-1",
        "confidence": 0.95,
        "reason": "High GPU compute capability match",
        "benefits": ["ML workloads", "Parallel processing"],
        "estimated_utilization": 85
      }
    ]
  },
  "id": 12
}

# biomeOS → petalTongue
{
  "jsonrpc": "2.0",
  "method": "ui.show_suggestions",
  "params": {
    "suggestions": [/* ... */]
  },
  "id": 13
}
```

**Acceptance Criteria:**
- [ ] Squirrel analyzes device capabilities
- [ ] Suggestions include confidence scores
- [ ] UI shows suggestions with reasoning
- [ ] User can accept/dismiss suggestions
- [ ] Suggestions update as topology changes

---

### **FR-5: Real-Time Topology Visualization**

**User Story**: As a user, I want to see a live graph of how devices are connected to primals so I can understand the system topology.

**Participating Primals:**
1. **Songbird**: Maintains topology graph
2. **petalTongue**: Renders interactive graph
3. **biomeOS**: Streams topology updates

**JSON-RPC Flow:**
```json
# biomeOS → Songbird
{
  "jsonrpc": "2.0",
  "method": "songbird.get_topology",
  "params": {},
  "id": 14
}

# Songbird → biomeOS (response)
{
  "jsonrpc": "2.0",
  "result": {
    "nodes": [
      {"id": "beardog-1", "type": "primal", "name": "BearDog"},
      {"id": "gpu-rtx", "type": "device", "name": "GPU RTX 4090"}
    ],
    "edges": [
      {"from": "beardog-1", "to": "gpu-rtx", "type": "assigned"}
    ]
  },
  "id": 14
}

# biomeOS → petalTongue
{
  "jsonrpc": "2.0",
  "method": "ui.render",
  "params": {
    "graph_data": {
      "nodes": [/* ... */],
      "edges": [/* ... */]
    }
  },
  "id": 15
}
```

**Acceptance Criteria:**
- [ ] petalTongue renders force-directed graph
- [ ] Graph updates in real-time (<100ms)
- [ ] Nodes show status colors (green/yellow/red)
- [ ] Edges show relationship types
- [ ] User can click nodes for details

---

### **FR-6: Chat-like Event Log**

**User Story**: As a user, I want to see a Discord-like log of system events so I can track what's happening.

**Participating Primals:**
1. **All primals**: Emit events via Songbird
2. **Songbird**: Aggregates and broadcasts events
3. **petalTongue**: Renders chat-like log panel
4. **biomeOS**: Subscribes to event stream

**JSON-RPC Flow:**
```json
# Primal → Songbird (event emission)
{
  "jsonrpc": "2.0",
  "method": "songbird.emit_event",
  "params": {
    "source": "toadstool-1",
    "type": "device_assigned",
    "message": "GPU assigned (RTX 4090)",
    "level": "info",
    "timestamp": "2026-01-11T10:34:00Z"
  },
  "id": 16
}

# Songbird → biomeOS (event broadcast via WebSocket)
{
  "jsonrpc": "2.0",
  "method": "events.stream",
  "params": {
    "event": {
      "source": "toadstool-1",
      "message": "GPU assigned (RTX 4090)",
      "level": "info",
      "timestamp": "2026-01-11T10:34:00Z"
    }
  }
}

# biomeOS → petalTongue
{
  "jsonrpc": "2.0",
  "method": "ui.add_log_entry",
  "params": {
    "entry": {
      "source": "ToadStool",
      "message": "GPU assigned (RTX 4090) ✓",
      "timestamp": "10:34",
      "icon": "🍄"
    }
  },
  "id": 17
}
```

**Acceptance Criteria:**
- [ ] All primal events appear in log
- [ ] Log entries show timestamp, source, message
- [ ] Different log levels (info, warn, error)
- [ ] User can filter by primal or level
- [ ] Log auto-scrolls to latest entry
- [ ] Log persists to NestGate (last 1000 entries)

---

## 🏗️ **Technical Architecture**

### **New Components to Build**

#### **1. biomeos-ui Crate**

```rust
// crates/biomeos-ui/src/lib.rs

pub struct InteractiveUIOrchestrator {
    // Primal clients
    petaltongue: PetalTongueClient,
    songbird: SongbirdClient,
    beardog: BearDogClient,
    nestgate: NestGateClient,
    toadstool: ToadStoolClient,
    squirrel: SquirrelClient,
    
    // State
    state: Arc<RwLock<UIState>>,
    event_tx: mpsc::Sender<UIEvent>,
}

pub struct UIState {
    pub devices: Vec<Device>,
    pub primals: Vec<PrimalInfo>,
    pub assignments: HashMap<DeviceId, PrimalId>,
    pub topology: Graph,
    pub logs: VecDeque<LogEntry>,
}

pub enum UIEvent {
    DeviceDiscovered(Device),
    PrimalRegistered(PrimalInfo),
    AssignmentCreated { device: DeviceId, primal: PrimalId },
    AssignmentRemoved { device: DeviceId },
    LogEntry(LogEntry),
    TopologyChanged(Graph),
}

impl InteractiveUIOrchestrator {
    pub async fn new() -> Result<Self>;
    pub async fn start(&mut self) -> Result<()>;
    pub async fn handle_user_action(&self, action: UserAction) -> Result<()>;
    pub async fn sync_state(&self) -> Result<()>;
}
```

**Files to Create:**
- `crates/biomeos-ui/Cargo.toml`
- `crates/biomeos-ui/src/lib.rs`
- `crates/biomeos-ui/src/orchestrator.rs`
- `crates/biomeos-ui/src/state.rs`
- `crates/biomeos-ui/src/events.rs`
- `crates/biomeos-ui/src/actions.rs`

---

#### **2. petalTongue Extensions**

**New JSON-RPC Methods Needed:**

```rust
// Methods petalTongue team needs to implement:

"ui.render_device_list" -> Renders device tree
"ui.render_primal_list" -> Renders primal status panels
"ui.update_assignment" -> Updates device→primal connection
"ui.add_log_entry" -> Adds entry to chat log
"ui.show_suggestions" -> Shows AI suggestions
"ui.handle_drag_drop" -> Captures drag-and-drop events

// WebSocket support:
"ui.subscribe_updates" -> Opens WebSocket for real-time updates
"ui.push_event" -> Server→client event push
```

**Handoff to petalTongue Team:**
- See: `specs/PETALTONGUE_UI_METHODS.md` (to be created)
- Interactive GUI components needed
- Drag-and-drop support
- WebSocket client implementation

---

#### **3. Songbird Extensions**

**New JSON-RPC Methods Needed:**

```rust
// Device registry (new!)
"songbird.register_device" -> Register hardware device
"songbird.discover_devices" -> List all devices
"songbird.get_device_info" -> Get device details
"songbird.validate_assignment" -> Check compatibility

// Assignment tracking (new!)
"songbird.register_assignment" -> Record device→primal
"songbird.get_assignments" -> List all assignments
"songbird.remove_assignment" -> Unassign device

// Event streaming (new!)
"songbird.emit_event" -> Emit system event
"songbird.subscribe_events" -> Subscribe to event stream
```

**Handoff to Songbird Team:**
- See: `specs/SONGBIRD_DEVICE_REGISTRY.md` (to be created)
- Device registry data structure
- Assignment validation logic
- Event aggregation system

---

#### **4. CLI Command**

```bash
# New CLI command:
biomeos ui

# Options:
--mode interactive    # Full interactive UI (default)
--mode dashboard      # Read-only dashboard
--mode headless       # No GUI, just orchestration

# Examples:
biomeos ui                          # Launch interactive UI
biomeos ui --mode dashboard         # Launch dashboard
biomeos ui --family-id mynetwork    # Connect to specific family
```

**CLI Implementation:**
- `crates/biomeos-cli/src/commands/ui.rs` (new file)
- Launches InteractiveUIOrchestrator
- Connects to petalTongue
- Handles graceful shutdown

---

## 📊 **API Contracts**

### **biomeOS ↔ petalTongue**

| Method | Direction | Purpose |
|--------|-----------|---------|
| `ui.render_device_list` | biomeOS → pT | Display devices |
| `ui.render_primal_list` | biomeOS → pT | Display primals |
| `ui.update_assignment` | biomeOS → pT | Update topology |
| `ui.add_log_entry` | biomeOS → pT | Add log message |
| `ui.show_suggestions` | biomeOS → pT | Show AI hints |
| `biomeos.ui.assign_device` | pT → biomeOS | User assigns device |
| `biomeos.ui.start_primal` | pT → biomeOS | User starts primal |
| `biomeos.ui.stop_primal` | pT → biomeOS | User stops primal |

---

### **biomeOS ↔ Songbird**

| Method | Direction | Purpose |
|--------|-----------|---------|
| `songbird.discover_devices` | biomeOS → SB | Get device list |
| `songbird.get_all_primals` | biomeOS → SB | Get primal list |
| `songbird.validate_assignment` | biomeOS → SB | Check compatibility |
| `songbird.register_assignment` | biomeOS → SB | Record assignment |
| `songbird.get_topology` | biomeOS → SB | Get graph data |
| `songbird.emit_event` | Any → SB | Emit event |
| `songbird.subscribe_events` | biomeOS → SB | Subscribe to events |

---

### **biomeOS ↔ Other Primals**

| Primal | Methods | Purpose |
|--------|---------|---------|
| **BearDog** | `beardog.authorize` | Check user permissions |
| **NestGate** | `storage.store`, `storage.retrieve` | Persist UI state |
| **ToadStool** | `toadstool.check_resources` | Validate resources |
| **Squirrel** | `squirrel.suggest_assignment` | AI recommendations |

---

## 🚀 **Implementation Phases**

### **Phase 1: Foundation** (Week 1-2)

**Goal**: Basic UI with static data

**Tasks:**
- [ ] Create `biomeos-ui` crate structure
- [ ] Implement `UIState` and `UIEvent` types
- [ ] Create `InteractiveUIOrchestrator` skeleton
- [ ] Add `biomeos ui` CLI command
- [ ] Handoff petalTongue method specs
- [ ] Handoff Songbird device registry spec

**Deliverable**: Can launch UI and see hardcoded devices/primals

---

### **Phase 2: Discovery** (Week 3-4)

**Goal**: UI shows real live data

**Tasks:**
- [ ] Implement device discovery via Songbird
- [ ] Implement primal discovery via Songbird
- [ ] Wire up data flow to petalTongue
- [ ] Add periodic refresh (5 sec intervals)
- [ ] Implement topology graph rendering

**Deliverable**: UI shows live devices and primals

---

### **Phase 3: Interaction** (Week 5-7)

**Goal**: User can assign devices

**Tasks:**
- [ ] Implement drag-and-drop capture
- [ ] Implement assignment orchestration
- [ ] Add BearDog authorization
- [ ] Add Songbird validation
- [ ] Add NestGate persistence
- [ ] Wire up UI feedback

**Deliverable**: Working device assignment with validation

---

### **Phase 4: Real-Time** (Week 8-9)

**Goal**: Live updates without refresh

**Tasks:**
- [ ] Implement WebSocket in petalTongue
- [ ] Implement event streaming in Songbird
- [ ] Wire up event subscription in biomeOS
- [ ] Add real-time topology updates
- [ ] Add chat-like log panel

**Deliverable**: UI updates in real-time (<100ms)

---

### **Phase 5: Intelligence** (Week 10-11)

**Goal**: AI-powered suggestions

**Tasks:**
- [ ] Integrate Squirrel client
- [ ] Implement suggestion request flow
- [ ] Add suggestion UI rendering
- [ ] Add accept/dismiss actions
- [ ] Add optimization hints

**Deliverable**: AI assists with device assignments

---

### **Phase 6: Polish** (Week 12)

**Goal**: Production-ready UI

**Tasks:**
- [ ] Add comprehensive error handling
- [ ] Add loading states
- [ ] Add confirmation dialogs
- [ ] Add keyboard shortcuts
- [ ] Add accessibility features
- [ ] Write user documentation

**Deliverable**: Production-ready interactive UI

---

## ✅ **Acceptance Criteria**

### **Functional**

- [ ] User can see all available devices
- [ ] User can see all running primals
- [ ] User can drag device to primal to assign
- [ ] Assignment validates capabilities
- [ ] Assignment persists across restarts
- [ ] UI shows real-time topology
- [ ] UI shows chat-like event log
- [ ] AI suggestions appear when relevant
- [ ] User can accept AI suggestions

### **Non-Functional**

- [ ] UI updates in <100ms
- [ ] No hardcoded primal dependencies
- [ ] All communication via JSON-RPC
- [ ] Gracefully handles primal failures
- [ ] Works with 1-100 devices
- [ ] Works with 1-50 primals
- [ ] Memory usage <500MB
- [ ] CPU usage <5% idle

### **Security**

- [ ] All actions authorized via BearDog
- [ ] No sensitive data in logs
- [ ] Encrypted configuration storage
- [ ] Session timeout after 1 hour
- [ ] Audit trail in NestGate

---

## 📝 **Network Effect Analysis**

### **Without This Feature:**

```
petalTongue: Can render graphs ✓
Songbird: Can discover services ✓
BearDog: Can authorize ✓
NestGate: Can store data ✓
ToadStool: Can manage compute ✓
Squirrel: Can suggest ✓

Result: 6 independent capabilities
```

### **With This Feature:**

```
Network Effect = petalTongue × Songbird × BearDog × NestGate × ToadStool × Squirrel

Result: Interactive runtime management UI!

Value = n² (Metcalfe's Law)
     = 6² = 36 potential interactions
     
This feature didn't exist in any single primal,
it EMERGED from their cooperation!
```

---

## 🎯 **Success Metrics**

| Metric | Target | How Measured |
|--------|--------|--------------|
| **Network Effect** | 7 primals cooperating | Count unique primal interactions |
| **Zero Hardcoding** | 0 hardcoded deps | Code audit |
| **Performance** | <100ms UI updates | Latency monitoring |
| **User Satisfaction** | >90% positive | User surveys |
| **Adoption** | >80% users | Usage analytics |

---

## 📚 **Related Specs**

**To Be Created:**
1. `specs/PETALTONGUE_UI_METHODS.md` - petalTongue API additions
2. `specs/SONGBIRD_DEVICE_REGISTRY.md` - Songbird device tracking
3. `specs/BIOMEOS_UI_ORCHESTRATOR.md` - biomeOS orchestration layer

---

## 🎊 **Conclusion**

This is a **perfect example** of TRUE PRIMAL network effects:

1. **No single owner**: No primal "owns" this feature
2. **Emergent capability**: Didn't exist, now it does
3. **Network effect**: Value = n² interactions
4. **Zero hardcoding**: All dynamic, capability-based
5. **Inter-primal behavior**: Primals cooperate, not compete

**This is biomeOS at its best!** 🚀

---

**Status**: ✅ Spec complete, ready to implement  
**Next**: Create `biomeos-ui` crate and start Phase 1!

