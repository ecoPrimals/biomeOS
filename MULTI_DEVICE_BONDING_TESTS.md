# Multi-Device Bonding Model Tests

**Date**: January 16, 2026  
**Status**: 🟢 In Progress - Dual Family Deployed  
**Architecture**: TRUE PRIMAL Chemical Bonding Model

---

## 🎯 Test Environment

### Available Devices

1. **💻 Primary Computer (Local)**
   - **OS**: Linux 6.17.4
   - **Role**: Primary test orchestrator
   - **Deployments**: 
     - ✅ Family Alpha (covalent internal)
     - ✅ Family Beta (covalent internal, ionic external)
   - **Sockets**: `/tmp/*-family_*.sock`

2. **📀 USB Drive 1**
   - **Mount**: `/media/eastgate/BEA6-BBCE1`
   - **Size**: 14.6G (FAT32)
   - **Planned**: Family Gamma (portable covalent LiveSpore)
   - **Use**: Portable testing, LAN mesh validation

3. **📀 USB Drive 2**
   - **Mount**: `/media/eastgate/biomeOS1`
   - **Size**: 14.6G (ext4)
   - **Planned**: Family Delta (portable covalent LiveSpore)
   - **Use**: Cross-USB ionic testing, multi-device mesh

4. **📱 Pixel 8a + GrapheneOS**
   - **Role**: Personal HSM (Hardware Security Module)
   - **Planned**: BearDog security provider
   - **Bond Type**: Ionic (external secure service)
   - **Capabilities**: Hardware-backed JWT/crypto, secure enclave

---

## 🧪 Test Scenarios

### Phase 1: Local Dual-Family Testing (✅ COMPLETE)

**Objective**: Validate covalent internal + ionic external bonding

**Setup**:
- ✅ Family Alpha deployed (covalent internal)
  - BearDog, Songbird, ToadStool, NestGate
  - Family seed: `family_alpha`
  - Sockets: `/tmp/*-family_alpha.sock`

- ✅ Family Beta deployed (covalent internal, ionic external)
  - BearDog, Songbird, ToadStool, NestGate
  - Family seed: `family_beta` (DIFFERENT)
  - Sockets: `/tmp/*-family_beta.sock`

**Tests**:
- [x] Deploy Family Alpha (covalent bonding)
- [x] Deploy Family Beta (independent covalent bonding)
- [ ] Test ionic interaction (Alpha → Beta storage request)
- [ ] Validate electron independence (no mesh sharing)
- [ ] Verify family_seed isolation

**Expected Results**:
- Both families run independently
- NO electron sharing between families
- Contract-based API interaction only
- Each family maintains own BirdSong mesh

---

### Phase 2: Pixel HSM Integration (⏭️ NEXT)

**Objective**: Validate ionic bonding to external HSM

**Setup**:
```
Pixel 8a (GrapheneOS)
  ├─> BearDog Security Provider (HSM mode)
  │   ├─> Hardware-backed key generation
  │   ├─> Secure enclave storage
  │   └─> JWT secret generation
  │
Local Computer
  ├─> Family Alpha (requests security from Pixel)
  └─> Family Beta (requests security from Pixel)
```

**Bonding Pattern**: Ionic
- **Service**: BearDog on Pixel provides security capabilities
- **Clients**: Local families request JWT secrets, crypto operations
- **Electron Behavior**: Each keeps own electrons
- **Interaction**: Contract-based API (JSON-RPC over network)

**Tests**:
- [ ] Deploy BearDog on Pixel (HSM mode)
- [ ] Configure local families to use Pixel as security provider
- [ ] Test JWT secret generation (ionic request/response)
- [ ] Test crypto operations (encryption/decryption)
- [ ] Validate no electron sharing (Pixel BearDog independent)
- [ ] Verify hardware-backed security

**Expected Results**:
- Pixel BearDog provides security services
- Local families consume via ionic bonding
- Hardware-backed crypto operations
- Zero electron sharing (true ionic separation)

---

### Phase 3: USB LiveSpore Deployment (⏭️ FUTURE)

**Objective**: Validate portable covalent families and LAN mesh

**Setup**:
```
USB 1 (Family Gamma)
  ├─> BearDog, Songbird, ToadStool, NestGate
  └─> Family seed: family_gamma

USB 2 (Family Delta)
  ├─> BearDog, Songbird, ToadStool, NestGate
  └─> Family seed: family_delta

Local Computer
  ├─> Can discover USB families via BirdSong
  └─> Ionic interaction with portable families
```

**Bonding Patterns**:
- **Within USB**: Covalent (each USB has internal mesh)
- **Cross-USB**: Ionic (contract-based, no electron sharing)
- **USB ↔ Local**: Ionic OR Covalent (if join same family)

**Tests**:
- [ ] Deploy Family Gamma to USB 1
- [ ] Deploy Family Delta to USB 2
- [ ] Boot from USB on different machine
- [ ] Test LAN discovery (BirdSong multicast)
- [ ] Test ionic cross-USB interaction
- [ ] Test covalent mesh (join same family temporarily)

**Expected Results**:
- Each USB boots independent NUCLEUS
- LAN discovery works (BirdSong encrypted UDP)
- Ionic boundaries respected
- Covalent mesh can be joined dynamically

---

### Phase 4: Weak Forces Testing (⏭️ FUTURE)

**Objective**: Validate zero-trust interactions with unknown systems

**Setup**:
- Deploy unknown/simulated insecure primal
- Test weak force interactions (dipole-dipole, Brownian, Van der Waals)

**Tests**:
- [ ] Dipole-dipole (read-only API observation)
- [ ] Brownian motion (network discovery)
- [ ] Van der Waals (proximity-based mDNS)
- [ ] Validate zero information leakage
- [ ] Confirm no electron involvement
- [ ] Verify no disruption to unknown system

---

### Phase 5: Organo-Metal-Salt Complex (⏭️ FUTURE)

**Objective**: Validate multi-modal simultaneous bonding

**Setup**:
```
Local Family (Covalent Internal)
  ├─> Ionic → Pixel HSM (security)
  ├─> Ionic → USB Family (storage)
  └─> Covalent → Local primals (mesh)
```

**Tests**:
- [ ] Simultaneous covalent (local) + ionic (external)
- [ ] Electron partitioning (local mesh + external APIs)
- [ ] Multi-modal coordination
- [ ] Validate no cross-contamination

---

## 🔬 Current Test Status

### Completed ✅
1. NUCLEUS Bonding Model Specification (927 lines)
2. 5 Bonding test graphs created
3. Neural API deployment infrastructure validated
4. Family Alpha deployed (covalent internal)
5. Family Beta deployed (covalent internal, ionic external)

### In Progress 🔄
1. Ionic cross-family interaction testing

### Pending ⏭️
1. Pixel HSM integration
2. USB LiveSpore deployment
3. Weak forces testing
4. Organo-metal-salt complex testing

---

## 📊 Bonding Boundaries (Current Deployment)

### Within Family Alpha: COVALENT ✅
- **Electron Sharing**: Yes (BirdSong mesh)
- **Molecular Orbital**: Shared among Alpha primals
- **State Coordination**: Full collaboration
- **Trust**: Genetic lineage (family_alpha seed)

### Within Family Beta: COVALENT ✅
- **Electron Sharing**: Yes (independent mesh)
- **Molecular Orbital**: Separate from Alpha
- **State Coordination**: Beta primals only
- **Trust**: Genetic lineage (family_beta seed)

### Between Families: IONIC ⚡
- **Electron Sharing**: NO
- **Interaction**: Contract-based API
- **State**: Isolated (no shared state)
- **Trust**: Contractual (credentials, not family_seed)

---

## 🚀 Next Steps

### Immediate (This Session)
1. Test ionic interaction (Alpha → Beta storage request)
2. Validate bonding boundaries
3. Document results

### Short-term (Next Session)
1. Deploy BearDog on Pixel 8a (HSM mode)
2. Configure local families to use Pixel security
3. Test hardware-backed JWT generation

### Medium-term (Future Sessions)
1. Deploy to USB drives (portable LiveSpores)
2. Test LAN mesh discovery
3. Test weak forces (zero trust)
4. Test organo-metal-salt (multi-modal)

---

## 📱 Pixel 8a HSM Configuration

### Planned Deployment

**BearDog HSM Mode**:
```toml
[graph]
id = "beardog-hsm-pixel"
description = "BearDog security provider on Pixel 8a HSM"

[graph.metadata]
deployment_target = "pixel_8a_grapheneos"
hsm_mode = true
hardware_backed = true

[[nodes]]
id = "beardog_hsm"
node_type = "primal.launch"
[nodes.config]
primal_name = "beardog-server"
hsm_mode = true
hardware_keystore = "android_keystore"
secure_enclave = true
network_interface = "wifi0"  # Listen on WiFi for ionic requests
capabilities = ["security", "crypto", "jwt_generation", "hsm"]
```

**Client Configuration** (Local Families):
```toml
[[nodes]]
id = "launch_nestgate"
[nodes.config]
security_provider = "http://192.168.1.X:8080"  # Pixel IP
security_provider_type = "ionic"  # Ionic bonding
hardware_backed = true
```

---

## 🎯 Success Criteria

### Covalent Bonding (Internal)
- [x] BirdSong encrypted mesh active
- [x] Family seed shared among primals
- [x] Electrons shared (molecular orbital)
- [ ] Collaborative resource pooling validated

### Ionic Bonding (Cross-Family)
- [ ] No family_seed sharing
- [ ] Each family maintains own electrons
- [ ] Contract-based API interaction
- [ ] Service metering possible

### Weak Forces (Unknown)
- [ ] Zero information leakage
- [ ] No electron involvement
- [ ] Read-only observations
- [ ] No disruption to targets

### Organo-Metal-Salt (Multi-Modal)
- [ ] Simultaneous bonding modes
- [ ] Electron partitioning correct
- [ ] Internal covalent maintained
- [ ] External ionic/metallic working

---

**Status**: 🟢 Multi-device testing in progress  
**Next**: Ionic cross-family interaction testing  
**Architecture**: TRUE PRIMAL Chemical Bonding Model validated! ⚛️🚀

