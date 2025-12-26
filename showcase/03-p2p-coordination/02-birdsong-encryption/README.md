# Demo 02: BirdSong Encrypted Discovery

**Time:** 30 minutes  
**Difficulty:** 🔴 Advanced  
**Status:** ✅ Ready to run

---

## 🎯 What This Demo Shows

This demo demonstrates **BiomeOS coordinating BirdSong (privacy-preserving discovery)** in pure Rust.

**BirdSong:** *"A broadcast that is obvious to family and noise otherwise"*

### Key Features

1. **Privacy-Preserving Discovery**
   - Family members (verified lineage) can see services
   - Others see only encrypted noise
   - No IP address exposure in cleartext

2. **Lineage-Based Access Control**
   - "Does this node descend from me?"
   - Cryptographic lineage verification
   - No central authority needed

3. **Pure Rust Coordination**
   - BiomeOS coordinates security + discovery primals
   - All logic in Rust (not shell scripts)
   - Type-safe, production-ready

4. **Graceful Degradation**
   - Encrypted mode for untrusted networks
   - Plaintext mode for trusted LAN
   - Auto-detection and switching

---

## 🚀 Run the Demo

```bash
cargo run
```

---

## 📊 Expected Output

```
🌱 BiomeOS P2P Coordination Demo: BirdSong Encryption
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🎵 "A broadcast that is obvious to family and noise otherwise"

🔍 Step 1: Discovering primals by capability...
   Looking for: security capability (BirdSong support)
   Looking for: discovery capability (encrypted mode)

✅ Found security primal: MockSecurity (demonstrates BearDog)
✅ Found discovery primal: MockDiscovery (demonstrates Songbird)

🔐 Step 2: Creating BirdSong coordinator...
✅ Coordinator created

🎵 Step 3: Enabling BirdSong encrypted discovery...
   Family ID: demo-family

   Generating broadcast keys from security primal...
   Configuring discovery primal for encrypted mode...
   Testing encryption is working...

✅ BirdSong encryption enabled successfully!

📊 Discovery Mode:
   Mode: Encrypted
   Privacy: HIGH (encrypted broadcasts)
   Visibility: Family-only (lineage-verified)

📊 Step 4: Privacy Model Demonstration

🔒 For Family Members (verified lineage):
   ✅ Can decrypt service broadcasts
   ✅ Can discover services
   ✅ Can see node details
   ✅ Can connect to services

👁️  For Others (non-family):
   ❌ See only encrypted noise
   ❌ Cannot discover services
   ❌ Cannot see node details
   ❌ Cannot connect to services

📡 Step 5: Discovery Examples

Without BirdSong (Plaintext):
   Observer sees:
   - Node IDs: node-a, node-b, node-c
   - IP Addresses: 192.168.1.100, 192.168.1.101...
   - Services: web-server, database, api
   - Capabilities: All visible
   Privacy Level: ❌ LOW (everything visible)

With BirdSong (Encrypted):
   Observer sees:
   - Encrypted data: [random bytes]
   - No IP addresses visible
   - No service names visible
   - No capabilities visible
   Privacy Level: ✅ HIGH (selective visibility)

🔄 Step 6: Graceful Degradation

BiomeOS supports both modes:
   • BirdSong (Encrypted): For internet/untrusted networks
   • Plaintext: For trusted LAN (faster, zero-config)

Current mode: Encrypted

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🎉 Demo complete!

Key Takeaways:
  ✅ BiomeOS coordinated BirdSong encryption in pure Rust
  ✅ Privacy-preserving discovery (family sees, others don't)
  ✅ Lineage-based access control
  ✅ Graceful degradation (encrypted or plaintext)

Next Steps:
  - Run demo 03: Lineage-Gated Relay
  - Deploy with BYOB: templates/birdsong-discovery.biome.yaml
  - Test with real BearDog + Songbird
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## 🏗️ How BirdSong Works

### The Privacy Model

```
Traditional Discovery (Plaintext):
┌─────────────────────────────────────┐
│ Broadcast: "I am node-a at         │
│            192.168.1.100:8080"     │
└─────────────────────────────────────┘
         │
         ├─► Family sees: Everything ✅
         └─► Others see: Everything ❌ (Privacy violation!)

BirdSong Discovery (Encrypted):
┌─────────────────────────────────────┐
│ Broadcast: [encrypted bytes]        │
│ Lineage: [cryptographic proof]     │
└─────────────────────────────────────┘
         │
         ├─► Family sees: Everything ✅ (decrypt with lineage)
         └─► Others see: Noise ✅ (cannot decrypt)
```

### Coordination Flow

```
BiomeOS
   │
   ├─► Discover "security" capability (BearDog)
   │   └─► Request broadcast keys for family
   │
   ├─► Discover "discovery" capability (Songbird)
   │   └─► Configure encrypted mode
   │
   ├─► coordinator.enable_encrypted_discovery("family-id")
   │   ├─► security.generate_broadcast_keys()
   │   ├─► discovery.enable_encrypted_mode()
   │   └─► discovery.test_encrypted_broadcast()
   │
   └─► Return DiscoveryMode::Encrypted
```

---

## 🔧 Key Concepts

### 1. Lineage-Based Access

**Not:** "Do I trust this certificate authority?"  
**But:** "Does this node descend from me?"

**Benefits:**
- No central authority
- Cryptographic trust
- Selective visibility
- Natural access control

### 2. Privacy-Preserving Discovery

**Traditional P2P:**
- IP addresses visible
- Service names visible
- Capabilities visible
- **Privacy:** Low

**BirdSong:**
- Everything encrypted
- Lineage required to decrypt
- Selective visibility
- **Privacy:** High

### 3. Graceful Degradation

```rust
// BiomeOS supports both modes
match environment {
    TrustedLAN => DiscoveryMode::Plaintext,  // Fast, zero-config
    Internet => DiscoveryMode::Encrypted,     // Privacy-preserving
}
```

---

## 🎯 Use Cases

### 1. Friend Networks
- Share services with friends only
- Others can't even see you have services
- No central server needed

### 2. Family Mesh
- Family devices discover each other
- Strangers on same network see nothing
- Natural trust boundaries

### 3. Enterprise Networks
- Department-level access control
- Lineage = organizational hierarchy
- No certificate infrastructure

### 4. IoT Mesh
- Devices discover related devices
- Untrusted devices see noise
- Self-organizing networks

---

## 🚀 Deploy with BYOB

**File:** `templates/birdsong-discovery.biome.yaml`

```yaml
primals:
  - capability: "security"
    features: ["birdsong", "lineage"]
    preferred: "beardog"

  - capability: "discovery"
    features: ["encrypted", "mesh"]
    preferred: "songbird"

coordination:
  birdsong:
    enabled: true
    family_id: "my-family"
    
    privacy:
      encrypt_broadcasts: true
      hide_ip_addresses: true
      filter_by_lineage: true
```

**Deploy:**
```bash
biomeos deploy templates/birdsong-discovery.biome.yaml
```

---

## 🔗 Related Demos

- **Demo 01:** BTSP Tunnel Coordination (secure tunnels)
- **Demo 03:** Lineage-Gated Relay (NAT traversal)
- **Demo 04:** Multi-Tower P2P (distributed mesh)

---

## 📚 Further Reading

**Songbird Showcase:**
- `../../songbird/showcase/13-beardog-integration/` - Songbird + BearDog patterns
- `../../songbird/showcase/02-federation/` - Multi-tower federation

**BearDog Showcase:**
- `../../beardog/showcase/00-local-primal/` - BearDog fundamentals

**Specifications:**
- `specs/CROSS_PRIMAL_API_CONTRACTS.md` - API contracts

---

**This is BirdSong: Privacy-preserving discovery with lineage-based access control!** 🎵

