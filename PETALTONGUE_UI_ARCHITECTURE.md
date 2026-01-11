# 🎨 petalTongue UI Architecture - Discord-like Interface

**Date**: January 11, 2026  
**Context**: User wants Discord-like UI for device assignment and runtime management  
**Question**: What does petalTongue provide vs what does biomeOS orchestrate?

---

## 🎯 **The Vision: Discord-like UI for biomeOS**

**User Experience:**
- Chat-like interface for primal communication
- Device/node panel showing available hardware
- Drag-and-drop device assignment to primals
- Real-time topology visualization
- Status dashboards per primal
- Configuration panels

**Key Question**: Who does what?

---

## 🏗️ **Architecture: TRUE PRIMAL Division of Labor**

### **petalTongue's Role** (UI Framework & Rendering)

**What petalTongue Provides:**

1. **UI Framework** (Rust + egui)
   ```rust
   // petalTongue provides:
   - Interactive GUI framework
   - Layout system (panels, tabs, modals)
   - Input handling (mouse, keyboard, touch)
   - Rendering engine (native + web)
   - Component library (buttons, lists, forms)
   ```

2. **Core UI Components**
   - Chat/log panel (Discord-like messages)
   - Device tree view (available hardware)
   - Topology graph (live primal connections)
   - Configuration forms (dynamic based on primal capabilities)
   - Status indicators (health, metrics, alerts)

3. **Rendering Capabilities**
   - Multi-window support
   - Real-time updates (via websocket or IPC)
   - Drag-and-drop interactions
   - Context menus
   - Keyboard shortcuts

4. **JSON-RPC Methods** (Already Implemented)
   ```rust
   // petalTongue's current API:
   - ui.render(graph_data) -> Rendered visualization
   - ui.display_status(primal_status) -> Status display
   - health_check() -> Health status
   - announce_capabilities() -> UI capabilities
   - get_capabilities() -> Available UI modes
   ```

**What petalTongue DOESN'T Do:**
- ❌ Business logic (device assignment rules)
- ❌ State management (which device goes where)
- ❌ Orchestration (starting/stopping primals)
- ❌ Discovery (finding devices/primals)
- ❌ Persistence (saving configurations)

---

### **biomeOS's Role** (Orchestration & Runtime Logic)

**What biomeOS Provides:**

1. **Runtime Orchestration**
   ```rust
   // biomeOS orchestrates:
   - Device assignment logic
   - Primal lifecycle management
   - Configuration validation
   - State synchronization
   - Event coordination
   ```

2. **UI Integration Layer**
   ```rust
   // New: biomeos-ui crate (needs evolution!)
   
   pub struct BiomeOSUIOrchestrator {
       petaltongue_client: PetalTongueClient,
       songbird_client: SongbirdClient,
       state: Arc<RwLock<UIState>>,
   }
   
   impl BiomeOSUIOrchestrator {
       // Coordinate UI state with primal state
       pub async fn sync_ui_state(&self) -> Result<()>;
       
       // Handle user actions from UI
       pub async fn handle_device_assignment(
           &self,
           device_id: &str,
           primal_name: &str,
       ) -> Result<()>;
       
       // Push updates to UI
       pub async fn update_topology(&self) -> Result<()>;
   }
   ```

3. **Business Logic**
   - Device assignment rules (capabilities, resources)
   - Conflict resolution (device already assigned)
   - Validation (primal supports this device?)
   - Permissions (user authorized?)

4. **State Management**
   ```rust
   pub struct UIState {
       pub available_devices: Vec<Device>,
       pub active_primals: Vec<PrimalInfo>,
       pub assignments: HashMap<DeviceId, PrimalId>,
       pub topology: Graph,
   }
   ```

---

### **Other Primals' Roles** (Data & Services)

#### **1. Songbird** (Discovery & Registry)

**Provides:**
```rust
// What Songbird supplies:
- discover_devices() -> Vec<Device>
- discover_primals() -> Vec<Primal>
- get_primal_capabilities(primal_id) -> Capabilities
- register_assignment(device_id, primal_id) -> Result<()>
- get_topology() -> Graph
```

**UI Integration:**
- biomeOS queries Songbird for available devices/primals
- biomeOS pushes to petalTongue for rendering
- User interacts in petalTongue UI
- biomeOS validates and commits via Songbird

---

#### **2. BearDog** (Security & Access Control)

**Provides:**
```rust
// What BearDog supplies:
- authenticate_user(credentials) -> Token
- authorize_action(user, action) -> bool
- encrypt_config(config) -> EncryptedConfig
- verify_device(device_id) -> DeviceIdentity
```

**UI Integration:**
- petalTongue shows login panel
- biomeOS authenticates via BearDog
- User sees only authorized devices/primals
- Sensitive data encrypted before display

---

#### **3. NestGate** (Persistence & Configuration)

**Provides:**
```rust
// What NestGate supplies:
- store_assignment(assignment) -> Result<()>
- retrieve_assignments() -> Vec<Assignment>
- store_ui_preferences(prefs) -> Result<()>
- retrieve_topology_snapshot() -> Topology
```

**UI Integration:**
- User makes device assignment in petalTongue
- biomeOS validates and stores via NestGate
- On reload, biomeOS fetches from NestGate
- petalTongue renders restored state

---

#### **4. ToadStool** (Compute & Resources)

**Provides:**
```rust
// What ToadStool supplies:
- get_device_resources(device_id) -> Resources
- get_workload_status(primal_id) -> WorkloadInfo
- assign_workload(workload, device) -> Result<()>
```

**UI Integration:**
- petalTongue shows resource meters (CPU, RAM, GPU)
- biomeOS queries ToadStool for live metrics
- User assigns compute-heavy primal to device
- ToadStool validates resource availability

---

#### **5. Squirrel** (AI Assistance)

**Provides:**
```rust
// What Squirrel supplies:
- suggest_assignment(device, available_primals) -> Vec<Suggestion>
- analyze_topology() -> OptimizationHints
- predict_resource_usage(assignment) -> Prediction
```

**UI Integration:**
- petalTongue shows AI suggestion panel
- User hovers over device, sees Squirrel's recommendations
- biomeOS asks Squirrel for optimal assignments
- petalTongue renders suggestions with confidence scores

---

## 🎨 **Discord-like UI: Concrete Example**

### **UI Layout** (petalTongue Provides)

```
╔═══════════════════════════════════════════════════════════════════════╗
║ biomeOS Runtime Manager                                    [_][□][X] ║
╠═══════════════╦═══════════════════════════════════════════════════════╣
║ DEVICES       ║ TOPOLOGY VIEW                                         ║
║               ║                                                       ║
║ 🖥️  Computer 1 ║     [BearDog] ──┬── [Device: TPM]                  ║
║  • TPM Chip   ║                 └── [Device: Yubikey]               ║
║  • Yubikey    ║                                                       ║
║               ║     [Songbird] ─── [Device: eth0]                    ║
║ 🖥️  Computer 2 ║                                                       ║
║  • GPU (RTX)  ║     [ToadStool] ── [Device: GPU RTX]                 ║
║  • NVMe SSD   ║                                                       ║
║               ║     [NestGate] ─── [Device: NVMe SSD]                ║
║ 🌐 Network    ║                                                       ║
║  • eth0       ║                                                       ║
║  • wlan0      ║                                                       ║
╠═══════════════╬═══════════════════════════════════════════════════════╣
║ PRIMALS       ║ LOGS & CHAT                                           ║
║               ║                                                       ║
║ 🔒 BearDog    ║ [10:32] BearDog: TPM initialized ✓                    ║
║ 🎵 Songbird   ║ [10:33] Songbird: Discovered 2 nodes                  ║
║ 🍄 ToadStool  ║ [10:34] ToadStool: GPU assigned (RTX 4090)            ║
║ 🏠 NestGate   ║ [10:35] User: Assign NVMe to NestGate                 ║
║ 🐿️ Squirrel   ║ [10:35] biomeOS: Validating assignment...            ║
║ 🌸 petalTongue║ [10:36] NestGate: NVMe mounted ✓                      ║
║               ║ [10:36] 💡 Squirrel: Consider RAID for redundancy    ║
╚═══════════════╩═══════════════════════════════════════════════════════╝
```

---

## 🔄 **Interaction Flow: Device Assignment**

### **Step-by-Step:**

1. **User Action** (petalTongue)
   ```rust
   // User drags "GPU RTX" from device list
   // Drops on "ToadStool" in primal list
   petaltongue.on_drag_drop(device_id: "gpu-rtx", primal_id: "toadstool-1");
   ```

2. **Event to biomeOS** (JSON-RPC)
   ```json
   {
     "jsonrpc": "2.0",
     "method": "biomeos.assign_device",
     "params": {
       "device_id": "gpu-rtx",
       "primal_id": "toadstool-1",
       "user_id": "alice"
     },
     "id": 42
   }
   ```

3. **biomeOS Orchestration** (Business Logic)
   ```rust
   async fn handle_assign_device(params: AssignDeviceParams) -> Result<()> {
       // 1. Authenticate & authorize via BearDog
       beardog.authorize(params.user_id, "assign_device").await?;
       
       // 2. Validate device exists via Songbird
       let device = songbird.get_device(params.device_id).await?;
       
       // 3. Check primal supports device via Songbird
       let primal = songbird.get_primal(params.primal_id).await?;
       if !primal.supports_device(&device) {
           return Err(anyhow!("Primal doesn't support this device"));
       }
       
       // 4. Check resources via ToadStool
       let resources = toadstool.get_device_resources(params.device_id).await?;
       if !primal.has_capacity(resources) {
           return Err(anyhow!("Insufficient resources"));
       }
       
       // 5. Commit assignment via Songbird
       songbird.register_assignment(params.device_id, params.primal_id).await?;
       
       // 6. Store configuration via NestGate
       nestgate.store_assignment(params.into()).await?;
       
       // 7. Update UI via petalTongue
       let new_topology = self.build_topology().await?;
       petaltongue.update_topology(new_topology).await?;
       
       Ok(())
   }
   ```

4. **UI Update** (petalTongue)
   ```rust
   // petalTongue receives update
   petaltongue.render_assignment(
       device_id: "gpu-rtx",
       primal_id: "toadstool-1",
       status: "assigned"
   );
   
   // Shows in logs panel:
   // "[10:34] ToadStool: GPU assigned (RTX 4090) ✓"
   ```

5. **AI Suggestion** (Squirrel, optional)
   ```rust
   // Squirrel analyzes assignment
   let suggestion = squirrel.analyze_assignment(assignment).await?;
   
   // petalTongue shows suggestion:
   // "💡 Squirrel: This GPU is powerful! Consider assigning
   //  additional compute workloads for optimal utilization."
   ```

---

## 🛠️ **What Needs to be Evolved**

### **1. biomeOS Side** (High Priority)

**New Crate: `biomeos-ui`**
```rust
// crates/biomeos-ui/src/lib.rs

pub struct UIOrchestrator {
    petaltongue: PetalTongueClient,
    songbird: SongbirdClient,
    beardog: BearDogClient,
    nestgate: NestGateClient,
    toadstool: ToadStoolClient,
    squirrel: SquirrelClient,
}

impl UIOrchestrator {
    // Device Management
    pub async fn get_available_devices(&self) -> Result<Vec<Device>>;
    pub async fn assign_device(&self, device_id: &str, primal_id: &str) -> Result<()>;
    pub async fn unassign_device(&self, device_id: &str) -> Result<()>;
    
    // Primal Management
    pub async fn start_primal(&self, primal_name: &str) -> Result<()>;
    pub async fn stop_primal(&self, primal_id: &str) -> Result<()>;
    pub async fn restart_primal(&self, primal_id: &str) -> Result<()>;
    
    // State Sync
    pub async fn sync_topology(&self) -> Result<()>;
    pub async fn watch_events(&self) -> impl Stream<Item = UIEvent>;
    
    // User Actions
    pub async fn handle_user_action(&self, action: UserAction) -> Result<()>;
}
```

**New CLI Command:**
```bash
# Launch interactive UI
biomeos ui --mode interactive

# Launch dashboard (read-only)
biomeos ui --mode dashboard

# Launch specific view
biomeos ui --view devices
```

---

### **2. petalTongue Side** (Medium Priority)

**Enhanced JSON-RPC Methods:**
```rust
// petalTongue needs to add:

// Interactive UI
"ui.launch_interactive" -> Opens interactive window
"ui.handle_event" -> Processes user events (drag, click)
"ui.update_panel" -> Updates specific UI panel

// Device Assignment UI
"ui.render_device_list" -> Shows available devices
"ui.render_primal_list" -> Shows active primals
"ui.render_assignments" -> Shows device→primal mappings

// Real-time Updates
"ui.subscribe_updates" -> WebSocket for live updates
"ui.push_event" -> Server pushes event to UI
```

**Component Library:**
```rust
// petalTongue provides reusable components:
- DeviceTreeView
- PrimalStatusPanel
- TopologyGraph (already exists!)
- LogPanel (chat-like)
- ConfigurationForm
- MetricsDashboard
```

---

### **3. Songbird Side** (Low Priority - Mostly Ready)

**Device Registry API:**
```rust
// Songbird needs to track devices:
"songbird.register_device" -> Register new device
"songbird.discover_devices" -> List available devices
"songbird.get_device_info" -> Get device details
"songbird.assign_device" -> Assign device to primal
"songbird.get_assignments" -> List current assignments
```

**Current Status**: Songbird already has service registry. Device registry is a natural extension.

---

## 🎯 **Implementation Phases**

### **Phase 1: Foundation** (1-2 weeks)

1. Create `biomeos-ui` crate
2. Implement `UIOrchestrator` with basic device queries
3. Extend petalTongue with interactive UI methods
4. Create basic device list + primal list panels

**Deliverable**: Static UI showing devices and primals

---

### **Phase 2: Interaction** (2-3 weeks)

1. Implement drag-and-drop in petalTongue
2. Wire up device assignment in biomeOS
3. Add validation and error handling
4. Integrate with Songbird for state

**Deliverable**: Working device assignment (drag & drop)

---

### **Phase 3: Real-time** (1-2 weeks)

1. Add WebSocket support to petalTongue
2. Implement event streaming from biomeOS
3. Add real-time topology updates
4. Integrate logs panel with primal events

**Deliverable**: Live updating UI (no manual refresh)

---

### **Phase 4: Intelligence** (1-2 weeks)

1. Integrate Squirrel for suggestions
2. Add AI-powered recommendations
3. Implement auto-assignment mode
4. Add optimization hints in UI

**Deliverable**: AI-assisted device management

---

### **Phase 5: Persistence** (1 week)

1. Store UI preferences in NestGate
2. Save/load topology configurations
3. Add configuration templates
4. Implement backup/restore

**Deliverable**: Persistent configurations

---

## 📊 **Primal Interaction Summary**

| Primal | Role | What It Provides | UI Integration |
|--------|------|------------------|----------------|
| **petalTongue** | UI Framework | Interactive GUI, rendering, input handling | The visual interface itself |
| **biomeOS** | Orchestrator | Business logic, validation, coordination | Runtime orchestration layer |
| **Songbird** | Registry | Device/primal discovery, assignments | Data source for UI |
| **BearDog** | Security | Authentication, authorization, encryption | Access control for UI |
| **NestGate** | Storage | Configuration persistence, backups | Saves UI state & configs |
| **ToadStool** | Resources | Resource metrics, workload management | Shows compute availability |
| **Squirrel** | AI | Suggestions, optimization, predictions | Smart recommendations |

---

## 🎨 **Concrete Example: Full Flow**

### **Scenario**: User wants to assign GPU to ToadStool

1. **UI Display** (petalTongue)
   - Shows device list with "GPU RTX 4090"
   - Shows primal list with "ToadStool"
   - User drags GPU to ToadStool

2. **Event Capture** (petalTongue → biomeOS)
   ```rust
   petaltongue.emit_event(UIEvent::DragDrop {
       source: "device/gpu-rtx",
       target: "primal/toadstool-1"
   });
   ```

3. **Validation** (biomeOS orchestrates)
   ```rust
   // Check auth
   beardog.authorize(user, "assign_device")?;
   
   // Check device exists
   let device = songbird.get_device("gpu-rtx")?;
   
   // Check primal supports it
   let primal = songbird.get_primal("toadstool-1")?;
   assert!(primal.capabilities.contains("gpu"));
   
   // Check resources
   let resources = toadstool.get_resources("gpu-rtx")?;
   assert!(resources.available > 0);
   ```

4. **Execution** (biomeOS commits)
   ```rust
   // Register assignment
   songbird.register_assignment("gpu-rtx", "toadstool-1")?;
   
   // Store config
   nestgate.store_assignment(assignment)?;
   
   // Apply to primal
   toadstool.attach_device("gpu-rtx")?;
   ```

5. **AI Suggestion** (Squirrel)
   ```rust
   let suggestion = squirrel.suggest_optimization(assignment)?;
   // "💡 GPU is powerful! Consider enabling
   //  CUDA for ML workloads"
   ```

6. **UI Update** (biomeOS → petalTongue)
   ```rust
   petaltongue.update_assignment({
       device: "gpu-rtx",
       primal: "toadstool-1",
       status: "assigned",
       metrics: { utilization: 0, temp: 45 }
   });
   
   petaltongue.add_log({
       timestamp: now(),
       source: "ToadStool",
       message: "GPU assigned (RTX 4090) ✓"
   });
   ```

---

## ✅ **Summary**

### **Division of Labor (TRUE PRIMAL)**

**petalTongue:**
- ✅ Provides the UI framework (Rust + egui)
- ✅ Handles rendering and input
- ✅ Offers reusable components
- ❌ Does NOT contain business logic

**biomeOS:**
- ✅ Orchestrates runtime interactions
- ✅ Implements business logic
- ✅ Coordinates between primals
- ✅ Validates user actions

**Other Primals:**
- ✅ Each provides domain-specific services
- ✅ No primal knows about the UI directly
- ✅ All communicate via JSON-RPC

### **Key Insight**

**petalTongue does the Rust UI code.**  
**biomeOS does the orchestration.**  
**Other primals provide the data.**

This maintains TRUE PRIMAL separation: petalTongue focuses on being the best UI it can be, while biomeOS orchestrates the runtime, and each primal provides its domain expertise.

---

**Status**: Architecture defined, ready for implementation!  
**Next**: Create `biomeos-ui` crate and start Phase 1

🎨 Discord-like UI for biomeOS is 100% feasible! 🎨

