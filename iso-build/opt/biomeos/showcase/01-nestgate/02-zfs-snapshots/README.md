# 02 - ZFS Snapshots: Time-Travel for Your Data

**Duration**: 4 minutes  
**Prerequisites**: NestGate running, understanding of sovereign storage

---

## Overview

This demo shows NestGate's **ZFS snapshot** capabilities: instant point-in-time recovery, ransomware protection, and data time-travel.

**What it demonstrates**:
- Automatic hourly snapshots
- Instant rollback (seconds)
- Zero-copy snapshots (no space overhead)
- Ransomware resilience
- Data archaeology

---

## The Power of Snapshots

### Traditional Backups
```
Full backup every night:
  • Takes hours
  • Consumes massive storage
  • Recovery is slow
  • Limited history (7-30 days)
  • Expensive

Ransomware attack at 2 PM:
  ❌ Last backup was midnight
  ❌ Lose 14 hours of work
  ❌ Recovery takes hours
```

### ZFS Snapshots
```
Snapshot every hour:
  • Takes 1 second
  • Zero extra storage (copy-on-write)
  • Recovery is instant
  • Unlimited history
  • Free

Ransomware attack at 2 PM:
  ✅ Rollback to 1 PM (1 hour lost)
  ✅ Recovery takes 5 seconds
  ✅ All data restored
```

---

## Run the Demo

```bash
cd showcase/01-nestgate/02-zfs-snapshots
./demo.sh
```

---

## What You'll See

### Phase 1: Create Data + Snapshot
```
📝 Storing document v1...
✅ Stored: document_123 (v1: "Initial content")
📸 Snapshot created: nestgate@2025-12-28_15:00:00
```

### Phase 2: Modify Data + Snapshot
```
✏️  Updating document to v2...
✅ Updated: document_123 (v2: "Modified content")
📸 Snapshot created: nestgate@2025-12-28_16:00:00
```

### Phase 3: Simulate Ransomware
```
💀 SIMULATING RANSOMWARE ATTACK...
❌ All data encrypted by attacker!
📊 Files affected: 1,234
💰 Ransom demanded: 5 BTC
```

### Phase 4: Instant Recovery
```
⏮️  Rolling back to pre-attack snapshot...
✅ Rolled back to: nestgate@2025-12-28_15:00:00
✅ All data restored in 3 seconds!
✅ Content restored: "Initial content"
```

### Phase 5: Time Travel
```
🕰️  Available snapshots:
  1. nestgate@2025-12-28_14:00:00 (2 hours ago)
  2. nestgate@2025-12-28_15:00:00 (1 hour ago)
  3. nestgate@2025-12-28_16:00:00 (current)

Select version to restore: 2
✅ Restored document from 1 hour ago!
```

---

## Architecture

```
┌─────────────────────────────────────────┐
│       ZFS Storage Pool                  │
│                                         │
│  Active Filesystem                      │
│  └─ /data/document_123 (v3)             │
│                                         │
│  Snapshots (read-only, instant)         │
│  ├─ @14:00 → document_123 (v1)          │
│  ├─ @15:00 → document_123 (v2)          │
│  └─ @16:00 → document_123 (v3)          │
│                                         │
│  Copy-on-Write:                         │
│  • Snapshots share unchanged blocks     │
│  • Only deltas consume space            │
│  • Recovery is instant (pointer swap)   │
└─────────────────────────────────────────┘
```

---

## ZFS Snapshot Features

### 1. Instant Creation
```bash
# Create snapshot (1 second!)
zfs snapshot nestgate@now

# No file copying
# No performance impact
# Instant completion
```

### 2. Zero-Copy Technology
```bash
# 1 TB of data
# Snapshot 1: +0 bytes (shares all blocks)
# Modify 1 GB
# Snapshot 2: +1 GB (only delta)

# Result: Unlimited snapshots, minimal space!
```

### 3. Instant Rollback
```bash
# Rollback to previous snapshot
zfs rollback nestgate@15:00:00

# Time: ~3 seconds (regardless of data size!)
# 1 TB? 3 seconds
# 100 TB? 3 seconds
# Just pointer updates
```

### 4. Ransomware Protection
```
Attack Timeline:
  14:00 - Snapshot (clean data)
  14:30 - Ransomware infection
  14:31 - Files encrypted
  14:32 - Discovery
  14:33 - Rollback to 14:00
  14:35 - FULLY RESTORED

Downtime: 2 minutes
Data loss: 0 bytes
Ransom paid: $0
```

---

## API Reference

### Create Snapshot
```bash
POST /api/snapshot/create
Headers:
  Authorization: Bearer <jwt_token>
Body:
  {
    "name": "before_upgrade",
    "description": "Pre-system-upgrade snapshot"
  }

Response:
  {
    "snapshot": "nestgate@before_upgrade",
    "created_at": "2025-12-28T15:30:00Z",
    "size_mb": 0
  }
```

### List Snapshots
```bash
GET /api/snapshot/list
Headers:
  Authorization: Bearer <jwt_token>

Response:
  {
    "snapshots": [
      {
        "name": "nestgate@2025-12-28_14:00:00",
        "created_at": "2025-12-28T14:00:00Z",
        "used_mb": 120,
        "referenced_mb": 5000
      },
      ...
    ]
  }
```

### Rollback
```bash
POST /api/snapshot/rollback
Headers:
  Authorization: Bearer <jwt_token>
Body:
  {
    "snapshot": "nestgate@2025-12-28_14:00:00"
  }

Response:
  {
    "success": true,
    "rolled_back_to": "2025-12-28T14:00:00Z",
    "files_restored": 1234,
    "time_seconds": 3
  }
```

### Delete Snapshot
```bash
DELETE /api/snapshot/:name
Headers:
  Authorization: Bearer <jwt_token>

Response:
  {
    "deleted": "nestgate@old_snapshot",
    "space_freed_mb": 450
  }
```

---

## Use Cases

### Use Case 1: Ransomware Recovery
```
Timeline:
  Monday 9 AM: Clean data, snapshot created
  Monday 2 PM: Ransomware encrypts everything
  Monday 2:05 PM: Detected + rolled back to 9 AM
  
Result:
  ✅ All data restored
  ✅ 5 hours of work lost (acceptable)
  ✅ $0 ransom paid
  ✅ 5 minutes downtime
```

### Use Case 2: Software Upgrade Safety
```
Process:
  1. Take snapshot: @before_upgrade
  2. Upgrade NestGate to v2.0
  3. Test new version
  4. Problem? Rollback to @before_upgrade
  5. Works? Delete @before_upgrade
  
Result:
  ✅ Zero-risk upgrades
  ✅ Instant rollback if issues
  ✅ Production confidence
```

### Use Case 3: Data Archaeology
```
Scenario: "What did this document say 3 weeks ago?"

Solution:
  1. List snapshots from 3 weeks ago
  2. Mount snapshot as read-only
  3. View historical version
  4. Extract if needed
  
Result:
  ✅ Perfect audit trail
  ✅ Compliance requirements met
  ✅ Historical analysis possible
```

---

## Snapshot Strategies

### Strategy 1: Frequent + Pruning
```yaml
schedule:
  hourly: keep 24    # Last day (hourly granularity)
  daily: keep 30     # Last month (daily granularity)
  weekly: keep 12    # Last quarter (weekly granularity)
  monthly: keep 12   # Last year (monthly granularity)

Result: 78 snapshots, minimal space
```

### Strategy 2: Event-Based
```yaml
triggers:
  - before_system_upgrade
  - before_data_migration
  - after_successful_backup
  - on_demand (user request)

Result: Snapshots when it matters
```

### Strategy 3: Continuous
```yaml
schedule:
  every: 15 minutes
  retention: 7 days
  
Result: Maximum granularity, 672 snapshots
```

---

## Technical Details

### Copy-on-Write Explained
```
Initial State:
  Block 1: "Hello"
  Block 2: "World"
  
Snapshot @snap1 created:
  snap1 → Block 1, Block 2 (shared)
  
Modify Block 1 to "Goodbye":
  Block 1 (old): "Hello" (snap1)
  Block 3 (new): "Goodbye" (active)
  Block 2: "World" (shared by both)
  
Result: 1 block overhead (only changed block)
```

### Space Efficiency
```
Example: 1 TB dataset, 100 snapshots

Traditional backups:
  1 TB × 100 = 100 TB storage
  
ZFS snapshots:
  1 TB + (changes only) ≈ 1.2 TB storage
  
Savings: 98.8%!
```

### Performance Impact
```
Snapshot creation:
  Time: O(1) - constant time
  CPU: Minimal
  I/O: None
  
Rollback:
  Time: O(1) - constant time
  CPU: Minimal
  I/O: Metadata updates only
  
Result: No performance penalty!
```

---

## Comparison

| Feature | Traditional | ZFS Snapshots |
|---------|-------------|---------------|
| **Creation Time** | Hours | **1 second** |
| **Storage Overhead** | 100% per backup | **~2% total** |
| **Recovery Time** | Hours | **3 seconds** |
| **Granularity** | Daily | **Every 15 min** |
| **History** | 7-30 days | **Unlimited** |
| **Ransomware Protection** | Partial | **Complete** |
| **Space Efficiency** | Low | **Very High** |
| **Cost** | Expensive | **Free** |

---

## Security Considerations

### Protection Against
- ✅ Ransomware (rollback before encryption)
- ✅ Accidental deletion (restore from snapshot)
- ✅ Data corruption (checksums + snapshots)
- ✅ Bad updates (rollback system)
- ✅ User errors (undo changes)

### Limitations
- ⚠️ Snapshots are NOT backups (same disk)
- ⚠️ Hardware failure destroys snapshots too
- ⚠️ Need separate replication for DR

### Best Practice: 3-2-1 Rule
```
3 copies of data:
  1. Production (active filesystem)
  2. Snapshots (same disk, instant recovery)
  3. Remote replica (different tower, DR)
  
2 different media:
  1. Local NVMe/SSD
  2. Remote tower
  
1 offsite:
  1. Different geographic location
```

---

## Success Criteria

✅ **Instant Snapshots**: Created in < 1 second  
✅ **Instant Recovery**: Rollback in < 5 seconds  
✅ **Space Efficient**: < 5% overhead for 24 snapshots  
✅ **Ransomware Proof**: Can recover from any snapshot  
✅ **Time Travel**: Access historical versions  

---

## Integration with Federation

### Replicate Snapshots Across Towers
```bash
# Tower 1: Create snapshot
POST tower1.local/api/snapshot/create

# Tower 1: Replicate to Tower 2
POST tower1.local/api/snapshot/replicate
  destination: tower2.local
  snapshot: nestgate@2025-12-28_15:00:00

# Tower 2: Receives incremental changes
# Now both towers have the snapshot!

Result: Geographic redundancy + instant local recovery
```

---

## Next Steps

After this demo:
- **03-lineage-collaboration**: Share snapshots via lineage
- **04-federation-replication**: Replicate across towers
- **05-benchscale-validation**: Test at scale

---

**Philosophy**: *"Time-travel for data. Protection from chaos. Sovereignty from fear."*

