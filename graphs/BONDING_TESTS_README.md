# Bonding Model Test Graphs

**Purpose**: Validate the NUCLEUS Bonding Model specification (`specs/NUCLEUS_BONDING_MODEL.md`)  
**Created**: January 16, 2026  
**Test Infrastructure**: 2 USB drives with different family_seeds

---

## 🧪 Test Suite Overview

This directory contains 5 comprehensive test graphs for validating all bonding types and interaction patterns defined in the NUCLEUS Bonding Model.

### Test Graphs

1. **`bonding_test_covalent_family_alpha.toml`** - Covalent Internal (Family Alpha)
2. **`bonding_test_covalent_family_beta.toml`** - Covalent Internal (Family Beta)
3. **`bonding_test_ionic_interaction.toml`** - Ionic Cross-Family Interaction
4. **`bonding_test_weak_forces.toml`** - Weak Forces (Unknown Systems)
5. **`bonding_test_organo_metal_salt.toml`** - Organo-Metal-Salt Complex

---

## 📀 USB Deployment Plan

### USB 1: Family Alpha (Covalent Internal)
**Mount**: `/media/eastgate/BEA6-BBCE1` (14.6G, FAT32)  
**Family ID**: `family_alpha`  
**Graph**: `bonding_test_covalent_family_alpha.toml`

**Internal Bonding**: Covalent (electron-sharing)
- All 5 primals share `family_alpha` seed
- BirdSong encrypted mesh coordination
- Shared molecular orbital (Tower/Songbird)
- Collaborative compute and storage

**Sockets Created**:
```
/tmp/beardog-family_alpha.sock
/tmp/songbird-family_alpha.sock
/tmp/squirrel-family_alpha.sock
/tmp/toadstool-family_alpha.sock
/tmp/nestgate-family_alpha.sock
```

### USB 2: Family Beta (Covalent Internal, Ionic External)
**Mount**: `/media/eastgate/biomeOS1` (14.6G, ext4)  
**Family ID**: `family_beta`  
**Graph**: `bonding_test_covalent_family_beta.toml`

**Internal Bonding**: Covalent (electron-sharing)
- All 5 primals share `family_beta` seed (DIFFERENT from Alpha)
- Independent BirdSong mesh
- Separate genetic lineage

**External Bonding**: Ionic (contract-based)
- Can interact with Family Alpha via API contracts
- No family_seed sharing
- Each family maintains own electrons

**Sockets Created**:
```
/tmp/beardog-family_beta.sock
/tmp/songbird-family_beta.sock
/tmp/squirrel-family_beta.sock
/tmp/toadstool-family_beta.sock
/tmp/nestgate-family_beta.sock
```

---

## 🧪 Test Execution Order

### Phase 1: Deploy Both Families (Covalent Internal)

```bash
# Deploy Family Alpha to USB 1
./plasmidBin/primals/neural-deploy bonding-test-covalent-family-alpha

# Deploy Family Beta to USB 2
./plasmidBin/primals/neural-deploy bonding-test-covalent-family-beta

# Verify both NUCLEUS are operational
ps aux | grep -E "(beardog|songbird|toadstool|nestgate)"
ls -l /tmp/*-family_*.sock
```

**Expected Result**:
- ✅ 10 sockets created (5 per family)
- ✅ Each family has internal covalent bonding
- ✅ Families are genetically separate (different family_seeds)

### Phase 2: Test Ionic Interaction (Cross-Family)

```bash
# Test ionic bonding between families
./plasmidBin/primals/neural-deploy bonding-test-ionic-interaction
```

**Expected Result**:
- ✅ Family Alpha can request storage from Family Beta
- ✅ No family_seed sharing (ionic boundary respected)
- ✅ Each family maintains own electrons (Towers)
- ✅ Contract-based API interaction only

### Phase 3: Test Weak Forces (Unknown Primals)

```bash
# Test weak forces with unknown systems
./plasmidBin/primals/neural-deploy bonding-test-weak-forces
```

**Expected Result**:
- ✅ Zero information leakage
- ✅ No electron involvement
- ✅ Read-only observations
- ✅ No disruption to unknown systems

### Phase 4: Test Organo-Metal-Salt Complex

```bash
# Test multi-modal interactions
./plasmidBin/primals/neural-deploy bonding-test-organo-metal-salt
```

**Expected Result**:
- ✅ Simultaneous covalent (local) + ionic (cloud) + metallic (datacenter)
- ✅ Electrons correctly partitioned
- ✅ Internal covalent bonding maintained despite external connections

---

## 🎯 Validation Criteria

### Covalent Bonding (Internal)
- [ ] BirdSong encrypted mesh active
- [ ] Family_seed shared among primals
- [ ] Electrons shared (molecular orbital)
- [ ] Collaborative resource pooling

### Ionic Bonding (Cross-Family)
- [ ] No family_seed sharing
- [ ] Each family maintains own electrons
- [ ] API-based contract interaction
- [ ] Metering/billing possible

### Weak Forces (Unknown)
- [ ] Zero trust maintained
- [ ] No electron involvement
- [ ] Minimal information disclosure
- [ ] No disruption or leakage

### Organo-Metal-Salt (Multi-Modal)
- [ ] Multiple bonding modes simultaneous
- [ ] Electron partitioning correct
- [ ] Internal covalent maintained
- [ ] External ionic/metallic working

---

## 📊 Test Graph Details

### 1. Covalent Family Alpha
**File**: `bonding_test_covalent_family_alpha.toml`  
**Bonding**: Covalent (internal)  
**Family**: `family_alpha`  
**Primals**: 5 (BearDog, Songbird, Squirrel, ToadStool, NestGate)

**Key Metadata**:
```toml
[metadata]
internal_bond_type = "covalent"
trust_model = "genetic_lineage"
family_id = "family_alpha"
```

**Tests**:
- Covalent mesh coordination
- Electron sharing (BirdSong)
- Collaborative compute/storage

### 2. Covalent Family Beta
**File**: `bonding_test_covalent_family_beta.toml`  
**Bonding**: Covalent (internal), Ionic (external)  
**Family**: `family_beta`  
**Primals**: 5 (independent from Alpha)

**Key Metadata**:
```toml
[metadata]
internal_bond_type = "covalent"
default_interaction_bond = "ionic"  # External
trust_model = "genetic_lineage"
family_id = "family_beta"
```

**Tests**:
- Independent covalent bonding
- Ionic external boundary
- Storage service API (ionic)

### 3. Ionic Interaction
**File**: `bonding_test_ionic_interaction.toml`  
**Bonding**: Ionic (cross-family)  
**Parties**: Family Alpha → Family Beta

**Key Metadata**:
```toml
[metadata]
test_scenario = "ionic_cross_family"
interaction_bond_type = "ionic"
trust_model = "contractual"
electron_behavior = "separate"
```

**Tests**:
- Storage request (Alpha → Beta)
- Ionic boundary validation
- Family separation verification
- Electron independence confirmation

### 4. Weak Forces
**File**: `bonding_test_weak_forces.toml`  
**Bonding**: Weak (dipole-dipole, Brownian, Van der Waals)  
**Target**: Unknown primals

**Key Metadata**:
```toml
[metadata]
test_scenario = "weak_forces_unknown_primal"
interaction_bond_type = "weak"
trust_model = "zero_trust"
electron_involvement = false
information_disclosure = "minimal"
```

**Tests**:
- Dipole-dipole (public API observation)
- Brownian motion (network discovery)
- Van der Waals (proximity-based)
- Zero leakage validation

### 5. Organo-Metal-Salt Complex
**File**: `bonding_test_organo_metal_salt.toml`  
**Bonding**: Multi-modal (covalent + ionic + metallic)  
**System**: Basement (Family Alpha) with external connections

**Key Metadata**:
```toml
[metadata]
test_scenario = "organo_metal_salt_complex"
local_bond = "covalent"
remote_bond_1 = "ionic"
remote_bond_2 = "metallic"
interaction_complexity = "multi_modal"
```

**Tests**:
- Covalent internal (basement nodes)
- Ionic external (cloud GPU)
- Metallic integration (datacenter)
- Multi-modal validation
- Electron partitioning

---

## 🚀 Quick Start

### 1. Build Deployment Tools
```bash
cargo build --release --bin livespore-deploy
cargo build --release --bin neural-api-server
cargo build --release --bin neural-deploy
```

### 2. Deploy Families
```bash
# Start Neural API server
./plasmidBin/primals/neural-api-server &

# Deploy Family Alpha
./plasmidBin/primals/neural-deploy bonding-test-covalent-family-alpha

# Deploy Family Beta
./plasmidBin/primals/neural-deploy bonding-test-covalent-family-beta
```

### 3. Run Interaction Tests
```bash
# Test ionic bonding
./plasmidBin/primals/neural-deploy bonding-test-ionic-interaction

# Test weak forces
./plasmidBin/primals/neural-deploy bonding-test-weak-forces

# Test organo-metal-salt
./plasmidBin/primals/neural-deploy bonding-test-organo-metal-salt
```

### 4. Validate Results
Check logs, sockets, and interaction boundaries!

---

## 📚 References

- **Specification**: `specs/NUCLEUS_BONDING_MODEL.md`
- **Architecture**: `README.md`, `STATUS.md`
- **Neural API**: `specs/NEURAL_API_SPEC.md`

---

**Status**: ✅ Graphs Created (January 16, 2026)  
**Next**: Build livespore-deploy and deploy to USB drives!  
**Goal**: Validate TRUE PRIMAL Chemical Bonding Model! ⚛️🚀

