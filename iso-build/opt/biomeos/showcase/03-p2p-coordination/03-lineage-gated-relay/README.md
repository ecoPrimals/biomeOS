# 03 - Lineage-Gated Relay

**Demonstrates**: Sovereign data routing with lineage verification  
**Status**: Ready to build  
**Prerequisites**: BearDog lineage, Songbird relay, NestGate storage  

---

## What This Demonstrates

- Lineage-based relay authorization
- Multi-hop secure routing
- Geographic sovereignty policies
- Audit trail at every hop
- Privacy-preserving relay
- Sovereign data control

---

## Architecture

```
Source          Relay 1         Relay 2      Destination
  │               │               │              │
  ├─[Lineage]────►│               │              │
  │               ├─[Verify]      │              │
  │               │  BearDog      │              │
  │               ├─[Lineage]────►│              │
  │               │               ├─[Verify]     │
  │               │               │  BearDog     │
  │               │               ├─[Lineage]───►│
  │               │               │              ├─[Verify]
  │               │               │              │  BearDog
  │◄──────────────┴───────────────┴──────────────┤
                    [Ack Trail]
```

---

## Lineage-Gated Routing

### Why Lineage Matters

Traditional routing: "Can I reach destination?"  
Lineage-gated: "Am I **authorized** to relay this data?"

### Sovereignty Properties

1. **Data Owner Control**: Only authorized relays
2. **Geographic Policies**: Route through approved regions
3. **Lineage Verification**: Each hop proves authorization
4. **Audit Trail**: Complete routing history
5. **Privacy**: Relays can't read content

---

## Demo Flow

```bash
# Phase 1: Setup
1. Generate lineage proofs (BearDog)
2. Configure relay policies
3. Define authorized relays
4. Set geographic constraints

# Phase 2: Authorized Relay
5. Source sends with lineage
6. Relay 1 verifies lineage ✅
7. Relay 1 forwards to Relay 2
8. Relay 2 verifies lineage ✅
9. Relay 2 forwards to destination
10. Destination verifies lineage ✅

# Phase 3: Unauthorized Attempt
11. Malicious relay intercepts
12. Lineage verification fails ❌
13. Relay blocked
14. Alert generated

# Phase 4: Audit
15. Review complete relay trail
16. Verify all hops authorized
17. Confirm geographic compliance
```

---

## Key Capabilities

### Lineage Proofs (BearDog)
- Cryptographic proof of authorization
- Hierarchical lineage chains
- Revocation support
- Geographic tagging

### Relay Policies
- Whitelist of authorized relays
- Geographic constraints
- Data sensitivity levels
- Expiration times

### Privacy Preservation
- Relays can't read content
- End-to-end encryption maintained
- Zero-knowledge routing
- Minimal metadata exposure

---

## Use Cases

### 1. Healthcare Data
```
Doctor → Hospital Relay → Insurance → Patient
        (HIPAA compliant)  (approved)
```

### 2. Financial Transactions
```
Bank A → Clearing House → Bank B
        (regulated)      (trusted)
```

### 3. Government Communications
```
Agency A → Secure Relay → Agency B
          (classified)   (clearance verified)
```

---

## Geographic Sovereignty

### Policy Example
```yaml
data:
  origin: US
  sensitivity: high
  
routing:
  allowed_regions: [US, CA, UK]
  forbidden_regions: [*]
  
relays:
  - relay-us-1 (Virginia, US)  ✅
  - relay-ca-1 (Toronto, CA)    ✅
  - relay-cn-1 (Beijing, CN)    ❌ Blocked
```

---

## Running the Demo

```bash
bash showcase/03-p2p-coordination/03-lineage-gated-relay/demo.sh
```

---

**Status**: 📋 Ready to Implement  
**Next**: Create demo.sh with lineage verification  
**Goal**: Demonstrate sovereign data routing  

🛡️ **Lineage-Gated Relay: Data Sovereignty in Action**
