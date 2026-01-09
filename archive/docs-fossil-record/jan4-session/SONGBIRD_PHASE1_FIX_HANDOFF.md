# 🔧 Songbird Phase 1 Fix - Handoff to Team

**Date**: January 6, 2026  
**Priority**: **CRITICAL** - Blocks federation  
**Status**: Root cause identified, fix required  

---

## 🎯 Problem Summary

**Current Situation**: Federation is blocked because Songbird cannot parse BearDog's Phase 1 trust responses.

**Error**: `"invalid type: integer \`0\`, expected a string"`

**Root Cause Found**: The custom `TrustLevel` deserializer (that accepts both int and string) exists but **is not being used**!

---

## 🔍 Root Cause Analysis

### What We Found:

1. ✅ **Custom deserializer EXISTS** in `crates/songbird-types/src/trust.rs`:
   ```rust
   impl<'de> Deserialize<'de> for TrustLevel {
       // Accepts BOTH integer (0-3) and string ("none", "limited", etc.)
   }
   ```

2. ❌ **But it's NOT USED** because `TrustEvaluationResponse` uses `String`:
   ```rust
   // In crates/songbird-universal/src/trust_types.rs:36
   pub struct TrustEvaluationResponse {
       pub decision: String,
       pub trust_level: String,  // ❌ This bypasses the custom deserializer!
       //               ^^^^^^ Should be TrustLevel enum!
   }
   ```

3. 🔁 **The Chain**:
   ```
   BearDog sends: {"trust_level": 1}
        ↓
   Songbird tries to deserialize into TrustEvaluationResponse
        ↓
   Field is `trust_level: String` (expects string)
        ↓
   Serde tries to deserialize integer 1 as String
        ↓
   ❌ ERROR: "expected a string"
   
   The custom TrustLevel deserializer NEVER RUNS because the field is String!
   ```

---

## ✅ Required Fix

### Step 1: Change the field type

**File**: `crates/songbird-universal/src/trust_types.rs`

```rust
// BEFORE (line 30-36):
pub struct TrustEvaluationResponse {
    pub decision: String,
    pub trust_level: String,  // ❌ Wrong type!
    pub reason: String,
    // ...
}

// AFTER:
use songbird_types::TrustLevel;  // ← Add import at top

pub struct TrustEvaluationResponse {
    pub decision: String,
    pub trust_level: TrustLevel,  // ✅ Use enum (with Phase 1 deserializer)
    pub reason: String,
    // ...
}
```

### Step 2: Fix test code

**File**: `crates/songbird-universal/src/trust_types.rs` (tests at bottom)

```rust
// BEFORE (line 145, 160, etc.):
TrustEvaluationResponse {
    decision: "auto_accept".to_string(),
    trust_level: "high".to_string(),  // ❌ String literal
    //           ^^^^^^^^^^^^^^^^^^
    reason: "test".to_string(),
}

// AFTER:
use songbird_types::TrustLevel;  // ← Add to test imports

TrustEvaluationResponse {
    decision: "auto_accept".to_string(),
    trust_level: TrustLevel::Highest,  // ✅ Use enum variant
    //           ^^^^^^^^^^^^^^^^^^^
    reason: "test".to_string(),
}
```

**Mapping**:
- `"high"` → `TrustLevel::Highest`
- `"medium"` → `TrustLevel::Elevated`
- `"low"` → `TrustLevel::Limited`
- `"none"` → `TrustLevel::None`

### Step 3: Fix any other usages

Search the codebase for places that access `response.trust_level` and expect a string:

```bash
cd crates/songbird-orchestrator
grep -r "trust_level" --include="*.rs" | grep -v "test"
```

**Common patterns to fix**:
```rust
// BEFORE:
if response.trust_level == "high" { ... }

// AFTER:
if response.trust_level == TrustLevel::Highest { ... }

// Or use the convenience methods:
response.trust_level.is_highest()
```

---

## 🧪 Verification

After fixing, these should work:

### Test 1: Integer format (BearDog primary)
```json
{
  "decision": "auto_accept",
  "trust_level": 1,  // ← Integer!
  "reason": "same_genetic_family"
}
```

### Test 2: String format (backward compat)
```json
{
  "decision": "auto_accept",
  "trust_level": "limited",  // ← String!
  "reason": "same_genetic_family"
}
```

### Test 3: Full BearDog Phase 1 response
```json
{
  "trust_level": 1,
  "trust_level_name": "limited",
  "capabilities": {
    "allowed": ["birdsong/*", "coordination/*"],
    "denied": ["data/*", "commands/*"]
  },
  "metadata": {
    "policy_version": 1,
    "evaluation_method": "genetic_family_match"
  }
}
```

**All three should deserialize successfully!**

---

## 📋 Files to Modify

1. **`crates/songbird-universal/src/trust_types.rs`**
   - Line 8: Add `use songbird_types::TrustLevel;`
   - Line 36: Change `trust_level: String` → `trust_level: TrustLevel`
   - Lines 145, 160, 217, 241, 264, 287: Update test code to use `TrustLevel` enum

2. **`crates/songbird-orchestrator/src/trust/peer_trust.rs`** (if needed)
   - Check for string comparisons like `response.trust_level == "high"`
   - Replace with enum comparisons: `response.trust_level == TrustLevel::Highest`

3. **`crates/songbird-orchestrator/src/security_capability_client.rs`** (if needed)
   - Same as above

---

## 🎯 Expected Result

**Before fix**:
```
❌ BearDog: {"trust_level": 1}
❌ Songbird: Parse error: "expected a string"
❌ Federation: BLOCKED
```

**After fix**:
```
✅ BearDog: {"trust_level": 1}
✅ Songbird: Parses as TrustLevel::Limited
✅ Federation: WORKING!
```

---

## 💡 Why This Happened

The Phase 1 custom deserializer was added to `TrustLevel` enum, but the **struct that uses it** (`TrustEvaluationResponse`) was never updated from `String` to `TrustLevel`.

**Analogy**: You built a fancy translator (custom deserializer) but never told anyone to use it! The old code is still using the basic translator (String deserialization) which doesn't understand the new format.

---

## ⏱️ Estimated Time

- **Fix**: 15 minutes (change types, update tests)
- **Build**: 2 minutes
- **Test**: 5 minutes
- **Total**: ~25 minutes

---

## 🚀 Testing After Fix

```bash
# 1. Build
cargo build --release --bin songbird-orchestrator

# 2. Run trust tests
cargo test --package songbird-types trust_level
cargo test --package songbird-universal trust_types

# 3. Copy binary
cp target/release/songbird-orchestrator /path/to/plasmidBin/songbird

# 4. Deploy and verify federation
# (biomeOS team will handle deployment)
```

---

## 📞 Questions?

**Q**: Why didn't the custom deserializer work?  
**A**: It was never called because the field type was `String`, not `TrustLevel`.

**Q**: Will this break anything?  
**A**: Only test code that uses string literals. Production code should work better!

**Q**: What about backward compatibility?  
**A**: The custom deserializer handles both formats, so old responses still work!

---

## ✅ Acceptance Criteria

- [ ] `TrustEvaluationResponse.trust_level` is type `TrustLevel` (not `String`)
- [ ] All tests pass
- [ ] Can parse BearDog integer responses: `{"trust_level": 1}`
- [ ] Can parse string responses: `{"trust_level": "limited"}`
- [ ] Federation works with BearDog Phase 1

---

**Bottom Line**: Change ONE field type from `String` to `TrustLevel`, update some test code, and federation will work! The custom deserializer already exists and works perfectly—it just wasn't being used.

🎯 **Simple fix, huge impact!**

---

_Handoff from: biomeOS team_  
_Date: January 6, 2026_  
_Status: Ready for Songbird team to execute_

