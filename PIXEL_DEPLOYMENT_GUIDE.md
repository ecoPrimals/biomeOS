# Pixel 8a NUCLEUS Deployment Guide

**Date**: January 16, 2026  
**Device**: Pixel 8a + GrapheneOS  
**Architecture**: ARM64 (aarch64-linux-android)  
**Security**: Titan M2 Hardware Security Module  
**Status**: 🟢 Ready for Deployment

---

## 🎯 Deployment Strategy

### Option A: Cross-Compile + ADB Deploy (Recommended - Quick Start)

**Advantages**:
- Fastest deployment (no on-device compilation)
- Test hardware HSM immediately
- Minimal phone setup
- Easy iteration

**Steps**:
1. Cross-compile NUCLEUS binaries for Android ARM64
2. Push binaries to Pixel via ADB
3. Set up minimal Termux environment
4. Deploy BearDog HSM (hardware-backed)
5. Test ionic bonding from local families

**Timeline**: 30-60 minutes

---

### Option B: Native Build on Pixel (Full Capability)

**Advantages**:
- Full development environment on device
- Native performance
- Self-contained deployment
- Complete NUCLEUS capabilities

**Steps**:
1. Install Termux + proot-distro on Pixel
2. Install Rust toolchain in Termux
3. Clone biomeOS repository
4. Build all binaries natively
5. Deploy full NUCLEUS

**Timeline**: 2-3 hours (includes compilation time)

---

## 🚀 Option A: Cross-Compile Deployment (Quick Start)

### Step 1: Prepare Cross-Compilation Environment

```bash
# Verify aarch64-linux-android target is installed
rustup target list | grep aarch64-linux-android

# If not installed:
rustup target add aarch64-linux-android

# Set up Android NDK (if not already)
# GrapheneOS/Termux uses standard Android NDK
```

### Step 2: Cross-Compile Primal Binaries

```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS

# Cross-compile BearDog for Android ARM64
cargo build --release \
  --target aarch64-linux-android \
  --bin beardog-server

# Optionally compile other primals
cargo build --release \
  --target aarch64-linux-android \
  --bin neural-api-server

cargo build --release \
  --target aarch64-linux-android \
  --bin songbird-orchestrator

# Check compiled binaries
ls -lh target/aarch64-linux-android/release/beardog-server
```

### Step 3: Prepare Pixel 8a

```bash
# Enable USB Debugging on Pixel:
# Settings → About Phone → tap Build Number 7 times
# Settings → System → Developer Options → USB Debugging → ON

# Connect Pixel via USB and verify ADB connection
adb devices

# Expected output:
# List of devices attached
# <device-id>    device

# Install Termux on Pixel (via F-Droid or direct APK)
# https://f-droid.org/en/packages/com.termux/

# Grant Termux storage access
adb shell am start -a android.intent.action.VIEW \
  -d "package:com.termux"
# Then in Termux: termux-setup-storage
```

### Step 4: Push Binaries to Pixel

```bash
# Create deployment directory on Pixel
adb shell mkdir -p /data/local/tmp/biomeOS/primals
adb shell mkdir -p /data/local/tmp/biomeOS/graphs

# Push BearDog binary
adb push target/aarch64-linux-android/release/beardog-server \
  /data/local/tmp/biomeOS/primals/

# Push graphs (optional, for Neural API)
adb push graphs/01_nucleus_enclave.toml \
  /data/local/tmp/biomeOS/graphs/

# Make binaries executable
adb shell chmod +x /data/local/tmp/biomeOS/primals/beardog-server

# Verify
adb shell ls -lh /data/local/tmp/biomeOS/primals/
```

### Step 5: Deploy BearDog HSM on Pixel

```bash
# Open ADB shell
adb shell

# Navigate to biomeOS directory
cd /data/local/tmp/biomeOS

# Set environment variables
export FAMILY_ID=nat0
export NODE_ID=pixel_hsm
export BEARDOG_HSM_MODE=hardware
export BEARDOG_SOCKET=/data/local/tmp/beardog-pixel.sock

# Launch BearDog (hardware-backed HSM mode)
./primals/beardog-server &

# Verify socket created
ls -l /data/local/tmp/beardog-pixel.sock

# Test BearDog capabilities
# (In another terminal/adb shell)
# curl --unix-socket /data/local/tmp/beardog-pixel.sock \
#   -d '{"jsonrpc":"2.0","method":"beardog.capabilities","id":1}'
```

### Step 6: Configure Local Families to Use Pixel HSM

**On Local Computer**:

Update NUCLEUS graphs to use Pixel as security provider:

```toml
# In graphs/bonding-test-*.toml or new graph

[[nodes]]
id = "launch_nestgate"
[nodes.config]
primal_name = "nestgate"
family_id = "family_alpha"

# Use Pixel BearDog as security provider (ionic bonding!)
security_provider = "http://192.168.1.XXX:8080"  # Pixel IP
security_provider_type = "ionic"  # Contract-based, no electron sharing
hardware_backed = true
```

**Enable Network Access on Pixel BearDog**:

```bash
# On Pixel (via ADB shell)
# Restart BearDog with network listener

export BEARDOG_LISTEN_ADDRESS=0.0.0.0:8080  # Listen on WiFi
export BEARDOG_ALLOW_NETWORK=true
./primals/beardog-server &

# Verify listening
netstat -an | grep 8080
```

### Step 7: Test Ionic Bonding (Local → Pixel HSM)

```bash
# On local computer

# Update Family Alpha to request JWT from Pixel
./plasmidBin/primals/neural-deploy \
  bonding-test-pixel-hsm \
  --family-id test_bonding

# Expected behavior:
# - Family Alpha primals detect Pixel security provider
# - Ionic bonding: contract-based API request
# - Pixel BearDog generates JWT (hardware-backed)
# - No electron sharing (each maintains own Tower)
# - Hardware security for local NUCLEUS!
```

---

## 🔐 Hardware HSM Features (Titan M2)

### BearDog HSM Mode Capabilities

**Hardware-Backed Operations**:
- JWT secret generation (hardware-backed random)
- Key derivation (secure enclave)
- Encryption/decryption (hardware accelerated)
- Biometric-protected operations

**Ionic Bonding Pattern**:
```
Local Family Alpha (Covalent Internal)
  ├─> Ionic → Pixel BearDog HSM
  │   • Contract: JWT generation
  │   • Hardware-backed security
  │   • No electron sharing
  │   • Electrostatic interaction (JSON-RPC)
  └─> Maintains own Tower (Songbird)
      • Independent BirdSong mesh
      • Separate from Pixel
```

### Android Keystore Integration

**Configuration** (when supported):
```toml
[beardog.hsm]
hardware_backend = "android_keystore"
require_strongbox = true  # Use Titan M2 if available
biometric_auth = true     # Require fingerprint/face unlock
key_attestation = true    # Verify hardware-backed keys
```

---

## 📱 Full NUCLEUS Deployment (Option B)

### Step 1: Set Up Termux Environment

```bash
# On Pixel (in Termux app)
pkg update && pkg upgrade
pkg install rust binutils proot-distro git

# Optional: Use Ubuntu proot for full Linux environment
proot-distro install ubuntu
proot-distro login ubuntu
```

### Step 2: Clone and Build biomeOS

```bash
# In Termux/proot
git clone https://github.com/your-org/biomeOS
cd biomeOS

# Build all primals (this will take time on mobile)
cargo build --release --workspace

# Verify binaries
ls -lh target/release/beardog-server
ls -lh target/release/songbird-orchestrator
ls -lh target/release/toadstool
ls -lh target/release/nestgate
```

### Step 3: Deploy NUCLEUS

```bash
# Deploy via Neural API
./target/release/neural-api-server \
  --graphs-dir graphs \
  --family-id pixel_nucleus &

sleep 3

./target/release/neural-deploy \
  01_nucleus_enclave \
  --family-id pixel_nucleus

# Verify deployment
ps aux | grep -E "(beardog|songbird|toadstool|nestgate)"
ls -l /data/local/tmp/*.sock
```

---

## 🧪 Bonding Model Tests with Pixel

### Test 1: Ionic Bonding (Local Families → Pixel HSM)

**Scenario**: Use Pixel as external security provider

**Bonding Type**: Ionic
- Local families maintain own electrons
- Pixel BearDog provides security service
- Contract-based interaction (JSON-RPC)
- Hardware-backed security

**Expected Behavior**:
- ✅ JWT secrets hardware-generated on Pixel
- ✅ Biometric authentication for sensitive ops
- ✅ No electron sharing between local and Pixel
- ✅ Each maintains independent BirdSong mesh

### Test 2: Covalent Bonding (Pixel joins Local Family)

**Scenario**: Pixel NUCLEUS joins family_alpha temporarily

**Bonding Type**: Covalent
- Share family_seed with Pixel
- Pixel joins BirdSong mesh
- Electron sharing (Songbird coordination)
- Collaborative compute/storage

**Expected Behavior**:
- ✅ Pixel discovers local primals via BirdSong
- ✅ Shared molecular orbital (Tower mesh)
- ✅ Collaborative resource pooling
- ✅ Mobile primal in local ecosystem

### Test 3: Weak Forces (Pixel observes Unknown Network)

**Scenario**: Pixel passively scans public WiFi

**Bonding Type**: Weak (Dipole-Dipole, Brownian)
- Zero trust, no authentication
- Passive observation only
- No electron involvement
- Minimal disclosure

**Expected Behavior**:
- ✅ Network discovery (no disruption)
- ✅ Read-only observations
- ✅ Zero information leakage
- ✅ No fingerprinting of Pixel

---

## 🔧 Troubleshooting

### ADB Connection Issues

```bash
# Verify USB connection
lsusb | grep Google

# Restart ADB server
adb kill-server
adb start-server
adb devices

# If "unauthorized", accept prompt on Pixel screen
```

### Cross-Compilation Errors

```bash
# Ensure Android NDK is configured
export ANDROID_NDK_HOME=/path/to/ndk

# Or use cargo-ndk wrapper
cargo install cargo-ndk
cargo ndk --target aarch64-linux-android \
  --platform 30 \
  build --release
```

### Termux Permission Issues

```bash
# Grant storage permissions
termux-setup-storage

# If needed, use /data/local/tmp instead
# (accessible without storage permissions)
```

### Network Listener Not Accessible

```bash
# Check Pixel IP address
# In Termux:
ip addr show wlan0 | grep inet

# Verify firewall/network
ping <pixel-ip>

# Test from local computer
curl http://<pixel-ip>:8080/health
```

---

## 📊 Performance Expectations

### Pixel 8a Specs (Tensor G3)
- **CPU**: Octa-core ARM64 (up to 2.91 GHz)
- **RAM**: 8GB
- **Storage**: 128GB+ (UFS 3.1)
- **Security**: Titan M2 hardware security chip

### Expected Performance

**BearDog HSM**:
- JWT generation: <10ms (hardware-backed)
- Key derivation: <5ms
- Encryption: ~50 MB/s (hardware accelerated)

**Full NUCLEUS**:
- Primal startup: 1-3 seconds each
- Socket creation: <100ms
- BirdSong discovery: 2-5 seconds
- Graph orchestration: Similar to desktop

**Compilation Time** (on device):
- BearDog: ~10 minutes
- Full workspace: ~30-45 minutes

---

## 🎯 Success Criteria

### Option A (HSM Only)
- [x] aarch64-linux-android target installed
- [x] ADB connection working
- [ ] BearDog cross-compiled for Android
- [ ] BearDog deployed on Pixel
- [ ] Hardware HSM operational
- [ ] Local families using Pixel for JWT
- [ ] Ionic bonding validated

### Option B (Full NUCLEUS)
- [ ] Termux installed on Pixel
- [ ] Rust toolchain in Termux
- [ ] biomeOS cloned and built
- [ ] NUCLEUS deployed on Pixel
- [ ] Covalent mesh with local families
- [ ] Mobile primal capabilities validated

---

## 🚀 Next Steps

**Immediate** (This Session):
1. Cross-compile BearDog for Android
2. Push to Pixel via ADB
3. Launch BearDog HSM on Pixel
4. Test hardware-backed JWT generation
5. Configure local family to use Pixel HSM
6. Validate ionic bonding

**Short-term** (Next Session):
1. Full NUCLEUS deployment on Pixel
2. Covalent mesh with local families
3. Mobile mesh participant testing
4. Performance benchmarks

**Long-term** (Future):
1. Native Android app with UI
2. Android Keystore integration
3. Background service mode
4. Battery optimization

---

**Status**: 🟢 Ready for Cross-Compilation  
**Toolchain**: ✅ aarch64-linux-android installed  
**ADB**: ✅ Available and ready  
**Device**: Pixel 8a + GrapheneOS + Titan M2  
**Next**: Cross-compile BearDog for Android! 🚀📱

