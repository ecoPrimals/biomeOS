# Live Spore USB - Auto-Deploy System

**Concept**: USB "infects" towers with genetic lineage  
**Philosophy**: Plug in → Copies DNA → Activates with local seed → Federation!

---

## 🧬 The Live Spore Concept

### Biological Analogy

**USB = Spore**:
- Contains genetic material (binaries)
- Carries family DNA (iidn lineage)
- Activates when it finds substrate (tower)

**Tower = Substrate**:
- Provides local nutrients (seeds, config)
- Gives unique identity (tower-specific)
- Hosts the activated organism (running services)

**Result**: Same family DNA, unique tower identity

---

## 🚀 Auto-Deploy System Design

### What Happens When USB Plugged In

```
1. USB detected → Mount at /media/usb/
2. User runs: bash /media/usb/activate-tower.sh
3. Script copies binaries to /opt/ecoPrimals/
4. Script reads tower config from /etc/ecoPrimals/tower.conf
5. Script starts services with tower-specific seed
6. Tower joins family with its own identity!
```

---

## 📁 USB Structure (v11.0 - Live Spore)

```
/media/usb/biomeOS-LAN-Deploy/
├── primals/
│   ├── beardog-server          (6.0 MB)
│   ├── songbird-orchestrator   (24 MB)
│   ├── petal-tongue            (19 MB)
│   └── CHECKSUMS.txt
├── scripts/
│   ├── activate-tower.sh       ← MAIN ACTIVATION SCRIPT
│   ├── deploy-local.sh
│   └── tower-status.sh
├── configs/
│   ├── tower.conf.template     ← Template for tower config
│   └── family-seed.conf        ← Family DNA (shared)
└── docs/
    └── LIVE-SPORE-GUIDE.txt
```

---

## 🎯 Tower Configuration System

### Tower-Specific Config

**Location**: `/etc/ecoPrimals/tower.conf`

```bash
# Tower Identity (unique per tower)
TOWER_NAME="tower1"
TOWER_IP="192.168.1.144"
TOWER_SEED="<unique-per-tower>"

# Family Identity (shared across family)
FAMILY_ID="iidn"
FAMILY_SEED="iIDnVX3Tein1LFkrkkq7Wo3wsxPNek9XZqp0VL4Kn88="

# Service Ports
BEARDOG_PORT=9000
SONGBIRD_PORT=8080
```

### First-Time Setup

**If no config exists**, script prompts:
```bash
🧬 Welcome to ecoPrimals Live Spore!
   
No tower configuration found. Let's set one up:

Tower name [tower1]: tower2
Tower IP [auto-detect]: 192.168.1.134
Generate new tower seed? [y/n]: y

✅ Tower configuration saved to /etc/ecoPrimals/tower.conf
✅ Tower identity: tower2 (iidn family)
```

---

## 🔧 The Activation Script

### activate-tower.sh

```bash
#!/bin/bash
# Live Spore USB - Tower Activation Script
# Plugs USB genetic material into tower substrate

set -e

USB_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
INSTALL_DIR="/opt/ecoPrimals"
CONFIG_DIR="/etc/ecoPrimals"
TOWER_CONFIG="$CONFIG_DIR/tower.conf"

echo "════════════════════════════════════════════════════════════════"
echo "  🧬 ecoPrimals Live Spore - Tower Activation"
echo "════════════════════════════════════════════════════════════════"
echo ""

# Step 1: Check for tower configuration
if [ ! -f "$TOWER_CONFIG" ]; then
    echo "⚠️  No tower configuration found"
    echo "   Creating first-time setup..."
    bash "$USB_ROOT/scripts/setup-tower-config.sh"
fi

# Step 2: Load tower configuration
echo "📋 Loading tower configuration..."
source "$TOWER_CONFIG"
echo "   Tower: $TOWER_NAME"
echo "   Family: $FAMILY_ID"
echo ""

# Step 3: Copy binaries (with execute permissions)
echo "📦 Copying genetic material to tower..."
sudo mkdir -p "$INSTALL_DIR/bin"
sudo cp "$USB_ROOT/primals/beardog-server" "$INSTALL_DIR/bin/"
sudo cp "$USB_ROOT/primals/songbird-orchestrator" "$INSTALL_DIR/bin/"
sudo cp "$USB_ROOT/primals/petal-tongue" "$INSTALL_DIR/bin/"
sudo chmod +x "$INSTALL_DIR/bin/"*
echo "✅ Binaries installed to $INSTALL_DIR/bin/"
echo ""

# Step 4: Verify checksums
echo "🔍 Verifying genetic integrity..."
cd "$INSTALL_DIR/bin"
sha256sum -c "$USB_ROOT/primals/CHECKSUMS.txt" || {
    echo "❌ Checksum mismatch! Binaries may be corrupted."
    exit 1
}
echo "✅ All checksums verified"
echo ""

# Step 5: Stop old services
echo "🛑 Stopping old services..."
sudo pkill -f beardog-server 2>/dev/null || true
sudo pkill -f songbird-orchestrator 2>/dev/null || true
sudo pkill -f petal-tongue 2>/dev/null || true
sleep 2
echo "✅ Old services stopped"
echo ""

# Step 6: Start BearDog with tower-specific + family seed
echo "🐻 Activating BearDog with genetic lineage..."
BEARDOG_FAMILY_ID="$FAMILY_ID" \
BEARDOG_FAMILY_SEED="$FAMILY_SEED" \
BEARDOG_TOWER_ID="$TOWER_NAME" \
nohup "$INSTALL_DIR/bin/beardog-server" > /var/log/beardog.log 2>&1 &
BEARDOG_PID=$!
echo $BEARDOG_PID > /var/run/beardog.pid
echo "✅ BearDog activated: PID $BEARDOG_PID"
sleep 5

# Step 7: Verify BearDog family
echo "🔍 Verifying genetic lineage..."
FAMILY_CHECK=$(curl -s http://localhost:$BEARDOG_PORT/api/v1/trust/identity | jq -r '.family_id')
if [ "$FAMILY_CHECK" != "$FAMILY_ID" ]; then
    echo "❌ Family verification failed! Expected: $FAMILY_ID, Got: $FAMILY_CHECK"
    exit 1
fi
echo "✅ Family verified: $FAMILY_ID"
echo ""

# Step 8: Start Songbird with family awareness
echo "🐦 Activating Songbird with genetic awareness..."
SONGBIRD_BEARDOG_URL="http://localhost:$BEARDOG_PORT" \
SONGBIRD_TOWER_NAME="$TOWER_NAME" \
RUST_LOG="info" \
nohup "$INSTALL_DIR/bin/songbird-orchestrator" > /var/log/songbird.log 2>&1 &
SONGBIRD_PID=$!
echo $SONGBIRD_PID > /var/run/songbird.pid
echo "✅ Songbird activated: PID $SONGBIRD_PID"
echo ""

# Step 9: Wait for discovery
echo "⏳ Waiting 10 seconds for genetic expression..."
sleep 10

# Step 10: Verify activation
echo "🔍 Checking genetic expression..."
if grep -q "Family ID: $FAMILY_ID" /var/log/songbird.log; then
    echo "✅ Genetic lineage expressed successfully!"
else
    echo "⚠️  Warning: Genetic lineage not yet visible in logs"
fi
echo ""

# Step 11: Show status
echo "════════════════════════════════════════════════════════════════"
echo "  🎊 Tower Activation Complete!"
echo "════════════════════════════════════════════════════════════════"
echo ""
echo "Tower Identity:"
echo "  Name:        $TOWER_NAME"
echo "  Family:      $FAMILY_ID"
echo "  BearDog:     PID $BEARDOG_PID (port $BEARDOG_PORT)"
echo "  Songbird:    PID $SONGBIRD_PID (port $SONGBIRD_PORT)"
echo ""
echo "Logs:"
echo "  BearDog:     /var/log/beardog.log"
echo "  Songbird:    /var/log/songbird.log"
echo ""
echo "Next Steps:"
echo "  - Wait 30 seconds for UDP discovery"
echo "  - Check for federation: tail -f /var/log/songbird.log | grep AUTO"
echo "  - View status: bash $USB_ROOT/scripts/tower-status.sh"
echo ""
echo "🧬 This tower is now part of the $FAMILY_ID family!"
echo "════════════════════════════════════════════════════════════════"
```

---

## 🔧 Supporting Scripts

### setup-tower-config.sh

```bash
#!/bin/bash
# First-time tower configuration setup

CONFIG_DIR="/etc/ecoPrimals"
TOWER_CONFIG="$CONFIG_DIR/tower.conf"

echo "🧬 First-Time Tower Setup"
echo ""

# Create config directory
sudo mkdir -p "$CONFIG_DIR"

# Detect or prompt for tower name
read -p "Tower name [tower1]: " TOWER_NAME
TOWER_NAME=${TOWER_NAME:-tower1}

# Auto-detect or prompt for IP
DEFAULT_IP=$(hostname -I | awk '{print $1}')
read -p "Tower IP [$DEFAULT_IP]: " TOWER_IP
TOWER_IP=${TOWER_IP:-$DEFAULT_IP}

# Generate unique tower seed
echo "Generating unique tower seed..."
TOWER_SEED=$(openssl rand -base64 32)

# Family configuration (from USB)
USB_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
if [ -f "$USB_ROOT/configs/family-seed.conf" ]; then
    source "$USB_ROOT/configs/family-seed.conf"
else
    FAMILY_ID="iidn"
    FAMILY_SEED="iIDnVX3Tein1LFkrkkq7Wo3wsxPNek9XZqp0VL4Kn88="
fi

# Write configuration
sudo tee "$TOWER_CONFIG" > /dev/null <<EOF
# ecoPrimals Tower Configuration
# Generated: $(date)

# Tower Identity (unique per tower)
TOWER_NAME="$TOWER_NAME"
TOWER_IP="$TOWER_IP"
TOWER_SEED="$TOWER_SEED"

# Family Identity (shared across family)
FAMILY_ID="$FAMILY_ID"
FAMILY_SEED="$FAMILY_SEED"

# Service Ports
BEARDOG_PORT=9000
SONGBIRD_PORT=8080
BIOMEOS_PORT=3000
EOF

sudo chmod 600 "$TOWER_CONFIG"

echo ""
echo "✅ Tower configuration saved to $TOWER_CONFIG"
echo ""
echo "Tower Identity:"
echo "  Name:   $TOWER_NAME"
echo "  IP:     $TOWER_IP"
echo "  Seed:   ${TOWER_SEED:0:16}... (unique)"
echo ""
echo "Family Identity:"
echo "  ID:     $FAMILY_ID"
echo "  Seed:   ${FAMILY_SEED:0:16}... (shared)"
echo ""
```

### tower-status.sh

```bash
#!/bin/bash
# Show tower status

CONFIG_DIR="/etc/ecoPrimals"
TOWER_CONFIG="$CONFIG_DIR/tower.conf"

if [ ! -f "$TOWER_CONFIG" ]; then
    echo "❌ Tower not configured"
    exit 1
fi

source "$TOWER_CONFIG"

echo "════════════════════════════════════════════════════════════════"
echo "  🧬 Tower Status: $TOWER_NAME"
echo "════════════════════════════════════════════════════════════════"
echo ""

# BearDog status
if [ -f /var/run/beardog.pid ] && ps -p $(cat /var/run/beardog.pid) > /dev/null; then
    BEARDOG_STATUS="✅ Running (PID: $(cat /var/run/beardog.pid))"
    FAMILY_CHECK=$(curl -s http://localhost:$BEARDOG_PORT/api/v1/trust/identity | jq -r '.family_id')
else
    BEARDOG_STATUS="❌ Not running"
    FAMILY_CHECK="N/A"
fi

# Songbird status  
if [ -f /var/run/songbird.pid ] && ps -p $(cat /var/run/songbird.pid) > /dev/null; then
    SONGBIRD_STATUS="✅ Running (PID: $(cat /var/run/songbird.pid))"
else
    SONGBIRD_STATUS="❌ Not running"
fi

echo "Tower Identity:"
echo "  Name:     $TOWER_NAME"
echo "  Family:   $FAMILY_ID"
echo "  IP:       $TOWER_IP"
echo ""
echo "Services:"
echo "  BearDog:  $BEARDOG_STATUS"
echo "  Family:   $FAMILY_CHECK"
echo "  Songbird: $SONGBIRD_STATUS"
echo ""

# Show recent discoveries
if [ -f /var/log/songbird.log ]; then
    echo "Recent Discoveries:"
    tail -50 /var/log/songbird.log | grep "Discovered peer" | tail -5 | sed 's/^/  /'
    echo ""
fi

echo "════════════════════════════════════════════════════════════════"
```

---

## 🎊 Usage Flow

### Tower 1 (First Time)

```bash
# 1. Plug in USB
# 2. Run activation
sudo bash /media/usb/biomeOS-LAN-Deploy/scripts/activate-tower.sh

# Output:
# 🧬 First-Time Tower Setup
# Tower name [tower1]: tower1
# Tower IP [192.168.1.144]: 
# ✅ Tower configuration saved
# 📦 Copying genetic material...
# 🐻 Activating BearDog...
# ✅ Family verified: iidn
# 🐦 Activating Songbird...
# 🎊 Tower Activation Complete!
# 🧬 This tower is now part of the iidn family!
```

### Tower 2 (First Time)

```bash
# 1. Plug SAME USB into Tower 2
# 2. Run activation
sudo bash /media/usb/biomeOS-LAN-Deploy/scripts/activate-tower.sh

# Output:
# 🧬 First-Time Tower Setup
# Tower name [tower1]: tower2  ← Different name
# Tower IP [192.168.1.134]: 
# ✅ Tower configuration saved
# 📦 Copying genetic material...
# 🐻 Activating BearDog...
# ✅ Family verified: iidn  ← SAME family!
# 🐦 Activating Songbird...
# 🎊 Tower Activation Complete!
# 🧬 This tower is now part of the iidn family!
```

### Result: Auto-Federation!

**Both towers**:
- Same family DNA (`iidn`)
- Unique tower identities
- Automatic discovery
- Auto-trust (same family)
- 🎊 **FEDERATION!** 🎊

---

## 🔐 Security Benefits

### Tower-Specific Seeds

Each tower has **two seeds**:
1. **Family Seed** (shared): Proves family membership
2. **Tower Seed** (unique): Proves tower identity

**Result**: Same family, unique identities

### Privacy

- Different families can't see each other
- Same family auto-trusts
- Tower seeds prevent impersonation

---

## 📊 Comparison: Old vs New

### Old Way (Manual)

```bash
# On each tower:
export BEARDOG_FAMILY_ID="iidn"
export BEARDOG_FAMILY_SEED="..."
./beardog-server &

export SONGBIRD_BEARDOG_URL="http://localhost:9000"
./songbird-orchestrator &

# Repeat for every tower... tedious!
```

### New Way (Live Spore)

```bash
# On ANY tower:
sudo bash /media/usb/activate-tower.sh

# Done! Tower joins family automatically!
```

---

## 🎯 Next Steps to Implement

1. **Create USB Structure**:
   ```bash
   cd /media/usb/biomeOS-LAN-Deploy
   mkdir -p scripts configs
   ```

2. **Add Scripts**:
   - `scripts/activate-tower.sh`
   - `scripts/setup-tower-config.sh`
   - `scripts/tower-status.sh`

3. **Add Family Config**:
   ```bash
   # configs/family-seed.conf
   FAMILY_ID="iidn"
   FAMILY_SEED="iIDnVX3Tein1LFkrkkq7Wo3wsxPNek9XZqp0VL4Kn88="
   ```

4. **Update Checksums**:
   ```bash
   cd primals/
   sha256sum beardog-server songbird-orchestrator petal-tongue > CHECKSUMS.txt
   ```

5. **Test**:
   - Plug USB into Tower 1 → Run script → Verify
   - Plug USB into Tower 2 → Run script → Verify
   - Wait 30s → Check for federation! 🎊

---

## 🏆 Benefits

✅ **One USB, Many Towers**: Same USB activates all towers  
✅ **Automatic Configuration**: First-time setup wizard  
✅ **Unique Identities**: Each tower gets unique seed  
✅ **Shared Family**: All towers join same family  
✅ **Secure by Default**: Tower + family seeds  
✅ **Easy Updates**: Re-run script to update binaries  
✅ **Status Monitoring**: Built-in status script  

---

**Concept**: 🧬 **Live Spore USB**  
**Result**: Plug → Activate → Federate!  
**Time**: ~2 minutes per tower  

🎵 **One USB to bind them, one family to find them!** 🎵

