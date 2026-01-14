# 🎯 Deep Debt Evolution Plan - January 14, 2026

**Status**: In Progress  
**Goal**: Evolve to modern idiomatic Rust, eliminate mocks, refactor large files

---

## 📊 Audit Results

### **1. Production Mocks Found** (HIGH PRIORITY)

**BearDog Client Stubs** (3 files - PRODUCTION CODE!):
```
crates/biomeos-core/src/clients/beardog/crypto.rs    (53 lines) - STUB
crates/biomeos-core/src/clients/beardog/access.rs    (51 lines) - STUB
crates/biomeos-core/src/clients/beardog/keys.rs      (44 lines) - STUB
```

**Status**: ⚠️ These are labeled as stubs for "BTSP Wave 2B" but are in PRODUCTION code!

**Action**: Evolve to real implementations using BearDog JSON-RPC API

---

### **2. Large Files Needing Smart Refactoring** (>800 lines)

**Files** (soft limit 800, hard limit 1000):
```
crates/biomeos-ui/src/petaltongue_bridge.rs          (964 lines) - OVER LIMIT
crates/biomeos-cli/src/tui/widgets.rs                 (904 lines) - OVER LIMIT
crates/biomeos-core/src/clients/toadstool.rs          (901 lines) - OVER LIMIT
crates/biomeos-ui/src/orchestrator.rs                 (847 lines) - WARNING
```

**Strategy**: Smart refactoring (not just splitting):
- Extract cohesive modules
- Separate concerns (UI, logic, types)
- Create trait-based abstractions
- Improve testability

---

### **3. External Dependencies Analysis**

**Strategy**: Analyze and evolve to Rust where possible:
- Shell scripts → Rust binaries
- HTTP clients → tarpc or Unix sockets
- External tools → Rust implementations

**Next**: Run full dependency analysis

---

### **4. tarpc Transport** (8-12h task)

**Current**: Only Unix socket transport implemented  
**Goal**: Add type-safe tarpc for inter-primal communication

**Benefits**:
- Type-safe RPC (compile-time checks)
- Bidirectional streaming
- Better error handling
- Performance comparable to Unix sockets

---

## 🎯 Execution Order

### **Phase 1: Evolve Production Mocks** (2-4h)
1. ✅ Audit for mocks in production
2. 🔄 Evolve BearDog crypto client
3. 🔄 Evolve BearDog access client
4. 🔄 Evolve BearDog keys client

### **Phase 2: Smart Refactor Large Files** (4-6h)
1. 🔄 Analyze petaltongue_bridge.rs (964 lines)
2. 🔄 Refactor into cohesive modules
3. 🔄 Apply to other large files

### **Phase 3: External Dependencies** (2-3h)
1. 🔄 Audit external dependencies
2. 🔄 Identify candidates for Rust evolution
3. 🔄 Plan migration strategy

### **Phase 4: tarpc Transport** (8-12h)
1. ⏳ Design tarpc integration
2. ⏳ Implement tarpc transport
3. ⏳ Migrate clients
4. ⏳ Test and validate

---

## 🚀 Starting with Phase 1: Production Mocks

**Priority**: HIGHEST - These are labeled as stubs but are in production code!

**Goal**: Evolve to real implementations that call BearDog's JSON-RPC API

---

**Created**: January 14, 2026  
**Status**: In Progress  
**Next**: Evolve BearDog stub clients to real implementations

