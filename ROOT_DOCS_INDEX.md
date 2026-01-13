# 📚 biomeOS Root Documentation Index

**Last Updated**: 2026-01-13

This index organizes all root-level documentation for easy navigation.

---

## 🎯 **Start Here** (New Contributors)

1. **START_HERE.md** - Onboarding guide for new developers
2. **README.md** - Project overview and quick start
3. **STATUS.md** - Current state, metrics, and next steps

---

## 🏗️ **Architecture & Design**

### Core Architecture
- **BIOMEOS_ATOMICS_ARCHITECTURE.md** - Atomic deployment patterns
- **BIOMEOS_VS_PRIMAL_RESPONSIBILITIES.md** - Boundary definitions

### Roadmaps & Plans
- **LIVESPORE_ROADMAP.md** - LiveSpore deployment strategy
- **GENETIC_LINEAGE_DEPLOYMENT_DEMO.md** - Family lineage patterns

---

## 🔧 **Technical Deep Dives**

### Code Quality & Evolution
- **DEEP_DEBT_INDEX_JAN13_2026.md** - Technical debt tracking
- **UNSAFE_CODE_EVOLUTION_JAN13_2026.md** - Unsafe code elimination (A+ achieved)
- **UNWRAP_ELIMINATION_STRATEGY_JAN13_2026.md** - Error handling improvements
- **TEST_COVERAGE_STRATEGY_JAN13_2026.md** - Path to 90% coverage

### Specific Evolutions
- **HARDCODING_EVOLUTION_PROGRESS.md** - Hardcoding removal progress
- **HARDCODING_EVOLUTION_MILESTONE3.md** - Milestone achievements
- **LARGE_FILE_REFACTORING_PLAN_JAN13_2026.md** - File size management
- **JSON_RPC_CLIENTS_STATUS_JAN13_2026.md** - Client implementation status

---

## 🧪 **Testing & Quality**

- **TEST_COVERAGE_STRATEGY_JAN13_2026.md** - Coverage roadmap
- **tests/helpers/sync.rs** - Concurrent test helpers (in code)
- **scripts/enable-concurrent-tests.sh** - Test automation (in code)

---

## 🔌 **Integration Guides**

### Primal Integration
- **NESTGATE_ATOMIC_HANDOFF.md** - NestGate atomic integration
- **PETALTONGUE_TUI_INTEGRATION.md** - petalTongue UI integration
- **PRIMAL_LAUNCHER_README.md** - Primal launching guide

### Assessment
- **TRUE_PRIMAL_FINAL_ASSESSMENT.md** - Architecture compliance

---

## 📁 **Archived Sessions**

### January 13, 2026 - Concurrent Evolution
**Location**: `archive/sessions-jan13-2026/`

**Key Documents**:
- `CONCURRENT_EVOLUTION_COMPLETE_JAN13.md` - Final summary (A+ 96/100)
- `COMPREHENSIVE_AUDIT_JAN13_2026_FINAL.md` - Full codebase audit
- `DEEP_DEBT_CONCURRENT_EVOLUTION_PLAN_JAN13.md` - Evolution strategy
- `CONCURRENT_TESTS_ENABLED_JAN13.md` - 326 tests converted
- `FINAL_SESSION_SUMMARY_JAN13_2026.md` - Complete session report

**Achievements**:
- ✅ 326 tests now concurrent (multi-thread)
- ✅ Concurrent test infrastructure complete
- ✅ Zero unsafe code maintained
- ✅ Clean workspace build

### January 12, 2026 - Deep Debt Sessions
**Location**: `archive/sessions-jan12-2026/`

---

## 📊 **Quick Reference**

### Current Priorities (from STATUS.md)
1. Re-enable client module (2-3h)
2. Reach 90% test coverage (12-15h)
3. Reduce unwrap/expect to <100 (6-8h)

### Quick Commands
```bash
# Build
cargo build --workspace

# Test (concurrent)
cargo test --workspace --lib -- --test-threads=8

# Coverage
cargo llvm-cov --workspace --html

# Quality checks
cargo clippy --workspace -- -D warnings
cargo fmt --check
```

---

## 🗂️ **Directory Structure**

```
biomeOS/
├── README.md                    # Project overview
├── START_HERE.md                # New contributor guide
├── STATUS.md                    # Current state
├── ROOT_DOCS_INDEX.md          # This file
│
├── docs/                        # Detailed documentation
│   ├── architecture/           # System design
│   ├── guides/                 # How-to guides
│   ├── api/                    # API documentation
│   ├── primal-integrations/    # Integration guides
│   └── collaborative-intelligence/
│
├── specs/                       # Technical specifications
│
├── archive/                     # Historical documents
│   ├── sessions-jan13-2026/    # Today's session
│   ├── sessions-jan12-2026/    # Previous sessions
│   └── docs-fossil-record/     # Old documentation
│
├── crates/                      # Rust crates
├── tests/                       # Integration tests
├── scripts/                     # Automation scripts
└── examples/                    # Usage examples
```

---

## 🎯 **Documentation by Role**

### For New Developers
1. START_HERE.md
2. README.md
3. docs/guides/

### For Architecture Review
1. BIOMEOS_ATOMICS_ARCHITECTURE.md
2. docs/architecture/
3. specs/

### For Quality/Testing
1. TEST_COVERAGE_STRATEGY_JAN13_2026.md
2. UNSAFE_CODE_EVOLUTION_JAN13_2026.md
3. archive/sessions-jan13-2026/COMPREHENSIVE_AUDIT_JAN13_2026_FINAL.md

### For Integration Work
1. NESTGATE_ATOMIC_HANDOFF.md
2. PETALTONGUE_TUI_INTEGRATION.md
3. docs/primal-integrations/

---

## 📈 **Evolution Tracking**

### Completed Evolutions
- ✅ **Unsafe Code Elimination** (Jan 12-13) - Grade: A++
- ✅ **Concurrent Test Evolution** (Jan 13) - Grade: A+ (96/100)
- ✅ **TRUE PRIMAL Architecture** (Jan 12) - Grade: A+

### In Progress
- 🔄 **Client Module Re-Enable** - 91 errors to fix
- 🔄 **Test Coverage to 90%** - Currently 60%
- 🔄 **Unwrap/Expect Reduction** - 322 → <100 target

### Planned
- ⏳ **Large File Refactoring** - 2 files > 900 lines
- ⏳ **Sleep Elimination** - 30 files identified
- ⏳ **E2E Test Suite** - Comprehensive coverage

---

## 🔗 **External Resources**

- **Specs**: `specs/README.md`
- **API Docs**: `docs/api/`
- **Architecture**: `docs/architecture/`
- **wateringHole**: `../../../wateringHole/` (inter-primal discussions)

---

## ✨ **Recent Updates**

### 2026-01-13
- Archived 12 session documents to `archive/sessions-jan13-2026/`
- Updated STATUS.md with concurrent evolution results
- Created ROOT_DOCS_INDEX.md for navigation
- Clean root directory (31 → 19 markdown files)

---

**"Different orders of the same architecture - now well-documented!"** 🍄🐸✨

