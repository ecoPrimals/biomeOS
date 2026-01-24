# 🔬 Track 3: Wireshark + SSLKEYLOGFILE Implementation Guide
## Decrypt and Analyze Our TLS Handshake

**Date**: January 24, 2026, 10:15 AM  
**Priority**: 🟡 **HIGH** - Provides ground truth of our handshake!  
**ETA**: 1 hour  
**Confidence**: 90% - Will show exactly what we're sending!  

---

## 🎯 GOAL

Enable SSLKEYLOGFILE support in BearDog so we can:
1. Capture our TLS handshake with tcpdump
2. Decrypt it in Wireshark using session keys
3. See EXACTLY what we're sending (decrypted!)
4. Compare with what server expects

---

## 📍 WHERE TO ADD THE CODE

### **File**: `beardog/src/tls.rs` (or wherever TLS key derivation happens)

Based on the fact that Songbird calls BearDog for key derivation, we need to add SSLKEYLOGFILE export in BearDog after deriving keys.

---

## 💻 CODE TO ADD

### **Step 1: Add SSLKEYLOGFILE Export Function**

Add this function to BearDog:

```rust
use std::fs::OpenOptions;
use std::io::Write;

/// Export TLS session keys in SSLKEYLOGFILE format (RFC 8446)
/// 
/// Format for TLS 1.3:
/// CLIENT_HANDSHAKE_TRAFFIC_SECRET <client_random> <secret>
/// SERVER_HANDSHAKE_TRAFFIC_SECRET <client_random> <secret>
/// CLIENT_TRAFFIC_SECRET_0 <client_random> <secret>
/// SERVER_TRAFFIC_SECRET_0 <client_random> <secret>
/// 
/// This allows Wireshark to decrypt the TLS session!
pub fn export_to_sslkeylogfile(
    client_random: &[u8],
    handshake_secrets: Option<(&[u8], &[u8])>, // (client_hs_secret, server_hs_secret)
    application_secrets: Option<(&[u8], &[u8])>, // (client_app_secret, server_app_secret)
) -> Result<()> {
    // Check if SSLKEYLOGFILE env var is set
    let keylog_path = match std::env::var("SSLKEYLOGFILE") {
        Ok(path) => path,
        Err(_) => {
            // SSLKEYLOGFILE not set, skip (this is normal in production)
            return Ok(());
        }
    };
    
    info!("🔐 Exporting TLS session keys to SSLKEYLOGFILE: {}", keylog_path);
    
    // Open file in append mode
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&keylog_path)
        .map_err(|e| anyhow!("Failed to open SSLKEYLOGFILE: {}", e))?;
    
    let client_random_hex = hex::encode(client_random);
    
    // Export handshake secrets
    if let Some((client_hs_secret, server_hs_secret)) = handshake_secrets {
        writeln!(file, "CLIENT_HANDSHAKE_TRAFFIC_SECRET {} {}", 
                 client_random_hex, hex::encode(client_hs_secret))
            .map_err(|e| anyhow!("Failed to write to SSLKEYLOGFILE: {}", e))?;
        
        writeln!(file, "SERVER_HANDSHAKE_TRAFFIC_SECRET {} {}", 
                 client_random_hex, hex::encode(server_hs_secret))
            .map_err(|e| anyhow!("Failed to write to SSLKEYLOGFILE: {}", e))?;
        
        info!("  ✅ Exported handshake traffic secrets");
    }
    
    // Export application secrets
    if let Some((client_app_secret, server_app_secret)) = application_secrets {
        writeln!(file, "CLIENT_TRAFFIC_SECRET_0 {} {}", 
                 client_random_hex, hex::encode(client_app_secret))
            .map_err(|e| anyhow!("Failed to write to SSLKEYLOGFILE: {}", e))?;
        
        writeln!(file, "SERVER_TRAFFIC_SECRET_0 {} {}", 
                 client_random_hex, hex::encode(server_app_secret))
            .map_err(|e| anyhow!("Failed to write to SSLKEYLOGFILE: {}", e))?;
        
        info!("  ✅ Exported application traffic secrets");
    }
    
    info!("🔐 Session keys successfully exported!");
    info!("   Wireshark can now decrypt this TLS session!");
    
    Ok(())
}
```

### **Step 2: Call This Function After Key Derivation**

In your `tls_derive_handshake_secrets` function:

```rust
pub fn tls_derive_handshake_secrets(
    pre_master_secret: &[u8],
    client_random: &[u8],
    server_random: &[u8],
    cipher_suite: u16,
) -> Result<HandshakeSecrets> {
    // ... existing key derivation code ...
    
    let client_handshake_secret = /* ... */;
    let server_handshake_secret = /* ... */;
    
    // Export to SSLKEYLOGFILE if env var is set
    export_to_sslkeylogfile(
        client_random,
        Some((&client_handshake_secret, &server_handshake_secret)),
        None, // No application secrets yet
    )?;
    
    // ... rest of function ...
}
```

In your `tls_derive_application_secrets` function:

```rust
pub fn tls_derive_application_secrets(
    pre_master_secret: &[u8],
    client_random: &[u8],
    server_random: &[u8],
    transcript_hash: &[u8],
    cipher_suite: u16,
) -> Result<ApplicationSecrets> {
    // ... existing key derivation code ...
    
    let client_application_secret = /* ... */;
    let server_application_secret = /* ... */;
    
    // Export to SSLKEYLOGFILE if env var is set
    export_to_sslkeylogfile(
        client_random,
        None, // Already exported handshake secrets
        Some((&client_application_secret, &server_application_secret)),
    )?;
    
    // ... rest of function ...
}
```

---

## 🧪 USAGE

### **Step 1: Set Environment Variable**

```bash
export SSLKEYLOGFILE=/tmp/tls-keys.log
```

### **Step 2: Start Packet Capture**

In one terminal:
```bash
sudo tcpdump -i any -w /tmp/songbird-tls.pcap 'host example.com and port 443'
```

### **Step 3: Make HTTPS Request**

In another terminal:
```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS

# Start Neural API with SSLKEYLOGFILE
SSLKEYLOGFILE=/tmp/tls-keys.log ./target/release/neural-api-server > /tmp/neural.log 2>&1 &
sleep 5

# Make request
echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"https://example.com","headers":{}},"id":1}' | \
    nc -N -U /tmp/songbird-nat0.sock
```

### **Step 4: Stop Packet Capture**

Press Ctrl+C in the tcpdump terminal.

### **Step 5: Open in Wireshark**

```bash
wireshark /tmp/songbird-tls.pcap &
```

**In Wireshark GUI**:
1. Go to: Edit → Preferences
2. Navigate to: Protocols → TLS
3. Set "(Pre)-Master-Secret log filename" to: `/tmp/tls-keys.log`
4. Click OK
5. Right-click any TLS packet → "Decode As..." → (if needed)
6. Right-click again → "Protocol Preferences" → "Try to decrypt TLS"

### **Step 6: View Decrypted Data**

In Wireshark, you should now see:
- **Decrypted handshake messages** (ClientHello, ServerHello, etc.)
- **Decrypted HTTP request** (GET / HTTP/1.1...)
- **Decrypted HTTP response**

---

## 📊 EXPECTED SSLKEYLOGFILE FORMAT

The `/tmp/tls-keys.log` file should contain:

```
CLIENT_HANDSHAKE_TRAFFIC_SECRET 697459304c535a61686f767d848b9299a0a7aeb5bcc3cad1d8dfe6edf4fb0209 2c6504277fb08472a0e1c7d65e9cf09f8f7c6b5a4938271605f4e3d2c1b0a099
SERVER_HANDSHAKE_TRAFFIC_SECRET 697459304c535a61686f767d848b9299a0a7aeb5bcc3cad1d8dfe6edf4fb0209 8d7c6b5a4938271605f4e3d2c1b0a0992c6504277fb08472a0e1c7d65e9cf09f
CLIENT_TRAFFIC_SECRET_0 697459304c535a61686f767d848b9299a0a7aeb5bcc3cad1d8dfe6edf4fb0209 48d566dbe8bb07d33ab06fc01a71a8fe94e5a16e6cc8c0b8a9e6f5d4c3b2a190
SERVER_TRAFFIC_SECRET_0 697459304c535a61686f767d848b9299a0a7aeb5bcc3cad1d8dfe6edf4fb0209 3b7a69582746351402f3d1c0afbeaf0e8c5d4e3f2a1b0c9d8e7f6a5b4c3d2e1f
```

Format: `LABEL <client_random_hex> <secret_hex>`

---

## 🔍 WHAT TO LOOK FOR IN WIRESHARK

### **1. Handshake Messages**

Filter: `tls.handshake`

Look at each handshake message:
- **ClientHello**: Expand → Handshake Protocol → Copy bytes
- **ServerHello**: Same
- **EncryptedExtensions**: Should be DECRYPTED! Compare with our transcript
- **Certificate**: Should be DECRYPTED! Compare with our transcript
- **CertificateVerify**: Should be DECRYPTED! Compare with our transcript
- **Finished**: Should be DECRYPTED! Compare with our transcript

### **2. Our HTTP Request**

Filter: `http`

You should see:
```
GET / HTTP/1.1
Host: example.com
User-Agent: Songbird
Connection: close

```

**If you see this**: Our HTTP request is correct! ✅  
**If you don't see this**: Our HTTP encryption is wrong! ❌

### **3. TLS Alerts**

Filter: `tls.alert`

Look for alerts from server:
- **decrypt_error (0x33)**: Server can't decrypt our request
- **bad_record_mac (0x14)**: MAC/tag verification failed
- **illegal_parameter (0x2f)**: Parameter issue

### **4. Compare Transcript**

For each encrypted handshake message:
1. Right-click → Copy → Bytes → Hex Stream
2. Compare with our transcript hex dump
3. Should be IDENTICAL!

**If different**: We know exactly what byte(s) differ!

---

## 🎯 DIAGNOSTIC SCENARIOS

### **Scenario 1: Wireshark CAN decrypt everything** ✅
```
✅ All handshake messages decrypted
✅ HTTP request visible
✅ HTTP response visible
```
**Conclusion**: Our TLS is WORKING! Issue is elsewhere (maybe HTTP format?)

### **Scenario 2: Wireshark CANNOT decrypt handshake messages** ❌
```
❌ EncryptedExtensions: [Encrypted Application Data]
❌ Certificate: [Encrypted Application Data]
❌ Wireshark says "Decryption failed"
```
**Conclusion**: Our handshake keys are wrong! Check HKDF derivation!

### **Scenario 3: Wireshark CAN decrypt handshake, but NOT HTTP request** ❌
```
✅ All handshake messages decrypted
❌ HTTP request: [Encrypted Application Data]
❌ Wireshark says "Decryption failed"
```
**Conclusion**: Our application keys are wrong! Check transcript hash!

### **Scenario 4: Wireshark CAN decrypt everything, but server rejects** ❌
```
✅ All handshake messages decrypted
✅ HTTP request visible
❌ Server sends decrypt_error anyway
```
**Conclusion**: Server's transcript differs from ours! Need to find why!

---

## 🛠️ TROUBLESHOOTING

### **Problem: SSLKEYLOGFILE is empty**

**Solution**: Make sure:
1. Environment variable is set: `echo $SSLKEYLOGFILE`
2. BearDog can write to the file: `touch /tmp/tls-keys.log && chmod 666 /tmp/tls-keys.log`
3. BearDog is actually calling `export_to_sslkeylogfile()`

### **Problem: Wireshark doesn't decrypt anything**

**Solution**: Check:
1. SSLKEYLOGFILE is loaded in Wireshark preferences
2. File format is correct (see expected format above)
3. Client random matches between keylog and pcap
4. TLS version is 1.3 (Wireshark needs different format for TLS 1.2)

### **Problem: Only partial decryption**

**Solution**: Check if:
1. All 4 secrets are exported (handshake + application, client + server)
2. Secrets are exported BEFORE use (timing issue?)
3. Correct secrets are used for correct phase

---

## 🎯 NEXT STEPS

1. **Add SSLKEYLOGFILE support** to BearDog (30 min)
2. **Build**: `cargo build --release`
3. **Set env var**: `export SSLKEYLOGFILE=/tmp/tls-keys.log`
4. **Capture traffic**: `sudo tcpdump ...`
5. **Make request**: Via Neural API
6. **Open in Wireshark**: Load pcap + keylog
7. **Analyze**: Compare decrypted data with our transcript
8. **Fix**: If any difference found!

---

## 💡 BONUS: Compare with OpenSSL

You can also capture OpenSSL's handshake and compare:

```bash
# Capture OpenSSL
export SSLKEYLOGFILE=/tmp/openssl-keys.log
sudo tcpdump -i any -w /tmp/openssl-tls.pcap 'host example.com and port 443' &
openssl s_client -connect example.com:443 -tls1_3 < /dev/null

# Open both in Wireshark
wireshark /tmp/songbird-tls.pcap &
wireshark /tmp/openssl-tls.pcap &

# Load respective keylogs
# Compare side-by-side!
```

---

**Prepared by**: biomeOS Team  
**Date**: January 24, 2026, 10:15 AM  
**For**: BearDog + Songbird Teams  
**Priority**: HIGH - Provides ground truth!  
**ETA**: 1 hour to results!  
**Confidence**: 90% - Will show exactly what we're sending!  

**"Decrypt our handshake with Wireshark!"** 🔬  
**"See exactly what we're sending!"** 👀  
**"Ground truth comparison!"** ✅

