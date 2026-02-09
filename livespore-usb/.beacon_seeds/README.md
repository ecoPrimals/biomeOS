# Beacon Seeds Directory

**Purpose**: Stores encrypted beacon seeds from meetings

**Format**: `{beacon_id_prefix}.seed`

**Contents**: Each file contains the **actual beacon seed** (32 bytes) from someone we've met, encrypted at rest with our lineage key.

**Why encrypted?**
- Beacon seeds are secrets - they let us decrypt that person's broadcasts
- If device is lost, attacker can't read our "address book"
- Same-lineage devices can decrypt (for sync)

**File format**:
```
BEACON-SEED-V1\n     # Magic header
<16 bytes salt>      # Random salt
<12 bytes nonce>     # Random nonce
<32 bytes cipher>    # Encrypted seed
<16 bytes tag>       # Auth tag
```

**Encryption**: ChaCha20-Poly1305 AEAD with HKDF-derived key from lineage seed

**See**: `../specs/BEACON_GENETICS_BUILD_SPEC.md` for full architecture
