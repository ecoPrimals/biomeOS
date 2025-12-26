# Demo 01: BTSP Tunnel Coordination

**Time:** 30 minutes  
**Difficulty:** 🔴 Advanced  
**Status:** ✅ Ready to run

---

## 🎯 What This Demo Shows

This demo demonstrates **BiomeOS coordinating BTSP tunnel creation in pure Rust**.

### Key Features

1. **Capability-Based Discovery**
   - BiomeOS discovers primals by capability (not by name!)
   - "I need security capability" (not "I need BearDog")
   - "I need discovery capability" (not "I need Songbird")

2. **Pure Rust Coordination**
   - All coordination logic in Rust
   - No shell scripts calling CLIs
   - Type-safe, production-ready code

3. **Agnostic Architecture**
   - Works with any primal implementing `SecurityProvider`
   - Works with any primal implementing `DiscoveryProvider`
   - BearDog + Songbird today, YourPrimal tomorrow

4. **Real Error Handling**
   - Proper `Result` types
   - Context on failures
   - Graceful degradation

---

## 🚀 Run the Demo

```bash
cargo run
```

---

## 📊 Expected Output

```
🌱 BiomeOS P2P Coordination Demo: BTSP Tunnel
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━

🔍 Step 1: Discovering primals by capability...
   Looking for: security capability (BTSP support)
   Looking for: discovery capability (transport registration)

⚠️  Note: Using mock providers for demonstration
   In production, BiomeOS discovers real primals by capability

✅ Found security primal: MockSecurity (demonstrates BearDog)
✅ Found discovery primal: MockDiscovery (demonstrates Songbird)

🔐 Step 2: Creating BTSP tunnel coordinator...
✅ Coordinator created

🔗 Step 3: Coordinating BTSP tunnel creation...
   Node A: alice
   Node B: bob

   Requesting tunnel from security primal...
   Registering endpoints with discovery primal...
   Verifying tunnel health...

✅ BTSP tunnel created successfully!

📊 Tunnel Information:
   Tunnel ID: tunnel-alice-bob
   Status: Active
   Endpoints: 2 nodes
   Established: 2025-12-26 10:30:45

📊 Step 4: Monitoring tunnel health...
✅ Health check complete:
   Security: Healthy
   Transport: Healthy
   Overall: Healthy

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
🎉 Demo complete!

Key Takeaways:
  ✅ BiomeOS discovered primals by capability (not by name)
  ✅ Pure Rust coordination (no shell scripts)
  ✅ Agnostic architecture (works with any compatible primals)
  ✅ Real error handling and health monitoring

Next Steps:
  - Run demo 02: BirdSong Encryption
  - Deploy with BYOB: templates/btsp-tunnel-only.biome.yaml
  - Test with real BearDog + Songbird
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

---

## 🏗️ Architecture

### Coordination Flow

```
BiomeOS
   │
   ├─► Discover "security" capability
   │   └─► Find primal (e.g., BearDog)
   │
   ├─► Discover "discovery" capability
   │   └─► Find primal (e.g., Songbird)
   │
   ├─► Create BtspCoordinator(security, discovery)
   │
   ├─► coordinator.create_tunnel("alice", "bob", proof)
   │   ├─► security.request_tunnel()
   │   ├─► discovery.register_transport(endpoint_a)
   │   ├─► discovery.register_transport(endpoint_b)
   │   └─► coordinator.monitor_tunnel()
   │
   └─► Return TunnelInfo
```

### Code Structure

```rust
// Capability-based discovery (agnostic!)
let security = biome.discover_primal("security").await?;
let discovery = biome.discover_primal("discovery").await?;

// Pure Rust coordination
let coordinator = BtspCoordinator::new(security, discovery);
let tunnel = coordinator.create_tunnel("alice", "bob", proof).await?;

// Health monitoring
let health = coordinator.monitor_tunnel(&tunnel.id).await?;
```

---

## 🔧 Extending This Demo

### Connect to Real Primals

Replace mock providers with real adapters:

```rust
// Real BearDog adapter
let security = Arc::new(BearDogAdapter::new("/path/to/beardog")?);

// Real Songbird adapter
let discovery = Arc::new(SongbirdAdapter::new("http://localhost:3000")?);

let coordinator = BtspCoordinator::new(security, discovery);
```

### Add Custom Security Provider

Implement your own security primal:

```rust
struct MySecurityPrimal;

#[async_trait]
impl SecurityProvider for MySecurityPrimal {
    async fn request_tunnel(...) -> Result<TunnelRequest> {
        // Your implementation
    }
}
```

---

## 📚 Key Concepts

### 1. Capability-Based Discovery

**Traditional:**
```rust
let beardog = find_beardog();  // Hardcoded!
```

**BiomeOS:**
```rust
let security = discover_primal("security").await?;  // Agnostic!
```

### 2. Trait-Based Coordination

```rust
pub trait SecurityProvider {
    async fn request_tunnel(...) -> Result<TunnelRequest>;
}
```

Works with **any** primal implementing this trait!

### 3. Pure Rust (Not Shell Scripts)

**Traditional:**
```bash
beardog create-tunnel alice bob
```

**BiomeOS:**
```rust
coordinator.create_tunnel("alice", "bob", proof).await?
```

---

## 🎯 Next Steps

1. **Run demo 02:** BirdSong Encryption
2. **Deploy with BYOB:** `templates/btsp-tunnel-only.biome.yaml`
3. **Test with real primals:** Connect to actual BearDog + Songbird
4. **Implement custom provider:** Create your own security primal

---

**This is BiomeOS's killer feature: Pure Rust P2P coordination!** 🚀

