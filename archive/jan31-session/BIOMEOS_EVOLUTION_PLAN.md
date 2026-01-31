# biomeOS Evolution Plan - Phase 2.5
**Date**: January 31, 2026  
**Phase**: biomeOS Orchestrator Evolution  
**Duration**: Estimated 10-15 hours

---

## 🎯 Mission

Evolve biomeOS from reference implementation to production-grade orchestrator that coordinates sovereign primals according to deep debt principles:

- ✅ Pure Rust (zero unsafe code in biomeOS)
- ✅ Platform agnostic (runtime discovery, zero hardcoding)
- ✅ Smart refactored (domain-driven, testable modules)
- ✅ Proper boundaries (uses primal APIs, doesn't reimplement)
- ✅ Complete implementations (no mocks in production)

---

## 🔍 Current State Assessment

### **Compilation Status**: ❌ **BROKEN**

**Critical Issue**:
```
crates/biomeos-test-utils/src/mock_primal.rs:239:22
error: use of unresolved module or unlinked crate `reqwest`
```

**Root Cause**: Test utils still using deprecated `reqwest` (removed in TRUE ecoBin v2.0)

### **Codebase Metrics**:
- **Rust files**: 395 files
- **Workspace crates**: 21 crates
- **TODOs**: Unknown (ripgrep not installed, need to search manually)
- **Test status**: Cannot run (compilation fails)
- **Coverage**: Unknown (blocked by compilation)

### **Known Issues**:
1. ❌ `biomeos-test-utils` uses deprecated `reqwest`
2. ❌ Test compilation broken
3. ⚠️ `genome-deploy` has unused imports
4. ❓ Unknown TODO count
5. ❓ Unknown unsafe code count

---

## 🎯 Evolution Priorities

### **Priority 0: Fix Compilation** 🔥 (1-2 hours)
**Impact**: Blocks everything  
**Status**: CRITICAL

**Tasks**:
- [ ] Fix `biomeos-test-utils` reqwest usage
- [ ] Update mocks to use Songbird/BearDog patterns
- [ ] Clean unused imports in `genome-deploy`
- [ ] Verify all crates compile
- [ ] Run test suite baseline

---

### **Priority 1: Inventory & Assessment** 📊 (2-3 hours)
**Impact**: Understand scope  
**Status**: REQUIRED

**Tasks**:
- [ ] Install ripgrep for searching
- [ ] Count all TODOs/FIXMEs/HACKs
- [ ] Find unsafe code blocks
- [ ] Analyze test coverage
- [ ] Document current architecture
- [ ] Identify deprecated patterns

---

### **Priority 2: Core Evolution** 🦀 (5-8 hours)
**Impact**: Production readiness  
**Status**: SYSTEMATIC

**Categories**:

#### **A. Primal SDK Enhancement**
Location: `crates/biomeos-primal-sdk/`

**Current**: Basic primal interface  
**Target**: Complete SDK for primal developers

Tasks:
- [ ] Standard discovery patterns
- [ ] BTSP communication helpers
- [ ] Service registration templates
- [ ] Health check utilities
- [ ] Error handling patterns
- [ ] Example implementations

#### **B. Graph Execution Hardening**
Location: `crates/biomeos-graph/`

**Current**: Basic graph parsing and execution  
**Target**: Production-grade neural graph orchestration

Tasks:
- [ ] Robust error recovery
- [ ] Partial execution support
- [ ] Graph validation
- [ ] Execution tracing
- [ ] Performance metrics
- [ ] Retry strategies

#### **C. Federation Coordination**
Location: `crates/biomeos-federation/`

**Current**: Single-family federation  
**Target**: Multi-family cross-platform coordination

Tasks:
- [ ] Cross-family discovery
- [ ] Trust boundary enforcement
- [ ] Federation health monitoring
- [ ] Conflict resolution
- [ ] Load balancing
- [ ] Failover strategies

#### **D. Deployment Orchestration**
Location: `crates/biomeos-deploy/`, `crates/biomeos-atomic-deploy/`

**Current**: Manual deployment scripts  
**Target**: Automated atomic composition deployment

Tasks:
- [ ] TOWER deployment automation
- [ ] NODE deployment automation
- [ ] NEST deployment automation
- [ ] Health validation post-deploy
- [ ] Rollback on failure
- [ ] Platform-specific adaptations

#### **E. UI/CLI Enhancement**
Location: `crates/biomeos-ui/`, `crates/biomeos-cli/`

**Current**: Basic CLI  
**Target**: Comprehensive user experience

Tasks:
- [ ] Interactive dashboard
- [ ] Real-time health monitoring
- [ ] Graph submission interface
- [ ] Federation visualization
- [ ] Deployment wizard
- [ ] Troubleshooting tools

---

### **Priority 3: Testing & Quality** ✅ (2-3 hours)
**Impact**: Confidence  
**Status**: ESSENTIAL

**Tasks**:
- [ ] Fix broken tests
- [ ] Add integration tests
- [ ] Cross-primal coordination tests
- [ ] Graph execution tests
- [ ] Federation tests
- [ ] Deployment tests
- [ ] Achieve >80% coverage

---

### **Priority 4: Documentation** 📚 (1-2 hours)
**Impact**: Adoption  
**Status**: IMPORTANT

**Tasks**:
- [ ] API documentation (rustdoc)
- [ ] Architecture guide
- [ ] Development guide
- [ ] Integration examples
- [ ] Troubleshooting guide
- [ ] Release notes

---

## 🎯 Execution Strategy

### **Phase 1: Foundation** (Hours 1-3)
```
1. Fix compilation (P0)
2. Run inventory (P1)
3. Document current state
4. Create detailed task breakdown
```

### **Phase 2: Core Work** (Hours 4-11)
```
5. Primal SDK enhancement
6. Graph execution hardening
7. Federation coordination
8. Deployment orchestration
9. UI/CLI enhancement
```

### **Phase 3: Quality** (Hours 12-14)
```
10. Test suite completion
11. Coverage verification
12. Integration validation
13. Performance testing
```

### **Phase 4: Documentation** (Hours 14-15)
```
14. API docs
15. Guides and examples
16. Release preparation
```

---

## 🎯 Success Criteria

### **Technical Excellence**:
- ✅ All crates compile without warnings
- ✅ All tests pass
- ✅ >80% test coverage
- ✅ Zero unsafe code in biomeOS
- ✅ Zero deprecated patterns
- ✅ Zero compilation warnings

### **Architectural Clarity**:
- ✅ Clear primal boundaries
- ✅ Uses primal APIs, doesn't reimplement
- ✅ Coordinates, doesn't control
- ✅ Platform agnostic (no hardcoding)
- ✅ Runtime discovery
- ✅ Graceful degradation

### **Production Readiness**:
- ✅ Robust error handling
- ✅ Comprehensive logging
- ✅ Health monitoring
- ✅ Automated deployment
- ✅ Rollback support
- ✅ Performance metrics

### **Developer Experience**:
- ✅ Complete SDK for primal developers
- ✅ Clear examples
- ✅ Comprehensive docs
- ✅ Easy integration
- ✅ Good error messages

---

## 🚀 Immediate Action: Priority 0

**Next Step**: Fix `biomeos-test-utils` compilation

**File**: `crates/biomeos-test-utils/src/mock_primal.rs`  
**Issue**: Lines 239, 266 use deprecated `reqwest::Client`  
**Fix**: Replace with Songbird HTTP gateway or remove if not needed

**Estimated Time**: 30 minutes  
**Impact**: Unblocks entire workspace compilation

---

## 📊 Tracking

### **Crates to Evolve** (21 total):

**Core** (5):
- [ ] biomeos-core
- [ ] biomeos-types
- [ ] biomeos
- [ ] biomeos-nucleus
- [ ] biomeos-test-utils 🔥

**Graph & API** (2):
- [ ] biomeos-graph
- [ ] biomeos-api

**Federation** (1):
- [ ] biomeos-federation

**Deployment** (5):
- [ ] biomeos-deploy
- [ ] biomeos-atomic-deploy
- [ ] biomeos-boot
- [ ] biomeos-spore
- [ ] genome-deploy

**UI** (2):
- [ ] biomeos-ui
- [ ] biomeos-cli

**System** (3):
- [ ] biomeos-system
- [ ] biomeos-primal-sdk
- [ ] biomeos-manifest

**Advanced** (3):
- [ ] biomeos-chimera
- [ ] biomeos-niche
- [ ] biomeos-compute

---

## 🎯 Estimated Effort

**Total TODOs**: ~58 (estimated from handoff)  
**Critical**: 2 (compilation fixes)  
**High**: ~15 (core functionality)  
**Medium**: ~25 (enhancements)  
**Low**: ~16 (polish)

**Total Effort**: 10-15 hours  
**Session Goal**: Complete P0 + P1, start P2

---

**Status**: Ready to begin P0 (Fix Compilation) 🚀
