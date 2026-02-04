# Primal Deployment Standard v1.0

**Date**: February 3, 2026  
**Status**: Specification  
**Purpose**: Ensure deterministic primal behavior across all architectures

---

## 1. Core Principle

**Primals MUST behave identically regardless of architecture.**

The same binary with the same environment variables produces the same behavior on:
- x86_64 (Intel/AMD)
- aarch64 (ARM64: Pixel 8a, Raspberry Pi, Apple Silicon)
- Future architectures

---

## 2. IPC Standard: Unix Sockets First

### 2.1 Socket Path Resolution (5-Tier Fallback)

Primals MUST attempt socket paths in this order:

```
Tier 1: $PRIMAL_SOCKET                    (explicit override)
Tier 2: $XDG_RUNTIME_DIR/biomeos/         (XDG compliant)
Tier 3: /run/user/$UID/biomeos/           (Linux standard)
Tier 4: /data/local/tmp/biomeos/          (Android termux)
Tier 5: /tmp/biomeos/                     (universal fallback)
```

**Socket Naming Convention**:
```
{primal_name}-{family_id}.sock

Examples:
  beardog-ecosystem_alpha.sock
  songbird-ecosystem_alpha.sock
  squirrel-ecosystem_alpha.sock
```

### 2.2 Socket vs HTTP Decision Tree

```
START
  │
  ├─► Is XDG_RUNTIME_DIR available?
  │     YES → Use Unix sockets (Tier 1-3)
  │     NO  → Is Android?
  │             YES → Use /data/local/tmp (Tier 4)
  │             NO  → Use /tmp (Tier 5)
  │
  └─► Socket creation successful?
        YES → IPC via Unix socket
        NO  → Fallback to TCP (same machine only)
```

### 2.3 TCP Fallback Rules

TCP SHOULD only be used when:
1. Unix sockets are unavailable (sandboxed environments)
2. Cross-machine communication is required
3. Platform limitations prevent socket creation

**TCP Binding Standard**:
```
ALWAYS bind to 127.0.0.1 (localhost only)
NEVER bind to 0.0.0.0 unless explicitly configured
```

---

## 3. Environment Variables (Required)

### 3.1 Universal Environment Variables

| Variable | Required | Default | Purpose |
|----------|----------|---------|---------|
| `FAMILY_ID` | YES | `default` | Genetic family identifier |
| `NODE_ID` | NO | `$(hostname)` | Node identifier |
| `RUST_LOG` | NO | `info` | Log verbosity |
| `XDG_RUNTIME_DIR` | NO | auto-detect | Socket directory |

### 3.2 Primal-Specific Environment

**BearDog**:
| Variable | Required | Default | Purpose |
|----------|----------|---------|---------|
| `BEARDOG_SOCKET` | NO | auto | Override socket path |
| `BEARDOG_LISTEN` | NO | none | TCP fallback address |

**Songbird**:
| Variable | Required | Default | Purpose |
|----------|----------|---------|---------|
| `SONGBIRD_SOCKET` | NO | auto | Override socket path |
| `SONGBIRD_PORT` | NO | none | TCP fallback port |
| `BEARDOG_MODE` | NO | `neural` | BearDog integration mode |
| `BEARDOG_SOCKET` | NO | auto | BearDog socket for TLS |

**NestGate**:
| Variable | Required | Default | Purpose |
|----------|----------|---------|---------|
| `NESTGATE_SOCKET` | NO | auto | Override socket path |
| `NESTGATE_JWT_SECRET` | NO | random | JWT signing key |

**Squirrel**:
| Variable | Required | Default | Purpose |
|----------|----------|---------|---------|
| `SQUIRREL_SOCKET` | NO | auto | Override socket path |
| `NEURAL_API_SOCKET` | NO | auto | Neural API for capability discovery |
| `AI_HTTP_PROVIDERS` | NO | none | Comma-separated AI providers |

**Toadstool**:
| Variable | Required | Default | Purpose |
|----------|----------|---------|---------|
| `TOADSTOOL_SOCKET` | NO | auto | Override socket path |

---

## 4. Startup Sequence Standard

### 4.1 Tower Atomic (BearDog + Songbird)

```bash
#!/bin/bash
# Standard Tower Atomic startup - Architecture agnostic

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ARCH="$(uname -m)"

# Environment
export FAMILY_ID="${FAMILY_ID:-ecosystem}"
export NODE_ID="${NODE_ID:-$(hostname)}"
export RUST_LOG="${RUST_LOG:-info}"

# Socket directory (5-tier resolution)
if [ -n "$XDG_RUNTIME_DIR" ]; then
    SOCKET_DIR="$XDG_RUNTIME_DIR/biomeos"
elif [ -d "/run/user/$(id -u)" ]; then
    SOCKET_DIR="/run/user/$(id -u)/biomeos"
elif [ -d "/data/local/tmp" ]; then
    SOCKET_DIR="/data/local/tmp/biomeos"  # Android
else
    SOCKET_DIR="/tmp/biomeos"
fi

mkdir -p "$SOCKET_DIR"

# Start BearDog
"$SCRIPT_DIR/../primals/beardog" server \
    --socket "$SOCKET_DIR/beardog-$FAMILY_ID.sock" \
    --family-id "$FAMILY_ID" &

sleep 3

# Start Songbird
BEARDOG_SOCKET="$SOCKET_DIR/beardog-$FAMILY_ID.sock" \
"$SCRIPT_DIR/../primals/songbird" server \
    --socket "$SOCKET_DIR/songbird-$FAMILY_ID.sock" &
```

### 4.2 Node Atomic (Tower + Toadstool)

```bash
# Start Tower first
source "$SCRIPT_DIR/start_tower.sh"

# Add Toadstool
"$SCRIPT_DIR/../primals/toadstool" daemon \
    --socket "$SOCKET_DIR/toadstool-$FAMILY_ID.sock" \
    --register &
```

### 4.3 Nest Atomic (Tower + NestGate + Squirrel)

```bash
# Start Tower first
source "$SCRIPT_DIR/start_tower.sh"

# Add NestGate
export NESTGATE_JWT_SECRET="${NESTGATE_JWT_SECRET:-$(openssl rand -base64 48)}"
"$SCRIPT_DIR/../primals/nestgate" service start \
    --socket "$SOCKET_DIR/nestgate-$FAMILY_ID.sock" \
    --daemon &

# Add Squirrel
NEURAL_API_SOCKET="$SOCKET_DIR/neural-api-$FAMILY_ID.sock" \
"$SCRIPT_DIR/../primals/squirrel" server &
```

---

## 5. Architecture-Specific Considerations

### 5.1 x86_64 (Intel/AMD Linux)

| Aspect | Standard |
|--------|----------|
| Socket Dir | `/run/user/$UID/biomeos/` |
| IPC | Unix sockets only |
| TCP | Not required |

### 5.2 aarch64 (ARM64 Linux)

| Aspect | Standard |
|--------|----------|
| Socket Dir | `/run/user/$UID/biomeos/` |
| IPC | Unix sockets only |
| TCP | Not required |

**Note**: ARM64 Linux (Raspberry Pi, etc.) behaves identically to x86_64.

### 5.3 Android (Termux/proot-distro)

| Aspect | Standard |
|--------|----------|
| Socket Dir | `/data/local/tmp/biomeos/` |
| IPC | Unix sockets preferred |
| TCP | Only if sockets fail |

**Android-Specific Notes**:
- `XDG_RUNTIME_DIR` typically unavailable
- Use `/data/local/tmp/` for writable storage
- SELinux may restrict socket creation

### 5.4 GrapheneOS (Pixel 8a)

| Aspect | Standard |
|--------|----------|
| Socket Dir | `/data/local/tmp/biomeos/` |
| IPC | Unix sockets in proot |
| TCP | Fallback for raw Android |

---

## 6. Anti-Patterns (DO NOT DO)

### 6.1 Hardcoded Ports

```bash
# WRONG - hardcoded HTTP port
./songbird server --port 8081

# CORRECT - socket-first, TCP optional
./songbird server --socket "$SOCKET_DIR/songbird.sock"
```

### 6.2 Different Behavior Per Architecture

```bash
# WRONG - architecture-specific logic
if [ "$ARCH" = "x86_64" ]; then
    USE_SOCKETS=true
else
    USE_HTTP=true  # Why? ARM64 supports sockets!
fi

# CORRECT - same logic, different paths only
SOCKET_DIR=$(resolve_socket_dir)  # 5-tier resolution
```

### 6.3 HTTP by Default

```bash
# WRONG - HTTP as primary
./nestgate service start --port 8090

# CORRECT - Socket as primary
./nestgate service start --socket "$SOCKET_DIR/nestgate.sock"
```

---

## 7. Validation Checklist

For each primal deployment, verify:

- [ ] Uses 5-tier socket resolution
- [ ] No hardcoded HTTP ports (unless configured)
- [ ] Same startup script works on x86_64 and aarch64
- [ ] FAMILY_ID is set consistently
- [ ] Logs show socket path, not HTTP binding
- [ ] Health check works via socket

---

## 8. Migration Guide

### From HTTP to Socket

**Before** (pixel8a-deploy/start_tower.sh):
```bash
./songbird server --port 8081
```

**After**:
```bash
./songbird server --socket "$SOCKET_DIR/songbird-$FAMILY_ID.sock"
```

### From Hardcoded Paths to 5-Tier

**Before**:
```bash
mkdir -p /tmp/biomeos
```

**After**:
```bash
SOCKET_DIR=$(resolve_socket_dir)  # See Section 4.1
mkdir -p "$SOCKET_DIR"
```

---

## 9. Summary

| Principle | Standard |
|-----------|----------|
| **IPC** | Unix sockets first, TCP fallback |
| **Paths** | 5-tier resolution, never hardcoded |
| **Behavior** | Identical across architectures |
| **Config** | Environment variables, not flags |
| **HTTP** | Only when sockets unavailable |

---

**Version**: 1.0  
**Effective**: February 3, 2026  
**Review**: Quarterly or on major architecture changes
