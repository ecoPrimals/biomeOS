# ecoPrimals Deployment Status - February 3, 2026

## Family Configuration

| Parameter | Value |
|-----------|-------|
| **Family ID** | `ecosystem_alpha` |
| **Family Seed** | HSM-generated from Pixel (32 bytes) |
| **Seed Hash** | `21c644585121f7a3` |

## Deployed Primals

### USB (x86_64-unknown-linux-musl)

| Primal | Socket | Port | Status |
|--------|--------|------|--------|
| BearDog | `/tmp/beardog.sock` | - | ✅ Running |
| Songbird | `/tmp/songbird-ecosystem.sock` | 8082 | ✅ Running |
| Rogue BearDog | `/tmp/beardog-rogue.sock` | - | ✅ Test instance |

### Pixel (aarch64-linux-android)

| Primal | Socket | Port | Status |
|--------|--------|------|--------|
| BearDog | TCP | 9900 | ✅ Running |
| Songbird | TCP | 9901/8081 | ✅ Running |
| Toadstool | TCP | - | ✅ Running |
| NestGate | TCP | 8085 | ✅ Running |
| Squirrel | TCP | - | ✅ Running |

## Verified Capabilities

### Dark Forest Protocol

| Component | Status | Description |
|-----------|--------|-------------|
| Genetic Handshake | ✅ Verified | USB ↔ Pixel mutual verification |
| BirdSong Encryption | ✅ Verified | Bidirectional encrypted messaging |
| Adversary Rejection | ✅ Verified | Rogue tower cannot participate |
| Traffic Analysis | ✅ Verified | All ciphertext appears as random noise |

### Network Discovery

| Component | Status | Description |
|-----------|--------|-------------|
| USB STUN | ✅ Working | Public IP: 162.226.225.148 |
| Pixel STUN | ✅ Working | Public IP: 162.226.225.148 |
| Same NAT | ✅ Confirmed | Both devices share public IP |

### Recommended STUN Server

```
74.125.250.129:19302  (Google - reliable on both USB and Pixel)
```

## Quick Reference

### USB Commands

```bash
# BearDog RPC
echo '{"jsonrpc":"2.0","method":"primal.info","id":1}' | nc -q 1 -U /tmp/beardog.sock

# Songbird RPC
echo '{"jsonrpc":"2.0","method":"primal.info","id":1}' | nc -q 1 -U /tmp/songbird-ecosystem.sock

# STUN Discovery
echo '{"jsonrpc":"2.0","method":"stun.get_public_address","params":{"server":"74.125.250.129:19302"},"id":1}' | nc -q 1 -U /tmp/songbird-ecosystem.sock

# BirdSong Encrypt
MSG=$(echo -n "hello" | base64)
echo '{"jsonrpc":"2.0","method":"birdsong.encrypt","params":{"plaintext":"'$MSG'","family_id":"ecosystem_alpha"},"id":1}' | nc -q 1 -U /tmp/beardog.sock
```

### Pixel Commands

```bash
# BearDog RPC
adb shell "echo '{\"jsonrpc\":\"2.0\",\"method\":\"primal.info\",\"id\":1}' | nc 127.0.0.1 9900"

# Songbird RPC
adb shell "echo '{\"jsonrpc\":\"2.0\",\"method\":\"primal.info\",\"id\":1}' | nc 127.0.0.1 9901"

# STUN Discovery
adb shell "echo '{\"jsonrpc\":\"2.0\",\"method\":\"stun.get_public_address\",\"params\":{\"server\":\"74.125.250.129:19302\"},\"id\":1}' | nc 127.0.0.1 9901"
```

### Cross-Device Genetic Handshake

```bash
# 1. Generate challenge on USB
CHALLENGE=$(echo '{"jsonrpc":"2.0","method":"genetic.generate_challenge","params":{"target_family_id":"ecosystem_alpha","challenger_node_id":"usb_hub"},"id":1}' | nc -q 1 -U /tmp/beardog.sock)
CHALLENGE_ID=$(echo "$CHALLENGE" | jq -r '.result.challenge_id')
NONCE=$(echo "$CHALLENGE" | jq -r '.result.nonce')

# 2. Pixel responds
RESPONSE=$(adb shell "echo '{\"jsonrpc\":\"2.0\",\"method\":\"genetic.respond_to_challenge\",\"params\":{\"challenge_id\":\"$CHALLENGE_ID\",\"nonce\":\"$NONCE\",\"responder\":\"pixel_hsm\",\"our_node_id\":\"pixel_hsm\",\"our_family_seed_path\":\"/data/local/tmp/.family.seed\"},\"id\":2}' | nc 127.0.0.1 9900")

# 3. USB verifies
# ... extract response values and verify
```

## Known Issues

| Issue | Workaround | Status |
|-------|------------|--------|
| IPv6 DNS priority | Always specify IPv4 server address | Workaround |
| Some STUN server timeouts on Pixel | Use Google STUN | Resolved |

## File Locations

### USB
- Family Seed: `/home/eastgate/Development/ecoPrimals/phase2/biomeOS/.family.seed`
- Primals: `/home/eastgate/Development/ecoPrimals/phase2/biomeOS/livespore-usb/primals/`

### Pixel
- Family Seed: `/data/local/tmp/.family.seed`
- Primals: `/data/local/tmp/primals/`
- Other primals: `/data/local/tmp/`

---

*Last Updated: February 3, 2026*
