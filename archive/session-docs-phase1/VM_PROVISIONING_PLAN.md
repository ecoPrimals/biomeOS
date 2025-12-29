# VM Provisioning Plan - Phase 1

**Goal**: Get 2 VMs running using benchScale + agentReagents

---

## Problem

benchScale CLI is trying to use Docker by default, not libvirt.

## Solution Options

### Option A: Use benchScale's Rust API directly ✅ RECOMMENDED
```rust
use benchscale::{LibvirtBackend, CloudInit};

let backend = LibvirtBackend::new()?;
let cloud_init = CloudInit::builder()
    .add_user("biomeos", "~/.ssh/id_rsa.pub")
    .build();

let vm = backend.create_desktop_vm(
    "test-vm1",
    Path::new("/var/lib/libvirt/images/rustdesk-ubuntu-22.04-template.qcow2"),
    &cloud_init,
    2048,  // RAM
    2,      // CPUs
    10,     // Disk GB
).await?;
```

**Benefits**:
- Type-safe
- Direct control
- No CLI parsing
- Better error handling

### Option B: Fix benchScale CLI to use libvirt backend
- Need to check if there's a `--backend libvirt` flag
- Or if it needs configuration

### Option C: Use virt-install directly (temporary)
- Quick and dirty
- Not using benchScale
- Just to validate the substrate works

---

## Recommended Approach

**Use benchScale's Rust API** (Option A):

1. Add benchScale as dependency to biomeOS
2. Use `LibvirtBackend` directly
3. Create VMs programmatically
4. Validate they work
5. Then move to Phase 2 (biomeOS deployment)

---

## Simple Test Plan

```bash
# Step 1: Create test binary
cargo new --bin test-vm-provisioning

# Step 2: Add benchScale dependency
# Add to Cargo.toml: benchscale = { path = "../../primalTools/benchscale", features = ["libvirt"] }

# Step 3: Run test
cargo run --bin test-vm-provisioning

# Step 4: Validate
# - VMs created
# - VMs boot
# - VMs get IPs
# - SSH works
```

---

## Success Criteria

✅ 2 VMs created using benchScale LibvirtBackend  
✅ VMs boot successfully  
✅ VMs get DHCP IPs  
✅ SSH access works  
✅ VMs can ping each other  

**Then**: Move to Phase 2 (biomeOS USB deployment)

---

*Keep it simple, validate the substrate!* 🔧
