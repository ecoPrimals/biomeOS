# BiomeOS Smart Refactoring Plan

**Date:** December 23, 2025  
**Purpose:** Intelligent refactoring of large files (>800 LOC) by logical concern, not arbitrary splitting

---

## 🎯 Philosophy: Smart Refactoring

**NOT:** Arbitrary file splitting just to meet line counts  
**YES:** Logical separation by domain concern with clear module boundaries

**Principles:**
1. Group related types together
2. Maintain cohesion - things that change together stay together
3. Clear module boundaries with minimal coupling
4. Preserve API compatibility
5. Improve discoverability and maintainability

---

## 📊 Files Requiring Refactoring

| File | Lines | Status | Approach |
|------|-------|--------|----------|
| `crates/biomeos-types/src/health.rs` | 1011 | ⚠️ Needs refactoring | Split by concern |
| `ui/src/minimal_app.rs` | 989 | ⚠️ Needs refactoring | Extract view modules |
| `src/universal_adapter.rs` | 905 | ✅ Acceptable | Well-structured, just under limit |

---

## 🏗️ Refactoring Strategy: health.rs (1011 lines)

### Current Structure Analysis

The file contains 26 public types organized into logical groups:

**Group 1: Core Health Status (Lines 1-176)**
- `Health` enum (primary type)
- Implementation methods

**Group 2: Health Issues & Remediation (Lines 177-320)**
- `HealthIssue` struct
- `HealthIssueCategory` enum
- `HealthIssueSeverity` enum
- `RemediationAction` struct
- `RemediationActionType` enum

**Group 3: Lifecycle States (Lines 321-364)**
- `StartupPhase` enum
- `ShutdownPhase` enum
- `MaintenanceType` enum

**Group 4: Health Reporting (Lines 365-442)**
- `HealthReport` struct
- `HealthSubject` struct
- `HealthSubjectType` enum
- `ComponentHealth` struct

**Group 5: Metrics (Lines 443-529)**
- `HealthMetrics` struct
- `ResponseTimeMetrics` struct
- `ResourceMetrics` struct
- `NetworkIoMetrics` struct
- `ErrorMetrics` struct
- `AvailabilityMetrics` struct

**Group 6: Events (Lines 530-579)**
- `HealthEvent` struct
- `HealthEventTrigger` enum

**Group 7: Configuration (Lines 580-690)**
- `HealthCheckConfig` struct
- `HealthCheckTarget` enum
- `MetricThreshold` struct
- `ThresholdOperator` enum
- `ThresholdAction` enum

**Group 8: Test Utilities (Lines 691-1011)**
- Test helper functions

### Proposed Module Structure

```
crates/biomeos-types/src/health/
├── mod.rs                  # Re-exports, main Health enum (150 lines)
├── issues.rs               # HealthIssue, categories, severity, remediation (150 lines)
├── lifecycle.rs            # Startup, shutdown, maintenance phases (50 lines)
├── reporting.rs            # HealthReport, subjects, components (100 lines)
├── metrics.rs              # All metrics structs (150 lines)
├── events.rs               # HealthEvent and triggers (80 lines)
├── config.rs               # Health check configuration (120 lines)
└── testing.rs              # Test utilities (200 lines)
```

**Benefits:**
- ✅ Each module under 200 lines
- ✅ Clear separation of concerns
- ✅ Easy to find related types
- ✅ Maintains cohesion
- ✅ Backward compatible via re-exports

### Implementation Steps

1. Create `health/` directory
2. Move `Health` enum and core impl to `mod.rs`
3. Extract each logical group to its own module
4. Add comprehensive re-exports in `mod.rs`
5. Update imports in dependent crates
6. Run tests to verify no breakage

---

## 🎨 Refactoring Strategy: ui/src/minimal_app.rs (989 lines)

### Current Structure Analysis

**Main sections:**
- App state management (100 lines)
- View rendering logic (400 lines)
- Event handling (200 lines)
- Helper functions (289 lines)

### Proposed Module Structure

```
ui/src/minimal_app/
├── mod.rs                  # App struct, main loop (150 lines)
├── state.rs                # Application state management (150 lines)
├── views/
│   ├── mod.rs              # View coordination (50 lines)
│   ├── dashboard.rs        # Dashboard view (150 lines)
│   ├── primals.rs          # Primals view (150 lines)
│   ├── services.rs         # Services view (150 lines)
│   └── settings.rs         # Settings view (100 lines)
├── events.rs               # Event handling (150 lines)
└── helpers.rs              # Helper functions (150 lines)
```

**Benefits:**
- ✅ Separation of view logic
- ✅ Easier to add new views
- ✅ Better testability
- ✅ Clear responsibilities

---

## ✅ Files NOT Requiring Refactoring

### src/universal_adapter.rs (905 lines)

**Analysis:** Well-structured with clear sections:
- Type definitions (100 lines)
- UniversalAdapter impl (200 lines)
- ToadstoolClient impl (200 lines)
- SongbirdClient impl (200 lines)
- Helper types (205 lines)

**Decision:** ✅ **KEEP AS IS**

**Rationale:**
- Just under 1000 line limit
- Logically cohesive - all adapter coordination in one place
- Clear internal structure with good comments
- Splitting would reduce cohesion
- Easy to navigate with IDE folding

**Alternative if needed:**
Could extract clients to separate files, but current structure is acceptable.

---

## 📈 Refactoring Priority

### Phase 1: High Priority (This Week)
1. ✅ **health.rs** - Most over limit (1011 lines), clear logical boundaries

### Phase 2: Medium Priority (Next Week)
2. ⚠️ **minimal_app.rs** - Slightly under limit but would benefit from view extraction

### Phase 3: Monitor
3. 👀 **universal_adapter.rs** - Watch for growth, refactor if exceeds 950 lines

---

## 🎯 Success Criteria

**For each refactored file:**
- ✅ All modules under 500 lines (ideally under 300)
- ✅ Clear module boundaries
- ✅ Backward compatible API
- ✅ All tests pass
- ✅ No performance regression
- ✅ Improved code discoverability

**Overall:**
- ✅ No files over 1000 lines
- ✅ Logical grouping preserved
- ✅ Easy to navigate codebase
- ✅ Maintainable long-term

---

## 🔄 Implementation Timeline

### Immediate (This Session)
- Document refactoring plan ✅
- Prepare module structure

### Next Session
- Implement health.rs refactoring
- Test and verify
- Update documentation

### Following Session
- Implement minimal_app.rs refactoring if needed
- Final verification

---

## 📝 Notes

**Why Not Refactor universal_adapter.rs Now?**
- It's well-structured and just under limit
- Splitting would reduce cohesion
- No immediate benefit
- Can revisit if it grows

**Why Refactor health.rs First?**
- Most over the limit (1011 lines)
- Clear logical boundaries
- High-value improvement
- Sets pattern for future refactoring

**Refactoring Philosophy:**
> "Refactor by concern, not by line count. Keep things that change together, together."

---

*Plan created: December 23, 2025*  
*Status: Ready for implementation*

