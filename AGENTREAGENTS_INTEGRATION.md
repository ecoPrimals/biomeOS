# agentReagents Integration with biomeOS

**Date**: December 28, 2025  
**Source**: syntheticChemistry Team (ionChannel Project)  
**Purpose**: External resource inventory for biomeOS validation  

---

## Overview

**agentReagents** is a shared resources repository maintained by the syntheticChemistry subgroup. It provides VM templates, ISOs, and automation tools for validating ecoPrimals across different platforms.

### syntheticChemistry / ionChannel

**Project**: ionChannel  
**Focus**: Solving Wayland compatibility issues with RustDesk and other RDP solutions  
**Contribution**: Pre-built VM images and cloud-init templates for testing  

---

## Resources for biomeOS Validation

### 1. Cloud Images (Base Templates)

Located in: `agentReagents/images/cloud/`

**Ubuntu 22.04 LTS (Jammy)**:
- Purpose: Primary validation platform for biomeOS
- Cloud-init: Supported natively
- Size: ~700MB
- Download: Auto-fetched via setup script
- Path: `images/cloud/ubuntu-22.04-cloudimg.img`

**Ubuntu 24.04 LTS (Noble)**:
- Purpose: Future-proofing, testing newer kernels
- Wayland: Native support for ionChannel testing
- Size: ~800MB
- Path: `images/cloud/ubuntu-24.04-cloudimg.img`

**Pop!_OS 24.04 (Cosmic)**:
- Purpose: Testing with custom COSMIC desktop
- System76: Hardware-optimized for NUC deployments
- Wayland: First-class support
- Size: ~2.5GB ISO
- Path: `images/isos/pop-os_24.04_amd64_intel_nvidia.iso`

### 2. Pre-built Templates

Located in: `agentReagents/images/templates/`

These are **ready-to-deploy** VM images with:
- ✅ Cloud-init pre-configured
- ✅ SSH keys set up
- ✅ RustDesk installed (for ionChannel testing)
- ✅ Avahi/mDNS configured
- ✅ All packages pre-installed

**Templates Available**:
- `rustdesk-ubuntu-22.04-template.qcow2` - Ubuntu 22.04 + RustDesk + Wayland
- `rustdesk-ubuntu-24.04-template.qcow2` - Ubuntu 24.04 + RustDesk + COSMIC
- `biomeos-validation-template.qcow2` - Minimal template for biomeOS testing

**Advantage**: Skip 10-30 minute cloud-init package installation!

### 3. Intermediate Snapshots

Located in: `agentReagents/images/intermediates/`

Saved VM states for validation checkpoints:
- Post-cloud-init (OS ready, no apps)
- Post-avahi (mDNS configured)
- Post-rustdesk (RDP ready)
- Post-biomeos (Full primal stack)

---

## Integration with biomeOS VM Federation

### Current Flow (Without Templates)

```rust
// biomeOS creates VMs from scratch
let node = backend.create_desktop_vm(
    "my-vm",
    Path::new("/var/lib/libvirt/images/ubuntu-22.04-cloudimg.img"),
    &cloud_init,
    4096, 2, 30
).await?;

// Wait 10-30 minutes for cloud-init package installation
manager.wait_for_all_vms_ready(&[node.ip_address]).await?;
```

**Time**: 10-30 minutes (package installation)

### Optimized Flow (With agentReagents Templates)

```rust
// Use pre-built template from agentReagents
let node = backend.create_from_template(
    "my-vm",
    Path::new("../agentReagents/images/templates/biomeos-validation-template.qcow2"),
    None,  // No cloud-init needed!
    4096, 2, 30,
    false, // Don't save intermediate
).await?;

// VM is ready in seconds (CoW disk creation only)
// SSH works immediately!
```

**Time**: 30-60 seconds (copy-on-write disk only)

---

## Usage with benchScale

### Option 1: Cloud Images (Fresh Install)

```rust
use benchscale::{LibvirtBackend, CloudInit};
use std::path::Path;

let backend = LibvirtBackend::new()?;

let cloud_init = CloudInit::builder()
    .add_user("biomeos", &ssh_key)
    .package("avahi-daemon")
    .package("curl")
    .build();

// Use cloud image from agentReagents
let node = backend.create_desktop_vm(
    "biomeos-tower-alpha",
    Path::new("../agentReagents/images/cloud/ubuntu-22.04-cloudimg.img"),
    &cloud_init,
    4096, 2, 30,
).await?;
```

**Best for**: Fresh, customizable installs

### Option 2: Templates (Fast Deployment)

```rust
// Use pre-built template (much faster!)
let node = backend.create_from_template(
    "biomeos-tower-alpha",
    Path::new("../agentReagents/images/templates/biomeos-validation-template.qcow2"),
    None,
    4096, 2, 30,
    true, // Save intermediate for debugging
).await?;
```

**Best for**: Rapid testing, CI/CD validation

---

## Directory Structure

```
agentReagents/
├── images/
│   ├── cloud/              ← Base cloud images (Ubuntu, Pop!_OS)
│   ├── templates/          ← Pre-built VM templates (ready to use)
│   ├── intermediates/      ← Saved snapshots for checkpoints
│   └── backups/            ← Backup copies
├── scripts/
│   ├── setup-reagents.sh   ← Download all ISOs and images
│   ├── verify-setup.sh     ← Verify downloads complete
│   └── build-template.sh   ← Create new templates
├── tars/
│   └── packaged-templates/ ← Compressed templates for transfer
└── docs/
    └── ionChannel/         ← Wayland/RDP testing docs
```

---

## Setup Instructions

### 1. Download Resources

```bash
cd /home/eastgate/Development/ecoPrimals/primalTools/agentReagents
bash scripts/setup-reagents.sh
```

This downloads:
- Ubuntu 22.04 cloud image (~700MB)
- Ubuntu 24.04 cloud image (~800MB)
- Pop!_OS 24.04 ISO (~2.5GB)
- Pre-built templates (if available)

### 2. Verify Downloads

```bash
bash scripts/verify-setup.sh
```

Checks:
- ✅ All ISOs downloaded
- ✅ SHA256 checksums match
- ✅ Templates are accessible
- ✅ Disk space sufficient

### 3. Use with biomeOS

Update biomeOS to use agentReagents paths:

```rust
// In biomeOS validation
const BASE_IMAGE: &str = "../../../primalTools/agentReagents/images/cloud/ubuntu-22.04-cloudimg.img";

// Or use templates for speed
const TEMPLATE: &str = "../../../primalTools/agentReagents/images/templates/biomeos-validation-template.qcow2";
```

---

## ionChannel Integration

### What is ionChannel?

**Project**: RDP/Remote Desktop solution for Wayland  
**Team**: syntheticChemistry  
**Problem**: RustDesk and traditional RDP have issues with Wayland compositors  
**Solution**: Native Wayland remote desktop protocol  

### Why This Matters for biomeOS

biomeOS federations need to support:
- Remote management (SSH, RDP, VNC)
- Wayland environments (modern Linux desktops)
- Multi-tower coordination

**ionChannel** provides the RDP stack that works with Wayland, enabling:
- Remote access to biomeOS nodes
- GUI management of federated towers
- Cross-platform administration

### Testing Strategy

agentReagents provides:
1. **Ubuntu 22.04**: X11 baseline (works everywhere)
2. **Ubuntu 24.04**: Wayland testing
3. **Pop!_OS 24.04**: COSMIC desktop + Wayland (cutting edge)

biomeOS validates across all three to ensure:
- ✅ SSH access works (baseline)
- ✅ RustDesk works on X11
- ✅ ionChannel works on Wayland
- ✅ mDNS discovery works on all platforms

---

## Resource Inventory

### Disk Space Requirements

| Resource | Size | Purpose |
|----------|------|---------|
| Ubuntu 22.04 Cloud | 700MB | Base validation |
| Ubuntu 24.04 Cloud | 800MB | Future validation |
| Pop!_OS 24.04 ISO | 2.5GB | Desktop testing |
| Templates (each) | 3-5GB | Fast deployment |
| Intermediates | 2-3GB | Checkpoints |
| **Total** | **~15GB** | Full resource set |

### Download Sources

All resources fetched from official sources:
- Ubuntu: `cloud-images.ubuntu.com`
- Pop!_OS: `iso.pop-os.org`
- Templates: Pre-built by syntheticChemistry team

**Security**: SHA256 checksums verified on download

---

## Maintenance

### Updating Resources

```bash
cd agentReagents
git pull origin master
bash scripts/update-resources.sh
```

### Building Custom Templates

```bash
# Create new template from cloud image
bash scripts/build-template.sh \
    --base ubuntu-22.04 \
    --name biomeos-tower-template \
    --packages "avahi-daemon,curl,rustdesk"
```

### Sharing Templates

```bash
# Compress and upload template
bash scripts/package-template.sh biomeos-tower-template
# Upload to agentReagents for team use
```

---

## Benefits for biomeOS

### 1. Speed ⚡
- **Without templates**: 10-30 min cloud-init
- **With templates**: 30-60 sec CoW disk

### 2. Consistency ✅
- Same base image for all validation
- Reproducible test environments
- Known-good configurations

### 3. Offline Capability 📦
- All resources downloaded once
- No internet needed after setup
- Faster CI/CD pipelines

### 4. Multi-Platform 🌍
- Test on Ubuntu, Pop!_OS, future distros
- Validate Wayland and X11
- Ensure broad compatibility

---

## Status

| Item | Status |
|------|--------|
| **agentReagents cloned** | ✅ Complete |
| **ISOs downloaded** | 🔄 In progress |
| **Templates available** | ✅ Ready |
| **biomeOS integration** | 📝 Documented |
| **benchScale support** | ✅ Native |

---

## Next Steps

1. ✅ Clone agentReagents ← **DONE**
2. 🔄 Download ISOs ← **IN PROGRESS**
3. 📝 Update biomeOS validation to use templates
4. 🧪 Test federation with fast template provisioning
5. 🎯 Validate ionChannel integration for RDP access

---

**Maintained by**: syntheticChemistry Team  
**Used by**: ecoPrimals (biomeOS, benchScale)  
**Purpose**: Shared resource inventory for validation  
**License**: Courtesy of syntheticChemistry  

---

**Status**: INTEGRATED ✅  
**Resources**: DOWNLOADING 🔄  
**Ready**: For biomeOS validation 🌟  

