# 04 - Federation Replication: Geographic Sovereignty

**Duration**: 5 minutes  
**Prerequisites**: Multiple towers, Songbird federation

---

## Overview

Replicate NestGate storage across federated towers for geographic redundancy and disaster recovery.

**What it demonstrates**:
- Multi-tower replication
- Incremental sync (ZFS send/receive)
- Geographic distribution
- Automatic failover
- Split-brain prevention

---

## Philosophy

> "One tower is sovereignty.  
>  Two towers is resilience.  
>  Three towers is invincibility."

---

## Run the Demo

```bash
cd showcase/01-nestgate/04-federation-replication
./demo.sh
```

---

## Key Concepts

### Replication Strategy
```
Tower 1 (Primary - US West)
  ↓ ZFS incremental send
Tower 2 (Replica - US East)
  ↓ ZFS incremental send
Tower 3 (Replica - EU)

Data exists in 3 geographic locations
Any tower failure → Others continue
Split-brain prevented via Songbird
```

### Sync Efficiency
- **Initial**: Full dataset copy
- **Incremental**: Only changed blocks (~2% daily)
- **Compression**: Reduced bandwidth (40-60%)
- **Deduplication**: Efficient for similar data

---

## Demo Flow

1. Store data on Tower 1
2. Replicate to Tower 2 (incremental)
3. Replicate to Tower 3 (incremental)
4. Simulate Tower 1 failure
5. Tower 2 becomes primary
6. Tower 1 recovers, syncs from Tower 2

---

**Philosophy**: *"Data in one place is hope. Data in three places is sovereignty."*

