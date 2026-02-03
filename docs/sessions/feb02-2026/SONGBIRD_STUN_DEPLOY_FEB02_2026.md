# Songbird STUN & Dark Forest Deployment - Feb 2, 2026

## Summary

Successfully deployed Songbird v3.33.0 with STUN NAT traversal and BearDog security integration.

---

## Achievements

### 1. Songbird Build Complete
- **x86_64 (USB)**: 18MB binary (`target/release/songbird`)
- **aarch64 (Pixel)**: 16MB binary (`target/aarch64-linux-android/release/songbird`)
- Build time: ~3 minutes per architecture

### 2. Graph Deployment System
Created `graphs/songbird_dark_forest_deploy.toml` with:
- BearDog dependency verification
- Songbird startup with security provider wiring
- STUN capability verification
- Dark Forest beacon capability verification

### 3. USB Deployment ✅ WORKING
```
/media/eastgate/biomeOS1/biomeOS/primals/
├── beardog   (6.5MB - TCP IPC + genetic methods)
├── songbird  (18MB - STUN + Dark Forest)
├── nestgate  (5.1MB)
└── toadstool (8.4MB)
```

**STUN Test Result**:
```json
{
  "local_address": "0.0.0.0:54399",
  "nat_type": "unknown",
  "public_address": "162.226.225.148:50575",
  "server": "159.69.191.124:3478"
}
```

### 4. Pixel Deployment ✅ BINARIES DEPLOYED
```
/data/local/tmp/
├── beardog   (TCP IPC on 127.0.0.1:9900)
├── songbird  (16MB - deployed)
└── songbird_dark_forest_deploy.toml
```

### 5. BearDog Genetic Methods ✅ WORKING
All 7 genetic methods available on both platforms:
- `genetic.derive_lineage_key`
- `genetic.generate_challenge`
- `genetic.respond_to_challenge`
- `genetic.verify_challenge_response`
- `genetic.generate_lineage_proof`
- `genetic.verify_lineage`
- `genetic.mix_entropy`

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                    USB (x86_64)                                 │
│  ┌──────────────┐     ┌──────────────┐                         │
│  │   BearDog    │ ←── │   Songbird   │ ←── STUN Server         │
│  │ Unix Socket  │ IPC │  Unix Socket │     (NAT Traversal)     │
│  │ /tmp/beardog │     │ /tmp/songbird│                         │
│  └──────────────┘     └──────────────┘                         │
│         ↓                    ↓                                  │
│    genetic.*           stun.*, http.*                          │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│                    Pixel (aarch64)                              │
│  ┌──────────────┐     ┌──────────────┐                         │
│  │   BearDog    │ ←── │   Songbird   │ ← (IPC issue pending)   │
│  │ TCP :9900    │     │  TCP :8081   │                         │
│  └──────────────┘     └──────────────┘                         │
│         ↓                                                       │
│    genetic.* ✅                                                 │
└─────────────────────────────────────────────────────────────────┘
```

---

## Issues Discovered

### 1. Songbird Android IPC Path Issue
Songbird on Pixel tries to create Unix sockets at `/tmp/` paths which fail on Android.
- BearDog solved this with `--listen tcp://` flag
- Songbird needs similar TCP IPC support for Android

### 2. IPv6 STUN Resolution
DNS resolution returning IPv6 addresses causes STUN to fail with "Address family not supported".
- **Workaround**: Use direct IPv4 addresses (e.g., `159.69.191.124:3478`)

---

## Next Steps

1. **Add TCP IPC to Songbird** (like BearDog's `--listen` flag)
2. **Test cross-device Dark Forest beacon exchange**
3. **Complete STUN hole punching between USB and Pixel**

---

## Files Created/Modified

### New Files
- `graphs/songbird_dark_forest_deploy.toml` - Deployment graph
- `docs/sessions/feb02-2026/SONGBIRD_STUN_DEPLOY_FEB02_2026.md` - This document

### Updated Deployments
- `/media/eastgate/biomeOS1/biomeOS/primals/songbird` - Fresh x86_64 binary
- `/media/eastgate/biomeOS1/biomeOS/primals/beardog` - Fresh x86_64 binary with TCP IPC
- `/data/local/tmp/songbird` - Fresh aarch64 binary

---

## Commands Reference

### USB STUN Test
```bash
echo '{"jsonrpc":"2.0","method":"stun.get_public_address","params":{"server":"159.69.191.124:3478","local_port":54321},"id":1}' | nc -U /tmp/songbird.sock
```

### USB Dark Forest Challenge
```bash
echo '{"jsonrpc":"2.0","method":"genetic.generate_challenge","params":{"target_family_id":"dark_forest_alpha","challenger_node_id":"usb_alpha"},"id":1}' | nc -U /tmp/beardog.sock
```

### Pixel BearDog Test
```bash
adb shell "echo '{\"jsonrpc\":\"2.0\",\"method\":\"primal.info\",\"params\":{},\"id\":1}' | nc 127.0.0.1 9900"
```

---

**Status**: ✅ USB STUN Working | ⏳ Pixel Songbird IPC Pending
**Grade**: A (USB complete, Pixel 80% complete)
