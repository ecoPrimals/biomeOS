# 🧪 Trust Evaluation Test Commands

**Date**: January 7, 2026  
**Purpose**: Comprehensive test commands for verifying trust evaluation at each phase  
**Status**: 🎯 **PRODUCTION READY**

---

## 📋 Quick Reference

| Phase | What to Test | Expected Result |
|-------|--------------|-----------------|
| **Pre-Phase 1** | Schema mismatch | ❌ Parse error |
| **Phase 1** | Dual representation | ✅ Both int and string |
| **Phase 2** | Custom policies | ✅ Policy-defined capabilities |
| **Phase 3** | Contact keys | ✅ Shared secret established |

---

## 🔬 Pre-Phase 1: Current State (Schema Mismatch)

### **Test 1: BearDog Trust Evaluation**

```bash
# Test trust evaluation response format
echo '{"jsonrpc":"2.0","method":"trust.evaluate_peer","params":{"peer_id":"tower2","peer_family":"nat0"},"id":1}' | \
  nc -U /tmp/beardog-nat0-tower1.sock | jq

# Current Response (Pre-Phase 1):
# {
#   "result": {
#     "trust_level": 0,              // ⚠️ INTEGER only
#     "reason": "unknown_family",
#     "peer_id": "tower2",
#     "peer_family": null,
#     "our_family": "nat0"
#   },
#   "id": 1
# }
```

### **Test 2: Songbird Logs (Parse Error)**

```bash
# Monitor Songbird logs for parse errors
tail -f /tmp/primals/*songbird*.log | grep -E '(parse|error|trust)'

# Expected Error (Pre-Phase 1):
# ⚠️ Failed to parse trust evaluation response: invalid type: integer `0`, expected a string
# ❌ Security provider rejects peer tower2
# ⚠️ Trust Decision: REJECT for 'tower2'
```

### **Test 3: Federation Status (Blocked)**

```bash
# Check if peers are added
echo '{"jsonrpc":"2.0","method":"discovery.list_peers","id":1}' | \
  nc -U /tmp/songbird-nat0-tower1.sock | jq '.result.total'

# Expected (Pre-Phase 1):
# 0  // No peers added (all rejected due to parse error)
```

---

## ✅ Phase 1: Dual Representation

### **Test 1: BearDog Response Format**

```bash
# Test Phase 1 trust evaluation
echo '{"jsonrpc":"2.0","method":"trust.evaluate_peer","params":{"peer_id":"tower2","peer_family":"nat0"},"id":1}' | \
  nc -U /tmp/beardog-nat0-tower1.sock | jq

# Expected Response (Phase 1):
# {
#   "result": {
#     "trust_level": 1,                    // ✅ INTEGER (compact)
#     "trust_level_name": "limited",       // ✅ STRING (readable)
#     "reason": "same_genetic_family",
#     "peer_id": "tower2",
#     "peer_family": "nat0",
#     "our_family": "nat0",
#     "capabilities": {                    // ✅ NEW: Capability hints
#       "allowed": ["birdsong/*", "coordination/*", "health"],
#       "denied": ["data/*", "commands/*", "keys/*"]
#     }
#   },
#   "id": 1
# }
```

**Validation Checks**:
```bash
# Check both formats present
RESPONSE=$(echo '{"jsonrpc":"2.0","method":"trust.evaluate_peer","params":{"peer_id":"tower2","peer_family":"nat0"},"id":1}' | nc -U /tmp/beardog-nat0-tower1.sock)

echo "$RESPONSE" | jq '.result.trust_level' | grep -E '^[0-9]+$'
# Expected: 1 (integer)

echo "$RESPONSE" | jq -r '.result.trust_level_name' | grep -E '^[a-z]+$'
# Expected: limited (string)

echo "$RESPONSE" | jq '.result.capabilities'
# Expected: {"allowed":[...],"denied":[...]}
```

### **Test 2: Songbird Parsing Success**

```bash
# Monitor Songbird logs for successful parsing
tail -f /tmp/primals/*songbird*.log | grep -E '(trust|parse|accept)'

# Expected (Phase 1):
# 📡 JSON-RPC client initialized for socket: /tmp/beardog-nat0-tower1.sock
# 🔍 Evaluating trust for peer: tower2
# ✅ Trust level: limited (same genetic family)  // ✅ NO parse error!
# ✅ Peer 'tower2' accepted - trust level 1
# 🤝 Peer 'tower2' joined federation
```

### **Test 3: Federation Established**

```bash
# Check peers added
echo '{"jsonrpc":"2.0","method":"discovery.list_peers","id":1}' | \
  nc -U /tmp/songbird-nat0-tower1.sock | jq

# Expected (Phase 1):
# {
#   "result": {
#     "total": 1,                         // ✅ Peers added!
#     "peers": [{
#       "peer_id": "tower2",
#       "trust_level": "limited",         // ✅ String format
#       "family_id": "nat0",
#       "capabilities": ["birdsong/*", "coordination/*"]
#     }]
#   }
# }
```

### **Test 4: Capability Validation**

```bash
# Test each trust level's capabilities

# Level 0: Anonymous (no trust)
echo '{"jsonrpc":"2.0","method":"trust.evaluate_peer","params":{"peer_id":"unknown","peer_family":"other"},"id":1}' | \
  nc -U /tmp/beardog-nat0-tower1.sock | jq '.result'

# Expected:
# {
#   "trust_level": 0,
#   "trust_level_name": "anonymous",
#   "capabilities": {
#     "allowed": [],
#     "denied": ["*"]
#   }
# }

# Level 1: Limited (same family)
echo '{"jsonrpc":"2.0","method":"trust.evaluate_peer","params":{"peer_id":"tower2","peer_family":"nat0"},"id":1}' | \
  nc -U /tmp/beardog-nat0-tower1.sock | jq '.result.capabilities'

# Expected:
# {
#   "allowed": ["birdsong/*", "coordination/*", "health"],
#   "denied": ["data/*", "commands/*", "keys/*"]
# }
```

### **Test 5: No Parse Errors**

```bash
# Verify no parse errors in logs
grep -i "parse.*error\|failed.*parse" /tmp/primals/*songbird*.log

# Expected (Phase 1):
# (no output) // ✅ Zero parse errors!
```

---

## 🔐 Phase 2: Configurable Policies (Future)

### **Test 1: Custom Policy Loading**

```bash
# Get current trust policy
echo '{"jsonrpc":"2.0","method":"trust.get_policy","id":1}' | \
  nc -U /tmp/beardog-nat0-tower1.sock | jq

# Expected Response (Phase 2):
# {
#   "result": {
#     "family_id": "nat0",
#     "version": 1,
#     "signature": "...",              // ✅ Genetic seed signature
#     "tiers": [
#       {
#         "index": 0,
#         "name": "none",
#         "allowed_capabilities": [],
#         "denied_capabilities": ["*"]
#       },
#       {
#         "index": 1,
#         "name": "limited",
#         "allowed_capabilities": ["birdsong/*", "coordination/*"],
#         "denied_capabilities": ["data/*", "commands/*"],
#         "requirements": [{"type": "SameFamily"}]
#       }
#     ]
#   }
# }
```

### **Test 2: Custom Tier Evaluation**

```bash
# Test with custom policy (example: organization has 5 tiers)
echo '{"jsonrpc":"2.0","method":"trust.evaluate_peer","params":{"peer_id":"tower2","peer_family":"nat0","evidence":[{"type":"SameFamily"}]},"id":1}' | \
  nc -U /tmp/beardog-nat0-tower1.sock | jq

# Expected (Phase 2 with custom policy):
# {
#   "trust_level": 1,
#   "trust_level_name": "limited",    // Or custom name like "basic_member"
#   "reason": "meets_tier_requirements",
#   "capabilities": {...},            // Policy-defined
#   "elevation_path": {               // ✅ How to get to next tier
#     "next_tier": 2,
#     "requirements": ["HumanApproval"],
#     "method": "user_consent_ui"
#   }
# }
```

### **Test 3: Policy Signature Verification**

```bash
# Verify policy signature with genetic seed
echo '{"jsonrpc":"2.0","method":"trust.verify_policy","params":{"family_id":"nat0","policy_version":1},"id":1}' | \
  nc -U /tmp/beardog-nat0-tower1.sock | jq

# Expected (Phase 2):
# {
#   "result": {
#     "verified": true,                   // ✅ Signature valid
#     "signer": "family_progenitor",
#     "signed_at": "2026-01-07T00:00:00Z"
#   }
# }
```

---

## 🔑 Phase 3: Contact Key Exchange (Future)

### **Test 1: Initiate Exchange**

```bash
# Initiate contact key exchange
echo '{"jsonrpc":"2.0","method":"contact.initiate_exchange","params":{"peer_id":"tower2"},"id":1}' | \
  nc -U /tmp/beardog-nat0-tower1.sock | jq

# Expected Response (Phase 3):
# {
#   "result": {
#     "exchange_id": "...",
#     "our_public_key": "...",        // ✅ Ephemeral DH public key
#     "lineage_proof": "...",         // ✅ Genetic signature
#     "timestamp": "2026-01-07T..."
#   }
# }
```

### **Test 2: Complete Exchange**

```bash
# Complete exchange with peer's public key
echo '{"jsonrpc":"2.0","method":"contact.complete_exchange","params":{"exchange_id":"...","peer_public_key":"..."},"id":1}' | \
  nc -U /tmp/beardog-nat0-tower1.sock | jq

# Expected (Phase 3):
# {
#   "result": {
#     "shared_secret_established": true,  // ✅ ECDH complete
#     "nat_key_available": true,          // ✅ NAT traversal key derived
#     "p2p_key_available": true,          // ✅ P2P encryption key derived
#     "contact_key_id": "..."
#   }
# }
```

### **Test 3: Trust Evaluation with Contact Key**

```bash
# Evaluate trust with contact key evidence
echo '{"jsonrpc":"2.0","method":"trust.evaluate_peer","params":{"peer_id":"tower2","peer_family":"nat0","evidence":[{"type":"ContactKeyEstablished","key_id":"..."}]},"id":1}' | \
  nc -U /tmp/beardog-nat0-tower1.sock | jq

# Expected (Phase 3):
# {
#   "trust_level": 2,                       // ✅ Higher tier!
#   "trust_level_name": "elevated",
#   "reason": "contact_key_established",
#   "capabilities": {
#     "allowed": ["birdsong/*", "coordination/*", "federation/*", "nat_traversal", "p2p_direct"],
#     "denied": ["data/write", "commands/sensitive"]
#   }
# }
```

---

## 🎯 Comprehensive Test Suite

### **Pre-Deployment Checklist**

```bash
#!/bin/bash
# comprehensive-test.sh

echo "🧪 Comprehensive Trust Evaluation Test Suite"
echo "============================================="
echo ""

# Test 1: BearDog Health
echo "Test 1: BearDog Health Check"
HEALTH=$(echo '{"jsonrpc":"2.0","method":"health.check","id":1}' | nc -U /tmp/beardog-nat0-tower1.sock | jq -r '.result.status')
if [ "$HEALTH" = "healthy" ]; then
  echo "✅ BearDog healthy"
else
  echo "❌ BearDog unhealthy: $HEALTH"
  exit 1
fi
echo ""

# Test 2: Dual Representation
echo "Test 2: Trust Level Dual Representation"
RESPONSE=$(echo '{"jsonrpc":"2.0","method":"trust.evaluate_peer","params":{"peer_id":"tower2","peer_family":"nat0"},"id":1}' | nc -U /tmp/beardog-nat0-tower1.sock)

INT_LEVEL=$(echo "$RESPONSE" | jq -r '.result.trust_level')
STR_LEVEL=$(echo "$RESPONSE" | jq -r '.result.trust_level_name')

if [ "$INT_LEVEL" != "null" ] && [ "$STR_LEVEL" != "null" ]; then
  echo "✅ Both integer ($INT_LEVEL) and string ($STR_LEVEL) present"
else
  echo "❌ Missing trust level format"
  echo "Integer: $INT_LEVEL"
  echo "String: $STR_LEVEL"
  exit 1
fi
echo ""

# Test 3: Capabilities Present
echo "Test 3: Capability Hints"
CAPABILITIES=$(echo "$RESPONSE" | jq '.result.capabilities')
if [ "$CAPABILITIES" != "null" ]; then
  echo "✅ Capabilities present:"
  echo "$CAPABILITIES" | jq
else
  echo "❌ Capabilities missing"
  exit 1
fi
echo ""

# Test 4: Songbird Parse Success
echo "Test 4: Songbird Parsing"
sleep 5  # Wait for logs
PARSE_ERRORS=$(grep -c "parse.*error\|failed.*parse" /tmp/primals/*songbird*.log 2>/dev/null || echo 0)
if [ "$PARSE_ERRORS" = "0" ]; then
  echo "✅ Zero parse errors"
else
  echo "❌ Found $PARSE_ERRORS parse errors"
  grep "parse.*error\|failed.*parse" /tmp/primals/*songbird*.log | tail -5
  exit 1
fi
echo ""

# Test 5: Federation Established
echo "Test 5: Federation Status"
PEER_COUNT=$(echo '{"jsonrpc":"2.0","method":"discovery.list_peers","id":1}' | nc -U /tmp/songbird-nat0-tower1.sock | jq -r '.result.total')
if [ "$PEER_COUNT" -gt 0 ]; then
  echo "✅ Federation established: $PEER_COUNT peer(s)"
else
  echo "❌ No peers in federation"
  exit 1
fi
echo ""

echo "🎉 All tests passed!"
echo ""
echo "Federation Summary:"
echo "- BearDog: Healthy and responding"
echo "- Trust format: Dual representation working"
echo "- Capabilities: Present and valid"
echo "- Parsing: No errors"
echo "- Federation: $PEER_COUNT peer(s) connected"
```

**Run the suite**:
```bash
chmod +x comprehensive-test.sh
./comprehensive-test.sh
```

---

## 📊 Performance Tests

### **Latency Test**

```bash
# Measure trust evaluation latency
for i in {1..10}; do
  START=$(date +%s%N)
  echo '{"jsonrpc":"2.0","method":"trust.evaluate_peer","params":{"peer_id":"tower2","peer_family":"nat0"},"id":1}' | nc -U /tmp/beardog-nat0-tower1.sock > /dev/null
  END=$(date +%s%N)
  LATENCY=$(( ($END - $START) / 1000000 ))  # Convert to ms
  echo "Attempt $i: ${LATENCY}ms"
done

# Expected: <10ms for local Unix socket
```

### **Throughput Test**

```bash
# Measure requests per second
COUNT=100
START=$(date +%s)
for i in $(seq 1 $COUNT); do
  echo '{"jsonrpc":"2.0","method":"trust.evaluate_peer","params":{"peer_id":"tower2","peer_family":"nat0"},"id":'$i'}' | nc -U /tmp/beardog-nat0-tower1.sock > /dev/null &
done
wait
END=$(date +%s)
DURATION=$(($END - $START))
RPS=$(($COUNT / $DURATION))
echo "Processed $COUNT requests in ${DURATION}s = ${RPS} RPS"

# Expected: >1000 RPS for local Unix socket
```

---

## 🔍 Debug Commands

### **View Full Trust Evaluation Flow**

```bash
# Terminal 1: Monitor BearDog
tail -f /tmp/primals/*beardog*.log | grep -E '(trust|evaluate)'

# Terminal 2: Monitor Songbird
tail -f /tmp/primals/*songbird*.log | grep -E '(trust|evaluate|parse)'

# Terminal 3: Trigger evaluation
echo '{"jsonrpc":"2.0","method":"trust.evaluate_peer","params":{"peer_id":"tower2","peer_family":"nat0"},"id":1}' | nc -U /tmp/beardog-nat0-tower1.sock | jq
```

### **Dump Trust Store**

```bash
# Get all trust relationships (if implemented)
echo '{"jsonrpc":"2.0","method":"trust.dump_store","id":1}' | \
  nc -U /tmp/beardog-nat0-tower1.sock | jq

# Expected:
# {
#   "result": {
#     "relationships": [
#       {"from": "tower1", "to": "tower2", "level": 1, "established_at": "..."}
#     ]
#   }
# }
```

---

## 📚 Expected Results Summary

| Test | Pre-Phase 1 | Phase 1 | Phase 2 | Phase 3 |
|------|-------------|---------|---------|---------|
| **Integer trust_level** | ✅ | ✅ | ✅ | ✅ |
| **String trust_level_name** | ❌ | ✅ | ✅ | ✅ |
| **Capabilities** | ❌ | ✅ | ✅ (policy) | ✅ (policy) |
| **Parse errors** | ❌ Many | ✅ Zero | ✅ Zero | ✅ Zero |
| **Federation** | ❌ Blocked | ✅ Working | ✅ Working | ✅ Working |
| **Custom policies** | ❌ | ❌ | ✅ | ✅ |
| **Contact keys** | ❌ | ❌ | ❌ | ✅ |

---

**Version**: v1.0  
**Date**: January 7, 2026  
**Status**: 🎯 **PRODUCTION READY**

---

*"Test early, test often, test comprehensively!"* 🧪

