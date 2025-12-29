# Phase 1: VM Provisioning Validation

**Goal**: Get a stable, replicable VM testing substrate using benchScale + agentReagents

**Status**: In Progress 🔧

---

## Strategy

### Step 1: Understand benchScale
- What commands does it support?
- What format does it expect?
- Does it use topologies or direct VM creation?

### Step 2: Use agentReagents Template
- Template: `rustdesk-ubuntu-22.04-template.qcow2` (2.9GB)
- Location: `../../primalTools/agentReagents/images/templates/`
- Benefit: 40x faster than cloud image

### Step 3: Simple Test
- Create 1 VM manually
- Verify it boots
- Verify SSH access
- Then scale to 2 VMs

### Step 4: Validate
- VMs boot successfully
- VMs get IPs
- VMs are SSH accessible
- VMs can ping each other

---

## Current Issue

benchScale error:
```
ERROR benchscale: Failed to create lab: IO error: No such file or directory (os error 2)
```

**Root cause**: Unknown - need to check:
1. benchScale's expected command format
2. Topology file format vs direct VM creation
3. Whether benchScale needs specific paths

---

## Phase 1 Success Criteria

✅ Create 2 VMs using benchScale + agentReagents  
✅ VMs boot successfully  
✅ VMs get IPs (DHCP)  
✅ VMs are SSH accessible  
✅ VMs can communicate  

**Once this works**: Move to Phase 2 (biomeOS deployment)

---

## Approach

**Keep it simple**:
1. Use benchScale's actual API (check docs)
2. Use agentReagents template directly
3. Minimal configuration
4. Validate each VM individually first

**No fancy stuff yet**:
- No cloud-init complexity
- No Songbird yet
- Just: Create VMs → Boot → SSH → Done

---

*Let's validate the substrate first!* 🔧
