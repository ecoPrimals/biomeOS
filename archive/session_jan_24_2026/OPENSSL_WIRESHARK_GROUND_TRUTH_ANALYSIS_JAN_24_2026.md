# 🔬 OpenSSL + Wireshark Ground Truth Analysis - January 24, 2026
## Immediate Parallel Investigation While Songbird Builds Server

**Date**: January 24, 2026, 9:35 AM  
**Status**: 🟢 **ACTIVE** - Running parallel investigation!  
**Goal**: Get ground truth from OpenSSL while Songbird builds server!  
**ETA**: 1-2 hours to findings!  

---

## 🎯 PARALLEL STRATEGY

**Track 1**: Songbird Team → Build client + server capabilities (3-4 hours)  
**Track 2**: biomeOS Team → OpenSSL + Wireshark analysis (1-2 hours) ← **WE ARE HERE!**  

**Advantage**: We get immediate insights while server is being built!

---

## 🔬 INVESTIGATION 1: OpenSSL Transcript Comparison

### **Goal**: Extract OpenSSL's transcript and compare with ours!

### **Step 1: Capture OpenSSL Handshake with example.com** (10 min)

```bash
# Capture full TLS 1.3 handshake with detailed output
openssl s_client -connect example.com:443 \
  -tls1_3 \
  -msg \
  -debug \
  -state \
  -showcerts \
  2>&1 | tee /tmp/openssl-example-com.log

# Press Ctrl+C after connection is established
```

**What This Shows**:
- All handshake messages in hex
- Message types and lengths
- Complete TLS flow
- Certificate chain

### **Step 2: Extract Handshake Messages** (20 min)

Create a parser to extract the exact bytes:

```bash
cat << 'EOF' > /tmp/extract_openssl_transcript.py
#!/usr/bin/env python3
"""Extract TLS handshake transcript from OpenSSL -msg output."""

import re
import sys

def parse_openssl_msg_output(log_file):
    """Parse OpenSSL -msg output to extract handshake messages."""
    
    with open(log_file, 'r') as f:
        content = f.read()
    
    # Find all handshake messages
    # OpenSSL format: ">>> TLS 1.3, Handshake [length XXXX]"
    # Followed by hex dump
    
    messages = []
    
    # Pattern for outgoing messages (>>>)
    out_pattern = r'>>> TLS 1\.3, Handshake \[length (\d+)\]\s+((?:[0-9a-f]{2}\s*)+)'
    
    # Pattern for incoming messages (<<<)
    in_pattern = r'<<< TLS 1\.3, Handshake \[length (\d+)\]\s+((?:[0-9a-f]{2}\s*)+)'
    
    for match in re.finditer(out_pattern, content, re.IGNORECASE | re.MULTILINE):
        length = int(match.group(1))
        hex_bytes = match.group(2).strip().replace(' ', '').replace('\n', '')
        messages.append(('OUT', length, hex_bytes))
    
    for match in re.finditer(in_pattern, content, re.IGNORECASE | re.MULTILINE):
        length = int(match.group(1))
        hex_bytes = match.group(2).strip().replace(' ', '').replace('\n', '')
        messages.append(('IN', length, hex_bytes))
    
    return messages

def reconstruct_transcript(messages):
    """Reconstruct the TLS transcript from messages."""
    
    transcript = b''
    
    print("═══════════════════════════════════════════════════════════")
    print("🔬 OPENSSL TRANSCRIPT RECONSTRUCTION")
    print("═══════════════════════════════════════════════════════════")
    print()
    
    for i, (direction, length, hex_str) in enumerate(messages, 1):
        msg_bytes = bytes.fromhex(hex_str)
        
        # First byte is message type
        if len(msg_bytes) > 0:
            msg_type = msg_bytes[0]
            msg_names = {
                0x01: "ClientHello",
                0x02: "ServerHello",
                0x08: "EncryptedExtensions",
                0x0b: "Certificate",
                0x0f: "CertificateVerify",
                0x14: "Finished",
            }
            msg_name = msg_names.get(msg_type, f"Unknown (0x{msg_type:02x})")
            
            print(f"Message #{i}: {msg_name} ({direction})")
            print(f"  Length: {length} bytes")
            print(f"  Type: 0x{msg_type:02x}")
            print(f"  First 32 bytes: {hex_str[:64]}")
            print()
            
            # Add to transcript (for ClientHello, ServerHello, and server's encrypted messages)
            # Note: We DON'T include client Finished in the transcript used for app key derivation!
            if msg_name in ["ClientHello", "ServerHello", "EncryptedExtensions", 
                           "Certificate", "CertificateVerify", "Finished"]:
                if not (msg_name == "Finished" and direction == "OUT"):
                    transcript += msg_bytes
    
    return transcript

if __name__ == "__main__":
    log_file = "/tmp/openssl-example-com.log"
    
    messages = parse_openssl_msg_output(log_file)
    transcript = reconstruct_transcript(messages)
    
    print("═══════════════════════════════════════════════════════════")
    print(f"Total transcript length: {len(transcript)} bytes")
    print()
    
    # Compute SHA-256
    import hashlib
    transcript_hash = hashlib.sha256(transcript).hexdigest()
    print(f"Transcript hash (SHA-256): {transcript_hash}")
    print()
    
    # Compare with Songbird
    songbird_hash = "2adfdd2271cf3eb30ad2b67c9aa68bab8e982a3bbfa8050244cc6045b90fdc42"
    print(f"Songbird hash:             {songbird_hash}")
    print()
    
    if transcript_hash == songbird_hash:
        print("✅ MATCH! Transcripts are identical!")
    else:
        print("❌ MISMATCH! Transcripts differ!")
        print()
        print("Saving transcripts for comparison...")
        
        # Save for byte-by-byte comparison
        with open('/tmp/openssl-transcript.bin', 'wb') as f:
            f.write(transcript)
        
        with open('/tmp/openssl-transcript.hex', 'w') as f:
            f.write(transcript.hex())
        
        print("OpenSSL transcript saved to:")
        print("  - /tmp/openssl-transcript.bin")
        print("  - /tmp/openssl-transcript.hex")
    
    print("═══════════════════════════════════════════════════════════")

EOF

chmod +x /tmp/extract_openssl_transcript.py
```

### **Step 3: Run Comparison** (5 min)

```bash
# Extract OpenSSL transcript
python3 /tmp/extract_openssl_transcript.py

# If they differ, compare byte-by-byte
if [ -f /tmp/openssl-transcript.hex ]; then
    echo "Comparing with Songbird transcript..."
    
    # Get Songbird transcript (from our hex dump)
    cat /tmp/transcript-hex-dump-complete.txt | \
        sed 's/^[0-9a-f]*: //' | \
        tr -d '\n' > /tmp/songbird-transcript.hex
    
    # Show lengths
    echo "OpenSSL transcript:  $(wc -c < /tmp/openssl-transcript.hex) hex chars"
    echo "Songbird transcript: $(wc -c < /tmp/songbird-transcript.hex) hex chars"
    
    # Byte-by-byte diff
    diff -u \
        <(fold -w 128 /tmp/openssl-transcript.hex) \
        <(fold -w 128 /tmp/songbird-transcript.hex) \
        | head -100
fi
```

---

## 🔬 INVESTIGATION 2: Wireshark with SSLKEYLOGFILE

### **Goal**: Capture our TLS handshake and decrypt it in Wireshark!

### **Step 1: Enable SSLKEYLOGFILE in BearDog** (20 min)

**We need BearDog to export session keys!**

Add to BearDog's TLS key derivation:

```rust
// In BearDog: After deriving keys, log them
pub fn log_sslkeylogfile(&self, client_random: &[u8], master_secret: &[u8]) -> Result<()> {
    let keylog_path = std::env::var("SSLKEYLOGFILE")
        .unwrap_or("/tmp/tls-keys.log".to_string());
    
    let mut file = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&keylog_path)?;
    
    // Format: "CLIENT_RANDOM <client_random> <master_secret>"
    writeln!(file, "CLIENT_RANDOM {} {}", 
        hex::encode(client_random),
        hex::encode(master_secret))?;
    
    info!("🔐 Logged session keys to {}", keylog_path);
    Ok(())
}
```

**Note**: For TLS 1.3, we actually need to log the traffic secrets:

```
CLIENT_HANDSHAKE_TRAFFIC_SECRET <client_random> <secret>
SERVER_HANDSHAKE_TRAFFIC_SECRET <client_random> <secret>
CLIENT_TRAFFIC_SECRET_0 <client_random> <secret>
SERVER_TRAFFIC_SECRET_0 <client_random> <secret>
```

### **Step 2: Capture Traffic** (10 min)

```bash
# Terminal 1: Start packet capture
sudo tcpdump -i any -w /tmp/songbird-tls.pcap \
    'host example.com and port 443' &
TCPDUMP_PID=$!

# Wait a moment for tcpdump to start
sleep 2

# Terminal 2: Enable SSLKEYLOGFILE and make request
export SSLKEYLOGFILE=/tmp/tls-keys.log
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
./target/release/neural-api-server > /tmp/neural.log 2>&1 &
sleep 5

echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"https://example.com","headers":{}},"id":1}' | \
    nc -N -U /tmp/songbird-nat0.sock

# Stop capture
sudo kill $TCPDUMP_PID
```

### **Step 3: Decrypt in Wireshark** (15 min)

```bash
# Open in Wireshark
wireshark /tmp/songbird-tls.pcap &

# In Wireshark GUI:
# 1. Edit → Preferences → Protocols → TLS
# 2. (Pre)-Master-Secret log filename: /tmp/tls-keys.log
# 3. Click OK
# 4. Right-click any TLS packet → Protocol Preferences → Try to decrypt TLS
```

**What to Look For**:
1. **ClientHello** packet
   - Right-click → Copy → Bytes → Hex Stream
   - Compare with our transcript offset 0x0000
2. **ServerHello** packet
   - Same process
   - Compare with our transcript offset 0x00bf
3. **Encrypted Handshake** packets
   - Wireshark should show decrypted content!
   - Compare with our transcript offsets

### **Step 4: Extract Decrypted Handshake** (10 min)

In Wireshark:
1. Filter: `tls.handshake`
2. For each handshake packet:
   - Expand "Transport Layer Security"
   - Expand "Handshake Protocol"
   - Right-click on handshake message → Copy → Bytes → Hex Stream
3. Save each message to files:
   - `/tmp/wireshark-clienthello.hex`
   - `/tmp/wireshark-serverhello.hex`
   - `/tmp/wireshark-encryptedext.hex`
   - etc.

### **Step 5: Compare with Our Transcript** (10 min)

```bash
# Compare each message
for msg in clienthello serverhello encryptedext certificate certverify finished; do
    if [ -f /tmp/wireshark-$msg.hex ]; then
        echo "Comparing $msg..."
        
        # Extract corresponding section from our transcript
        # (This requires knowing the offsets from our analysis)
        
        diff -u \
            <(cat /tmp/wireshark-$msg.hex | fold -w 64) \
            <(cat /tmp/our-$msg.hex | fold -w 64)
    fi
done
```

---

## 🔬 INVESTIGATION 3: Quick HTTP Encryption Check

### **Goal**: Verify our HTTP request encryption is correct!

### **Add Logging to Songbird** (Quick - 15 min)

In `songbird-http-client/src/client.rs` or wherever HTTP encryption happens:

```rust
info!("════════════════════════════════════════════════════════════");
info!("🔐 HTTP REQUEST ENCRYPTION DIAGNOSTICS");
info!("════════════════════════════════════════════════════════════");

// Before encryption
info!("HTTP request plaintext:");
info!("  Length: {} bytes", http_plaintext.len());
info!("  First 64 bytes: {}", hex::encode(&http_plaintext[..64.min(http_plaintext.len())]));
info!("  Method: GET, Path: /, Version: HTTP/1.1");

// Encryption parameters
info!("Encryption parameters:");
info!("  Sequence number: {}", sequence_number);  // Should be 0 for first request!
info!("  Key: {}", hex::encode(&client_write_key));
info!("  IV: {}", hex::encode(&client_write_iv));

// Compute nonce (RFC 8446 Section 5.3)
let mut nonce = client_write_iv.clone();
for i in 0..8 {
    nonce[12 - 8 + i] ^= ((sequence_number >> (8 * (7 - i))) & 0xff) as u8;
}
info!("  Nonce (IV XOR seq): {}", hex::encode(&nonce));

// AAD (RFC 8446 Section 5.2)
let aad = vec![
    0x17,  // ContentType: application_data
    0x03, 0x03,  // TLS 1.2 (legacy version)
    ((ciphertext_len >> 8) & 0xff) as u8,
    (ciphertext_len & 0xff) as u8,
];
info!("  AAD: {}", hex::encode(&aad));

// After encryption
info!("Encrypted TLS record:");
info!("  Ciphertext length: {} bytes", ciphertext.len());
info!("  First 64 bytes: {}", hex::encode(&ciphertext[..64.min(ciphertext.len())]));

info!("════════════════════════════════════════════════════════════");
```

**Deploy and Test** (10 min):
```bash
# Rebuild Songbird with new logging
cd /home/eastgate/Development/ecoPrimals/phase1/songbird
cargo build --release --bin songbird

# Relink
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
rm -f plasmidBin/primals/songbird/songbird
ln -s /home/eastgate/Development/ecoPrimals/phase1/songbird/target/release/songbird \
    plasmidBin/primals/songbird/songbird

# Deploy and test
./target/release/neural-api-server > /tmp/http-encryption-test.log 2>&1 &
sleep 5
echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"GET","url":"https://example.com","headers":{}},"id":1}' | \
    nc -N -U /tmp/songbird-nat0.sock

# Check logs
grep "HTTP REQUEST ENCRYPTION" -A30 /tmp/http-encryption-test.log
```

**Verify**:
- Sequence number should be 0 ✅
- Nonce should be IV XOR 0 (which is just IV) ✅
- AAD should be [0x17, 0x03, 0x03, length_hi, length_lo] ✅

---

## 📊 EXPECTED OUTCOMES

### **Scenario 1: OpenSSL transcript matches ours** (40%)
```
OpenSSL hash:  2adfdd2271cf3eb30ad2b67c9aa68bab...
Songbird hash: 2adfdd2271cf3eb30ad2b67c9aa68bab...
✅ MATCH!
```
**Conclusion**: Our transcript IS correct! Issue must be in HTTP encryption!  
**Next**: Focus on HTTP encryption diagnostics!

### **Scenario 2: OpenSSL transcript differs** (50%)
```
OpenSSL hash:  5f3c2a1b9d8e7f6a...
Songbird hash: 2adfdd2271cf3eb30ad2b67c9aa68bab...
❌ MISMATCH!
```
**Conclusion**: We're computing transcript differently than OpenSSL!  
**Next**: Byte-by-byte comparison to find exact difference!

### **Scenario 3: Can't extract OpenSSL transcript** (10%)
**Conclusion**: Need Wireshark approach!  
**Next**: Proceed with packet capture and Wireshark decryption!

---

## ⏱️ TIMELINE

| Task | Time | Status |
|------|------|--------|
| Capture OpenSSL handshake | 10 min | ⏳ Ready |
| Extract OpenSSL transcript | 20 min | ⏳ Ready |
| Compare hashes | 5 min | ⏳ Ready |
| **Subtotal: OpenSSL** | **35 min** | |
| Enable SSLKEYLOGFILE | 20 min | ⏳ Need BearDog update |
| Capture traffic | 10 min | ⏳ Ready |
| Decrypt in Wireshark | 15 min | ⏳ Ready |
| Extract & compare | 20 min | ⏳ Ready |
| **Subtotal: Wireshark** | **65 min** | |
| Add HTTP encryption logging | 15 min | ⏳ Ready |
| Test & verify | 10 min | ⏳ Ready |
| **Subtotal: HTTP Check** | **25 min** | |
| **TOTAL** | **~2 hours** | |

---

## 🎯 IMMEDIATE ACTION PLAN

**Right Now** (Choose based on what's easiest):

**Option A**: Start with OpenSSL (simplest, no code changes!)
```bash
openssl s_client -connect example.com:443 -tls1_3 -msg -debug 2>&1 | tee /tmp/openssl-example-com.log
# Then run the Python parser
```

**Option B**: Start with HTTP encryption logging (quick code change!)
- Add the diagnostic logging to Songbird
- Redeploy
- Check if encryption params are correct

**Option C**: Start with Wireshark (requires SSLKEYLOGFILE support)
- Need to update BearDog to export keys
- Then capture and decrypt

**RECOMMENDATION**: **Start with Option A (OpenSSL)** - requires no code changes, immediate results!

---

**Prepared by**: biomeOS Team  
**Date**: January 24, 2026, 9:35 AM  
**Status**: Ready to execute!  
**ETA**: 1-2 hours to findings!  
**Confidence**: 85% - One of these will reveal the issue!  

**"Parallel investigation while server is built!"** 🔬  
**"Multiple angles of attack!"** 🎯  
**"We WILL find it!"** 🚀

