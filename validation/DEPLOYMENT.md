# BiomeOS Deployment

**Deploying biomeOS and primals to validated VMs**

---

## Overview

The deployment system provides:
- **SSH-based deployment** to provisioned VMs
- **BiomeOS core** installation
- **Primal deployment** (Songbird, etc.)
- **Verification** of successful deployment

---

## Quick Start

### Full Validation Pipeline

```bash
cd validation

# Provision + Deploy (2-node federation)
cargo run --bin validate-federation

# Or 3-node federation
cargo run --bin validate-federation federation-3node
```

**Result**: VMs provisioned, biomeOS deployed, ready for Songbird P2P

---

## Deployment Architecture

### Phases

```
Phase 1: Provision VMs ✅
    ↓
Phase 2: Deploy biomeOS ✅
    ↓
Phase 3: Start Songbird P2P 🚧
    ↓
Phase 4: Validate mDNS 🚧
    ↓
Phase 5: Confirm Federation 🚧
```

### Current Status

**✅ Complete**:
- Phase 1: VM provisioning (all topologies)
- Phase 2: BiomeOS deployment with verification

**🚧 TODO**:
- Phase 3: Songbird orchestrate startup
- Phase 4: mDNS/UDP discovery validation
- Phase 5: Federation coordination testing

---

## Deployment Configuration

### Default Configuration

```rust
BiomeOsDeployment {
    install_path: "/opt/biomeos",
    primals: vec!["songbird"],
}
```

### Custom Deployment

```rust
use biomeos_validation::BiomeOsDeployment;

let deployment = BiomeOsDeployment {
    install_path: PathBuf::from("/opt/biomeos"),
    primals: vec![
        "songbird".to_string(),
        "toadstool".to_string(),
    ],
};
```

---

## Usage Examples

### Basic Validation

```bash
cd validation

# Provision and deploy
cargo run --bin validate-federation
```

**Output**:
```
Phase 1: Provision federation-2node Topology
✅ VM created (192.168.122.X)
✅ VM created (192.168.122.Y)

Phase 2: Deploy biomeOS
  📦 Deploying biomeOS to federation-vm1...
    • Testing SSH connectivity...
    ✅ SSH connected
    • Creating install directory...
    ✅ Install directory ready
    • Deploying biomeOS core...
    ✅ biomeOS core deployed
    • Deploying songbird...
    ✅ songbird deployed
  ✅ Deployment complete

Phase 2b: Verify Deployment
  🔍 Verifying deployment on federation-vm1...
  ✅ Deployment verified
```

### Manual Deployment

```bash
# Provision VMs first
cd validation
cargo run --bin provision-topology federation-2node

# Then deploy biomeOS
# (Currently integrated in validate-federation)
```

---

## Verification

### What Gets Verified

1. **Install Directory**
   - `/opt/biomeos` exists
   - Correct permissions

2. **BiomeOS Core**
   - Core marker file exists
   - Ready for primal orchestration

3. **Primals**
   - Each primal marker exists
   - Ready to start

### Manual Verification

```bash
# SSH to VM
ssh biomeos@<VM_IP>

# Check install
ls -la /opt/biomeos

# Check biomeOS
cat /opt/biomeos/biomeos.marker

# Check primals
cat /opt/biomeos/songbird.marker
```

---

## SSH Operations

### Test Connectivity

```rust
let vm = DeployedVm::new("vm1".to_string(), "192.168.122.100".to_string());
vm.test_ssh()?;  // Returns true if connected
```

### Execute Commands

```rust
let output = vm.ssh_exec("ls -la /opt")?;
println!("{}", output);
```

### Copy Files

```rust
vm.scp_to(&local_file, "/opt/biomeos/config.yaml")?;
```

---

## Next Steps (Phase 3-5)

### Phase 3: Start Songbird P2P

```rust
// TODO: Implement
for vm in &deployed_vms {
    vm.ssh_exec("cd /opt/biomeos && ./songbird orchestrate")?;
}
```

### Phase 4: Validate mDNS

```rust
// TODO: Implement
let services = vm.ssh_exec("avahi-browse -a -t -r")?;
// Parse and verify peer discovery
```

### Phase 5: Confirm Federation

```rust
// TODO: Implement
// Test P2P communication
// Verify coordination
// Validate data replication
```

---

## Error Handling

### SSH Connection Failures

```
Error: SSH connectivity test failed for federation-vm1
```

**Solution**: Wait for cloud-init to complete (~30 seconds)

### Deployment Verification Failures

```
Error: Deployment verification failed for federation-vm1
❌ biomeOS core not found
```

**Solution**: Re-run deployment or check SSH connectivity

---

## Cleanup

### After Validation

```bash
# Get VM names from output
sudo virsh destroy federation-vm1 federation-vm2
sudo virsh undefine federation-vm1 federation-vm2 --remove-all-storage
```

### Automated Cleanup (TODO)

```bash
cargo run --bin cleanup-vms federation-2node
```

---

## Architecture

### Deployment Module

```rust
// validation/src/deployment.rs
pub struct DeployedVm {
    pub name: String,
    pub ip_address: String,
    pub ssh_user: String,
}

pub struct BiomeOsDeployment {
    pub install_path: PathBuf,
    pub primals: Vec<String>,
}
```

### Benefits

- **Type-safe**: Rust error handling
- **Observable**: Clear logging at each step
- **Testable**: Unit tests for configurations
- **Composable**: Mix VMs and deployments

---

## Current Capabilities

### ✅ Working Now

1. **VM Provisioning**
   - Multiple types (Desktop, Server, Federation, Compute)
   - Topology-based (2-node, 3-node, mixed)

2. **BiomeOS Deployment**
   - SSH-based installation
   - Primal deployment
   - Verification

3. **Validation Pipeline**
   - End-to-end provisioning + deployment
   - Clear phase separation
   - Observable progress

### 🚧 In Progress

4. **Songbird P2P**
   - Orchestrate startup
   - mDNS discovery
   - Federation coordination

---

## Status

| Phase | Status | Description |
|-------|--------|-------------|
| **1** | ✅ | VM provisioning (all topologies) |
| **2** | ✅ | BiomeOS deployment with verification |
| **3** | 🚧 | Songbird P2P startup |
| **4** | 🚧 | mDNS/UDP discovery validation |
| **5** | 🚧 | Federation coordination testing |

**Next**: Implement phases 3-5 for complete validation pipeline

---

**Quality**: Production-ready for Phases 1-2 ✅  
**Architecture**: Clean, composable, testable 🌟  
**Status**: Ready for Songbird P2P integration 🚀

