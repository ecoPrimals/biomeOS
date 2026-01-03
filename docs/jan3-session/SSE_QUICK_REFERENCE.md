# 🌊 SSE Events Quick Reference - biomeOS API

**Last Updated**: January 3, 2026  
**Endpoint**: `GET /api/v1/events/stream`  
**Format**: Server-Sent Events (SSE)

---

## 📡 Quick Start

### JavaScript / TypeScript
```javascript
// Connect to event stream
const events = new EventSource('http://localhost:3000/api/v1/events/stream');

// Handle all events
events.onmessage = (e) => {
  const event = JSON.parse(e.data);
  console.log('Event:', event.type, event);
  
  // Route by type
  handleEvent(event);
};

// Handle connection errors
events.onerror = (e) => {
  console.error('SSE connection error:', e);
  // EventSource auto-reconnects
};
```

### React Hook Example
```typescript
import { useEffect, useState } from 'react';

export function useEcosystemEvents() {
  const [events, setEvents] = useState<EcosystemEvent[]>([]);
  
  useEffect(() => {
    const eventSource = new EventSource('/api/v1/events/stream');
    
    eventSource.onmessage = (e) => {
      const event = JSON.parse(e.data);
      setEvents(prev => [...prev, event]);
    };
    
    return () => eventSource.close();
  }, []);
  
  return events;
}
```

---

## 📋 Event Types Reference

### 1. primal_discovered
**When**: New primal joins the ecosystem  
**Use**: Add node to topology graph

```typescript
interface PrimalDiscovered {
  type: "primal_discovered";
  primal_id: string;           // e.g., "beardog-local"
  name: string;                // e.g., "BearDog"
  primal_type: string;         // e.g., "Security"
  family_id: string | null;    // e.g., "iidn" or null
  capabilities: string[];      // e.g., ["btsp", "birdsong"]
}
```

**Example**:
```json
{
  "type": "primal_discovered",
  "primal_id": "beardog-local",
  "name": "BearDog",
  "primal_type": "Security",
  "family_id": "iidn",
  "capabilities": ["btsp", "birdsong", "lineage"]
}
```

**UI Action**:
```javascript
case 'primal_discovered':
  addNodeToGraph({
    id: event.primal_id,
    label: event.name,
    type: event.primal_type,
    family: event.family_id,
    capabilities: event.capabilities
  });
  break;
```

---

### 2. health_changed
**When**: Primal health status changes  
**Use**: Update node color/status indicator

```typescript
interface HealthChanged {
  type: "health_changed";
  primal_id: string;
  name: string;
  old_health: string;  // "Healthy" | "Degraded" | "Unknown"
  new_health: string;
}
```

**Example**:
```json
{
  "type": "health_changed",
  "primal_id": "songbird-local",
  "name": "Songbird",
  "old_health": "Healthy",
  "new_health": "Degraded"
}
```

**UI Action**:
```javascript
case 'health_changed':
  updateNodeColor(event.primal_id, {
    'Healthy': 'green',
    'Degraded': 'yellow',
    'Unknown': 'gray'
  }[event.new_health]);
  
  if (event.new_health === 'Degraded') {
    showAlert(`⚠️ ${event.name} health degraded!`);
  }
  break;
```

---

### 3. family_joined
**When**: Primal joins a genetic family  
**Use**: Highlight family relationships, draw family edges

```typescript
interface FamilyJoined {
  type: "family_joined";
  primal_id: string;
  name: string;
  family_id: string;
}
```

**Example**:
```json
{
  "type": "family_joined",
  "primal_id": "beardog-local",
  "name": "BearDog",
  "family_id": "iidn"
}
```

**UI Action**:
```javascript
case 'family_joined':
  highlightFamilyRelationship(event.primal_id, event.family_id);
  updateNodeMetadata(event.primal_id, { family: event.family_id });
  showNotification(`🎉 ${event.name} joined family ${event.family_id}`);
  break;
```

---

### 4. trust_updated
**When**: Trust level or capabilities change  
**Use**: Update trust indicators, capability badges

```typescript
interface TrustUpdated {
  type: "trust_updated";
  primal_id: string;
  name: string;
  trust_level: number;  // 0-255
}
```

**Example**:
```json
{
  "type": "trust_updated",
  "primal_id": "beardog-local",
  "name": "BearDog",
  "trust_level": 5
}
```

**UI Action**:
```javascript
case 'trust_updated':
  updateTrustIndicator(event.primal_id, event.trust_level);
  
  // Map trust level to visual indicator
  const trustColor = event.trust_level > 3 ? 'green' : 
                     event.trust_level > 1 ? 'yellow' : 'red';
  updateNodeBorder(event.primal_id, trustColor);
  break;
```

---

### 5. topology_changed
**When**: Ecosystem topology changes (node/edge added/removed)  
**Use**: Trigger graph re-layout, update stats

```typescript
interface TopologyChanged {
  type: "topology_changed";
  nodes: number;
  edges: number;
  change: string;  // "primal_added" | "primal_removed" | "edge_added"
}
```

**Example**:
```json
{
  "type": "topology_changed",
  "nodes": 5,
  "edges": 3,
  "change": "primal_added"
}
```

**UI Action**:
```javascript
case 'topology_changed':
  updateStats({ nodes: event.nodes, edges: event.edges });
  
  if (event.change === 'primal_removed') {
    triggerGraphLayout(); // Re-layout after removal
  }
  break;
```

---

### 6. heartbeat
**When**: Every 5 seconds  
**Use**: Update dashboard stats, verify connection

```typescript
interface Heartbeat {
  type: "heartbeat";
  timestamp: number;      // Unix timestamp
  primals_count: number;
  healthy_count: number;
  families: string[];
}
```

**Example**:
```json
{
  "type": "heartbeat",
  "timestamp": 1767453377,
  "primals_count": 2,
  "healthy_count": 1,
  "families": ["iidn"]
}
```

**UI Action**:
```javascript
case 'heartbeat':
  updateDashboard({
    totalPrimals: event.primals_count,
    healthyPrimals: event.healthy_count,
    families: event.families,
    lastUpdate: new Date(event.timestamp * 1000)
  });
  
  // Connection health indicator
  markConnectionAlive();
  break;
```

---

## 🎯 Complete Example

### PetalTongue Integration
```typescript
import { useEffect, useState } from 'react';
import { Graph } from 'react-graph-vis';

export function LiveTopology() {
  const [graph, setGraph] = useState({ nodes: [], edges: [] });
  const [stats, setStats] = useState({ primals: 0, healthy: 0, families: [] });
  
  useEffect(() => {
    const events = new EventSource('/api/v1/events/stream');
    
    events.onmessage = (e) => {
      const event = JSON.parse(e.data);
      
      switch (event.type) {
        case 'primal_discovered':
          // Add new node to graph
          setGraph(prev => ({
            ...prev,
            nodes: [...prev.nodes, {
              id: event.primal_id,
              label: event.name,
              color: event.family_id ? '#00ff00' : '#cccccc',
              title: `${event.name} (${event.primal_type})`
            }]
          }));
          break;
          
        case 'health_changed':
          // Update node color
          setGraph(prev => ({
            ...prev,
            nodes: prev.nodes.map(node =>
              node.id === event.primal_id
                ? { ...node, color: event.new_health === 'Healthy' ? '#00ff00' : '#ffaa00' }
                : node
            )
          }));
          break;
          
        case 'family_joined':
          // Highlight family connection
          setGraph(prev => ({
            ...prev,
            edges: [...prev.edges, {
              from: event.primal_id,
              to: 'family-' + event.family_id,
              color: '#00ff00',
              dashes: true
            }]
          }));
          break;
          
        case 'heartbeat':
          // Update dashboard stats
          setStats({
            primals: event.primals_count,
            healthy: event.healthy_count,
            families: event.families
          });
          break;
      }
    };
    
    events.onerror = () => {
      console.error('SSE connection lost, reconnecting...');
    };
    
    return () => events.close();
  }, []);
  
  return (
    <div>
      <Dashboard stats={stats} />
      <Graph graph={graph} />
    </div>
  );
}
```

---

## 🔧 Best Practices

### 1. Event Batching
```javascript
// Batch rapid updates to avoid UI thrashing
let updateQueue = [];
let updateTimer = null;

events.onmessage = (e) => {
  const event = JSON.parse(e.data);
  updateQueue.push(event);
  
  if (!updateTimer) {
    updateTimer = setTimeout(() => {
      processBatch(updateQueue);
      updateQueue = [];
      updateTimer = null;
    }, 100); // Batch for 100ms
  }
};
```

### 2. Connection Monitoring
```javascript
let lastHeartbeat = Date.now();

events.onmessage = (e) => {
  const event = JSON.parse(e.data);
  
  if (event.type === 'heartbeat') {
    lastHeartbeat = Date.now();
    updateConnectionStatus('connected');
  }
};

// Check for stale connection
setInterval(() => {
  if (Date.now() - lastHeartbeat > 15000) {
    updateConnectionStatus('disconnected');
  }
}, 5000);
```

### 3. Event Filtering (Client-Side)
```javascript
// Only process events for specific families
const myFamily = 'iidn';

events.onmessage = (e) => {
  const event = JSON.parse(e.data);
  
  // Filter by family (if applicable)
  if (event.family_id && event.family_id !== myFamily) {
    return; // Ignore other families
  }
  
  handleEvent(event);
};
```

---

## 🐛 Troubleshooting

### Connection Issues
```javascript
events.onerror = (e) => {
  console.error('SSE error:', e);
  
  // Check API health
  fetch('/api/v1/health')
    .then(r => r.json())
    .then(health => {
      if (health.status !== 'healthy') {
        showAlert('API is unhealthy, reconnecting...');
      }
    });
};
```

### Missing Events
```javascript
// Verify you're receiving heartbeats
let heartbeatCount = 0;

events.onmessage = (e) => {
  const event = JSON.parse(e.data);
  
  if (event.type === 'heartbeat') {
    heartbeatCount++;
    console.log(`Heartbeat #${heartbeatCount}`);
  }
};

// Should see heartbeat every 5 seconds
```

---

## 📚 Additional Resources

- **Full Documentation**: `docs/jan3-session/ENHANCED_SSE_EVENTS_JAN_3_2026.md`
- **API Guide**: `docs/jan3-session/QUICKSTART.md`
- **Session Summary**: `docs/jan3-session/EVENING_SESSION_COMPLETE_ENHANCED_SSE_JAN_3_2026.md`

---

**Quick Test**:
```bash
curl -N http://localhost:3000/api/v1/events/stream
```

**Expected Output**:
```
data: {"type":"primal_discovered",...}
data: {"type":"family_joined",...}
data: {"type":"heartbeat",...}
```

🌊 **Happy streaming!** 🦀🌸

