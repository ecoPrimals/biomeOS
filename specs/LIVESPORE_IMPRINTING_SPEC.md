# LiveSpore Imprinting Specification

**Version**: 1.0.0  
**Date**: 2026-01-27  
**Status**: Implemented

## Overview

LiveSpore imprinting is the process of creating a deployable biomeOS instance on removable media (USB/SD card) with proper genetic lineage. This specification standardizes the imprinting process to ensure:

1. **Genetic Lineage**: Cryptographically verifiable family relationships
2. **Binary Integrity**: Fresh binaries from plasmidBin (phase1 harvest)
3. **Deterministic Discovery**: Dark Forest encrypted beacons

## Seed Structure (64 bytes)

```
┌─────────────────────┬─────────────────────┐
│   genesis_seed      │     node_key        │
│     (32 bytes)      │    (32 bytes)       │
├─────────────────────┼─────────────────────┤
│ Shared family root  │ Blake3(genesis,     │
│ Same for all        │ "node-identity-v1"  │
│ siblings            │ + node_id)          │
└─────────────────────┴─────────────────────┘
```

### Genesis Seed (bytes 0-31)
- Shared by all family members
- Used for Dark Forest broadcast key derivation
- Created once for genesis node, inherited by siblings

### Node Key (bytes 32-63)
- Unique per node
- Derived deterministically: `Blake3(genesis, "node-identity-v1:" + node_id)`
- Enables unique identity while proving family membership

## Environment Variables

The deploy.sh script exports these for BearDog:

| Variable | Content | Purpose |
|----------|---------|---------|
| `BEARDOG_FAMILY_SEED` | Base64 of full 64-byte seed | Full lineage identity |
| `BEARDOG_GENESIS_SEED` | Base64 of first 32 bytes | Dark Forest broadcast key |
| `BEARDOG_FAMILY_ID` | Family name (e.g., "nat0") | Family identification |
| `BEARDOG_NODE_ID` | Node name (e.g., "node-alpha") | Node identification |
| `BEARDOG_SOCKET` | Socket path | IPC endpoint |

## Broadcast Key Derivation

All family members derive the **same** broadcast key using Blake3:

```json
{
  "jsonrpc": "2.0",
  "method": "crypto.blake3_hash",
  "params": {
    "data": "<base64 genesis_seed>",
    "context": "dark_forest_broadcast_v1"
  },
  "id": 1
}
```

This enables:
- Encrypted Dark Forest beacons
- Only family members can decrypt
- Deterministic - no key exchange needed

## Directory Structure

```
/media/*/biomeOS/
├── .family.seed          # 64-byte genetic seed
├── .spore.json           # Spore metadata
├── deploy.sh             # Standardized deployment script
├── tower.toml            # Tower configuration
├── primals/
│   ├── beardog           # BearDog binary (crypto/trust)
│   └── songbird          # Songbird binary (HTTP/TLS)
├── graphs/               # Neural API graphs
├── logs/                 # Runtime logs
├── config/               # Additional configuration
└── certs/                # TLS certificates
```

## Imprinting Methods

### 1. Neural API Graph (Recommended)

```bash
neural-api execute graphs/livespore_create.toml \
  --env SPORE_TARGET=/media/user/USB \
  --env LINEAGE_MODE=sibling \
  --env PARENT_SEED_PATH=/media/parent/.family.seed \
  --env NODE_ID=node-gamma
```

### 2. Manual Imprinting

```bash
# 1. Create directory structure
mkdir -p /media/user/USB/biomeOS/{primals,graphs,logs,config,certs}

# 2. Copy binaries from plasmidBin
cp plasmidBin/primals/beardog/beardog-active /media/user/USB/biomeOS/primals/beardog
cp plasmidBin/primals/songbird/songbird-active /media/user/USB/biomeOS/primals/songbird

# 3. Create/derive seed
# For genesis:
head -c 32 /dev/urandom > /tmp/genesis.seed
# Derive node key via BearDog Blake3
# Concatenate: cat /tmp/genesis.seed /tmp/node.key > .family.seed

# For sibling:
head -c 32 /media/parent/biomeOS/.family.seed > /tmp/genesis.seed
# Derive different node key
# Concatenate

# 4. Copy deploy.sh template
cp templates/livespore_deploy.sh /media/user/USB/biomeOS/deploy.sh
# Substitute __FAMILY_ID__, __NODE_ID__, __SPORE_VERSION__
```

## Lineage Modes

| Mode | Genesis Seed | Node Key | Use Case |
|------|--------------|----------|----------|
| `genesis` | New random 32 bytes | Derived | First node in family |
| `sibling` | Copied from parent (bytes 0-31) | Derived | Additional nodes |
| `clone` | Copied entirely | Same | ⚠️ Discouraged - no unique identity |

## Verification

### Verify Seed Structure
```bash
SEED_SIZE=$(stat -c%s .family.seed)
if [[ "$SEED_SIZE" -ne 64 ]]; then
  echo "Invalid seed size"
  exit 1
fi
```

### Verify Family Membership
```bash
# All family members derive same broadcast key from genesis
GENESIS=$(head -c 32 .family.seed | base64 -w0)
KEY=$(echo '{"jsonrpc":"2.0","method":"crypto.blake3_hash","params":{"data":"'$GENESIS'","context":"dark_forest_broadcast_v1"},"id":1}' | nc -U $BEARDOG_SOCKET)
# Compare with expected family broadcast key
```

## BearDog API Requirements

Current BearDog v0.9.0 supports:

| Method | Status | Purpose |
|--------|--------|---------|
| `crypto.blake3_hash` | ✅ | Broadcast key derivation |
| `crypto.chacha20_poly1305_encrypt` | ✅ | Beacon encryption |
| `crypto.chacha20_poly1305_decrypt` | ✅ | Beacon decryption |
| `genetic.derive_lineage_key` | ✅ | Session key derivation |

Future evolution (handoff to BearDog team):

| Method | Priority | Purpose |
|--------|----------|---------|
| `genetic.verify_seed` | High | Validate 64-byte structure |
| `genetic.register_sibling` | Medium | Lineage chain tracking |
| `BEARDOG_FAMILY_SEED_FILE` | Low | Read seed from file (env var works) |

## Example Deployment

```bash
# On USB (node-beta)
cd /media/eastgate/BEA6-BBCE/biomeOS
./deploy.sh

# Output:
# 🌱 LiveSpore Deployment
# Family:  nat0
# Node:    node-beta
# 🧬 Verifying genetic lineage...
# ✅ Genetic lineage verified (64 bytes)
# 🐕 Starting BearDog...
# ✅ BearDog started (PID: 12345)
# 🐦 Starting Songbird...
# ✅ Songbird started (PID: 12346)
```

## Files

| Path | Description |
|------|-------------|
| `templates/livespore_deploy.sh` | Standardized deploy script template |
| `graphs/livespore_create.toml` | Neural API graph for imprinting |
| `specs/LIVESPORE_IMPRINTING_SPEC.md` | This specification |
| `plasmidBin/primals/` | Source binaries for deployment |

## Validated Deployments

| USB | Node ID | Seed | Broadcast Key |
|-----|---------|------|---------------|
| biomeOS1 | node-alpha | 64 bytes ✅ | TSfKMoujtx... |
| BEA6-BBCE | node-beta | 64 bytes ✅ | TSfKMoujtx... |
| BEA6-BBCE1 | node-epsilon | 64 bytes ✅ | TSfKMoujtx... |

All three nodes derive the **same broadcast key**, confirming shared genesis lineage.

