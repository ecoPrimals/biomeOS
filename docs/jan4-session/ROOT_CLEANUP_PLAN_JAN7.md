# 🧹 biomeOS Root Cleanup Plan - January 7, 2026

## 🎯 Goal

Clean up root directory documentation while waiting for Songbird BTSP/tarpc work.

## 📊 Current State Analysis

### Root Documentation Files (9 total):
```
CLEANUP_COMPLETE_JAN3_2026.md         - Jan 3 (session complete marker)
MASTER_DOCUMENTATION_INDEX.md         - Jan 5 (index)
README.md                             - Jan 5 (main entry point) ✅
ROOT_DOCS_INDEX.md                    - Jan 3 (duplicate index?)
START_HERE_NEXT_SESSION.md            - Jan 3 (entry point)
START_HERE_ZERO_HARDCODING.md         - Jan 3 (entry point)
STATUS.md                             - Jan 5 (current status)
TEST_COVERAGE_IMPROVEMENT_PLAN.md     - Dec 31 (planning doc)
WORKSPACE_STATUS.md                   - Jan 3 (workspace status)
```

### Issues Identified:

1. **Multiple Entry Points**: 3 different "START_HERE" files
2. **Duplicate Indexes**: MASTER_DOCUMENTATION_INDEX.md + ROOT_DOCS_INDEX.md
3. **Outdated Status**: All say "Federation Complete" but we're at v3.14.1 now
4. **Session Markers**: CLEANUP_COMPLETE_JAN3_2026.md should be archived
5. **Stale Planning**: TEST_COVERAGE_IMPROVEMENT_PLAN.md from Dec 31

## 🧹 Cleanup Actions

### Action 1: Consolidate Entry Points

**Problem**: 3 different START_HERE files confuse new users

**Solution**: Keep ONE primary entry point

**Keep**:
- `README.md` - Main entry point (standard)

**Archive** to `docs/jan4-session/`:
- `START_HERE_NEXT_SESSION.md` - Session-specific
- `START_HERE_ZERO_HARDCODING.md` - Session-specific

**Reason**: README.md is the universal standard for project entry points

---

### Action 2: Consolidate Indexes

**Problem**: MASTER_DOCUMENTATION_INDEX.md vs ROOT_DOCS_INDEX.md

**Solution**: Keep ONE index

**Keep**:
- `MASTER_DOCUMENTATION_INDEX.md` - More comprehensive

**Archive** to `docs/jan4-session/`:
- `ROOT_DOCS_INDEX.md` - Duplicate, less complete

**Reason**: One source of truth for documentation structure

---

### Action 3: Archive Session Markers

**Problem**: Session completion markers clutter root

**Files to Archive**:
- `CLEANUP_COMPLETE_JAN3_2026.md` → `docs/jan4-session/`
- `WORKSPACE_STATUS.md` → `docs/jan4-session/`

**Reason**: Session-specific docs belong in session folders

---

### Action 4: Update Current Status

**Problem**: STATUS.md says "Federation Complete" but we're past that now

**Solution**: Update STATUS.md with:
- ✅ Federation working (v3.14.1)
- ✅ Tag-based genetic lineage working
- 🎯 Waiting: BTSP tunnels + tarpc P2P
- 🎯 Current: Root cleanup

**Action**: Rewrite STATUS.md to reflect Jan 7, 2026 state

---

### Action 5: Handle Planning Docs

**Problem**: TEST_COVERAGE_IMPROVEMENT_PLAN.md from Dec 31

**Options**:
1. Archive to `docs/jan4-session/` (if completed)
2. Keep if still relevant
3. Update if partially completed

**Decision**: Review and decide (check if tests improved since Dec 31)

---

### Action 6: Clean Up bin/ Scripts

**Current State**:
```
bin/
├─ live-demo.sh
├─ nestgate-auth-showcase.sh
├─ pull-primals.sh
├─ showcase-runner.sh
└─ README.md
```

**Questions**:
- Are these scripts still used?
- Are they documented?
- Should they be in `scripts/` instead?

**Action**: Review each script's purpose and usage

---

## 📋 Execution Plan

### Phase 1: Archive Session Docs
```bash
# Move session-specific docs to docs/jan4-session/
mv START_HERE_NEXT_SESSION.md docs/jan4-session/
mv START_HERE_ZERO_HARDCODING.md docs/jan4-session/
mv ROOT_DOCS_INDEX.md docs/jan4-session/
mv CLEANUP_COMPLETE_JAN3_2026.md docs/jan4-session/
mv WORKSPACE_STATUS.md docs/jan4-session/
```

### Phase 2: Update STATUS.md
```bash
# Rewrite STATUS.md with current state (Jan 7, 2026)
# Include:
# - Tag-based federation working (v3.14.1)
# - BTSP/tarpc in progress (Songbird team)
# - Root cleanup in progress
# - Next: Cross-LAN deployment after BTSP ready
```

### Phase 3: Clean README.md
```bash
# Update README.md to reflect:
# - Current version and date
# - Latest achievements (tag-based trust)
# - Clear "What's Next" section
# - Point to MASTER_DOCUMENTATION_INDEX.md for deep dive
```

### Phase 4: Review bin/ Scripts
```bash
# For each script in bin/:
# 1. Check if still used
# 2. Document purpose in bin/README.md
# 3. Consider moving to scripts/ if appropriate
```

### Phase 5: Archive Planning Docs (if done)
```bash
# If TEST_COVERAGE_IMPROVEMENT_PLAN.md is complete:
mv TEST_COVERAGE_IMPROVEMENT_PLAN.md docs/jan4-session/

# If still active, update with current progress
```

---

## 🎯 Target Structure (After Cleanup)

### Root Level (Minimal):
```
biomeOS/
├─ README.md                          ← Primary entry point
├─ STATUS.md                          ← Current status (Jan 7, 2026)
├─ MASTER_DOCUMENTATION_INDEX.md      ← Documentation hub
├─ Cargo.toml, Cargo.lock             ← Rust project files
├─ bin/                               ← Executables (documented)
├─ crates/                            ← Source code
├─ docs/                              ← All documentation
│  ├─ jan4-session/                   ← Current session docs
│  └─ ... other docs
├─ primals/                           ← Primal binaries
├─ scripts/                           ← Utility scripts
└─ ... other directories
```

### Key Principles:
1. **One Entry Point**: README.md only
2. **One Index**: MASTER_DOCUMENTATION_INDEX.md only
3. **One Status**: STATUS.md only
4. **Session Docs**: In docs/jan4-session/
5. **Planning Docs**: Archived when complete

---

## ✅ Success Criteria

After cleanup, root directory should have:

- [ ] Single entry point (README.md)
- [ ] Single documentation index (MASTER_DOCUMENTATION_INDEX.md)
- [ ] Current status file (STATUS.md updated to Jan 7)
- [ ] No duplicate START_HERE files
- [ ] Session-specific docs in docs/jan4-session/
- [ ] bin/ scripts documented in bin/README.md
- [ ] Clear "What's Next" for new contributors

---

## 📝 Notes

### What NOT to Touch:
- `Cargo.toml`, `Cargo.lock` (project files)
- `crates/` (source code)
- `docs/` structure (except moving files into it)
- `primals/` (binaries)
- `scripts/` (utilities)

### What to Preserve:
- All documentation content (archive, don't delete)
- Git history
- Links between documents (update if needed)

---

## 🚀 Next Steps

1. ✅ Review this plan
2. Execute Phase 1 (archive session docs)
3. Execute Phase 2 (update STATUS.md)
4. Execute Phase 3 (clean README.md)
5. Execute Phase 4 (review bin/ scripts)
6. Execute Phase 5 (handle planning docs)
7. Verify all links still work
8. Update MASTER_DOCUMENTATION_INDEX.md if needed

---

**Date**: January 7, 2026, 21:40 UTC  
**Status**: Plan ready for execution  
**Priority**: Medium (housekeeping while waiting for Songbird)

