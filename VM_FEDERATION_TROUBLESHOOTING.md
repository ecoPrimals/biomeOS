# VM Federation Troubleshooting Guide

**Issue**: VMs created but SSH keys not provisioning via cloud-init  
**Date**: December 28, 2025  
**VMs**: tower-alpha (192.168.122.34), tower-beta (192.168.122.201)  

---

## Problem

Cloud-init is taking longer than expected (~40+ minutes) to provision SSH keys.

**Symptoms**:
- VMs are running (verified via `virsh list`)
- IPs acquired (192.168.122.34, 192.168.122.201)
- SSH connection refused with "Permission denied (publickey)"

**Root Cause**: Our cloud-init user-data might not have SSH keys properly injected.

---

## Quick Fix Options

### Option 1: Manual SSH Key Injection (Fastest)

```bash
# Stop VMs
sudo virsh shutdown biomeos-tower-alpha
sudo virsh shutdown biomeos-tower-beta

# Wait for shutdown
sleep 10

# Inject SSH keys manually
sudo virt-customize -d biomeos-tower-alpha \
  --ssh-inject biomeos:file:/home/eastgate/.ssh/id_rsa.pub \
  --run-command 'systemctl enable ssh'

sudo virt-customize -d biomeos-tower-beta \
  --ssh-inject biomeos:file:/home/eastgate/.ssh/id_rsa.pub \
  --run-command 'systemctl enable ssh'

# Restart VMs
sudo virsh start biomeos-tower-alpha
sudo virsh start biomeos-tower-beta

# Wait for boot
sleep 30

# Test SSH
ssh biomeos@192.168.122.34
```

### Option 2: Console Access & Manual Fix

```bash
# Access VM console
sudo virsh console biomeos-tower-alpha

# Login with cloud-init default
# User: ubuntu, Password: ubuntu (or check cloud-init logs)

# Once logged in:
sudo mkdir -p /home/biomeos/.ssh
sudo cat >> /home/biomeos/.ssh/authorized_keys << EOF
<paste your ~/.ssh/id_rsa.pub content here>
EOF
sudo chown -R biomeos:biomeos /home/biomeos/.ssh
sudo chmod 700 /home/biomeos/.ssh
sudo chmod 600 /home/biomeos/.ssh/authorized_keys

# Exit console: Ctrl+]
# Try SSH again
ssh biomeos@192.168.122.34
```

### Option 3: Fresh Start (Most Reliable)

```bash
# Destroy current VMs
sudo virsh destroy biomeos-tower-alpha
sudo virsh undefine biomeos-tower-alpha --remove-all-storage
sudo virsh destroy biomeos-tower-beta
sudo virsh undefine biomeos-tower-beta --remove-all-storage

# Fix the cloud-init script in validate-usb-federation.sh
# Ensure SSH key is explicitly in authorized_keys format

# Re-run with explicit SSH key
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
sudo ./validate-usb-federation.sh
```

---

## Root Cause Analysis

The `validate-usb-federation.sh` script creates cloud-init config but may have issues:

**Potential Issues**:
1. SSH key format in cloud-init YAML
2. Cloud-init timing (slow package installation)
3. Missing `ssh_pwauth: true` fallback
4. User creation race condition

**Our Cloud-Init**:
```yaml
users:
  - name: biomeos
    sudo: ALL=(ALL) NOPASSWD:ALL
    shell: /bin/bash
    ssh_authorized_keys:
      - <KEY_HERE>  # ← May need verification
```

---

## Verification Steps

After fix:

```bash
# 1. Check SSH works
ssh biomeos@192.168.122.34 "hostname && uptime"

# 2. Check cloud-init completed
ssh biomeos@192.168.122.34 "cloud-init status"

# 3. Check avahi-daemon running
ssh biomeos@192.168.122.34 "systemctl status avahi-daemon"

# 4. Deploy BiomeOS
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
scp biomeos-20251228-181049.tar.gz biomeos@192.168.122.34:/tmp/

# 5. Extract and start
ssh biomeos@192.168.122.34 "
  cd /tmp
  tar -xzf biomeos-20251228-181049.tar.gz
  sudo mv opt/biomeos /opt/
  sudo chown -R biomeos:biomeos /opt/biomeos
  cd /opt/biomeos
  ./start-songbird.sh
"
```

---

## Prevention (Next Time)

Update `validate-usb-federation.sh` to:

1. **Add password fallback**:
```yaml
ssh_pwauth: true
password: <random-password>
chpasswd:
  expire: false
```

2. **Verify SSH key format**:
```bash
# In script, validate before injecting
if [ ! -f ~/.ssh/id_rsa.pub ]; then
    echo "ERROR: SSH key not found"
    exit 1
fi
```

3. **Add timeout & retry**:
```bash
# Wait with exponential backoff
for i in {1..10}; do
    if ssh -o ConnectTimeout=5 biomeos@$VM_IP "exit"; then
        break
    fi
    echo "Attempt $i/10 failed, waiting..."
    sleep $((i * 10))
done
```

---

## Current Session Handoff

**Status**: VMs running, SSH pending manual fix  
**Recommendation**: Use Option 1 (virt-customize) - fastest  
**Time**: ~5 minutes to fix  

**All other work complete**:
- ✅ 68 commits pushed
- ✅ Modernization done
- ✅ USB package ready
- ✅ Documentation complete

**Next**: Fix SSH → Deploy BiomeOS → Test Federation → Deploy to NUC!

---

**Created**: December 28, 2025  
**For**: Next session continuation  

