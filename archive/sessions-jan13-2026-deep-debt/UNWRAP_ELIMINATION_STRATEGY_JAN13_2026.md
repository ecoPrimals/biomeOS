# 🛡️ Unwrap/Expect Elimination Strategy

**Date**: January 13, 2026  
**Current State**: 322 unwrap/expect calls in production code  
**Target**: < 50 in production code (exceptions allowed for infallible operations)  
**Estimated Effort**: 8-12 hours  
**Priority**: HIGH - Production reliability

---

## 📊 Current Statistics

### By Crate

| Crate | Count | Priority |
|-------|-------|----------|
| **biomeos-core** | 211 | HIGH |
| **biomeos-graph** | 69 | MEDIUM |
| **biomeos-atomic-deploy** | 42 | HIGH |
| **Other crates** | ~1000+ | LOW (mostly tests) |

### Analysis

From spot checks:
- ✅ **Most test code** - Acceptable to use `.unwrap()` in tests
- ⚠️ **Some production code** - Needs proper error handling
- ⚠️ **Some infallible operations** - Could use `.unwrap()` with documentation

---

## 🎯 Strategy: Categorize & Evolve

### Category 1: Legitimate Test Code ✅

**Keep as-is** - `.unwrap()` is acceptable in tests

```rust
#[test]
fn test_validation() {
    let report = validator.validate(&graph).unwrap(); // ✅ OK in tests
    assert!(report.is_valid());
}
```

**Why**: Tests should fail fast and clearly when something is wrong.

### Category 2: Truly Infallible Operations ✅

**Document and keep** - Add comment explaining why it's safe

```rust
// Example: String parsing that's guaranteed by type system
let capability: Capability = "storage".parse().unwrap(); // ✅ FromStr impl is Infallible

// Better: Document why
/// Parse capability (infallible - all strings map to Capability::Custom if unknown)
let capability: Capability = "storage".parse()
    .expect("Capability parsing is infallible");
```

**When allowed**:
- `FromStr` with `Err = Infallible`
- Static string operations that can't fail
- Type conversions guaranteed by construction

### Category 3: Error Propagation Needed ⚠️

**Replace with `?` operator**

```rust
// Before
pub fn load_config(path: &Path) -> Config {
    let content = std::fs::read_to_string(path).unwrap();
    serde_json::from_str(&content).unwrap()
}

// After
pub fn load_config(path: &Path) -> Result<Config> {
    let content = std::fs::read_to_string(path)
        .context("Failed to read config file")?;
    serde_json::from_str(&content)
        .context("Failed to parse config JSON")
}
```

### Category 4: Default Values ⚠️

**Use `unwrap_or`, `unwrap_or_else`, `unwrap_or_default`**

```rust
// Before
let port = env::var("PORT").unwrap();

// After  
let port = env::var("PORT").unwrap_or_else(|_| "8080".to_string());

// Or with Result propagation
let port = env::var("PORT")
    .context("PORT environment variable not set")?;
```

### Category 5: Early Validation ⚠️

**Validate at boundaries, propagate internally**

```rust
// Before (validation scattered)
pub fn process(data: &str) -> String {
    let json: Value = serde_json::from_str(data).unwrap();
    let name = json["name"].as_str().unwrap();
    name.to_string()
}

// After (validate at boundary)
pub fn process(data: &str) -> Result<String> {
    let json: Value = serde_json::from_str(data)
        .context("Invalid JSON input")?;
    
    let name = json["name"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Missing 'name' field"))?;
    
    Ok(name.to_string())
}
```

---

## 🔧 Refactoring Patterns

### Pattern 1: Chained Operations

```rust
// Before
let result = operation1()
    .unwrap()
    .operation2()
    .unwrap()
    .operation3()
    .unwrap();

// After
let result = operation1()
    .context("Operation 1 failed")?
    .operation2()
    .context("Operation 2 failed")?
    .operation3()
    .context("Operation 3 failed")?;
```

### Pattern 2: Option to Result

```rust
// Before
let value = map.get("key").unwrap();

// After
let value = map.get("key")
    .ok_or_else(|| anyhow::anyhow!("Key 'key' not found"))?;

// Or with context
let value = map.get("key")
    .context("Required configuration key 'key' is missing")?;
```

### Pattern 3: Panic with Context

```rust
// Before
let value = some_operation().unwrap();

// After (if you REALLY need to panic)
let value = some_operation()
    .expect("Critical invariant violated: some_operation should never fail here. \
             This indicates a bug in the initialization code.");
```

### Pattern 4: Infallible with Documentation

```rust
// Before
let id = Uuid::new_v4().to_string().parse().unwrap();

// After
/// Parse UUID string (infallible - UUID::to_string() always produces valid strings)
let id = Uuid::new_v4()
    .to_string()
    .parse()
    .expect("UUID string is always valid");
```

---

## 📋 Execution Plan

### Phase 1: Audit & Categorize (2 hours)

1. **Scan each file** and categorize unwraps:
   ```bash
   # Find all non-test unwraps
   rg '\.unwrap\(\)' crates/biomeos-core/src --type rust
   rg '\.expect\(' crates/biomeos-core/src --type rust
   ```

2. **Create spreadsheet**:
   - File path
   - Line number
   - Category (test, infallible, needs-fix)
   - Priority (high, medium, low)

### Phase 2: biomeos-atomic-deploy (2 hours)

**Priority**: HIGH - Production deployment code

Target: 42 instances → < 5

Focus areas:
- `orchestrator.rs` - 6 instances
- `primal_launcher.rs` - 18 instances
- `health_check.rs` - 17 instances

### Phase 3: biomeos-core (4-6 hours)

**Priority**: HIGH - Core infrastructure

Target: 211 instances → < 30

Focus on:
- Client code (50+ instances)
- Discovery code (30+ instances)
- Configuration loading (20+ instances)

### Phase 4: biomeos-graph (2 hours)

**Priority**: MEDIUM - Graph execution

Target: 69 instances → < 10

Most are in tests (acceptable), fix production code only.

---

## 🎓 Best Practices

### DO ✅

1. **Use `?` operator** for error propagation
2. **Add context** with `.context()` or `.with_context()`
3. **Document infallible operations**
4. **Validate at API boundaries**
5. **Provide helpful error messages**
6. **Keep `.unwrap()` in test code**

### DON'T ❌

1. **Don't use `.unwrap()` in production** without documentation
2. **Don't hide errors** - let them propagate
3. **Don't use generic error messages** - be specific
4. **Don't assume "this can't fail"** - it can
5. **Don't panic in library code** - return Result

---

## 📊 Success Metrics

### Before
```rust
// 322 unwrap/expect calls
// Potential panics in production
// Hard to debug when things fail
```

### After
```rust
// < 50 unwrap/expect calls (documented)
// All errors propagated with context
// Clear error messages
// Production-ready error handling
```

### Testing
```bash
# Add clippy lint
#![deny(clippy::unwrap_used)]  // In production crates
#![warn(clippy::expect_used)]  // Warn on expect too

# Run clippy
cargo clippy --workspace -- -D clippy::unwrap_used
```

---

## 🔍 Example Refactoring

### File: `crates/biomeos-atomic-deploy/src/orchestrator.rs`

#### Before
```rust
pub fn new(usb_seed_path: PathBuf) -> Self {
    let deployment_mode = DeploymentMode::detect().unwrap_or_else(|_| {
        DeploymentMode::SiblingSpore {
            host_os: biomeos_core::deployment_mode::HostOS::Linux {
                distro: "Unknown".to_string(),
            },
            install_dir: PathBuf::from("/tmp/biomeos"),
            isolation: biomeos_core::deployment_mode::IsolationLevel::Shared,
        }
    });
    
    Self {
        usb_seed_path,
        family_id: "nat0".to_string(),  // Hardcoded!
        deployment_batch: chrono::Utc::now().format("%Y%m%d").to_string(),
        binary_dir: PathBuf::from("/home/eastgate/Development/ecoPrimals/phase2/biomeOS/plasmidBin"),  // Hardcoded!
        runtime_dir: std::env::var("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                use nix::unistd::getuid;
                PathBuf::from(format!("/run/user/{}", getuid()))
            }),
        deployment_mode,
        neural_api_enabled: false,
        neural_api_endpoint: None,
    }
}
```

#### After (Full Evolution)
```rust
pub fn new(usb_seed_path: PathBuf) -> Result<Self> {
    // Detect deployment mode with proper error handling
    let deployment_mode = DeploymentMode::detect()
        .context("Failed to detect deployment mode")?;
    
    // Load family ID from seed file
    let family_id = Self::read_family_id(&usb_seed_path)
        .context("Failed to read family ID from USB seed")?;
    
    // Discover binary directory (capability-based)
    let binary_dir = Self::discover_binary_dir()
        .context("Failed to discover primal binary directory")?;
    
    // Get runtime directory (XDG-compliant)
    let runtime_dir = Self::get_runtime_dir()
        .context("Failed to determine runtime directory")?;
    
    Ok(Self {
        usb_seed_path,
        family_id,
        deployment_batch: chrono::Utc::now().format("%Y%m%d").to_string(),
        binary_dir,
        runtime_dir,
        deployment_mode,
        neural_api_enabled: false,
        neural_api_endpoint: None,
    })
}

fn read_family_id(seed_path: &Path) -> Result<String> {
    std::fs::read_to_string(seed_path.join("family_id"))
        .context("Family ID file not found in USB seed")
}

fn discover_binary_dir() -> Result<PathBuf> {
    // Try environment variable first
    if let Ok(dir) = std::env::var("BIOMEOS_BINARY_DIR") {
        return Ok(PathBuf::from(dir));
    }
    
    // Try common locations
    let candidates = vec![
        PathBuf::from("./plasmidBin"),
        PathBuf::from("/usr/local/bin/biomeos"),
        PathBuf::from("/opt/biomeos/bin"),
    ];
    
    for path in candidates {
        if path.exists() {
            return Ok(path);
        }
    }
    
    anyhow::bail!("Could not find biomeOS binary directory. Set BIOMEOS_BINARY_DIR environment variable.")
}

fn get_runtime_dir() -> Result<PathBuf> {
    std::env::var("XDG_RUNTIME_DIR")
        .map(PathBuf::from)
        .or_else(|_| {
            use nix::unistd::getuid;
            let path = PathBuf::from(format!("/run/user/{}", getuid()));
            if path.exists() {
                Ok(path)
            } else {
                anyhow::bail!("Runtime directory does not exist: {}", path.display())
            }
        })
}
```

**Benefits**:
- ✅ No unwrap/expect
- ✅ Clear error messages
- ✅ Capability-based (no hardcoding)
- ✅ Proper error propagation
- ✅ Easier to test
- ✅ Better user experience (helpful errors)

---

## ✅ Quick Wins (1 hour)

### Fix These First

1. **Configuration Loading** (~30 instances)
   - All config file reads
   - All environment variable reads
   - All TOML/JSON parsing

2. **Discovery Code** (~40 instances)
   - Socket path discovery
   - Primal endpoint discovery
   - Capability lookups

3. **Client Creation** (~30 instances)
   - HTTP client creation
   - JSON-RPC client creation
   - Transport initialization

---

## 📚 Resources

### Crates to Use

```toml
[dependencies]
anyhow = "1.0"        # For error handling with context
thiserror = "1.0"     # For custom error types
```

### Patterns

```rust
use anyhow::{Context, Result};

// Add context to errors
.context("What failed")?

// Lazy context (for expensive operations)
.with_context(|| format!("Failed on item {}", id))?

// Convert Option to Result
.ok_or_else(|| anyhow::anyhow!("Not found"))?
```

---

## ✅ Conclusion

**Current**: 322 unwrap/expect calls  
**Target**: < 50 (documented)  
**Effort**: 8-12 hours  
**Impact**: HIGH - Production reliability

**Priority Order**:
1. biomeos-atomic-deploy (production deployment)
2. biomeos-core clients (user-facing APIs)
3. biomeos-core discovery (critical infrastructure)
4. Everything else

**Next Steps**:
1. Create audit spreadsheet
2. Fix atomic-deploy (2 hours)
3. Fix core clients (3 hours)
4. Fix core discovery (2 hours)
5. Add clippy lint `deny(unwrap_used)`

---

**"Different orders of the same architecture - now with production-grade error handling."** 🍄🐸✨

