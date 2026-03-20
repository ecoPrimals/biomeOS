# CI/CD Pipeline Documentation

**Last Updated:** March 20, 2026  
**Status:** Production-ready, comprehensive quality enforcement

---

## 📊 **Pipeline Overview**

biomeOS uses GitHub Actions for continuous integration and delivery, with two main workflows:

### **1. ci.yml - Comprehensive CI/CD Pipeline**

**Triggers:**
- Push to `master`, `main`, or `develop` branches
- Pull requests to these branches

**Jobs:** 10 parallel jobs for comprehensive quality checks

#### **Job 1: Lint & Format Check**
- Runs `cargo fmt --all -- --check`
- Runs `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- Checks documentation with `cargo doc --workspace --no-deps --all-features` (`RUSTDOCFLAGS=-D warnings`)
- **Purpose:** Enforce code style and catch common issues

#### **Job 2: Build**
- Builds on Ubuntu and macOS
- Tests both debug and release profiles
- **Purpose:** Ensure cross-platform compatibility

#### **Job 3: Test Suite**
- Unit tests (`cargo test --workspace --lib --all-features`)
- Integration tests (`cargo test --workspace --test '*' --all-features`)
- Doc tests (`cargo test --workspace --doc --all-features`)
- **Purpose:** Validate functionality

#### **Job 4: Code Coverage**
- Uses `cargo-llvm-cov` for coverage generation (`cargo llvm-cov --workspace --lcov --output-path lcov.info`)
- Enforces a minimum line coverage threshold (`cargo llvm-cov --workspace --fail-under-lines 85 --no-run`)
- Uploads to Codecov
- Archives HTML coverage report (`cargo llvm-cov --workspace --html`)
- **Purpose:** Track test coverage trends

#### **Job 5: Security Audit**
- Runs `cargo audit` for known vulnerabilities
- **Purpose:** Identify security issues in dependencies

#### **Job 6: Dependency Check**
- Runs `cargo deny check`
- **Purpose:** Enforce dependency policies

#### **Job 7: File Size Compliance**
- Reports files exceeding 1000 line guideline
- **Blocking** when any file exceeds the guideline (job exits with failure)
- **Purpose:** Encourage maintainable file sizes

**Philosophy:**
```
1000 lines is a GUIDELINE, not a hard limit.

Well-structured code may appropriately exceed this when:
✅ Single clear responsibility
✅ Logic delegated to handlers
✅ Well-documented structure
✅ No code duplication
✅ Async coordination (inherently verbose)

Examples: neural_api_server.rs, orchestrator.rs, executor.rs
```

#### **Job 8: Standards Compliance**

**Check 1: TODO/FIXME markers**
- Scans for technical debt markers
- **Blocking** - fails CI if any are found

**Check 2: panic!() in production**
- Checks production code (excludes tests)
- **Blocking** - fails CI if any are found

**Important Notes:**
- ✅ `panic!()` in **test code is CORRECT** - makes failures immediately obvious
- ✅ `.unwrap()` in **test code is IDIOMATIC** - proper Rust testing practice
- ❌ `panic!()` in **production code** should return `Result<T, E>`

**Check 3: Unsafe code**
- **Zero tolerance** for `unsafe` blocks
- **Blocking** - fails CI if found
- biomeOS maintains 100% safe Rust

#### **Job 9: Performance Benchmarks**
- Runs on main/master pushes only
- Tracks performance regressions
- **Non-blocking** - informational

#### **Job 10: Release Readiness Check**
- Runs after **lint**, **build**, **test**, **coverage**, and **security** jobs complete (push to `master`/`main` only; does not wait for dependency check, file size, standards, or benchmarks)
- Generates release notes
- Validates version consistency (placeholder step in workflow)

---

### **2. quality-gates.yml - PR Quality Gates**

**Triggers:**
- Pull requests only
- Runs on changed files only (efficient)

**Features:**
- ✅ Format checking
- ✅ Linting enforcement
- ✅ Test validation
- ✅ Zero unsafe code enforcement
- ✅ File size monitoring
- ✅ Automated PR comment with quality report

**Sample PR Comment:**
```markdown
## 🔍 Quality Gate Report

✅ **Formatting:** Passed
✅ **Linting:** Passed  
✅ **Tests:** Passed
✅ **Unsafe Code:** Zero (maintained)
✅ **Standards:** Compliant

### Standards Verified:
- ecoBin Architecture (Pure Rust)
- UniBin Architecture (Single binary)
- Zero unsafe code policy
- File size guidelines (<1000 lines recommended)
- Comprehensive documentation

**Status:** Ready for review ✨
```

---

## 🎓 **Philosophy & Best Practices**

### **Guidelines vs Laws**

biomeOS CI/CD distinguishes between:

**Non-blocking or best-effort in `ci.yml`:**
- Benchmarks job uses `continue-on-error: true` (a failing bench run does not fail the workflow)
- Codecov upload uses `fail_ci_if_error: false` (upload problems do not fail CI)

**Blocking (failing job fails the workflow):**
- Format check, Clippy (all targets), and doc warnings (`-D warnings` on rustdoc)
- Build (debug and release) and full test suite
- Coverage below the enforced line threshold (85%) and `cargo audit` / `cargo deny check` failures
- File size job when any file exceeds the guideline
- Standards job (TODO/FIXME markers, `panic!()` in production paths, any `unsafe` block)
- Zero unsafe code remains a hard policy

### **Modern Rust Patterns**

The CI/CD acknowledges and encourages modern Rust idioms:

#### **✅ Correct Practices:**
- `unwrap()` in tests (immediate failure on unexpected error)
- `panic!()` in tests (clear test failure points)
- `Result<T, E>` in production code
- `anyhow::Context` for error enrichment
- Async patterns with proper error handling

#### **❌ Anti-Patterns:**
- `unsafe` code (zero tolerance)
- `panic!()` in production code
- Missing error context
- Hardcoded values (use env vars or discovery)

### **File Size Philosophy**

From Deep Debt Analysis (Jan 30, 2026):

> "Don't refactor well-structured code just to meet arbitrary metrics."

**When file size is appropriate:**
1. Async connection handling (event loops are verbose)
2. Request routing with pattern matching
3. Bootstrap coordination logic
4. Proper error handling (adds lines, but correct)

**When to refactor:**
1. Multiple unrelated responsibilities
2. Code duplication
3. Poor cohesion
4. Hard to test or understand

---

## 📈 **Quality Metrics**

### **Current State (March 20, 2026)**

- **Grade:** A++ (100/100)
- **Test Pass Rate:** 100% (6,959 tests)
- **Unsafe Code:** 0 blocks
- **Test Coverage:** 89.84% line / 90.74% function (llvm-cov)
- **Clippy Warnings:** 0 (pedantic + nursery)
- **Format Violations:** 0
- **Files >1000 LOC:** 0

### **CI/CD Coverage**

| Check | Status | Blocking |
|-------|--------|----------|
| Formatting | ✅ Enforced | Yes |
| Linting | ✅ Enforced | Yes |
| Tests | ✅ Enforced | Yes |
| Unsafe Code | ✅ Enforced | Yes |
| Dependency policy (`cargo deny`) | ✅ Enforced | Yes |
| Security Audit | ✅ Enforced | Yes (on `cargo audit` failure) |
| Coverage | ✅ Enforced | Yes (min. 85% lines) |
| File Size | ✅ Enforced | Yes (when over guideline) |
| Benchmarks | ✅ Tracked | No (`continue-on-error`) |

---

## 🚀 **Adding New Checks**

### **Example: Add a new quality check**

```yaml
- name: Check for specific pattern
  run: |
    echo "Checking for anti-pattern..."
    MATCHES=$(grep -rn "anti-pattern" crates/ src/ || true)
    if [ -n "$MATCHES" ]; then
      echo "❌ Found anti-pattern:"
      echo "$MATCHES"
      exit 1  # Fail CI
    fi
```

### **Example: Add caching**

```yaml
- name: Cache cargo
  uses: actions/cache@v4
  with:
    path: |
      ~/.cargo/registry
      ~/.cargo/git
      target
    key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
```

---

## 🔧 **Local Testing**

Run the same checks locally before pushing:

```bash
# Format check
cargo fmt --all -- --check

# Linting
cargo clippy --workspace --all-targets --all-features -- -D warnings

# Tests
cargo test --workspace

# Coverage
cargo install cargo-llvm-cov
cargo llvm-cov --workspace --html

# Security audit
cargo install cargo-audit
cargo audit

# Dependency check
cargo install cargo-deny
cargo deny check
```

---

## 📊 **Monitoring**

### **GitHub Actions Dashboard**

View workflow runs: `https://github.com/{org}/{repo}/actions`

### **Codecov Dashboard**

View coverage trends: `https://codecov.io/gh/{org}/{repo}`

### **Artifacts**

Each workflow run archives:
- Coverage reports (HTML)
- Release notes
- Benchmark results

---

## 🎯 **Continuous Improvement**

The CI/CD pipeline evolves with the codebase:

**Recent Enhancements (March 20, 2026):**
1. Coverage threshold raised from 75% to 85% (actual: 89.84%)
2. File size check is now blocking (0 files over limit)
3. `#[allow]` migrated to `#[expect(reason)]` workspace-wide
4. All flaky mock tests hardened (flush + shutdown + case-insensitive matching)
5. CWD-dependent tests evolved to env-based discovery

---

## References

- **Current Status:** `../CURRENT_STATUS.md`
- **Contributing:** `../CONTRIBUTING.md`
- **Standards:** `ecoPrimals/wateringHole/STANDARDS_AND_EXPECTATIONS.md`

---

**Status:** Production-ready CI/CD with comprehensive quality enforcement

**Philosophy:** Guidelines inform, laws enforce. Context matters more than metrics.
