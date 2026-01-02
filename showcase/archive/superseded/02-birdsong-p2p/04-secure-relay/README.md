# 04 - Secure Relay

**Demonstrates**: Lineage-gated message routing  
**Status**: Architecture demonstration  
**Prerequisites**: Songbird + BearDog  

---

## What This Demonstrates

- Lineage verification for message routing
- Secure peer-to-peer relay
- Access control enforcement
- Audit logging
- Sovereignty at the network layer

---

## Architecture

```
Sender ──[lineage proof]──► Relay ──[verify]──► Recipient
                              │
                           BearDog
                          (Lineage
                           Verifier)
```

---

## Key Capabilities

1. **Lineage Verification**
   - BearDog generates lineage proofs
   - Songbird verifies before relay
   - Unauthorized access denied

2. **Secure Routing**
   - Only authorized peers can send
   - Only authorized peers can receive
   - Full audit trail

3. **Sovereignty Enforcement**
   - User controls who can relay messages
   - Geographic boundaries respected
   - Human dignity preserved

---

## Running the Demo

```bash
bash showcase/02-birdsong-p2p/04-secure-relay/demo.sh
```

---

**Next Demo**: 05 - Full Ecosystem (all primals coordinated)

