# Code Cleanup Plan - January 19, 2026

**Status**: Ready to execute  
**Warnings Found**: 56  
**Strategy**: Automated cleanup via `cargo fix` + manual review

---

## 📊 Warnings Breakdown

### **By Type**

| Type | Count | Auto-Fixable |
|------|-------|--------------|
| Unused imports | ~20 | ✅ Yes |
| Unused variables | ~10 | ⚠️ Partial |
| Dead code (fields) | ~15 | ⚠️ Manual |
| Dead code (functions) | ~5 | ⚠️ Manual |
| Profile warnings | 1 | ✅ Yes |

### **By Crate**

| Crate | Warnings | Priority |
|-------|----------|----------|
| `biomeos-graph` | 13 | High |
| `biomeos-spore` | 7 | Medium |
| `biomeos-core` | 15+ | Medium |
| `biomeos-nucleus` | 2 | Low |
| `biomeos-ui` | 10+ | Medium |

---

## 🎯 Cleanup Strategy

### **Phase 1: Automated Cleanup** (5-10 minutes)

**Action**: Run `cargo fix --allow-dirty --allow-staged`

**What it fixes**:
- ✅ Unused imports (automatic removal)
- ✅ Unused mut declarations
- ✅ Unused variables (adds `_` prefix)
- ✅ Profile warnings (moves to root)

**Expected**: ~30-40 warnings fixed automatically

### **Phase 2: Manual Review** (10-15 minutes)

**What remains**:
- Dead code (fields never read)
- Dead code (functions never used)
- Structs with unread fields

**Options**:
1. **Add `#[allow(dead_code)]`** if code is intentionally unused (future features)
2. **Remove** if truly dead/outdated
3. **Use `_` prefix** for fields (e.g., `_jsonrpc`) to acknowledge intentional non-use

---

## 📋 Detailed Cleanup Items

### **biomeos-graph** (13 warnings)

**Unused Imports** (auto-fix):
- `HashSet` in `executor.rs`
- `mpsc` in `executor.rs`
- `Duration`, `timeout` in `executor.rs`
- `NodeMetrics`, `PrimalGraph` in `executor.rs`
- `Context` in `templates.rs`
- `CoordinationPattern`, `Operation`, `PrimalNode`, `PrimalSelector` in `templates.rs`

**Unused Variables** (auto-fix):
- `event` in `ai_advisor.rs:371`
- `node` in `executor.rs:359`
- `context` in `executor.rs:372`

**Dead Code** (manual review):
- `local_patterns` field in `ai_advisor.rs:179`
- `name`, `description`, `confidence` fields in `ai_advisor.rs:185`
- `parse_constraints`, `parse_retry_policy` functions in `parser.rs:207`
- `family_id` field in `templates.rs:271`

**Recommendation**: 
- Auto-fix imports and variables
- Add `#[allow(dead_code)]` to AI advisor fields (future ML features)
- Remove unused parser functions if truly dead

---

### **biomeos-spore** (7 warnings)

**Unused Imports** (auto-fix):
- `PathBuf` in `manifest.rs:8`
- `std::collections::HashMap` in `neural_spore.rs:8`
- `crate::error::SporeResult` in `refresh.rs:9`
- `std::collections::HashMap` in `verification.rs:6`

**Unused Variables** (auto-fix):
- `deploy_local` in `incubation.rs:308`
- `manifest_path` in `refresh.rs:193`

**Dead Code** (manual review):
- `spore_seed` field in `incubation.rs:276`

**Recommendation**:
- Auto-fix all imports and variables
- Prefix field with `_spore_seed` if intentionally unused

---

### **biomeos-nucleus** (2 warnings)

**Dead Code** (manual review):
- `jsonrpc` and `id` fields in `client.rs:36` (never read)
- `paths` field in `discovery.rs:106` (never read)

**Recommendation**:
- If these are JSON-RPC protocol fields, prefix with `_`
- If truly unused, investigate and potentially remove

---

### **biomeos-core** (15+ warnings)

**Unused Imports** (auto-fix):
- `error` in `atomic_client.rs:23`
- `HealthError as BirdSongError` in `primal_health.rs:36`
- `debug` in `primal_health.rs:58`

**Unused Variables** (auto-fix):
- `monitor` in various files

**Dead Code** (manual review):
- `config` field in health monitoring
- `endpoint`, `last_check`, `consecutive_successes`, etc. in health structs
- `SecurityKeyExpiring`, `PartialConnectivity` variants (never constructed)

**Recommendation**:
- Auto-fix imports and variables
- Review health monitoring structs - may be for future features
- Add `#[allow(dead_code)]` to variants if part of comprehensive enum

---

### **biomeos-ui** (10+ warnings)

**Profile Warning** (auto-fix):
- Move profiles to workspace root `Cargo.toml`

**Recommendation**:
- Let `cargo fix` handle this automatically

---

## 🚀 Execution Plan

### **Step 1: Backup** (1 minute)

```bash
git status  # Ensure clean state
git add -A
git commit -m "chore: pre-cleanup checkpoint"
```

### **Step 2: Run Automated Cleanup** (5 minutes)

```bash
cargo fix --allow-dirty --allow-staged
cargo fmt
cargo check
```

**Expected Output**:
- ~30-40 warnings automatically fixed
- ~15-20 warnings remain (dead code)

### **Step 3: Review Changes** (5 minutes)

```bash
git diff  # Review all changes
```

**Verify**:
- Imports removed safely
- Variables prefixed with `_` appropriately
- No functionality broken

### **Step 4: Manual Cleanup** (10 minutes)

For remaining dead code warnings:

**Option A: Preserve for Future** (recommended for most):
```rust
#[allow(dead_code)]
struct FutureFeature {
    // ...
}
```

**Option B: Prefix with `_`** (for struct fields):
```rust
struct JsonRpcRequest {
    _jsonrpc: String,  // Protocol field, not read directly
    method: String,
}
```

**Option C: Remove** (only if truly dead):
- Delete the code
- Ensure tests still pass

### **Step 5: Validate** (5 minutes)

```bash
cargo check --all-targets
cargo test
cargo build --release
```

### **Step 6: Commit** (1 minute)

```bash
git add -A
git commit -m "chore: clean up unused imports and dead code

- Remove 30+ unused imports via cargo fix
- Prefix unused variables with _
- Add #[allow(dead_code)] for future features
- 56 → 0 compiler warnings
"
```

---

## 📊 Expected Results

### **Before**

```
Compiling biomeos...
warning: 56 warnings found
   Finished `dev` profile [unoptimized + debuginfo] target(s)
```

### **After**

```
Compiling biomeos...
   Finished `dev` profile [unoptimized + debuginfo] target(s)
```

**Metrics**:
- Warnings: 56 → 0 ✅
- Build time: ~same
- Binary size: ~same
- Functionality: 100% preserved

---

## ⚠️ What NOT to Clean

### **Keep These**

1. **TODO comments** - Future work reminders
2. **Test files** - Even if unused, part of test coverage
3. **Example files** - Documentation/reference
4. **Dead code with `#[cfg(test)]`** - Test helpers
5. **Fields in public API** - Breaking change

### **Archive Instead of Delete**

If you find large unused modules:
1. Move to `archive/` directory
2. Document why in archive README
3. Keep in git history

---

## 🎯 Success Criteria

✅ Zero compiler warnings  
✅ All tests pass  
✅ Clean `cargo clippy` output  
✅ No functionality lost  
✅ Git history clean

---

## 🔗 Related

- Previous cleanup: `ROOT_DOCS_CLEANUP_COMPLETE_JAN_16_2026.md`
- Archive structure: `archive/jan_2026_evolution/`

---

**Ready to Execute**: Yes  
**Estimated Time**: 20-30 minutes  
**Risk**: Low (git checkpoint before changes)

🧹🦀 **Clean code = happy compiler!** ✨

