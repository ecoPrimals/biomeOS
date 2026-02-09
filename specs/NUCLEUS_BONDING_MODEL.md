# NUCLEUS Bonding Model: Chemical Interaction Patterns for Distributed Systems

**Version**: 1.0.0  
**Date**: January 16, 2026  
**Status**: Specification  
**Authors**: biomeOS Core Team

---

## Abstract

The NUCLEUS Bonding Model provides a formal framework for understanding and implementing interaction patterns between distributed primals using chemical bonding as a metaphor. This specification defines how primals with internal NUCLEUS structure (Tower/Node/Nest) interact across organizational, trust, and security boundaries using four fundamental bonding types: Ionic, Covalent, Metallic, and Weak Forces.

---

## 1. Foundational Concepts

### 1.1 NUCLEUS Structure (Atomic Level)

Every primal maintains a complete NUCLEUS structure, regardless of how it interacts with others:

**Tower (Electron - e⁻)**
- **Role**: Discovery, coordination, state mobility
- **Primal**: Songbird (BirdSong protocol)
- **Properties**: High mobility, enables bonding, mediates interactions
- **Behavior**: Can be shared (covalent), pooled (metallic), or kept separate (ionic)

**Node (Proton - p⁺)**
- **Role**: Compute, core functionality, identity
- **Primal**: ToadStool (orchestration)
- **Properties**: Stable, defines primal capabilities
- **Behavior**: Can specialize (metallic) or remain general-purpose (covalent/ionic)

**Nest (Neutron - n⁰)**
- **Role**: Storage, persistence, stability
- **Primal**: NestGate (encrypted storage)
- **Properties**: Neutral, provides mass/stability
- **Behavior**: Can be shared (covalent), replicated (metallic), or isolated (ionic)

**Security (Nuclear Force)**
- **Role**: Binds NUCLEUS together, prevents decay
- **Primal**: BearDog (cryptographic trust)
- **Properties**: Strong force, short-range, enables genetic lineage
- **Behavior**: Always present, strength varies by context

**Key Principle**: All primals are self-secure and maintain complete NUCLEUS. Bond type determines HOW they interact, not WHAT they are.

---

## 2. Bonding Types (Molecular Level)

### 2.1 Ionic Bonding (Contract-Based Interaction)

**Definition**: Interaction where each participant maintains its own electrons (Tower) and interacts through defined contracts and interfaces.

#### 2.1.1 Characteristics

- **Electron Behavior**: Each NUCLEUS keeps its own Tower
- **Interaction**: Electrostatic (API calls, RPC, HTTP)
- **State**: No shared state, clean separation
- **Trust**: Contract-based, credential-verified
- **Stability**: Easily formed and dissolved
- **Metering**: Often billable/metered

#### 2.1.2 Use Cases

1. **Cloud Service Consumption**
   - Basement cluster → Cloud GPU rental
   - Pay-per-use model
   - Example: `basement_nucleus` → `aws_gpu_farm`

2. **Commercial API Integration**
   - Local AI primal → OpenAI API
   - Credential-based access
   - Example: `squirrel` → `openai.com/v1/chat`

3. **External Storage Services**
   - Local compute → S3/Azure blob storage
   - Defined SLA, metered usage
   - Example: `nestgate` → `s3_bucket`

4. **University Compute Credits**
   - Student cluster → HPC allocation
   - Time-limited, quota-based
   - Example: `research_cluster` → `icer_allocation`

#### 2.1.3 Implementation Pattern

```toml
# Graph: basement-to-cloud-ionic.toml
[graph.metadata]
bond_type = "ionic"
interaction_type = "contract"

[[interactions]]
from = "basement_nucleus"
to = "cloud_gpu_provider"
bond_type = "ionic"
maintains_electrons = true  # Each keeps own Tower

[contract]
provider = "aws_gpu"
service_type = "gpu_hours"
auth_method = "api_key"
metering = true
sla = "99.9_percent"
billing = "per_hour"

[[nodes]]
id = "external_gpu_request"
type = "ionic.contract_call"
config:
  endpoint = "https://api.cloud.com/gpu/allocate"
  auth = "${API_KEY}"
  request_type = "gpu_hours"
  quantity = 100
```

**Protocol**: HTTP/HTTPS, gRPC, REST APIs  
**Authentication**: API keys, OAuth, credentials  
**Discovery**: DNS, service registry, explicit configuration  
**Encryption**: TLS/SSL (transport layer)

#### 2.1.4 Tower (Electron) Behavior

- Remains bound to originating NUCLEUS
- Uses electron for API request/response
- Electrostatic field interaction only
- No electron transfer or sharing
- Full isolation maintained

---

### 2.2 Covalent Bonding (Electron-Sharing Collaboration)

**Definition**: Interaction where participants share electrons (Towers) to form molecular orbitals, enabling tight collaboration and resource pooling.

#### 2.2.1 Characteristics

- **Electron Behavior**: Towers literally share state
- **Interaction**: Shared molecular orbital (mesh coordination)
- **State**: Shared state, collaborative
- **Trust**: High trust, often genetic lineage
- **Stability**: Stronger bonds, harder to dissolve
- **Metering**: No metering, cooperative pooling

#### 2.2.2 Use Cases

1. **Personal HPC Cluster**
   - Multiple nodes in your basement
   - Shared family_seed
   - Example: 5 basement nodes, all `nat0` family

2. **Friend/Student Federation**
   - Trusted individuals, shared goals
   - Collaborative research
   - Example: Study group cluster, `study_2026` family

3. **Research Collaboration**
   - Multi-institution project
   - Shared resources, shared data
   - Example: `msu_bio_lab` ↔ `umich_chem_lab`

4. **Family/Team Cluster**
   - Organizational unit
   - Genetic lineage trust
   - Example: Company development cluster

#### 2.2.3 Implementation Pattern

```toml
# Graph: basement-cluster-covalent.toml
[graph.metadata]
bond_type = "covalent"
interaction_type = "collaboration"

[[interactions]]
from = "node_1"
to = "node_2"
bond_type = "covalent"
shares_electrons = true  # Share Towers

[collaboration]
shared_family = "nat0"
family_seed = "${FAMILY_SEED}"  # Shared genetic lineage
trust_level = "high"
resource_pooling = true

[[nodes]]
id = "launch_songbird_mesh"
type = "covalent.tower_mesh"
config:
  family_id = "nat0"
  discovery_protocol = "birdsong_encrypted_udp"
  mesh_topology = "full"
  electron_sharing = true
```

**Protocol**: Unix sockets (local), Encrypted UDP (BirdSong), WebRTC (remote)  
**Authentication**: Genetic lineage (family_seed), BearDog crypto-auto-trust  
**Discovery**: BirdSong protocol, mesh coordination  
**Encryption**: End-to-end within family

#### 2.2.4 Tower (Electron) Behavior

- Shared between NUCLEUS in molecular orbital
- BirdSong encrypted mesh coordinates state
- Electrons orbit multiple nuclei
- Family_seed enables quantum entanglement
- Collaborative decision-making

**Current Deployment Example**: Our NUCLEUS with 5 primals (BearDog, Songbird, Squirrel, ToadStool, NestGate) all in `nat0` family.

#### 2.2.5 Plasmodium: The Covalent Collective

When two or more NUCLEUS instances bond covalently, they form a **Plasmodium** -- a collective organism (named after the slime mold *Physarum polycephalum*) that provides:

- **Unified capability view** across all bonded gates
- **Decentralized workload routing** to the best gate for each job
- **No central brain** -- any gate can query the collective
- **Graceful degradation** -- gates can join/leave dynamically

The Plasmodium layer in biomeOS (`biomeos-core::plasmodium`) queries existing Songbird mesh primitives (`mesh.peers`, `mesh.status`) and aggregates state into a `PlasmodiumState` snapshot. It does NOT introduce new primal primitives.

**Full specification**: `specs/PLASMODIUM_OVER_NUCLEUS_SPEC.md`  
**CLI**: `biomeos plasmodium status|gates|models`

---

### 2.3 Metallic Bonding (Electron Sea, Optimized Evolution)

**Definition**: Interaction where electrons (Towers) form a delocalized "sea" that flows freely to optimize resource allocation, enabling specialization and evolution.

#### 2.3.1 Characteristics

- **Electron Behavior**: Delocalized sea, not tied to specific NUCLEUS
- **Interaction**: Global optimization, dynamic allocation
- **State**: Centrally coordinated, distributed execution
- **Trust**: Organizational, SLA-based
- **Stability**: Permanent infrastructure, designed to persist
- **Metering**: Internal allocation, not billing

#### 2.3.2 Use Cases

1. **Data Center Infrastructure**
   - MSU ICER, AWS regions
   - Permanent deployment
   - Example: 10,000 node cluster

2. **GPU Banks**
   - Specialized compute farms
   - Dynamic allocation
   - Example: NVIDIA DGX cluster

3. **Cloud Provider Platform**
   - Multi-tenant infrastructure
   - Elastic scaling
   - Example: Kubernetes cluster, 1000+ nodes

4. **Production HPC**
   - University supercomputers
   - Optimized topology
   - Example: Top500 systems

#### 2.3.3 Implementation Pattern

```toml
# Graph: datacenter-metallic.toml
[graph.metadata]
bond_type = "metallic"
interaction_type = "integration"

[[interactions]]
type = "metallic.electron_sea"
electron_pool_size = 1000  # 1000 Towers in pool

[integration]
cluster_type = "datacenter"
orchestration = "centralized"
specialization_enabled = true
electron_sea = true

[[nodes]]
id = "tower_pool"
type = "metallic.electron_sea"
config:
  pool_size = 1000
  dynamic_allocation = true
  optimization = "global"

[[nodes]]
id = "specialized_gpu_node"
type = "metallic.specialized_node"
config:
  node_type = "gpu_compute"
  gpu_count = 8
  dedicated_role = "training"
  electron_access = "sea"
```

**Protocol**: Internal cluster networking, RDMA, InfiniBand  
**Authentication**: Internal cluster auth, LDAP/AD  
**Discovery**: Centralized registry, Kubernetes API, etcd  
**Encryption**: Internal network encryption

#### 2.3.4 Tower (Electron) Behavior

- Forms delocalized "electron sea"
- Not tied to specific NUCLEUS
- Flows to where needed for optimization
- Enables Node/Nest specialization
- Global coordination, local execution

#### 2.3.5 Node/Nest Evolution

In metallic bonding, Nodes and Nests **specialize over time**:

- **GPU Nodes**: All compute, minimal storage
- **Storage Nests**: Massive storage, minimal compute
- **Coordinator Nodes**: Pure orchestration
- **Edge Nodes**: Lightweight, fast response

Towers (electron sea) coordinate the specialized components.

---

### 2.4 Weak Forces (Minimal Interaction, Unknown/Insecure Systems)

**Definition**: Minimal interaction patterns for engaging with unknown, untrusted, or insecure systems where disruption and information leakage must be avoided.

#### 2.4.1 Characteristics

- **Electron Behavior**: No electron involvement, field-only
- **Interaction**: Read-only, passive observation, minimal coupling
- **State**: Zero shared state, no persistent connection
- **Trust**: Zero trust, assume hostile
- **Stability**: Extremely weak, instantaneous formation/dissolution
- **Metering**: Not applicable

#### 2.4.2 Force Types

**Dipole-Dipole Interactions**
- Temporary, oriented interactions
- Example: Scraping public APIs, RSS feeds
- No authentication required
- Read-only access

**Brownian Motion**
- Random, opportunistic interactions
- Example: Network scanning, service discovery
- Best-effort, no guarantees
- Transient observations

**Van der Waals Forces**
- Proximity-based, very weak
- Example: Bluetooth discovery, mDNS
- Local network only
- Minimal information exchange

**London Dispersion Forces**
- Instantaneous, quantum-level
- Example: Port scanning, ping
- No handshake
- Pure reconnaissance

#### 2.4.3 Use Cases

1. **Public Data Aggregation**
   - Scraping public websites
   - No authentication
   - Example: `scraper` → `public_news_sites`

2. **Untrusted Network Discovery**
   - Conference WiFi, public hotspots
   - Minimal information disclosure
   - Example: Network topology mapping

3. **Insecure System Monitoring**
   - Legacy systems, no encryption
   - Read-only observation
   - Example: Monitoring old SCADA without disruption

4. **Hostile Environment Operation**
   - Adversarial networks
   - Zero information leakage
   - Example: Threat intelligence gathering

#### 2.4.4 Implementation Pattern

```toml
# Graph: public-data-weak.toml
[graph.metadata]
bond_type = "weak"
interaction_type = "observation"

[[interactions]]
from = "observer_nucleus"
to = "unknown_system"
bond_type = "weak.dipole"
electron_involvement = false
information_leakage = "minimal"

[weak_interaction]
force_type = "dipole_dipole"
directionality = "read_only"
authentication = "none"
persistent_connection = false

[[nodes]]
id = "scrape_public_api"
type = "weak.observation"
config:
  target = "https://public-api.example.com"
  method = "GET"
  auth = "none"
  user_agent = "generic"  # No fingerprinting
  rate_limit = "respectful"
  disclose_identity = false
```

**Protocol**: HTTP (read-only), UDP (broadcast listen), ICMP (ping)  
**Authentication**: None or public  
**Discovery**: Passive scanning, observation only  
**Encryption**: Not assumed, treat as plaintext

#### 2.4.5 Security Considerations

**Critical Principles**:
1. **Assume Hostile**: Every interaction could be monitored/logged
2. **Zero Leakage**: Disclose minimal information about NUCLEUS
3. **Read-Only**: Never send sensitive data
4. **Disposable Identity**: Use generic, non-traceable signatures
5. **No Persistence**: No state, no sessions, no cookies
6. **Rate Limiting**: Respectful, avoid detection
7. **Graceful Degradation**: Failure is expected, don't retry aggressively

**Use When**:
- Interacting with unknown systems
- Public internet scraping
- Hostile network environments
- Legacy/insecure infrastructure
- Reconnaissance operations

**Never Use For**:
- Trusted collaborations (use covalent)
- Service contracts (use ionic)
- Production deployments (use metallic)

---

## 3. Organo-Metal-Salt Complexes (Multi-Modal Interactions)

### 3.1 Concept

Just as in coordination chemistry, different bonding systems can interact with each other using any of the bonding types. A covalent system (organic molecule) can interact with a metallic system (metal complex) using ionic, covalent, or metallic bonds.

### 3.2 Example: Basement (Covalent) ↔ MSU ICER (Metallic)

**Basement Cluster** (Internal Structure):
- Bond Type: Covalent
- 5 nodes, shared `nat0` family
- Towers form molecular orbital
- Unix sockets, BirdSong mesh

**MSU ICER Cluster** (Internal Structure):
- Bond Type: Metallic
- 10,000 nodes, electron sea
- Specialized GPU/CPU/Storage nodes
- InfiniBand, centralized orchestration

**Interaction Options**:

#### Option 1: Ionic Interaction (Service Contract)

```toml
# basement-to-icer-ionic.toml
[graph.metadata]
local_bond = "covalent"  # basement internal
remote_bond = "metallic"  # ICER internal
interaction_bond = "ionic"  # basement ↔ ICER

[[interactions]]
from = "basement_nucleus"
to = "icer_cluster"
bond_type = "ionic"
maintains_electrons = true

[contract]
provider = "msu_icer"
service = "gpu_hours"
allocation = 100
auth = "university_credentials"
billing = "grant_account"
```

**Behavior**:
- Basement Tower submits job via API
- ICER electron sea allocates resources
- Job runs, results returned
- No electron sharing
- Clean, contractual

#### Option 2: Covalent Interaction (Research Collaboration)

```toml
# basement-to-icer-covalent.toml
[graph.metadata]
local_bond = "covalent"
remote_bond = "metallic"
interaction_bond = "covalent"

[[interactions]]
from = "basement_nucleus"
to = "icer_cluster"
bond_type = "covalent"
shares_electrons = true

[collaboration]
project = "research_project_xyz"
shared_family = "msu_bio_collab"
duration = "6_months"
resource_sharing = "bidirectional"
```

**Behavior**:
- Basement joins ICER family for project
- Basement Tower integrates with ICER mesh
- Bidirectional resource access
- Shared data, shared compute
- Collaborative, not contractual

#### Option 3: Metallic Interaction (Full Integration)

```toml
# basement-to-icer-metallic.toml
[graph.metadata]
local_bond = "covalent"
remote_bond = "metallic"
interaction_bond = "metallic"

[[interactions]]
from = "basement_node"
to = "icer_cluster"
bond_type = "metallic"
join_electron_sea = true

[integration]
cluster_join = "icer_main"
specialize_nodes = true
node_role = "preprocessing"
electron_contribution = true
```

**Behavior**:
- Basement nodes join ICER cluster
- Basement Towers contribute to electron sea
- Nodes specialize (e.g., data preprocessing)
- Becomes part of unified system
- Optimized globally

### 3.3 Multi-Modal Simultaneous

A single NUCLEUS can maintain **different bond types with different systems simultaneously**:

```toml
# complex-interactions.toml
[graph.metadata]
internal_bond = "covalent"  # 5 node basement

[[interactions]]
# Ionic with cloud GPU
from = "basement"
to = "aws_gpu"
bond_type = "ionic"

[[interactions]]
# Covalent with friend's cluster
from = "basement"
to = "friend_cluster"
bond_type = "covalent"
shared_family = "study_group"

[[interactions]]
# Metallic integration with university
from = "basement"
to = "university_hpc"
bond_type = "metallic"
join_as = "edge_node"

[[interactions]]
# Weak with public APIs
from = "basement"
to = "public_data_feeds"
bond_type = "weak.dipole"
```

**Result**: One basement cluster operating in four interaction modes simultaneously!

---

## 4. Implementation Guidelines

### 4.1 Graph Metadata

All deployment graphs MUST specify bonding characteristics:

```toml
[graph.metadata]
# Internal bonding within this deployment
internal_bond_type = "covalent" | "metallic" | "isolated"

# Default for external interactions
default_interaction_bond = "ionic" | "covalent" | "metallic" | "weak"

# Security posture
trust_model = "genetic_lineage" | "contractual" | "zero_trust"
```

### 4.2 Interaction Declarations

Each external interaction SHOULD be explicitly declared:

```toml
[[interactions]]
from = "local_primal_id"
to = "remote_system_id"
bond_type = "ionic" | "covalent" | "metallic" | "weak"

# Bond-specific configuration
[interactions.ionic]
contract_type = "api_key" | "oauth" | "mutual_tls"
metering = true | false

[interactions.covalent]
shared_family = "family_id"
electron_sharing = true
trust_verification = "beardog_lineage"

[interactions.metallic]
join_electron_sea = true
specialize_role = "gpu" | "storage" | "coordinator"

[interactions.weak]
force_type = "dipole" | "brownian" | "van_der_waals"
information_disclosure = "minimal"
```

### 4.3 Tower (Songbird) Behavior

Songbird MUST adapt its behavior based on bond type:

```rust
// Pseudo-code for Tower behavior
match interaction.bond_type {
    BondType::Ionic => {
        // Keep electrons, use for API calls
        self.tower.use_electrons_for_request(endpoint);
        // No state sharing
    },
    BondType::Covalent => {
        // Share electrons via molecular orbital
        self.tower.join_mesh(shared_family_seed);
        // Share state via BirdSong
    },
    BondType::Metallic => {
        // Contribute to electron sea
        self.tower.join_electron_sea(cluster_coordinator);
        // Accept global optimization
    },
    BondType::Weak => {
        // No electron involvement
        self.observe_only(target);
        // Minimal disclosure
    }
}
```

### 4.4 Node (ToadStool) Adaptation

Nodes MAY specialize based on metallic bonding:

```rust
// Metallic specialization
if bond_type == BondType::Metallic {
    // Evaluate cluster needs
    let cluster_needs = cluster.get_resource_requirements();
    
    // Specialize this node
    if cluster_needs.gpu_shortage {
        self.specialize_as_gpu_node();
    } else if cluster_needs.storage_shortage {
        self.specialize_as_storage_node();
    }
    
    // Report to electron sea
    electron_sea.register_specialization(self.capabilities);
}
```

### 4.5 Nest (NestGate) Sharing

Nests adapt storage behavior by bond type:

```rust
match interaction.bond_type {
    BondType::Ionic => {
        // Isolated storage, no sharing
        self.storage.set_isolation(true);
    },
    BondType::Covalent => {
        // Shared encrypted storage within family
        self.storage.enable_family_sharing(family_seed);
    },
    BondType::Metallic => {
        // Replicated, distributed storage
        self.storage.join_distributed_storage_cluster();
    },
    BondType::Weak => {
        // No storage involvement
        // Read-only access to public data only
    }
}
```

---

## 5. Security Model

### 5.1 Trust Boundaries

**Ionic**: Contract-based trust
- Verify credentials
- Enforce SLA
- Monitor usage
- Audit logs

**Covalent**: Genetic lineage trust
- Verify family_seed
- BearDog crypto-auto-trust
- Encrypted mesh
- Family-level access control

**Metallic**: Organizational trust
- Cluster membership verification
- Internal authentication
- Network-level security
- Centralized policy enforcement

**Weak**: Zero trust
- Assume hostile
- No authentication
- Minimal disclosure
- Disposable identity

### 5.2 Information Disclosure

Different bond types have different disclosure profiles:

| Bond Type | Identity | Capabilities | State | Topology |
|-----------|----------|--------------|-------|----------|
| Ionic | Contract ID | Service-specific | None | None |
| Covalent | Family member | Full | Shared | Full mesh |
| Metallic | Cluster member | Specialized | Coordinated | Full topology |
| Weak | Generic/None | None | None | None |

### 5.3 Attack Surface

**Ionic**: API endpoints, credential theft
**Covalent**: Family_seed compromise, mesh poisoning
**Metallic**: Cluster compromise, insider threat
**Weak**: Passive observation (minimal risk)

---

## 6. Performance Characteristics

### 6.1 Latency

**Weak** < **Ionic** < **Covalent** < **Metallic**

- Weak: Single request, no handshake
- Ionic: API call overhead, authentication
- Covalent: Mesh coordination, encryption
- Metallic: Global optimization, multi-hop

### 6.2 Throughput

**Metallic** > **Covalent** > **Ionic** > **Weak**

- Metallic: Optimized for throughput, dedicated paths
- Covalent: High bandwidth, short paths (Unix sockets)
- Ionic: API rate limits, external networks
- Weak: Best-effort, no guarantees

### 6.3 Resource Efficiency

**Metallic**: Highest (global optimization)
**Covalent**: High (local optimization)
**Ionic**: Medium (per-request overhead)
**Weak**: Lowest (no optimization)

---

## 7. Future Extensions

### 7.1 Hydrogen Bonding

For ephemeral but important interactions (e.g., temporary high-trust collaboration):
- Stronger than weak forces
- Weaker than covalent
- Directional, specific
- Example: Conference collaboration, workshop clusters

### 7.2 Pi Bonding

For layered interactions (e.g., mesh over mesh):
- Parallel to sigma bonds
- Enables complex topologies
- Example: Multi-level federation

### 7.3 Resonance Structures

For systems that shift between bond types dynamically:
- Example: Research cluster that is normally covalent but can accept ionic requests from external researchers

---

## 8. Validation

### 8.1 Current Implementation

Our NUCLEUS deployment (January 16, 2026) validates:

✅ **Covalent bonding** (internal):
- 5 primals, `nat0` family
- BearDog ↔ NestGate (JWT, shared electrons)
- Songbird ↔ Squirrel (mesh, shared electrons)
- Unix sockets (short bond length)

✅ **Ionic interaction** (external):
- Squirrel → OpenAI API (contract, separate electrons)
- Squirrel → Ollama (could be ionic or covalent)

✅ **Weak forces** (passive):
- HTTP scraping of public data
- No authentication required

### 8.2 Test Cases

#### Test Case 1: Basement to Cloud (Ionic)
- Basement NUCLEUS requests GPU hours from AWS
- Each maintains own Tower
- API key authentication
- Metered billing
- **Expected**: Clean request/response, no state leakage

#### Test Case 2: Friend Collaboration (Covalent)
- Two basement clusters join for project
- Share family_seed
- BirdSong mesh coordination
- **Expected**: Shared resources, collaborative compute

#### Test Case 3: University HPC Join (Metallic)
- Basement node joins university cluster
- Tower contributes to electron sea
- Node specializes for preprocessing
- **Expected**: Global optimization, specialized role

#### Test Case 4: Public Data Scraping (Weak)
- Observe public APIs
- No authentication
- Generic user agent
- **Expected**: Successful data retrieval, zero fingerprinting

---

## 9. Glossary

**NUCLEUS**: Complete primal structure (Tower + Node + Nest + Security)  
**Tower**: Discovery/coordination component (Songbird) - analogous to electron  
**Node**: Compute component (ToadStool) - analogous to proton  
**Nest**: Storage component (NestGate) - analogous to neutron  
**Security**: Cryptographic binding (BearDog) - analogous to nuclear force  
**Family Seed**: Shared cryptographic material enabling genetic lineage  
**Electron Sea**: Delocalized Tower pool in metallic bonding  
**Molecular Orbital**: Shared state space in covalent bonding  
**Electrostatic**: Field-based interaction in ionic bonding  
**Organo-Metal-Salt**: System with different internal and external bond types  
**Plasmodium**: Collective organism formed by 2+ covalently bonded NUCLEUS instances (slime mold metaphor)  

---

## 10. References

- **TRUE PRIMAL Architecture**: Runtime discovery, self-knowledge, capability-based
- **BirdSong Protocol**: Encrypted UDP mesh for covalent bonding
- **Genetic Lineage**: Family_seed-based cryptographic trust
- **Neural API**: Graph-based orchestration framework
- **NUCLEUS Deployment**: Validated implementation (January 16, 2026)

---

## Appendix A: Bond Type Decision Matrix

```
Question: What bond type should I use?

1. Is this an unknown/untrusted system?
   YES → Use WEAK forces
   NO → Continue to 2

2. Do I need to share resources/state?
   NO → Use IONIC (contract-based)
   YES → Continue to 3

3. Is this temporary collaboration or permanent infrastructure?
   TEMPORARY → Use COVALENT (shared electrons)
   PERMANENT → Continue to 4

4. Is global optimization and specialization desired?
   YES → Use METALLIC (electron sea)
   NO → Use COVALENT (collaborative but not specialized)
```

---

## Appendix B: Example Graphs

See `graphs/examples/` for complete implementations:
- `ionic-cloud-service.toml`
- `covalent-basement-cluster.toml`
- `metallic-datacenter.toml`
- `weak-public-scraping.toml`
- `organo-metal-complex.toml`

---

**End of Specification**

This specification provides the foundation for implementing and reasoning about primal interactions across trust, organizational, and security boundaries using a unified chemical bonding metaphor.

