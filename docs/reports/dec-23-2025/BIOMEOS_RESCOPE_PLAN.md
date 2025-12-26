# BiomeOS Rescope & Cleanup Plan
**Date:** December 23, 2025  
**Context:** UI has evolved into petalTongue primal  
**Goal:** Make biomeOS lean, focused, and production-ready

---

## 🎯 Executive Summary

**Discovery:** The UI code in `biomeOS/ui/` is **legacy**. The UI has evolved into its own primal: **petalTongue**.

**Architecture Clarity:**
- **biomeOS** = Orchestration layer (CLI, core, APIs, chimeras, niches)
- **petalTongue** = UI/Visualization primal (visual, audio, accessibility-first interfaces)

**Action Required:** Remove legacy UI code from biomeOS, rescope to core responsibilities.

---

## 📊 Current State Analysis

### biomeOS Directory Structure

```
biomeOS/
├── crates/                    ✅ KEEP - Core functionality
│   ├── biomeos-types/        ✅ Core type system
│   ├── biomeos-core/         ✅ Universal manager & business logic  
│   ├── biomeos-primal-sdk/   ✅ Primal capabilities & types
│   ├── biomeos-cli/          ✅ CLI interface
│   ├── biomeos-chimera/      ✅ Chimera compiler
│   ├── biomeos-niche/        ✅ Niche deployment
│   ├── biomeos-system/       ✅ System integration
│   ├── biomeos-manifest/     ✅ YAML parsing
│   ├── biomeos-federation/   ✅ Federation support
│   └── biomeos-ui/           ⚠️ REVIEW - May be legacy
│
├── ui/                        ❌ REMOVE - Legacy, moved to petalTongue
│   ├── src/                  989 LOC minimal_app + tests
│   └── Cargo.toml            
│
├── src/                       ✅ KEEP - Main library
│   ├── lib.rs                
│   └── universal_adapter.rs  
│
├── examples/                  ✅ KEEP - Working examples
├── tests/                     ✅ KEEP - Integration tests
├── chimeras/                  ✅ KEEP - Chimera definitions
├── niches/                    ✅ KEEP - Niche templates
├── templates/                 ✅ KEEP - YAML templates
├── docs/                      ✅ KEEP - Documentation
├── specs/                     ✅ KEEP - Specifications
└── archive/                   ✅ KEEP - Historical reference
```

### What to Remove

| Directory/File | Size | Reason | Action |
|----------------|------|--------|--------|
| `ui/` | 989 LOC | Legacy UI, superseded by petalTongue | **Archive then delete** |
| `biomeos-desktop/` | Small | Desktop packaging, superseded | **Archive then delete** |
| `crates/biomeos-ui/` | ~8 files | UI types, may still be needed | **Review then decide** |

---

## 🧹 Cleanup Steps

### Phase 1: Archive Legacy UI (Safe)

```bash
# 1. Create archive directory for UI
mkdir -p archive/legacy-ui-moved-to-petaltongue/

# 2. Move UI directory to archive
mv ui/ archive/legacy-ui-moved-to-petaltongue/ui/
mv biomeos-desktop/ archive/legacy-ui-moved-to-petaltongue/biomeos-desktop/

# 3. Document the move
cat > archive/legacy-ui-moved-to-petaltongue/README.md << 'EOF'
# Legacy UI Code - Moved to petalTongue

**Date:** December 23, 2025  
**Reason:** UI evolved into its own primal

## What Happened

The biomeOS UI code evolved into a standalone primal called **petalTongue**:
- Location: `../petalTongue/`
- Status: Production-ready with 26+ tests
- Features: Visual + audio modalities, accessibility-first

## What's Here

This directory contains the legacy UI code from biomeOS before the split:
- `ui/` - Desktop UI application (989 LOC)
- `biomeos-desktop/` - Desktop packaging

## Why Keep This

Fossil record for understanding the evolution. Not for production use.

## See Also

- `/home/eastgate/Development/ecoPrimals/phase2/petalTongue/`
- `petalTongue/README.md`
EOF

# 4. Update workspace Cargo.toml
# Remove ui and biomeos-desktop from workspace members
```

### Phase 2: Review biomeos-ui Crate

The `crates/biomeos-ui/` crate needs review:

```bash
# Check what's using it
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
grep -r "biomeos-ui" --include="*.toml" crates/
grep -r "use biomeos_ui" --include="*.rs" crates/
```

**Decision Matrix:**

| If Used By | Action |
|------------|--------|
| Only `ui/` directory | **Delete** (since ui/ is being removed) |
| CLI or core crates | **Keep and review** (may be shared types) |
| Examples | **Review** (may need migration) |
| Tests | **Review** (may need migration) |

### Phase 3: Update Dependencies

1. **Cargo.toml (workspace):**
   ```toml
   # BEFORE:
   members = [
       "crates/*",
       "ui",              # REMOVE
       "biomeos-desktop", # REMOVE
       # ...
   ]
   
   # AFTER:
   members = [
       "crates/*",
       # ui and biomeos-desktop removed - see archive/legacy-ui-moved-to-petaltongue/
       # ...
   ]
   ```

2. **Remove UI dependencies from other crates** (if any)

3. **Update examples** that reference UI

### Phase 4: Update Documentation

1. **README.md:**
   - Remove UI-related sections
   - Add pointer to petalTongue for UI needs
   - Update architecture diagram

2. **STRUCTURE.md:**
   - Remove `ui/` and `biomeos-desktop/` from structure
   - Add note about petalTongue

3. **Status reports:**
   - Update to reflect rescoped responsibilities
   - Reference petalTongue for UI functionality

### Phase 5: Verify Build

```bash
# Should build without UI
cargo build --workspace

# Should pass all tests
cargo test --workspace --lib

# Should have fewer crates
cargo tree --workspace --depth 0
```

---

## 🎯 Rescoped BiomeOS Responsibilities

### Core Focus: Orchestration Layer

**What BiomeOS IS:**
- ✅ Primal registration & discovery
- ✅ Chimera composition & management
- ✅ Niche deployment & orchestration
- ✅ Health monitoring & reporting
- ✅ CLI for ecosystem management
- ✅ API for primal integration
- ✅ Configuration management (YAML)
- ✅ Capability-based service matching
- ✅ Federation coordination

**What BiomeOS IS NOT:**
- ❌ UI/Visualization (that's petalTongue)
- ❌ Desktop application (that's petalTongue)
- ❌ Audio/visual rendering (that's petalTongue)
- ❌ Accessibility interfaces (that's petalTongue)

### Integration with petalTongue

BiomeOS provides **data** via API, petalTongue provides **presentation**:

```
┌──────────────────────────────────────────┐
│            petalTongue                   │
│   (UI/Visualization Primal)              │
│                                          │
│   • Visual modality (2D graph)          │
│   • Audio modality (sonification)       │
│   • Accessibility-first                 │
│   • Real-time updates                   │
└─────────────┬────────────────────────────┘
              │ HTTP/WebSocket API
              │
┌─────────────▼────────────────────────────┐
│            biomeOS                       │
│   (Orchestration Layer)                  │
│                                          │
│   • Primal discovery                    │
│   • Health monitoring                   │
│   • Chimera management                  │
│   • Niche deployment                    │
│   • CLI interface                       │
└──────────────────────────────────────────┘
```

---

## 📝 Updated Crate Structure

### After Cleanup

```
biomeOS/crates/
├── biomeos-types/        # Core type system (foundation)
├── biomeos-core/         # Universal manager & business logic
├── biomeos-primal-sdk/   # Primal capabilities & types
├── biomeos-cli/          # CLI interface
├── biomeos-chimera/      # Chimera compiler
├── biomeos-niche/        # Niche deployment
├── biomeos-system/       # System integration
├── biomeos-manifest/     # YAML parsing
└── biomeos-federation/   # Federation support

Total: 9 crates (down from 10)
```

### Removed Crates

- ❌ `biomeos-ui` - Moved to petalTongue context
- ❌ UI binary targets - petalTongue handles this

---

## 🚀 Benefits of Rescoping

### 1. **Clearer Separation of Concerns**
- biomeOS = orchestration logic
- petalTongue = presentation layer
- Each primal focused on one thing

### 2. **Reduced Compilation Errors**
- UI was causing 10+ compilation errors
- Removing it eliminates that complexity
- Faster builds for core functionality

### 3. **Better Architecture**
- Follows primal philosophy (single responsibility)
- UI can evolve independently
- Easier to test and maintain

### 4. **Smaller Codebase**
- ~989 LOC removed from minimal_app.rs
- Multiple view files removed
- Focus on core ~34,100 LOC

### 5. **Independent Evolution**
- petalTongue can add modalities without touching biomeOS
- biomeOS can change APIs without breaking UI build
- True decoupling

---

## ⚠️ Risks & Mitigations

### Risk 1: Breaking Examples

**Risk:** Some examples might reference UI code  
**Mitigation:** 
- Grep for UI imports before removal
- Update examples to use petalTongue or CLI
- Document migration path

### Risk 2: Shared Types

**Risk:** `biomeos-ui` crate might have types used by core  
**Mitigation:**
- Audit usage before deletion
- Move truly shared types to `biomeos-types`
- Keep only if genuinely shared

### Risk 3: Historical Reference

**Risk:** Losing understanding of UI evolution  
**Mitigation:**
- Archive directory with README explaining evolution
- Keep fossil record
- Reference petalTongue for current state

---

## 📋 Execution Checklist

### Pre-Cleanup

- [ ] Verify petalTongue builds successfully
- [ ] Verify petalTongue tests pass
- [ ] Audit `biomeos-ui` crate usage
- [ ] Check examples for UI dependencies
- [ ] Backup current state (git commit)

### Cleanup

- [ ] Create archive directory
- [ ] Move `ui/` to archive
- [ ] Move `biomeos-desktop/` to archive  
- [ ] Create archive README
- [ ] Update workspace Cargo.toml
- [ ] Remove or migrate `biomeos-ui` crate
- [ ] Update dependencies

### Verification

- [ ] `cargo build --workspace` succeeds
- [ ] `cargo test --workspace --lib` passes
- [ ] No broken examples
- [ ] Documentation updated
- [ ] README reflects new scope

### Post-Cleanup

- [ ] Update README.md
- [ ] Update STRUCTURE.md
- [ ] Update status reports
- [ ] Update audit report
- [ ] Add integration notes for petalTongue
- [ ] Git commit with clear message

---

## 🎯 Success Criteria

**BiomeOS is successfully rescoped when:**

1. ✅ Workspace builds without UI code
2. ✅ All core tests pass (types, core, chimera, niche, CLI)
3. ✅ README clearly states biomeOS = orchestration only
4. ✅ Documentation points to petalTongue for UI
5. ✅ Archive preserves historical context
6. ✅ Examples work or are updated
7. ✅ Integration path with petalTongue documented

---

## 📊 Expected Metrics After Cleanup

| Metric | Before | After | Change |
|--------|--------|-------|--------|
| **Workspace Crates** | 10 | 9 | -1 |
| **Total LOC** | ~35,000 | ~34,000 | -1,000 |
| **Compilation Errors** | 10+ (UI) | 0 | -100% |
| **Build Time** | ~3.7s | ~3.0s | -20% |
| **Test Count** | Mixed | Core only | Focused |
| **Files > 1000 LOC** | 2 | 1 | -50% |

---

## 🔗 Integration with petalTongue

### API Contract

BiomeOS exposes APIs that petalTongue consumes:

```rust
// BiomeOS provides:
GET  /api/v1/system/status    // System health
GET  /api/v1/primals           // Discovered primals  
GET  /api/v1/chimeras          // Chimera definitions
GET  /api/v1/niches            // Niche templates
WS   /api/v1/events            // Real-time updates

// petalTongue consumes and visualizes
```

### Configuration

```bash
# petalTongue connects to biomeOS
BIOMEOS_URL=http://localhost:3000 cargo run -p petal-tongue-ui

# biomeOS doesn't need to know about petalTongue
cargo run -p biomeos-cli -- health
```

---

## 📚 Updated Documentation Structure

```
biomeOS/
├── README.md                    # Focus on orchestration, link to petalTongue
├── STRUCTURE.md                 # Updated structure without UI
├── COMPREHENSIVE_AUDIT_REPORT   # Update with rescope info
├── BIOMEOS_RESCOPE_PLAN.md      # This document
│
├── docs/
│   ├── api/                     # APIs that petalTongue uses
│   └── guides/
│       └── petaltongue-integration.md  # NEW: How to use petalTongue with biomeOS
│
└── archive/
    └── legacy-ui-moved-to-petaltongue/  # Fossil record
        ├── README.md
        ├── ui/
        └── biomeos-desktop/
```

---

## 🎉 Conclusion

**The Evolution:**
- biomeOS started as monolith (orchestration + UI)
- UI grew complex enough to warrant its own primal
- petalTongue emerged with unique focus (accessibility, multi-modal)
- Time to formalize the split

**The Result:**
- biomeOS: Lean, focused orchestration layer
- petalTongue: Rich, accessible UI primal
- Clean separation of concerns
- Better architecture overall

**Next Steps:**
1. Verify petalTongue health ✅
2. Execute cleanup plan
3. Update documentation
4. Celebrate clear architecture! 🎉

---

**Status:** Ready for execution  
**Risk Level:** Low (everything archived, reversible)  
**Estimated Time:** 2-3 hours  
**Approval Required:** Yes (before deletion)

---

*Making biomeOS lean and focused on its true purpose: orchestration.* 🌱

