# 🎯 BiomeOS Audit Action Plan
**Date**: January 25, 2026  
**Based On**: COMPREHENSIVE_AUDIT_REPORT_JAN_25_2026.md  
**Timeline**: 2-3 weeks to critical compliance

---

## 🔴 PHASE 1: CRITICAL FIXES (Week 1 - Days 1-3)

### Day 1: Make Tests Pass

**Priority**: 🔴 BLOCKING - Nothing else matters if tests don't compile

```bash
# Step 1: Fix import errors in biomeos-api tests
# File: crates/biomeos-api/tests/discovery_handler_tests.rs
# Add at top:
use tower::util::ServiceExt;

# Step 2: Fix unused imports (clippy errors)
# Remove:
- PathBuf from biomeos-spore/src/manifest.rs
- HashMap from biomeos-spore/src/neural_spore.rs
- SporeResult from biomeos-spore/src/refresh.rs
- HashMap from biomeos-spore/src/verification.rs

# Step 3: Fix unused variables
# Prefix with underscore:
- deploy_local → _deploy_local in biomeos-spore/src/incubation.rs:308

# Step 4: Fix dead code
# Use or document:
- paths field in biomeos-nucleus/src/discovery.rs:106

# Step 5: Verify all tests
cargo test --workspace --no-fail-fast
```

**Success Criteria**: All tests compile and pass (or are explicitly ignored with justification)

---

### Day 2: Code Quality Baseline

**Priority**: 🔴 CRITICAL - Must pass quality checks

```bash
# Step 1: Auto-fix formatting
cargo fmt

# Step 2: Fix clippy errors
cargo clippy --all-targets --all-features --fix --allow-dirty

# Step 3: Manual fixes for clippy warnings that need judgment
# - Add `# Errors` documentation to Result-returning public functions
# - Add backticks to code terms in docs (e.g., BearDog → `BearDog`)
# - Use inline format args where suggested

# Step 4: Verify clean build
cargo clippy --all-targets --all-features -- -D warnings
```

**Success Criteria**: Zero clippy warnings, zero formatting issues

---

### Day 3: Test Coverage Assessment

**Priority**: 🟡 HIGH - Need to know where we stand

```bash
# Step 1: Run coverage with llvm-cov
cargo llvm-cov --all-features --workspace --html

# Or with tarpaulin:
cargo tarpaulin --out Html --output-dir coverage/

# Step 2: Review coverage report
# - Identify modules below 90%
# - Prioritize critical paths
# - Document current baseline

# Step 3: Create coverage improvement plan
# - List untested modules
# - Estimate effort
# - Prioritize by criticality
```

**Success Criteria**: Coverage report generated, baseline established, plan created

---

## 🟡 PHASE 2: STANDARDS COMPLIANCE (Week 1-2 - Days 4-10)

### Days 4-6: UniBin Refactoring

**Priority**: 🟡 HIGH - Ecosystem standard compliance

**Goal**: Create single `biomeos` binary with subcommands

```rust
// Design: crates/biomeos/src/main.rs

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "biomeos")]
#[command(about = "BiomeOS Orchestration System", version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Start API server
    Api {
        #[arg(long, default_value = "3000")]
        port: u16,
    },
    
    /// Interactive CLI mode
    Cli {
        #[arg(long)]
        socket: Option<String>,
    },
    
    /// Deploy a graph or niche
    Deploy {
        #[arg(value_name = "FILE")]
        graph: String,
    },
    
    /// Verify genetic lineage
    Verify {
        #[arg(value_name = "SPORE")]
        spore: String,
    },
    
    /// Health diagnostics
    Doctor {
        #[arg(long)]
        comprehensive: bool,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    
    match cli.command {
        Commands::Api { port } => biomeos_api::run_server(port).await,
        Commands::Cli { socket } => biomeos_cli::run_interactive(socket).await,
        Commands::Deploy { graph } => biomeos_deploy::deploy_graph(&graph).await,
        Commands::Verify { spore } => biomeos_spore::verify_lineage(&spore).await,
        Commands::Doctor { comprehensive } => biomeos_core::run_diagnostics(comprehensive).await,
    }
}
```

**Tasks**:
1. ✅ Update Cargo.toml to define single binary
2. ✅ Implement main.rs with subcommands
3. ✅ Update documentation
4. ✅ Test all modes
5. ✅ Update deployment scripts

**Success Criteria**: 
- Single `biomeos` binary in target/release/
- All modes functional
- `biomeos --help` comprehensive
- Documentation updated

---

### Days 7-8: Remove reqwest (ecoBin Compliance)

**Priority**: 🟡 HIGH - Eliminate C dependencies

**Goal**: Pure Rust, cross-compiles to musl

```toml
# Remove from workspace dependencies:
# reqwest = { version = "0.11", features = ["json"] } # DELETED

# Keep only in test-utils as dev-dependency:
[dev-dependencies]
reqwest = { version = "0.11", features = ["json"] }
```

**Code changes**:
```rust
// Instead of reqwest:
let response = reqwest::get(url).await?;

// Use Songbird delegation:
let response = neural_api.call_capability("http.get", json!({ "url": url })).await?;
```

**Tasks**:
1. ✅ Audit all reqwest usage
2. ✅ Replace with Songbird delegation pattern
3. ✅ Move reqwest to test-utils only
4. ✅ Test musl cross-compilation:
   ```bash
   cargo build --release --target x86_64-unknown-linux-musl
   ```
5. ✅ Verify zero C dependencies:
   ```bash
   cargo tree | grep -E "(openssl-sys|ring|aws-lc-sys|native-tls)"
   # Should return nothing
   ```

**Success Criteria**: 
- Builds for x86_64-unknown-linux-musl without errors
- No C dependencies in production code
- All functionality preserved via Songbird delegation

---

### Days 9-10: Hardcoded Port Cleanup

**Priority**: 🟡 HIGH - TRUE PRIMAL compliance

**Goal**: Unix sockets only, config-driven

**Changes**:
```rust
// BEFORE (hardcoded):
const DEFAULT_BIND_ADDR: &str = "127.0.0.1:3000";

// AFTER (config only):
// Remove constant entirely
// Use environment or config file:
let bind_addr = std::env::var("BIOMEOS_BIND_ADDR")
    .ok()  // Only for test/dev mode
    .filter(|_| cfg!(debug_assertions));  // Fail in production without config
```

**Tasks**:
1. ✅ Move all port constants to test modules only
2. ✅ Update production code to use Unix sockets:
   ```rust
   // Production IPC:
   UnixStream::connect("/primal/songbird").await?
   
   // NOT:
   reqwest::get("http://localhost:8080").await?
   ```
3. ✅ Update tests to use test fixtures
4. ✅ Document configuration in README
5. ✅ Update deployment guides

**Success Criteria**:
- No hardcoded ports in src/ (only in tests/)
- Production uses Unix sockets exclusively
- Config-driven for legitimate HTTP (e.g., external API)

---

## 🟢 PHASE 3: FILE ORGANIZATION (Week 2 - Days 11-14)

### Days 11-12: Refactor Oversized Files

**Priority**: 🟢 MEDIUM - Code organization standard

**Files to split** (>1000 lines):
1. `crates/biomeos-atomic-deploy/src/neural_executor.rs` (1577 lines)
2. `crates/biomeos-atomic-deploy/src/neural_api_server.rs` (1403 lines)
3. `crates/biomeos-spore/src/logs.rs` (1039 lines)

**Strategy**:

#### neural_executor.rs → 4 modules
```
crates/biomeos-atomic-deploy/src/neural_executor/
  ├── mod.rs           (300 lines - core struct + coordination)
  ├── http.rs          (400 lines - HTTP execution logic)
  ├── storage.rs       (400 lines - storage execution logic)
  ├── ai.rs            (400 lines - AI/Squirrel integration)
  └── tests.rs         (remaining - test helpers)
```

#### neural_api_server.rs → 4 modules
```
crates/biomeos-atomic-deploy/src/neural_api_server/
  ├── mod.rs           (300 lines - server core)
  ├── routes.rs        (350 lines - route definitions)
  ├── handlers.rs      (400 lines - request handlers)
  └── state.rs         (350 lines - state management)
```

#### logs.rs → 4 modules
```
crates/biomeos-spore/src/logs/
  ├── mod.rs           (250 lines - core types)
  ├── session.rs       (300 lines - session management)
  ├── metrics.rs       (250 lines - metrics collection)
  └── query.rs         (250 lines - log querying)
```

**Success Criteria**: 
- All files <1000 lines
- Tests still pass
- Public API unchanged

---

### Days 13-14: TODO Reduction Campaign

**Priority**: 🟢 MEDIUM - Technical debt reduction

**Current**: 99 TODOs  
**Target**: <20 TODOs (critical only)

**Strategy**:
1. **Categorize** all 99 TODOs:
   - 🔴 Critical (must implement)
   - 🟡 Important (should implement)
   - 🟢 Nice-to-have (defer or delete)

2. **Action for each category**:
   - 🔴 Critical: Implement or create tracked issue
   - 🟡 Important: Create tracked issue, document decision
   - 🟢 Nice-to-have: Delete TODO, add to backlog doc

3. **Priority TODOs** (from audit):
   ```
   1. Implement tarpc transport (or document JSON-RPC only decision)
   2. Implement rollback strategy (executor.rs, neural_executor.rs)
   3. Get family seed from secure storage (client.rs)
   4. Use atomic counter for RPC IDs (client.rs)
   5. Complete Songbird/NestGate integration via JSON-RPC
   ```

**Success Criteria**: 
- <20 TODOs remaining
- All remaining TODOs are critical and tracked
- Decisions documented for deferred items

---

## 🔵 PHASE 4: TEST COVERAGE (Week 3 - Days 15-21)

### Days 15-17: Fix Ignored Tests

**Priority**: 🔵 MEDIUM - Validation completeness

**Current**: 14 tests ignored

**Tasks**:
1. Review each ignored test
2. Determine if still relevant
3. Fix underlying issues or update test
4. Remove `#[ignore]` attribute

**Example**:
```rust
// BEFORE:
#[ignore = "Depends on plasmidBin structure - TODO: fix test setup"]
#[tokio::test]
async fn test_spore_incubation() { ... }

// AFTER (fixed):
#[tokio::test]
async fn test_spore_incubation() {
    // Use test fixture:
    let temp_dir = TempDir::new().unwrap();
    setup_test_plasmidbin(&temp_dir).await;
    // ... test logic
}
```

**Success Criteria**: 
- All tests either pass or removed
- Zero ignored tests (or explicit justification documented)

---

### Days 18-20: Increase Coverage to 90%

**Priority**: 🔵 MEDIUM - Quality assurance

**Strategy**:
1. Identify uncovered modules from llvm-cov report
2. Prioritize by criticality:
   - Core (biomeos-core, biomeos-nucleus) → 95%+
   - API (biomeos-api, biomeos-atomic-deploy) → 90%+
   - Utils (biomeos-cli, biomeos-test-utils) → 80%+
3. Write missing tests
4. Focus on integration tests for complex workflows

**Areas likely needing coverage**:
- Error paths (test error handling)
- Edge cases (empty inputs, large inputs, etc.)
- Integration between modules
- Rollback/recovery logic

**Success Criteria**: 
- Overall workspace coverage ≥90%
- All critical modules ≥95%
- Coverage report committed to repo

---

### Day 21: E2E and Chaos Testing

**Priority**: 🔵 MEDIUM - Robustness validation

**Tasks**:
1. Review existing chaos tests
2. Add missing fault injection tests:
   - Network failures
   - Disk full scenarios
   - Process crashes
   - Resource exhaustion
3. Run E2E scenarios:
   - Full graph deployment
   - Multi-primal orchestration
   - Failure and recovery
4. Document test scenarios

**Success Criteria**: 
- E2E tests pass
- Chaos tests pass (or failures are expected and documented)
- Test documentation complete

---

## 📊 SUCCESS METRICS

### Week 1 End (Day 7)
- [ ] All tests compile and pass ✅
- [ ] Zero clippy warnings ✅
- [ ] Zero formatting issues ✅
- [ ] Test coverage baseline established ✅
- [ ] UniBin implemented ✅

### Week 2 End (Day 14)
- [ ] ecoBin compliant (musl builds) ✅
- [ ] No hardcoded ports in production ✅
- [ ] All files <1000 lines ✅
- [ ] TODOs <20 ✅

### Week 3 End (Day 21)
- [ ] Test coverage ≥90% ✅
- [ ] Zero ignored tests ✅
- [ ] E2E tests passing ✅
- [ ] Chaos tests passing ✅
- [ ] Documentation updated ✅

---

## 🎯 FINAL VALIDATION

After completing all phases, run full validation:

```bash
#!/bin/bash
# validate.sh - Full compliance check

set -e

echo "🔍 Phase 1: Build & Tests"
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --workspace --no-fail-fast

echo "🔍 Phase 2: Standards Compliance"
# UniBin check
test -f target/release/biomeos || (echo "❌ UniBin missing"; exit 1)
target/release/biomeos --help | grep -q "SUBCOMMANDS" || (echo "❌ No subcommands"; exit 1)

# ecoBin check
cargo build --release --target x86_64-unknown-linux-musl
cargo tree | grep -E "(openssl-sys|ring|aws-lc-sys)" && (echo "❌ C dependencies found"; exit 1) || echo "✅ No C deps"

echo "🔍 Phase 3: File Organization"
find crates -name "*.rs" -exec wc -l {} \; | awk '$1 > 1000 {print "❌ " $2 " exceeds 1000 lines"; exit 1}'

echo "🔍 Phase 4: Test Coverage"
cargo llvm-cov --all-features --workspace | grep -E "TOTAL.*([0-9]{2}\.[0-9]+%)" | awk '{if ($NF < 90.0) {print "❌ Coverage below 90%"; exit 1}}'

echo "🔍 Phase 5: TODOs"
TODO_COUNT=$(grep -r "TODO\|FIXME\|XXX\|HACK" crates/ --include="*.rs" | wc -l)
test $TODO_COUNT -lt 20 || (echo "❌ Too many TODOs: $TODO_COUNT"; exit 1)

echo "✅ ALL VALIDATION PASSED!"
echo "🎉 BiomeOS is fully compliant!"
```

**Success Criteria**: All checks pass, zero errors

---

## 📈 BEYOND COMPLIANCE (Future)

After achieving baseline compliance, consider:

1. **Performance Optimization**
   - Profile critical paths
   - Implement zero-copy where beneficial
   - Benchmark against targets

2. **Advanced Testing**
   - Property-based testing (proptest)
   - Fuzzing critical parsers
   - Long-running soak tests

3. **Documentation Excellence**
   - Architecture decision records (ADRs)
   - API documentation examples
   - Tutorial series

4. **Ecosystem Integration**
   - Validate against all Phase 1 primals
   - Cross-primal integration tests
   - End-to-end ecosystem demos

---

## 🚀 LET'S GO!

**Start Date**: [To be determined]  
**Target Completion**: 21 days from start  
**Review Date**: Day 22 (final validation)

**Team**: [Assign resources]  
**Communication**: [Daily standups, weekly reviews]  
**Tracking**: [Issue tracker, project board]

---

🦀🧬✨ **BiomeOS: From Good to Great in 3 Weeks!** ✨🧬🦀

**Questions? Issues? Blockers?**  
→ Discuss in wateringHole  
→ Tag relevant primal teams  
→ Document decisions in ADRs

