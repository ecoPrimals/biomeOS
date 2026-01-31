# BirdSong Cross-Platform Validation Plan
*Complete Genetic Verification & Communication*

## 🎯 Mission

Validate BirdSong beacon, genetic verification, and secure communication between USB Live Spore (x86_64) and Pixel 8a (ARM64) in TWO separate scenarios:

1. **Task 1: Local USB Handshake** (192.168.1.x network)
2. **Task 2: Web/STUN Handshake** (Public internet via STUN)

Both validations required for complete certification.

---

## 🧬 Genetic Configuration Status

### USB Live Spore (x86_64)
- **Family Seed**: `/media/eastgate/biomeOS21/biomeOS/.family.seed`
- **Hash**: `cfc8f7b15fae966d...` (32 bytes)
- **Status**: ✅ Unique genetic lineage

### Pixel 8a (ARM64)
- **Family Seed**: `/data/local/tmp/biomeos/.family.seed`
- **Hash**: `3a70ae0120bdd4f0...` (32 bytes)
- **Status**: ✅ Unique genetic lineage

### Genetic Model
- Seeds are DIFFERENT (by design - mixed lineage)
- Both derive from same ecosystem
- BearDog extracts family IDs via HKDF-SHA256
- BirdSong verifies lineage relationships

---

## 📋 Task 1: Local USB Handshake

**Network**: 192.168.1.144 ↔ 192.168.1.80 (local LAN)  
**Purpose**: Validate basic BirdSong beacon and genetic verification  
**Timeline**: 1-2 hours

### Phase 1.1: Service Startup with Genetic Context
```bash
# USB: Start BearDog with family seed
cd /media/eastgate/biomeOS21/biomeOS
~/.local/beardog/beardog server \
  --family-id usb_tower \
  --socket /tmp/beardog-usb.sock \
  2>&1 | tee /tmp/beardog-usb.log &

# Pixel: Start BearDog with family seed  
adb shell "cd /data/local/tmp && \
  /data/local/tmp/beardog/beardog server \
    --family-id pixel_tower \
    --socket /tmp/beardog-pixel.sock \
    > /tmp/beardog-pixel.log 2>&1 &"
```

### Phase 1.2: Start Songbird Discovery
```bash
# USB: Start Songbird with BearDog security
~/.local/songbird/songbird server \
  --port 8080 \
  2>&1 | tee /tmp/songbird-usb.log &

# Pixel: Start Songbird
adb shell "cd /data/local/tmp && \
  /data/local/tmp/songbird/songbird server \
    --port 8080 \
    > /tmp/songbird-pixel.log 2>&1 &"
```

### Phase 1.3: Verify BirdSong Beacons
```bash
# Check USB beacon
tail -f /tmp/songbird-usb.log | grep -E "(beacon|discovery|family|lineage)"

# Check Pixel beacon  
adb shell "tail -f /tmp/songbird-pixel.log" | grep -E "(beacon|discovery|family|lineage)"
```

### Phase 1.4: Monitor Genetic Verification
```bash
# Watch for genetic lineage verification
tail -f /tmp/beardog-usb.log | grep -E "(genetic|lineage|family_id|birdsong)"

adb shell "tail -f /tmp/beardog-pixel.log" | grep -E "(genetic|lineage|family_id|birdsong)"
```

### Phase 1.5: Test Communication
```bash
# Once discovered, test encrypted channel
# BearDog will establish BirdSong-encrypted tunnel
# Songbird will coordinate service registration
```

### Success Criteria (Task 1)
- ✅ Both BearDog instances derive family IDs from seeds
- ✅ Songbird beacons broadcast on local network
- ✅ Services discover each other via mDNS
- ✅ Genetic lineage verified (both from same ecosystem)
- ✅ BirdSong encrypted channel established
- ✅ Test message exchanged successfully

---

## 📋 Task 2: Web/STUN Handshake

**Network**: Public internet via STUN (stun.l.google.com)  
**Purpose**: Validate NAT traversal and internet-scale federation  
**Timeline**: 2-3 hours

### Phase 2.1: STUN Configuration
```bash
# USB: Configure STUN discovery
cat > /tmp/usb-stun-config.toml << EOF
[network]
stun_servers = [
    "stun.l.google.com:19302",
    "stun1.l.google.com:19302"
]
local_ip = "192.168.1.144"
enable_upnp = true

[discovery]
mode = "stun"
service_name = "tower-usb"
announce_interval = 5

[beardog]
family_id = "usb_tower"
enable_stun = true

[songbird]
enable_mdns = true
enable_stun = true
EOF

# Pixel: Configure STUN discovery
adb push /tmp/usb-stun-config.toml /data/local/tmp/pixel-stun-config.toml

# Edit for Pixel
adb shell "sed -i 's/192.168.1.144/192.168.1.80/g; s/tower-usb/tower-pixel/g; s/usb_tower/pixel_tower/g' /data/local/tmp/pixel-stun-config.toml"
```

### Phase 2.2: Start Services with STUN
```bash
# USB: Start with STUN config
~/.local/beardog/beardog server \
  --config /tmp/usb-stun-config.toml \
  2>&1 | tee /tmp/beardog-usb-stun.log &

~/.local/songbird/songbird server \
  --config /tmp/usb-stun-config.toml \
  2>&1 | tee /tmp/songbird-usb-stun.log &

# Pixel: Start with STUN config
adb shell "/data/local/tmp/beardog/beardog server \
  --config /data/local/tmp/pixel-stun-config.toml \
  > /tmp/beardog-pixel-stun.log 2>&1 &"

adb shell "/data/local/tmp/songbird/songbird server \
  --config /data/local/tmp/pixel-stun-config.toml \
  > /tmp/songbird-pixel-stun.log 2>&1 &"
```

### Phase 2.3: Monitor STUN Discovery
```bash
# Watch STUN endpoint discovery
tail -f /tmp/songbird-usb-stun.log | grep -E "(stun|public|endpoint|nat)"
adb shell "tail -f /tmp/songbird-pixel-stun.log" | grep -E "(stun|public|endpoint|nat)"
```

### Phase 2.4: Verify NAT Traversal
```bash
# Check if services discovered public endpoints
# Verify hole punching successful
# Monitor connection establishment via STUN
```

### Phase 2.5: Test Internet-Scale Communication
```bash
# Test encrypted communication via public endpoints
# Verify genetic lineage over internet
# Validate BirdSong encryption with NAT traversal
```

### Success Criteria (Task 2)
- ✅ STUN servers contacted successfully
- ✅ Public endpoints discovered for both platforms
- ✅ NAT traversal successful (hole punching)
- ✅ Services discover each other via STUN
- ✅ Genetic lineage verified over internet
- ✅ BirdSong encrypted channel via public IPs
- ✅ Test message exchanged over internet

---

## 🔍 Validation Checklist

### BirdSong Beacon Validation
- [ ] USB broadcasts beacon with genetic lineage
- [ ] Pixel broadcasts beacon with genetic lineage
- [ ] Beacons contain family ID information
- [ ] Beacon intervals consistent
- [ ] Both platforms receive beacons

### Genetic Verification Validation
- [ ] USB BearDog derives family ID from seed
- [ ] Pixel BearDog derives family ID from seed
- [ ] Family IDs are unique but related
- [ ] Lineage verification algorithm executes
- [ ] Genetic relationship confirmed
- [ ] Trust level established based on lineage

### Communication Validation
- [ ] BirdSong encrypted channel established
- [ ] ChaCha20-Poly1305 encryption active
- [ ] Ed25519 signatures verified
- [ ] Test messages sent successfully
- [ ] Latency acceptable (<100ms local, <500ms web)
- [ ] No packet loss

### Local Handshake Specific
- [ ] mDNS discovery working
- [ ] Local network routing correct
- [ ] Unix sockets (USB) and abstract sockets (Pixel) working
- [ ] Direct IP connection successful

### Web/STUN Handshake Specific
- [ ] STUN binding successful
- [ ] Public endpoints discovered
- [ ] NAT type detected correctly
- [ ] Hole punching successful
- [ ] Connection via public IPs established
- [ ] Works across different networks

---

## 📊 Testing Matrix

| Test | Local | STUN | Status |
|------|-------|------|--------|
| Service Startup | Required | Required | ⏳ |
| Family ID Derivation | Required | Required | ⏳ |
| Beacon Broadcast | Required | Required | ⏳ |
| Discovery | mDNS | STUN | ⏳ |
| Genetic Verification | Required | Required | ⏳ |
| Encryption Setup | Required | Required | ⏳ |
| Message Exchange | Required | Required | ⏳ |
| Connection Type | Direct | NAT Traversal | ⏳ |

---

## 🛠️ Troubleshooting Guide

### Issue: Services Don't Start
**Check**:
- Family seed files exist and are readable
- Required ports available (8080, 9000, 9001)
- Binaries have execute permissions
- Logs for specific error messages

### Issue: No Discovery
**Check**:
- mDNS daemon running (avahi-daemon on Linux)
- Firewall allowing UDP multicast (mDNS)
- STUN servers reachable
- Network connectivity between platforms

### Issue: Genetic Verification Fails
**Check**:
- Family seeds are correct format (32 bytes)
- BearDog successfully extracted family IDs
- Lineage algorithm implementation
- Logs for verification errors

### Issue: Encryption Fails
**Check**:
- BirdSong genetic engine initialized
- Key derivation from family seeds working
- ChaCha20-Poly1305 library loaded
- Certificate/key material available

---

## 📈 Expected Timeline

### Task 1: Local USB Handshake
- Setup: 15 minutes
- Service startup: 10 minutes  
- Discovery: 5 minutes
- Verification: 10 minutes
- Communication: 10 minutes
- **Total: ~1 hour**

### Task 2: Web/STUN Handshake
- STUN configuration: 20 minutes
- Service startup: 10 minutes
- STUN discovery: 15 minutes
- NAT traversal: 20 minutes
- Verification: 15 minutes
- Communication: 15 minutes
- **Total: ~2 hours**

### Complete Validation
- Task 1: 1 hour
- Task 2: 2 hours
- Documentation: 30 minutes
- **Total: ~3.5 hours**

---

## 🎯 Success Definition

### Complete Success = Both Tasks Pass

**Task 1 Success**: Local handshake with genetic verification ✅  
**Task 2 Success**: Web handshake with NAT traversal ✅

**Result**: BirdSong cross-platform federation **VALIDATED** ✅

---

## 📝 Logging & Observability

### Log Files
```
USB:
  /tmp/beardog-usb.log          # Local test
  /tmp/songbird-usb.log         # Local test
  /tmp/beardog-usb-stun.log     # STUN test
  /tmp/songbird-usb-stun.log    # STUN test

Pixel:
  /tmp/beardog-pixel.log        # Local test
  /tmp/songbird-pixel.log       # Local test
  /tmp/beardog-pixel-stun.log   # STUN test
  /tmp/songbird-pixel-stun.log  # STUN test
```

### Key Log Patterns
```bash
# Family ID derivation
grep "family_id" /tmp/beardog-*.log

# Beacon broadcasting
grep "beacon" /tmp/songbird-*.log

# Discovery events
grep "discovered" /tmp/songbird-*.log

# Genetic verification
grep -E "(genetic|lineage|verify)" /tmp/beardog-*.log

# Encryption establishment
grep -E "(birdsong|encryption|chacha20)" /tmp/beardog-*.log
```

---

## 🚀 Execution Scripts

Created automation scripts:
- `birdsong_local_handshake.sh` - Task 1 automation
- `birdsong_stun_handshake.sh` - Task 2 automation
- `birdsong_monitor.sh` - Real-time monitoring
- `birdsong_verify.sh` - Post-test verification

---

*Plan Created: January 31, 2026*  
*Validation Type: Dual-path (Local + Web)*  
*Status: Ready for Execution*  
*Expected Duration: 3.5 hours*
