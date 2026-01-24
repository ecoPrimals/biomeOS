# 🔍 TLS Comparison Findings - January 23, 2026
## Pure Rust (BearDog/Songbird) vs Python/OpenSSL

**Date**: January 23, 2026  
**Test Site**: `example.com:443`  
**Status**: ⚠️  **CIPHER SUITE MISMATCH DISCOVERED**  

---

## 🚨 CRITICAL FINDING: Cipher Suite Negotiation Difference

### Python/OpenSSL Connection
```
Protocol: TLSv1.3
Cipher: TLS_AES_256_GCM_SHA384 (0x1302)
Key Length: 32 bytes (AES-256)
Hash: SHA-384
```

### BearDog/Songbird Connection (from v0.19.0 logs)
```
Protocol: TLSv1.3
Cipher: TLS_AES_128_GCM_SHA256 (0x1301)
Key Length: 16 bytes (AES-128)
Hash: SHA-256
```

### Analysis

**Why This Matters**:
- ❌ **Cannot directly compare traffic secrets** (different cipher suites)
- ✅ **Both are valid TLS 1.3 cipher suites**
- ⚠️  **Shows different cipher negotiation behavior**

**Likely Cause**:
1. **ClientHello Differences**:
   - Python/OpenSSL sends more cipher suites in preference order
   - BearDog/Songbird may be sending fewer cipher suites
   - Server picks the first match from its own preference list

2. **Server Preference**:
   - `example.com` prefers AES-256-GCM over AES-128-GCM
   - If AES-256 is offered, server will choose it
   - If only AES-128 is offered, server will use that

---

## 📊 CAPTURED KEYS

### Python/OpenSSL (TLS_AES_256_GCM_SHA384)

**Client Random**:
```
10531fb10d47718b94c814e01226c0dbbb744dcb79b879148f7c4c5e00821f5f
```

**CLIENT_HANDSHAKE_TRAFFIC_SECRET** (48 bytes for SHA-384):
```
37a231a35d70d08d82abda2f89da871383f57c60cbd7378ba8c8777ed2154922347de1c967e80a62e90351798068c849
```

**SERVER_HANDSHAKE_TRAFFIC_SECRET** (48 bytes for SHA-384):
```
9e43724ce21c47051b2cfd80a680075a06df12b14740a39c6e9477572f950faa42dd30057d2aabafcfdb3f710160f07d
```

**CLIENT_TRAFFIC_SECRET_0** (48 bytes for SHA-384):
```
b980d8faa7cc6574dd1ea4a10fe782ca068969ee3fbbad4a570684cde5f2e0b99fe74ba0c8d11c1cfb379d2257eab401
```

**SERVER_TRAFFIC_SECRET_0** (48 bytes for SHA-384):
```
694a4bf7687200017c465d45f7fca4db088962923ebcb4ac6882189dcc0abe67b941a6a86e8a0262c739ee177d89e366
```

### BearDog/Songbird (TLS_AES_128_GCM_SHA256)

**CLIENT_TRAFFIC_SECRET_0** (32 bytes for SHA-256):
```
af38bd1558833132c711baf130b416c12992205557af3fa5e1286d8ead73699a
```

**SERVER_TRAFFIC_SECRET_0** (32 bytes for SHA-256):
```
4eebb0c23f26bec0a2545bcacb48d34230b6690148564731ce2a523277630bbe
```

**Transcript Hash** (32 bytes for SHA-256):
```
fb27b3a2bbd8d422ae5868fbaf5f9cbcf4aa4d34cdc05c22ed309aef975fed25
```

**Master Secret** (first 16 bytes):
```
8dfabcf4eccfef61756c064ee445357f
```

---

## 🎯 REVISED VALIDATION STRATEGY

### Option 1: Force Same Cipher Suite ✅ RECOMMENDED

**Action**: Modify Songbird to prefer TLS_AES_256_GCM_SHA384

**Benefit**: Can then directly compare with Python/OpenSSL

**Implementation**:
- Update Songbird's ClientHello to list AES-256-GCM first
- Or configure to only send AES-256-GCM
- Capture new BearDog logs with 0x1302 cipher suite
- Compare traffic secrets byte-for-byte

### Option 2: Manual HKDF Validation ✅ CURRENT BEST

**Action**: Manually compute keys using BearDog's inputs

**Process**:
1. Use BearDog's captured values:
   - Pre-master secret (shared secret from ECDH)
   - Client random
   - Server random
   - Transcript hash
   - Cipher suite: 0x1301

2. Manually derive using Python/Rust:
   - Early Secret
   - Handshake Secret
   - Master Secret
   - Application Traffic Secrets

3. Compare with BearDog's output

**Benefit**: Validates BearDog's HKDF implementation directly

### Option 3: RFC 8448 Test Vectors ✅ DEFINITIVE

**Action**: Use RFC 8448 known values

**Benefit**: 
- Independent of server behavior
- Known correct answers
- Can validate entire key schedule

---

## 🔧 NEXT STEPS

### Immediate (30 minutes)

1. **Extract Full Values from BearDog Logs**
   - Need: pre_master_secret (shared secret)
   - Need: client_random
   - Need: server_random
   - Have: transcript_hash ✅
   - Have: CLIENT_TRAFFIC_SECRET_0 ✅
   - Have: SERVER_TRAFFIC_SECRET_0 ✅

2. **Create Python HKDF Validator**
   - Implement RFC 8446 key schedule
   - Use BearDog's inputs
   - Compare outputs

3. **Document Results**
   - Match or mismatch
   - If mismatch, identify where in key schedule

### Short-term (1 hour)

4. **Force Cipher Suite Match**
   - Update Songbird to prefer AES-256-GCM
   - Retest with Python/OpenSSL
   - Direct comparison

5. **RFC 8448 Validation**
   - Implement test cases
   - Validate BearDog against known values

---

## 📋 FINDINGS SO FAR

### ✅ Good News

1. **Both connections successful** - TLS 1.3 handshake works!
2. **Valid cipher suites** - Both 0x1301 and 0x1302 are correct
3. **Key lengths correct** - 32 bytes for SHA-256, 48 bytes for SHA-384
4. **Secrets derived** - BearDog is generating traffic secrets

### ⚠️  To Investigate

1. **Cipher negotiation** - Why different cipher suites?
2. **HKDF correctness** - Does BearDog's implementation match RFC 8446?
3. **Cross-validation** - Need apples-to-apples comparison

### ❌ Blockers

1. **Cannot compare directly** - Different cipher suites, different keys
2. **Missing full inputs** - Need pre_master_secret for manual validation

---

## 🎯 VALIDATION PLAN UPDATE

### Phase 1: Get Missing Values from BearDog ✅

**Action**: Check BearDog v0.19.0 logs for:
- `pre_master_secret` (ECDH shared secret)
- `client_random` (32 bytes)
- `server_random` (32 bytes)

**Location**: `/tmp/neural-v0.19.0-OUTPUT.log`

### Phase 2: Manual HKDF Computation ✅

**Action**: Python script to:
1. Read BearDog's inputs
2. Implement RFC 8446 key schedule
3. Derive CLIENT_TRAFFIC_SECRET_0
4. Compare with BearDog's output

**Success Criteria**:
- ✅ Exact match → BearDog's HKDF is correct!
- ❌ Mismatch → Debug BearDog's implementation

### Phase 3: RFC 8448 Validation ✅

**Action**: Test against known values

**Success Criteria**:
- ✅ All test vectors pass → Production-ready!

---

## 🚀 LET'S CONTINUE

Next step: **Extract missing values from BearDog logs** for manual validation!

**ETA**: 10 minutes to extract values + 20 minutes to write Python validator = 30 minutes total

---

**Status**: In Progress 🔄  
**Next**: Extract full inputs from BearDog v0.19.0 logs  
**Goal**: Manual HKDF validation to prove correctness  

