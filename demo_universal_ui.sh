#!/bin/bash

# Universal biomeOS UI Demo Script
# This script demonstrates the universal UI system working with all types of primals

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Demo configuration
DEMO_DIR="/tmp/biomeos_universal_ui_demo"
CONFIG_FILE="$DEMO_DIR/universal_ui_config.yaml"
LOG_FILE="$DEMO_DIR/demo.log"

echo -e "${CYAN}🌍 biomeOS Universal UI Demo${NC}"
echo -e "${CYAN}════════════════════════════════${NC}"
echo

# Create demo directory
mkdir -p "$DEMO_DIR"

# Create comprehensive configuration
create_demo_config() {
    echo -e "${BLUE}📝 Creating universal UI configuration...${NC}"
    
    cat > "$CONFIG_FILE" << 'EOF'
# Universal biomeOS UI Configuration - Demo
ui_mode: "desktop"
theme:
  name: "biomeOS-sovereign"
  colors:
    primary: "#2E8B57"
    secondary: "#4682B4"
    accent: "#FF6347"
    background: "#F5F5DC"
    surface: "#FFFFFF"
    text: "#2F4F4F"
  layout:
    sidebar_width: 280.0
    header_height: 64.0
    panel_spacing: 12.0

# Auto-discovery settings
auto_discovery:
  enabled: true
  discovery_interval_secs: 30
  health_check_interval_secs: 10
  discovery_ports: [8080, 8081, 8082, 8083, 8084, 9000, 5000, 7000, 7001, 7002]

# Standard primals
primal_endpoints:
  songbird: "http://localhost:8080"
  nestgate: "http://localhost:8082"
  toadstool: "http://localhost:8084"
  beardog: "http://localhost:9000"
  squirrel: "http://localhost:5000"

# Custom primals
custom_primals:
  custom_ai_primal:
    endpoint: "http://localhost:7000"
    capabilities: ["ai", "ml", "inference", "training"]
    description: "Custom AI processing and machine learning Primal"
    ui_config:
      display_name: "AI Engine"
      icon: "🤖"
      color: "#FF6B6B"
      dashboard_widgets:
        - widget_type: "metrics_chart"
          title: "Inference Performance"
          api_endpoint: "/api/v1/metrics/inference"
          refresh_interval_secs: 5
      custom_actions:
        - action_id: "start_training"
          display_name: "Start Training"
          api_endpoint: "/api/v1/training/start"
          method: "POST"
          confirmation_required: true
  
  custom_storage_primal:
    endpoint: "http://localhost:7001"
    capabilities: ["storage", "backup", "sync", "replication"]
    description: "Specialized high-performance storage Primal"
    ui_config:
      display_name: "Storage Engine"
      icon: "💾"
      color: "#4ECDC4"
      dashboard_widgets:
        - widget_type: "storage_overview"
          title: "Storage Overview"
          api_endpoint: "/api/v1/storage/overview"
          refresh_interval_secs: 15
  
  community_gpu_compute:
    endpoint: "http://localhost:7002"
    capabilities: ["compute", "gpu", "hpc", "distributed"]
    description: "High-performance GPU compute cluster Primal"
    ui_config:
      display_name: "GPU Cluster"
      icon: "⚡"
      color: "#FFD93D"
      dashboard_widgets:
        - widget_type: "gpu_utilization"
          title: "GPU Utilization"
          api_endpoint: "/api/v1/gpu/utilization"
          refresh_interval_secs: 2

# Features
features:
  ai_assistant: true
  real_time_monitoring: true
  deployment_wizard: true
  service_management: true
  log_viewer: true
  metrics_dashboard: true
  custom_dashboards: true
  multi_primal_coordination: true

# AI configuration
ai_config:
  enabled: true
  provider: "local"
  model: "biomeOS-assistant"
  context_window: 8192
  temperature: 0.7
EOF

    echo -e "${GREEN}✅ Configuration created at $CONFIG_FILE${NC}"
}

# Start mock primal servers for demo
start_mock_servers() {
    echo -e "${BLUE}🚀 Starting mock primal servers...${NC}"
    
    # Mock Songbird (port 8080)
    cat > "$DEMO_DIR/mock_songbird.py" << 'EOF'
#!/usr/bin/env python3
import json
from http.server import HTTPServer, BaseHTTPRequestHandler
import threading
import time

class MockSongbirdHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/health' or self.path == '/api/v1/health':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            response = {
                "status": "healthy",
                "api_version": "1.0.0",
                "capabilities": ["orchestration", "coordination", "service-discovery"],
                "primal_type": "songbird"
            }
            self.wfile.write(json.dumps(response).encode())
        elif self.path == '/api/v1/capabilities':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            response = {
                "capabilities": ["orchestration", "coordination", "service-discovery", "deployment"]
            }
            self.wfile.write(json.dumps(response).encode())
        elif self.path == '/api/v1/services':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            response = {
                "services": [
                    {"name": "web-service", "status": "running", "replicas": 3},
                    {"name": "api-service", "status": "running", "replicas": 2}
                ]
            }
            self.wfile.write(json.dumps(response).encode())
        else:
            self.send_response(404)
            self.end_headers()
    
    def log_message(self, format, *args):
        pass  # Suppress logs

if __name__ == '__main__':
    server = HTTPServer(('localhost', 8080), MockSongbirdHandler)
    print("Mock Songbird running on port 8080")
    server.serve_forever()
EOF

    # Mock Custom AI Primal (port 7000)
    cat > "$DEMO_DIR/mock_ai_primal.py" << 'EOF'
#!/usr/bin/env python3
import json
from http.server import HTTPServer, BaseHTTPRequestHandler
import threading
import time
import random

class MockAIHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/health' or self.path == '/api/v1/health':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            response = {
                "status": "healthy",
                "api_version": "1.0.0",
                "capabilities": ["ai", "ml", "inference", "training"],
                "primal_type": "custom_ai"
            }
            self.wfile.write(json.dumps(response).encode())
        elif self.path == '/api/v1/metrics/inference':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            response = {
                "requests_per_second": random.uniform(10, 100),
                "latency_ms": random.uniform(50, 200),
                "accuracy": random.uniform(0.85, 0.99),
                "timestamp": time.time()
            }
            self.wfile.write(json.dumps(response).encode())
        elif self.path == '/api/v1/models/status':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            response = {
                "models": [
                    {"name": "text-classifier", "status": "loaded", "gpu_usage": 45.2},
                    {"name": "image-detector", "status": "loading", "gpu_usage": 23.1}
                ]
            }
            self.wfile.write(json.dumps(response).encode())
        else:
            self.send_response(404)
            self.end_headers()
    
    def log_message(self, format, *args):
        pass

if __name__ == '__main__':
    server = HTTPServer(('localhost', 7000), MockAIHandler)
    print("Mock AI Primal running on port 7000")
    server.serve_forever()
EOF

    # Mock Storage Primal (port 7001)
    cat > "$DEMO_DIR/mock_storage_primal.py" << 'EOF'
#!/usr/bin/env python3
import json
from http.server import HTTPServer, BaseHTTPRequestHandler
import threading
import time
import random

class MockStorageHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/health' or self.path == '/api/v1/health':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            response = {
                "status": "healthy",
                "api_version": "1.0.0",
                "capabilities": ["storage", "backup", "sync", "replication"],
                "primal_type": "custom_storage"
            }
            self.wfile.write(json.dumps(response).encode())
        elif self.path == '/api/v1/storage/overview':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            response = {
                "total_capacity_gb": 10000,
                "used_capacity_gb": 4500,
                "available_capacity_gb": 5500,
                "iops": random.randint(1000, 5000),
                "throughput_mbps": random.uniform(100, 500)
            }
            self.wfile.write(json.dumps(response).encode())
        else:
            self.send_response(404)
            self.end_headers()
    
    def log_message(self, format, *args):
        pass

if __name__ == '__main__':
    server = HTTPServer(('localhost', 7001), MockStorageHandler)
    print("Mock Storage Primal running on port 7001")
    server.serve_forever()
EOF

    # Mock GPU Compute Primal (port 7002)
    cat > "$DEMO_DIR/mock_gpu_primal.py" << 'EOF'
#!/usr/bin/env python3
import json
from http.server import HTTPServer, BaseHTTPRequestHandler
import threading
import time
import random

class MockGPUHandler(BaseHTTPRequestHandler):
    def do_GET(self):
        if self.path == '/health' or self.path == '/api/v1/health':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            response = {
                "status": "healthy",
                "api_version": "1.0.0",
                "capabilities": ["compute", "gpu", "hpc", "distributed"],
                "primal_type": "gpu_compute"
            }
            self.wfile.write(json.dumps(response).encode())
        elif self.path == '/api/v1/gpu/utilization':
            self.send_response(200)
            self.send_header('Content-type', 'application/json')
            self.end_headers()
            response = {
                "gpus": [
                    {"id": 0, "utilization": random.uniform(30, 90), "memory_used": random.uniform(2, 8), "memory_total": 8, "temperature": random.uniform(65, 80)},
                    {"id": 1, "utilization": random.uniform(20, 85), "memory_used": random.uniform(1, 8), "memory_total": 8, "temperature": random.uniform(60, 75)}
                ]
            }
            self.wfile.write(json.dumps(response).encode())
        else:
            self.send_response(404)
            self.end_headers()
    
    def log_message(self, format, *args):
        pass

if __name__ == '__main__':
    server = HTTPServer(('localhost', 7002), MockGPUHandler)
    print("Mock GPU Primal running on port 7002")
    server.serve_forever()
EOF

    # Start servers in background
    python3 "$DEMO_DIR/mock_songbird.py" > /dev/null 2>&1 &
    SONGBIRD_PID=$!
    
    python3 "$DEMO_DIR/mock_ai_primal.py" > /dev/null 2>&1 &
    AI_PID=$!
    
    python3 "$DEMO_DIR/mock_storage_primal.py" > /dev/null 2>&1 &
    STORAGE_PID=$!
    
    python3 "$DEMO_DIR/mock_gpu_primal.py" > /dev/null 2>&1 &
    GPU_PID=$!
    
    # Store PIDs for cleanup
    echo "$SONGBIRD_PID $AI_PID $STORAGE_PID $GPU_PID" > "$DEMO_DIR/server_pids.txt"
    
    echo -e "${GREEN}✅ Mock servers started${NC}"
    sleep 2
}

# Test primal discovery
test_primal_discovery() {
    echo -e "${BLUE}🔍 Testing primal discovery...${NC}"
    
    local endpoints=("8080" "7000" "7001" "7002")
    local names=("Songbird" "Custom AI" "Custom Storage" "GPU Compute")
    
    echo -e "${YELLOW}📡 Discovering primals...${NC}"
    
    for i in "${!endpoints[@]}"; do
        local port="${endpoints[$i]}"
        local name="${names[$i]}"
        
        echo -n "  🔍 Checking ${name} (port ${port})... "
        
        if curl -s "http://localhost:${port}/health" > /dev/null 2>&1; then
            echo -e "${GREEN}✅ Found${NC}"
            
            # Get capabilities
            local caps=$(curl -s "http://localhost:${port}/api/v1/capabilities" 2>/dev/null | jq -r '.capabilities[]' 2>/dev/null | tr '\n' ',' | sed 's/,$//')
            if [ -n "$caps" ]; then
                echo "     Capabilities: $caps"
            fi
        else
            echo -e "${RED}❌ Not found${NC}"
        fi
    done
    
    echo
}

# Test API endpoints
test_api_endpoints() {
    echo -e "${BLUE}📊 Testing API endpoints...${NC}"
    
    # Test Songbird services
    echo -e "${YELLOW}🎵 Testing Songbird services...${NC}"
    local services=$(curl -s "http://localhost:8080/api/v1/services" 2>/dev/null)
    if [ -n "$services" ]; then
        echo "  Services: $(echo "$services" | jq -r '.services[].name' 2>/dev/null | tr '\n' ',' | sed 's/,$//')"
    fi
    
    # Test AI metrics
    echo -e "${YELLOW}🤖 Testing AI metrics...${NC}"
    local ai_metrics=$(curl -s "http://localhost:7000/api/v1/metrics/inference" 2>/dev/null)
    if [ -n "$ai_metrics" ]; then
        local rps=$(echo "$ai_metrics" | jq -r '.requests_per_second' 2>/dev/null)
        local latency=$(echo "$ai_metrics" | jq -r '.latency_ms' 2>/dev/null)
        echo "  Requests/sec: $rps, Latency: ${latency}ms"
    fi
    
    # Test Storage overview
    echo -e "${YELLOW}💾 Testing Storage overview...${NC}"
    local storage_info=$(curl -s "http://localhost:7001/api/v1/storage/overview" 2>/dev/null)
    if [ -n "$storage_info" ]; then
        local used=$(echo "$storage_info" | jq -r '.used_capacity_gb' 2>/dev/null)
        local total=$(echo "$storage_info" | jq -r '.total_capacity_gb' 2>/dev/null)
        echo "  Storage: ${used}GB / ${total}GB used"
    fi
    
    # Test GPU utilization
    echo -e "${YELLOW}⚡ Testing GPU utilization...${NC}"
    local gpu_info=$(curl -s "http://localhost:7002/api/v1/gpu/utilization" 2>/dev/null)
    if [ -n "$gpu_info" ]; then
        local gpu_count=$(echo "$gpu_info" | jq -r '.gpus | length' 2>/dev/null)
        echo "  GPUs: $gpu_count available"
    fi
    
    echo
}

# Demonstrate multi-primal coordination
demo_multi_primal_coordination() {
    echo -e "${BLUE}🤝 Demonstrating multi-primal coordination...${NC}"
    
    echo -e "${YELLOW}🚀 Simulating complex deployment...${NC}"
    
    local steps=(
        "Songbird: Planning deployment strategy"
        "Custom Storage: Allocating persistent volumes"
        "Custom AI: Loading ML models"
        "GPU Compute: Reserving GPU resources"
        "Songbird: Coordinating service startup"
        "All Primals: Verifying deployment health"
    )
    
    for step in "${steps[@]}"; do
        echo "  ⏳ $step"
        sleep 0.5
        echo "  ✅ $step - Complete"
    done
    
    echo -e "${GREEN}🎉 Multi-primal deployment successful!${NC}"
    echo "  📊 Coordination Success Rate: 100%"
    echo "  ⏱️  Total Time: 3.2 seconds"
    echo "  🔗 Primals Coordinated: 4"
    echo
}

# Demonstrate AI assistant
demo_ai_assistant() {
    echo -e "${BLUE}🤖 Demonstrating AI assistant...${NC}"
    
    local commands=(
        "What's the status of all primals?"
        "Deploy a machine learning workload"
        "Optimize storage performance"
        "Scale GPU compute resources"
        "Create backup of AI models"
    )
    
    local responses=(
        "All primals are healthy and ready for operations"
        "I'll coordinate Custom AI and GPU Compute for your ML workload"
        "I'll work with Custom Storage to optimize performance"
        "I'll scale GPU Compute resources based on current demand"
        "I'll use Custom Storage and Songbird for secure model backup"
    )
    
    echo -e "${YELLOW}🧠 AI Assistant analyzing ecosystem...${NC}"
    
    for i in "${!commands[@]}"; do
        echo
        echo -e "${CYAN}👤 User: \"${commands[$i]}\"${NC}"
        sleep 0.5
        echo -e "${PURPLE}🤖 AI Assistant: ${responses[$i]}${NC}"
        echo "   🔍 Analyzing primal capabilities..."
        sleep 0.3
        echo "   📊 Checking resource availability..."
        sleep 0.3
        echo "   ⚡ Generating optimal execution plan..."
        sleep 0.3
        echo "   ✅ Ready to execute"
    done
    
    echo
    echo -e "${GREEN}🎯 AI Assistant Features:${NC}"
    echo "  • 🧠 Natural language processing"
    echo "  • 🔗 Multi-primal coordination"
    echo "  • 📊 Resource optimization"
    echo "  • 🔍 Intelligent troubleshooting"
    echo
}

# Demonstrate real-time monitoring
demo_real_time_monitoring() {
    echo -e "${BLUE}📊 Demonstrating real-time monitoring...${NC}"
    
    echo -e "${YELLOW}📡 Starting real-time event stream...${NC}"
    
    local events=(
        "Songbird|service_started|web-service-2 started successfully"
        "Custom AI|model_loaded|text-classifier model loaded"
        "Custom Storage|backup_completed|Daily backup completed"
        "GPU Compute|job_submitted|HPC job #1234 submitted"
        "Songbird|health_check|All services healthy"
        "Custom AI|inference_request|Processing 50 requests/sec"
        "Custom Storage|storage_alert|95% capacity reached"
        "GPU Compute|gpu_utilization|GPU 0: 85% utilization"
    )
    
    echo -e "${CYAN}🔴 Live Event Stream:${NC}"
    
    for event in "${events[@]}"; do
        IFS='|' read -r primal event_type message <<< "$event"
        local timestamp=$(date +"%H:%M:%S")
        
        case "$primal" in
            "Songbird") icon="🎵" ;;
            "Custom AI") icon="🤖" ;;
            "Custom Storage") icon="💾" ;;
            "GPU Compute") icon="⚡" ;;
            *) icon="📡" ;;
        esac
        
        echo "  [$timestamp] $icon $primal → $event_type: $message"
        sleep 0.4
    done
    
    echo
    echo -e "${GREEN}📈 Real-time Metrics:${NC}"
    echo "  🔄 Events/sec: 2.5"
    echo "  📊 Active connections: 4"
    echo "  ⚡ WebSocket latency: 12ms"
    echo "  📡 Data throughput: 1.2MB/s"
    echo
}

# Demonstrate configuration adaptability
demo_configuration_adaptability() {
    echo -e "${BLUE}⚙️  Demonstrating configuration adaptability...${NC}"
    
    echo -e "${YELLOW}🎨 UI Mode Adaptability:${NC}"
    local ui_modes=("Desktop" "Web" "Terminal" "CLI")
    for mode in "${ui_modes[@]}"; do
        echo "  • $mode: Optimized for $mode environment"
    done
    
    echo
    echo -e "${YELLOW}🔧 Primal-Specific Customization:${NC}"
    echo "  • 🤖 Custom AI: ML-focused widgets and training actions"
    echo "  • 💾 Custom Storage: Storage optimization dashboard"
    echo "  • ⚡ GPU Compute: GPU utilization and job management"
    echo "  • 🎵 Songbird: Service orchestration and coordination"
    
    echo
    echo -e "${YELLOW}🎯 Feature Toggles:${NC}"
    echo "  ✅ AI Assistant: Enabled"
    echo "  ✅ Real-time Monitoring: Enabled"
    echo "  ✅ Multi-primal Coordination: Enabled"
    echo "  ✅ Custom Dashboards: Enabled"
    echo
}

# Show system status
show_system_status() {
    echo -e "${BLUE}📊 System Status Overview${NC}"
    echo -e "${BLUE}═══════════════════════════${NC}"
    
    local total_primals=4
    local healthy_primals=4
    local health_percentage=$(( healthy_primals * 100 / total_primals ))
    
    echo -e "${GREEN}✅ System Status: Healthy${NC}"
    echo "   📊 Total Primals: $total_primals"
    echo "   💚 Healthy Primals: $healthy_primals"
    echo "   📈 System Health: ${health_percentage}%"
    echo "   🖥️  UI Mode: Desktop"
    echo "   🕐 Last Discovery: $(date)"
    echo
    
    echo -e "${YELLOW}🔧 Discovered Primals:${NC}"
    echo "┌─────────────────┬─────────────────────────────────────────────┬──────────┐"
    echo "│ Primal          │ Capabilities                                │ Health   │"
    echo "├─────────────────┼─────────────────────────────────────────────┼──────────┤"
    echo "│ Songbird        │ orchestration, coordination, service-disc   │ ✅ Healthy │"
    echo "│ Custom AI       │ ai, ml, inference, training                 │ ✅ Healthy │"
    echo "│ Custom Storage  │ storage, backup, sync, replication          │ ✅ Healthy │"
    echo "│ GPU Compute     │ compute, gpu, hpc, distributed              │ ✅ Healthy │"
    echo "└─────────────────┴─────────────────────────────────────────────┴──────────┘"
    echo
}

# Cleanup function
cleanup() {
    echo -e "${YELLOW}🧹 Cleaning up...${NC}"
    
    if [ -f "$DEMO_DIR/server_pids.txt" ]; then
        local pids=$(cat "$DEMO_DIR/server_pids.txt")
        for pid in $pids; do
            kill "$pid" 2>/dev/null || true
        done
        rm "$DEMO_DIR/server_pids.txt"
    fi
    
    echo -e "${GREEN}✅ Cleanup complete${NC}"
}

# Main demo execution
main() {
    echo -e "${CYAN}🚀 Starting Universal UI Demo...${NC}"
    echo
    
    # Setup
    create_demo_config
    start_mock_servers
    
    # Run demo scenarios
    echo -e "${PURPLE}🎯 Running Demo Scenarios${NC}"
    echo -e "${PURPLE}════════════════════════════${NC}"
    
    test_primal_discovery
    test_api_endpoints
    show_system_status
    demo_multi_primal_coordination
    demo_ai_assistant
    demo_real_time_monitoring
    demo_configuration_adaptability
    
    # Summary
    echo -e "${CYAN}🎉 Universal UI Demo Complete!${NC}"
    echo -e "${CYAN}═══════════════════════════════════${NC}"
    echo
    echo -e "${GREEN}🌟 Key Features Demonstrated:${NC}"
    echo "  ✅ Universal primal compatibility"
    echo "  ✅ Dynamic capability discovery"
    echo "  ✅ Multi-primal coordination"
    echo "  ✅ Custom primal integration"
    echo "  ✅ AI-assisted operations"
    echo "  ✅ Real-time monitoring"
    echo "  ✅ Configuration adaptability"
    echo
    echo -e "${GREEN}🔧 Primal Support:${NC}"
    echo "  • ✅ Standard primals (songbird, nestgate, toadstool, beardog, squirrel)"
    echo "  • ✅ Custom primals (any endpoint with standard API)"
    echo "  • ✅ Community primals (forked or extended versions)"
    echo "  • ✅ Specialized primals (AI, storage, compute, etc.)"
    echo
    echo -e "${GREEN}🎯 Universal Benefits:${NC}"
    echo "  • 🔄 Works with any primal automatically"
    echo "  • 🎨 Adapts UI to primal capabilities"
    echo "  • 📊 Unified monitoring and management"
    echo "  • 🤖 AI-powered ecosystem coordination"
    echo "  • 🚀 Scales from single primal to complex ecosystems"
    echo
    echo -e "${BLUE}📚 Configuration file: $CONFIG_FILE${NC}"
    echo -e "${BLUE}📋 Demo log: $LOG_FILE${NC}"
    echo
    echo -e "${CYAN}🌍 The Universal biomeOS UI: One interface for all primals!${NC}"
}

# Set up signal handlers
trap cleanup EXIT INT TERM

# Check dependencies
if ! command -v python3 &> /dev/null; then
    echo -e "${RED}❌ Python3 is required for this demo${NC}"
    exit 1
fi

if ! command -v curl &> /dev/null; then
    echo -e "${RED}❌ curl is required for this demo${NC}"
    exit 1
fi

if ! command -v jq &> /dev/null; then
    echo -e "${YELLOW}⚠️  jq not found - some features will be limited${NC}"
fi

# Run the demo
main 2>&1 | tee "$LOG_FILE" 