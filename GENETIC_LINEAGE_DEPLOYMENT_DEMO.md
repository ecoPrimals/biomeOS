# 🧬 Genetic Lineage Deployment Demo

**Concept**: Deploy all 3 atomics from a single seed key, enabling cryptographic lineage recognition and secure cooperation.

---

## 🎯 What We're Testing

**BearDog Genetic Lineage System**:
- Single family seed → Multiple atomic deployments
- Each atomic gets unique keys derived from the same lineage
- Automatic recognition: "We're family, we can cooperate"
- Cross-atomic encryption based on shared genetic heritage

**The Vision**:
```
USB Seed (genesis key)
    ↓
    ├─→ Tower BearDog (lineage: family/tower)
    ├─→ Node BearDog (lineage: family/node)
    └─→ Nest BearDog (lineage: family/nest)
         ↓
    All recognize shared ancestry
    All can cooperate securely
```

---

## 🔧 Setup: USB Seed Key

### Step 1: Create Family Genesis Seed

```bash
# Create a USB-based seed directory
USB_PATH="/media/usb0/biomeos-seed"  # Or your USB mount point
mkdir -p "$USB_PATH"

# Generate family genesis key
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS

# Use BearDog to generate a family seed
export BEARDOG_FAMILY_SEED="$USB_PATH/family-genesis.key"

# Generate the seed (this should be done once and kept secure)
./plasmidBin/primals/beardog-server generate-family-seed \
    --output "$BEARDOG_FAMILY_SEED" \
    --family-id "nat0"

# Backup the seed (CRITICAL!)
cp "$BEARDOG_FAMILY_SEED" "$USB_PATH/family-genesis.backup.key"
```

### Step 2: Verify Seed

```bash
# Check seed file exists
ls -lh "$USB_PATH/family-genesis.key"

# Expected: 64-byte file (512-bit seed)
```

---

## 🚀 Deployment: All 3 Atomics from USB Seed

### Tower Atomic Deployment

```bash
#!/bin/bash
# deploy_tower_from_usb.sh

USB_SEED="/media/usb0/biomeos-seed/family-genesis.key"
FAMILY_ID="nat0"
UID=$(id -u)

echo "🏰 Deploying Tower Atomic from USB seed..."

# BearDog (encryption foundation)
BEARDOG_SOCKET="/run/user/$UID/beardog-tower.sock" \
BEARDOG_FAMILY_ID="$FAMILY_ID" \
BEARDOG_FAMILY_SEED="$USB_SEED" \
BEARDOG_INSTANCE_ID="tower" \
./plasmidBin/primals/beardog-server &

# Wait for BearDog to initialize
sleep 2

# Songbird (discovery, using BearDog for security)
SONGBIRD_SOCKET="/run/user/$UID/songbird-tower.sock" \
SONGBIRD_FAMILY_ID="$FAMILY_ID" \
SONGBIRD_SECURITY_PROVIDER="/run/user/$UID/beardog-tower.sock" \
./plasmidBin/primals/songbird-orchestrator &

echo "✅ Tower deployed with genetic lineage: $FAMILY_ID/tower"
```

### Node Atomic Deployment

```bash
#!/bin/bash
# deploy_node_from_usb.sh

USB_SEED="/media/usb0/biomeos-seed/family-genesis.key"
FAMILY_ID="nat0"
UID=$(id -u)

echo "🖥️  Deploying Node Atomic from USB seed..."

# BearDog (derived from same seed, different instance)
BEARDOG_SOCKET="/run/user/$UID/beardog-node.sock" \
BEARDOG_FAMILY_ID="$FAMILY_ID" \
BEARDOG_FAMILY_SEED="$USB_SEED" \
BEARDOG_INSTANCE_ID="node" \
./plasmidBin/primals/beardog-server &

sleep 2

# Songbird
SONGBIRD_SOCKET="/run/user/$UID/songbird-node.sock" \
SONGBIRD_FAMILY_ID="$FAMILY_ID" \
SONGBIRD_SECURITY_PROVIDER="/run/user/$UID/beardog-node.sock" \
./plasmidBin/primals/songbird-orchestrator &

# ToadStool
TOADSTOOL_SOCKET="/run/user/$UID/toadstool-node.sock" \
TOADSTOOL_FAMILY_ID="$FAMILY_ID" \
./plasmidBin/toadstool &

echo "✅ Node deployed with genetic lineage: $FAMILY_ID/node"
```

### Nest Atomic Deployment

```bash
#!/bin/bash
# deploy_nest_from_usb.sh

USB_SEED="/media/usb0/biomeos-seed/family-genesis.key"
FAMILY_ID="nat0"
UID=$(id -u)

echo "🏠 Deploying Nest Atomic from USB seed..."

# BearDog (derived from same seed, different instance)
BEARDOG_SOCKET="/run/user/$UID/beardog-nest.sock" \
BEARDOG_FAMILY_ID="$FAMILY_ID" \
BEARDOG_FAMILY_SEED="$USB_SEED" \
BEARDOG_INSTANCE_ID="nest" \
./plasmidBin/primals/beardog-server &

sleep 2

# Songbird
SONGBIRD_SOCKET="/run/user/$UID/songbird-nest.sock" \
SONGBIRD_FAMILY_ID="$FAMILY_ID" \
SONGBIRD_SECURITY_PROVIDER="/run/user/$UID/beardog-nest.sock" \
./plasmidBin/primals/songbird-orchestrator &

# NestGate
NESTGATE_SOCKET="/run/user/$UID/nestgate-nest.sock" \
NESTGATE_FAMILY_ID="$FAMILY_ID" \
./plasmidBin/primals/nestgate &

echo "✅ Nest deployed with genetic lineage: $FAMILY_ID/nest"
```

---

## 🧬 How Genetic Lineage Works

### Key Derivation

```
Family Seed (USB)
    ↓
HKDF (HMAC-based Key Derivation Function)
    ↓
    ├─→ Tower Keys = HKDF(seed, "nat0/tower")
    ├─→ Node Keys = HKDF(seed, "nat0/node")
    └─→ Nest Keys = HKDF(seed, "nat0/nest")
```

### Lineage Recognition

Each BearDog instance can:
1. **Derive its own keys** from seed + instance ID
2. **Recognize siblings** by verifying shared family ID
3. **Establish trust** through cryptographic lineage proof
4. **Enable cooperation** without explicit key exchange

### Example Recognition Flow

```
Tower BearDog: "I'm from family nat0, instance tower"
Node BearDog:  "I'm from family nat0, instance node"

Tower: "We share lineage! Here's my proof..."
Node:  "Verified! We can cooperate securely."

Result: Encrypted channel established automatically
```

---

## ✅ Verification Tests

### Test 1: Lineage Recognition

```bash
# Query each BearDog for its lineage
curl --unix-socket /run/user/$(id -u)/beardog-tower.sock \
    http://localhost/lineage

curl --unix-socket /run/user/$(id -u)/beardog-node.sock \
    http://localhost/lineage

curl --unix-socket /run/user/$(id -u)/beardog-nest.sock \
    http://localhost/lineage

# Expected: All show family_id: "nat0", different instance_ids
```

### Test 2: Cross-Atomic Recognition

```bash
# Tower recognizes Node
curl --unix-socket /run/user/$(id -u)/beardog-tower.sock \
    -d '{"jsonrpc":"2.0","method":"verify_sibling","params":{"socket":"/run/user/'$(id -u)'/beardog-node.sock"},"id":1}' \
    http://localhost

# Expected: {"result": {"verified": true, "family": "nat0", "instance": "node"}}
```

### Test 3: Secure Cooperation

```bash
# Tower encrypts message for Node
curl --unix-socket /run/user/$(id -u)/beardog-tower.sock \
    -d '{"jsonrpc":"2.0","method":"encrypt_for_sibling","params":{"message":"Hello Node!","target":"node"},"id":1}' \
    http://localhost

# Node decrypts message from Tower
# (Uses shared lineage to derive decryption key)
```

### Test 4: Cross-Atomic Communication

```bash
# Use Songbird to discover all family members
curl --unix-socket /run/user/$(id -u)/songbird-tower.sock \
    -d '{"jsonrpc":"2.0","method":"discover_family","params":{"family_id":"nat0"},"id":1}' \
    http://localhost

# Expected: List of all 3 atomics (tower, node, nest)
```

---

## 🎯 What This Demonstrates

### 1. **Genetic Lineage Security**
- ✅ No pre-shared keys needed
- ✅ Automatic trust establishment
- ✅ Cryptographically verifiable family relationships
- ✅ Unique keys per instance, shared heritage

### 2. **USB-Based Seed Portability**
- ✅ Single seed deploys entire ecosystem
- ✅ Seed can be backed up securely
- ✅ Lost atomic? Redeploy from seed with same lineage
- ✅ Multi-machine deployment with shared family

### 3. **Cross-Atomic Cooperation**
- ✅ Tower, Node, Nest recognize each other
- ✅ Secure channels established automatically
- ✅ Resource sharing without manual configuration
- ✅ Coordinated behavior through Songbird discovery

### 4. **Fault Tolerance**
- ✅ One atomic fails? Others continue
- ✅ New atomic joins? Recognized immediately
- ✅ Seed backup enables disaster recovery
- ✅ Genetic proof prevents imposters

---

## 🔐 Security Properties

### Cryptographic Guarantees

1. **Lineage Proof**: 
   - Each instance can prove family membership
   - Derived keys are cryptographically bound to seed

2. **Forward Secrecy**:
   - Even with one instance compromised, others remain secure
   - Session keys derived independently

3. **Imposter Prevention**:
   - Cannot join family without seed
   - Cannot fake lineage proofs

4. **Key Isolation**:
   - Tower keys ≠ Node keys ≠ Nest keys
   - Compromise of one doesn't compromise others

---

## 🚀 Production Deployment Scenario

### Scenario: Deploy Distributed NUCLEUS

```bash
# USB seed: /media/usb0/biomeos-seed/family-genesis.key

# Machine 1 (Tower - Security Gateway)
./deploy_tower_from_usb.sh

# Machine 2 (Node - Compute)
./deploy_node_from_usb.sh

# Machine 3 (Nest - Storage)
./deploy_nest_from_usb.sh

# Result: 3 machines, same family, automatic cooperation
```

### Benefits

- **Zero Configuration**: No manual key exchange
- **Automatic Discovery**: Songbird finds all siblings
- **Secure by Default**: Genetic lineage enforces trust
- **Scalable**: Add more atomics with same seed
- **Portable**: USB seed works anywhere

---

## 📊 Lineage Mixing Test Plan

### Phase 1: Deploy All 3 Atomics
- [x] Create USB seed
- [ ] Deploy Tower
- [ ] Deploy Node  
- [ ] Deploy Nest
- [ ] Verify all running

### Phase 2: Lineage Verification
- [ ] Query each BearDog for lineage info
- [ ] Verify family ID consistency
- [ ] Verify unique instance IDs
- [ ] Check cryptographic proofs

### Phase 3: Cross-Recognition
- [ ] Tower recognizes Node
- [ ] Tower recognizes Nest
- [ ] Node recognizes Tower & Nest
- [ ] Nest recognizes Tower & Node
- [ ] All 6 pairwise verifications pass

### Phase 4: Cooperation Tests
- [ ] Cross-atomic encryption
- [ ] Songbird family discovery
- [ ] Resource sharing coordination
- [ ] ToadStool cross-atomic compute
- [ ] NestGate cross-atomic storage

### Phase 5: Fault Tolerance
- [ ] Kill one atomic, others continue
- [ ] Restart atomic, rejoins family
- [ ] Deploy 4th atomic (same seed)
- [ ] Verify new atomic recognized

---

## 🎊 Expected Results

**Genetic Lineage System Working**:
- ✅ All atomics derive keys from USB seed
- ✅ Each recognizes others as siblings
- ✅ Secure cooperation without manual config
- ✅ Cryptographic lineage verifiable
- ✅ Family-based trust model operational

**This demonstrates**:
- BearDog's genetic key system
- Automatic trust establishment
- Cross-atomic cooperation
- Production-ready security

---

**Different orders of the same architecture.** 🍄🐸

**Status**: Ready to test genetic lineage deployment!

