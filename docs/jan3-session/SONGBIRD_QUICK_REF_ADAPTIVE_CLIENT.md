# 🎯 Quick Reference: Adaptive Client Integration for Songbird

**Time Required**: 15 minutes  
**Risk**: Very low  
**Impact**: Completes genetic federation! 🎊

---

## The Problem (1-Sentence)

Songbird expects BearDog API field `"encrypted"` but receives `"ciphertext"`, causing it to interpret success as failure.

## The Solution (1-Sentence)

Use adaptive client that accepts BOTH field names using serde aliases.

---

## Integration (Copy-Paste Ready)

### Option 1: Full Integration (Recommended)

#### Step 1: Add Dependency

```toml
# crates/songbird-discovery/Cargo.toml
[dependencies]
biomeos-core = { path = "../../../phase2/biomeOS/crates/biomeos-core" }
```

#### Step 2: Replace BearDogBirdSongProvider

```rust
// crates/songbird-discovery/src/beardog_birdsong_provider.rs

use biomeos_core::BirdSongClient;
use anyhow::Result;
use async_trait::async_trait;

pub struct BearDogBirdSongProvider {
    client: BirdSongClient,
}

impl BearDogBirdSongProvider {
    pub fn new(endpoint: String) -> Self {
        Self {
            client: BirdSongClient::new(endpoint),
        }
    }
}

#[async_trait]
impl BirdSongProvider for BearDogBirdSongProvider {
    async fn encrypt(&mut self, plaintext: String, family_id: String) -> Result<String> {
        self.client.encrypt(plaintext, family_id).await
    }

    async fn decrypt(&mut self, encrypted: String, family_id: String) -> Result<String> {
        self.client.decrypt(encrypted, family_id).await
    }
}
```

That's it! The adaptive client handles everything:
- ✅ Tries v1, falls back to v2
- ✅ Remembers which version works
- ✅ Comprehensive logging
- ✅ Retry with backoff

---

### Option 2: Minimal Change (Just Fix Parsing)

```rust
// In your existing BearDogBirdSongProvider response struct

#[derive(Debug, Deserialize)]
struct BirdSongEncryptResponse {
    #[serde(alias = "ciphertext")]  // v2 format
    pub encrypted: String,          // v1 format (canonical)
    pub family_id: String,
}

// That's it! Now works with both v1 and v2
```

---

## Testing

```bash
# Build
cd /home/eastgate/Development/ecoPrimals/phase1/songbird
cargo build --release

# Test locally
RUST_LOG=debug \
SONGBIRD_BEARDOG_URL="http://localhost:9000" \
SONGBIRD_TOWER_NAME="test-tower" \
./target/release/songbird-orchestrator
```

### Expected Logs (Success!)

```
✅ BirdSong API version detected: v1
🎵 BirdSong encrypted discovery packet (family: iidn, size: 2216 bytes)
📡 Broadcasting encrypted discovery...
```

### NOT (Failure):

```
⚠️  BirdSong encryption failed: BirdSong encryption failed, using plaintext
```

---

## Two-Tower Verification

### Tower 1 (Local)

```bash
RUST_LOG=info \
SONGBIRD_BEARDOG_URL="http://localhost:9000" \
SONGBIRD_TOWER_NAME="tower-one" \
./songbird-orchestrator
```

### Tower 2 (Check Logs)

```bash
tail -f /tmp/songbird_tower-one.log | grep -E "genetic|family|trust"
```

### Expected Output:

```
👨‍👩‍👧‍👦 Peer has genetic lineage: family=iidn
✅ Same family detected
✅ Trust Decision: AUTO-ACCEPT (reason: same_family)
🎊 HISTORIC MOMENT: First genetic federation auto-trust!
```

---

## Files Modified

**Option 1 (Full)**:
- `crates/songbird-discovery/Cargo.toml` (1 line added)
- `crates/songbird-discovery/src/beardog_birdsong_provider.rs` (replace ~30 lines)

**Option 2 (Minimal)**:
- `crates/songbird-discovery/src/beardog_birdsong_provider.rs` (add 1 line: `#[serde(alias)]`)

---

## Documentation

**Full Details**:
- Root Cause: `docs/jan3-session/FINAL_INTEGRATION_DEBUG_JAN_3_2026.md`
- Integration Guide: `docs/jan3-session/ADAPTIVE_CLIENT_INTEGRATION_GUIDE.md`
- Source Code: `crates/biomeos-core/src/adaptive_client.rs`

**Quick Start**:
- This card! 😊

---

## Troubleshooting

### If it still fails after integration:

1. **Check logs** with `RUST_LOG=debug`
2. **Look for**:
   ```
   📡 AdaptiveClient POST: http://...
   📤 Request body: ...
   📥 Response status: ...
   📥 Response body: ...
   ```
3. **Verify BearDog is running**: `curl http://localhost:9000/api/v1/health`
4. **Check family seed**: `curl http://localhost:9000/api/v1/trust/identity | jq '.family_id'`

### If BearDog returns empty family_id:

Start BearDog with environment variables:
```bash
export BEARDOG_FAMILY_ID="iidn"
export BEARDOG_FAMILY_SEED="iIDnVX3Tein1LFkrkkq7Wo3wsxPNek9XZqp0VL4Kn88="
./beardog-server-v0.15.0-with-v2-api
```

---

## Next Steps After Success

1. ✅ Verify auto-trust between two towers
2. ✅ Update USB spore with new Songbird binary
3. ✅ Test across different networks
4. 🎊 **Celebrate historic genetic federation!**
5. 🚀 Plan next features (cross-family relay, NAT traversal)

---

**Questions?** See full docs or ping the biomeOS team!

🦀 **Let's complete this federation!** 🌸

