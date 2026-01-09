# 🌸 petalTongue GUI User Guide

**Date**: January 9, 2026  
**Version**: petalTongue v0.5.0 + biomeOS v0.1.0  
**Status**: ✅ **4 GUI Windows Open and Running**

---

## 🎊 **You Have petalTongue Running!**

You currently have **4 petalTongue windows** open, showing the Universal Representation System.

### **Window Details**
```
Window 88080389: 🌸 petalTongue - Universal Representation System (1280x655)
Window 92274693: 🌸 petalTongue - Universal Representation System (1280x655)
Window 96468997: 🌸 petalTongue - Universal Representation System (1280x655)
Window 100663301: 🌸 petalTongue - Universal Representation System (1280x655)
```

---

## 🎨 **What You're Seeing**

### **Live Topology Visualization**
Your petalTongue windows are connected to biomeOS and displaying:

- **2 Live Primals**:
  - Songbird (Communication, P2P, BTSP)
  - BearDog (Security, Encryption, Identity)
- **1 Connection**: Songbird → BearDog (Trust Relationship)
- **3 USB Spores**: Attached and ready for visualization

### **Data Source**
- **biomeOS API**: http://localhost:3000/api/v1/topology
- **Mode**: LIVE (discovering real primals)
- **Update Rate**: Real-time

---

## 🖱️ **How to Interact with the GUI**

### **Basic Navigation**
- **Mouse Wheel**: Zoom in/out
- **Click & Drag**: Pan the graph
- **Click on Primal**: See detailed information
- **Double-Click**: Center on primal

### **What You Can See**
1. **Primal Nodes** - Circles representing each primal
2. **Connections** - Lines showing relationships
3. **Health Status** - Color-coded (green=healthy, yellow=degraded, red=unhealthy)
4. **Metadata** - Hover to see version, endpoints, capabilities

### **Current View**
```
     ┌──────────┐
     │ Songbird │ (Communication)
     └────┬─────┘
          │ Trust Relationship
          │ (lineage_verification)
          ↓
     ┌──────────┐
     │ BearDog  │ (Security)
     └──────────┘
```

---

## 📊 **What the GUI Shows**

### **Primal Information**
Each primal node displays:
- **Name**: Songbird, BearDog, etc.
- **Type**: Communication, Security, Compute, Data
- **Health**: Healthy, Degraded, Unhealthy, Unknown
- **Capabilities**: What the primal can do
- **Endpoints**: Unix socket, HTTP, UDP addresses
- **Metadata**: Version, family ID, node ID, trust level

### **Connection Information**
Each connection line shows:
- **Type**: API call, Federation, Trust relationship
- **Capability**: What capability is being used
- **Metrics**: Request count, latency, packet loss

### **Health Status Color Coding**
- 🟢 **Green**: Healthy and operational
- 🟡 **Yellow**: Degraded but functional
- 🔴 **Red**: Unhealthy or down
- ⚪ **Gray**: Unknown status

---

## 🧬 **USB Spore Visualization** (Future)

### **Current Status**
- **3 USB Spores Detected**:
  - `/media/eastgate/BEA6-BBCE` - Spore with biomeOS deployment
  - `/media/eastgate/biomeOS1` - Spore with biomeOS deployment
  - `/media/eastgate/BEA6-BBCE1` - Spore with biomeOS deployment

### **Future Visualization**
When fully integrated, petalTongue will show:
- Spore genetic lineage
- Deployment status
- Sibling relationships
- Family membership
- Federation topology across spores

---

## 🚀 **Advanced Features**

### **1. Real-Time Updates**
- Topology refreshes automatically
- New primals appear as they're discovered
- Health status updates in real-time
- Connection metrics update live

### **2. Multi-Modal Display**
petalTongue supports multiple display modes:
- ✅ **Visual2D**: Current graph rendering
- ✅ **Animation**: Smooth transitions and updates
- ✅ **TextDescription**: Accessible text descriptions
- ❌ **Audio**: Sonification (requires libasound2-dev)
- ❌ **Haptic**: Tactile feedback (future)
- ❌ **VR3D**: Immersive 3D view (future)

### **3. Tutorial Mode** (If Available)
petalTongue includes a tutorial that walks you through:
- Basic navigation
- Understanding the topology
- Interacting with primals
- Reading health indicators

---

## 🔍 **Troubleshooting**

### **Can't See the Window?**
```bash
# List all petalTongue windows
xdotool search --name "petalTongue"

# Focus on a specific window
xdotool windowactivate <WINDOW_ID>
```

### **Window Not Updating?**
- Check that biomeOS API is running: `curl http://localhost:3000/api/v1/health`
- Check petalTongue logs: `tail -f logs/petaltongue-gui.log`
- Refresh by restarting petalTongue

### **Display Issues?**
```bash
# Check DISPLAY variable
echo $DISPLAY  # Should be :0 or similar

# Verify X11 is running
xdpyinfo | head -10
```

---

## 🎯 **What You Can Do Now**

### **Immediate Actions**
1. **Explore the Current Topology**
   - Look at the 2 primals (Songbird, BearDog)
   - See how they're connected
   - Check their health status

2. **Test Real-Time Updates**
   - Start/stop a primal
   - Watch the topology update
   - See health status change

3. **Inspect Primal Details**
   - Click on Songbird to see its capabilities
   - Click on BearDog to see its security features
   - Hover over connections to see metrics

### **Next Steps**
1. **Deploy More Primals**
   - Start Toadstool (compute)
   - Start NestGate (data)
   - See them appear in the topology

2. **Test Federation**
   - Deploy spore on another machine
   - Watch LAN federation form
   - See cross-machine connections

3. **Monitor Health**
   - Use GUI as dashboard
   - Watch for degraded primals
   - Track connection metrics

---

## 📚 **Running the Full Stack**

### **Start biomeOS API (LIVE Mode)**
```bash
cd /path/to/biomeOS
BIOMEOS_STANDALONE_MODE=false \
RUST_LOG=info \
cargo run --package biomeos-api

# API discovers real primals
# Endpoint: http://localhost:3000/api/v1/topology
```

### **Start petalTongue GUI**
```bash
BIOMEOS_URL=http://localhost:3000 \
PETALTONGUE_ENABLE_MDNS=false \
RUST_LOG=info \
./plasmidBin/primals/petal-tongue

# GUI opens and displays topology
```

### **Start Additional Primals**
```bash
# Start Songbird (if not running)
/path/to/songbird

# Start BearDog (if not running)
/path/to/beardog-server

# Watch them appear in petalTongue!
```

---

## 🎊 **Bottom Line**

**✅ YOU HAVE A WORKING GUI!**

- 4 petalTongue windows open
- Connected to biomeOS LIVE topology
- Visualizing 2 real primals
- Real-time updates enabled

**Look for the windows titled:**
```
🌸 petalTongue - Universal Representation System
```

**You can:**
- See your primals visually
- Monitor their health
- Track connections
- Explore the topology interactively

**The ecoPrimals ecosystem now has a beautiful, functional interface!** 🌸✨

---

## 📞 **Quick Commands**

### **Check What's Running**
```bash
# List windows
xdotool search --name "petalTongue"

# List processes
ps aux | grep -E "(biomeos-api|petal-tongue)" | grep -v grep

# Check API
curl http://localhost:3000/api/v1/topology | jq '.'
```

### **Focus a Window**
```bash
# Focus on petalTongue (replace with actual window ID)
xdotool windowactivate 88080389
```

### **Restart for Fresh View**
```bash
# Stop all
pkill -f "biomeos-api"
pkill -f "petal-tongue"

# Start biomeOS
BIOMEOS_STANDALONE_MODE=false cargo run --package biomeos-api &

# Start petalTongue
BIOMEOS_URL=http://localhost:3000 ./plasmidBin/primals/petal-tongue &
```

---

**Your GUI is live and beautiful!** Go explore it! 🌸🎨✨

