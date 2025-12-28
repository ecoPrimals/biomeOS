# 01 - BTSP Tunnel Coordination

**Demonstrates**: Complete BTSP tunnel lifecycle management  
**Status**: Ready to implement  
**Prerequisites**: BiomeOS, BearDog, Songbird  

---

## What This Demonstrates

- BTSP tunnel establishment between peers
- Real-time health monitoring
- Automatic recovery from degradation
- Key rotation for security
- Graceful tunnel shutdown
- BiomeOS as BTSP coordinator

---

## Architecture

```
┌─────────┐                  ┌─────────┐
│ Peer A  │                  │ Peer B  │
└────┬────┘                  └────┬────┘
     │                            │
     │    ┌──────────────┐       │
     └───►│   BiomeOS    │◄──────┘
          │ BTSP Coord.  │
          └──────┬───────┘
                 │
        ┌────────┴────────┐
        │                 │
    ┌───▼────┐      ┌────▼────┐
    │BearDog │      │Songbird │
    │Crypto  │      │Discovery│
    └────────┘      └─────────┘
```

---

## BTSP Tunnel Lifecycle

### 1. Discovery Phase
```bash
# Songbird discovers peers
peers=$(songbird discover)
# Result: peer-a, peer-b found
```

### 2. Establishment Phase
```bash
# BiomeOS establishes tunnel
tunnel_id=$(biomeos btsp establish peer-a peer-b)
# Result: Tunnel established with BearDog encryption
```

### 3. Monitoring Phase
```bash
# BiomeOS monitors tunnel health
health=$(biomeos btsp health $tunnel_id)
# Result: Healthy | Degraded | Unhealthy
```

### 4. Recovery Phase (if degraded)
```bash
# BiomeOS automatically recovers
biomeos btsp recover $tunnel_id
# Actions: Key rotation, transport optimization, path reestablishment
```

### 5. Shutdown Phase
```bash
# BiomeOS graceful shutdown
biomeos btsp shutdown $tunnel_id
# Result: Clean termination, resources released
```

---

## Key Capabilities

### Tunnel Establishment
- Peer discovery via Songbird
- Automatic endpoint negotiation
- BearDog encryption integration
- Tunnel ID generation
- State tracking

### Health Monitoring
- Real-time status checks
- Security health (key expiration, rotation status)
- Transport health (latency, packet loss)
- Overall status computation
- Alert generation

### Automatic Recovery
- Degradation diagnosis
- Recovery strategy selection
  - Key rotation (security issues)
  - Transport optimization (performance issues)
  - Path reestablishment (connectivity issues)
- Recovery verification
- Success/failure reporting

### Key Rotation
- Automatic expiration detection
- Coordinate with BearDog
- Zero-downtime rotation
- Verification of new keys

### Graceful Shutdown
- Clean tunnel termination
- Resource cleanup
- State persistence
- Notification to peers

---

## Demo Script Structure

```bash
#!/bin/bash
# Demo 01: BTSP Tunnel Coordination

echo "🌐 BTSP Tunnel: Complete Lifecycle"
echo "==================================="

# 1. Discovery
echo "🔍 Discovering peers..."
# Use Songbird to find available peers

# 2. Establishment
echo "🔗 Establishing BTSP tunnel..."
# BiomeOS coordinates tunnel setup

# 3. Health Check
echo "💓 Monitoring tunnel health..."
# Real-time health status

# 4. Simulate Degradation
echo "⚠️  Simulating degradation..."
# Inject latency or security issue

# 5. Automatic Recovery
echo "🔄 Automatic recovery..."
# BiomeOS diagnoses and repairs

# 6. Verify Recovery
echo "✅ Verifying recovery..."
# Confirm tunnel healthy again

# 7. Graceful Shutdown
echo "👋 Graceful shutdown..."
# Clean termination

echo "🎉 BTSP tunnel lifecycle complete!"
```

---

## Testing Scenarios

### Happy Path
1. Establish tunnel ✅
2. Monitor health (Healthy) ✅
3. Send data ✅
4. Shutdown gracefully ✅

### Degradation - Security
1. Establish tunnel ✅
2. Key approaching expiration ⚠️
3. Auto-recovery (key rotation) 🔄
4. Verify healthy ✅

### Degradation - Transport
1. Establish tunnel ✅
2. High latency detected ⚠️
3. Auto-recovery (optimize path) 🔄
4. Verify healthy ✅

### Degradation - Connectivity
1. Establish tunnel ✅
2. Peer unreachable ⚠️
3. Auto-recovery (reestablish) 🔄
4. Verify healthy ✅

---

## Success Criteria

### Establishment
- ✅ Tunnel created successfully
- ✅ Encryption active (BearDog)
- ✅ Peers connected
- ✅ Health status: Healthy

### Monitoring
- ✅ Real-time health updates
- ✅ Security status tracked
- ✅ Transport metrics available
- ✅ Overall status computed

### Recovery
- ✅ Degradation detected
- ✅ Root cause diagnosed
- ✅ Recovery strategy executed
- ✅ Tunnel restored to healthy

### Shutdown
- ✅ Clean termination
- ✅ Resources released
- ✅ No leaked connections
- ✅ State cleaned up

---

## Implementation Notes

### BiomeOS BTSP Coordinator

**Already Implemented** (from earlier session):
```rust
// btsp.rs - Recovery implementation
async fn recover_degraded_tunnel(&self, tunnel_id: &str) -> Result<TunnelInfo> {
    // Diagnose degradation
    let cause = self.diagnose_degradation(tunnel_id).await?;
    
    // Apply recovery strategy
    match cause {
        DegradationCause::SecurityKeyExpiring => self.rotate_tunnel_keys(tunnel_id).await?,
        DegradationCause::TransportLatency => self.optimize_transport_path(tunnel_id).await?,
        DegradationCause::PartialConnectivity => self.reestablish_transport(tunnel_id).await?,
    }
    
    // Verify recovery
    let health = self.security.check_tunnel_health(tunnel_id).await?;
    // ...
}
```

**What the Demo Will Show**:
- This code running with real primals
- Actual recovery in action
- Real health monitoring
- Honest gap reporting if issues

---

## Validation

### E2E Test
```bash
# Will be added to run-e2e-tests.sh
run_demo_test "showcase/03-p2p-coordination/01-btsp-tunnel-coordination/demo.sh"

# Expected: PASS (if all primals available)
# Expected: Graceful skip/gap report (if missing)
```

### Manual Validation
```bash
# Run demo
bash showcase/03-p2p-coordination/01-btsp-tunnel-coordination/demo.sh

# Verify:
# - Tunnel established
# - Health monitoring active
# - Recovery triggered
# - Tunnel restored
# - Clean shutdown
```

---

**Status**: 📋 Planned, Ready to Implement  
**Next**: Build demo.sh script  
**Integration**: Real primals, no mocks  

🌐 **BTSP: Production-Grade P2P Tunnel Coordination**
