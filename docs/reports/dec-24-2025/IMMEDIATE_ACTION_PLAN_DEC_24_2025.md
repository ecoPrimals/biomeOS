# BiomeOS Immediate Action Plan
**Date**: December 24, 2025  
**Status**: 🔴 **CRITICAL - BUILD BROKEN**  
**Priority**: Fix blocking issues immediately

---

## 🚨 CRITICAL: Fix Build (30 minutes)

### Problem

Tests fail to compile with 6 errors:
```
error[E0599]: no function or associated item named `toadstool` found for struct `PrimalType`
error[E0599]: no function or associated item named `songbird` found for struct `PrimalType`
error[E0599]: no function or associated item named `nestgate` found for struct `PrimalType`
error[E0599]: no function or associated item named `beardog` found for struct `PrimalType`
error[E0599]: no function or associated item named `squirrel` found for struct `PrimalType`
error[E0599]: no function or associated item named `petaltongue` found for struct `PrimalType`
```

### Root Cause

Helper functions were removed during hardcoding elimination, but tests still reference them.

### Fix

**File**: `crates/biomeos-types/src/primal/core.rs`

Replace test helper calls:

```rust
// ❌ OLD (broken)
let pt = PrimalType::toadstool();

// ✅ NEW (working)
let pt = PrimalType::new("compute", "toadstool", "1.0.0");
```

### Affected Lines

1. Line 242: `test_primal_type_toadstool`
2. Line 250: `test_primal_type_songbird`
3. Line 257: `test_primal_type_nestgate`
4. Line 264: `test_primal_type_beardog`
5. Line 271: `test_primal_type_squirrel`
6. Line 337: `test_primal_service_info` (uses `PrimalType::toadstool()`)

### Commands

```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS

# Edit the file
vim crates/biomeos-types/src/primal/core.rs

# Verify fix
cargo test --package biomeos-types --lib primal::core::tests

# Run all tests
cargo test --workspace

# Should see: test result: ok
```

---

## 🔧 IMMEDIATE: Format Code (1 minute)

### Problem

2 files need formatting:
- `crates/biomeos-core/src/clients/base.rs`

### Fix

```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS

# Format all code
cargo fmt

# Verify
cargo fmt --check

# Should see: no output (all formatted)
```

---

## ⚠️ HIGH PRIORITY: Remove Hardcoded Endpoints (2 days)

### Problem

53 instances of hardcoded `localhost:*` endpoints violate architecture principles.

### Files to Fix

1. **`crates/biomeos-types/src/constants.rs`**
   - Remove deprecated `FALLBACK_*_ENDPOINT` constants
   - Keep only `DEFAULT_*_PORT` for reference

2. **`crates/biomeos-core/src/clients/*.rs`** (6 files)
   - Remove hardcoded endpoint defaults
   - Use environment variables with clear errors

3. **`crates/biomeos-core/src/config/mod.rs`**
   - Remove hardcoded fallbacks
   - Use discovery or fail with helpful message

### Pattern to Follow

```rust
// ❌ BAD - Hardcoded fallback
let endpoint = env::var("TOADSTOOL_ENDPOINT")
    .unwrap_or("http://localhost:8080".to_string());

// ✅ GOOD - Clear error
let endpoint = env::var("TOADSTOOL_ENDPOINT")
    .map_err(|_| anyhow!(
        "TOADSTOOL_ENDPOINT not set. Set environment variable or ensure \
         Songbird discovery is running. See docs/guides/primal-integration-guide.md"
    ))?;

// ✅ BETTER - Use discovery
let toadstool = self.discover_primal("compute").await
    .map_err(|e| anyhow!(
        "ToadStool compute service required but not found. \
         Ensure ToadStool is running and registered with Songbird. \
         Original error: {}", e
    ))?;
```

### Commands

```bash
# Find all hardcoded endpoints
grep -r "localhost" crates/biomeos-core/src --exclude-dir=tests
grep -r ":3000\|:8080\|:9000\|:8001\|:8002" crates/biomeos-core/src --exclude-dir=tests

# After fixing, verify none remain (except in tests)
grep -r "localhost" crates/biomeos-core/src --exclude-dir=tests
# Should see: no matches (or only in comments/docs)
```

---

## 🧪 HIGH PRIORITY: Add Integration Tests (1 week)

### Problem

All tests use mocks. Never tested with actual primal binaries.

### Available Binaries

All in `../phase1bins/`:
- `beardog-v0.9.3-senderfixed-dec24` (4.5M)
- `toadstool-bin` (4.3M)
- `squirrel-bin` (15M)
- `nestgate-bin` (3.4M)
- `songbird-bin` (21M)

### Test Framework

Create `tests/real_primal_integration_tests.rs`:

```rust
//! Integration tests with real primal binaries
//!
//! These tests start actual primal services and test BiomeOS integration.

use std::process::{Command, Child};
use std::time::Duration;
use tokio::time::sleep;

/// Start a real primal binary
fn start_primal(binary: &str, port: u16) -> Child {
    Command::new(format!("../phase1bins/{}", binary))
        .arg("--port")
        .arg(port.to_string())
        .spawn()
        .expect("Failed to start primal")
}

#[tokio::test]
async fn test_songbird_discovery() {
    // Start Songbird
    let mut songbird = start_primal("songbird-bin", 3000);
    sleep(Duration::from_secs(2)).await;
    
    // Test discovery
    let client = SongbirdClient::new("http://localhost:3000");
    let services = client.discover_all().await.unwrap();
    
    assert!(!services.is_empty());
    
    // Cleanup
    songbird.kill().unwrap();
}

#[tokio::test]
async fn test_toadstool_deployment() {
    // Start ToadStool
    let mut toadstool = start_primal("toadstool-bin", 8080);
    sleep(Duration::from_secs(2)).await;
    
    // Test deployment
    let client = ToadStoolClient::new("http://localhost:8080");
    let result = client.deploy_workload(/* ... */).await;
    
    assert!(result.is_ok());
    
    // Cleanup
    toadstool.kill().unwrap();
}

// Add tests for BearDog, NestGate, Squirrel...
```

### Commands

```bash
# Create test file
touch tests/real_primal_integration_tests.rs

# Run integration tests
cargo test --test real_primal_integration_tests

# Run all tests including integration
cargo test --workspace
```

---

## 📝 MEDIUM PRIORITY: Update Documentation (1 hour)

### Problem

STATUS.md and other docs claim "Production-Ready Grade A-" when reality is "Grade D+".

### Files to Update

1. **`STATUS.md`**
   - Change grade from A- to D+
   - Note build is broken
   - List known issues

2. **`DEPLOYMENT_READY.md`**
   - Mark as NOT ready
   - List blocking issues
   - Add prerequisites

3. **`README.md`**
   - Add "Work in Progress" notice
   - Link to audit reports
   - Set realistic expectations

### Example Update

```markdown
# BiomeOS Status

**Current Grade**: D+ (Not Production Ready)  
**Build Status**: ❌ BROKEN (6 test failures)  
**Last Updated**: December 24, 2025

## Critical Issues

1. ❌ Build broken - tests fail to compile
2. ❌ No real integration tests
3. ⚠️ 53 hardcoded endpoints
4. ⚠️ Test coverage unknown (build must pass first)

## See Also

- [Comprehensive Audit](COMPREHENSIVE_FINAL_AUDIT_DEC_24_2025.md)
- [Executive Summary](AUDIT_EXECUTIVE_SUMMARY_DEC_24_2025.md)
- [Action Plan](IMMEDIATE_ACTION_PLAN_DEC_24_2025.md)
```

---

## 📊 Progress Tracking

### Week 1 Checklist

- [ ] Fix 6 test compilation errors (30 min)
- [ ] Run `cargo fmt` (1 min)
- [ ] Verify `cargo test --workspace` passes
- [ ] Verify `cargo clippy --workspace` passes
- [ ] Remove hardcoded endpoints (2 days)
- [ ] Update STATUS.md with reality (1 hour)

**Goal**: Clean build, honest documentation

### Week 2 Checklist

- [ ] Create integration test framework
- [ ] Test with Songbird binary
- [ ] Test with ToadStool binary
- [ ] Test with BearDog binary
- [ ] Test with NestGate binary
- [ ] Test with Squirrel binary

**Goal**: Real primal integration working

### Week 3-4 Checklist

- [ ] Measure test coverage
- [ ] Add unit tests (coverage 37% → 60%+)
- [ ] Add E2E tests
- [ ] Add chaos tests
- [ ] Complete missing specs
- [ ] Performance testing

**Goal**: Production-ready system

---

## 🎯 Success Criteria

### Minimum Viable (Grade C)

- ✅ Build passes
- ✅ All tests run
- ✅ Zero hardcoded endpoints
- ✅ Documentation accurate

### Production Ready (Grade B)

- ✅ Integration tests with real primals
- ✅ 60%+ test coverage
- ✅ All critical specs implemented
- ✅ E2E tests passing

### Excellent (Grade A)

- ✅ 85%+ test coverage
- ✅ Chaos tests passing
- ✅ Performance benchmarked
- ✅ All specs implemented

---

## 🚀 Quick Start

### Step 1: Fix Build (30 min)

```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS

# Edit test file
vim crates/biomeos-types/src/primal/core.rs

# Replace PrimalType::toadstool() with PrimalType::new("compute", "toadstool", "1.0.0")
# (Do this for all 6 test failures)

# Verify
cargo test --workspace
```

### Step 2: Format Code (1 min)

```bash
cargo fmt
cargo fmt --check
```

### Step 3: Verify Clean Build

```bash
cargo build --workspace
cargo test --workspace
cargo clippy --workspace -- -D warnings

# All should pass
```

### Step 4: Remove Hardcoding (2 days)

See "HIGH PRIORITY: Remove Hardcoded Endpoints" section above.

### Step 5: Add Integration Tests (1 week)

See "HIGH PRIORITY: Add Integration Tests" section above.

---

## 📞 Need Help?

### Documentation

- **Comprehensive Audit**: `COMPREHENSIVE_FINAL_AUDIT_DEC_24_2025.md`
- **Executive Summary**: `AUDIT_EXECUTIVE_SUMMARY_DEC_24_2025.md`
- **This Action Plan**: `IMMEDIATE_ACTION_PLAN_DEC_24_2025.md`

### Previous Audits

- `AUDIT_SUMMARY_DEC_24_2025.md` - Initial audit
- `HARDCODING_AUDIT_DEC_24_2025.md` - Hardcoding analysis
- `BIOMEOS_EVOLUTION_PLAN_DEC_24_2025.md` - Evolution strategy

### Architecture Docs

- `BIOMEOS_RESPONSIBILITIES.md` - What BiomeOS should/shouldn't do
- `PRIMAL_AVAILABILITY.md` - Available primal binaries
- `specs/ARCHITECTURE_OVERVIEW.md` - System architecture

---

## ⏱️ Time Estimates

| Task | Estimated Time | Priority |
|------|----------------|----------|
| Fix build | 30 minutes | 🔴 CRITICAL |
| Format code | 1 minute | 🔴 CRITICAL |
| Remove hardcoding | 2 days | 🟡 HIGH |
| Update docs | 1 hour | 🟡 HIGH |
| Integration tests | 1 week | 🟡 HIGH |
| Improve coverage | 1 week | 🟢 MEDIUM |
| Complete specs | 2 weeks | 🟢 MEDIUM |

**Total to Production**: 3-4 weeks

---

**Status**: 🔴 Critical Issues  
**Next**: Fix build immediately  
**Goal**: Grade B (Production-Ready) by mid-January 2026

---

*"Fix the build. Test with real primals. Remove hardcoding. Then we're ready."*

