# 🔧 Wave 2B: Smart Refactoring beardog.rs

**Date**: January 10, 2026 (Late Night)  
**Current**: 1,062 lines (monolithic)  
**Target**: Modular semantic structure  
**Approach**: Domain-driven design, not arbitrary splits

---

## 📊 Current Structure Analysis

### **File Size**: 1,062 lines

### **Components Identified**:
1. **Client Core** (lines 1-105): Imports, struct, discovery
2. **Crypto Operations** (lines 106-366): encrypt, decrypt, sign, verify
3. **Key Management** (lines 367-402): generate_key
4. **Access Control** (lines 403-478): validate_access, audit_log
5. **BTSP Tunnels** (lines 479-766): establish, status, close
6. **BTSP Module** (lines 767-937): High-level tunnel API
7. **Types** (lines 573-766): Shared types (10 structs)
8. **Tests** (lines 938-1062): Unit tests

---

## 🎯 Smart Refactoring Plan

### **Principle**: Semantic modules based on domain concepts

```
beardog/
├── mod.rs              (~100 lines)  - Public API, re-exports
├── client.rs           (~150 lines)  - Client struct, discovery, core
├── crypto.rs           (~250 lines)  - Encryption, signing operations
├── keys.rs             (~100 lines)  - Key generation & management
├── access.rs           (~150 lines)  - Access control & audit
├── tunnels.rs          (~200 lines)  - Low-level tunnel operations
├── btsp.rs             (~200 lines)  - High-level BTSP API (from btsp mod)
└── types.rs            (~200 lines)  - All shared types

Total: ~1,350 lines (with proper spacing, docs)
```

### **Why This Structure?**

1. **crypto.rs**: Encryption & signing are ONE domain (cryptography)
2. **keys.rs**: Key lifecycle is separate from using keys
3. **access.rs**: Authorization & audit are ONE domain (access control)
4. **tunnels.rs**: Low-level tunnel protocol operations
5. **btsp.rs**: High-level user-facing tunnel API
6. **types.rs**: All shared types in one place (clear imports)

---

## 📋 Step-by-Step Execution Plan

### **Phase 1: Create Module Structure** (30 min)
1. ✅ Create `beardog/` directory
2. ✅ Create skeleton files with SPDX headers
3. ✅ Set up `mod.rs` with re-exports
4. ✅ Verify compilation (empty modules)

### **Phase 2: Extract Types** (30 min)
1. Move all `pub struct` to `types.rs`
2. Update imports in `beardog.rs`
3. Verify compilation
4. Commit: "refactor(beardog): Extract types to beardog/types.rs"

### **Phase 3: Extract Crypto** (45 min)
1. Move encrypt, decrypt, sign, verify to `crypto.rs`
2. Keep client reference via `&BearDogClient`
3. Update imports
4. Verify compilation & tests
5. Commit: "refactor(beardog): Extract crypto operations to beardog/crypto.rs"

### **Phase 4: Extract Keys** (30 min)
1. Move generate_key to `keys.rs`
2. Add future key management methods
3. Update imports
4. Verify compilation & tests
5. Commit: "refactor(beardog): Extract key management to beardog/keys.rs"

### **Phase 5: Extract Access** (30 min)
1. Move validate_access, get_audit_log to `access.rs`
2. Update imports
3. Verify compilation & tests
4. Commit: "refactor(beardog): Extract access control to beardog/access.rs"

### **Phase 6: Extract Tunnels** (45 min)
1. Move low-level tunnel methods to `tunnels.rs`
2. Move btsp module to `btsp.rs`
3. Update imports
4. Verify compilation & tests
5. Commit: "refactor(beardog): Extract tunnel operations to beardog/tunnels.rs and beardog/btsp.rs"

### **Phase 7: Finalize Client** (30 min)
1. Clean up `client.rs` (discovery, core only)
2. Update `mod.rs` documentation
3. Verify all tests pass
4. Commit: "refactor(beardog): Finalize modular structure"

### **Phase 8: Documentation** (30 min)
1. Add module-level documentation
2. Update examples
3. Add cross-references
4. Commit: "docs(beardog): Complete modular documentation"

**Total Estimated Time**: 4-5 hours

---

## 🎯 Success Criteria

### **Technical**:
- [ ] All tests pass (no regressions)
- [ ] Zero compilation errors
- [ ] Zero linter warnings
- [ ] Backward compatible (no API changes)
- [ ] Clear module boundaries

### **Quality**:
- [ ] Each module <300 lines
- [ ] Clear semantic separation
- [ ] Comprehensive documentation
- [ ] Easy navigation
- [ ] Testable in isolation

### **User Experience**:
- [ ] Same public API (no breaking changes)
- [ ] Better IDE navigation
- [ ] Clearer error messages
- [ ] Easier to contribute

---

## 🚀 Starting Phase 1: Module Structure

Let's begin!

---

**Document Version**: v1.0  
**Status**: Ready to execute  
**Next**: Phase 1 - Create module structure

