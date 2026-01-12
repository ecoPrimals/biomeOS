# 🎯 biomeOS Evolution Roadmap - Focused Priorities

**Date**: January 10, 2026  
**Status**: Phase 2 Wave 1 (50% complete)  
**Goal**: Robust foundation → UI & AI integration → Network effects

---

## 📋 **Priority Order**

```
1. ✅ Phase 1: Foundation COMPLETE
   └─> CapabilityTaxonomy + SystemPaths + Zero unsafe

2. 🔄 Phase 2: Core Evolution (Current - 2-3 months)
   └─> Capability-based + Path-agnostic + Smart refactoring

3. ⏳ Phase 3: Neural API Maturation (3-4 months)
   └─> Production hardening + Advanced patterns + Performance

4. 🎨 UI/AI Integration (2-3 months)
   ├─> petalTongue (Universal User Interface)
   └─> Squirrel (AI MCP Coordinator)

5. 🌐 Network Effects (Ongoing)
   └─> Primals interweaving + Emergent capabilities
```

---

## 🎯 **Phase 2: Core Evolution** (CURRENT FOCUS)

**Goal**: Make biomeOS robust, agnostic, and production-ready  
**Timeline**: 2-3 months  
**Status**: Wave 1 (50% complete)

### **Waves Overview**

| Wave | Focus | Duration | Status |
|------|-------|----------|--------|
| Wave 1 | Capability-Based Discovery | 1-2 weeks | 🔄 50% |
| Wave 2 | Smart Refactoring | 3-4 weeks | ⏳ Planned |
| Wave 3 | Path Evolution | 2 weeks | ⏳ Planned |
| Wave 4 | Polish & Testing | 2 weeks | ⏳ Planned |

---

### **Wave 1: Capability-Based Discovery** (50% complete)

**✅ Complete**:
- CapabilityTaxonomy integrated into NUCLEUS
- SystemPaths for XDG compliance
- 5 hardcoded path patterns eliminated

**🔄 In Progress**:
- Quick Win #2: SystemPaths in remaining files (~15-20 min)
- Quick Win #3: PrimalRegistry capability methods (1 hour)

**Deliverable**: All discovery uses CapabilityTaxonomy

---

### **Wave 2: Smart Refactoring** (3-4 weeks)

#### **Target: Large Files → Domain Modules**

**biomeos-core/src/clients/beardog.rs** (895 lines) → modular:
```
beardog/
├── mod.rs          # Main client, discovery
├── identity.rs     # Identity verification
├── security.rs     # Encryption/decryption
├── federation.rs   # Family verification
├── trust.rs        # Trust evaluation
└── error.rs        # Errors
```

**biomeos-spore/src/spore.rs** (807 lines) → modular:
```
biomeos-spore/src/
├── creation.rs     # Spore creation
├── deployment.rs   # USB deployment
├── incubation.rs   # Local incubation
├── genetic.rs      # Lineage
├── validation.rs   # Verification
└── logs.rs         # Fossil record
```

**Deliverable**: No files >500 lines, domain-focused

---

### **Wave 3: Path Evolution** (2 weeks)

**Goal**: Eliminate all hardcoded paths

**Pattern**:
```rust
// BEFORE:
let socket = PathBuf::from("/tmp/primal.sock");

// AFTER:
let paths = SystemPaths::new()?;
let socket = paths.primal_socket("primal-id");
```

**Deliverable**: <30 hardcoded paths (from 183)

---

### **Wave 4: Polish & Testing** (2 weeks)

**Coverage**:
- Unit tests (100% new code)
- E2E tests (tower, node, spore, federation)
- Chaos/fault tests (failures, crashes, concurrency)
- Performance benchmarks

**Deliverable**: Production-ready core

---

## 🚀 **Phase 3: Neural API Maturation** (START AFTER PHASE 2)

**Goal**: Harden Neural API, prepare for RootPulse  
**Timeline**: 3-4 months  
**Status**: Planned

### **Phase 3.1: Production Hardening** (4-6 weeks)

**Key Tasks**:
1. **Evolve nucleus_executor.rs**
   - Use CapabilityTaxonomy (not String)
   - Re-enable in biomeos-graph
   
2. **Full NUCLEUS Integration**
   - All discovery through 5-layer protocol
   - Trust-based routing
   
3. **Error Handling**
   - Graceful degradation
   - Retry strategies
   - Health monitoring

**Deliverable**: Production-grade NUCLEUS

---

### **Phase 3.2: Advanced Patterns** (4-6 weeks)

**New Capabilities**:
```toml
# Multi-pattern graphs
[graph.coordination]
pattern = "Hybrid"

[graph.sections.parallel]
nodes = ["task1", "task2", "task3"]

[graph.sections.sequential]
nodes = ["setup", "execute", "teardown"]

# Conditional execution
[graph.nodes.backup]
condition = "{{ env.production == true }}"

# Loops
[graph.nodes.process_files]
loop = { over = "{{ inputs.files }}" }
```

**Deliverable**: Rich coordination patterns

---

### **Phase 3.3: Metrics & Learning** (3-4 weeks)

**Enhanced Learning**:
```rust
// Neural API learns optimal strategies:
pub async fn get_optimal_primal(
    &self,
    capability: CapabilityTaxonomy,
) -> Result<String>;

pub async fn detect_bottlenecks(
    &self,
    graph_id: &str,
) -> Result<Vec<Bottleneck>>;

pub async fn recommend_parallelization(
    &self,
    graph: &PrimalGraph,
) -> Result<ParallelizationPlan>;
```

**Deliverable**: Self-optimizing execution

---

### **Phase 3.4: RootPulse Prep** (2-3 weeks)

**Foundation**:
1. Extend CapabilityTaxonomy for VCS:
   - `TemporalTracking` (LoamSpine)
   - `EphemeralWorkspace` (rhizoCrypt)
   - `ContentStorage` (NestGate)
   - `SemanticAttribution` (SweetGrass)

2. Create RootPulse scaffolding:
   ```
   niches/rootpulse/
   ├── rootpulse.toml
   └── workflows/
       ├── init.toml
       ├── commit.toml
       ├── push.toml
       └── pull.toml
   ```

**Deliverable**: Ready for RootPulse implementation

---

## 🎨 **Phase 4: UI & AI Integration** (AFTER PHASE 3)

**Goal**: Add visual and intelligent interfaces  
**Timeline**: 2-3 months  
**Status**: Planned

### **Phase 4.1: petalTongue Integration** (4-6 weeks)

#### **What is petalTongue?**
```
Universal User Interface (UUI)

Capabilities:
- Multi-modal rendering (visual, audio, text)
- Runtime capability discovery
- Zero hardcoded dependencies
- 6 display backends
- Awakening Experience

Status: Phase 2 primal, production-ready
Location: ecoPrimals/phase2/petalTongue/
```

#### **Integration**
```toml
# New niche: UI
[niche]
name = "ui"
description = "Universal User Interface"

[primals.required]
petaltongue = "^1.0.0"
biomeos = "^0.2.0"
```

#### **Use Cases**
- Visualize primal topology
- Interactive niche deployment
- Real-time metrics dashboard
- Awakening experience for new users

**Deliverable**: Visual interface for biomeOS

---

### **Phase 4.2: Squirrel Integration** (4-6 weeks)

#### **What is Squirrel?**
```
AI MCP (Model Context Protocol) Coordinator

Capabilities:
- Multi-provider AI (OpenAI, Anthropic, Ollama, Gemini)
- Intelligent routing
- Privacy-first
- Makes ANYTHING agentic

Status: Phase 1 primal, production-ready (v2.1.0)
Location: ecoPrimals/phase1/squirrel/
```

#### **Integration**
```toml
# Extend niches with AI
[primals.optional]
squirrel = "^2.1.0"

[ai]
enabled = true
capabilities = [
    "deployment-optimization",
    "anomaly-detection",
    "performance-tuning",
    "error-diagnosis",
]
```

#### **Use Cases**
- AI-assisted deployment
- Anomaly detection
- Performance optimization
- Interactive troubleshooting
- Code review (RootPulse)

**Deliverable**: AI-enhanced operations

---

### **Phase 4.3: Network Effects** (Ongoing)

#### **UI + AI Synergy**

**Example: Visual Deployment with AI**
```
1. User opens petalTongue UI
2. UI shows available primals, topology
3. User drags to create niche
4. Squirrel: "I suggest placing BearDog on node-alpha for HSM access"
5. User accepts
6. Neural API deploys with optimal placement
```

**Example: Interactive Troubleshooting**
```
1. petalTongue shows error
2. User clicks "Diagnose"
3. Squirrel analyzes logs, metrics
4. Squirrel: "Port 900 in use. Suggest Unix socket."
5. petalTongue shows "Apply Fix" button
6. User clicks
7. Neural API redeploys with fix
```

#### **Primal Network Effects**
```
biomeOS (orchestrator)
├─> Songbird (discovery)
├─> BearDog (security)
├─> Toadstool (compute)
├─> NestGate (storage)
├─> petalTongue (UI) ← Makes everything visual
├─> Squirrel (AI) ← Makes everything intelligent
└─> [Future primals automatically benefit!]
```

**Each primal enhances the others!**

---

## 📊 **Timeline Summary**

```
Now:            Phase 2 Wave 1 (50% complete)
+2 weeks:       Phase 2 Wave 1 complete
+6 weeks:       Phase 2 Wave 2 complete
+8 weeks:       Phase 2 Wave 3 complete
+10 weeks:      Phase 2 Wave 4 complete
                ✅ Phase 2 COMPLETE

+14 weeks:      Phase 3.1 complete
+20 weeks:      Phase 3.2 complete
+24 weeks:      Phase 3.3 complete
+27 weeks:      Phase 3.4 complete
                ✅ Phase 3 START COMPLETE

+33 weeks:      Phase 4.1 complete
+39 weeks:      Phase 4.2 complete
                ✅ Phase 4 COMPLETE

Total: ~9 months to robust UI/AI-enhanced biomeOS!
```

---

## 🎯 **Success Criteria**

### **Phase 2 Complete**
- ✅ All discovery uses CapabilityTaxonomy
- ✅ <30 hardcoded paths (>80% reduction)
- ✅ <20 hardcoded names (>80% reduction)
- ✅ No files >500 lines
- ✅ 100% test coverage
- ✅ Production-ready core

### **Phase 3 Start Complete**
- ✅ NUCLEUS fully integrated
- ✅ Advanced coordination patterns
- ✅ Metrics-based learning
- ✅ RootPulse foundation ready

### **Phase 4 Complete**
- ✅ petalTongue integrated
- ✅ Squirrel integrated
- ✅ Network effects demonstrated
- ✅ User-friendly operations
- ✅ Intelligent automation

---

## 💡 **Why This Order?**

### **1. Foundation First** (Phase 2)
```
Must have robust core before adding complexity:
- Capability-based discovery
- Path-agnostic operations
- Clean architecture
- Comprehensive tests

Without this: UI and AI built on shaky ground
```

### **2. Neural API Maturation** (Phase 3)
```
Must have solid orchestration:
- Production-grade error handling
- Advanced patterns
- Learning capabilities
- Performance optimization

Without this: Can't handle complex workflows
```

### **3. UI & AI Enhancement** (Phase 4)
```
Add interfaces AFTER core is solid:
- petalTongue makes it visual
- Squirrel makes it intelligent
- Network effects multiply value

Without solid core: Pretty but fragile
```

### **4. RootPulse Later** (Phase 5+)
```
Builds on everything:
- Needs mature Neural API
- Benefits from UI (petalTongue)
- Enhanced by AI (Squirrel)
- Validates ecosystem

Timing: After Phase 4, ~9-12 months
```

---

## 🎊 **Bottom Line**

### **Strategic Path**
```
Solid Core (Phase 2)
  ↓
Mature Orchestration (Phase 3)
  ↓
UI & AI Enhancement (Phase 4)
  ↓
Network Effects (Ongoing)
  ↓
RootPulse & Beyond (Phase 5+)
```

### **Network Effects**
```
Each integration multiplies value:
- petalTongue × Squirrel = Intelligent visual interface
- Neural API × NUCLEUS = Secure orchestration
- All primals × UI/AI = Enhanced for humans & machines
- Foundation × Enhancement = Production-ready ecosystem
```

### **Timeline to UI/AI biomeOS**
- **Phase 2**: ~10 weeks remaining
- **Phase 3**: ~4 months
- **Phase 4**: ~2-3 months
- **Total**: ~9 months

---

**Focus: Complete Phase 2 → Start Phase 3 → Integrate UI/AI!** 🎯

**Result: Robust, visual, intelligent primal ecosystem!** 🚀

