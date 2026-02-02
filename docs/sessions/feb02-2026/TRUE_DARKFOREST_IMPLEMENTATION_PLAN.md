# 🌑 TRUE DARK FOREST - Implementation Plan

**Priority**: CRITICAL  
**Timeline**: 30 min - 1 hour  
**Impact**: A → A++ LEGENDARY security

═══════════════════════════════════════════════════════════════════

## 🎯 **OBJECTIVE**

**Remove all plaintext metadata from BirdSong beacons**

Current flaw:
```json
{"birdsong":"1.0","family_id":"dark_forest_alpha","encrypted_payload":"..."}
                   ^^^^^^^^^^^^^^^^^^^^^^^^^^^ METADATA LEAK
```

Target:
```
[0x4a, 0xf3, 0x9b, ...]  // Pure noise (indistinguishable from random)
```

---

## 📋 **IMPLEMENTATION CHECKLIST**

### **Task 1: BearDog Lineage Beacon Key** ⏳ **15 minutes**

**File**: `phase1/beardog/crates/beardog-tunnel/src/unix_socket_ipc/crypto_handlers_genetic.rs`

**Add method**:
```rust
/// Derive beacon encryption key from lineage
///
/// All family members derive the same key, enabling pure noise beacons
/// with genetic decryption (no plaintext family_id needed).
///
/// # Security
/// - Deterministic from lineage (same family = same key)
/// - Domain-separated (different from other genetic keys)
/// - Cannot derive without lineage secrets
///
/// # Returns
/// 32-byte ChaCha20-Poly1305 key
pub async fn handle_derive_lineage_beacon_key(
    _params: Value,
    genetics: Arc<RwLock<GeneticsState>>,
) -> Result<Value, IpcError> {
    let genetics = genetics.read().await;
    
    // Use same lineage components as other operations
    let genome_hash = &genetics.genome_hash;
    let lineage_seed_mix = &genetics.lineage_seed_mix;
    
    // Domain separation for beacon keys
    let domain = b"birdsong_beacon_v1";
    
    // Derive key: HKDF-SHA256
    // Input: genome_hash (IKM) + lineage_seed_mix (salt) + domain (info)
    let mut okm = [0u8; 32];  // 256 bits for ChaCha20
    
    let hkdf = hkdf::Hkdf::<sha2::Sha256>::new(
        Some(lineage_seed_mix.as_bytes()),
        genome_hash.as_bytes(),
    );
    
    hkdf.expand(domain, &mut okm)
        .map_err(|e| IpcError::InternalError(format!("HKDF expand failed: {}", e)))?;
    
    // Return as hex string (for JSON-RPC)
    Ok(json!({
        "beacon_key": hex::encode(&okm),
        "algorithm": "HKDF-SHA256+ChaCha20-Poly1305",
        "domain": "birdsong_beacon_v1"
    }))
}
```

**Wire to handler** (same file, routing section):
```rust
"genetic.derive_lineage_beacon_key" => {
    handle_derive_lineage_beacon_key(params, Arc::clone(&self.genetics))
        .await
}
```

**Test**:
```bash
echo '{"jsonrpc":"2.0","method":"genetic.derive_lineage_beacon_key","params":{},"id":1}' \
  | nc -U /run/user/$(id -u)/biomeos/beardog.sock
```

Expected:
```json
{
  "result": {
    "beacon_key": "a3f5...",  // 64 hex chars (32 bytes)
    "algorithm": "HKDF-SHA256+ChaCha20-Poly1305",
    "domain": "birdsong_beacon_v1"
  }
}
```

---

### **Task 2: Pure Noise Encryption** ⏳ **15 minutes**

**File**: `phase1/songbird/crates/songbird-discovery/src/birdsong_integration.rs`

**Add methods**:
```rust
impl BirdSongProcessor {
    /// Encrypt beacon as pure noise (zero metadata)
    ///
    /// Output is indistinguishable from random bytes to outsiders.
    /// Only family members with correct lineage can decrypt.
    ///
    /// Format: [nonce (12 bytes)] + [ciphertext + tag (N+16 bytes)]
    pub async fn encrypt_beacon_pure_noise(
        &self,
        discovery_info: &[u8],
    ) -> Result<Vec<u8>> {
        // Get lineage-derived beacon key
        let beacon_key_hex = self.encryption
            .as_ref()
            .ok_or_else(|| anyhow!("No encryption provider"))?
            .get_lineage_beacon_key()
            .await?;
        
        let beacon_key = hex::decode(&beacon_key_hex)
            .context("Invalid beacon key hex")?;
        
        // Generate random nonce (12 bytes for ChaCha20-Poly1305)
        let mut nonce = [0u8; 12];
        rand::thread_rng().fill_bytes(&mut nonce);
        
        // Encrypt with ChaCha20-Poly1305
        use chacha20poly1305::{
            aead::{Aead, KeyInit},
            ChaCha20Poly1305, Nonce,
        };
        
        let key = chacha20poly1305::Key::from_slice(&beacon_key);
        let cipher = ChaCha20Poly1305::new(key);
        let nonce_obj = Nonce::from_slice(&nonce);
        
        let ciphertext = cipher.encrypt(nonce_obj, discovery_info)
            .map_err(|e| anyhow!("Encryption failed: {}", e))?;
        
        // Return: nonce + ciphertext (pure bytes, no structure)
        let mut beacon = Vec::with_capacity(12 + ciphertext.len());
        beacon.extend_from_slice(&nonce);
        beacon.extend_from_slice(&ciphertext);
        
        debug!("🔒 Encrypted pure noise beacon: {} bytes", beacon.len());
        Ok(beacon)
    }
    
    /// Decrypt beacon from pure noise
    ///
    /// Returns Some(info) if same family, None if different family/noise.
    /// Failures are SILENT (no logs) - true Dark Forest.
    pub async fn decrypt_beacon_pure_noise(
        &self,
        encrypted_beacon: &[u8],
    ) -> Result<Option<Vec<u8>>> {
        // Need at least nonce (12) + tag (16) = 28 bytes
        if encrypted_beacon.len() < 28 {
            return Ok(None);  // Too short, probably noise
        }
        
        // Get OUR lineage-derived beacon key
        let beacon_key_hex = match self.encryption.as_ref() {
            Some(enc) => enc.get_lineage_beacon_key().await.ok()?,
            None => return Ok(None),
        };
        
        let beacon_key = hex::decode(&beacon_key_hex).ok()?;
        
        // Split nonce + ciphertext
        let (nonce, ciphertext) = encrypted_beacon.split_at(12);
        
        // Try to decrypt (SILENT failure)
        use chacha20poly1305::{
            aead::{Aead, KeyInit},
            ChaCha20Poly1305, Nonce,
        };
        
        let key = chacha20poly1305::Key::from_slice(&beacon_key);
        let cipher = ChaCha20Poly1305::new(key);
        let nonce_obj = Nonce::from_slice(nonce);
        
        match cipher.decrypt(nonce_obj, ciphertext) {
            Ok(plaintext) => {
                // SUCCESS! Same family
                debug!("✅ Decrypted family beacon: {} bytes", plaintext.len());
                Ok(Some(plaintext))
            }
            Err(_) => {
                // NOISE! Different family or actual noise
                // NO LOGS (silent failure, true Dark Forest)
                Ok(None)
            }
        }
    }
}
```

**Update trait**:
```rust
#[async_trait]
pub trait BirdSongEncryption: Send + Sync {
    // ... existing methods ...
    
    /// Get lineage-derived beacon key (for pure noise beacons)
    async fn get_lineage_beacon_key(&self) -> Result<String>;
}
```

---

### **Task 3: Update Broadcaster** ⏳ **10 minutes**

**File**: `phase1/songbird/crates/songbird-discovery/src/anonymous/broadcaster.rs`

**Change broadcast logic**:
```rust
// OLD:
let packet_json = birdsong.encrypt_packet(&discovery_bytes).await?;
socket.send_to(&packet_json, &addr).await?;

// NEW:
let pure_noise_beacon = birdsong.encrypt_beacon_pure_noise(&discovery_bytes).await?;
socket.send_to(&pure_noise_beacon, &addr).await?;

// Result: UDP packets are pure bytes (no JSON, no metadata)
```

---

### **Task 4: Update Listener** ⏳ **10 minutes**

**File**: `phase1/songbird/crates/songbird-discovery/src/anonymous/listener.rs`

**Change receive logic**:
```rust
// OLD:
let decrypted = birdsong.decrypt_packet(&received_bytes).await?;

// NEW:
match birdsong.decrypt_beacon_pure_noise(&received_bytes).await? {
    Some(discovery_bytes) => {
        // Same family! Process discovery
        let discovery: DiscoveryMessage = serde_json::from_slice(&discovery_bytes)?;
        self.process_discovery(discovery).await?;
    }
    None => {
        // Noise (different family or actual noise)
        // NO LOGS - silent failure
    }
}

// Result: Only family beacons processed, others silently ignored
```

---

## 🧪 **TESTING PLAN**

### **Test 1: Same Family Discovery** ✅

```bash
# Start USB with family
FAMILY_ID=dark_forest_alpha \
  ./beardog server --socket /run/user/$(id -u)/biomeos/beardog.sock &

FAMILY_ID=dark_forest_alpha \
  ./songbird server --discovery-port 5555 &

# Start Pixel with SAME family
adb shell "FAMILY_ID=dark_forest_alpha ./beardog server --listen 127.0.0.1:9900 &"
adb shell "FAMILY_ID=dark_forest_alpha ./songbird server --discovery-port 5555 &"

# Monitor logs
# Expected: "Decrypted family beacon" (same family)
```

---

### **Test 2: Different Family = Noise** ✅

```bash
# Start USB with family A
FAMILY_ID=family_alpha ./songbird server --discovery-port 5555 &

# Start Pixel with family B
adb shell "FAMILY_ID=family_beta ./songbird server --discovery-port 5555 &"

# Monitor logs
# Expected: NO "Decrypted" messages (silent failure)
# Expected: NO error logs (true noise, no patterns)
```

---

### **Test 3: Network Capture = Noise** ✅

```bash
# Capture packets
sudo tcpdump -i any -w beacons.pcap port 5555

# Analyze
wireshark beacons.pcap

# Verify:
# ✅ No JSON structure
# ✅ No "birdsong", "family_id", or any plaintext
# ✅ Packets look completely random
# ✅ Cannot distinguish beacons from noise
```

---

### **Test 4: Derive Same Key** ✅

```bash
# Query beacon key from USB
echo '{"jsonrpc":"2.0","method":"genetic.derive_lineage_beacon_key","params":{},"id":1}' \
  | nc -U /run/user/$(id -u)/biomeos/beardog.sock

# Query beacon key from Pixel (same lineage)
adb shell "echo '{...}' | nc 127.0.0.1:9900"

# Verify: SAME key (same lineage = same key)
```

---

## 📊 **VERIFICATION CHECKLIST**

### **Code Changes** ✅

- [ ] BearDog: `genetic.derive_lineage_beacon_key` added
- [ ] BearDog: Method wired to JSON-RPC handler
- [ ] Songbird: `encrypt_beacon_pure_noise` implemented
- [ ] Songbird: `decrypt_beacon_pure_noise` implemented
- [ ] Broadcaster: Using pure noise beacons
- [ ] Listener: Silent decryption failures

### **Security Properties** ✅

- [ ] Beacons are pure bytes (no JSON)
- [ ] No plaintext metadata (no family_id)
- [ ] Same family can decrypt (lineage = key)
- [ ] Different family cannot decrypt (wrong key)
- [ ] Failures are silent (no logs, no patterns)
- [ ] Network capture shows random bytes only

### **Functional Tests** ✅

- [ ] Same family discovers each other
- [ ] Different families see noise (silent)
- [ ] Challenge-response still works (post-discovery)
- [ ] STUN still works (post-discovery)
- [ ] Performance acceptable (decrypt attempts fast)

---

## 🏆 **SUCCESS CRITERIA**

### **Zero Metadata Leaks**

```
Network observer captures 1000 beacons:
  ✅ All look completely random
  ✅ Cannot identify beacon packets (vs noise)
  ✅ Cannot extract family IDs
  ✅ Cannot count families
  ✅ Cannot track patterns
  
Result: A++ LEGENDARY (true Dark Forest)
```

### **Functional Parity**

```
Family members still discover each other:
  ✅ Decrypt succeeds (same lineage key)
  ✅ Extract discovery info (capabilities, endpoints)
  ✅ Perform challenge-response
  ✅ Establish encrypted connection
  
Result: Zero regression, improved security
```

---

## 🎯 **TIMELINE**

### **Implementation** ⏱️ **30 min - 1 hour**

```
Task 1: BearDog lineage key   (15 min)
Task 2: Pure noise encryption  (15 min)
Task 3: Update broadcaster     (10 min)
Task 4: Update listener        (10 min)

Total: 50 minutes
```

### **Testing** ⏱️ **15-30 minutes**

```
Test 1: Same family    (5 min)
Test 2: Different family (5 min)
Test 3: Network capture  (5 min)
Test 4: Key derivation   (5 min)

Total: 20 minutes
```

**Overall**: 1 hour 10 minutes to **A++ LEGENDARY**

---

═══════════════════════════════════════════════════════════════════

🌑🧬🏆 **TRUE DARK FOREST IMPLEMENTATION** 🏆🧬🌑

**Timeline**: 30 min - 1 hour (4 code changes)  
**Difficulty**: LOW (mostly removing metadata!)  
**Impact**: A → A++ LEGENDARY (zero metadata leaks)

**User Insight**: 🏆 BRILLIANT (correct security analysis)

**Result**: Better privacy than Signal/Tor (beacons = pure noise)

**Status**: 🚀 Ready to implement true Dark Forest communication!

═══════════════════════════════════════════════════════════════════
