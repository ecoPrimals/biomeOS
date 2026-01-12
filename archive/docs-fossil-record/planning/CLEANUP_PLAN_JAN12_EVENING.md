# 🧹 Cleanup Plan - January 12, 2026 Evening

**Goal**: Archive completed session docs, clean outdated TODOs, organize codebase  
**Principle**: Keep as fossil record (archive, don't delete)

---

## 📦 **Root Docs to Archive**

### **Session Complete Docs** (Move to `archive/docs-fossil-record/jan12-session/`)

These are complete and archived for historical reference:

1. `SESSION_COMPLETE_JAN12_AFTERNOON.md` - ✅ Complete, archive
2. `SESSION_FINAL_JAN12_2026.md` - ✅ Complete, archive
3. `SESSION_FINAL_RUST_NEURAL_API.md` - ✅ Complete, archive
4. `SESSION_SUMMARY_JAN12_ATOMIC_LIVESPORE.md` - ✅ Complete, archive
5. `START_HERE_JAN12.md` - Superseded by `START_HERE_JAN12_EVENING.md`, archive
6. `ROOT_DOCS_UPDATED_JAN12.md` - Duplicate info, archive
7. `ATOMIC_DEPLOYMENT_PROGRESS_JAN12.md` - Session doc, archive
8. `QUICK_STATUS_JAN12.md` - Superseded by STATUS.md, archive

### **Implementation Complete Docs** (Move to `archive/docs-fossil-record/implementations/`)

These document completed features:

1. `TESTING_SUITE_COMPLETE.md` - ✅ Tests complete, keep as reference
2. `POLISHED_TESTED_COMPLETE.md` - ✅ Polish complete, keep as reference
3. `NEURAL_API_EXECUTOR_COMPLETE.md` - ✅ Implementation complete
4. `RUST_EVOLUTION_COMPLETE.md` - ✅ Evolution complete
5. `GENETIC_LINEAGE_IMPLEMENTATION_COMPLETE.md` - ✅ Implementation complete
6. `GENETIC_LINEAGE_TEST_REPORT.md` - ✅ Tests complete
7. `PURE_RUST_EVOLUTION_COMPLETE.md` - ✅ Evolution complete
8. `LIVESPORE_PHASE1_COMPLETE.md` - ✅ Phase 1 complete
9. `NUCLEUS_EVOLUTION_COMPLETE.md` - ✅ Evolution complete
10. `UI_PHASES_4_5_6_COMPLETE.md` - ✅ Phases complete
11. `NEURAL_API_ATOMIC_INTEGRATION_COMPLETE.md` - ✅ Integration complete

### **Handoff Docs** (Move to `archive/docs-fossil-record/handoffs/`)

These are historical handoffs:

1. `FINAL_HANDOFF_JAN11.md` - From Jan 11, archive
2. `PRODUCTION_HANDOFF_JAN11_2026.md` - From Jan 11, archive
3. `COLLABORATIVE_INTELLIGENCE_HANDOFF.md` - Completed, archive
4. `PRIMAL_SOCKET_CONFIG_HANDOFF.md` - Completed, archive
5. `DEEP_DEBT_AUDIT_JAN11_2026.md` - From Jan 11, archive
6. `DEPLOYMENT_TESTING_SUMMARY_JAN11.md` - From Jan 11, archive

### **Keep in Root** (Active/Current)

These should stay:

- `README.md` ✅
- `STATUS.md` ✅
- `START_HERE.md` ✅
- `START_HERE_JAN12_EVENING.md` ✅ (Latest)
- `ROOT_DOCS_INDEX.md` ✅
- `REMAINING_WORK_SUMMARY_JAN12.md` ✅ (Active planning)
- `BIOMEOS_VS_PRIMAL_RESPONSIBILITIES.md` ✅ (Active reference)
- `SPECS_CLEANUP_PLAN.md` ✅ (Active planning)
- `BIOMEOS_ATOMICS_ARCHITECTURE.md` ✅ (Core reference)
- `LIVESPORE_ROADMAP.md` ✅ (Active planning)
- `LIVESPORE_INVESTIGATION.md` ✅ (Active planning)
- `TOWER_ATOMIC_SUCCESS_JAN12.md` ✅ (Recent success)
- `NESTGATE_ATOMIC_HANDOFF.md` ✅ (Active handoff)
- `NESTGATE_UNBLOCKED_JAN12.md` ✅ (Recent status)
- All petalTongue docs ✅ (Recent integration)
- `GENETIC_LINEAGE_DEPLOYMENT_DEMO.md` ✅ (Active demo)
- `QUICK_LINEAGE_REFERENCE.md` ✅ (Active reference)
- `PRIMAL_LAUNCHER_README.md` ✅ (Active guide)
- `REFINED_ROADMAP.md` ✅ (Active planning)
- `MASTER_DOCUMENTATION_INDEX.md` ✅ (Navigation)

---

## 💻 **Code TODOs Analysis**

### **Legitimate TODOs** (Keep - Future Work)

These are real future work items:

**High Priority**:
```rust
// crates/biomeos-ui/src/orchestrator.rs - Real integration work needed
// TODO: Implement discovery method in PetalTongueClient (line 183)
// TODO: Implement discovery method in SongbirdClient (line 188)
// TODO: Implement discovery method in BearDogClient (line 193)
// TODO: Implement discovery method in NestGateClient (line 198)
// TODO: Implement discovery method in ToadStoolClient (line 203)
// TODO: Implement discovery method in SquirrelClient (line 208)
```

**Medium Priority**:
```rust
// crates/biomeos-graph/src/templates.rs - NestGate integration
// TODO: Use Songbird to discover NestGate by capability (line 103)
// TODO: Call NestGate storage.* via JSON-RPC (lines 277, 289, 300, 311)
```

**Low Priority**:
```rust
// crates/biomeos-graph/src/ai_advisor.rs - Squirrel integration
// TODO: Implement actual Squirrel discovery via Songbird (line 213)
// TODO: Implement actual Squirrel integration (line 236)
```

### **Obsolete TODOs** (Convert to comments or remove)

These are already done or no longer relevant:

```rust
// crates/biomeos-atomic-deploy/Cargo.toml:23
# TODO: Create neural-api-client crate
// ❌ OBSOLETE - neuralAPI is in biomeos-atomic-deploy now

// crates/biomeos-graph/src/lib.rs:24,61
// pub mod nucleus_executor; // TODO: Re-enable after Wave 2 evolution
// ❌ OBSOLETE - Wave 2 complete, just needs cleanup

// crates/biomeos-core/src/graph_deployment.rs:401
// TODO: Re-enable after Wave 2 evolution
// ❌ OBSOLETE - Wave 2 complete
```

### **False Positives** (Already Implemented)

These are implemented but TODO still exists:

```rust
// crates/biomeos-atomic-deploy/src/health_check.rs:6,58
// TODO: Implement JSON-RPC health checks
// ✅ DONE - health_check.rs has working implementation

// crates/biomeos-atomic-deploy/src/deployment_graph.rs:141,147
// TODO: Implement TOML export / topological sort
// ✅ DONE - neural_executor.rs has full implementation
```

---

## 📂 **Proposed Archive Structure**

```
archive/
├── docs-fossil-record/
│   ├── jan12-session/
│   │   ├── SESSION_COMPLETE_JAN12_AFTERNOON.md
│   │   ├── SESSION_FINAL_JAN12_2026.md
│   │   ├── SESSION_FINAL_RUST_NEURAL_API.md
│   │   ├── SESSION_SUMMARY_JAN12_ATOMIC_LIVESPORE.md
│   │   ├── START_HERE_JAN12.md
│   │   ├── ROOT_DOCS_UPDATED_JAN12.md
│   │   ├── ATOMIC_DEPLOYMENT_PROGRESS_JAN12.md
│   │   └── QUICK_STATUS_JAN12.md
│   │
│   ├── implementations/
│   │   ├── TESTING_SUITE_COMPLETE.md
│   │   ├── POLISHED_TESTED_COMPLETE.md
│   │   ├── NEURAL_API_EXECUTOR_COMPLETE.md
│   │   ├── RUST_EVOLUTION_COMPLETE.md
│   │   ├── GENETIC_LINEAGE_IMPLEMENTATION_COMPLETE.md
│   │   ├── GENETIC_LINEAGE_TEST_REPORT.md
│   │   ├── PURE_RUST_EVOLUTION_COMPLETE.md
│   │   ├── LIVESPORE_PHASE1_COMPLETE.md
│   │   ├── NUCLEUS_EVOLUTION_COMPLETE.md
│   │   ├── UI_PHASES_4_5_6_COMPLETE.md
│   │   └── NEURAL_API_ATOMIC_INTEGRATION_COMPLETE.md
│   │
│   ├── handoffs/
│   │   ├── FINAL_HANDOFF_JAN11.md
│   │   ├── PRODUCTION_HANDOFF_JAN11_2026.md
│   │   ├── COLLABORATIVE_INTELLIGENCE_HANDOFF.md
│   │   ├── PRIMAL_SOCKET_CONFIG_HANDOFF.md
│   │   ├── DEEP_DEBT_AUDIT_JAN11_2026.md
│   │   └── DEPLOYMENT_TESTING_SUMMARY_JAN11.md
│   │
│   ├── jan4-session/ (existing)
│   ├── jan9-session/ (existing)
│   ├── jan10-final-session/ (existing)
│   └── README.md (update with new archives)
│
└── legacy_code/ (existing)
```

---

## 🎯 **Execution Plan**

### **Step 1: Create Archive Directories**
```bash
mkdir -p archive/docs-fossil-record/jan12-session
mkdir -p archive/docs-fossil-record/implementations
mkdir -p archive/docs-fossil-record/handoffs
```

### **Step 2: Move Session Docs**
```bash
mv SESSION_COMPLETE_JAN12_AFTERNOON.md archive/docs-fossil-record/jan12-session/
mv SESSION_FINAL_JAN12_2026.md archive/docs-fossil-record/jan12-session/
mv SESSION_FINAL_RUST_NEURAL_API.md archive/docs-fossil-record/jan12-session/
mv SESSION_SUMMARY_JAN12_ATOMIC_LIVESPORE.md archive/docs-fossil-record/jan12-session/
mv START_HERE_JAN12.md archive/docs-fossil-record/jan12-session/
mv ROOT_DOCS_UPDATED_JAN12.md archive/docs-fossil-record/jan12-session/
mv ATOMIC_DEPLOYMENT_PROGRESS_JAN12.md archive/docs-fossil-record/jan12-session/
mv QUICK_STATUS_JAN12.md archive/docs-fossil-record/jan12-session/
```

### **Step 3: Move Implementation Docs**
```bash
mv TESTING_SUITE_COMPLETE.md archive/docs-fossil-record/implementations/
mv POLISHED_TESTED_COMPLETE.md archive/docs-fossil-record/implementations/
mv NEURAL_API_EXECUTOR_COMPLETE.md archive/docs-fossil-record/implementations/
mv RUST_EVOLUTION_COMPLETE.md archive/docs-fossil-record/implementations/
mv GENETIC_LINEAGE_IMPLEMENTATION_COMPLETE.md archive/docs-fossil-record/implementations/
mv GENETIC_LINEAGE_TEST_REPORT.md archive/docs-fossil-record/implementations/
mv PURE_RUST_EVOLUTION_COMPLETE.md archive/docs-fossil-record/implementations/
mv LIVESPORE_PHASE1_COMPLETE.md archive/docs-fossil-record/implementations/
mv NUCLEUS_EVOLUTION_COMPLETE.md archive/docs-fossil-record/implementations/
mv UI_PHASES_4_5_6_COMPLETE.md archive/docs-fossil-record/implementations/
mv NEURAL_API_ATOMIC_INTEGRATION_COMPLETE.md archive/docs-fossil-record/implementations/
```

### **Step 4: Move Handoff Docs**
```bash
mv FINAL_HANDOFF_JAN11.md archive/docs-fossil-record/handoffs/
mv PRODUCTION_HANDOFF_JAN11_2026.md archive/docs-fossil-record/handoffs/
mv COLLABORATIVE_INTELLIGENCE_HANDOFF.md archive/docs-fossil-record/handoffs/
mv PRIMAL_SOCKET_CONFIG_HANDOFF.md archive/docs-fossil-record/handoffs/
mv DEEP_DEBT_AUDIT_JAN11_2026.md archive/docs-fossil-record/handoffs/
mv DEPLOYMENT_TESTING_SUMMARY_JAN11.md archive/docs-fossil-record/handoffs/
```

### **Step 5: Update Archive README**
Add sections for new archives with descriptions.

### **Step 6: Clean Code TODOs**
- Remove 3 obsolete TODOs (already implemented)
- Update 2 false positive TODOs with actual status
- Keep 88 legitimate future work TODOs

---

## 📊 **Before vs After**

### **Root Directory**
**Before**: 60+ docs  
**After**: ~35 docs (25 archived as fossil record)

### **Code TODOs**
**Before**: 93 TODOs  
**After**: 88 legitimate TODOs (5 cleaned)

### **Organization**
**Before**: Mixed session/implementation/handoff docs in root  
**After**: Clean root with active docs, historical docs archived

---

## ✅ **Benefits**

1. **Cleaner Root** - Only active/current docs in root
2. **Preserved History** - All docs kept as fossil record
3. **Better Navigation** - Clear separation of concerns
4. **No Data Loss** - Everything archived, nothing deleted
5. **Git Ready** - Clean state for push via SSH

---

## 🚀 **Ready to Execute**

Run the cleanup script or execute steps manually.

**Different orders of the same architecture.** 🍄🐸

