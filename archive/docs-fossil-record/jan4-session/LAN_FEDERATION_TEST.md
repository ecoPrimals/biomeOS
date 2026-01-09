# 🌐 LAN Federation Test - Two Physical Towers

**Date**: January 4, 2026 17:40 EST  
**Test**: Multi-Tower LAN Discovery & Communication  
**Status**: 🟡 **IN PROGRESS - Tower 1 Ready**

---

## 🎯 Test Objective

Validate **true LAN federation** between two physical towers:
- **Tower 1**: Local machine (this computer)
- **Tower 2**: Remote machine (other tower)

Both towers share the same **family** (nat0) but have **unique identities** (tower1 vs tower2).

---

## 📊 Tower 1 Status (Ready!)

### **Configuration**
```
Location: Local machine (pop-os)
NODE_ID: tower1
FAMILY_ID: nat0
Parent Seed: Nat0C/G/b4B7u06n0r14... (from biomeOS1 USB)
IP Address: (will be displayed after network detection)
Multicast: 239.255.42.99:4242 (listening)
```

### **Running Primals**
- ✅ **Tower** process (orchestrator)
- ✅ **BearDog** (security, encryption)
- ✅ **Songbird** (UDP multicast discovery)

### **Sockets**
- `/tmp/beardog-nat0-tower1.sock` (Unix socket IPC)
- `/tmp/songbird-nat0-tower1.sock` (Unix socket IPC)

### **Logs**
- `/tmp/tower1-lan.log` (full tower log)

---

## 🚀 Tower 2 Setup Instructions

### **On Tower 2 (After USB Insertion)**

1. **Mount the USB**:
   ```bash
   # USB should auto-mount at /media/.../biomeOS
   ls /media/*/biomeOS  # Verify it's mounted
   ```

2. **Navigate to biomeOS directory**:
   ```bash
   cd /media/.../biomeOS
   # Or specifically:
   cd /media/.../biomeOS2/biomeOS
   ```

3. **Start Tower 2**:
   ```bash
   ./activate-tower.sh
   ```

4. **Expected Output**:
   ```
   🚀 Starting Tower with TOML configuration...
   Configuration: tower.toml
   BearDog (Security) + Songbird (Discovery)
   
   INFO tower: 🚀 Starting tower with modern config-driven orchestration
   INFO tower: 📦 Loading primal from config: ./primals/beardog
   INFO tower: 📦 Loading primal from config: ./primals/songbird
   INFO tower: ✅ Tower started successfully!
   ```

---

## 🔍 What Should Happen

### **1. UDP Multicast Discovery**

**Tower 2 broadcasts**:
```
UDP Multicast → 239.255.42.99:4242
Message: "I am tower2 in family nat0, here's my info..."
```

**Tower 1 receives**:
```
Songbird on Tower 1 receives multicast
Adds tower2 to peer registry
Logs: "Discovered peer: tower2 (family: nat0)"
```

### **2. Peer Registry Update**

Both towers maintain a peer registry:
```
Tower 1 Registry:
  - Self: tower1 (local)
  - Peer: tower2 (discovered via UDP multicast)

Tower 2 Registry:
  - Self: tower2 (local)
  - Peer: tower1 (discovered via UDP multicast)
```

### **3. Family Membership Verification**

Towers verify family membership via BirdSong protocol:
```
tower1 → tower2: "Prove you're in family nat0"
tower2 → tower1: [cryptographic proof derived from family seed]
tower1: ✅ Verified! tower2 is in family nat0
```

---

## ✅ Validation Tests

### **Test 1: Check Tower 1's Peer List**

**On Tower 1 (this machine)**:
```bash
echo '{"jsonrpc":"2.0","method":"discovery.list_peers","id":1}' | \
  nc -U /tmp/songbird-nat0-tower1.sock
```

**Expected Response**:
```json
{
  "jsonrpc": "2.0",
  "result": [
    {
      "node_id": "tower2",
      "family_id": "nat0",
      "ip_address": "192.168.x.x",
      "last_seen": "2026-01-04T17:45:00Z",
      "capabilities": ["Discovery"]
    }
  ],
  "id": 1
}
```

### **Test 2: Check Tower 2's Peer List**

**On Tower 2 (remote machine)**:
```bash
echo '{"jsonrpc":"2.0","method":"discovery.list_peers","id":1}' | \
  nc -U /tmp/songbird-nat0-tower2.sock
```

**Expected Response**:
```json
{
  "jsonrpc": "2.0",
  "result": [
    {
      "node_id": "tower1",
      "family_id": "nat0",
      "ip_address": "192.168.x.x",
      "last_seen": "2026-01-04T17:45:00Z",
      "capabilities": ["Discovery"]
    }
  ],
  "id": 1
}
```

### **Test 3: Ping Each Other**

**From Tower 1 to Tower 2**:
```bash
# (This might require discovering Tower 2's socket path first)
# Or use HTTP API if available
curl -X POST http://tower2-ip:port/api/ping
```

### **Test 4: Check Multicast Traffic**

**On either tower**:
```bash
# Listen for UDP multicast traffic
sudo tcpdump -i any -n 'dst host 239.255.42.99 and dst port 4242'
```

**Expected Output**:
```
IP tower1-ip > 239.255.42.99.4242: UDP, length 128
IP tower2-ip > 239.255.42.99.4242: UDP, length 128
```

---

## 📋 Validation Checklist

### **Tower 1 (Local)**
- [x] Processes running (Tower, BearDog, Songbird)
- [x] Sockets created
- [x] Listening on UDP multicast (239.255.42.99:4242)
- [ ] Discovered Tower 2 as peer
- [ ] Can query Tower 2's peer list
- [ ] Can ping Tower 2

### **Tower 2 (Remote)**
- [ ] USB inserted and mounted
- [ ] Tower started successfully
- [ ] Processes running (Tower, BearDog, Songbird)
- [ ] Sockets created
- [ ] Broadcasting on UDP multicast
- [ ] Discovered Tower 1 as peer
- [ ] Can query Tower 1's peer list
- [ ] Can ping Tower 1

---

## 🧬 Genetic Lineage Over LAN

Both towers share the **same parent seed** but have **unique identities**:

```
Tower 1:
  Parent Seed: Nat0C/G/b4B7u06n0r14... (from biomeOS1 USB)
  Local Mixing: pop-os + machine-uuid-1 + tower1 + rng1
  Child Key: derived_key_1 (unique)

Tower 2:
  Parent Seed: Nat0C/G/b4B7u06n0r14... (from biomeOS2 USB, same seed!)
  Local Mixing: other-hostname + machine-uuid-2 + tower2 + rng2
  Child Key: derived_key_2 (unique, different from tower1!)

Result:
  - Same family (nat0) → can verify membership
  - Different identities → unique child keys
  - Secure federation → cryptographic proof of family membership
```

---

## 🎯 What This Validates

### **Architecture**
- ✅ **UDP Multicast Discovery**: Cross-tower peer discovery
- ✅ **Port-Free Architecture**: No TCP port management needed
- ✅ **Genetic Lineage**: Same family, different identities
- ✅ **Fractal Scaling**: N towers can join the family

### **Security**
- ✅ **Family Membership**: Cryptographic verification via BirdSong
- ✅ **Unique Identities**: Each tower has a unique child key
- ✅ **Encrypted Communication**: BearDog provides encryption layer

### **Performance**
- ✅ **Zero Configuration**: No manual IP/port setup
- ✅ **Automatic Discovery**: Towers find each other via multicast
- ✅ **Resilient**: If one tower goes down, others keep running

---

## 📚 Expected Logs

### **Tower 1 (When Tower 2 Joins)**
```
INFO songbird: 📡 Received multicast announcement
INFO songbird: 🔍 Discovered new peer: tower2
INFO songbird:    Family: nat0 ✅
INFO songbird:    IP: 192.168.x.x
INFO songbird:    Capabilities: [Discovery]
INFO songbird: ✅ Added tower2 to peer registry
```

### **Tower 2 (When It Starts)**
```
INFO songbird: 📡 Broadcasting discovery announcement
INFO songbird:    NODE_ID: tower2
INFO songbird:    FAMILY_ID: nat0
INFO songbird:    Multicast: 239.255.42.99:4242
INFO songbird: 📡 Received multicast announcement
INFO songbird: 🔍 Discovered existing peer: tower1
INFO songbird:    Family: nat0 ✅
INFO songbird: ✅ Added tower1 to peer registry
```

---

## 🚨 Troubleshooting

### **Tower 2 Doesn't Appear in Peer List**

**Possible Causes**:
1. **Firewall blocking UDP multicast**:
   ```bash
   # On both towers:
   sudo ufw allow from 239.255.42.99
   ```

2. **Different subnets**:
   - UDP multicast may not cross router boundaries
   - Ensure both towers are on the same LAN

3. **Multicast not enabled on network interface**:
   ```bash
   # Check multicast support:
   ip maddr show
   ```

4. **Tower 2 not broadcasting**:
   - Check Tower 2's logs: `tail -f /tmp/tower2-lan.log`
   - Verify Songbird is running: `ps aux | grep songbird`

### **Can See Tower 2 But Can't Communicate**

**Possible Causes**:
1. **Socket paths different** (should be fixed with NODE_ID scoping)
2. **Authentication required** (BearDog might require family proof)
3. **Network latency** (increase timeout in tests)

---

## 🎊 Success Criteria

**Test is successful when**:
1. ✅ Tower 1 discovers Tower 2 (appears in peer list)
2. ✅ Tower 2 discovers Tower 1 (appears in peer list)
3. ✅ Both towers verify family membership (nat0)
4. ✅ Both towers can query each other's capabilities
5. ✅ UDP multicast traffic visible on network

**This proves**:
- Multi-tower LAN federation works! 🌐
- Genetic lineage enables secure federation 🧬
- Port-free architecture scales to N towers 🚀

---

## 📝 Notes

- **Tower 1 IP**: (detected at runtime)
- **Tower 2 IP**: (will be detected when Tower 2 starts)
- **Multicast Group**: 239.255.42.99:4242
- **Family ID**: nat0
- **Parent Seed**: Same on both USB spores
- **Child Keys**: Different per tower (due to NODE_ID mixing)

---

**Status**: 🟡 **Tower 1 Ready, Awaiting Tower 2**  
**Next**: Move biomeOS2 USB to Tower 2 and start it  
**Then**: Validate discovery and communication!

🎯 **This is the moment of truth for true LAN federation!** 🌐

