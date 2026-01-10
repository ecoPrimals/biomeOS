# 🌸 petalTongue: The Human Interface for biomeOS

**Date**: January 10, 2026  
**Status**: Architecture Document  
**Version**: 1.0

---

## 🎯 Core Principle

> **petalTongue is THE human interface for biomeOS. Agents use CLIs and APIs. Humans use petalTongue.**

### The Separation

```
┌─────────────────────────────────────────────────────────────────┐
│                      biomeOS ECOSYSTEM                          │
│                                                                 │
│  🤖 AGENTS                        👤 HUMANS                    │
│  ────────────                     ────────────                 │
│  • CLI tools                      • petalTongue GUI             │
│  • Binaries                       • Visual topology             │
│  • JSON-RPC API                   • Interactive graphs          │
│  • Scripts                        • Real-time monitoring        │
│  • Automation                     • Click & explore             │
│  • Agentic USB                    • Beautiful visualizations    │
│  • CI/CD                          • Intuitive controls          │
│  • MCP/Squirrel                   • Multi-modal rendering       │
│                                                                 │
│  Fast, Scriptable, Headless       Intuitive, Visual, Interactive│
└─────────────────────────────────────────────────────────────────┘
```

---

## 🎨 What Humans See in petalTongue

### 1. **Live Ecosystem View** 📊

Real-time visualization of the entire biomeOS ecosystem:

- **All Running Primals**: See every active primal (BearDog, Songbird, ToadStool, etc.)
- **Real-time Connections**: Live visualization of inter-primal communication
- **Health Status**: Color-coded indicators (🟢 healthy, 🟡 degraded, 🔴 critical)
- **Performance Metrics**: CPU, memory, network usage per primal
- **Interactive Topology**: Click, drag, zoom, inspect

**Use Case**: System administrator wants to check if all primals are healthy.

```
Human: Opens petalTongue
        → Sees live topology
        → All primals green
        → Clicks "BearDog" node
        → Views encryption operations/sec
        → Confirms system is healthy
```

---

### 2. **Spore Deployment Visualization** 📀

Visual representation of USB spore creation and deployment:

- **Cold Spore → Live Awakening**: Watch the transformation
- **Genetic Lineage Tree**: Visual family relationships
- **Deployment Progress**: Real-time status updates
- **Success/Failure States**: Clear visual feedback
- **Clone Sibling Operations**: See parent → child relationships

**Use Case**: Human operator deploying a Live USB Spore.

```
Human: Opens petalTongue
        → Click "Deploy Spore"
        → Select "Parent: tower1"
        → Create "Sibling: usb-portable"
        → Watch visualization:
           - Parent spore (blue, desktop)
           - .family.seed shared (dashed line)
           - USB spore created (cold, gray)
           - Inserted into PC (animation)
           - Boot sequence (orange → yellow → green)
           - Live awakening (green, deployed)
        → Success! Beautiful visual confirmation
```

---

### 3. **Discovery Architecture** 🧬

Shows HOW biomeOS works internally:

- **Capability-Based Discovery**: Visual demonstration of `discover_by_capability()`
- **Zero Hardcoding**: See runtime service mesh formation
- **Songbird Coordination**: Watch primals register and discover
- **Unix Socket Connections**: Visualize JSON-RPC over Unix sockets

**Use Case**: Developer/operator understanding the system architecture.

```
Human: Opens petalTongue
        → Select "NUCLEUS Discovery" visualization
        → Sees:
           - Application queries NUCLEUS
           - NUCLEUS calls discover_by_capability("encryption")
           - Songbird returns BearDog endpoint
           - Connection established over Unix socket
        → Understands: Zero hardcoding, runtime discovery!
```

---

### 4. **Orchestration Flows** 🧠

Intent-driven multi-primal workflows:

- **Intent → DAG**: Watch natural language become execution graph
- **Multi-Primal Workflows**: See 5+ primals working together
- **Task Scheduling**: Visualize parallel execution
- **Result Aggregation**: See data flow through ecosystem

**Use Case**: Understanding complex multi-primal operations.

```
Human: "Secure my data and show status"
        → Opens petalTongue
        → Watches Neural API parse intent
        → RootPulse builds execution graph:
           Step 1: Songbird (discover storage)
           Step 2: BearDog (encrypt data)
           Step 3: NestGate (store encrypted)
           Step 4: petalTongue (show status)
           Step 5: Squirrel (AI insights)
        → Sees execution flow in real-time
        → Result: 234ms, all green checkmarks
```

---

## 🚀 Use Cases

### **Human Operator: System Monitoring**

**Goal**: Check biomeOS ecosystem health

```
1. Open petalTongue GUI
2. Auto-discovers via Songbird
3. Renders live topology:
   - 7 primals running
   - 49 connections active
   - All green (healthy)
   - BearDog: 1,247 ops/sec
   - NestGate: 89% storage used
4. Clicks "ToadStool" node
5. Drills down:
   - 3 workloads running
   - 2 on WASM, 1 native
   - CPU: 23%, Memory: 512MB
6. Satisfied: All systems operational ✅
```

---

### **Human Deployer: USB Spore Creation**

**Goal**: Deploy a Live USB Spore to a fresh PC

```
1. Open petalTongue GUI
2. Click "Deploy Spore" button
3. Wizard appears:
   - Parent: tower1 (desktop)
   - Target: USB (/dev/sdb)
   - Sibling name: usb-portable
4. Click "Create Sibling"
5. Watch visualization:
   - Genetic lineage established (.family.seed)
   - Binaries copied (progress bar)
   - Config generated (✓)
   - Bootloader written (✓)
6. USB ready! Visual confirmation
7. Insert USB into fresh PC
8. Watch in petalTongue:
   - USB detected (gray → orange)
   - Boot sequence (orange → yellow)
   - biomeOS awakens (yellow → green)
   - BearDog activates (encryption ready)
   - Songbird discovers family (trust established)
9. Success! Agentic USB operational ✅
```

---

### **AI Agent (Squirrel): Automated Deployment**

**Goal**: Deploy 3 spores to nodes A, B, C

```bash
# Agent uses CLI (no GUI)
biomeos spore create --sibling node-a --output /dev/sdc
biomeos spore create --sibling node-b --output /dev/sdd
biomeos spore create --sibling node-c --output /dev/sde

# JSON responses for automation
{
  "status": "success",
  "spore_id": "node-a",
  "family_seed": "abc123...",
  "deployment_time_ms": 2341
}
```

**Meanwhile**: Human can watch all 3 deployments in petalTongue GUI if desired!

---

## 🌸 petalTongue as Universal UI

### Multi-Modal Input

- **GUI**: Native window (egui)
- **Terminal**: ASCII art (SSH-friendly)
- **Web**: Browser-based (future)
- **Voice**: Audio commands (future, accessibility)
- **Gesture**: Touch interfaces (future, tablets)

### Multi-Modal Output

- **Interactive Graphs**: Pan, zoom, click, explore
- **Terminal ASCII**: Works over SSH
- **SVG Exports**: Documentation-ready
- **PNG Exports**: Presentation-ready
- **JSON**: API integration
- **DOT**: Graphviz advanced layouts
- **Audio Feedback**: Accessibility, signatures

### Universal Accessibility

- **Works Everywhere**: Linux, macOS, Windows, SSH
- **No Dependencies**: Pure Rust + egui (self-contained)
- **Headless Mode**: Servers, CI/CD, automation
- **GUI Mode**: Desktops, laptops, workstations
- **Responsive**: Adapts to terminal size/window resolution

---

## 🎯 Design Principles

### 1. **Separation of Concerns**

- **Agents**: Fast, scriptable, headless, JSON-based
- **Humans**: Visual, interactive, intuitive, beautiful

### 2. **Zero Friction for Humans**

- No command memorization (visual exploration)
- No JSON parsing (rendered graphs)
- No mental model building (see it live)
- No guessing (tooltips, labels, colors)

### 3. **Full Power for Agents**

- All functionality accessible via CLI/API
- JSON responses for parsing
- Scriptable workflows
- Automatable deployments

### 4. **Live by Default**

- petalTongue always shows LIVE data (not sandbox)
- Auto-discovers via Songbird
- Real-time updates (not snapshots)
- Interactive (not static)

---

## 🔧 Configuration

### For Live Discovery (Production)

```bash
# Ensure Songbird is running
bin/primals/songbird-bin &

# Start petalTongue (auto-discovers)
bin/primals/petal-tongue
```

### For Static Visualizations (Documentation)

```bash
# Render JSON to interactive GUI
bin/primals/petal-tongue visualizations/nucleus__discovery_architecture.json

# Export to SVG (no GUI)
bin/primals/petal-tongue-headless --mode svg --output diagram.svg < viz.json
```

---

## 📊 Visualization Types

### 1. **Topology (Live)**

Real-time ecosystem:
- Nodes: Primals
- Edges: Connections
- Colors: Health status
- Labels: Names, IPs, metrics

### 2. **Workflows (Historical)**

Past executions:
- DAG graphs
- Execution paths
- Timing data
- Success/failure states

### 3. **Lineage (Genealogy)**

Spore families:
- Parent → child relationships
- .family.seed sharing
- Trust networks
- Deployment history

### 4. **Architecture (Educational)**

System design:
- How primals work
- Discovery mechanisms
- Capability taxonomy
- Zero hardcoding demos

---

## 🎊 Impact

### Before petalTongue

```bash
# Human trying to understand system:
curl http://localhost:3000/health
curl http://localhost:8080/health
curl http://localhost:9000/health
# ... mental model required
# ... parse JSON manually
# ... no visual feedback
```

### After petalTongue

```bash
# Human:
bin/primals/petal-tongue
# → Beautiful GUI opens
# → All primals visible
# → Health status clear
# → Interactive exploration
# → Intuitive understanding ✨
```

---

## 🚀 Next Steps

1. **Ensure Live Discovery Works**
   - Songbird must be running
   - petalTongue auto-discovers
   - No sandbox/showcase mode

2. **Add Spore Deployment Wizard**
   - Interactive spore creation
   - Visual progress tracking
   - Genetic lineage display

3. **Real-time Metrics**
   - Live performance graphs
   - Historical data
   - Alerting visualization

4. **Multi-Instance Support**
   - Visualize multiple biomeOS instances
   - Cross-node federation
   - BTSP tunnels visualization

---

## 📖 Summary

**petalTongue is not just a visualization tool - it's THE way humans interact with biomeOS.**

- **Agents** (AI/automation): CLI, binaries, JSON-RPC
- **Humans** (operators/developers): petalTongue GUI

This separation ensures:
- ✅ Agents can work fast and headless
- ✅ Humans get intuitive, beautiful interfaces
- ✅ Both have full access to biomeOS capabilities
- ✅ Zero friction for their respective use cases

**Result**: biomeOS becomes accessible to everyone - from automated agents to human operators.

---

**🌸 petalTongue: Universal User Interface for the ecoPrimals Ecosystem 🌸**

