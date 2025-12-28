# benchScale Validation Topologies for BiomeOS

**Location**: `validation/benchscale-topologies/`  
**Purpose**: Validation topologies for benchScale testing  
**Architecture**: mDNS/UDP ecosystem coordination (NO HTTP between primals!)  
**Updated**: December 28, 2025  

---

## Overview

These YAML topologies are used with `benchScale` as a validation tool for BiomeOS deployments. They validate the correct mDNS/UDP ecosystem architecture.

---

## Available Topologies

### **1. `rootpulse-local.yaml`**
**Description**: Single-node RootPulse niche validation  
**Purpose**: Test primal coordination via mDNS/UDP without federation complexity  
**Key Features**:
- Validates mDNS discovery (not HTTP!)
- Confirms UDP coordination between primals
- Tests that Songbird has NO HTTP endpoint (correct!)
- Validates standalone HTTP is only for external access
- 7 validation tests including security checks

**Usage**:
```bash
cd /home/eastgate/Development/ecoPrimals/phase2/biomeOS
./validate-with-benchscale.sh validation/benchscale-topologies/rootpulse-local.yaml
```

### **2. `rootpulse-federation.yaml`**
**Description**: 3-tower RootPulse federation with geographic distribution  
**Purpose**: Validate mDNS/UDP federation across multiple towers  
**Key Features**:
- 3 towers (NA, EU, Asia) with realistic latencies
- mDNS discovery across federation
- UDP coordination (NO HTTP!)
- P2P commit propagation
- Lineage verification across federation
- Performance benchmarking

**Towers**:
- `tower-na`: 10ms latency (local)
- `tower-eu`: 80ms latency (transatlantic)
- `tower-asia`: 150ms latency (transpacific)

**Usage**:
```bash
./validate-with-benchscale.sh validation/benchscale-topologies/rootpulse-federation.yaml
```

### **3. `usb-federation-test.yaml`**
**Description**: 2-VM federation test with USB deployment  
**Purpose**: Validate complete deployment pipeline before NUC  
**Key Features**:
- Uses actual USB package (biomeos-20251228-163320.tar.gz)
- Tests real deployment process
- mDNS discovery between towers
- UDP coordination validation
- Security checks (NO HTTP between primals!)
- Expected: 15/15 E2E tests (100%)

**Usage**:
```bash
./validate-with-benchscale.sh validation/benchscale-topologies/usb-federation-test.yaml
```

---

## Architectural Validation

All topologies validate the correct BiomeOS architecture:

### **✅ What's Tested:**

1. **mDNS Discovery**:
   - Primals discover each other via mDNS
   - No hardcoded endpoints
   - Automatic service discovery

2. **UDP Coordination**:
   - Inter-primal communication via UDP
   - Songbird coordination via UDP messages
   - No HTTP between primals

3. **Security**:
   - Verify NO HTTP used for ecosystem coordination
   - Confirm mDNS local network only
   - Validate lineage-based authentication
   - Check UDP application-layer encryption

4. **Standalone HTTP**:
   - HTTP only for external/human access
   - NestGate standalone API working
   - BearDog CLI working
   - Songbird has NO HTTP (correct!)

### **❌ What's NOT Allowed:**

1. **HTTP Between Primals**:
   - Tests explicitly check for NO HTTP coordination
   - Any HTTP between primals = test failure
   - This is a security requirement!

2. **Hardcoded Endpoints**:
   - No hardcoded primal addresses
   - All discovery must be via mDNS
   - Runtime discovery only

3. **Songbird HTTP Endpoint**:
   - Songbird must NOT have HTTP endpoint
   - Pure mDNS/UDP coordination only
   - Tests validate this architecture

---

## YAML Structure

### **Common Sections:**

```yaml
metadata:
  name: topology-name
  version: "2.0"
  architecture: "mDNS/UDP ecosystem coordination"

network:
  mdns_enabled: true
  multicast_dns: true

nodes:
  - name: node-name
    env:
      PRIMAL_DISCOVERY: mdns
      COORDINATION_PROTOCOL: udp
      STANDALONE_HTTP: enabled  # External access only
      ECOSYSTEM_HTTP: disabled  # NO HTTP between primals!
    
    services:
      - name: songbird
        mode: ecosystem_only  # NO HTTP!
        discovery: mdns
        coordination: udp

validation:
  tests:
    - name: mdns_discovery_working
      command: avahi-browse -t _songbird._tcp ...
    
    - name: no_http_coordination
      command: ! netstat ... | grep HTTP ...

security:
  checks:
    - no_http_between_primals: true
    - mdns_discovery_only: true
    - udp_coordination_only: true
```

---

## Integration with benchScale

### **How BiomeOS Uses benchScale:**

1. **Validation Tool** (not deployment target)
   - benchScale validates BiomeOS functionality
   - Creates test environments
   - Runs automated validation
   - Collects results and logs

2. **Topology-Driven Testing**
   - YAML defines infrastructure
   - benchScale creates VMs/containers
   - BiomeOS deploys into environment
   - Tests run automatically

3. **Agnostic Capability Pattern**
   - benchScale discovers BiomeOS capabilities
   - No hardcoded assumptions
   - Runtime adaptation
   - Same pattern BiomeOS uses for primals!

### **Validation Script:**

```bash
#!/bin/bash
# validate-with-benchscale.sh

TOPOLOGY=$1
BENCHSCALE=$(realpath ../../primalTools/benchscale/target/release/benchscale)

# Use benchScale as validation tool
$BENCHSCALE topology validate $TOPOLOGY
$BENCHSCALE topology deploy $TOPOLOGY
$BENCHSCALE topology test $TOPOLOGY
$BENCHSCALE topology report $TOPOLOGY
```

---

## Success Criteria

### **All Topologies Must:**

✅ Discover all primals via mDNS  
✅ Coordinate via UDP (not HTTP!)  
✅ Pass security checks (no HTTP coordination)  
✅ Maintain lineage enforcement  
✅ Pass E2E tests (15/15 expected)  
✅ Demonstrate ecosystem network effect  

### **Security Requirements:**

✅ NO HTTP between primals (ecosystem mode)  
✅ mDNS local network only  
✅ UDP application-layer encryption  
✅ Lineage-based authentication  
✅ Zero-configuration security  

---

## Why These Topologies Are Here

**User Insight**: "benchScale YAMLs should be in biomeOS as we call benchScale as a validation tool"

**Reasoning**:
1. benchScale is a **validation tool** for BiomeOS
2. These topologies **validate** BiomeOS functionality
3. They belong with what they validate (biomeOS/)
4. They describe BiomeOS deployment scenarios
5. They're part of BiomeOS test suite

**Benefits**:
- Co-located with code being tested
- Version controlled with BiomeOS
- Easy to update when BiomeOS evolves
- Clear relationship: BiomeOS → benchScale (validation)

---

## Next Steps

1. Run local validation:
   ```bash
   ./validate-with-benchscale.sh validation/benchscale-topologies/rootpulse-local.yaml
   ```

2. Run federation validation:
   ```bash
   ./validate-with-benchscale.sh validation/benchscale-topologies/rootpulse-federation.yaml
   ```

3. Run USB deployment validation:
   ```bash
   ./validate-with-benchscale.sh validation/benchscale-topologies/usb-federation-test.yaml
   ```

4. Deploy to NUC (after validation passes):
   ```bash
   AUTO_CONFIRM=1 ./quick-usb.sh
   # Boot NUC from USB
   ```

---

## Architecture Validation

These topologies validate that BiomeOS correctly implements:

- **mDNS/UDP for ecosystem** (network effect!)
- **HTTP only for standalone** (external access)
- **Songbird ecosystem-only** (no HTTP!)
- **Security through decentralization** (not centralization)
- **Zero-configuration discovery** (no hardcoding)
- **Lineage-based trust** (self-sovereign)

🔒 **Secure by design. Validated by benchScale.**

