# 01 - Encrypted P2P Communication

**Demonstrates**: BirdSong encrypted messaging using BearDog  
**Status**: Live demonstration  
**Prerequisites**: Songbird + BearDog running  

---

## What This Demonstrates

- Discovering P2P coordination capabilities (Songbird)
- Discovering encryption capabilities (BearDog)
- Establishing encrypted communication channels
- Lineage-based message security
- BiomeOS as orchestration substrate

---

## Architecture

```
┌─────────────┐
│   BiomeOS   │ ← Discovers and orchestrates
└──────┬──────┘
       │
   ┌───┴────┐
   │        │
┌──▼──┐  ┌──▼────┐
│Bird │  │Bear   │
│Song │  │Dog    │
│P2P  │  │Crypto │
└─────┘  └───────┘
```

---

## Key Capabilities

1. **Capability Discovery**
   - Find orchestration primal (Songbird)
   - Find encryption primal (BearDog)
   - No hardcoded endpoints

2. **Channel Establishment**
   - Create secure P2P channel
   - BearDog encryption integration
   - Automatic key management

3. **Encrypted Messaging**
   - Send encrypted messages
   - Lineage verification
   - End-to-end security

---

## Running the Demo

```bash
bash showcase/02-birdsong-p2p/01-encrypted-p2p/demo.sh
```

---

## Expected Output

```
🎵 BirdSong P2P: Encrypted Communication
========================================
🔍 Discovering primals...
✅ Found Songbird (orchestration)
✅ Found BearDog (encryption)

🔐 Establishing encrypted P2P channel...
✅ Channel established

📨 Sending encrypted message...
✅ Message sent (encrypted with BearDog)

🔍 Verifying lineage enforcement...
✅ Sovereignty preserved

🎉 BirdSong P2P encryption demonstrated!
```

---

## What Success Looks Like

- ✅ Primals discovered automatically
- ✅ Encrypted channel created
- ✅ Message encrypted with BearDog
- ✅ Lineage verification working
- ✅ No hardcoded configuration

## Failure Modes (Expected)

- ⚠️ Songbird not running → Gap exposed in PRIMAL_GAPS.md
- ⚠️ BearDog not available → Gap exposed  
- ⚠️ Channel creation fails → Integration gap documented

**This is maturity**: We expose real gaps, not hide behind mocks!

---

**Next Demo**: 02 - Peer Discovery (mDNS automatic discovery)

