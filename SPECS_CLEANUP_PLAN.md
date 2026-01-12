# 🧹 Specs Directory Cleanup Plan

**Date**: January 12, 2026  
**Current Count**: 36 specs + 3 examples  
**Goal**: Organize, mark status, archive outdated

---

## 📋 **Cleanup Actions**

### **Action 1: Add Implementation Status Banners**

Add status banner to top of each implemented spec:

**Files to Update**:
```
specs/GRAPH_BASED_ORCHESTRATION_SPEC.md
specs/COLLABORATIVE_INTELLIGENCE_SPEC.md
specs/INTERACTIVE_UI_SPEC.md
```

**Banner Format**:
```markdown
---
**Implementation Status**: ✅ COMPLETE  
**Implemented**: January 2026  
**Code**: `crates/biomeos-graph/`, `crates/biomeos-ui/`  
**Tests**: Passing  
---
```

### **Action 2: Create Archive Directory**

Move outdated/superseded specs to archive:

**Create**: `specs/archive/`

**Move These**:
- `UNIVERSAL_CONNECTOR_SPEC.md` → Superseded by JSON-RPC
- `UNIVERSAL_PARSER_ADAPTER_SPEC.md` → Likely obsolete
- `BOOTSTRAP_ORCHESTRATION_SEQUENCE.md` → Superseded by neuralAPI graphs
- `boot-observability.md` → Review needed, likely obsolete

**Reasoning**: Keep specs/ focused on current/active specifications

### **Action 3: Update README.md Status**

Update `specs/README.md` with current reality:

**Changes Needed**:
1. Mark neuralAPI as "✅ Engine Complete, ⏳ Server Pending"
2. Mark Collaborative Intelligence as "✅ Complete"
3. Update petalTongue status to "✅ Harvested, ⏳ Integration Pending"
4. Add NUCLEUS status: "🟢 Spec Complete, Ready for Implementation"
5. Add liveSpore status: "🟢 Spec Complete (990 lines), Ready for Implementation"
6. Update atomic deployment: "✅ Tower, ✅ Node, ⏳ Nest"

### **Action 4: Create Missing Implementation Specs**

**New Specs Needed**:

1. **`NUCLEUS_IMPLEMENTATION_ROADMAP.md`**
   - Break down NUCLEUS into 4-week implementation phases
   - Define milestones
   - Integration checkpoints

2. **`LIVESPORE_IMPLEMENTATION_PHASES.md`**
   - Detailed 12-week phase breakdown
   - Testing strategy per phase
   - Primal integration timeline

3. **`JSONRPC_SERVER_GUIDE.md`**
   - Standard JSON-RPC server implementation pattern
   - Socket management
   - Error handling
   - Testing approach

### **Action 5: Mark Primal-Owned Specs**

Add clear headers to primal-owned specs:

**Files**:
- `PETALTONGUE_UI_AND_VISUALIZATION_SPECIFICATION.md`
- `TOADSTOOL_BIOMEOS_UNIFICATION_SPEC.md`

**Header to Add**:
```markdown
---
**Ownership**: External Primal (petalTongue/ToadStool)  
**biomeOS Role**: Integration via JSON-RPC  
**Status**: ✅ Primal implements, biomeOS integrates  
---
```

### **Action 6: Update Specification Categories**

Reorganize README categories to reflect current work:

**New Category Structure**:
1. **Core Architecture** (9 specs) - Keep as-is
2. **Neural API & Graphs** (5 specs) - Mark as ✅ Implemented
3. **Collaborative Intelligence** (1 spec) - Mark as ✅ Complete
4. **User Interface** (2 specs) - Update status
5. **Security & Encryption** (5 specs) - Reference/Planning
6. **Federation & Distribution** (3 specs) - Reference/Planning
7. **Primal Integration** (4 specs) - Reference/Planning
8. **BYOB** (2 specs + 3 examples) - Future
9. **Manifests & Management** (3 specs) - Future
10. **LiveSpore** (2 specs) - **ACTIVE - Ready for Implementation**
11. **NUCLEUS** (3 specs) - **ACTIVE - Ready for Implementation**
12. **Archived** (4 specs) - **NEW** - Outdated/Superseded

---

## 📊 **Proposed New Structure**

```
specs/
├── README.md (updated with new categories)
│
├── active/ (Currently being implemented)
│   ├── NUCLEUS_SECURE_DISCOVERY_PROTOCOL.md
│   ├── LIVESPORE_ARCHITECTURE_SPEC.md
│   ├── LIVESPORE_PRIMAL_RESPONSIBILITIES.md
│   ├── NUCLEUS_IMPLEMENTATION_ROADMAP.md (NEW)
│   ├── LIVESPORE_IMPLEMENTATION_PHASES.md (NEW)
│   └── JSONRPC_SERVER_GUIDE.md (NEW)
│
├── implemented/ (Completed specs - reference)
│   ├── GRAPH_BASED_ORCHESTRATION_SPEC.md
│   ├── COLLABORATIVE_INTELLIGENCE_SPEC.md
│   ├── INTERACTIVE_UI_SPEC.md
│   └── ATOMIC_DEPLOYMENT_SYSTEM_SPEC.md
│
├── planning/ (Future work)
│   ├── BYOB_*.md
│   ├── CRYPTO_LOCK_*.md
│   ├── ENCRYPTION_STRATEGY_SPEC.md
│   ├── UNIVERSAL_FEDERATION_SPEC.md
│   └── ... (all reference specs)
│
├── primal-owned/ (External primal specs)
│   ├── PETALTONGUE_UI_AND_VISUALIZATION_SPECIFICATION.md
│   └── TOADSTOOL_BIOMEOS_UNIFICATION_SPEC.md
│
├── archive/ (Outdated/Superseded)
│   ├── UNIVERSAL_CONNECTOR_SPEC.md
│   ├── UNIVERSAL_PARSER_ADAPTER_SPEC.md
│   ├── BOOTSTRAP_ORCHESTRATION_SEQUENCE.md
│   └── boot-observability.md
│
└── examples/ (BYOB examples)
    ├── ai-research.biome.yaml
    ├── basic-development.biome.yaml
    └── secure-enterprise.biome.yaml
```

**Alternative**: Keep flat structure, just update README categories and add status banners

---

## 🎯 **Recommended Approach**

**Option A: Minimal (Recommended for now)**
- Keep flat structure
- Add status banners to implemented specs
- Create `archive/` subdirectory for outdated specs
- Update README.md with current status
- Create 3 new implementation guides

**Option B: Full Reorganization**
- Reorganize into subdirectories (active, implemented, planning, etc.)
- More organized but requires updating all cross-references
- Better for long-term but disruptive now

**Recommendation**: Option A (minimal) - defer full reorganization until after NUCLEUS/liveSpore implementation

---

## 📝 **Immediate Cleanup Tasks** (1-2 hours)

1. ✅ Create `specs/archive/` directory
2. Move 4 specs to archive
3. Add status banners to 3 implemented specs
4. Update `specs/README.md`:
   - Mark implemented specs as ✅
   - Update atomic deployment status
   - Add petalTongue integration status
   - Highlight NUCLEUS & liveSpore as active
5. Create 3 new implementation guides:
   - NUCLEUS_IMPLEMENTATION_ROADMAP.md
   - LIVESPORE_IMPLEMENTATION_PHASES.md
   - JSONRPC_SERVER_GUIDE.md

---

**Different orders of the same architecture.** 🍄🐸

**Next**: Execute cleanup → Clear picture of remaining work

