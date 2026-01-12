# 🚀 Primal Launcher - Pure Rust Primal Management

**Date**: January 11, 2026  
**Status**: ✅ Complete - Ready for Testing  

---

## 🎯 **Purpose**

Pure Rust launcher for starting biomeOS primals with proper configuration.

**Replaces**: Bash scripts, manual primal starting  
**Provides**: Type-safe, XDG-compliant, capability-based primal launching  

---

## 📦 **Usage**

### **Launch Complete Atomics**

```bash
# Tower (BearDog + Songbird)
cargo run --bin launch_primal -- tower nat0

# Node (BearDog + Songbird + ToadStool)
cargo run --bin launch_primal -- node nat0

# Nest (BearDog + Songbird + NestGate)
cargo run --bin launch_primal -- nest nat0
```

### **Launch Individual Primals**

```bash
# Security
cargo run --bin launch_primal -- beardog nat0

# Discovery
cargo run --bin launch_primal -- songbird nat0

# Compute
cargo run --bin launch_primal -- toadstool nat0

# Storage
cargo run --bin launch_primal -- nestgate nat0

# AI
cargo run --bin launch_primal -- squirrel nat0
```

---

## 🔧 **What It Does**

For each primal:
1. ✅ Validates binary exists in `plasmidBin/`
2. ✅ Generates XDG-compliant socket path: `/run/user/{uid}/{primal}-{family}.sock`
3. ✅ Sets environment variables (family_id, socket paths)
4. ✅ Redirects logs to `/tmp/{primal}-{family}.log`
5. ✅ Spawns process in background
6. ✅ Verifies process didn't crash immediately

---

## 📂 **Socket Paths Generated**

```
/run/user/1000/beardog-nat0.sock
/run/user/1000/songbird-nat0.sock
/run/user/1000/toadstool-nat0.sock
/run/user/1000/nestgate-nat0.sock
/run/user/1000/squirrel-nat0.sock
```

---

## 📝 **Log Paths**

```
/tmp/beardog-nat0.log
/tmp/songbird-nat0.log
/tmp/toadstool-nat0.log
/tmp/nestgate-nat0.log
/tmp/squirrel-nat0.log
```

---

## 🧬 **Integration with Atomic Deployment**

**Step 1**: Launch primals for an atomic
```bash
cargo run --bin launch_primal -- tower nat0
```

**Step 2**: Deploy the atomic
```bash
cargo run --bin deploy_atomic -- tower
```

**Step 3**: Verify deployment
```bash
# Check sockets exist
ls /run/user/1000/*-nat0.sock

# Check logs
tail /tmp/beardog-nat0.log
tail /tmp/songbird-nat0.log
```

---

## ⚠️  **Current Limitations**

1. **Primal CLI Compatibility**: Each primal may require specific CLI args/env vars
2. **Socket Path Discovery**: Primals must respect env var configuration
3. **Auto-Registration**: Primals should auto-register with Songbird

**Status**: Primal teams need to verify their binaries accept these env vars!

---

## 🎊 **Next Steps**

1. ⏳ Test with real primal binaries
2. ⏳ Verify socket creation
3. ⏳ Verify Songbird auto-registration
4. ⏳ Deploy Tower atomic live
5. ⏳ Test BearDog genetic lineage verification

---

**Different orders of the same architecture.** 🍄🐸

No more bash! Pure concurrent Rust for primal management! 🦀


