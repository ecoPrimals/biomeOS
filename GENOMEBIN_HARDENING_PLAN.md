# genomeBin Deployment Hardening Plan
*Making Deployments Deterministic & Production-Ready*

## 🎯 Objective

Harden genomeBin wrapper scripts for deterministic, reliable deployments with clear evolution path to Rust.

---

## 📋 Current State Analysis

### ✅ What's Working Well
1. **POSIX sh compatibility** - Works across bash, dash, ash, Android sh
2. **Architecture detection** - Correctly maps x86_64, aarch64, arm64
3. **Platform detection** - Identifies Linux, Android, macOS
4. **Self-extraction** - Archive marker + tail works reliably
5. **User/root handling** - Respects privilege level for install paths

### ⚠️  Issues Identified

#### 1. **Non-Deterministic Behavior**
- `echo -e` not POSIX (works on bash/dash but not all sh)
- `mktemp -d` implementation varies across platforms
- `trap` cleanup may not fire on all signals
- Exit codes not always checked
- Some commands succeed silently even on partial failure

#### 2. **Error Handling Gaps**
- Missing validation after `mkdir -p`
- No checksum verification of extracted binaries
- Partial extraction failures not detected
- No rollback on failed installation

#### 3. **Idempotency Issues**
- Running twice may leave partial state
- No detection of previous installation
- Overwrites without version checking
- No upgrade vs fresh install distinction

#### 4. **Platform-Specific Edge Cases**
- Android `/data/local/tmp` may be noexec
- macOS `$HOME/Library` may not exist
- Linux `$HOME/.local` may not be in PATH
- Symlink creation assumes `/usr/local/bin` exists

#### 5. **Security Concerns**
- No verification of genomeBin integrity
- Temp directory predictable path risk
- No secure permission setting before writing sensitive data
- Race condition in temp dir creation

#### 6. **Logging & Observability**
- No structured logging
- No deployment metrics
- No success/failure reporting mechanism
- Difficult to debug deployment issues remotely

---

## 🔧 Hardening Strategy

### Phase 1: Shell Script Hardening (Immediate)

**Priority: High**  
**Timeline: 1-2 days**  
**Approach: Fix critical issues while maintaining POSIX compatibility**

#### 1.1 Strict Error Handling
```bash
# Current: set -eu
# Hardened:
set -euo pipefail  # bash
set -eu            # POSIX sh (no pipefail)

# Add comprehensive error checking
check_command() {
    if ! command -v "$1" >/dev/null 2>&1; then
        log_error "Required command not found: $1"
        exit 1
    fi
}

# Check every critical command
check_command awk
check_command tar
check_command tail

# Validate after every critical operation
mkdir -p "$INSTALL_DIR" || {
    log_error "Failed to create install directory"
    exit 1
}

[ -d "$INSTALL_DIR" ] || {
    log_error "Install directory does not exist after creation"
    exit 1
}
```

#### 1.2 POSIX-Compliant Logging
```bash
# Replace echo -e with printf for true POSIX compatibility
log_info() { printf "[INFO] %s\n" "$*"; }
log_success() { printf "[SUCCESS] %s\n" "$*"; }
log_warn() { printf "[WARN] %s\n" "$*"; }
log_error() { printf "[ERROR] %s\n" "$*" >&2; }

# Colors only if terminal supports it
if [ -t 1 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    # ...
else
    RED=''
    GREEN=''
    # ...
fi
```

#### 1.3 Deterministic Temp Directory
```bash
# Replace: TEMP_DIR=$(mktemp -d)
# With:
create_secure_tempdir() {
    # Try mktemp first (most secure)
    if command -v mktemp >/dev/null 2>&1; then
        TEMP_DIR=$(mktemp -d 2>/dev/null) && [ -d "$TEMP_DIR" ] && return 0
    fi
    
    # Fallback for systems without mktemp
    TEMP_DIR="/tmp/genome-$$-$(date +%s)"
    mkdir -p "$TEMP_DIR" 2>/dev/null || {
        log_error "Failed to create temporary directory"
        return 1
    }
    chmod 700 "$TEMP_DIR"
}

create_secure_tempdir || exit 1
```

#### 1.4 Comprehensive Trap Handling
```bash
# Cleanup on all possible exit conditions
cleanup() {
    EXIT_CODE=$?
    if [ -n "${TEMP_DIR:-}" ] && [ -d "$TEMP_DIR" ]; then
        rm -rf "$TEMP_DIR" 2>/dev/null || true
    fi
    
    if [ $EXIT_CODE -ne 0 ]; then
        log_error "Deployment failed with exit code: $EXIT_CODE"
        # Rollback partial installation
        if [ -n "${INSTALL_DIR:-}" ] && [ -d "$INSTALL_DIR.backup" ]; then
            log_info "Rolling back to previous installation..."
            rm -rf "$INSTALL_DIR"
            mv "$INSTALL_DIR.backup" "$INSTALL_DIR"
        fi
    fi
    
    exit $EXIT_CODE
}

trap cleanup EXIT INT TERM HUP QUIT
```

#### 1.5 Checksum Verification
```bash
# Add checksum to metadata
EXPECTED_SHA256_x86_64="abc123..."
EXPECTED_SHA256_aarch64="def456..."

verify_binary() {
    BINARY="$1"
    EXPECTED="$2"
    
    if command -v sha256sum >/dev/null 2>&1; then
        ACTUAL=$(sha256sum "$BINARY" | awk '{print $1}')
    elif command -v shasum >/dev/null 2>&1; then
        ACTUAL=$(shasum -a 256 "$BINARY" | awk '{print $1}')
    else
        log_warn "No checksum tool available, skipping verification"
        return 0
    fi
    
    if [ "$ACTUAL" != "$EXPECTED" ]; then
        log_error "Checksum mismatch for $BINARY"
        log_error "  Expected: $EXPECTED"
        log_error "  Actual:   $ACTUAL"
        return 1
    fi
    
    log_success "Checksum verified for $BINARY"
    return 0
}
```

#### 1.6 Idempotent Installation
```bash
# Detect existing installation
detect_existing_installation() {
    if [ -d "$INSTALL_DIR" ] && [ -x "$INSTALL_DIR/${GENOME_NAME}" ]; then
        EXISTING_VERSION=$("$INSTALL_DIR/${GENOME_NAME}" --version 2>/dev/null | head -1 || echo "unknown")
        log_info "Existing installation detected: $EXISTING_VERSION"
        log_info "Current genomeBin version: $GENOME_VERSION"
        
        # Ask for confirmation (or use --force flag)
        if [ "${FORCE_INSTALL:-0}" != "1" ]; then
            log_warn "Use --force to overwrite existing installation"
            exit 0
        fi
        
        # Backup existing installation
        log_info "Backing up existing installation..."
        rm -rf "$INSTALL_DIR.backup" 2>/dev/null || true
        cp -r "$INSTALL_DIR" "$INSTALL_DIR.backup"
        log_success "Backup created: $INSTALL_DIR.backup"
    fi
}
```

#### 1.7 Platform-Specific Hardening
```bash
# Android: Check for noexec mount
check_android_permissions() {
    if [ "$PLATFORM" = "android" ]; then
        # Test if we can actually execute from install dir
        TEST_FILE="$INSTALL_DIR/.test-exec-$$"
        printf '#!/bin/sh\nexit 0\n' > "$TEST_FILE"
        chmod +x "$TEST_FILE"
        
        if ! "$TEST_FILE" 2>/dev/null; then
            log_error "Installation directory is mounted noexec!"
            log_error "Try: adb shell 'mount -o remount,exec /data'"
            rm -f "$TEST_FILE"
            exit 1
        fi
        
        rm -f "$TEST_FILE"
        log_success "Execution permissions verified"
    fi
}
```

#### 1.8 Structured Deployment Report
```bash
# Generate deployment report
generate_deployment_report() {
    REPORT_FILE="$INSTALL_DIR/.deployment-report"
    
    cat > "$REPORT_FILE" << EOF
{
  "genome_name": "$GENOME_NAME",
  "genome_version": "$GENOME_VERSION",
  "architecture": "$BINARY_ARCH",
  "platform": "$PLATFORM",
  "install_dir": "$INSTALL_DIR",
  "install_time": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "install_user": "$(id -un)",
  "install_uid": "$(id -u)",
  "hostname": "$(hostname)",
  "binaries": [
$(ls -1 "$INSTALL_DIR" | grep -v '^\.' | while read -r bin; do
    printf '    {"name": "%s", "size": %d}' "$bin" "$(stat -c%s "$INSTALL_DIR/$bin" 2>/dev/null || stat -f%z "$INSTALL_DIR/$bin" 2>/dev/null)"
    [ "$bin" != "$(ls -1 "$INSTALL_DIR" | grep -v '^\.' | tail -1)" ] && printf ","
    printf "\n"
done)
  ],
  "success": true
}
EOF
    
    log_info "Deployment report saved: $REPORT_FILE"
}
```

---

### Phase 2: Rust Evolution (1-2 weeks)

**Priority: Medium**  
**Timeline: After shell hardening complete**  
**Approach: Leverage existing `genome-deploy` crate**

#### 2.1 Feature Parity
```rust
// crates/genome-deploy/src/hardened.rs

pub struct HardenedDeployer {
    genome_path: PathBuf,
    install_dir: Option<PathBuf>,
    force: bool,
    checksum_verify: bool,
    backup_existing: bool,
}

impl HardenedDeployer {
    pub fn deploy(&self) -> Result<DeploymentReport> {
        // 1. Detect existing installation
        self.detect_existing()?;
        
        // 2. Backup if exists
        if self.backup_existing {
            self.create_backup()?;
        }
        
        // 3. Extract with checksum verification
        let extracted = self.extract_with_verification()?;
        
        // 4. Validate extracted binaries
        self.validate_binaries(&extracted)?;
        
        // 5. Install atomically (temp + rename)
        self.install_atomic(&extracted)?;
        
        // 6. Run health checks
        self.verify_installation()?;
        
        // 7. Generate report
        Ok(self.generate_report())
    }
    
    fn extract_with_verification(&self) -> Result<ExtractedBinaries> {
        // SHA-256 verification
        // Platform-specific extraction
        // Error recovery
    }
    
    fn install_atomic(&self, binaries: &ExtractedBinaries) -> Result<()> {
        // Install to temp location first
        let temp_install = self.install_dir.with_extension(".tmp");
        
        // Install binaries
        self.copy_binaries(binaries, &temp_install)?;
        
        // Set permissions
        self.set_permissions(&temp_install)?;
        
        // Atomic rename
        fs::rename(&temp_install, &self.install_dir)?;
        
        Ok(())
    }
}

#[derive(Serialize)]
pub struct DeploymentReport {
    genome_name: String,
    genome_version: String,
    architecture: String,
    platform: String,
    install_dir: PathBuf,
    install_time: DateTime<Utc>,
    binaries: Vec<BinaryInfo>,
    checksums_verified: bool,
    health_check_passed: bool,
    success: bool,
}
```

#### 2.2 Advanced Features (Rust-only)
```rust
// Concurrent extraction for large genomes
async fn extract_concurrent(&self) -> Result<()> {
    let tasks = vec![
        extract_arch("x86_64"),
        extract_arch("aarch64"),
    ];
    
    futures::future::try_join_all(tasks).await?;
    Ok(())
}

// Progress reporting
pub fn deploy_with_progress(&self, progress: ProgressBar) -> Result<()> {
    progress.set_message("Extracting...");
    self.extract()?;
    progress.inc(33);
    
    progress.set_message("Installing...");
    self.install()?;
    progress.inc(33);
    
    progress.set_message("Verifying...");
    self.verify()?;
    progress.inc(34);
    
    progress.finish_with_message("Complete!");
    Ok(())
}

// Rollback capability
pub fn rollback(&self) -> Result<()> {
    let backup = self.install_dir.with_extension(".backup");
    if backup.exists() {
        fs::remove_dir_all(&self.install_dir)?;
        fs::rename(&backup, &self.install_dir)?;
        Ok(())
    } else {
        Err(anyhow!("No backup found"))
    }
}
```

---

## 📊 Testing Strategy

### Unit Tests (Rust)
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_checksum_verification() {
        let deployer = HardenedDeployer::new("test.genome");
        assert!(deployer.verify_checksum(b"test", "expected").is_err());
    }
    
    #[test]
    fn test_atomic_install() {
        // Test atomic rename semantics
    }
    
    #[test]
    fn test_rollback() {
        // Test rollback on failure
    }
}
```

### Integration Tests
```bash
#!/bin/bash
# test-hardened-deployment.sh

test_idempotency() {
    ./beardog.genome
    ./beardog.genome  # Should succeed without error
}

test_rollback() {
    ./beardog.genome
    # Corrupt installation
    rm $HOME/.local/beardog/beardog
    # Deploy again should restore
    ./beardog.genome --force
    # Verify works
    $HOME/.local/beardog/beardog --version
}

test_noexec_detection() {
    # Mock noexec mount
    # Should fail with clear error
}
```

---

## 🎯 Implementation Roadmap

### Week 1: Shell Hardening
- [ ] Day 1-2: Implement error handling improvements
- [ ] Day 3: Add checksum verification
- [ ] Day 4: Implement idempotency
- [ ] Day 5: Platform-specific hardening
- [ ] Day 6-7: Testing & validation

### Week 2: Rust Evolution
- [ ] Day 1-2: Port shell logic to Rust
- [ ] Day 3-4: Add advanced features
- [ ] Day 5: Integration testing
- [ ] Day 6-7: Documentation & handoff

---

## 📈 Success Metrics

### Determinism
- ✅ Same input → Same output (100%)
- ✅ Idempotent (multiple runs = same state)
- ✅ Predictable error messages
- ✅ Reproducible deployments

### Reliability
- ✅ 99.9% success rate in testing
- ✅ Rollback on any failure
- ✅ No partial state left on error
- ✅ Clear error reporting

### Security
- ✅ Checksum verification
- ✅ Secure temp directory creation
- ✅ Permission validation
- ✅ No race conditions

### Observability
- ✅ Structured deployment reports
- ✅ Machine-readable logs
- ✅ Deployment metrics
- ✅ Debug information available

---

## 🚀 Next Steps

1. **Review this plan** with team
2. **Create hardened template** for beardog.genome
3. **Test thoroughly** on all platforms
4. **Replicate to all primals** (6 genomeBins)
5. **Begin Rust evolution** of genome-deploy
6. **Document patterns** for future genomeBins

---

*Plan Created: January 31, 2026*  
*Status: Ready for Implementation*  
*Approach: Shell first, Rust evolution*  
*Timeline: 2 weeks to production-ready*
