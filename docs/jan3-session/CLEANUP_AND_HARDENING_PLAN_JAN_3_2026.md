# 🧹 Cleanup & Hardening Plan - Production Ready biomeOS

**Date**: January 3, 2026  
**Status**: Moving from "works" to "production-ready, validated, deterministic"  
**Goal**: Clean ecosystem + robust biomeOS + multi-family validation

---

## 🎯 Objectives

1. **Cleanup**: Remove old binaries, test artifacts, obsolete code
2. **Harden biomeOS**: Evolve to maintain primal interactions robustly
3. **Validate**: Test with multiple family seeds to prove determinism
4. **Document**: Production-ready state with clear validation results

---

## 📋 Phase 1: Comprehensive Cleanup

### Old Binaries to Remove

**Songbird** (`/home/eastgate/Development/ecoPrimals/primalBins/`):
- ❌ `songbird-orchestrator-v3.3-tested` (superseded by v3.6)
- ❌ `songbird-orchestrator-v3.4-adaptive` (intermediate version)
- ❌ `songbird-orchestrator-v3.5-serialization` (intermediate version)
- ✅ **KEEP**: `songbird-orchestrator-v3.6-api-wrapper` (production)
- ✅ **UPDATE**: `songbird-orchestrator` symlink → v3.6

**BearDog** (`/tmp/`):
- ❌ `/tmp/beardog-server-zero` (old)
- ❌ `/tmp/beardog-server-v014` (old)
- ✅ **KEEP**: `/tmp/beardog-server-live` (v0.15.0 with v2 API)

### Old Logs to Clean

**Test/Debug Logs** (`/tmp/`):
- ❌ `songbird_clean.log` (test artifact)
- ❌ `songbird_discovery_tests.log`
- ❌ `songbird_fresh.log`
- ❌ `songbird_historic.log`
- ❌ `songbird_local.log`
- ❌ `songbird_tower*.log` (old test logs)
- ❌ `songbird_v3*.log` (version-specific test logs)
- ❌ `beardog_clean.log`
- ✅ **KEEP**: Latest production logs only

### Old Documentation to Archive

**Session Docs** (move to `docs/jan3-session/archive/`):
- `SONGBIRD_V34_INTEGRATION_TEST_JAN_3_2026.md` (intermediate)
- `SONGBIRD_V35_ROOT_CAUSE_IDENTIFIED_JAN_3_2026.md` (intermediate)
- Keep final success documentation

---

## 🔧 Phase 2: biomeOS Hardening

### 2.1 Enhanced BirdSong Client Integration

**Goal**: Make biomeOS's integration with BearDog/Songbird robust and reusable

**File**: `crates/biomeos-core/src/adaptive_client.rs`

**Enhancements**:
1. Add connection pooling for efficiency
2. Add retry logic with exponential backoff
3. Add circuit breaker for fault tolerance
4. Add metrics collection (success/failure rates)
5. Add comprehensive error types

**New Error Types**:
```rust
#[derive(Debug, thiserror::Error)]
pub enum BirdSongError {
    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("API error: {status} - {message}")]
    ApiError { status: u16, message: String },
    
    #[error("Encryption failed: {0}")]
    EncryptionFailed(String),
    
    #[error("Decryption failed: {0}")]
    DecryptionFailed(String),
    
    #[error("Invalid family: expected {expected}, got {actual}")]
    FamilyMismatch { expected: String, actual: String },
}
```

### 2.2 Family Seed Management

**Goal**: Secure, flexible family seed handling

**New Module**: `crates/biomeos-core/src/family_credentials.rs`

**Features**:
1. Load from environment (for testing)
2. Load from encrypted config (for production)
3. Support multiple families
4. Validation and format checking

```rust
pub struct FamilyCredentials {
    family_id: FamilyId,
    seed: SecretString, // zeroize on drop
}

impl FamilyCredentials {
    pub fn from_env() -> Result<Self, CredentialError>;
    pub fn from_encrypted_file(path: &Path, key: &[u8]) -> Result<Self, CredentialError>;
    pub fn validate(&self) -> Result<(), CredentialError>;
}
```

### 2.3 Primal Health Monitoring

**Goal**: Continuous health checks for Songbird/BearDog integration

**New Module**: `crates/biomeos-core/src/primal_health.rs`

**Features**:
1. Periodic health checks
2. Automatic recovery attempts
3. Alert on degraded state
4. Metrics export

```rust
pub struct PrimalHealthMonitor {
    check_interval: Duration,
    unhealthy_threshold: usize,
    recovery_strategy: RecoveryStrategy,
}

pub enum HealthStatus {
    Healthy,
    Degraded { reason: String },
    Unhealthy { reason: String, since: Instant },
}
```

### 2.4 Integration Test Suite

**New File**: `crates/biomeos-core/tests/birdsong_integration.rs`

**Tests**:
1. ✅ Encryption roundtrip with single family
2. ✅ Encryption with multiple families
3. ✅ Cross-family noise rejection
4. ✅ API version fallback (v1 → v2)
5. ✅ Error handling and recovery
6. ✅ Connection failure recovery

---

## 🧪 Phase 3: Multi-Family Validation

### 3.1 Test Families

**Family 1: iidn (current)**
```bash
FAMILY_ID="iidn"
FAMILY_SEED="iIDnVX3Tein1LFkrkkq7Wo3wsxPNek9XZqp0VL4Kn88="
```

**Family 2: test-alpha (new)**
```bash
FAMILY_ID="test-alpha"
FAMILY_SEED="<generate new base64 seed>"
```

**Family 3: test-beta (new)**
```bash
FAMILY_ID="test-beta"
FAMILY_SEED="<generate new base64 seed>"
```

### 3.2 Validation Scenarios

**Scenario 1: Same Family Auto-Trust**
- Tower 1: Family iidn
- Tower 2: Family iidn
- **Expected**: Auto-trust, encrypted discovery, genetic lineage detected

**Scenario 2: Different Family Isolation**
- Tower 1: Family iidn
- Tower 2: Family test-alpha
- **Expected**: Discovery packets appear as noise, no decryption, no trust

**Scenario 3: Multi-Family Network**
- Tower 1: Family iidn
- Tower 2: Family test-alpha
- Tower 3: Family iidn
- **Expected**: Tower 1 & 3 auto-trust, Tower 2 isolated

**Scenario 4: Family Rotation**
- Tower 1: Start with iidn, rotate to test-alpha
- **Expected**: Clean transition, no leaks, new family detected

### 3.3 Validation Script

**New File**: `scripts/validate-multi-family.sh`

```bash
#!/bin/bash
# Validate BirdSong encryption with multiple family seeds

set -euo pipefail

FAMILIES=(
  "iidn:iIDnVX3Tein1LFkrkkq7Wo3wsxPNek9XZqp0VL4Kn88="
  "test-alpha:ALPHA_SEED_HERE"
  "test-beta:BETA_SEED_HERE"
)

echo "🧪 Multi-Family Validation Suite"
echo "================================"

for family_spec in "${FAMILIES[@]}"; do
  IFS=':' read -r family_id family_seed <<< "$family_spec"
  
  echo ""
  echo "Testing family: $family_id"
  
  # Start BearDog with family
  # Start Songbird
  # Verify encryption
  # Verify decryption with same family
  # Verify rejection of other families
  
done

echo ""
echo "✅ All families validated!"
```

---

## 📊 Phase 4: Production Readiness Checklist

### Code Quality

- [ ] All `clippy` warnings resolved
- [ ] All tests passing (unit + integration)
- [ ] No `unwrap()` or `expect()` in production code paths
- [ ] Comprehensive error handling
- [ ] Logging at appropriate levels
- [ ] Documentation comments for public APIs

### Security

- [ ] Family seeds never logged
- [ ] Encrypted config support
- [ ] Secure defaults (encrypted by default)
- [ ] Input validation on all external data
- [ ] Rate limiting on API calls

### Performance

- [ ] Connection pooling for HTTP clients
- [ ] Efficient serialization (zero-copy where possible)
- [ ] Resource limits (max connections, timeouts)
- [ ] Memory profiling (no leaks)

### Reliability

- [ ] Retry logic with exponential backoff
- [ ] Circuit breakers for external services
- [ ] Graceful degradation (fallback to plaintext if needed)
- [ ] Health monitoring and recovery
- [ ] Deterministic behavior (same input → same output)

### Observability

- [ ] Structured logging
- [ ] Metrics export (Prometheus-compatible)
- [ ] Tracing support (jaeger/opentelemetry)
- [ ] Health check endpoints
- [ ] Version information in all logs

---

## 🚀 Execution Order

### Step 1: Cleanup (30 minutes)
1. Remove old binaries
2. Archive old logs
3. Archive intermediate documentation
4. Update symlinks

### Step 2: Harden biomeOS (2-3 hours)
1. Add `BirdSongError` types
2. Add `FamilyCredentials` module
3. Add `PrimalHealthMonitor` module
4. Add retry and circuit breaker logic
5. Update `adaptive_client.rs`

### Step 3: Validation Tests (1-2 hours)
1. Generate new test family seeds
2. Write integration tests
3. Write validation script
4. Run validation suite

### Step 4: Documentation (1 hour)
1. Update README with validation results
2. Create production deployment guide
3. Update STATUS.md
4. Create handoff document

---

## 📄 Success Criteria

✅ **Cleanup**: Zero obsolete binaries, clean working directory  
✅ **Hardening**: All production readiness checklist items complete  
✅ **Validation**: All multi-family scenarios pass deterministically  
✅ **Documentation**: Complete handoff with validation evidence  

**Target**: Production-ready, validated, deterministic biomeOS! 🎯

---

**Let's build it right.** 🦀

