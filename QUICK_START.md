# Quick Start - biomeOS

Deploy biomeOS in under 5 minutes.

---

## Prerequisites

- Linux (x86_64 or aarch64) or Android
- `nc` (netcat) for testing
- `socat` (optional, for debugging)

---

## Option 1: Pure Rust NUCLEUS (Recommended)

```bash
# Full NUCLEUS (all primals + Neural API + AI bridge)
biomeos nucleus start --mode full --node-id tower1

# Tower Atomic only (BearDog + Songbird)
biomeos nucleus start --mode tower --node-id tower1

# Nest Atomic (Tower + NestGate)
biomeos nucleus start --mode nest --node-id tower1
```

The pure Rust nucleus mode automatically:
- Derives FAMILY_ID from `.family.seed` or `$FAMILY_ID` env var
- Discovers primal binaries from `livespore-usb/`, `plasmidBin/`, `target/release/`, `$PATH`
- Detects bootstrap vs. coordinated mode (joins existing ecosystem if primals are already running)
- Starts primals in dependency order with family-suffixed sockets
- Deep JSON-RPC health checks to verify each primal responds
- Registers with LifecycleManager for auto-monitoring and auto-resurrection
- Graceful coordinated shutdown on Ctrl+C (SIGTERM -> timeout -> SIGKILL)

### Verify Deployment

```bash
# Check all sockets
ls -la /run/user/$(id -u)/biomeos/*.sock

# BearDog health
echo '{"jsonrpc":"2.0","method":"health","params":{},"id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/beardog.sock -w 2 -q 1

# Squirrel health (shows active AI providers)
echo '{"jsonrpc":"2.0","method":"health","params":{},"id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/squirrel.sock -w 2 -q 1

# Query local AI (via Ollama through Songbird)
echo '{"jsonrpc":"2.0","method":"http.request","params":{"method":"POST","url":"http://localhost:11434/v1/chat/completions","headers":{"content-type":"application/json"},"body":"{\"model\":\"tinyllama\",\"messages\":[{\"role\":\"user\",\"content\":\"hello\"}],\"max_tokens\":10}"},"id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/songbird.sock -w 15 -q 1

# Query cloud AI (via Squirrel -> Songbird -> Anthropic)
echo '{"jsonrpc":"2.0","method":"query_ai","params":{"prompt":"hello","model":"claude-3-haiku-20240307","max_tokens":10},"id":1}' | \
  nc -U /run/user/$(id -u)/biomeos/squirrel.sock -w 15 -q 1
```

---

## Option 2: Manual Primal Start

```bash
SOCKET_DIR=/run/user/$(id -u)/biomeos
mkdir -p $SOCKET_DIR

# 1. BearDog (crypto) - must start first
livespore-usb/x86_64/primals/beardog server --socket $SOCKET_DIR/beardog.sock &

# 2. Songbird (network) - needs BearDog
SONGBIRD_SECURITY_PROVIDER=$SOCKET_DIR/beardog.sock \
BEARDOG_SOCKET=$SOCKET_DIR/beardog.sock \
  livespore-usb/x86_64/primals/songbird server \
    --port 8080 --socket $SOCKET_DIR/songbird.sock &

# 3. NestGate (storage)
NESTGATE_JWT_SECRET=$(head -c 48 /dev/urandom | base64) \
  livespore-usb/x86_64/primals/nestgate daemon --socket-only &

# 4. Squirrel (AI) - needs Songbird for HTTP
HTTP_REQUEST_PROVIDER_SOCKET=$SOCKET_DIR/songbird.sock \
AI_HTTP_PROVIDERS=anthropic,openai \
ANTHROPIC_API_KEY=your-key-here \
  livespore-usb/x86_64/primals/squirrel server --socket $SOCKET_DIR/squirrel.sock &

# 5. Neural API (orchestration) - needs all primals
BEARDOG_SOCKET=$SOCKET_DIR/beardog.sock \
SONGBIRD_SOCKET=$SOCKET_DIR/songbird.sock \
NODE_ID=tower1 \
  target/release/biomeos neural-api --socket $SOCKET_DIR/neural-api.sock &
```

---

## Option 3: Pixel 8a (Android)

```bash
adb push pixel8a-deploy /data/local/tmp/biomeos
adb shell /data/local/tmp/biomeos/start_nucleus_mobile.sh
```

---

## Build from Source

```bash
cargo build --workspace --release
```

---

## Environment Variables

| Variable | Purpose | Example |
|----------|---------|---------|
| `NODE_ID` | Node identifier | `tower1` |
| `FAMILY_ID` | Derived from .family.seed (or set manually) | `cf7e8729dc4ff05f` |
| `RUST_LOG` | Logging level | `info` |
| `HTTP_REQUEST_PROVIDER_SOCKET` | Squirrel HTTP capability provider | `/run/user/1000/biomeos/songbird.sock` |
| `AI_HTTP_PROVIDERS` | Enabled cloud AI providers | `anthropic,openai` |
| `ANTHROPIC_API_KEY` | Anthropic Claude API key | `sk-ant-...` |
| `OPENAI_API_KEY` | OpenAI GPT API key | `sk-proj-...` |
| `SONGBIRD_SECURITY_PROVIDER` | BearDog socket for Songbird TLS | `/run/user/1000/biomeos/beardog.sock` |
| `BEARDOG_SOCKET` | BearDog socket path | `/run/user/1000/biomeos/beardog.sock` |
| `NEURAL_API_SOCKET` | Neural API socket path | `/run/user/1000/biomeos/neural-api.sock` |
| `SONGBIRD_MESH_PORT` | Plasmodium remote gate port | `8080` |
| `SONGBIRD_HTTP_PORT` | Songbird HTTP listen port (beacon payload) | `8080` |

---

## Socket Paths

Sockets are resolved via `SystemPaths` (XDG-compliant) in this priority order:

1. `$PRIMAL_SOCKET` (primal-specific env var, e.g. `$BEARDOG_SOCKET`)
2. `$XDG_RUNTIME_DIR/biomeos/` (XDG standard)
3. `/run/user/$UID/biomeos/` (Linux default)
4. `/data/local/tmp/biomeos/` (Android)
5. `/tmp/biomeos/` (fallback)

---

## Troubleshooting

### Squirrel shows "0 providers"

Ensure `HTTP_REQUEST_PROVIDER_SOCKET` points to the Songbird socket
and `AI_HTTP_PROVIDERS` or API key env vars are set.

### Songbird won't start

Songbird requires `SONGBIRD_SECURITY_PROVIDER` or `SECURITY_ENDPOINT`
pointing to BearDog's socket. Start BearDog first.

### NestGate socket not found

Use `nestgate daemon --socket-only` (not `service start`).
The `--socket-only` flag has a known inverted-boolean bug in some versions.

### Claude returns "model not found"

Pass `model: "claude-3-haiku-20240307"` explicitly. The default (Opus)
may not be available on your API key tier.

### Primal crashes after startup

Check LifecycleManager auto-resurrection. If a primal degrades, the lifecycle
manager will attempt restart with exponential backoff. Run `biomeos doctor`
for diagnostics.

---

## Next Steps

1. Read [CURRENT_STATUS.md](CURRENT_STATUS.md) for validated systems
2. See `ecoPrimals/wateringHole/handoffs/` for evolution reports
3. Review `specs/EVOLUTION_ROADMAP.md` for the full roadmap

---

**Status**: Production Ready (v2.72)
**Updated**: March 27, 2026
**Tests**: 7,167 passing (0 failures), 90%+ coverage (llvm-cov verified) | **Clippy**: PASS (0 warnings) | **C deps**: 0 | **Unsafe**: 0 production

**Note**: `biomeos monitor dashboard` now redirects to petalTongue (TUI deprecated).
