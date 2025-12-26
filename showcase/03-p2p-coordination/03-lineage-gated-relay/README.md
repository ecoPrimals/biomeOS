# Demo 03: Lineage-Gated Relay

**Time:** 30 minutes  
**Difficulty:** 🔴 Advanced  
**Status:** ✅ Ready to run

---

## 🎯 What This Demo Shows

This demo demonstrates **BiomeOS coordinating NAT traversal with lineage-based access control**.

**Lineage-Gated Relay:** *"Only family can use my relay"*

### Key Features

1. **NAT Traversal**
   - Coordinate secure connections through relays
   - Works even when nodes are behind firewalls
   - No port forwarding needed

2. **Lineage-Based Access Control**
   - Only family members can use your relay
   - Cryptographic trust (not IP-based)
   - No central authority needed

3. **Bandwidth Protection**
   - Limit bandwidth for relay users
   - Prevent abuse
   - Fair resource sharing

4. **Dynamic Relay Selection**
   - BiomeOS discovers multiple relay offers
   - Selects best relay based on lineage + performance
   - Automatic fallback if relay fails

---

## 🚀 Run the Demo

```bash
cargo run
```

---

## 📊 Expected Output

```
🌱 BiomeOS P2P Coordination Demo: Lineage-Gated Relay
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🔒 "Only family can use my relay"

📋 Scenario:
   Alice: Behind NAT, needs to connect to internet
   Bob: Public IP, willing to relay for family
   Carol: Alice's family member, needs relay
   Dave: Not family, wants relay (will be denied)

🔍 Step 1: Discovering primals by capability...
✅ Found security primal: MockSecurity (demonstrates BearDog)
✅ Found routing primal: MockRouting (demonstrates Songbird)

🚪 Step 2: Bob offers relay with lineage gate...
✅ Bob's relay offer created
   Endpoint: bob.example.com:9000
   Lineage Gate: family-root
   Bandwidth Limit: 10 Mbps

👨‍👩‍👧 Step 3: Carol (family) requests relay...
✅ Lineage verified: Carol is family!
✅ Relay connection established!
   Carol → Bob → Internet

👤 Step 4: Dave (not family) requests relay...
❌ Lineage verification failed: Dave is not family
   Relay request denied

📊 Step 5: Privacy Model Demonstration

🔒 Lineage-Gated Relay Benefits:
   ✅ Only family can use your resources
   ✅ No central authority needed
   ✅ Cryptographic trust (not IP-based)
   ✅ Bandwidth protection
   ✅ Automatic access control

🔄 Step 6: NAT Traversal Coordination
✅ NAT traversal complete with lineage-based access control

🎯 Step 7: Dynamic Relay Selection
BiomeOS selects Bob's relay:
   ✅ Family member (lineage verified)
   ✅ Best latency
   ✅ Sufficient bandwidth

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🎉 Demo complete!
```

---

## 🏗️ How Lineage-Gated Relay Works

### The Problem: NAT Traversal

```
Alice (Behind NAT)          Internet
     │                         │
     │  ❌ Cannot directly     │
     │     connect             │
     │                         │
     └─────── NAT ─────────────┘
```

### The Solution: Family Relay

```
Alice (Behind NAT)    Bob (Public IP)    Internet
     │                     │                 │
     │ 1. Request Relay   │                 │
     ├────────────────────>│                 │
     │                     │                 │
     │ 2. Verify Lineage  │                 │
     │<────────────────────┤                 │
     │                     │                 │
     │ 3. Establish Tunnel│                 │
     │<═══════════════════>│                 │
     │                     │                 │
     │ 4. Relay Traffic    │ 5. Forward     │
     │<════════════════════│<════════════════│
     │                     │                 │
     ✅ Connected!         ✅ Relaying       ✅
```

### Lineage Verification

```
Bob receives relay request from Carol:
   │
   ├─> Query BearDog: "Is Carol family?"
   │   └─> BearDog verifies lineage proof
   │       └─> Carol's lineage: family-root -> carol ✅
   │
   ├─> Accept relay request
   └─> Establish BTSP tunnel

Bob receives relay request from Dave:
   │
   ├─> Query BearDog: "Is Dave family?"
   │   └─> BearDog verifies lineage proof
   │       └─> Dave's lineage: stranger-lineage -> dave ❌
   │
   └─> Reject relay request
```

---

## 🔧 Key Concepts

### 1. Lineage-Based Access

**Not:** "Do you have the right IP address?"  
**But:** "Are you part of my family?"

**Benefits:**
- Cryptographic trust
- No IP whitelist management
- Works across networks
- Natural access control

### 2. NAT Traversal

**Traditional Approach:**
- Configure port forwarding
- Static IP addresses
- Firewall rules
- Manual setup

**BiomeOS Approach:**
- Discover relay offers
- Verify lineage
- Establish BTSP tunnel
- Automatic coordination

### 3. Bandwidth Protection

```yaml
relay_offer:
  bandwidth_limit: "10 Mbps"
  connection_limit: 5
  time_limit: "1 hour"
```

**Prevents:**
- Bandwidth abuse
- Resource exhaustion
- Unfair usage

### 4. Dynamic Relay Selection

BiomeOS discovers multiple relay offers and selects the best one:

```rust
let relay = coordinator
    .discover_relay_offers()
    .await?
    .filter(|r| security.verify_lineage(r.lineage_gate).await?)
    .min_by_key(|r| r.latency)?;
```

**Selection Criteria:**
1. **Lineage:** Must be family (or approved lineage)
2. **Latency:** Prefer low latency
3. **Bandwidth:** Must meet requirements
4. **Availability:** Must be online

---

## 🎯 Use Cases

### 1. Home Network NAT Traversal
- Family devices behind home router
- Relay through family member with public IP
- No port forwarding configuration

### 2. Mobile Devices
- Mobile device behind carrier NAT
- Relay through trusted family relay
- Seamless connectivity

### 3. Enterprise Networks
- Corporate firewall restrictions
- Relay through approved department relay
- Lineage = organizational hierarchy

### 4. IoT Mesh
- IoT devices behind NAT
- Relay through coordinator node
- Lineage = device ownership

---

## 🚀 Deploy with BYOB

**File:** `templates/lineage-gated-relay.biome.yaml`

```yaml
primals:
  - capability: "security"
    features: ["lineage", "verification"]
    preferred: "beardog"

  - capability: "routing"
    features: ["relay", "nat-traversal"]
    preferred: "songbird"

coordination:
  relay:
    enabled: true
    lineage_gate: "family-root"
    
    policies:
      bandwidth_limit: "10 Mbps"
      connection_limit: 5
      require_lineage: true
```

**Deploy:**
```bash
biomeos deploy templates/lineage-gated-relay.biome.yaml
```

---

## 🔗 Related Demos

- **Demo 01:** BTSP Tunnel Coordination (secure tunnels)
- **Demo 02:** BirdSong Encryption (privacy-preserving discovery)
- **Demo 04:** Multi-Tower P2P (distributed mesh)

---

## 📚 Further Reading

**Songbird Showcase:**
- `../../songbird/showcase/13-beardog-integration/` - Relay coordination

**BearDog Showcase:**
- `../../beardog/showcase/02-ecosystem-integration/` - Lineage verification

**Specifications:**
- `specs/CROSS_PRIMAL_API_CONTRACTS.md` - Relay API contracts

---

## 🔒 Security Considerations

### Relay Trust Model

**Trust Chain:**
```
User trusts relay → Relay verifies lineage → BearDog validates proof
```

**Relay cannot:**
- See plaintext (BTSP encryption)
- Modify traffic (authenticated encryption)
- Impersonate nodes (lineage proof)

**Relay can:**
- See traffic metadata (source/dest)
- Enforce bandwidth limits
- Refuse service

### Lineage Proof Security

**Properties:**
- Cryptographically signed
- Cannot be forged
- Cannot be replayed
- Time-limited validity

---

**This is Lineage-Gated Relay: NAT traversal with family-based access control!** 🔒🔗

