# Quick Start - biomeOS

Deploy biomeOS in under 5 minutes.

---

## Prerequisites

- Linux (x86_64 or aarch64) or Android
- Rust toolchain (for building from source)
- `nc` (netcat) for testing

---

## Option 1: Deploy from USB (Recommended)

### x86_64 Linux

```bash
cd livespore-usb/x86_64/scripts/

# Set your family ID
export FAMILY_ID=my_ecosystem

# Deploy Tower Atomic (BearDog + Songbird)
./start_tower.sh

# Or deploy full NUCLEUS
./deploy_atomic.sh nucleus
```

### aarch64 Linux/Android

```bash
cd livespore-usb/aarch64/scripts/
export FAMILY_ID=my_ecosystem
./start_tower.sh
```

### Pixel 8a (Android)

```bash
# Push deployment package
adb push pixel8a-deploy /data/local/tmp/biomeos

# Start Tower Atomic
adb shell /data/local/tmp/biomeos/start_nucleus_mobile.sh
```

---

## Option 2: Build from Source

```bash
# Build all workspace crates
cargo build --workspace --release

# Run biomeOS CLI
cargo run --package biomeos-cli --bin biomeos -- --help
```

---

## Verify Deployment

### Check BearDog Health

```bash
echo '{"jsonrpc":"2.0","method":"health","id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/beardog-${FAMILY_ID}.sock
```

### Check Songbird Health

```bash
echo '{"jsonrpc":"2.0","method":"health","id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/songbird-${FAMILY_ID}.sock
```

### List Active Sockets

```bash
ls -la /run/user/$(id -u)/biomeos/*.sock
```

---

## Deployment Commands

| Command | Description |
|---------|-------------|
| `./start_tower.sh` | BearDog + Songbird |
| `./start_node.sh` | Tower + Toadstool |
| `./start_nest.sh` | Node + NestGate |
| `./deploy_atomic.sh tower` | Same as start_tower.sh |
| `./deploy_atomic.sh nucleus` | Full NUCLEUS |
| `./deploy_atomic.sh --graph nucleus` | Graph-based deployment |

---

## Environment Variables

| Variable | Purpose | Example |
|----------|---------|---------|
| `FAMILY_ID` | Genetic lineage identifier | `my_ecosystem` |
| `NODE_ID` | Node identifier | `node1` |
| `RUST_LOG` | Logging level | `info` |
| `PRIMAL_SOCKET` | Override socket path | `/tmp/beardog.sock` |
| `XDG_RUNTIME_DIR` | XDG runtime directory | `/run/user/1000` |
| `BIOMEOS_SECURITY_PROVIDER` | Security primal name | `beardog` |
| `BIOMEOS_NETWORK_PROVIDER` | Network primal name | `songbird` |

---

## Socket Paths

Sockets are created at (in priority order):

1. `$PRIMAL_SOCKET` (if set)
2. `$XDG_RUNTIME_DIR/biomeos/`
3. `/run/user/$UID/biomeos/`
4. `/data/local/tmp/biomeos/` (Android)
5. `/tmp/biomeos/` (fallback)

---

## Troubleshooting

### Socket not found

```bash
# Check socket directory exists
mkdir -p /run/user/$(id -u)/biomeos

# Check permissions
ls -la /run/user/$(id -u)/biomeos/
```

### Connection refused

```bash
# Check if primal is running
ps aux | grep beardog

# Check logs
cat /tmp/biomeos/beardog.log
```

### Android issues

```bash
# Ensure /data/local/tmp is writable
adb shell mkdir -p /data/local/tmp/biomeos

# Check binary permissions
adb shell chmod +x /data/local/tmp/biomeos/primals/*
```

---

## Next Steps

1. Read [START_HERE.md](START_HERE.md) for architecture overview
2. See [CURRENT_STATUS.md](CURRENT_STATUS.md) for latest status
3. Review [DOCUMENTATION.md](DOCUMENTATION.md) for full documentation index

---

**Status**: Production Ready
**Updated**: February 7, 2026
**Tests**: 1,747 passing
