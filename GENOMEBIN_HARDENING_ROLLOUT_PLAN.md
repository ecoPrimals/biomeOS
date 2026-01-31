# genomeBin Hardening Rollout Plan
*Production-Grade Enhancement for All Primals*

**Date**: January 31, 2026  
**Status**: Pilot Complete, Rolling Out  
**Progress**: 1/6 genomeBins Hardened

---

## 🎯 Rollout Strategy

### Phase 1: Pilot ✅ **COMPLETE**
- [x] Create hardening template (`genomeBin-hardened-template.sh`)
- [x] Apply to BearDog (pilot implementation)
- [x] Test CLI flags and features
- [x] Validate production readiness
- **Result**: BearDog hardened (455 lines, 11/11 features)

### Phase 2: TOWER Completion ⏳ **IN PROGRESS**
- [ ] Apply hardening to Songbird
- [ ] Test TOWER atomic with hardened genomeBins
- [ ] Validate BearDog + Songbird coordination
- **Goal**: Complete TOWER with production-grade deployment

### Phase 3: Full Ecosystem 🔄 **PENDING**
- [ ] Apply hardening to Squirrel
- [ ] Apply hardening to Toadstool  
- [ ] Apply hardening to NestGate
- [ ] Apply hardening to biomeOS
- **Goal**: All 6 genomeBins hardened

---

## 📊 Hardening Progress Tracker

| Primal | Size (Original) | Status | Features | Priority |
|--------|-----------------|--------|----------|----------|
| **BearDog** | 203 lines | ✅ **COMPLETE** | 11/11 | CRITICAL |
| **Songbird** | ~200 lines | ⏳ In Progress | 0/11 | CRITICAL |
| **Squirrel** | ~190 lines | ⏳ Pending | 0/11 | HIGH |
| **Toadstool** | ~190 lines | ⏳ Pending | 0/11 | HIGH |
| **NestGate** | ~190 lines | ⏳ Pending | 0/11 | HIGH |
| **biomeOS** | ~190 lines | ⏳ Pending | 0/11 | MEDIUM |

**Progress**: 1/6 (16.7%)  
**Estimated Total Lines**: ~2700 lines (hardened)  
**Current Total Lines**: ~1150 lines (original)  
**Growth Factor**: ~2.35x (comprehensive hardening)

---

## 🔧 Hardening Features Checklist

### Per genomeBin Implementation

**Core Hardening** (11 features):
1. ✅ Strict error handling (`set -eu`)
2. ✅ Comprehensive trap handlers (`EXIT/INT/TERM/HUP/QUIT`)
3. ✅ Automatic rollback on failure
4. ✅ SHA-256 checksum verification
5. ✅ Idempotent deployments
6. ✅ CLI flags (`--force`, `--verify-only`, `--skip-checksums`)
7. ✅ Structured logging (color-coded, leveled)
8. ✅ JSON deployment reports
9. ✅ Android noexec detection
10. ✅ Secure temporary directories
11. ✅ POSIX sh compatibility

**Primal-Specific Customization**:
- Genome name
- Genome description
- Version number
- Platform-specific features (e.g., HSM for BearDog)
- Service-specific next steps

---

## 🚀 Efficient Rollout Process

### Template-Based Approach

**Step 1: Read Original**
```bash
# Read the original genomeBin
cat primal.genome
```

**Step 2: Customize Template**
```bash
# Replace placeholders in hardened template:
GENOME_NAME="primal_name"
GENOME_VERSION="x.y.z"
GENOME_DESCRIPTION="Primal description"
```

**Step 3: Preserve Primal-Specific Logic**
```bash
# Copy any unique features from original:
# - Platform detection logic
# - Service-specific installation steps
# - Primal-specific environment variables
# - Custom health checks
```

**Step 4: Test**
```bash
# Test CLI flags
./primal.genome.hardened --help

# Test verify-only
./primal.genome.hardened --verify-only

# Test actual deployment (if binaries available)
./primal.genome.hardened --force
```

**Step 5: Replace Original**
```bash
# Backup original
mv primal.genome primal.genome.original

# Use hardened version
mv primal.genome.hardened primal.genome
```

---

## 📋 Primal-Specific Customizations

### BearDog ✅ **COMPLETE**
```sh
GENOME_NAME="beardog"
GENOME_VERSION="0.9.0"
GENOME_DESCRIPTION="BearDog Sovereign Crypto Orchestrator"
```

**Unique Features**:
- HSM (StrongBox) support on Android
- Abstract socket namespace
- Biometric authentication integration

### Songbird ⏳ **NEXT**
```sh
GENOME_NAME="songbird"
GENOME_VERSION="0.9.0"
GENOME_DESCRIPTION="Songbird Discovery & Federation Orchestrator"
```

**Unique Features**:
- mDNS service discovery
- BirdSong genetic verification
- Progressive trust escalation
- Federation state management

### Squirrel 🔄 **PENDING**
```sh
GENOME_NAME="squirrel"
GENOME_VERSION="0.9.0"
GENOME_DESCRIPTION="Squirrel AI Coordination & Task Management"
```

**Unique Features**:
- AI model coordination
- Task queue management
- Distributed AI workflows

### Toadstool 🔄 **PENDING**
```sh
GENOME_NAME="toadstool"
GENOME_VERSION="0.9.0"
GENOME_DESCRIPTION="Toadstool GPU Compute & Parallel Processing"
```

**Unique Features**:
- GPU detection and management
- Compute task scheduling
- Parallel processing coordination

### NestGate 🔄 **PENDING**
```sh
GENOME_NAME="nestgate"
GENOME_VERSION="0.9.0"
GENOME_DESCRIPTION="NestGate Storage & Persistence Layer"
```

**Unique Features**:
- Distributed storage
- Data persistence
- Backup and replication

### biomeOS 🔄 **PENDING**
```sh
GENOME_NAME="biomeos"
GENOME_VERSION="5.1.0"
GENOME_DESCRIPTION="biomeOS Nucleus Orchestrator"
```

**Unique Features**:
- Complete NUCLEUS orchestration
- All three atomics coordination
- System-wide configuration

---

## 🎯 Success Criteria

### Per genomeBin
- [ ] All 11 hardening features implemented
- [ ] CLI flags working (`--help`, `--force`, etc.)
- [ ] JSON deployment report generated
- [ ] Rollback tested (simulated failure)
- [ ] Idempotency verified (multiple runs)
- [ ] Android noexec detection tested (if applicable)

### Ecosystem-Wide
- [ ] 6/6 genomeBins hardened
- [ ] Consistent deployment experience
- [ ] All deployment reports compatible
- [ ] Documentation updated
- [ ] Production deployment validated

---

## 📈 Estimated Timeline

### Per genomeBin Effort
- Template customization: 15 minutes
- Testing: 10 minutes
- Documentation: 5 minutes
- **Total per primal**: ~30 minutes

### Remaining Work
- Songbird: 30 minutes (TOWER completion)
- Squirrel: 30 minutes
- Toadstool: 30 minutes
- NestGate: 30 minutes
- biomeOS: 30 minutes
- **Total remaining**: ~2.5 hours

### Complete Rollout
- Phase 1 (Pilot): ✅ Complete (2 hours)
- Phase 2 (TOWER): ⏳ 30 minutes
- Phase 3 (Ecosystem): 🔄 2 hours
- **Total**: ~4.5 hours

---

## 🎊 Expected Outcomes

### Code Quality
- **Total lines**: ~2700 (hardened) vs ~1150 (original)
- **Feature completeness**: 11/11 per primal (66 features total)
- **Error handling**: Comprehensive across all primals
- **Production readiness**: 100%

### User Experience
- **Deployment predictability**: Idempotent operations
- **Failure recovery**: Automatic rollback
- **Integrity assurance**: Checksum verification
- **Operational visibility**: JSON reports + structured logs

### Operational Benefits
- **Debugging**: Easier with structured logs
- **Auditing**: Complete with JSON reports
- **Reliability**: Higher with rollback capability
- **Security**: Enhanced with checksums

---

## 🚀 Immediate Next Steps

1. **Apply hardening to Songbird** ⏳ **IN PROGRESS**
   - Read `songbird.genome`
   - Customize hardened template
   - Test all features
   - Complete TOWER atomic

2. **Test TOWER with hardened genomeBins**
   - Deploy hardened BearDog + Songbird
   - Validate coordination
   - Test genetic verification
   - Confirm discovery

3. **Roll out to remaining 4 primals**
   - Squirrel → Toadstool → NestGate → biomeOS
   - Parallel processing (if possible)
   - Systematic testing

4. **Final ecosystem validation**
   - Deploy complete NUCLEUS
   - Test all atomics
   - Validate coordination
   - Production certification

---

## 📝 Documentation Updates

**Per Primal**:
- `PRIMAL_GENOMEBIN_HARDENING_COMPLETE.md`

**Ecosystem**:
- `GENOMEBIN_HARDENING_ROLLOUT_COMPLETE.md` (final)
- `ECOSYSTEM_STATUS.md` (update progress)
- `README.md` (update status badges)

---

*Plan Created: 2026-01-31T13:05:00Z*  
*Current Phase: 2 (TOWER Completion)*  
*Next: Harden Songbird genomeBin* 🚀
