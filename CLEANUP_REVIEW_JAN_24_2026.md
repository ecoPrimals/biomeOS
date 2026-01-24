# 🧹 biomeOS Cleanup Review - January 24, 2026

## Overview

This document reviews archive code, disabled tests, and TODOs for potential cleanup while maintaining the fossil record.

---

## 📊 Current State

### Archive Code
- **Location**: `archive/stale_examples_jan_25_2026/`
- **Files**: 7 Rust files
- **Status**: Already archived, safe to keep
- **Purpose**: Historical reference, learning examples

### Disabled Tests
- **Count**: 13 `.disabled` test files
- **Status**: Candidates for cleanup or re-enabling
- **Impact**: Not running, not affecting CI

### TODOs/FIXMEs
- **Count**: 99 occurrences across 41 files
- **Types**: Implementation notes, future work, integration points
- **Priority**: Review for outdated items

---

## 🗂️ Archive Code Analysis

### Files in `archive/stale_examples_jan_25_2026/`

1. **squirrel_nucleus_integration.rs** ✅ KEEP
   - Historical example of Squirrel integration
   - Useful for understanding evolution

2. **universal_client_beardog.rs** ✅ KEEP
   - Shows old client pattern
   - Reference for capability evolution

3. **rust_atomic_deployment.rs** ✅ KEEP
   - Early atomic deployment prototype
   - Shows architecture evolution

4. **neural_graph_execution.rs** ✅ KEEP
   - Neural API early example
   - Demonstrates graph pattern development

5. **neural_api_integration_test.rs** ✅ KEEP
   - Test pattern evolution
   - Shows integration approach changes

6. **neural_api_client_demo.rs** ✅ KEEP
   - Client usage examples
   - Useful for understanding API evolution

7. **client_tests.rs** ✅ KEEP
   - Test infrastructure evolution
   - Shows testing approach changes

**Recommendation**: ✅ **KEEP ALL** - These are properly archived and serve as fossil record.

---

## 🚫 Disabled Tests Analysis

### Tests Currently Disabled

#### 1. UI Tests
```
./crates/biomeos-ui/tests/integration_tests.rs.disabled
```
**Reason**: Likely waiting for primal integration
**Action**: 🔄 **REVIEW** - Check if can be re-enabled or removed

#### 2. Core Protocol Tests
```
./crates/biomeos-core/tests/protocol_integration_tests.rs.disabled
./crates/biomeos-core/tests/squirrel_integration_test.rs.disabled
```
**Reason**: Squirrel integration pending
**Action**: ✅ **KEEP** - Will be re-enabled with Squirrel work

#### 3. Spore E2E Tests
```
./crates/biomeos-spore/tests/e2e_tests.rs.disabled
```
**Reason**: Requires full primal infrastructure
**Action**: 🔄 **REVIEW** - Check if plasmidBin structure is ready

#### 4. Atomic Deploy Tests
```
./crates/biomeos-atomic-deploy/tests/graph_execution_tests.rs.disabled
./crates/biomeos-atomic-deploy/tests/fault_injection_tests.rs.disabled
```
**Reason**: Complex integration requirements
**Action**: ✅ **KEEP** - Important for future quality

#### 5. Graph Tests
```
./crates/biomeos-graph/tests/collaborative_intelligence_e2e.rs.disabled
```
**Reason**: Requires Squirrel integration
**Action**: ✅ **KEEP** - Part of roadmap

#### 6. API Tests
```
./crates/biomeos-api/tests/websocket_integration.rs.disabled
```
**Reason**: WebSocket infrastructure pending
**Action**: 🔄 **REVIEW** - Check current UI capabilities

#### 7. Root Level Tests
```
./tests/atomic_lineage_deployment_test.rs.disabled
./tests/e2e_tests.rs.disabled
./tests/health_monitoring_integration_tests.rs.disabled
./tests/real_primal_integration.rs.disabled
./tests/chaos_tests.rs.disabled
```
**Reason**: Require full ecosystem deployment
**Action**: ✅ **KEEP** - Critical for production readiness

### Disabled Tests Recommendation

| Test | Status | Action |
|------|--------|--------|
| UI integration | Outdated? | 🔄 Review |
| Protocol integration | Waiting | ✅ Keep |
| Squirrel integration | Waiting | ✅ Keep |
| Spore E2E | Blocked | 🔄 Review |
| Atomic deploy tests | Important | ✅ Keep |
| Graph collaborative | Roadmap | ✅ Keep |
| API websocket | Check status | 🔄 Review |
| E2E tests (root) | Critical | ✅ Keep |
| Chaos tests | Quality | ✅ Keep |

**Summary**:
- ✅ **KEEP**: 9 tests (important for roadmap)
- 🔄 **REVIEW**: 4 tests (check if salvageable)

---

## 📝 TODO Analysis

### Categories of TODOs

#### 1. Integration Points (High Priority)
**Count**: ~30
**Examples**:
- Songbird capability discovery
- BearDog crypto integration
- NestGate storage calls
- Squirrel AI integration

**Action**: ✅ **KEEP** - These mark known integration work

#### 2. Implementation Details (Medium Priority)
**Count**: ~25
**Examples**:
- Key caching
- Atomic counters
- Parameter validation
- Health checks

**Action**: ✅ **KEEP** - Valid future improvements

#### 3. Test Infrastructure (Low Priority)
**Count**: ~20
**Examples**:
- "Fix test setup"
- "Update to new format"
- "Unify schemas"

**Action**: 🔄 **REVIEW** - Some may be outdated

#### 4. Configuration/Metadata (Low Priority)
**Count**: ~15
**Examples**:
- "Get from config"
- "Extract from binary"
- "Read from manifest"

**Action**: ✅ **KEEP** - Known technical debt

#### 5. Stub Implementations (Very Low Priority)
**Count**: ~9
**Examples**:
- "Full CLI integration pending"
- "This is a stub implementation"

**Action**: 🔄 **REVIEW** - May be outdated after recent work

### Specific TODOs to Review

#### Potentially Outdated

1. **`crates/biomeos/src/modes/cli.rs`**
   ```rust
   // TODO: This is a stub implementation
   ```
   **Check**: Has CLI been implemented more fully?

2. **`crates/biomeos-graph/tests/integration_tests.rs`**
   ```rust
   #[ignore = "Neural API graphs use different format - TODO: unify graph schemas"]
   ```
   **Check**: Have schemas been unified?

3. **`crates/biomeos-manifest/tests/niche_integration_tests.rs`**
   ```rust
   #[ignore = "Requires tower_deploy.toml - TODO: update niches to new graph format"]
   ```
   **Check**: Are niches still using old format?

4. **`crates/biomeos-graph/src/lib.rs`**
   ```rust
   // TODO: Re-enable after Wave 2 evolution to use CapabilityTaxonomy
   ```
   **Check**: What wave are we on? Is this done?

#### Definitely Keep

1. **Integration TODOs**: All Songbird, BearDog, NestGate, Squirrel integration points
2. **Future Features**: Rollback strategies, advanced health checks, etc.
3. **Performance**: Caching, optimization notes
4. **Security**: "Get from secure storage", "Verify genetic lineage"

---

## 🎯 Cleanup Recommendations

### Phase 1: Safe Cleanup (Do Now)

#### 1. Review & Update Test Ignore Messages
**Files**:
- `crates/biomeos-graph/tests/integration_tests.rs` (2 TODOs)
- `crates/biomeos-manifest/tests/niche_integration_tests.rs` (5 TODOs)
- `crates/biomeos-spore/tests/unit_tests.rs` (1 TODO)
- `crates/biomeos-spore/tests/chaos_tests.rs` (2 TODOs)
- `crates/biomeos-spore/tests/fault_injection_tests.rs` (2 TODOs)
- `crates/biomeos-spore/tests/nucleus_integration_test.rs` (1 TODO)

**Action**: Update ignore messages to reflect current status or remove if tests can run.

#### 2. Check Stub Implementations
**Files**:
- `crates/biomeos/src/modes/cli.rs`
- `crates/biomeos/src/modes/api.rs`
- `crates/biomeos/src/modes/deploy.rs`
- `crates/biomeos/src/modes/verify_lineage.rs`

**Action**: Verify if these are still stubs or have been implemented.

#### 3. Review Disabled Tests Status
**Files**: 4 tests marked for review
- `crates/biomeos-ui/tests/integration_tests.rs.disabled`
- `crates/biomeos-spore/tests/e2e_tests.rs.disabled`
- `crates/biomeos-api/tests/websocket_integration.rs.disabled`

**Action**: Determine if can be:
- Re-enabled with current infrastructure
- Updated to work with new patterns
- Removed if superseded by other tests

### Phase 2: Documentation Cleanup (Low Priority)

#### 1. Add TODO Tracking Document
Create `TODO_ROADMAP.md` categorizing TODOs by:
- Priority (High/Medium/Low)
- Category (Integration/Performance/Testing)
- Estimated effort
- Dependencies

#### 2. Update TODOs with Context
For important TODOs, add:
- Issue tracker reference (if applicable)
- Related primals/components
- Estimated completion timeline

### Phase 3: Archive Management (Keep Current)

#### 1. Archive Organization
Current structure is good:
```
archive/
├── sessions/2026-01-24/       # Today's work
├── stale_examples_jan_25_2026/ # Old code examples
├── docs-fossil-record/         # Historical docs
└── ... other archives
```

**Action**: ✅ No changes needed

#### 2. Archive README
Consider adding: `archive/README.md` explaining:
- What gets archived
- When to archive
- How to find historical information

---

## 🚀 Immediate Action Items

### 1. Quick Wins (15 minutes)

```bash
# Check if graph schemas have been unified
cargo test -p biomeos-graph --lib integration_tests

# Check if niches have been updated
ls niches/*.toml | head -3

# Verify CLI implementation status
wc -l crates/biomeos/src/modes/cli.rs
```

### 2. Test Review (30 minutes)

For each disabled test:
1. Read the test code
2. Check if dependencies are available
3. Try enabling and running
4. Document decision (enable/keep disabled/remove)

### 3. TODO Audit (1 hour)

1. Group TODOs by category
2. Mark outdated ones for removal
3. Update valid ones with more context
4. Create tracking issue for high-priority items

---

## 📋 Specific Cleanup Commands

### Check Test Viability

```bash
# Check disabled tests one by one
for test in $(find . -name "*.disabled" | grep -v target); do
    echo "=== $test ==="
    head -20 "$test"
    echo ""
done
```

### Update Test Ignore Messages

```bash
# Find all ignored tests with TODO
grep -r "#\[ignore.*TODO" --include="*.rs" crates/ tests/
```

### Count TODOs by Category

```bash
# Integration TODOs
grep -r "TODO.*Songbird\|TODO.*BearDog\|TODO.*NestGate\|TODO.*Squirrel" \
  --include="*.rs" crates/ | wc -l

# Implementation TODOs
grep -r "TODO: Implement" --include="*.rs" crates/ | wc -l

# Test TODOs
grep -r "TODO.*test\|TODO.*Test" --include="*.rs" crates/ | wc -l
```

---

## ✅ Summary

### What to Keep
- ✅ All archived code in `archive/` (fossil record)
- ✅ Most disabled tests (roadmap items)
- ✅ Integration TODOs (known work)
- ✅ Future feature TODOs (valid improvements)

### What to Review
- 🔄 4 disabled tests (check viability)
- 🔄 Test ignore messages (update or clarify)
- 🔄 Stub implementation comments (verify status)
- 🔄 Old format references (check if updated)

### What to Remove
- ❌ None immediately - all serve documentation purpose
- ❌ Consider after review phase

### Recommendation

**Do NOT remove** anything yet. Instead:

1. **Audit Phase**: Review the 4 questionable items
2. **Update Phase**: Clarify TODOs and ignore messages
3. **Document Phase**: Create TODO_ROADMAP.md
4. **Then Decide**: Remove only what's truly obsolete

---

## 🎯 Next Steps

1. Run quick check commands above
2. Review 4 disabled tests
3. Update test ignore messages with current status
4. Create TODO tracking document (optional)
5. Push all changes via SSH

**Estimated Time**: 1-2 hours for complete review

---

**Status**: Ready for cleanup audit  
**Risk**: Low (keeping almost everything)  
**Benefit**: Better clarity on technical debt

