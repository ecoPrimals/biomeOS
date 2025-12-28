# 03 - Multi-Tower Federation

**Demonstrates**: Geographic distribution with automatic coordination  
**Status**: Architecture demonstration  
**Prerequisites**: Songbird (single or multi-tower)  

---

## What This Demonstrates

- Multi-tower deployment patterns
- Cross-tower communication
- Geographic sovereignty
- Load distribution
- Automatic failover

---

## Architecture

```
┌──────────┐    ┌──────────┐    ┌──────────┐
│ Tower US │◄───┤ Tower EU ├───►│ Tower AS │
└──────────┘    └──────────┘    └──────────┘
     │               │               │
  Primals         Primals         Primals
     │               │               │
  ┌──┴──┐        ┌───┴───┐       ┌──┴──┐
  │Bio  │        │ Bio   │       │Bio  │
  │meOS │        │ meOS  │       │meOS │
  └─────┘        └───────┘       └─────┘
```

**Key**: Each tower runs BiomeOS + primals, Songbird coordinates federation

---

## Running the Demo

```bash
bash showcase/02-birdsong-p2p/03-multi-tower/demo.sh
```

---

## Validation with benchScale

This demo is **designed for benchScale validation**:

```bash
# Deploy to multi-VM federation
cd ../primalsTools/benchScale
./scripts/deploy-biomeos.sh --towers 5

# Validate federation
./scripts/validate-federation.sh
```

---

**Next Demo**: 04 - Secure Relay (lineage-gated routing)

