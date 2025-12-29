# biomeOS Quick Start

**Get running in 5 minutes!**

---

## Option 1: Local Development

```bash
# Clone and build
git clone git@github.com:ecoPrimals/biomeOS.git
cd biomeOS
cargo build --release

# Run biomeOS
cargo run --release

# Run tests
cargo test --workspace
```

---

## Option 2: VM Testing (Phase 1 Validated ✅)

```bash
# Requires: libvirt, benchScale, agentReagents template

# Create 2 VMs for testing
sudo cargo run --release --bin test-vm-provisioning

# Result: 2 VMs running in 5 seconds!
# - VM1: biomeos-test-vm1 (IP assigned)
# - VM2: biomeos-test-vm2 (IP assigned)
```

**Prerequisites**:
- libvirt installed: `sudo apt install libvirt-daemon-system libvirt-dev`
- User in libvirt group: `sudo usermod -aG libvirt $USER`
- agentReagents template: See `AGENTREAGENTS_INTEGRATION.md`

---

## Option 3: NUC USB Deployment

```bash
# Create bootable USB
./quick-usb.sh

# Boot NUC from USB
# Songbird P2P auto-starts
# Ready to federate!
```

See: `NUC_USB_DEPLOYMENT_GUIDE.md`

---

## Validation

```bash
# Run all tests
cargo test --workspace

# Run E2E tests
./run-e2e-tests.sh

# Run showcases
cd showcase/00-substrate/01-hello-biomeos
./demo.sh
```

---

## Next Steps

1. **Read**: [STATUS.md](STATUS.md) - Current status
2. **Learn**: [ROOT_INDEX.md](ROOT_INDEX.md) - All docs
3. **Deploy**: Phase 2 ready to execute!

---

**Status**: Phase 1 Complete ✅  
**Ready**: Phase 2 (biomeOS deployment to VMs) 🚀
