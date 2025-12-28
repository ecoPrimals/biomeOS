# 05 - benchScale Validation: Production Testing

**Duration**: 10 minutes  
**Prerequisites**: benchScale VM infrastructure

---

## Overview

Use `ecoPrimals/primalsTools/benchScale/` to deploy and validate NestGate at scale across multiple VMs.

**What it demonstrates**:
- Multi-VM deployment
- Load testing (1000s of requests/sec)
- Federation at scale (10+ towers)
- Chaos engineering
- Performance validation

---

## Philosophy

> "Demos show what's possible.  
>  benchScale proves what's reliable."

---

## Run the Demo

```bash
cd showcase/01-nestgate/05-benchscale-validation

# Deploy 5-tower federation
./deploy-federation.sh --towers 5

# Run validation
./validate.sh
```

---

## Key Metrics

### Performance Targets
- **Throughput**: 10,000 requests/sec
- **Latency**: p99 < 50ms
- **Availability**: 99.99%
- **Federation**: 10+ towers
- **Replication**: < 5 min sync

### Chaos Scenarios
1. Random tower failures (20% failure rate)
2. Network partitions (split-brain)
3. Disk failures (ZFS resilience)
4. Load spikes (10x normal)
5. Slow networks (100ms latency)

---

## Demo Flow

1. Deploy 5-tower federation (benchScale)
2. Generate load (10K req/sec)
3. Measure baseline performance
4. Inject chaos (tower failures)
5. Validate recovery
6. Report metrics

---

**Philosophy**: *"Trust through testing. Sovereignty through validation."*

