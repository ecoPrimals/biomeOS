# USB Live Spore Clean Deployment Plan
**Date**: January 31, 2026  
**Goal**: Deploy biomeOS + NUCLEUS using NEW genomeBin v3.0 standard  
**Target**: USB Live Spore (clean, validated deployment)

---

## 🎯 Objectives

1. **Validate uniBin Compliance**: Ensure biomeOS follows ONE binary standard
2. **Clean USB Deployment**: Fresh deployment using genomeBin v3.0
3. **neuralAPI Validation**: Demonstrate full system spin-up
4. **Pixel Ready**: Prepare for Pixel 8a validation

---

## ✅ uniBin Compliance Status

### Current biomeOS Binaries

**CLI uniBin** (`biomeos`): ✅ COMPLIANT
- Single binary: `biomeos` (4.5 MB)
- All CLI functions included:
  - `genome` - genomeBin management
  - `chimera` - Chimera management
  - `niche` - Niche templates
  - `primal` - Primal management
  - `spore` - USB spore management
  - `discover` - Service discovery
  - `deploy` - Deployment orchestration
  - `deploy-graph` - Neural API deployment
  - `health`, `monitor`, `dashboard` - System monitoring
  - `federation` - Federation management
  - `node` - Node management
- Status: ✅ **TRUE uniBin** - all functionality in one binary

**API Server** (`biomeos-api`): ✅ SEPARATE SERVICE (Correct)
- Single binary: `biomeos-api` (2.6 MB)
- Purpose: neuralAPI REST server (long-running service)
- Status: ✅ Correctly separate (services vs CLI tools)

**Nucleus uniBin** (`nucleus`): ✅ COMPLIANT
- Single binary: `nucleus` (2.3 MB)
- Purpose: Core biomeOS orchestration daemon
- Status: ✅ uniBin compliant

### Legacy Binaries (TO DEPRECATE)

**Old Tools** (No longer needed):
- `genome-deploy` (937K) - ❌ DEPRECATED → Use `biomeos genome` instead
- `verify-lineage` (765K) - ❌ DEPRECATED → Use `biomeos verify` instead

**Action**: These are already superseded by `biomeos` uniBin CLI

---

## 📦 New genomeBin Standard

### Individual Primals (Production)

```
plasmidBin/
├── beardog-linux-multi.genome    3.2 MB  (x86_64 + ARM64)
├── songbird-linux.genome         7.5 MB  (x86_64)
├── toadstool-linux.genome        3.4 MB  (x86_64)
└── nestgate-linux.genome         3.6 MB  (x86_64 + ARM64)
```

### Atomic Compositions

```
plasmidBin/
├── tower.genome                  18 MB   (BearDog + Songbird)
├── node.genome                   27 MB   (TOWER + Toadstool)
├── nest.genome                   22 MB   (TOWER + NestGate)
└── nucleus.genome                30 MB   (ALL 4 PRIMALS)
```

### biomeOS System

**To Create**:
```
plasmidBin/
└── biomeos-system.genome         ~8 MB   (biomeos CLI + biomeos-api)
```

This will contain:
- `biomeos` uniBin (CLI + genome factory)
- `biomeos-api` (neuralAPI server)
- `nucleus` (orchestration daemon)

**Purpose**: Single genomeBin for complete biomeOS system deployment

---

## 🚀 USB Live Spore Deployment Plan

### Phase 1: Create biomeOS System genomeBin ✅

```bash
cd ~/Development/ecoPrimals/phase2/biomeOS

# Build for x86_64 (USB is x86_64)
cargo build --release --target x86_64-unknown-linux-musl

# Create biomeOS system genomeBin
./target/x86_64-unknown-linux-musl/release/biomeos genome create biomeos-system \
  --binary x86_64=/path/to/biomeos \
  --binary x86_64=/path/to/biomeos-api \
  --binary x86_64=/path/to/nucleus \
  --version "0.9.0" \
  --description "biomeOS Complete System (CLI + API + Nucleus)"
```

**Result**: `biomeos-system.genome` (~8 MB, all system components)

### Phase 2: Clean USB Preparation

```bash
# Mount USB
sudo mkdir -p /mnt/live_spore
sudo mount /dev/sdb1 /mnt/live_spore

# Backup old deployment (if needed)
sudo cp -r /mnt/live_spore/biomeos /tmp/biomeos_backup_$(date +%Y%m%d)

# Clean old binaries
sudo rm -rf /mnt/live_spore/biomeos
sudo mkdir -p /mnt/live_spore/biomeos
```

### Phase 3: Deploy New genomeBins

```bash
# Copy genomeBins to USB
sudo cp plasmidBin/biomeos-system.genome /mnt/live_spore/biomeos/
sudo cp plasmidBin/nucleus.genome /mnt/live_spore/biomeos/
sudo cp plasmidBin/tower.genome /mnt/live_spore/biomeos/

# Make executable
sudo chmod +x /mnt/live_spore/biomeos/*.genome
```

### Phase 4: Deploy System

```bash
# Extract biomeOS system
cd /mnt/live_spore/biomeos
sudo ./biomeos-system.genome

# Result: 
#   - biomeos CLI installed
#   - biomeos-api server binary installed
#   - nucleus daemon installed
```

### Phase 5: Deploy NUCLEUS Ecosystem

```bash
# Use biomeOS CLI to deploy complete ecosystem
biomeos genome extract nucleus.genome --output /mnt/live_spore/biomeos/primals/

# Or use neuralAPI graph deployment
biomeos deploy-graph /path/to/nucleus_deployment.toml
```

### Phase 6: Start neuralAPI

```bash
# Start neuralAPI server
biomeos-api server --bind 127.0.0.1:9090

# Verify health
curl http://localhost:9090/api/v1/health

# Expected: {"status":"healthy","version":"0.9.0"}
```

### Phase 7: Validate Full System

```bash
# List available genomeBins
biomeos genome list

# Verify integrity
biomeos genome verify nucleus

# Check system status
biomeos status

# Discover services
biomeos discover --capability encryption
# Should find BearDog

biomeos discover --capability discovery
# Should find Songbird
```

---

## 🧬 Deployment Validation Tests

### Test 1: uniBin Compliance

```bash
# Verify single binary does everything
biomeos genome list
biomeos primal list
biomeos discover --all
biomeos health

# Expected: All commands work from ONE binary
```

### Test 2: genomeBin Extraction

```bash
# Extract and verify binaries
biomeos genome extract biomeos-system.genome --output /tmp/test
ls -lh /tmp/test/x86_64/

# Expected: biomeos, biomeos-api, nucleus binaries
```

### Test 3: neuralAPI Integration

```bash
# Create genomeBin via API
curl -X POST http://localhost:9090/api/v1/genome/create \
  -H "Content-Type: application/json" \
  -d '{
    "name": "test-primal",
    "version": "1.0.0",
    "binaries": {"x86_64": "/path/to/binary"}
  }'

# List via API
curl http://localhost:9090/api/v1/genome/list

# Expected: JSON response with genomeBin list
```

### Test 4: NUCLEUS Deployment

```bash
# Deploy complete ecosystem
biomeos deploy-graph graphs/nucleus_genome.toml

# Check all primals running
biomeos primal list

# Expected: BearDog, Songbird, Toadstool, NestGate all operational
```

---

## 📊 Success Criteria

### uniBin Compliance

- [x] `biomeos` CLI is single binary ✅
- [x] All CLI functions in one binary ✅
- [ ] Legacy binaries deprecated
- [ ] Documentation updated

### USB Deployment

- [ ] Clean USB with genomeBin v3.0 standard
- [ ] biomeOS system deployed via genomeBin
- [ ] NUCLEUS ecosystem deployable
- [ ] neuralAPI server operational

### Validation

- [ ] All genomeBins extract correctly
- [ ] Multi-arch selection works
- [ ] SHA256 verification passes
- [ ] neuralAPI endpoints respond
- [ ] Primals discover each other
- [ ] Full NUCLEUS coordination working

---

## 🎯 Next Steps After USB Validation

### Pixel 8a Deployment (ARM64)

```bash
# Use multi-arch genomeBins
adb push plasmidBin/biomeos-system.genome /data/local/tmp/
adb shell
cd /data/local/tmp
./biomeos-system.genome

# Deploy ARM64 primals
biomeos genome extract nucleus.genome --output ./primals/
# Auto-selects ARM64 binaries
```

### STUN Validation

```bash
# USB ↔ Pixel cross-platform handshake
# Both sides discover via mDNS
# Establish encrypted channel
# Validate genetic trust
```

---

## 📝 Implementation Order

1. ✅ **Validate uniBin compliance** (DONE - `biomeos` is compliant)
2. **Create biomeOS-system genomeBin** (Next)
3. **Clean USB deployment** (After genomeBin created)
4. **Start neuralAPI** (After deployment)
5. **Deploy NUCLEUS** (After neuralAPI up)
6. **Validate full system** (After NUCLEUS deployed)
7. **Pixel deployment** (After USB validated)

---

**Status**: Ready to proceed with biomeOS-system genomeBin creation  
**Blocker**: None  
**Confidence**: High (uniBin compliance already met)
