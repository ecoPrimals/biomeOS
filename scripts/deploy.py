#!/usr/bin/env python3
"""Tower Atomic + Squirrel Deployment"""

import os
import subprocess
import sys
import time

FAMILY_ID = sys.argv[1] if len(sys.argv) > 1 else "nat0"
RUNTIME_DIR = "/tmp"
API_KEY = os.environ.get("ANTHROPIC_API_KEY", "")

if not API_KEY:
    print("❌ ANTHROPIC_API_KEY not set")
    sys.exit(1)

print(f"🧬 Deploying Tower Atomic + Squirrel (family: {FAMILY_ID})\n")

# Clean up
print("Cleaning previous deployment...")
subprocess.run(f"pkill -f 'beardog.*{FAMILY_ID}' 2>/dev/null || true", shell=True)
subprocess.run(f"pkill -f 'songbird.*{FAMILY_ID}' 2>/dev/null || true", shell=True)
subprocess.run(f"pkill -f 'squirrel.*{FAMILY_ID}' 2>/dev/null || true", shell=True)
time.sleep(1)

# Socket paths
BEARDOG_SOCKET = f"{RUNTIME_DIR}/beardog-{FAMILY_ID}.sock"
SONGBIRD_SOCKET = f"{RUNTIME_DIR}/songbird-{FAMILY_ID}.sock"
SQUIRREL_SOCKET = f"{RUNTIME_DIR}/squirrel-{FAMILY_ID}.sock"

# Phase 1: BearDog
print("Phase 1/3: Starting BearDog...")
with open(f"{RUNTIME_DIR}/beardog-{FAMILY_ID}.log", "w") as log:
    subprocess.Popen([
        "./plasmidBin/primals/beardog/beardog-x86_64-musl", "server",
        "--socket", BEARDOG_SOCKET,
        "--family-id", FAMILY_ID
    ], stdout=log, stderr=log)

print("  Waiting for BearDog socket", end="", flush=True)
for _ in range(30):
    if os.path.exists(BEARDOG_SOCKET):
        break
    print(".", end="", flush=True)
    time.sleep(0.2)
print()

if not os.path.exists(BEARDOG_SOCKET):
    print("❌ BearDog failed to start")
    with open(f"{RUNTIME_DIR}/beardog-{FAMILY_ID}.log") as f:
        print(f.read())
    sys.exit(1)
print(f"  ✅ BearDog ready: {BEARDOG_SOCKET}")

# Phase 2: Songbird
print("\nPhase 2/3: Starting Songbird (bonded to BearDog)...")
env = os.environ.copy()
env.update({
    "SONGBIRD_SOCKET": SONGBIRD_SOCKET,
    "SONGBIRD_SECURITY_PROVIDER": BEARDOG_SOCKET,
    "SONGBIRD_ORCHESTRATOR_FAMILY_ID": FAMILY_ID
})

with open(f"{RUNTIME_DIR}/songbird-{FAMILY_ID}.log", "w") as log:
    subprocess.Popen([
        "./plasmidBin/primals/songbird/songbird-x86_64-musl", "server"
    ], env=env, stdout=log, stderr=log)

print("  Waiting for Songbird socket", end="", flush=True)
for _ in range(30):
    if os.path.exists(SONGBIRD_SOCKET):
        break
    print(".", end="", flush=True)
    time.sleep(0.2)
print()

if not os.path.exists(SONGBIRD_SOCKET):
    print("❌ Songbird failed to start")
    with open(f"{RUNTIME_DIR}/songbird-{FAMILY_ID}.log") as f:
        print(f.read())
    sys.exit(1)
print(f"  ✅ Songbird ready: {SONGBIRD_SOCKET}")

# Phase 3: Squirrel
print("\nPhase 3/3: Starting Squirrel...")
env = os.environ.copy()
env.update({
    "AI_PROVIDER_SOCKETS": SONGBIRD_SOCKET,  # Songbird is the AI provider!
    "SONGBIRD_ENDPOINT": SONGBIRD_SOCKET,
    "ANTHROPIC_API_KEY": API_KEY,
    "OPENAI_API_KEY": os.environ.get("OPENAI_API_KEY", ""),
})

with open(f"{RUNTIME_DIR}/squirrel-{FAMILY_ID}.log", "w") as log:
    subprocess.Popen([
        "./plasmidBin/primals/squirrel/squirrel-x86_64-musl", "server",
        "--socket", SQUIRREL_SOCKET
    ], env=env, stdout=log, stderr=log)

print("  Waiting for Squirrel socket", end="", flush=True)
for _ in range(30):
    if os.path.exists(SQUIRREL_SOCKET):
        break
    print(".", end="", flush=True)
    time.sleep(0.2)
print()

if not os.path.exists(SQUIRREL_SOCKET):
    print("⚠️  Squirrel socket not found (may still be starting)")
else:
    print(f"  ✅ Squirrel ready: {SQUIRREL_SOCKET}")

# Summary
print("\n" + "=" * 42)
print("✅ Tower Atomic + Squirrel Deployed!")
print("=" * 42)
print(f"\nSockets:")
print(f"  BearDog:  {BEARDOG_SOCKET}")
print(f"  Songbird: {SONGBIRD_SOCKET}")
print(f"  Squirrel: {SQUIRREL_SOCKET}")
print(f"\nTest AI call:")
print(f'  echo \'{{"jsonrpc":"2.0","method":"ai.chat","params":{{"messages":[{{"role":"user","content":"Hello!"}}]}},"id":1}}\' | nc -U {SQUIRREL_SOCKET}')
print(f"\nLogs:")
print(f"  tail -f {RUNTIME_DIR}/beardog-{FAMILY_ID}.log")
print(f"  tail -f {RUNTIME_DIR}/songbird-{FAMILY_ID}.log")
print(f"  tail -f {RUNTIME_DIR}/squirrel-{FAMILY_ID}.log")
print()


