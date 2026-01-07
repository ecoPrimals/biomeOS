# 🧬 Genetic Lineage Spore System - January 7, 2026

## 🎯 Vision

biomeOS should be **self-propagating** - capable of creating its own USB spores with proper genetic lineage, enabling cryptographic family recognition without manual configuration.

## 📊 Current State Analysis

### ✅ What Exists

#### 1. Infrastructure Scripts (Complete!)
```
scripts/
├── create-usb-family-seed.sh         ✅ Creates genesis seed
├── verify-usb-genetic-lineage.sh     ✅ Verifies setup
├── prepare-usb-spore.sh              ✅ Prepares USB
├── format-and-deploy-usb.sh          ✅ Full deployment
└── validate-usb-spore.sh             ✅ Validation
```

#### 2. USB Spores (Partially Complete)
```
biomeOS1/biomeOS/.family.seed  (32 bytes, 600 perms) ✅
biomeOS21/biomeOS/.family.seed (32 bytes, 600 perms) ✅
```

#### 3. BearDog Support (Complete!)
```rust
// From: crates/beardog-tunnel/src/api/server.rs:114
if let Ok(family_seed) = std::env::var("BEARDOG_FAMILY_SEED") {
    // Extract family ID, derive node ID
    // Create genetic lineage
}

// HKDF-based key derivation exists:
use hkdf::Hkdf;
let hk = Hkdf::<Sha256>::new(Some(salt), &ikm);
```

### ⚠️ What's Incomplete

#### 1. Hardcoded Seeds in tower.toml
```toml
# Current (BAD - hardcoded):
BEARDOG_FAMILY_SEED = "Nat0C/G/b4B7u06n0r14SuZXrp/IZ/38fZHh8aJQMVg="

# Should be (GOOD - reference file):
BEARDOG_FAMILY_SEED_FILE = "/media/usb/.family.seed"
```

#### 2. No secrets/ Directory Structure
```
# Should have:
biomeOS/
├── secrets/
│   ├── family-genesis.key      (PRIVATE parent seed)
│   ├── family-genesis.pub      (PUBLIC hash for verification)
│   └── README-SECURITY.txt     (Documentation)
├── .family.seed                (Binary 32-byte seed)
└── tower.toml                  (References .family.seed)
```

#### 3. Missing biomeOS Integration
- ✅ Scripts exist but aren't called by biomeOS
- ⚠️ No `biomeos spore create` command
- ⚠️ No `biomeos spore verify` command
- ⚠️ No `biomeos spore clone` command

#### 4. Sibling Derivation Not Automated
```
# Vision:
1. Local seed exists (machine entropy)
2. USB creation: local_seed + usb_entropy → parent_seed
3. Tower launch: parent_seed + hostname → child_seed
4. Siblings share parent_seed, unique child_seeds

# Current:
- Manual seed management
- No automated sibling creation
- Seeds hardcoded in config
```

---

## 🏗️ Architecture Design

### Tier 1: USB Seed Only (Home/Testing)
```
┌─────────────────────────────────────────────────────┐
│  USB Genesis Seed (Parent DNA)                     │
│  - Created ONCE during USB package                 │
│  - Stored: /media/usb/.family.seed (binary)        │
│  - Shared across all towers from this USB          │
└─────────────────────────────────────────────────────┘
           ↓ HKDF(parent_seed, hostname, entropy)
┌─────────────────────────────────────────────────────┐
│  Tower Child Seeds (Unique Identities)             │
│  - tower1: HKDF(parent, "tower1", uuid_1)          │
│  - tower2: HKDF(parent, "tower2", uuid_2)          │
│  - tower3: HKDF(parent, "tower3", uuid_3)          │
│  → All siblings, cryptographically verifiable       │
└─────────────────────────────────────────────────────┘
```

### Tier 2: USB + Phone HSM (Production)
```
┌──────────┐      ┌──────────┐
│   USB    │  +   │  Phone   │  → Enhanced Parent Seed
│  Entropy │      │  HSM     │     (Human presence proof)
└──────────┘      └──────────┘
```

### Tier 3: USB + Hardware HSM (Enterprise)
```
┌──────────┐      ┌──────────┐
│   USB    │  +   │ YubiKey  │  → Maximum Security
│  Entropy │      │ SoloKeys │     (Hardware-backed)
└──────────┘      └──────────┘
```

---

## 🔧 Technical Implementation

### Seed Derivation Flow

#### Step 1: Genesis Seed Creation (One Time)
```bash
# When creating USB spore:
$ biomeos spore create /dev/sdb --family nat0

# Internally:
1. Generate high-entropy parent seed (256 bits)
   parent_seed = openssl rand 32

2. Store on USB (encrypted, binary)
   echo $parent_seed > /media/usb/.family.seed
   chmod 600 .family.seed

3. Create secrets/ directory
   mkdir -p secrets/
   echo $parent_seed > secrets/family-genesis.key
   sha256($parent_seed) > secrets/family-genesis.pub
```

#### Step 2: Tower Initialization (Each Boot)
```rust
// When tower starts:
fn initialize_genetic_lineage() -> Result<(String, String)> {
    // 1. Read parent seed from USB
    let parent_seed = if let Ok(seed_path) = env::var("BEARDOG_FAMILY_SEED_FILE") {
        fs::read(seed_path)?  // Binary 32 bytes
    } else {
        return Err("No family seed file found");
    };
    
    // 2. Gather local entropy
    let hostname = gethostname();  // "tower1", "tower2", etc.
    let uuid = machine_uuid();     // Unique per machine
    let sys_entropy = rand::random::<[u8; 32]>();
    
    // 3. Derive child seed via HKDF
    let child_seed = derive_child_key(&parent_seed, &hostname, &uuid, &sys_entropy)?;
    
    // 4. Extract family ID (first 4 alphanumeric)
    let family_id = extract_family_id(&parent_seed);  // "nat0"
    
    // 5. Derive node ID
    let node_id = hash(&[child_seed, hostname.as_bytes()])[..8].hex();
    
    // 6. Zeroize parent_seed (never store on disk!)
    parent_seed.zeroize();
    
    Ok((family_id, node_id))
}

fn derive_child_key(
    parent: &[u8],
    hostname: &str,
    uuid: &str,
    entropy: &[u8]
) -> Result<Vec<u8>> {
    use hkdf::Hkdf;
    use sha2::Sha256;
    
    // Combine all inputs as IKM (Input Key Material)
    let mut ikm = Vec::new();
    ikm.extend_from_slice(parent);
    ikm.extend_from_slice(hostname.as_bytes());
    ikm.extend_from_slice(uuid.as_bytes());
    ikm.extend_from_slice(entropy);
    
    // Salt for domain separation
    let salt = b"biomeos-genetic-lineage-v1";
    
    // HKDF-SHA256 expansion
    let hk = Hkdf::<Sha256>::new(Some(salt), &ikm);
    
    let mut child_key = vec![0u8; 32];
    hk.expand(b"child-tower-key", &mut child_key)?;
    
    Ok(child_key)
}
```

#### Step 3: Sibling Recognition
```rust
// When towers meet:
fn verify_sibling(peer_proof: &LineageProof, our_family_id: &str) -> bool {
    // Both towers derived from same parent_seed
    // → Both have family_id "nat0"
    // → Cryptographically verifiable
    
    peer_proof.family_id == our_family_id  // "nat0" == "nat0" ✅
}
```

---

## 📋 Implementation Plan

### Phase 1: Standardize USB Spore Creation ✅ (Scripts Exist!)

**Status**: Scripts complete, need integration with biomeOS

**Commands to Add**:
```bash
# Create new USB spore with genetic lineage
biomeos spore create /dev/sdb --family nat0

# Clone existing spore (creates sibling)
biomeos spore clone /dev/sdb --from /dev/sdc --sibling tower2

# Verify spore integrity
biomeos spore verify /dev/sdb

# Show spore info
biomeos spore info /dev/sdb
```

**Implementation**:
- Wrap existing scripts in biomeOS CLI
- Add to `crates/biomeos-cli/src/commands/spore.rs`
- Call scripts from Rust with proper error handling

---

### Phase 2: Update tower.toml → File Reference

**Current** (Hardcoded):
```toml
[primals.env]
BEARDOG_FAMILY_SEED = "Nat0C/G/b4B7u06n0r14SuZXrp/IZ/38fZHh8aJQMVg="
BEARDOG_FAMILY_ID = "nat0"
BEARDOG_NODE_ID = "tower1"
```

**Target** (File Reference):
```toml
[primals.env]
BEARDOG_FAMILY_SEED_FILE = "${USB_ROOT}/.family.seed"
BEARDOG_NODE_ID = "${HOSTNAME}"  # Auto-derived
# BEARDOG_FAMILY_ID auto-extracted from seed
```

**Changes Needed**:
1. Update tower config parser to support file references
2. BearDog reads from file path instead of env var
3. Auto-derive family_id and node_id
4. Remove hardcoded values from config

---

### Phase 3: Implement Sibling Cloning

**Goal**: Create genetic siblings automatically

**Command**:
```bash
# Create sibling USB from parent USB
biomeos spore clone /dev/sdb --from /dev/sdc --sibling tower2

# Internally:
1. Copy parent .family.seed from source USB
2. Generate new tower.toml with BEARDOG_NODE_ID = "tower2"
3. Copy binaries (beardog, songbird, toadstool)
4. Verify genetic lineage matches
```

**Result**: Both USBs share parent seed → siblings → auto-trust

---

### Phase 4: Add Verification Commands

**Commands**:
```bash
# Verify spore has correct structure
biomeos spore verify /dev/sdb

# Show genetic lineage info
biomeos spore info /dev/sdb
# Output:
#   Family ID: nat0
#   Genesis Hash: a1b2c3...
#   Node ID: tower1
#   Siblings: tower2, tower3
#   Created: 2026-01-07
```

---

## 🔐 Security Properties

### ✅ Achieved

1. **Parent Seed Never Leaves USB**
   - Stored in `.family.seed` (binary, 600 perms)
   - Read at boot, never copied to disk
   - Child keys derived in memory only

2. **Unique Per-Tower Identity**
   - Each tower derives unique child_seed
   - Cannot be linked without parent_seed
   - Privacy preserved

3. **Cryptographic Family Verification**
   - Same parent_seed → same family_id
   - Siblings mathematically verifiable
   - Cannot be forged

### ⚠️ To Implement

1. **Zeroization**
   - Parent seed must be zeroized after derivation
   - Child keys zeroized on exit
   - Memory protection (mlock if possible)

2. **Seed Rotation**
   - Generation numbers for key rotation
   - Forward secrecy
   - Revocation support

3. **Human Entropy** (Tier 2)
   - Phone HSM integration
   - Proof of human presence
   - Enhanced cryptographic strength

---

## 🚀 Quick Start (Using Existing Scripts)

### Creating First USB Spore

```bash
# 1. Format USB
sudo mkfs.ext4 -L biomeOS1 /dev/sdb1

# 2. Mount
mkdir -p /media/usb
sudo mount /dev/sdb1 /media/usb

# 3. Create family seed
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
./scripts/create-usb-family-seed.sh

# 4. Prepare spore
./scripts/prepare-usb-spore.sh /media/usb biomeOS1 tower1

# 5. Verify
./scripts/verify-usb-genetic-lineage.sh
```

### Creating Sibling Spore

```bash
# 1. Format second USB
sudo mkfs.ext4 -L biomeOS2 /dev/sdc1
sudo mount /dev/sdc1 /media/usb2

# 2. Copy parent seed from first USB
sudo cp /media/usb/.family.seed /media/usb2/.family.seed
sudo chmod 600 /media/usb2/.family.seed

# 3. Prepare spore (different node)
./scripts/prepare-usb-spore.sh /media/usb2 biomeOS2 tower2

# 4. Verify sibling relationship
./scripts/verify-usb-genetic-lineage.sh
# → Should show same genesis hash
```

---

## 📊 Current vs Target

| Feature | Current | Target |
|---------|---------|--------|
| **Seed Creation** | ✅ Script exists | 🎯 biomeOS CLI command |
| **Seed Storage** | ✅ .family.seed file | ✅ Already correct |
| **Config** | ⚠️ Hardcoded in TOML | 🎯 File reference |
| **Sibling Creation** | ⚠️ Manual copy | 🎯 Automated cloning |
| **Verification** | ✅ Script exists | 🎯 biomeOS CLI command |
| **Auto-Derivation** | ⚠️ Manual node_id | 🎯 Auto from hostname |
| **Zeroization** | ❌ Not implemented | 🎯 Memory protection |
| **Human Entropy** | ❌ Not implemented | 🎯 Tier 2 feature |

---

## 🎯 Next Steps

### Immediate (Today/Tomorrow)
1. ✅ Document current state (this file!)
2. 🎯 Update tower.toml to use BEARDOG_FAMILY_SEED_FILE
3. 🎯 Test with file reference instead of hardcoded seed
4. 🎯 Verify siblings still recognize each other

### Short-Term (This Week)
1. Add `biomeos spore` commands to CLI
2. Implement sibling cloning automation
3. Add spore verification to startup
4. Document standard workflow

### Medium-Term (Next Week)
1. Implement zeroization
2. Add seed rotation support
3. Create production deployment guide
4. Test with multiple families

### Long-Term (Future)
1. Phone HSM integration (Tier 2)
2. Hardware HSM support (Tier 3)
3. Cross-org federation testing
4. Revocation infrastructure

---

**Date**: January 7, 2026, 22:00 UTC  
**Status**: Infrastructure exists, needs integration  
**Priority**: High - Foundation for production deployment  
**Blocker**: None - can implement incrementally

