# biomeOS Validation Tools

**Using benchScale v2.0.0 for VM provisioning and validation**

---

## Philosophy

> "A carpenter, a mechanic, and a silversmith might all have a hammer"

**benchScale** is a tool we use for validation. It exists independently in `primalTools/`, but we utilize it here without becoming chimeras. We're using the tool properly, not embedding it.

---

## Structure

```
validation/
├── Cargo.toml           # Validation workspace
├── README.md            # This file
└── src/
    ├── lib.rs           # Shared validation utilities
    └── bin/
        ├── provision_vms.rs       # VM provisioning using benchScale v2.0.0
        └── validate_federation.rs # Full federation validation
```

---

## Usage

### Provision VMs (Phase 1)

```bash
cd validation
cargo run --bin provision-vms
```

**Uses**: `benchscale::LibvirtBackend::create_desktop_vm_ready()`
- Guaranteed SSH-ready VMs
- Cloud-init validation built-in
- No workarounds needed

### Validate Federation (Phase 2)

```bash
cd validation
cargo run --bin validate-federation
```

**Validates**:
1. VM provisioning
2. biomeOS deployment
3. Songbird P2P startup
4. mDNS/UDP discovery
5. Federation coordination

---

## benchScale v2.0.0 Integration

We use the **new API** that addresses our evolution gaps:

### Old Way (v1.x - what we used in Phase 1)
```rust
let vm = backend.create_desktop_vm(...).await?;
// Hope cloud-init finishes... ⏳
// Manual SSH validation needed
```

### New Way (v2.0.0 - proper!)
```rust
let vm = backend.create_desktop_vm_ready(...).await?;
// Guaranteed SSH-ready! ✅
// Framework validates everything
```

**Benefits**:
- No timing assumptions
- Framework-level validation
- Clear error messages
- Proper tool usage

---

## Why validation/ ?

**Separation of Concerns**:
- `src/` - biomeOS core (substrate, orchestration)
- `validation/` - Validation tools (using benchScale)
- `primalTools/` - Shared tools (benchScale, agentReagents)

This keeps:
- biomeOS core clean
- Validation explicit
- Tools reusable
- No chimeras

---

## Prerequisites

### System
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

### Quick VM Test
```bash
cd validation
cargo run --bin provision-vms
# Creates 2 VMs in ~5 seconds
# Both SSH-ready ✅
```

### Full Federation
```bash
cd validation
cargo run --bin validate-federation
# Complete Phase 1 + 2 validation
# Songbird P2P federation tested
```

---

## Status

- ✅ benchScale v2.0.0 integrated
- ✅ Cloud-init validation API used
- ✅ Old workarounds removed
- ✅ Proper tool usage
- ✅ Clean architecture

**Quality**: Production-ready 🌟

