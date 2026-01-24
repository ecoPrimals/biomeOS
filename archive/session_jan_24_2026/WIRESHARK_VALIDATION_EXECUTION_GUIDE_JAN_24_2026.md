# 🔬 WIRESHARK VALIDATION GUIDE - EXECUTION
## Step-by-Step Analysis of TLS 1.3 Handshake

**Date**: January 24, 2026, 10:20 AM  
**Status**: Wireshark opened - Ready for analysis  
**Files**:
- Packet capture: `/tmp/songbird-wireshark.pcap` (6.8 KB)
- SSLKEYLOGFILE: `/tmp/tls-keys.log` (632 bytes, 4 secrets)

---

## ✅ STEP 1: CONFIGURE WIRESHARK

Wireshark should now be open. Follow these steps:

### **A. Load SSLKEYLOGFILE**
1. **Edit** → **Preferences** (or press `Ctrl+Shift+P`)
2. Expand **Protocols** in left sidebar
3. Scroll to **TLS** (or type "TLS" in search box)
4. Find **(Pre)-Master-Secret log filename**
5. Click **Browse** or type: `/tmp/tls-keys.log`
6. Click **OK**

### **B. Apply Display Filter**
In the filter bar at top, enter:
```
tls
```
This will show only TLS packets.

---

## 🔍 STEP 2: VERIFY DECRYPTION

### **What to Look For:**

#### **A. Handshake Messages (Should Be Decrypted!)**

Look for these packets in Wireshark:
1. **ClientHello** (plaintext - always visible)
2. **ServerHello** (plaintext - always visible)
3. **EncryptedExtensions** ← Should show decrypted content!
4. **Certificate** ← Should show certificate details!
5. **CertificateVerify** ← Should show signature!
6. **Finished** ← Should show verify data!

#### **B. Application Data (HTTP Request)**

After handshake, look for:
- **Application Data** packets
- Should show decrypted HTTP request!
- Look for `GET / HTTP/1.1`

---

## 📊 STEP 3: ANALYSIS SCENARIOS

### **Scenario 1: ✅ Everything Decrypts**

If you see:
- ✅ EncryptedExtensions content visible
- ✅ Certificate details visible
- ✅ CertificateVerify visible
- ✅ Finished verify data visible
- ✅ HTTP request visible: `GET / HTTP/1.1`

**Conclusion**: Our TLS 1.3 is WORKING!
- Keys are correct
- Secrets are valid
- Implementation is RFC 8446 compliant

**Why server rejects**:
- Server-specific issue
- Or subtle content difference in transcript

**Next**: Compare decrypted handshake with our transcript (see Step 4)

---

### **Scenario 2: ❌ Handshake Doesn't Decrypt**

If you see:
- ❌ EncryptedExtensions: `[Encrypted Application Data]`
- ❌ Certificate: `[Encrypted Application Data]`
- ❌ Wireshark shows: "Decryption failed"

**Conclusion**: Handshake keys are wrong!
- Problem in handshake secret derivation
- Check early secret → handshake secret calculation
- Issue is in HANDSHAKE phase, not application

**To Fix**:
1. Check `CLIENT_HANDSHAKE_TRAFFIC_SECRET` in SSLKEYLOGFILE
2. Verify it matches what Wireshark expects
3. Check handshake secret derivation in BearDog

---

### **Scenario 3: ❌ Can't Decrypt HTTP (MOST LIKELY!)**

If you see:
- ✅ Handshake messages decrypt fine
- ❌ Application Data: `[Encrypted Application Data]`
- ❌ HTTP request not visible

**Conclusion**: Application keys are wrong! **THIS IS THE ISSUE!**
- Handshake keys: ✅ Working
- Application keys: ❌ Wrong
- Problem: **TRANSCRIPT HASH** used for app key derivation

**Root Cause**:
- Our transcript STRUCTURE: ✅ Correct
- Our transcript CONTENT: ❌ Has differences
- Different content → different SHA-256 hash
- Different hash → wrong application keys
- Wrong keys → server can't decrypt HTTP request
- → `decrypt_error (0x33)`

**To Fix**: See Step 4 - Compare transcript content

---

### **Scenario 4: ❌ Decrypts But Server Rejects**

If you see:
- ✅ Everything decrypts in Wireshark
- ✅ HTTP request visible
- ❌ Server still sends `decrypt_error`

**Conclusion**: Our TLS is correct, but server computes transcript differently
- Implementation: ✅ Correct (Wireshark confirms)
- Server expectation: Different

**Possible Causes**:
- Server includes/excludes different messages
- Server has different extension handling
- Server-specific TLS stack behavior

---

## 🔬 STEP 4: COMPARE TRANSCRIPT CONTENT

If Scenario 1 or 3 (most likely), compare decrypted handshake with our transcript:

### **A. Extract Decrypted Messages from Wireshark**

For each handshake message:
1. Right-click the packet
2. **Copy** → **Bytes** → **Hex Stream**
3. Paste into text editor
4. Label it (e.g., "Wireshark_EncryptedExtensions.hex")

Extract:
- ClientHello (plaintext)
- ServerHello (plaintext)
- EncryptedExtensions (decrypted)
- Certificate (decrypted)
- CertificateVerify (decrypted)
- Finished (decrypted)

### **B. Compare with Our Transcript**

Our transcript (from Songbird v5.12.9):
- Total: 4455 bytes
- Hex dump available in logs
- Message breakdown:
  - ClientHello: ~187 bytes
  - ServerHello: ~122 bytes
  - EncryptedExtensions: ~119 bytes
  - Certificate: ~1604 bytes
  - CertificateVerify: ~204 bytes
  - Finished: ~36 bytes

### **C. Find Differences**

Compare byte-by-byte:
```bash
# If you save Wireshark extracts as files:
diff <(xxd wireshark_handshake.bin) <(xxd our_transcript.bin)
```

**Look for**:
- Different certificate chains
- Different extensions
- Different signature algorithms
- Different session data
- Extra/missing bytes

---

## 🛠️ STEP 5: WIRESHARK TIPS

### **Useful Filters**:
```
tls.handshake.type == 1    # ClientHello
tls.handshake.type == 2    # ServerHello
tls.handshake.type == 8    # EncryptedExtensions
tls.handshake.type == 11   # Certificate
tls.handshake.type == 15   # CertificateVerify
tls.handshake.type == 20   # Finished
tls.app_data               # Application Data
http                       # HTTP (if decrypted)
```

### **Export Decrypted Data**:
1. File → Export Packet Dissections → As Plain Text
2. Save to file
3. Analyze offline

### **Check TLS Version**:
Look for:
- `Version: TLS 1.3 (0x0304)`
- `Supported Version: TLS 1.3`

### **Check Cipher Suite**:
Should see:
- `Cipher Suite: TLS_AES_128_GCM_SHA256 (0x1301)`

---

## 📋 STEP 6: DOCUMENT FINDINGS

### **Create Report**:

```
WIRESHARK VALIDATION RESULTS
Date: 2026-01-24
Capture: /tmp/songbird-wireshark.pcap
SSLKEYLOGFILE: /tmp/tls-keys.log

DECRYPTION STATUS:
[ ] ClientHello: Visible (plaintext)
[ ] ServerHello: Visible (plaintext)
[ ] EncryptedExtensions: Decrypted? YES/NO
[ ] Certificate: Decrypted? YES/NO
[ ] CertificateVerify: Decrypted? YES/NO
[ ] Finished: Decrypted? YES/NO
[ ] HTTP Request: Visible? YES/NO

SCENARIO: 1/2/3/4 (from Step 3)

FINDINGS:
- Handshake keys: Working/Not Working
- Application keys: Working/Not Working
- Issue: <describe>

TRANSCRIPT COMPARISON:
- Differences found: YES/NO
- Details: <describe differences>

RECOMMENDED FIX:
<describe what needs to be changed>
```

---

## 🎯 EXPECTED OUTCOME

Based on our validation so far, **most likely scenario is 3**:
- ✅ Handshake decrypts (handshake keys correct)
- ❌ HTTP doesn't decrypt (application keys wrong)
- Issue: Transcript content differences

**Why?**
- We've validated transcript STRUCTURE is correct
- But CONTENT likely has subtle differences
- Different content → different SHA-256 hash
- → Wrong application keys

**The Fix**:
1. Find exact differences in Wireshark comparison
2. Update Songbird's transcript construction
3. Rebuild and test
4. **HTTP 200 OK!** 🎉

---

## 💡 QUICK REFERENCE

### **Files**:
- Capture: `/tmp/songbird-wireshark.pcap`
- Keys: `/tmp/tls-keys.log`
- Logs: `/tmp/final-test.log`

### **Our Transcript**:
- Total: 4455 bytes
- Hash: `32a32ff17353e812980ec17595700bd885cba22eb6b0e1ffc38216060e5acfa3`
- Messages: 6 (ClientHello through server Finished)

### **SSLKEYLOGFILE Contents**:
```
CLIENT_HANDSHAKE_TRAFFIC_SECRET 6974e278... 62db39a7...
SERVER_HANDSHAKE_TRAFFIC_SECRET 6974e278... e8f3f677...
CLIENT_TRAFFIC_SECRET_0 6974e278... df4db5f6...
SERVER_TRAFFIC_SECRET_0 6974e278... 83efc858...
```

---

**Status**: Wireshark open - Ready for analysis  
**ETA**: 30 minutes to identify issue  
**Confidence**: 95% - Ground truth validation!  

**"Analyze Wireshark decryption results!"** 🔬  
**"Compare with our transcript!"** 📊  
**"Find the differences!"** 🎯

