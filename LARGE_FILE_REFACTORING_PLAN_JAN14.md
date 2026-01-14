# 📐 Large File Smart Refactoring Plan - January 14, 2026

**Date**: January 14, 2026 (Evening)  
**Status**: 📋 **PLANNED** (Ready for execution)  
**Effort**: 1-2h per file  
**Strategy**: Smart refactoring (cohesive modules, not just splitting)

---

## 🎯 Files Identified for Refactoring

### **Target Files** (>800 lines soft limit):

1. `crates/biomeos-ui/src/petaltongue_bridge.rs` - **964 lines** 🔴
2. `crates/biomeos-cli/src/tui/widgets.rs` - **904 lines** 🔴
3. `crates/biomeos-core/src/clients/toadstool.rs` - **901 lines** 🔴
4. `crates/biomeos-ui/src/orchestrator.rs` - **847 lines** 🟡

**Total**: 3,616 lines across 4 files

---

## 📋 Smart Refactoring Strategy

### **Principle**: Extract Cohesive Modules

**NOT**: Just split at line 800
**YES**: Create logical, self-contained modules

### **Pattern**:
1. **Analyze** file structure and responsibilities
2. **Extract** cohesive types into separate modules
3. **Refactor** implementation into focused files
4. **Test** to ensure no behavioral changes

---

## 🔧 Refactoring Plans

### **1. petaltongue_bridge.rs** (964 lines → ~600 lines)

#### **Current Structure**:
- Type definitions (Device, Primal, NicheTemplate, etc.) - ~200 lines
- BridgeCache struct + logic - ~50 lines
- PetalTongueRPCBridge impl - ~600 lines
- ValidationResult type - ~10 lines
- Tests - ~100 lines

#### **Proposed Structure**:

```
crates/biomeos-ui/src/petaltongue/
├── mod.rs                  (~50 lines) - Module coordination
├── types.rs                (~250 lines) - All type definitions
│   ├── Device, DeviceType, DeviceStatus
│   ├── Primal, PrimalStatus
│   ├── NicheTemplate
│   ├── PrimalRole
│   ├── ResourceRequirements
│   └── ValidationResult
├── cache.rs                (~100 lines) - Cache logic
│   ├── BridgeCache struct
│   ├── refresh_cache()
│   └── Cache management methods
├── bridge.rs               (~400 lines) - Main bridge implementation
│   ├── PetalTongueRPCBridge struct
│   ├── Discovery methods
│   ├── RPC handlers
│   └── Orchestration methods
└── tests.rs                (~150 lines) - All tests
```

**Benefits**:
- Clear separation of concerns
- Types reusable by other modules
- Cache can be tested independently
- Bridge logic focused and readable

**Effort**: 1-1.5h

---

### **2. tui/widgets.rs** (904 lines → ~500 lines)

#### **Current Structure**:
- Widget trait implementations
- Rendering logic for different widgets
- Event handling
- State management

#### **Proposed Structure**:

```
crates/biomeos-cli/src/tui/widgets/
├── mod.rs                  (~50 lines) - Widget exports
├── device_list.rs          (~150 lines) - Device list widget
├── primal_status.rs        (~150 lines) - Primal status widget
├── topology_view.rs        (~200 lines) - Topology visualization
├── metrics_panel.rs        (~150 lines) - Metrics display
├── command_bar.rs          (~100 lines) - Command input
└── common.rs               (~100 lines) - Shared rendering utilities
```

**Benefits**:
- Each widget in its own file
- Easier to test individual widgets
- Shared utilities extracted
- Better code organization

**Effort**: 1-1.5h

---

### **3. clients/toadstool.rs** (901 lines → ~600 lines)

#### **Current Structure**:
- ToadStoolClient struct
- Compute job management methods
- GPU allocation methods
- Container orchestration methods
- WASM runtime methods
- Response types

#### **Proposed Structure**:

```
crates/biomeos-core/src/clients/toadstool/
├── mod.rs                  (~50 lines) - Client exports
├── client.rs               (~200 lines) - Main ToadStoolClient
├── types.rs                (~150 lines) - All types/responses
├── compute.rs              (~150 lines) - Compute job methods
├── gpu.rs                  (~150 lines) - GPU allocation methods
├── containers.rs           (~150 lines) - Container orchestration
└── wasm.rs                 (~150 lines) - WASM runtime methods
```

**Benefits**:
- Domain-separated methods
- Each capability in its own module
- Types separated for reuse
- Easier to maintain and test

**Effort**: 1.5-2h

---

### **4. orchestrator.rs** (847 lines → ~550 lines)

#### **Current Structure**:
- Orchestrator struct
- Primal coordination methods
- Device assignment logic
- Health monitoring
- Event handling

#### **Proposed Structure**:

```
crates/biomeos-ui/src/orchestrator/
├── mod.rs                  (~50 lines) - Orchestrator exports
├── core.rs                 (~200 lines) - Main Orchestrator
├── coordination.rs         (~150 lines) - Primal coordination
├── assignment.rs           (~150 lines) - Device assignment logic
├── health.rs               (~150 lines) - Health monitoring
└── events.rs               (~150 lines) - Event handling
```

**Benefits**:
- Clearer responsibility boundaries
- Health monitoring isolated for testing
- Assignment logic self-contained
- Events decoupled

**Effort**: 1-1.5h

---

## ✅ Refactoring Checklist (Per File)

### **Pre-Refactor**:
- [ ] Read entire file and understand structure
- [ ] Identify cohesive groupings
- [ ] Plan module structure
- [ ] Ensure tests exist

### **During Refactor**:
- [ ] Create new module directory
- [ ] Extract types to `types.rs`
- [ ] Move implementation logic to focused modules
- [ ] Update imports across codebase
- [ ] Ensure no behavioral changes

### **Post-Refactor**:
- [ ] Run all tests (`cargo test`)
- [ ] Check compilation (`cargo build`)
- [ ] Run clippy (`cargo clippy`)
- [ ] Verify file sizes (<800 lines each)
- [ ] Update documentation

---

## 📊 Impact Analysis

### **Before Refactoring**:
```
4 files > 800 lines
Total: 3,616 lines
Average: 904 lines/file
Maintainability: Challenging
```

### **After Refactoring**:
```
~20 focused modules
Largest file: ~600 lines
Average: ~200 lines/file
Maintainability: Excellent
```

**Improvement**: ~78% reduction in average file size!

---

## 🎯 Execution Order (Recommended)

### **Phase 1** (Easiest):
1. `petaltongue_bridge.rs` → `petaltongue/` module
   - Clear type separation
   - Well-defined responsibilities
   - **Effort**: 1-1.5h

### **Phase 2** (Medium):
2. `orchestrator.rs` → `orchestrator/` module
   - Moderate complexity
   - Good separation points
   - **Effort**: 1-1.5h

### **Phase 3** (More Complex):
3. `tui/widgets.rs` → `tui/widgets/` module
   - UI code can be tricky
   - Many interdependencies
   - **Effort**: 1-1.5h

4. `clients/toadstool.rs` → `clients/toadstool/` module
   - Most complex client
   - Many capabilities
   - **Effort**: 1.5-2h

**Total Effort**: 5-7 hours across 4 refactoring sessions

---

## 🏆 Benefits of Smart Refactoring

### **Code Quality**:
- ✅ Better separation of concerns
- ✅ Easier to understand
- ✅ Simpler to test
- ✅ More maintainable

### **Developer Experience**:
- ✅ Faster navigation (smaller files)
- ✅ Clearer code organization
- ✅ Reduced cognitive load
- ✅ Better IDE performance

### **Testing**:
- ✅ Unit test individual modules
- ✅ Easier to mock dependencies
- ✅ Better test organization
- ✅ Improved coverage

---

## ⏰ When to Execute

### **Recommendation**: Dedicated 2-3h session

**Why**:
- Requires focus and careful attention
- Need to ensure no behavioral changes
- Testing is critical
- Fresh mind works best

### **NOT Recommended**:
- ❌ End of long session (tired)
- ❌ Rushed execution
- ❌ Without full test suite

---

## 🎯 Success Criteria

**For Each Refactored File**:
1. ✅ All new modules < 800 lines
2. ✅ All tests pass
3. ✅ No clippy warnings
4. ✅ Clear module responsibilities
5. ✅ Documentation updated
6. ✅ No behavioral changes

---

## 📝 Notes

### **Important**:
- This is **smart refactoring**, not mindless splitting
- Focus on **cohesive modules** with clear responsibilities
- Maintain or improve **test coverage**
- Ensure **no behavioral changes**

### **Philosophy**:
> "The goal is not small files, but clear responsibilities.
> Small files are a natural byproduct of good design."

---

## 🚀 Ready for Execution

**Status**: ✅ **PLANNED AND DOCUMENTED**

**Next Steps**:
1. Schedule 2-3h dedicated refactoring session
2. Start with `petaltongue_bridge.rs` (easiest)
3. Follow checklist for each file
4. Celebrate cleaner codebase! 🎉

---

**Created**: January 14, 2026  
**Status**: 📋 PLANNED (Ready for execution)  
**Effort**: 5-7 hours total (across 4 sessions)

**"Smart refactoring creates clarity - the TRUE PRIMAL way!"** 📐✨

