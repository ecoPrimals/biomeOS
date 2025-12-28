# Multi-Primal Demo: Storage + Discovery

**Primals:** NestGate (Storage) + Songbird (Discovery)  
**Pattern:** Standalone binaries with dynamic discovery  
**Purpose:** Show how BiomeOS discovers and uses storage without hardcoding

---

## 🎯 What This Demonstrates

### Dynamic Service Discovery
- BiomeOS asks Songbird: "Find me storage"
- Songbird discovers NestGate
- BiomeOS gets endpoint dynamically
- No hardcoded endpoints!

### Real Primal Interaction
- Both services run as separate processes
- Real network communication
- Real API calls
- Gap discovery through actual use

---

## 🚀 How to Run

```bash
./demo.sh
```

**Requirements:**
- Songbird standalone binary
- NestGate binary
- Ports 8080 (Songbird) and 9000 (NestGate) available

---

## 📊 What You'll See

1. **Songbird starts** - Discovery service mesh
2. **NestGate starts** - Storage service
3. **BiomeOS discovers** - Via Songbird
4. **Storage operations** - Create, store, retrieve
5. **Gap discovery** - Document what works/doesn't

---

## 🔍 Expected Gaps

Based on real testing, we expect to find:
- API endpoint variations
- Health check patterns
- Service registration details
- Storage API specifics

**This is good!** Gaps drive improvements.

---

## 📝 After Running

Check:
- `gaps-discovered.md` - What we learned
- `logs/songbird.log` - Songbird output
- `logs/nestgate.log` - NestGate output

---

**Philosophy:** Real integration reveals real gaps. This is how we improve!

