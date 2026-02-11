# Beacon Genetics Build Specification
## Mitochondrial DNA Model - Address Book / Friend Graph

**Version**: 2.0.0  
**Date**: February 11, 2026  
**Status**: DESIGN SPECIFICATION  
**Author**: Kevin Mok + AI Collaborative Intelligence

---

## Executive Summary

The beacon genetics system is like **mitochondrial DNA**:
- Has its own genetic material (separate from nuclear/lineage DNA)
- Controls "cellular energy" → discovery/visibility
- Inherited loosely (maternal in biology, social in ecoPrimals)
- Can be **exchanged, mixed, shared** through meetings
- Independent from permissions (which come from lineage)

**Key Insight**: 
- **Beacon Seed** = "Who can decode my broadcasts?" = Social network
- **Lineage Seed** = "What can they DO once connected?" = Security

---

## 1. The Core Problem

### What BearDog Phase 1 Built (Foundation) ✅

```rust
// Encryption/decryption works
let beacon = BeaconSeed::generate();
let encrypted = beacon.encrypt(payload)?;
let decrypted = beacon.try_decrypt(&encrypted)?;
```

### What's Missing (The Friend Graph)

**Question**: When I receive an encrypted beacon broadcast, how do I decrypt it?

**Answer**: I need the **sender's beacon seed** - which I got when we **met**.

But currently, `.known_beacons.json` only has **metadata** (IDs, names, endpoints).
To decrypt, we need the actual **seeds**!

```
Current:
  known_beacons.json → BeaconId, metadata (can't decrypt!)
  
Needed:
  known_beacons.json → BeaconId, metadata
  beacon_seeds/      → Actual seeds (can decrypt!)
```

---

## 2. The Mitochondrial Model

### Biological Parallel

| Property | Mitochondrial DNA | Beacon Genetics |
|----------|-------------------|-----------------|
| **Inheritance** | Maternal only | Social (meetings) |
| **Independence** | Own genome, own replication | Own seed, own address book |
| **Mixing** | Limited (rare paternal leak) | Frequent (every meeting) |
| **Function** | Energy production | Discovery/visibility |
| **Relationship to nuclear** | Loose (coordinate but separate) | Loose (lineage hint for sync) |

### Key Properties

1. **Independent Genome**: Beacon seed is NOT derived from lineage seed
2. **Social Inheritance**: Gained through meetings, not family
3. **Address Book**: Collection of met beacon seeds
4. **Promiscuous Sharing**: Can share beacon with anyone I meet
5. **Loose Lineage Tie**: Devices with same lineage can sync address books

---

## 3. Data Model

### 3.1 Beacon Genetics Package

```rust
/// Complete beacon genetics for a node
struct BeaconGenetics {
    // ═══════════════════════════════════════════════════════════
    // MY BEACON - What I broadcast with
    // ═══════════════════════════════════════════════════════════
    
    /// My beacon seed - encrypts MY broadcasts
    /// Others need this to decode MY signals
    own_seed: BeaconSeed,
    
    /// My public beacon ID (derived from own_seed)
    own_id: BeaconId,
    
    // ═══════════════════════════════════════════════════════════
    // ADDRESS BOOK - Who I've met (their beacon seeds)
    // ═══════════════════════════════════════════════════════════
    
    /// Seeds from meetings - I can decrypt THEIR broadcasts
    /// Key: Their beacon ID
    /// Value: Their seed (so I can decrypt) + metadata
    met_seeds: HashMap<BeaconId, MeetingRecord>,
    
    /// Reverse lookup: Who have I shared MY seed with?
    /// (They can decrypt my broadcasts)
    shared_with: HashSet<BeaconId>,
    
    // ═══════════════════════════════════════════════════════════
    // CLUSTERS - Group beacon memberships
    // ═══════════════════════════════════════════════════════════
    
    /// Cluster beacons I'm a member of
    clusters: Vec<ClusterMembership>,
    
    // ═══════════════════════════════════════════════════════════
    // LINEAGE TIE - For sync across family devices
    // ═══════════════════════════════════════════════════════════
    
    /// Hint to parent lineage (first 8 bytes of family_id)
    /// Used only for: "Should we sync address books?"
    /// NOT used for: Permissions, identity, security
    lineage_hint: [u8; 8],
    
    /// Sync token for merge conflict resolution
    sync_token: Uuid,
    
    /// Last sync timestamp
    last_sync: Timestamp,
}
```

### 3.2 Meeting Record

```rust
/// Record of a meeting - contains their seed so we can decrypt
struct MeetingRecord {
    // ═══════════════════════════════════════════════════════════
    // CRYPTOGRAPHIC - Needed for decryption
    // ═══════════════════════════════════════════════════════════
    
    /// Their beacon ID (public, identifies them)
    beacon_id: BeaconId,
    
    /// Their beacon seed (SECRET - lets me decrypt their broadcasts!)
    /// This is the KEY difference from the current implementation
    beacon_seed: BeaconSeed,
    
    // ═══════════════════════════════════════════════════════════
    // METADATA - Human-readable, helps identify them
    // ═══════════════════════════════════════════════════════════
    
    /// Friendly name (human-readable)
    node_name: String,
    
    /// When we first met
    first_met: Timestamp,
    
    /// Last seen (updated on each broadcast)
    last_seen: Timestamp,
    
    /// Known endpoints (updated on each broadcast)
    endpoints: Vec<String>,
    
    /// Capabilities hint (e.g., ["compute", "storage"])
    capabilities_hint: Vec<String>,
    
    /// Human notes (e.g., "Met at coffee shop")
    notes: String,
    
    // ═══════════════════════════════════════════════════════════
    // RELATIONSHIP - How did we meet?
    // ═══════════════════════════════════════════════════════════
    
    /// How this meeting was established
    relationship: MeetingRelationship,
    
    /// Who introduced us (if Introduced)
    introduced_by: Option<BeaconId>,
    
    /// Whether this is one-way or mutual
    visibility: MeetingVisibility,
}

/// How the meeting was established
enum MeetingRelationship {
    /// Direct meeting (face-to-face or network)
    Direct,
    
    /// Introduced by a mutual contact
    Introduced { by: BeaconId },
    
    /// Met through cluster membership
    Cluster { cluster_id: String },
    
    /// Federated partner (formal agreement)
    Federated,
    
    /// Same lineage (auto-meet family devices)
    SameLineage,
}

/// Is this meeting mutual or one-way?
enum MeetingVisibility {
    /// I can see them, they can see me
    Mutual,
    
    /// I can see them, they can't see me (I received their seed)
    OneWayIn,
    
    /// They can see me, I can't see them (I shared my seed)
    OneWayOut,
}
```

### 3.3 Cluster Membership

```rust
/// Membership in a cluster beacon
struct ClusterMembership {
    /// Cluster identifier
    cluster_id: String,
    
    /// Cluster beacon seed (shared among members)
    cluster_seed: BeaconSeed,
    
    /// Entry point info (if this is the entry)
    entry_point: Option<EntryPointConfig>,
    
    /// Members we know (their beacon IDs)
    known_members: Vec<BeaconId>,
    
    /// When we joined
    joined_at: Timestamp,
    
    /// Our role in the cluster
    role: ClusterRole,
}

enum ClusterRole {
    /// Entry point - external peers find us first
    EntryPoint,
    
    /// Internal - only visible after entry point meeting
    Internal,
    
    /// Hub - connects to other clusters
    Hub,
}
```

---

## 4. Storage Model

### 4.1 File Structure

```
livespore-usb/
├── .beacon.seed                    # My beacon seed (32 bytes, encrypted)
├── .beacon.genetics.json           # Full genetics package (JSON)
├── .known_beacons.json             # Address book metadata (human-readable)
└── .beacon_seeds/                  # Individual met seeds (encrypted)
    ├── a3f912...seed               # Encrypted seed for beacon a3f912...
    ├── b7e4a9...seed               # Encrypted seed for beacon b7e4a9...
    └── ...
```

### 4.2 Encryption At Rest

**Critical**: Met beacon seeds are secrets! Encrypt with lineage seed at rest.

```rust
/// Store a met beacon seed (encrypted with lineage)
fn store_met_seed(
    beacon_id: &BeaconId,
    beacon_seed: &BeaconSeed,
    lineage_seed: &LineageSeed,
) -> Result<()> {
    // Encrypt the beacon seed with lineage-derived key
    let storage_key = lineage_seed.derive_storage_key("beacon-storage-v1")?;
    let encrypted = encrypt_with_key(&storage_key, beacon_seed.as_bytes())?;
    
    // Write to .beacon_seeds/{beacon_id}.seed
    let path = format!(".beacon_seeds/{}.seed", beacon_id.to_hex());
    fs::write(path, encrypted)?;
    
    Ok(())
}
```

**Why encrypt with lineage?**
- Lineage is the "family key"
- If device is lost, attacker can't read address book
- Family devices with same lineage can access (for sync)

### 4.3 JSON Metadata (Human-Readable)

```json
// .beacon.genetics.json
{
  "version": "2.0.0",
  "own_beacon_id": "d03029e5...",
  "lineage_hint": "8ff3b864a4bc589a",
  "sync_token": "4d5e6f7a-b1c2-d3e4-f5a6-b7c8d9e0f1a2",
  "last_sync": "2026-02-04T12:00:00Z",
  
  "meetings": {
    "c86cb868...": {
      "node_name": "pixel8a",
      "first_met": "2026-02-04T11:17:00Z",
      "last_seen": "2026-02-04T11:45:00Z",
      "endpoints": ["192.168.1.50:9900"],
      "capabilities_hint": ["mobile", "compute", "ai-client"],
      "notes": "Same lineage - family device",
      "relationship": "same_lineage",
      "visibility": "mutual",
      "seed_file": "c86cb868...seed"
    }
  },
  
  "clusters": [
    {
      "cluster_id": "basement-towers",
      "role": "entry_point",
      "joined_at": "2026-01-15T10:00:00Z",
      "known_members": ["a1b2c3...", "d4e5f6...", "g7h8i9..."]
    }
  ],
  
  "shared_with": ["c86cb868...", "a1b2c3..."]
}
```

---

## 5. Meeting Protocol

### 5.1 Direct Meeting (QR / NFC / Manual)

```
┌────────────┐                            ┌────────────┐
│  Alice     │                            │   Bob      │
│  (USB)     │                            │  (Pixel)   │
└─────┬──────┘                            └─────┬──────┘
      │                                         │
      │  1. Exchange public beacon IDs          │
      │  ────────────────────────────────────►  │
      │  ◄────────────────────────────────────  │
      │                                         │
      │  2. Verify via out-of-band channel      │
      │     (QR code, NFC, verbal, etc.)        │
      │                                         │
      │  3. Exchange beacon SEEDS (encrypted)   │
      │  ────────────────────────────────────►  │
      │  ◄────────────────────────────────────  │
      │                                         │
      │  4. Both add to address book            │
      │  (Met seeds stored, encrypted at rest)  │
      │                                         │
      │  5. NOW: Can decrypt each other's       │
      │     Dark Forest beacons! 🌑            │
      └─────────────────────────────────────────┘
```

### 5.2 Introduction Protocol

Alice knows Bob. Alice knows Carol. Alice introduces Bob to Carol.

```
┌────────────┐        ┌────────────┐        ┌────────────┐
│  Alice     │        │   Bob      │        │   Carol    │
└─────┬──────┘        └─────┬──────┘        └─────┬──────┘
      │                     │                     │
      │  1. Alice knows both Bob and Carol        │
      │     (has their beacon seeds)              │
      │                                           │
      │  2. Alice creates introduction packet:    │
      │     - Bob's beacon seed (encrypted for Carol)
      │     - Carol's beacon seed (encrypted for Bob)
      │                                           │
      │  3. Send to Bob ──────────────────────►   │
      │  4. Send to Carol ──────────────────────► │
      │                                           │
      │  5. Bob and Carol can now see each other  │
      │     (but know Alice introduced them)      │
      └───────────────────────────────────────────┘
```

### 5.3 Cluster Entry Meeting

External peer meets entry point → gets cluster beacon → can see internal members.

```
External Peer                Entry Point            Internal Node
     │                           │                       │
     │  1. Discover entry point  │                       │
     │     (public STUN, etc.)   │                       │
     │  ────────────────────────►│                       │
     │                           │                       │
     │  2. Meeting exchange      │                       │
     │  ◄───────────────────────►│                       │
     │                           │                       │
     │  3. Entry point shares    │                       │
     │     cluster beacon seed   │                       │
     │  ◄────────────────────────│                       │
     │                           │                       │
     │  4. External can now      │                       │
     │     decode cluster        │                       │
     │     broadcasts            │                       │
     │  ─────────────────────────┼──────────────────────►│
     │                           │                       │
     │  5. Connect to internal   │                       │
     │     nodes directly        │                       │
     │                           │                       │
```

---

## 6. Sync Protocol (Same Lineage)

### 6.1 When Two Lineage-Related Devices Connect

```
USB (Desktop)                                  Pixel (Phone)
     │                                              │
     │  Lineage verification                        │
     │  ──────────────────────────────────────────► │
     │  ◄────────────────────────────────────────── │
     │                                              │
     │  Lineage matches! (same family)              │
     │                                              │
     │  Exchange beacon genetics packages           │
     │  ──────────────────────────────────────────► │
     │  ◄────────────────────────────────────────── │
     │                                              │
     │  MERGE address books:                        │
     │  - Union of met_seeds                        │
     │  - Take newer last_seen                      │
     │  - Keep all endpoints                        │
     │  - Concatenate notes                         │
     │                                              │
     │  RESULT: Both know everyone either met!      │
     └──────────────────────────────────────────────┘
```

### 6.2 Merge Rules

```rust
/// Merge two beacon genetics packages (same lineage sync)
fn merge_genetics(local: &mut BeaconGenetics, remote: &BeaconGenetics) -> MergeResult {
    let mut added = 0;
    let mut updated = 0;
    
    for (beacon_id, remote_record) in &remote.met_seeds {
        if let Some(local_record) = local.met_seeds.get_mut(beacon_id) {
            // Exists locally - merge
            
            // Take newer timestamp
            if remote_record.last_seen > local_record.last_seen {
                local_record.last_seen = remote_record.last_seen;
            }
            
            // Union endpoints
            for ep in &remote_record.endpoints {
                if !local_record.endpoints.contains(ep) {
                    local_record.endpoints.push(ep.clone());
                }
            }
            
            // Concatenate notes (if different)
            if !local_record.notes.contains(&remote_record.notes) {
                local_record.notes = format!(
                    "{}\n[Synced]: {}",
                    local_record.notes,
                    remote_record.notes
                );
            }
            
            updated += 1;
        } else {
            // New meeting - add it
            local.met_seeds.insert(beacon_id.clone(), remote_record.clone());
            added += 1;
        }
    }
    
    // Also sync shared_with
    local.shared_with.extend(remote.shared_with.iter().cloned());
    
    // Update sync token
    local.sync_token = Uuid::new_v4();
    local.last_sync = Timestamp::now();
    
    MergeResult { added, updated }
}
```

---

## 7. Beacon Broadcast Flow

### 7.1 Broadcasting (Sender)

```rust
async fn broadcast_beacon(genetics: &BeaconGenetics) -> Result<()> {
    // Build payload (what family members will see)
    let payload = BeaconPayload {
        beacon_id: genetics.own_id.clone(),
        node_id: get_node_id(),
        endpoints: get_endpoints(),
        capabilities_hash: hash_capabilities(&my_caps),
        timestamp: Timestamp::now(),
    };
    
    // Encrypt with OWN seed
    let encrypted = genetics.own_seed.encrypt(&serialize(payload))?;
    
    // Broadcast (observers see only noise!)
    multicast_send(DarkForestBeacon {
        encrypted_payload: encrypted.ciphertext,
        nonce: encrypted.nonce,
        timestamp: encrypted.timestamp,
    });
    
    Ok(())
}
```

### 7.2 Receiving (Listener)

```rust
async fn receive_beacon(
    beacon: DarkForestBeacon,
    genetics: &BeaconGenetics,
) -> Option<DiscoveredPeer> {
    // Try all met beacon seeds
    for (beacon_id, record) in &genetics.met_seeds {
        if let Some(plaintext) = record.beacon_seed.try_decrypt(&beacon.to_ciphertext())? {
            let payload: BeaconPayload = deserialize(&plaintext)?;
            
            return Some(DiscoveredPeer {
                beacon_id: payload.beacon_id,
                node_id: payload.node_id,
                endpoints: payload.endpoints,
                met_via: beacon_id.clone(),
            });
        }
    }
    
    // Also try cluster beacons
    for cluster in &genetics.clusters {
        if let Some(plaintext) = cluster.cluster_seed.try_decrypt(&beacon.to_ciphertext())? {
            let payload: BeaconPayload = deserialize(&plaintext)?;
            
            return Some(DiscoveredPeer {
                beacon_id: payload.beacon_id,
                node_id: payload.node_id,
                endpoints: payload.endpoints,
                met_via: cluster.cluster_id.clone().into(),
            });
        }
    }
    
    // Can't decrypt - not someone we've met (TRUE Dark Forest!)
    None
}
```

---

## 8. Relationship to Lineage

### 8.1 Beacon ≠ Lineage

| Aspect | Beacon Genetics | Lineage Genetics |
|--------|----------------|------------------|
| **Purpose** | Discovery/visibility | Permissions |
| **Question** | "Can I see them?" | "What can I do?" |
| **Acquisition** | Meetings (social) | Inheritance (family) |
| **Sharing** | Promiscuous | Strict |
| **Storage** | Address book | Identity |
| **Rotation** | Can rotate freely | Stable (identity) |

### 8.2 Relationship Flow

```
┌─────────────────────────────────────────────────────────────┐
│  LAYER 1: BEACON GENETICS (Discovery)                       │
│                                                             │
│  "Can I decode their beacon?"                               │
│                                                             │
│  YES = I've met them (have their beacon seed)               │
│  NO  = They're invisible to me (TRUE Dark Forest)           │
└─────────────────────────────────────────────────────────────┘
                            │
                            │ If YES, proceed to Layer 2
                            ▼
┌─────────────────────────────────────────────────────────────┐
│  LAYER 2: LINEAGE GENETICS (Permissions)                    │
│                                                             │
│  "What can they do on my systems?"                          │
│                                                             │
│  Close lineage → Full access                                │
│  Distant lineage → Read-only                                │
│  Federated → Specific capabilities                          │
│  No lineage → Beacon visible, no access                     │
└─────────────────────────────────────────────────────────────┘
```

### 8.3 Use Case: Pixel Meets, Desktop Uses

```
1. Pixel meets "Dave" at conference
   → Pixel address book: Dave's beacon seed added
   
2. Pixel syncs with Desktop (same lineage)
   → Desktop address book: Dave's beacon seed copied
   
3. Desktop can now discover Dave's broadcasts
   → Uses Dave's beacon seed to decrypt
   
4. Desktop connects to Dave
   → Lineage verification for permissions
   → Dave might have read-only access (not full family)
```

---

## 9. Implementation Plan

### Phase 2A: Beacon Genetics Package (biomeOS)

**Tasks**:
- [ ] Design `BeaconGenetics` struct
- [ ] Implement storage model (JSON + encrypted seeds)
- [ ] Add sync protocol between same-lineage devices
- [ ] Unit tests for merge/sync

**Location**: `crates/biomeos-genetics/` (new crate)

**Estimated**: 2-3 hours

### Phase 2B: Meeting Protocol (BearDog)

**Tasks**:
- [ ] Implement `beacon.meeting.initiate`
- [ ] Implement `beacon.meeting.complete`
- [ ] Store met seeds (encrypted at rest)
- [ ] Integration with existing `beacon.*` methods

**Location**: `beardog-genetics/src/birdsong/meeting.rs`

**Estimated**: 2-3 hours

### Phase 2C: Cluster Beacons (BearDog + biomeOS)

**Tasks**:
- [ ] Implement cluster beacon derivation
- [ ] Entry point configuration
- [ ] Internal beacon sharing after meeting
- [ ] Cluster management in biomeOS

**Estimated**: 3-4 hours

### Phase 2D: Songbird Dark Forest Integration

**Tasks**:
- [ ] Use BeaconGenetics for broadcast encryption
- [ ] Try all met seeds on receive
- [ ] Update discovery to use new model

**Estimated**: 2-3 hours

---

## 10. Summary

### The Mitochondrial Model

```
Beacon Genetics (Mitochondria)
├── Own seed (my genome)
├── Address book (met genomes)
├── Clusters (colony membership)
└── Lineage hint (loose tie for sync)
```

### Key Insights

1. **Beacon seed is SEPARATE from lineage seed**
   - Not derived from it
   - Independent lifecycle
   - Can be exchanged freely

2. **Address book contains ACTUAL SEEDS**
   - Not just IDs and metadata
   - Seeds let you decrypt their broadcasts
   - Encrypted at rest with lineage key

3. **Sync across family devices**
   - Same lineage → sync address books
   - "Pixel meets, Desktop uses"

4. **Promiscuous but secure**
   - Can meet anyone (build social network)
   - Permissions still controlled by lineage
   - "See them ≠ access them"

### The Biological Elegance

*"Your beacon genetics is your address book - who you've met, who can see you. Your lineage genetics is your security - what they can do. You can share your contacts without sharing your keys."*

---

## Appendix A: File Format Examples

### A.1 .beacon.genetics.json (Full)

```json
{
  "version": "2.0.0",
  "own_beacon_id": "d03029e5c5cd0c3b44e2e316118943d8",
  "lineage_hint": "8ff3b864a4bc589a",
  "sync_token": "4d5e6f7a-b1c2-d3e4-f5a6-b7c8d9e0f1a2",
  "last_sync": "2026-02-04T12:00:00Z",
  
  "meetings": {
    "c86cb868b057f996dbbbf9d2f41fbe60": {
      "node_name": "pixel8a",
      "first_met": "2026-02-04T11:17:00Z",
      "last_seen": "2026-02-04T14:30:00Z",
      "endpoints": ["192.168.1.50:9900", "10.0.0.5:9900"],
      "capabilities_hint": ["mobile", "compute", "ai-client"],
      "notes": "Same lineage - family device",
      "relationship": {
        "type": "same_lineage"
      },
      "visibility": "mutual",
      "seed_file": "c86cb868.seed"
    },
    "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4": {
      "node_name": "alice-laptop",
      "first_met": "2026-01-20T09:00:00Z",
      "last_seen": "2026-02-03T16:45:00Z",
      "endpoints": ["alice.example.com:9900"],
      "capabilities_hint": ["compute", "storage"],
      "notes": "Met at rust meetup, trusted collaborator",
      "relationship": {
        "type": "direct"
      },
      "visibility": "mutual",
      "seed_file": "a1b2c3d4.seed"
    },
    "e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2": {
      "node_name": "bob-server",
      "first_met": "2026-02-01T10:30:00Z",
      "last_seen": "2026-02-01T10:35:00Z",
      "endpoints": ["192.168.5.100:9900"],
      "capabilities_hint": ["storage", "ai-inference"],
      "notes": "Introduced by Alice",
      "relationship": {
        "type": "introduced",
        "by": "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4"
      },
      "visibility": "mutual",
      "seed_file": "e7f8a9b0.seed"
    }
  },
  
  "clusters": [
    {
      "cluster_id": "basement-towers",
      "role": "entry_point",
      "joined_at": "2026-01-15T10:00:00Z",
      "known_members": [
        "northgate-beacon-id",
        "southgate-beacon-id",
        "strandgate-beacon-id"
      ],
      "seed_file": "cluster-basement-towers.seed"
    }
  ],

  "tags": [
    {
      "tag": "gaming",
      "transport": "birdsong",
      "behavior": { "gpu_sharing": "true", "provenance": "relaxed" },
      "cluster": {
        "cluster_id": "gaming-friends",
        "role": "hub",
        "joined_at": "2026-02-10T18:00:00Z",
        "known_members": ["alice-beacon-id", "bob-beacon-id"],
        "seed_file": "cluster-gaming.seed"
      }
    },
    {
      "tag": "research",
      "transport": "dark_forest",
      "behavior": { "provenance": "strict", "data_pipelines": "true" },
      "cluster": {
        "cluster_id": "research-collab",
        "role": "entry_point",
        "joined_at": "2026-02-08T10:00:00Z",
        "known_members": ["bob-beacon-id"],
        "seed_file": "cluster-research.seed"
      }
    }
  ],
  
  "shared_with": [
    "c86cb868b057f996dbbbf9d2f41fbe60",
    "a1b2c3d4e5f6a1b2c3d4e5f6a1b2c3d4",
    "e7f8a9b0c1d2e3f4a5b6c7d8e9f0a1b2"
  ]
}
```

### A.2 Encrypted Seed File Format

```
.beacon_seeds/c86cb868.seed
├── Header: "BEACON-SEED-V1\n"
├── Salt: 16 bytes (random)
├── Nonce: 12 bytes (random)
├── Ciphertext: 32 bytes (encrypted seed)
├── Auth tag: 16 bytes (ChaCha20-Poly1305)
└── Total: ~80 bytes
```

---

**Document Version**: 2.0.0  
**Created**: February 4, 2026  
**Updated**: February 11, 2026 — Added Beacon Tags (phenotype expression layer)  
**Status**: DESIGN SPECIFICATION  
**Next Step**: Implement `BeaconTag` in `beacon_genetics/types.rs`, then Phase 2A

---

🧬 *"Beacon genetics is your address book. Lineage genetics is your security. Together, they create a social graph overlaid on cryptographic trust."* 🌑
