# Deep Debt Evolution Plan - January 12, 2026

**Philosophy**: Evolve to modern idiomatic Rust, not just fix issues  
**Status**: Analysis Complete, Ready for Execution  
**Approach**: Smart evolution over quick fixes

---

## 📊 Current State Analysis

### ✅ **EXCELLENT** - No Action Needed

1. **File Sizes**: ✅ ALL files < 1000 lines
   - Largest: 904 lines (`widgets.rs`)
   - **Grade**: A+ (Perfect)
   - **Action**: None needed

2. **Unsafe Code**: ✅ Only 2 justified blocks
   - `libc::kill` - Process existence check
   - `libc::getuid` - User ID retrieval
   - **Grade**: A+ (Justified syscalls only)
   - **Action**: Document as intentional, consider safe wrappers

3. **Production Mocks**: ✅ Isolated to tests
   - `biomeos-test-utils/src/mock_primal.rs` - Test utility ✅
   - Other 4 files need verification
   - **Grade**: A (Needs verification)
   - **Action**: Verify remaining 4 files

---

## ⚠️ **NEEDS EVOLUTION** - Priority Work

### 1. Hardcoded Primal Names (1,263 matches across 135 files)

**Current State**: HIGH
- 1,263 instances of primal names in code
- Most are acceptable (tests, docs, capability definitions)
- Need to identify production code hardcoding

**Evolution Target**: Capability-based discovery throughout

**Strategy**:
```rust
// BEFORE (hardcoded):
let client = connect_to_primal("beardog:nat0")?; // ❌

// AFTER (capability-based):
let crypto_primal = discover_by_capability("encryption")?; // ✅
let client = connect_to_primal(&crypto_primal.endpoint)?;
```

**Action Items**:
1. ✅ Identify production code (non-test, non-doc, non-example)
2. ⏳ Categorize by severity:
   - Critical: Discovery/connection code
   - Medium: Configuration/defaults
   - Low: Error messages/logging
3. ⏳ Evolve critical paths first
4. ⏳ Add capability-based wrappers
5. ⏳ Update tests to use capability discovery

**Estimate**: 12-16 hours

---

### 2. External Process Dependencies (8 instances)

**Current State**: Using `std::process::Command` in 8 places

**Files**:
1. `biomeos-boot/src/bootable.rs` - ⚠️ Needs analysis
2. `biomeos-boot/src/rootfs.rs` - ⚠️ Needs analysis
3. `biomeos-boot/src/initramfs.rs` - ⚠️ Needs analysis
4. `biomeos-chimera/src/builder.rs` - ⚠️ Auto-generated
5. `biomeos-core/src/lab/mod.rs` - ⚠️ Needs analysis
6. `biomeos-core/src/vm_federation.rs` - ⚠️ Needs analysis
7. `biomeos-deploy/src/network.rs` - ⚠️ Needs analysis
8. `biomeos-federation/src/modules/manifest.rs` - ⚠️ Needs analysis

**Evolution Target**: Pure Rust or capability-delegated

**Strategy**:
```rust
// BEFORE (external process):
Command::new("ip").args(&["addr", "add"]).output()?; // ❌

// OPTION A (Pure Rust):
use netlink::addr_add; // ✅
addr_add(interface, ip_address)?;

// OPTION B (Capability-based):
let network = discover_by_capability("network.configuration")?; // ✅
network.configure_interface(interface, ip_address)?;
```

**Action Items**:
1. ⏳ Analyze each usage:
   - Can it be pure Rust? (netlink, nix, etc.)
   - Should it be delegated to a primal?
   - Is it truly needed?
2. ⏳ Prioritize by criticality
3. ⏳ Evolve to pure Rust where possible
4. ⏳ Delegate to primals where appropriate
5. ⏳ Document remaining external calls

**Estimate**: 8-12 hours

---

### 3. Mock Usage in Production Code

**Current State**: 5 files with "Mock" structures

**Files to Verify**:
1. `biomeos-graph/src/executor.rs` - ✅ Checked (only in tests)
2. `biomeos-core/src/primal_orchestrator.rs` - ⏳ Needs check
3. `biomeos-core/src/discovery_modern.rs` - ⏳ Needs check
4. `biomeos-api/src/state.rs` - ⏳ Needs check
5. `biomeos-test-utils/src/mock_primal.rs` - ✅ Test utility

**Evolution Target**: Zero mocks in production

**Strategy**:
```rust
// BEFORE (mock in production):
struct MockPrimalClient { ... } // ❌ In production code

// AFTER (proper abstraction):
#[cfg(test)]
struct MockPrimalClient { ... } // ✅ Only in tests

// Production uses trait:
trait PrimalClient {
    async fn call(&self, method: &str) -> Result<Value>;
}
```

**Action Items**:
1. ⏳ Verify each file's mock usage
2. ⏳ Move test mocks to `#[cfg(test)]` modules
3. ⏳ Replace production mocks with real implementations
4. ⏳ Use trait abstractions for testability

**Estimate**: 4-6 hours

---

### 4. Test Coverage Expansion (71.54% → 90%)

**Current State**: Good baseline coverage

**Modules Needing Work**:
- `executor.rs`: 34.94% → 90% (6-8 hours)
- `metrics.rs`: 52.33% → 90% (3-4 hours)
- `parser.rs`: 56.59% → 90% (2-3 hours)

**Evolution Target**: 90% coverage with quality tests

**Strategy**: Quality over quantity
- Test critical paths
- Test error handling
- Test edge cases
- No trivial tests

**Action Items**: See [TEST_COVERAGE_REPORT_JAN12.md](TEST_COVERAGE_REPORT_JAN12.md)

**Estimate**: 11-15 hours

---

## 📋 Deep Debt Principles Applied

### 1. ✅ Modern Idiomatic Rust
- async/await throughout
- Result<T,E> error handling
- Type-safe configuration
- No "jelly string" bash scripts

### 2. ✅ External Dependencies → Rust
- Identify: std::process::Command usage
- Analyze: Can it be pure Rust?
- Evolve: netlink, nix, or capability delegation

### 3. ✅ Smart Refactoring
- All files < 1000 lines ✅
- No need for splitting
- Focus on logical separation

### 4. ✅ Unsafe → Safe Rust
- Only 2 justified unsafe blocks
- Both are low-level syscalls
- Consider safe wrappers

### 5. ⏳ Hardcoding → Capability-Based
- 1,263 instances to analyze
- Evolve critical paths first
- Runtime discovery only

### 6. ✅ Primal Self-Knowledge
- No assumptions about other primals
- Discovery at runtime
- Capability-based selection

### 7. ⏳ Mocks → Real Implementations
- Verify 4 files
- Ensure #[cfg(test)] isolation
- No production mocks

---

## 🎯 Execution Plan

### Phase 1: Verification & Analysis (4-6 hours)
1. ⏳ Analyze hardcoding patterns
2. ⏳ Categorize by severity (critical/medium/low)
3. ⏳ Verify mock usage in 4 files
4. ⏳ Analyze external process dependencies
5. ⏳ Document findings

### Phase 2: Critical Evolution (12-16 hours)
1. ⏳ Evolve critical hardcoding paths
2. ⏳ Replace external processes with pure Rust
3. ⏳ Move/remove production mocks
4. ⏳ Add capability-based wrappers

### Phase 3: Coverage Expansion (11-15 hours)
1. ⏳ Add executor.rs tests (6-8h)
2. ⏳ Add metrics.rs tests (3-4h)
3. ⏳ Add parser.rs tests (2-3h)

### Phase 4: Documentation & Verification (2-3 hours)
1. ⏳ Document remaining intentional items
2. ⏳ Verify all tests pass
3. ⏳ Measure final coverage
4. ⏳ Create completion report

**Total Estimate**: 29-40 hours

---

## 📊 Success Metrics

### Target State
- ✅ Zero compilation errors
- ⏳ < 50 hardcoded primal names in production code
- ⏳ Zero external process calls (or all justified)
- ⏳ Zero production mocks
- ⏳ 90% test coverage
- ✅ 100% safe Rust (except justified syscalls)
- ✅ All files < 1000 lines

### Quality Gates
- [ ] All hardcoding analyzed and categorized
- [ ] Critical paths use capability discovery
- [ ] External processes evolved or justified
- [ ] No mocks in production code
- [ ] 90% test coverage achieved
- [ ] Documentation current
- [ ] All tests passing

---

## 🔍 Detailed Analysis Needed

### Hardcoding Analysis
```bash
# Find production code hardcoding (exclude tests/docs/examples)
grep -r "beardog\|songbird\|toadstool" crates/*/src \
  --include="*.rs" \
  --exclude-dir=tests \
  | grep -v "//\|#\[cfg(test)\]\|///\|capability_taxonomy"
```

### Mock Verification
```bash
# Check each file for Mock usage
grep -A 10 "struct.*Mock\|Mock.*struct" \
  crates/biomeos-core/src/primal_orchestrator.rs \
  crates/biomeos-core/src/discovery_modern.rs \
  crates/biomeos-api/src/state.rs
```

### External Process Audit
```bash
# Analyze each Command usage
grep -B 5 -A 10 "Command::new" \
  crates/biomeos-boot/src/*.rs \
  crates/biomeos-core/src/lab/mod.rs \
  crates/biomeos-core/src/vm_federation.rs
```

---

## 💡 Evolution Examples

### Example 1: Hardcoding → Capability Discovery

**Before**:
```rust
// biomeos-core/src/discovery_bootstrap.rs
pub async fn bootstrap_discovery() -> Result<()> {
    let beardog = connect("beardog:nat0")?; // ❌ Hardcoded
    let songbird = connect("songbird:nat0")?; // ❌ Hardcoded
    Ok(())
}
```

**After**:
```rust
// biomeos-core/src/discovery_bootstrap.rs
pub async fn bootstrap_discovery() -> Result<()> {
    // Discover by capability, not name
    let crypto = find_by_capability("encryption").await?; // ✅
    let discovery = find_by_capability("discovery").await?; // ✅
    Ok(())
}
```

### Example 2: External Process → Pure Rust

**Before**:
```rust
// biomeos-deploy/src/network.rs
pub fn configure_interface(name: &str, ip: &str) -> Result<()> {
    Command::new("ip") // ❌ External process
        .args(&["addr", "add", ip, "dev", name])
        .output()?;
    Ok(())
}
```

**After (Option A - Pure Rust)**:
```rust
// biomeos-deploy/src/network.rs
use netlink_packet_route::AddressMessage;

pub fn configure_interface(name: &str, ip: &str) -> Result<()> {
    let addr = parse_ip(ip)?;
    netlink::add_address(name, addr)?; // ✅ Pure Rust
    Ok(())
}
```

**After (Option B - Capability Delegation)**:
```rust
// biomeos-deploy/src/network.rs
pub async fn configure_interface(name: &str, ip: &str) -> Result<()> {
    // Delegate to primal with network capability
    let network = discover_by_capability("network.configuration").await?;
    network.call("configure_interface", json!({
        "interface": name,
        "ip": ip
    })).await?;
    Ok(())
}
```

### Example 3: Mock → Real Implementation

**Before**:
```rust
// Production code
pub struct MockDiscovery {  // ❌ Mock in production
    primals: Vec<String>,
}
```

**After**:
```rust
// Production code
pub struct Discovery {  // ✅ Real implementation
    client: NucleusClient,
}

#[cfg(test)]
pub struct MockDiscovery {  // ✅ Mock only in tests
    primals: Vec<String>,
}
```

---

## 🚀 Next Steps

### Immediate (Today/Tomorrow)
1. Start Phase 1: Verification & Analysis
2. Analyze hardcoding patterns
3. Verify mock usage
4. Document findings

### Short-Term (This Week)
1. Begin Phase 2: Critical Evolution
2. Evolve highest-priority hardcoding
3. Replace external processes
4. Remove production mocks

### Medium-Term (Next 2 Weeks)
1. Complete Phase 3: Coverage Expansion
2. Complete Phase 4: Documentation
3. Final verification and testing

---

## 📚 References

- [COMPREHENSIVE_AUDIT_JAN12_2026.md](COMPREHENSIVE_AUDIT_JAN12_2026.md) - Full audit
- [DEEP_DEBT_EXECUTION_SUMMARY_JAN12.md](DEEP_DEBT_EXECUTION_SUMMARY_JAN12.md) - Philosophy
- [TEST_COVERAGE_REPORT_JAN12.md](TEST_COVERAGE_REPORT_JAN12.md) - Coverage details
- [docs/deep-debt/](docs/deep-debt/) - Deep debt documentation

---

**Status**: Ready for Execution  
**Priority**: High (aligns with core principles)  
**Estimate**: 29-40 hours total  
**Impact**: Significant improvement in code quality and maintainability  

**"Different orders of the same architecture."** 🍄🐸

