# 🌱 Spore Incubation & Hierarchical Federation

**Date:** January 8, 2026  
**Status:** 🎯 **Design Phase - Next Evolution**

---

## 🎊 Vision: Distributed Family Networks

Create a system where USB spores can be:
1. **Incubated** on local computers (mixing spore seed + local entropy)
2. **Distributed** to family, friends, schools, organizations
3. **Federated** hierarchically (family trust + granular sub-federations)
4. **Recognized** across the network (genetic lineage NAT via BirdSong)

---

## 🏗️ Architecture Overview

### Concept: Spore + Local Entropy = Deployed Node

```
USB Spore (Genetic Seed)
    │
    ├─ Plug into Computer A
    │  └─ Mix: Spore Seed + Computer A Entropy → Node-A-Instance
    │     - Creates local config on Computer A
    │     - Spore remains portable
    │
    ├─ Eject & Plug into Computer B
    │  └─ Mix: Spore Seed + Computer B Entropy → Node-B-Instance
    │     - Creates NEW local config on Computer B
    │     - Both instances share spore lineage
    │
    └─ Spore logs track all deployments
       - Computer A deployment (timestamp, hostname, entropy hash)
       - Computer B deployment (timestamp, hostname, entropy hash)
```

### Genetic Lineage + Local Identity

```
Parent Seed (genesis)
  │
  ├─ Spore-Alpha (LiveSpore)
  │  ├─ Deployed on Computer-1 → Node-Alpha-C1 (family trust + local identity)
  │  ├─ Deployed on Computer-2 → Node-Alpha-C2 (family trust + local identity)
  │  └─ Both instances recognize each other as siblings
  │
  ├─ Spore-Beta (LiveSpore)
  │  ├─ Deployed on Computer-3 → Node-Beta-C3
  │  └─ Deployed on Computer-4 → Node-Beta-C4
  │
  └─ Spore-Gamma (ColdSpore - Archive)
     └─ No deployment, backup only
```

**Key Insight**: Same spore on different computers = Related but distinct nodes

---

## 🌐 Use Cases

### 1. Family & Friends Network

**Setup**:
```bash
# Create parent spore (you keep this)
biomeos spore create --mount /media/usb-parent --label ParentSeed --node genesis

# Create 5 sibling LiveSpores from parent
for i in alpha beta gamma delta epsilon; do
    biomeos spore clone \
        --from /media/usb-parent \
        --to /media/usb-$i \
        --node $i \
        --deployment-batch "family-2026"
done

# Create 1 ColdSpore for backup
biomeos spore clone \
    --from /media/usb-parent \
    --to /media/usb-archive \
    --node archive \
    --spore-type cold
```

**Distribution**:
- **Spore-Alpha** → Give to your partner (deploys on their laptop)
- **Spore-Beta** → Give to your sibling (deploys on their desktop)
- **Spore-Gamma** → Deploy at school computer lab
- **Spore-Delta** → Deploy at friend's gaming rig
- **Spore-Epsilon** → You keep for your own systems
- **Spore-Archive** → Cold storage backup

**Result**:
- All nodes can federate (shared genetic lineage)
- Each node has unique local identity
- BirdSong uses genetic lineage for NAT traversal
- Users can create sub-federations (gaming, family-only, etc.)

### 2. Gaming Federation

**Scenario**: You want to set up a gaming network with friends

```bash
# All nodes can federate (family trust)
# But you create a "gaming" sub-federation

# On your node
biomeos federation create-subfed \
    --name "gaming" \
    --parent-family "nat0" \
    --members "node-alpha-C1,node-delta-C1,node-epsilon-C1"

# Gaming-specific primals can now coordinate
# - Voice chat (via BearDog encrypted channels)
# - Game server discovery (via Songbird)
# - Lobby management (via Toadstool)
```

**Trust Model**:
- **Family Trust**: All nodes can discover each other
- **Gaming Sub-Fed**: Only specific nodes share gaming resources
- **Granular Privileges**: Different access levels per sub-federation

### 3. School Computing

**Scenario**: Deploy spores to school computer lab

```bash
# Create a school-specific sub-federation
biomeos federation create-subfed \
    --name "school" \
    --parent-family "nat0" \
    --isolation "high" \
    --capabilities "compute-only"

# Deploy Spore-Gamma at school
# - Students can access compute resources
# - Limited to school-specific primals
# - Cannot access family-only resources
# - But genetic lineage allows you to manage remotely
```

**Benefits**:
- You control the spore (parent seed)
- School systems isolated from personal data
- Can still coordinate (family trust)
- Granular capabilities (compute-only, no storage access)

### 4. Family Data Federation

**Scenario**: Share photos/docs with family members only

```bash
# Create family-data sub-federation
biomeos federation create-subfed \
    --name "family-data" \
    --parent-family "nat0" \
    --members "node-alpha-C*,node-beta-C*,node-epsilon-C*" \
    --capabilities "storage,sync"

# Deploy Loamspine (storage primal) only on family sub-fed
# - Encrypted with BearDog
# - Only accessible by family members
# - Gaming friends can't access
# - School systems can't access
```

---

## 🧬 Genetic Lineage + Local Entropy

### Seed Derivation Formula

```
Step 1: Spore Creation (from parent)
    spore_seed = SHA256(parent_seed || spore_id || deployment_batch)

Step 2: Local Deployment (on computer)
    local_entropy = SHA256(
        hostname || 
        machine_id || 
        timestamp || 
        network_interface_mac || 
        random_nonce
    )
    
    deployed_node_seed = SHA256(spore_seed || local_entropy)

Step 3: Node Identity
    node_id = "spore-{id}-{hostname}"
    
    Example:
    - Spore-Alpha on Computer-1 → "node-alpha-computer1"
    - Spore-Alpha on Computer-2 → "node-alpha-computer2"
```

### Properties

**Family Trust** (Genetic Lineage):
- All nodes from same parent can federate
- BirdSong uses genetic lineage for NAT
- Zero-config mutual discovery

**Local Identity** (Entropy):
- Each deployment is unique
- Multiple deployments from same spore = siblings
- Prevents collision even if same spore used twice

**Hierarchical Federation**:
- Family trust = baseline (everyone can discover)
- Sub-federations = granular (specific capabilities)
- User-defined access control

---

## 🔧 Implementation Design

### 1. Spore Incubation Module

**Location**: `crates/biomeos-spore/src/incubation.rs`

```rust
pub struct SporeIncubator {
    spore_seed: FamilySeed,
    local_entropy: LocalEntropy,
}

impl SporeIncubator {
    /// Incubate spore on local computer
    pub async fn incubate(
        spore_path: impl AsRef<Path>,
        computer_name: &str,
    ) -> SporeResult<IncubatedNode> {
        // 1. Read spore seed
        let spore_seed = FamilySeed::from_file(
            spore_path.as_ref().join(".family.seed")
        )?;
        
        // 2. Generate local entropy
        let local_entropy = LocalEntropy::generate(computer_name)?;
        
        // 3. Derive deployed node seed
        let deployed_seed = Self::derive_deployed_seed(
            &spore_seed,
            &local_entropy,
        )?;
        
        // 4. Create local config
        let local_config_path = Self::get_local_config_path(computer_name)?;
        Self::create_local_config(
            &local_config_path,
            &deployed_seed,
            spore_path.as_ref(),
        ).await?;
        
        // 5. Log deployment to spore
        let log_tracker = SporeLogTracker::new(spore_path)?;
        log_tracker.record_incubation(
            computer_name,
            &local_entropy.hash(),
        ).await?;
        
        Ok(IncubatedNode {
            node_id: format!("spore-{}-{}", spore_seed.id(), computer_name),
            deployed_seed,
            local_config_path,
        })
    }
}

pub struct LocalEntropy {
    hostname: String,
    machine_id: String,
    timestamp: DateTime<Utc>,
    mac_address: String,
    random_nonce: [u8; 32],
}

impl LocalEntropy {
    pub fn generate(computer_name: &str) -> SporeResult<Self> {
        use sha2::{Digest, Sha256};
        
        // Gather local system info
        let hostname = std::env::var("HOSTNAME")
            .unwrap_or_else(|_| computer_name.to_string());
        
        let machine_id = std::fs::read_to_string("/etc/machine-id")
            .unwrap_or_else(|_| uuid::Uuid::new_v4().to_string());
        
        let timestamp = Utc::now();
        
        let mac_address = get_primary_mac_address()
            .unwrap_or_else(|_| "00:00:00:00:00:00".to_string());
        
        let mut random_nonce = [0u8; 32];
        getrandom::getrandom(&mut random_nonce)?;
        
        Ok(Self {
            hostname,
            machine_id,
            timestamp,
            mac_address,
            random_nonce,
        })
    }
    
    pub fn hash(&self) -> String {
        use sha2::{Digest, Sha256};
        
        let mut hasher = Sha256::new();
        hasher.update(self.hostname.as_bytes());
        hasher.update(self.machine_id.as_bytes());
        hasher.update(self.timestamp.to_rfc3339().as_bytes());
        hasher.update(self.mac_address.as_bytes());
        hasher.update(&self.random_nonce);
        
        format!("{:x}", hasher.finalize())
    }
}
```

### 2. Local Configuration Storage

**Location**: `~/.config/biomeos/deployed-nodes/{spore-id}/`

```
~/.config/biomeos/
└── deployed-nodes/
    ├── spore-alpha/
    │   ├── node.toml           # Node configuration
    │   ├── .deployed.seed      # Deployed node seed
    │   ├── deployment.log      # Deployment history
    │   └── tower.toml          # Local tower config
    │
    └── spore-beta/
        ├── node.toml
        ├── .deployed.seed
        ├── deployment.log
        └── tower.toml
```

**node.toml**:
```toml
[node]
spore_id = "alpha"
node_id = "node-alpha-laptop"
deployed_at = "2026-01-08T20:00:00Z"
computer_name = "laptop"
entropy_hash = "abc123..."

[lineage]
parent_seed_hash = "parent123..."
spore_seed_hash = "spore456..."
deployed_seed_hash = "deployed789..."

[spore]
original_path = "/media/usb-alpha/biomeOS"
last_seen = "2026-01-08T20:00:00Z"
deployment_count = 1

[federation]
family_id = "nat0"
sub_federations = ["gaming", "family-data"]
```

### 3. Sub-Federation System

**Location**: `crates/biomeos-federation/src/subfederation.rs`

```rust
pub struct SubFederation {
    name: String,
    parent_family: String,
    members: Vec<NodeId>,
    capabilities: Vec<Capability>,
    isolation_level: IsolationLevel,
}

pub enum IsolationLevel {
    None,       // Full federation
    Low,        // Limited capabilities
    Medium,     // Specific primals only
    High,       // Compute-only, no data access
    Critical,   // Air-gapped, manual approval
}

pub enum Capability {
    Storage,
    Compute,
    Gaming,
    Sync,
    Voice,
    Video,
    Discovery,
    Custom(String),
}

impl SubFederation {
    /// Create a new sub-federation
    pub fn create(
        name: &str,
        parent_family: &str,
        members: Vec<NodeId>,
        capabilities: Vec<Capability>,
        isolation_level: IsolationLevel,
    ) -> FederationResult<Self> {
        // Verify all members share same genetic lineage
        Self::verify_genetic_lineage(&members)?;
        
        Ok(Self {
            name: name.to_string(),
            parent_family: parent_family.to_string(),
            members,
            capabilities,
            isolation_level,
        })
    }
    
    /// Check if node has access to capability
    pub fn has_capability(
        &self,
        node_id: &NodeId,
        capability: &Capability,
    ) -> bool {
        // 1. Check if node is member
        if !self.members.contains(node_id) {
            return false;
        }
        
        // 2. Check if capability is allowed
        if !self.capabilities.contains(capability) {
            return false;
        }
        
        // 3. Check isolation level
        match self.isolation_level {
            IsolationLevel::Critical => {
                // Require manual approval
                false
            }
            _ => true,
        }
    }
}
```

### 4. CLI Commands

```bash
# Incubate spore on local computer
biomeos spore incubate \
    --spore /media/usb-alpha/biomeOS \
    --computer-name laptop \
    --deploy-local

# List local deployments
biomeos node list-local

# Create sub-federation
biomeos federation create-subfed \
    --name gaming \
    --parent-family nat0 \
    --members "node-alpha-*,node-delta-*" \
    --capabilities "gaming,voice,discovery"

# Join sub-federation
biomeos federation join-subfed \
    --name gaming \
    --node node-epsilon-desktop

# List sub-federations
biomeos federation list-subfeds

# Check node access
biomeos federation check-access \
    --node node-alpha-laptop \
    --capability storage \
    --subfed family-data
```

---

## 🎯 Benefits

### For Users

**Flexibility**:
- One spore can deploy on multiple computers
- Each deployment is unique but related
- Portable: eject spore, use on another computer

**Control**:
- Parent seed gives ultimate control
- Sub-federations for granular access
- Hierarchical trust model

**Privacy**:
- Isolate work from personal
- Isolate gaming from family data
- School deployments sandboxed

### For Federation

**Genetic Lineage NAT** (BirdSong):
- All nodes from same parent can discover
- NAT traversal using family trust
- Zero-config mesh networking

**Scalability**:
- Distribute spores to 10s, 100s, 1000s of people
- Automatic federation
- Hierarchical organization

**Security**:
- BearDog encryption for all sub-feds
- Parent seed can revoke spores
- Granular capabilities

---

## 🌟 Example Deployment Scenario

### Setup Phase

```bash
# 1. Create parent seed (secure this!)
biomeos spore create \
    --mount /media/usb-parent \
    --label ParentSeed \
    --node genesis

# 2. Create 10 LiveSpores for distribution
for i in {1..10}; do
    biomeos spore clone \
        --from /media/usb-parent \
        --to /media/usb-spore-$i \
        --node spore-$i \
        --deployment-batch "family-distribution-2026"
done

# 3. Create 2 ColdSpores for backup
for i in {1..2}; do
    biomeos spore clone \
        --from /media/usb-parent \
        --to /media/usb-cold-$i \
        --node cold-$i \
        --spore-type cold
done
```

### Distribution Phase

```bash
# Give spores to:
# - Partner (Spore-1)
# - Sibling (Spore-2)
# - Friend-1 (Spore-3)
# - Friend-2 (Spore-4)
# - School (Spore-5)
# - Gaming buddy (Spore-6)
# - You keep (Spore-7, 8, 9, 10)
```

### Incubation Phase

**At Partner's House**:
```bash
# Partner plugs in Spore-1
biomeos spore incubate \
    --spore /media/usb-spore-1/biomeOS \
    --computer-name partners-laptop

# Result: node-spore1-partners-laptop
# - Unique identity
# - Family trust with all other nodes
# - Can join sub-federations
```

**At Your House**:
```bash
# You plug in Spore-7 on your desktop
biomeos spore incubate \
    --spore /media/usb-spore-7/biomeOS \
    --computer-name my-desktop

# You plug in Spore-7 on your laptop
biomeos spore incubate \
    --spore /media/usb-spore-7/biomeOS \
    --computer-name my-laptop

# Result: 
# - node-spore7-my-desktop
# - node-spore7-my-laptop
# Both from same spore, both unique, both siblings
```

### Sub-Federation Phase

**Create Gaming Sub-Fed**:
```bash
biomeos federation create-subfed \
    --name gaming \
    --members "node-spore6-*,node-spore7-*" \
    --capabilities "gaming,voice"
```

**Create Family Sub-Fed**:
```bash
biomeos federation create-subfed \
    --name family \
    --members "node-spore1-*,node-spore2-*,node-spore7-*" \
    --capabilities "storage,sync,photos"
```

**Create School Sub-Fed**:
```bash
biomeos federation create-subfed \
    --name school \
    --members "node-spore5-*" \
    --capabilities "compute-only" \
    --isolation high
```

### Result

**Federation Topology**:
```
Family Trust (Genetic Lineage: nat0)
│
├── Gaming Sub-Federation
│   ├── node-spore6-gaming-rig (friend's computer)
│   └── node-spore7-my-desktop (your computer)
│
├── Family Sub-Federation
│   ├── node-spore1-partners-laptop
│   ├── node-spore2-siblings-desktop
│   └── node-spore7-my-laptop
│
└── School Sub-Federation
    └── node-spore5-lab-computer-01
    └── node-spore5-lab-computer-02
    └── node-spore5-lab-computer-03
```

**Access Matrix**:
| Node | Gaming Voice | Family Photos | School Compute |
|------|--------------|---------------|----------------|
| spore6-gaming-rig | ✅ | ❌ | ❌ |
| spore7-my-desktop | ✅ | ❌ | ❌ |
| spore1-partners-laptop | ❌ | ✅ | ❌ |
| spore2-siblings-desktop | ❌ | ✅ | ❌ |
| spore7-my-laptop | ❌ | ✅ | ❌ |
| spore5-lab-computer-* | ❌ | ❌ | ✅ |

---

## 🚀 Future: Niche Blueprints

This pattern becomes the foundation for complex niche deployments:

### Blueprint 1: Community Mesh Network
- Distribute spores to neighbors
- Everyone gets internet failover
- Gaming + file sharing
- Encrypted with BearDog

### Blueprint 2: Educational Cluster
- Schools deploy student nodes
- Teachers have admin privileges
- Students isolated but can collaborate
- Remote management via parent seed

### Blueprint 3: Business Federation
- Each department gets sub-federation
- HR has different access than Engineering
- All share company resources
- Granular audit trails

### Blueprint 4: IoT Deployment
- Spores deployed on edge devices
- Local entropy = device-specific
- Sub-federations = device types
- Parent seed = central management

---

## 📋 Next Steps

### Phase 1: Core Implementation
1. ✅ Design complete (this document)
2. [ ] Implement `incubation.rs` module
3. [ ] Add local config storage
4. [ ] Update `SporeLogTracker` for incubation events
5. [ ] CLI commands for incubation

### Phase 2: Sub-Federation
1. [ ] Implement `subfederation.rs` module
2. [ ] Capability system
3. [ ] Isolation levels
4. [ ] CLI commands for sub-federation management

### Phase 3: Integration
1. [ ] BirdSong integration (genetic lineage NAT)
2. [ ] BearDog integration (sub-fed encryption)
3. [ ] Songbird integration (sub-fed discovery)
4. [ ] Tower integration (privilege enforcement)

### Phase 4: Testing & Validation
1. [ ] Unit tests (incubation, sub-federation)
2. [ ] E2E tests (multi-spore, multi-computer)
3. [ ] LAN tests (gaming federation)
4. [ ] Real-world validation (family deployment)

---

## 🌟 Conclusion

**This architecture enables:**
- **Portable Identity**: Same spore, different computers
- **Hierarchical Trust**: Family baseline + granular sub-federations
- **Flexible Deployment**: Gaming, school, family, business
- **Genetic Lineage NAT**: BirdSong coordination
- **Future-Proof**: Blueprint for complex niches

**🌱 From single USB spore → Global distributed network**

**🎊 Ready to revolutionize how we deploy and federate systems!**

