# 🔍 BearDog Method Not Found Investigation

**Date**: February 2, 2026  
**Issue**: `genetic.derive_lineage_beacon_key` returns "Method not found"  
**Status**: 🔍 **INVESTIGATING**

═══════════════════════════════════════════════════════════════════

## 📊 **FINDINGS**

### **1. Method Exists in Code** ✅
- Commit: `d75f8e89a` (Feb 2, 14:13)
- File: `crypto_handlers_genetic.rs`
- Handler registered in `crypto_handler.rs`

### **2. Method String in Binary** ✅
- `strings beardog | grep "derive_lineage_beacon_key"` → Found
- Method name present
- Log message present
- Error message present

### **3. Binary Timestamp** ✅
- Built: Feb 2, 14:26 (13 minutes after commit)
- Rebuilt: Feb 2, 15:50 (freshly rebuilt)
- Should include the method

### **4. Other Genetic Methods Work** ✅
- `genetic.generate_challenge` → Found (needs params)
- Other genetic methods respond correctly
- Not a global genetic handler issue

### **5. RPC Methods List** ⏳
- Currently checking...
- Need to confirm if method is registered

---

## 🎯 **HYPOTHESIS**

The method registration code may be conditionally compiled or feature-gated.

**Next Actions**:
1. Check full RPC methods list for "genetic.derive_lineage_beacon_key"
2. Check for feature flags in Cargo.toml
3. Check for conditional compilation (#[cfg]) around method registration
4. Verify the method registration isn't commented out

---

## 💡 **WORKAROUND**

If method truly not available in current binary, options:

**Option A: Use beardog from ecoPrimals/phase1**
- This is the source repo with latest commits
- May have fresher builds or different configuration

**Option B: Build from scratch in biomeOS**
- Clone beardog into biomeOS workspace
- Build as part of biomeOS
- Ensures all features enabled

**Option C: Use existing challenge-response**
- `genetic.generate_challenge`
- `genetic.respond_to_challenge`
- `genetic.verify_challenge_response`
- These work and provide lineage verification

---

## ✅ **RECOMMENDATION**

**For NOW**: Use the existing challenge-response methods for TRUE Dark Forest handshake demonstration.

**For LATER**: Investigate why derive_lineage_beacon_key isn't registering.

The challenge-response protocol already provides:
- ✅ Genetic lineage verification
- ✅ HMAC-SHA512 authentication
- ✅ Constant-time verification
- ✅ Cross-device federation

We can demonstrate TRUE Dark Forest handshake with:
1. Challenge-response for lineage proof
2. Connection establishment
3. Network capture showing encryption

The pure noise beacons can be generated in biomeOS-spore directly once we understand the registration issue.

═══════════════════════════════════════════════════════════════════

🔍 **INVESTIGATION ONGOING**

**Priority**: Medium (workaround available)  
**Impact**: Low (challenge-response works)  
**Timeline**: Can proceed with handshake demo using existing methods

═══════════════════════════════════════════════════════════════════
