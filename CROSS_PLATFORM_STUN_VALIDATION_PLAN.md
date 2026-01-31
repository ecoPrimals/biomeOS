# Cross-Platform STUN Validation Plan
*Live Spore USB (x86_64) ↔ Pixel 8a (ARM64) Rendezvous*

## 🎯 Objective

Validate complete genomeBin ecosystem with real-world cross-platform federation using public STUN servers for NAT traversal and discovery.

---

## 🌐 Network Topology

### Linux Host (Live Spore USB)
- **IP**: 192.168.1.144 (eno1) / 192.168.1.242 (wlp0s20f3)
- **Architecture**: x86_64
- **Platform**: Ubuntu 24.04
- **USB Mount**: `/media/eastgate/biomeOS21/biomeOS`

### Android Device (Pixel 8a)
- **IP**: 192.168.1.80 (wlan0)
- **Architecture**: ARM64 (aarch64)
- **Platform**: Android 14 (GrapheneOS)
- **Deployment**: `/data/local/tmp/`

### Public STUN Servers
- `stun.l.google.com:19302`
- `stun1.l.google.com:19302`
- `stun2.l.google.com:19302`

---

## 📋 Validation Phases

### Phase 1: Deploy TOWER on Live Spore USB (5 min)

**Objective**: Deploy BearDog + Songbird to x86_64 USB environment

**Commands**:
```bash
# Copy genomeBins to USB
cp ~/Development/ecoPrimals/phase2/biomeOS/plasmidBin/stable/beardog.genome \
   /media/eastgate/biomeOS21/biomeOS/

cp ~/Development/ecoPrimals/phase2/biomeOS/plasmidBin/stable/songbird.genome \
   /media/eastgate/biomeOS21/biomeOS/

# Deploy TOWER
cd /media/eastgate/biomeOS21/biomeOS/
./beardog.genome
./songbird.genome
```

**Expected**:
- BearDog installed to `/opt/beardog` or `~/.local/beardog`
- Songbird installed to `/opt/songbird` or `~/.local/songbird`
- Both binaries executable and health checks passing

---

### Phase 2: Verify Pixel 8a Deployment (2 min)

**Objective**: Confirm TOWER already deployed on Android

**Commands**:
```bash
# Verify deployment
adb shell "/data/local/tmp/beardog/beardog --version"
adb shell "/data/local/tmp/songbird/songbird --version"
```

**Expected**:
- BearDog 0.9.0 ✅
- Songbird 0.1.0 ✅

---

### Phase 3: Configure STUN Discovery (10 min)

**Objective**: Configure both platforms to use public STUN for NAT traversal

#### Linux Configuration

**Create STUN config**: `/tmp/tower_stun_config.toml`
```toml
[network]
stun_servers = [
    "stun.l.google.com:19302",
    "stun1.l.google.com:19302"
]
local_ip = "192.168.1.144"
enable_upnp = true
enable_nat_pmp = true

[discovery]
mode = "stun"
service_name = "tower-usb"
announce_interval = 5

[beardog]
bind_address = "0.0.0.0:9000"
enable_stun = true

[songbird]
bind_address = "0.0.0.0:9001"
enable_mdns = true
enable_stun = true
```

#### Android Configuration

**Push STUN config to Pixel**:
```bash
cat > /tmp/tower_pixel_config.toml << 'EOF'
[network]
stun_servers = [
    "stun.l.google.com:19302",
    "stun1.l.google.com:19302"
]
local_ip = "192.168.1.80"
enable_upnp = false

[discovery]
mode = "stun"
service_name = "tower-pixel"
announce_interval = 5

[beardog]
bind_address = "0.0.0.0:9000"
enable_stun = true
abstract_socket = "@beardog"

[songbird]
bind_address = "0.0.0.0:9001"
enable_mdns = true
enable_stun = true
abstract_socket = "@songbird"
EOF

adb push /tmp/tower_pixel_config.toml /data/local/tmp/tower_config.toml
```

---

### Phase 4: Start Services (5 min)

#### Linux (USB) - Start TOWER

**Terminal 1 - BearDog**:
```bash
# Start BearDog with STUN
~/.local/beardog/beardog server \
  --bind 0.0.0.0:9000 \
  --stun stun.l.google.com:19302 \
  --log-level debug
```

**Terminal 2 - Songbird**:
```bash
# Start Songbird with discovery
~/.local/songbird/songbird server \
  --bind 0.0.0.0:9001 \
  --stun stun.l.google.com:19302 \
  --service tower-usb \
  --log-level debug
```

#### Android (Pixel) - Start TOWER

**Terminal 3 - BearDog**:
```bash
adb shell "/data/local/tmp/beardog/beardog server \
  --bind 0.0.0.0:9000 \
  --stun stun.l.google.com:19302 \
  --log-level debug &"
```

**Terminal 4 - Songbird**:
```bash
adb shell "/data/local/tmp/songbird/songbird server \
  --bind 0.0.0.0:9001 \
  --stun stun.l.google.com:19302 \
  --service tower-pixel \
  --log-level debug &"
```

---

### Phase 5: Test Discovery (10 min)

**Objective**: Verify both TOWER instances discover each other

#### Check Songbird Discovery

**Linux**:
```bash
# Query local Songbird for discovered services
curl http://localhost:9001/api/v1/services | jq
```

**Android**:
```bash
# Query Pixel Songbird for discovered services
adb shell "curl http://localhost:9001/api/v1/services" | jq
```

**Expected**:
- Linux Songbird should see `tower-pixel`
- Pixel Songbird should see `tower-usb`
- Both should have correct IP addresses and ports

#### Test mDNS Discovery (Local Network)

**Linux**:
```bash
# Scan for mDNS services
avahi-browse -rt _tower._tcp
```

**Expected**:
- Should see both `tower-usb` and `tower-pixel` advertised

---

### Phase 6: Crypto Handshake (10 min)

**Objective**: Establish secure encrypted channel between platforms

#### Linux → Android Handshake

**Linux**:
```bash
# Connect Linux BearDog to Pixel BearDog
~/.local/beardog/beardog connect \
  --target 192.168.1.80:9000 \
  --log-level debug
```

**Expected**:
```
✅ Key exchange (Ed25519) successful
✅ Encrypted channel (ChaCha20-Poly1305) established
✅ Genetic lineage verified
```

#### Android → Linux Handshake

**Android**:
```bash
# Connect Pixel BearDog to Linux BearDog
adb shell "/data/local/tmp/beardog/beardog connect \
  --target 192.168.1.144:9000"
```

**Expected**:
```
✅ Key exchange successful
✅ Encrypted channel established
✅ Genetic lineage verified
```

---

### Phase 7: STUN NAT Traversal (15 min)

**Objective**: Test connection via public STUN (simulating internet deployment)

#### Get Public Endpoints

**Linux**:
```bash
# Get public IP:port via STUN
~/.local/beardog/beardog stun-info --server stun.l.google.com:19302
```

**Android**:
```bash
# Get public IP:port via STUN
adb shell "/data/local/tmp/beardog/beardog stun-info \
  --server stun.l.google.com:19302"
```

#### Connect via STUN-Discovered Endpoints

**Linux → Android (via STUN)**:
```bash
# Use public endpoint from STUN
~/.local/beardog/beardog connect \
  --target <pixel-public-ip>:<pixel-public-port> \
  --via-stun stun.l.google.com:19302
```

**Expected**:
- NAT traversal successful
- Connection established via public endpoints
- Same crypto handshake as local connection

---

### Phase 8: Federation Validation (15 min)

**Objective**: Test full federated communication

#### Register Services Cross-Platform

**Linux → Register with Pixel**:
```bash
# Register Linux services with Pixel discovery
~/.local/songbird/songbird register \
  --target 192.168.1.80:9001 \
  --service tower-usb \
  --capabilities crypto,discovery
```

**Android → Register with Linux**:
```bash
# Register Pixel services with Linux discovery
adb shell "/data/local/tmp/songbird/songbird register \
  --target 192.168.1.144:9001 \
  --service tower-pixel \
  --capabilities crypto,discovery,hsm"
```

#### Query Federated Registry

**Linux**:
```bash
# Query federated services
~/.local/songbird/songbird query --all
```

**Expected**:
```json
{
  "local": ["tower-usb"],
  "federated": ["tower-pixel"],
  "capabilities": {
    "tower-usb": ["crypto", "discovery"],
    "tower-pixel": ["crypto", "discovery", "hsm"]
  }
}
```

---

### Phase 9: Secure Messaging (10 min)

**Objective**: Exchange encrypted messages cross-platform

#### Linux → Android Message

```bash
# Send encrypted message
~/.local/beardog/beardog send \
  --target tower-pixel \
  --message "Hello from USB x86_64! 🧬" \
  --encrypt
```

#### Android → Linux Message

```bash
# Send encrypted message
adb shell "/data/local/tmp/beardog/beardog send \
  --target tower-usb \
  --message 'Hello from Pixel ARM64! 📱' \
  --encrypt"
```

**Expected**:
- Messages encrypted with ChaCha20-Poly1305
- Authenticated with Ed25519 signatures
- Decrypted and verified on receiving end

---

### Phase 10: Performance Benchmarking (15 min)

**Objective**: Measure cross-platform performance

#### Latency Test

```bash
# Ping-pong test (1000 iterations)
~/.local/beardog/beardog benchmark \
  --target 192.168.1.80:9000 \
  --iterations 1000 \
  --size 1024
```

**Metrics to Capture**:
- Round-trip latency (ms)
- Throughput (MB/s)
- Crypto overhead
- Connection stability

#### Throughput Test

```bash
# Large transfer test (100MB)
~/.local/beardog/beardog transfer \
  --target 192.168.1.80:9000 \
  --size 100M \
  --encrypt
```

**Expected**:
- Latency: <5ms (local network)
- Throughput: >100 MB/s (local network)
- Encryption overhead: <10%

---

## 📊 Success Criteria

### Deployment
- ✅ TOWER deployed on USB (x86_64)
- ✅ TOWER already deployed on Pixel (ARM64)
- ✅ Same genomeBin files used
- ✅ Health checks passing on both

### Discovery
- ✅ mDNS discovery working (local network)
- ✅ STUN discovery working (public endpoints)
- ✅ Both platforms discover each other
- ✅ Service capabilities advertised

### Security
- ✅ Ed25519 key exchange successful
- ✅ ChaCha20-Poly1305 encryption working
- ✅ Genetic lineage verification passing
- ✅ HSM integration ready (Android)

### Connectivity
- ✅ Direct connection (local IPs)
- ✅ STUN-assisted connection (NAT traversal)
- ✅ Bi-directional communication
- ✅ Connection stability (no drops)

### Federation
- ✅ Service registration cross-platform
- ✅ Federated discovery working
- ✅ Capability negotiation working
- ✅ Secure messaging operational

### Performance
- ✅ Latency acceptable (<10ms local)
- ✅ Throughput acceptable (>50 MB/s)
- ✅ Encryption overhead acceptable (<20%)
- ✅ Connection stable under load

---

## 🎯 Validation Matrix

| Test | USB (x86_64) | Pixel (ARM64) | Cross-Platform | Status |
|------|--------------|---------------|----------------|--------|
| genomeBin Deploy | ⏳ | ✅ | N/A | In Progress |
| Health Check | ⏳ | ✅ | N/A | In Progress |
| Service Start | ⏳ | ⏳ | N/A | Pending |
| mDNS Discovery | ⏳ | ⏳ | ⏳ | Pending |
| STUN Discovery | ⏳ | ⏳ | ⏳ | Pending |
| Direct Connect | ⏳ | ⏳ | ⏳ | Pending |
| STUN Connect | ⏳ | ⏳ | ⏳ | Pending |
| Crypto Handshake | ⏳ | ⏳ | ⏳ | Pending |
| Secure Messaging | ⏳ | ⏳ | ⏳ | Pending |
| Performance | ⏳ | ⏳ | ⏳ | Pending |

---

## 🔧 Troubleshooting

### No Discovery
- Check firewall rules (ports 9000, 9001)
- Verify both on same network or STUN reachable
- Check mDNS daemon running (avahi/mdnsresponder)
- Verify STUN servers accessible

### Connection Failed
- Check NAT type (symmetric NAT may require relay)
- Verify public endpoints from STUN
- Check network connectivity
- Verify services listening on correct ports

### Handshake Failed
- Check genetic lineage keys
- Verify crypto libraries loaded
- Check HSM availability (Android)
- Verify key exchange protocol version

---

## 📈 Expected Timeline

| Phase | Duration | Total |
|-------|----------|-------|
| Deploy USB TOWER | 5 min | 5 min |
| Verify Pixel | 2 min | 7 min |
| Configure STUN | 10 min | 17 min |
| Start Services | 5 min | 22 min |
| Test Discovery | 10 min | 32 min |
| Crypto Handshake | 10 min | 42 min |
| STUN Traversal | 15 min | 57 min |
| Federation | 15 min | 72 min |
| Secure Messaging | 10 min | 82 min |
| Benchmarking | 15 min | 97 min |

**Total Estimated Time**: ~1.5 hours

---

## 🎊 Success = Production Ready!

**If all phases pass**:
- ✅ Universal genomeBin deployment proven
- ✅ Cross-platform federation working
- ✅ Real-world NAT traversal validated
- ✅ Production-ready infrastructure confirmed

**Vision Realized**:
**ONE COMMAND → ANY PLATFORM → GLOBAL FEDERATION**

---

*Plan Created: January 31, 2026*  
*Validation Type: Cross-Platform STUN Rendezvous*  
*Platforms: Linux x86_64 + Android ARM64*  
*Network: Local (192.168.1.x) + Public STUN*
