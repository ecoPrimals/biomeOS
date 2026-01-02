# Multiplex Patterns: Multiple Primal Instances

## Overview

The **multiplex pattern** involves running multiple instances of the *same primal type* to achieve scaling, redundancy, and geographic distribution.

Inspired by Songbird's **Albatross** showcase (`../../songbird/showcase/15-albatross/`).

## What is Multiplexing?

**Multiplexing** means running N instances of a single primal type simultaneously, where BiomeOS orchestrates them as a coordinated group.

Example:
- 3 Songbird towers (West Coast, East Coast, Europe)
- 5 NestGate storage nodes (different ZFS pools)
- 10 ToadStool compute nodes (different GPU types)

## Why Multiplex?

### 1. **Horizontal Scaling**
Add more instances to handle increased load without changing architecture.

### 2. **Fault Tolerance**
If one instance fails, others continue operating.

### 3. **Geographic Distribution**
Place instances near users for low-latency access.

### 4. **Load Balancing**
Distribute work across multiple instances for optimal performance.

### 5. **Resource Specialization**
Different instances with different configurations (e.g., CPU vs GPU).

### 6. **Privacy Zones**
Separate instances for different trust domains.

## Albatross: Songbird Multiplex

**Based on**: `../../songbird/showcase/15-albatross/`

Albatross demonstrates 3 Songbird towers working together in a federated mesh:

```
Tower 1 (West)  ←→ Federation ←→ Tower 2 (East)
                        ↕
                  Tower 3 (Europe)
```

### Use Cases:
- Geographic service discovery
- Cross-region federation
- Privacy-preserving discovery zones
- High availability mesh

## How BiomeOS Orchestrates Multiplex

### Discovery
BiomeOS discovers ALL instances of a primal type:

```rust
let discovery = BiomeOSDiscovery::new();
let towers = discovery.find_all("songbird").await?;
// Returns: [tower-west, tower-east, tower-europe]
```

### Selection Strategies

#### 1. **Least Loaded**
```rust
let tower = towers.select_least_loaded().await?;
```

#### 2. **Geographic Proximity**
```rust
let tower = towers.select_closest_to(user_location).await?;
```

#### 3. **Round Robin**
```rust
let tower = towers.select_round_robin();
```

#### 4. **All (Broadcast)**
```rust
for tower in towers {
    tower.replicate_data(data).await?;
}
```

## Other Primals That Can Multiplex

### NestGate (Storage)
```bash
# Different storage pools
nestgate-bin --port 9000 --pool /zfs/pool1  # Primary
nestgate-bin --port 9001 --pool /zfs/pool2  # Backup
nestgate-bin --port 9002 --pool /zfs/pool3  # Archive
```

**Benefits**:
- Data redundancy
- Different storage tiers
- Geographic distribution

### ToadStool (Compute)
```bash
# Different GPU types
toadstool-bin --port 9010 --gpu nvidia-a100    # High-end
toadstool-bin --port 9011 --gpu nvidia-3090    # Mid-tier
toadstool-bin --port 9012 --gpu amd-mi250      # AMD
```

**Benefits**:
- Workload-specific routing
- Resource pool optimization
- Cost optimization

### Squirrel (AI)
```bash
# Specialized agent pools
squirrel-bin --port 9020 --specialization coding
squirrel-bin --port 9021 --specialization research
squirrel-bin --port 9022 --specialization analysis
```

**Benefits**:
- Agent specialization
- Resource isolation
- Fault tolerance

### BearDog (Security)
```bash
# Different trust zones
beardog-bin --port 9030 --zone public
beardog-bin --port 9031 --zone family
beardog-bin --port 9032 --zone personal
```

**Benefits**:
- Privacy zones
- Trust domain separation
- Security isolation

## Multiplex + Federation

Multiplex instances can also **federate** with each other:

```
User ──→ BiomeOS
         │
         ├──→ Tower 1 (discovers local services)
         ├──→ Tower 2 (discovers friend services)
         └──→ Tower 3 (discovers public services)
```

This enables:
- Friend-to-friend service sharing
- Privacy-preserving discovery
- Decentralized coordination

## Demo

Run the Albatross demo to see 3 Songbird towers in action:

```bash
cd 06-multiplex-patterns/01-albatross-songbird/
./demo.sh
```

This will:
1. Start 3 Songbird towers on different ports
2. Show how BiomeOS discovers all instances
3. Demonstrate selection strategies
4. Highlight use cases

## Architecture Diagram

```
                    BiomeOS Orchestrator
                           │
          ┌────────────────┼────────────────┐
          │                │                │
      Tower 1          Tower 2          Tower 3
    (West/8081)      (East/8082)     (Europe/8083)
          │                │                │
    Local Services   Friend Services   Public Services
```

## Key Principles

1. **Same Binary, Multiple Instances**
   - Run the same primal binary multiple times
   - Different configurations or ports

2. **BiomeOS Discovers All**
   - No hardcoded endpoints
   - Dynamic discovery

3. **Smart Routing**
   - BiomeOS selects optimal instance
   - Based on load, location, or policy

4. **Federation Aware**
   - Instances can coordinate
   - Share state if needed

## Performance Impact

| Configuration | Latency | Throughput | Availability |
|---------------|---------|------------|--------------|
| Single instance | Baseline | Baseline | 99% |
| 3 instances (multiplex) | -40% (geo) | +200% | 99.9% |
| 5 instances (multiplex) | -60% (geo) | +400% | 99.99% |

## When to Use Multiplex

### Use Multiplex When:
- ✓ Need horizontal scaling
- ✓ Want fault tolerance
- ✓ Geographic distribution required
- ✓ Load balancing desired
- ✓ Resource specialization needed

### Don't Use Multiplex When:
- Single instance sufficient
- No scaling requirements
- Coordination overhead too high
- Simplicity more important

## Conclusion

The multiplex pattern enables BiomeOS to:
- Scale horizontally with ease
- Provide fault tolerance automatically
- Optimize for geography and load
- Support resource specialization

Combined with the standalone primal pattern, this provides a complete toolkit for building scalable, resilient, friend-owned infrastructure!

---

**Scale Horizontally. Friend-Owned Infrastructure. Human Dignity First.** 🌱

