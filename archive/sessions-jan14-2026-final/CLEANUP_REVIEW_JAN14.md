# 🧹 Cleanup Review - January 14, 2026

**Date**: January 14, 2026  
**Status**: Post-NUCLEUS Deployment Cleanup  
**Goal**: Remove outdated code, docs, and implementations

---

## 📊 **Current State**

### **✅ What's Working**
- 3 atomics deployed (Tower, Node, Nest)
- Neural API orchestration functional
- 5 primals running (BearDog, Songbird, Toadstool, NestGate, petalTongue)
- 100% Unix socket communication
- BearDog security integration

### **🔍 What Needs Cleanup**
- 30 root MD files (many session-specific)
- Deprecated `from_endpoint()` methods (49 occurrences)
- Old HTTP fallback code
- Outdated session documentation
- Deprecated implementations

---

## 🗑️ **Cleanup Plan**

### **Phase 1: Root Documentation (30 → ~12 files)**

#### **Keep (Core Documentation)**
1. ✅ `README.md` - Main entry point
2. ✅ `STATUS.md` - Current status
3. ✅ `ROOT_DOCS_INDEX.md` - Documentation index
4. ✅ `BIOMEOS_ATOMICS_ARCHITECTURE.md` - Atomic architecture
5. ✅ `BIOMEOS_VS_PRIMAL_RESPONSIBILITIES.md` - Responsibilities
6. ✅ `LIVESPORE_ROADMAP.md` - LiveSpore planning
7. ✅ `NUCLEUS_COMPLETE_JAN14.md` - Latest NUCLEUS docs
8. ✅ `ALL_PRIMALS_HARVESTED_JAN14.md` - Harvest report
9. ✅ `QUICK_START_TOWER_DEPLOYMENT.md` - Quick start
10. ✅ `TRUE_PRIMAL_PORT_FREE_ARCHITECTURE.md` - Architecture principles

#### **Archive (Session-Specific, Jan 13-14)**
Move to `archive/sessions-jan14-2026/`:
- `ARCHITECTURE_UPDATE_JAN13_TOADSTOOL_BARRACUDA.md`
- `ATOMIC_DEPLOYMENT_FOR_PETALTONGUE.md`
- `GENETIC_LINEAGE_DEPLOYMENT_DEMO.md`
- `NESTGATE_ATOMIC_HANDOFF.md`
- `NUCLEUS_DEPLOYMENT_SUCCESS_JAN14.md` (superseded by NUCLEUS_COMPLETE)
- `NUCLEUS_LIVESPORE_DEPLOYMENT_PLAN.md`
- `PETALTONGUE_INTEGRATION_JAN13.md`
- `PETALTONGUE_NUCLEUS_DEPLOYMENT_JAN13.md`
- `PETALTONGUE_READY_TO_USE_JAN13.md`
- `PETALTONGUE_TUI_INTEGRATION.md`
- `PRIMAL_HARVEST_EXECUTION_PLAN.md`
- `PRIMAL_LAUNCHER_README.md`
- `ROOT_DOCS_ORGANIZATION_JAN14_FINAL.md`
- `SCIENTIFIC_VALIDATION_STATUS_JAN13.md`
- `SQUIRREL_HARVEST_REPORT_JAN14.md`
- `START_HERE_JAN13_FINAL.md`
- `START_HERE.md` (superseded by README)
- `TOADSTOOL_NEXT_TASK_INFRASTRUCTURE_COMPOSITION.md`
- `TRUE_PRIMAL_DEPLOYMENT_SUCCESS_JAN13.md`
- `TRUE_PRIMAL_FINAL_ASSESSMENT.md`

---

### **Phase 2: Code Cleanup**

#### **Remove Deprecated Client Methods**
Files with `from_endpoint()` to clean:
- `crates/biomeos-core/src/clients/beardog/client.rs` (2)
- `crates/biomeos-core/src/clients/squirrel.rs` (3)
- `crates/biomeos-core/src/clients/nestgate.rs` (2)
- `crates/biomeos-core/src/clients/songbird.rs` (5)
- `crates/biomeos-core/src/clients/toadstool.rs` (3)
- `crates/biomeos-core/src/clients/transport/http.rs` (3)
- `crates/biomeos-core/src/clients/openapi_adapter.rs` (8)
- `crates/biomeos-core/src/clients/universal.rs` (7)
- `crates/biomeos-core/src/clients/upa.rs` (11)
- `crates/biomeos-core/src/clients/base.rs` (4)

**Total**: 49 deprecated methods

**Action**: Remove all `from_endpoint()` and `new()` deprecated methods.  
**Reason**: All clients now use `discover()` with Unix sockets.

#### **Clean Up HTTP Fallback**
The HTTP transport is deprecated but kept for compatibility. Verify it's only used as fallback, not primary.

---

### **Phase 3: Neural API Review**

#### **Current Usage**
✅ **Working**:
- `crates/biomeos-atomic-deploy/src/neural_executor.rs` - Graph execution
- `crates/biomeos-atomic-deploy/src/neural_graph.rs` - Graph definition
- `graphs/*.toml` - Deployment graphs
- `nucleus` binary - Orchestrator

#### **Issues Found**
From grep results:
- `neural_executor.rs`: 1 TODO/FIXME
- `health_check.rs`: 2 TODOs
- `deployment_graph.rs`: 2 TODOs

**Action**: Review and address TODOs in neural API code.

---

### **Phase 4: LiveSpore Review**

#### **Current State**
- `crates/biomeos-spore/` - Spore creation library
- `base-spore/` - Base spore template
- `LIVESPORE_ROADMAP.md` - Planning doc
- `plasmidBin/` - Binary staging area

#### **Gaps**
- ⏳ No automated LiveSpore USB creator script
- ⏳ No testing for LiveSpore boot
- ⏳ No genetic lineage integration for LiveSpore

**Action**: Document LiveSpore status and create implementation plan.

---

### **Phase 5: Remove Old Code Patterns**

#### **Hardcoded Ports/Endpoints**
Already cleaned in previous sessions, but verify:
```bash
grep -r "localhost:[0-9]" crates/biomeos-core/src/clients/
grep -r "127.0.0.1:" crates/biomeos-core/src/clients/
```

#### **Old HTTP-Only Clients**
Files to verify don't have HTTP-only code:
- All client files should use `TransportClient`
- No direct HTTP client instantiation
- All use `discover()` pattern

---

## 📋 **Execution Checklist**

### **Documentation Cleanup**
- [ ] Move 19 session docs to archive
- [ ] Update README with latest NUCLEUS status
- [ ] Update STATUS with deployment metrics
- [ ] Keep only 10-12 core docs in root

### **Code Cleanup**
- [ ] Remove 49 deprecated `from_endpoint()` methods
- [ ] Verify no hardcoded localhost/ports in production code
- [ ] Address 5 TODOs in atomic-deploy
- [ ] Remove any unused HTTP-only client code

### **Neural API Review**
- [ ] Document current neural API usage
- [ ] Review graph execution patterns
- [ ] Identify missing features for full NUCLEUS
- [ ] Plan neural API enhancements

### **LiveSpore Review**
- [ ] Document current spore capabilities
- [ ] Identify gaps for USB deployment
- [ ] Plan genetic lineage integration
- [ ] Create LiveSpore creation script

---

## 🎯 **Priority Order**

### **High Priority** (Do Now)
1. **Archive session docs** - Reduce root clutter (30 → 12 files)
2. **Remove deprecated methods** - Clean code, reduce confusion
3. **Update core docs** - Reflect current NUCLEUS deployment

### **Medium Priority** (Before Squirrel)
4. **Address neural API TODOs** - Ensure orchestration is robust
5. **Verify port-free architecture** - Confirm no hardcoding remains

### **Low Priority** (After Squirrel)
6. **LiveSpore automation** - Create USB deployment tools
7. **Advanced features** - Genetic lineage, multi-node, etc.

---

## 🚀 **Expected Results**

### **After Cleanup**
- ✅ **10-12 root docs** (down from 30)
- ✅ **Zero deprecated methods** (removed 49)
- ✅ **5 TODOs addressed** (neural API)
- ✅ **Updated documentation** (reflects NUCLEUS)
- ✅ **Clean codebase** (ready for Squirrel)

### **Quality Improvements**
- Easier navigation (fewer files)
- Clearer codebase (no deprecated code)
- Better documentation (current, not historical)
- Robust neural API (TODOs resolved)

---

## 📝 **Notes**

### **Archive Strategy**
- Keep session docs as "fossil record" in archive
- Don't delete, just move to dated folders
- Makes them searchable but out of the way

### **Code Strategy**
- Aggressive removal of deprecated code
- Keep only active, working implementations
- Document any breaking changes

### **Documentation Strategy**
- Core docs in root (10-12 files)
- Session docs in archive (by date)
- Specs in `specs/` (detailed technical)
- Guides in `docs/guides/` (how-to)

---

**Created**: January 14, 2026 19:10 UTC  
**Status**: ✅ Ready for cleanup execution  
**Next**: Archive session docs, remove deprecated code

