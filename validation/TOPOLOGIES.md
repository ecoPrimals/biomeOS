# Validation Topologies

**Control Substrate**: Reliable VM provisioning for biomeOS validation

---

## Overview

The validation substrate provides **topology-based VM provisioning** - spin up multiple VMs of different types for various testing scenarios.

**Philosophy**: Once we can reliably spin up VMs of different types, our control substrate is ready.

---

## Available Topologies

### 1. Simple Test
**Purpose**: Basic connectivity and SSH testing  
**VMs**: 2 desktop VMs  
**Resources**: 2GB RAM, 2 CPUs each  

```bash
cd validation
cargo run --bin provision-topology simple-test
```

**Use Case**: Quick validation, connectivity tests

---

### 2. Federation 2-Node
**Purpose**: Minimal Songbird P2P federation  
**VMs**: 2 federation nodes  
**Resources**: 3GB RAM, 2 CPUs each  

```bash
cd validation
cargo run --bin provision-topology federation-2node
```

**Use Case**: Songbird P2P testing, mDNS discovery, basic federation

---

### 3. Federation 3-Node
**Purpose**: Full federation with redundancy  
**VMs**: 3 federation nodes  
**Resources**: 3GB RAM, 2 CPUs each  

```bash
cd validation
cargo run --bin provision-topology federation-3node
```

**Use Case**: Multi-node coordination, fault tolerance, network partitioning tests

---

### 4. Mixed Ecosystem
**Purpose**: Heterogeneous primal composition  
**VMs**: 2 federation nodes + 1 compute node  
**Resources**:
- Federation: 3GB RAM, 2 CPUs
- Compute: 4GB RAM, 4 CPUs

```bash
cd validation
cargo run --bin provision-topology mixed-ecosystem
```

**Use Case**: Real-world scenarios, compute offloading, diverse primal interactions

---

## VM Types

### Desktop VM
- **RAM**: 2GB
- **CPUs**: 2
- **Disk**: 25GB
- **Template**: rustdesk-ubuntu-22.04
- **Use**: Development, testing, SSH access

### Server VM
- **RAM**: 1GB
- **CPUs**: 1
- **Disk**: 20GB
- **Template**: ubuntu base
- **Use**: Lightweight services

### Federation Node
- **RAM**: 3GB
- **CPUs**: 2
- **Disk**: 30GB
- **Template**: rustdesk-ubuntu-22.04
- **Use**: biomeOS + Songbird P2P

### Compute Node
- **RAM**: 4GB
- **CPUs**: 4
- **Disk**: 40GB
- **Template**: rustdesk-ubuntu-22.04
- **Use**: Toadstool compute, heavy workloads

---

## Usage

### Provision a Topology

```bash
cd validation

# Simple test (2 VMs)
cargo run --bin provision-topology simple-test

# Federation (2 nodes)
cargo run --bin provision-topology federation-2node

# Federation (3 nodes)
cargo run --bin provision-topology federation-3node

# Mixed ecosystem
cargo run --bin provision-topology mixed-ecosystem
```

### Validate Provisioned VMs

```bash
# SSH to VMs (IPs shown after provisioning)
ssh biomeos@<VM_IP>

# Test connectivity between VMs
ssh biomeos@<VM1_IP> ping -c 3 <VM2_IP>

# Check cloud-init completion
ssh biomeos@<VM_IP> cloud-init status
```

### Cleanup

```bash
# Destroy VMs (names shown after provisioning)
sudo virsh destroy vm1 vm2 vm3

# Remove VMs and storage
sudo virsh undefine vm1 vm2 vm3 --remove-all-storage
```

---

## Custom Topologies

You can create custom topologies programmatically:

```rust
use biomeos_validation::{Topology, VmConfig};

let mut topology = Topology::new("my-custom");
topology
    .add_vm(VmConfig::federation_node("node1"))
    .add_vm(VmConfig::compute_node("compute1"))
    .add_vm(VmConfig::desktop("dev1"));
```

---

## Validation Substrate Status

### ✅ Complete
- Multiple VM types
- Topology-based provisioning
- Reliable, repeatable
- Different configurations
- Type-safe Rust API

### 🚧 Next Steps
- Deploy biomeOS to provisioned VMs
- Start Songbird P2P on federation nodes
- Validate mDNS discovery
- Test inter-VM coordination

---

## Architecture

```
validation/
├── src/
│   ├── vm_types.rs           # VM configs & topologies
│   ├── lib.rs                # Shared utilities
│   └── bin/
│       ├── provision_vms.rs       # Basic 2-VM provisioning
│       ├── provision_topology.rs  # Topology-based ✅
│       └── validate_federation.rs # Phase 2 (todo)
```

**Principle**: Use benchScale as a tool (not embedded)

---

## Prerequisites

### System Requirements
```bash
sudo apt install libvirt-daemon-system libvirt-dev
sudo usermod -aG libvirt $USER
# Log out and back in
```

### agentReagents Template
See: `../AGENTREAGENTS_INTEGRATION.md`

Required: `agentReagents/images/templates/rustdesk-ubuntu-22.04-template.qcow2`

---

## Examples

### Quick Start (2 VMs)
```bash
cd validation
cargo run --bin provision-topology simple-test
# Result: 2 VMs in ~10 seconds
```

### Federation Test
```bash
cd validation
cargo run --bin provision-topology federation-2node
# Result: 2 federation nodes ready for Songbird P2P
```

### Full Ecosystem
```bash
cd validation
cargo run --bin provision-topology mixed-ecosystem
# Result: 3 VMs (2 federation + 1 compute)
```

---

**Status**: Control substrate ready ✅  
**Quality**: Production-grade 🌟  
**Next**: Deploy biomeOS and validate ecosystem 🚀

