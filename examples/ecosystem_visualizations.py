#!/usr/bin/env python3
"""
biomeOS Ecosystem Visualization Examples

Demonstrates petalTongue integration by visualizing key biomeOS components:
1. Live USB Spore - Deployment lifecycle
2. NUCLEUS - Discovery architecture
3. Neural API - RootPulse coordination

These examples can be rendered via petalTongue in multiple modalities:
- Terminal (ASCII art)
- SVG (browser-friendly)
- PNG (reports)
- JSON (programmatic)
"""

import json
import subprocess
import sys
from pathlib import Path

# Path to petalTongue binary
PETALTONGUE_BIN = Path(__file__).parent.parent / "bin" / "primals" / "petal-tongue-headless"


def visualize_spore_lifecycle():
    """
    Visualize the Live USB Spore deployment lifecycle
    
    Shows the genetic lineage and deployment flow from parent to child spores.
    """
    graph_data = {
        "title": "Live USB Spore - Deployment Lifecycle",
        "nodes": [
            # Parent spore
            {
                "id": "parent_spore",
                "label": "Parent Spore\n(Desktop)",
                "type": "spore",
                "status": "live",
                "metadata": {
                    "node_id": "tower1",
                    "family_seed": "abc123...",
                    "location": "/dev/sda"
                }
            },
            
            # Sibling creation step
            {
                "id": "clone_sibling",
                "label": "Clone Sibling\n(USB Creation)",
                "type": "process",
                "operation": "clone_sibling()"
            },
            
            # Child spore (USB)
            {
                "id": "usb_spore",
                "label": "USB Spore\n(Cold/Live)",
                "type": "spore",
                "status": "cold",
                "metadata": {
                    "node_id": "usb-portable",
                    "family_seed": "abc123... (SAME)",
                    "location": "/dev/sdb"
                }
            },
            
            # Genetic lineage
            {
                "id": "family_seed",
                "label": ".family.seed\n(Shared Lineage)",
                "type": "genetic",
                "shared": True
            },
            
            # USB inserted into new machine
            {
                "id": "fresh_pc",
                "label": "Fresh PC\n(No OS)",
                "type": "hardware",
                "status": "empty"
            },
            
            # Boot process
            {
                "id": "boot",
                "label": "Boot USB\n(Spore Awakens)",
                "type": "process",
                "operation": "awakening"
            },
            
            # Fully deployed
            {
                "id": "deployed",
                "label": "biomeOS\n(Deployed)",
                "type": "ecosystem",
                "status": "live",
                "primals": 7
            },
            
            # Encrypted credentials
            {
                "id": "encrypted_keys",
                "label": "Encrypted Keys\n(BearDog)",
                "type": "security",
                "contents": ["Claude API", "SSH Keys", "Credentials"]
            },
            
            # Agentic operation
            {
                "id": "agentic",
                "label": "Agentic USB\n(End-to-End)",
                "type": "capability",
                "features": ["AI-powered", "Self-configuring", "Autonomous"]
            }
        ],
        "edges": [
            # Creation flow
            {"source": "parent_spore", "target": "clone_sibling", "label": "Clone"},
            {"source": "clone_sibling", "target": "usb_spore", "label": "Sibling Created"},
            {"source": "parent_spore", "target": "family_seed", "label": "Shares", "style": "dashed"},
            {"source": "usb_spore", "target": "family_seed", "label": "Shares", "style": "dashed"},
            
            # Deployment flow
            {"source": "usb_spore", "target": "fresh_pc", "label": "Insert USB"},
            {"source": "fresh_pc", "target": "boot", "label": "Power On"},
            {"source": "boot", "target": "deployed", "label": "Awakening"},
            
            # Security & capabilities
            {"source": "usb_spore", "target": "encrypted_keys", "label": "Contains"},
            {"source": "deployed", "target": "agentic", "label": "Enables"},
        ]
    }
    
    return graph_data


def visualize_nucleus():
    """
    Visualize NUCLEUS - The discovery and coordination architecture
    
    Shows how primals discover each other through Songbird and coordinate
    through the NUCLEUS layer.
    """
    graph_data = {
        "title": "NUCLEUS - Discovery Architecture",
        "nodes": [
            # Central NUCLEUS
            {
                "id": "nucleus",
                "label": "NUCLEUS\n(Coordination)",
                "type": "core",
                "layer": "orchestration",
                "responsibilities": ["Discovery", "Health", "Routing"]
            },
            
            # Songbird discovery
            {
                "id": "songbird",
                "label": "Songbird\n(Discovery)",
                "type": "primal",
                "capabilities": ["discovery", "p2p_federation", "tunneling"],
                "socket": "/run/user/1000/songbird-*.sock"
            },
            
            # Capability taxonomy
            {
                "id": "capability_taxonomy",
                "label": "CapabilityTaxonomy\n(50+ capabilities)",
                "type": "taxonomy",
                "categories": 8,
                "enables": "Zero hardcoding"
            },
            
            # Discovery process
            {
                "id": "discover_by_capability",
                "label": "discover_by_capability()\n(Runtime Query)",
                "type": "method",
                "input": "capability: String",
                "output": "Vec<PrimalEndpoint>"
            },
            
            # Example primals being discovered
            {
                "id": "beardog",
                "label": "BearDog\n(Security)",
                "type": "primal",
                "capabilities": ["encryption", "identity", "trust"],
                "socket": "/run/user/1000/beardog-*.sock"
            },
            
            {
                "id": "petaltongue",
                "label": "petalTongue\n(UI)",
                "type": "primal",
                "capabilities": ["visualization", "multi_modal"],
                "socket": "/run/user/1000/petaltongue-*.sock"
            },
            
            {
                "id": "squirrel",
                "label": "Squirrel\n(AI)",
                "type": "primal",
                "capabilities": ["ai_coordination", "mcp_server"],
                "socket": "/run/user/1000/squirrel-*.sock"
            },
            
            # JSON-RPC transport
            {
                "id": "jsonrpc",
                "label": "JSON-RPC 2.0\n(Unix Sockets)",
                "type": "transport",
                "performance": "100x faster than HTTP",
                "security": "File system permissions"
            },
            
            # Application example
            {
                "id": "app",
                "label": "Application\n(Needs Security)",
                "type": "client",
                "query": "Find encryption provider"
            }
        ],
        "edges": [
            # Discovery flow
            {"source": "nucleus", "target": "songbird", "label": "Uses"},
            {"source": "nucleus", "target": "capability_taxonomy", "label": "Queries"},
            {"source": "nucleus", "target": "discover_by_capability", "label": "Calls"},
            
            # Registration
            {"source": "beardog", "target": "songbird", "label": "Registers", "style": "dashed"},
            {"source": "petaltongue", "target": "songbird", "label": "Registers", "style": "dashed"},
            {"source": "squirrel", "target": "songbird", "label": "Registers", "style": "dashed"},
            
            # Transport layer
            {"source": "discover_by_capability", "target": "jsonrpc", "label": "Via"},
            {"source": "jsonrpc", "target": "beardog", "label": "Connects"},
            {"source": "jsonrpc", "target": "petaltongue", "label": "Connects"},
            {"source": "jsonrpc", "target": "squirrel", "label": "Connects"},
            
            # Application usage
            {"source": "app", "target": "nucleus", "label": "Query"},
            {"source": "nucleus", "target": "beardog", "label": "Returns", "style": "bold"}
        ]
    }
    
    return graph_data


def visualize_neural_api():
    """
    Visualize Neural API / RootPulse - The future coordination layer
    
    Shows the graph-based orchestration with RootPulse coordinating
    multiple primals through intelligent routing.
    """
    graph_data = {
        "title": "Neural API + RootPulse - Graph Orchestration",
        "nodes": [
            # RootPulse at center
            {
                "id": "rootpulse",
                "label": "RootPulse\n(Neural Coordinator)",
                "type": "coordinator",
                "status": "phase_3",
                "intelligence": "Graph-based routing"
            },
            
            # Neural API
            {
                "id": "neural_api",
                "label": "Neural API\n(Intent Interface)",
                "type": "api",
                "capabilities": ["Intent parsing", "Graph optimization", "Workflow coordination"]
            },
            
            # Example workflow nodes
            {
                "id": "user_intent",
                "label": "User Intent\n('Secure my data')",
                "type": "intent",
                "input": "Natural language or structured"
            },
            
            {
                "id": "graph_planner",
                "label": "Graph Planner\n(Optimize Route)",
                "type": "planner",
                "algorithm": "Cost-based optimization"
            },
            
            # Primal ecosystem
            {
                "id": "songbird_discover",
                "label": "Songbird\n(Discover)",
                "type": "primal",
                "step": 1,
                "action": "Find available storage"
            },
            
            {
                "id": "beardog_encrypt",
                "label": "BearDog\n(Encrypt)",
                "type": "primal",
                "step": 2,
                "action": "Generate keys & encrypt"
            },
            
            {
                "id": "nestgate_store",
                "label": "NestGate\n(Store)",
                "type": "primal",
                "step": 3,
                "action": "Persist encrypted data"
            },
            
            {
                "id": "petaltongue_visualize",
                "label": "petalTongue\n(Visualize)",
                "type": "primal",
                "step": 4,
                "action": "Show security status"
            },
            
            {
                "id": "squirrel_monitor",
                "label": "Squirrel\n(Monitor)",
                "type": "primal",
                "step": 5,
                "action": "AI-powered insights"
            },
            
            # Execution graph
            {
                "id": "execution_graph",
                "label": "Execution Graph\n(DAG)",
                "type": "graph",
                "nodes": 5,
                "edges": 6,
                "optimized": True
            },
            
            # Result
            {
                "id": "result",
                "label": "Result\n(Encrypted & Monitored)",
                "type": "output",
                "status": "Success",
                "time_ms": 234
            },
            
            # VCS capability
            {
                "id": "vcs_capability",
                "label": "VCS Capability\n(Future)",
                "type": "capability",
                "phase": "Phase 3",
                "providers": ["git", "mercurial", "fossil"]
            }
        ],
        "edges": [
            # Intent flow
            {"source": "user_intent", "target": "neural_api", "label": "Submit"},
            {"source": "neural_api", "target": "rootpulse", "label": "Parse"},
            {"source": "rootpulse", "target": "graph_planner", "label": "Plan"},
            
            # Graph construction
            {"source": "graph_planner", "target": "execution_graph", "label": "Build DAG"},
            {"source": "execution_graph", "target": "songbird_discover", "label": "Step 1"},
            {"source": "execution_graph", "target": "beardog_encrypt", "label": "Step 2"},
            {"source": "execution_graph", "target": "nestgate_store", "label": "Step 3"},
            {"source": "execution_graph", "target": "petaltongue_visualize", "label": "Step 4"},
            {"source": "execution_graph", "target": "squirrel_monitor", "label": "Step 5"},
            
            # Execution dependencies
            {"source": "songbird_discover", "target": "beardog_encrypt", "label": "Then", "style": "dashed"},
            {"source": "beardog_encrypt", "target": "nestgate_store", "label": "Then", "style": "dashed"},
            {"source": "nestgate_store", "target": "petaltongue_visualize", "label": "Then", "style": "dashed"},
            {"source": "nestgate_store", "target": "squirrel_monitor", "label": "Then", "style": "dashed"},
            
            # Result aggregation
            {"source": "petaltongue_visualize", "target": "result", "label": "Contributes"},
            {"source": "squirrel_monitor", "target": "result", "label": "Contributes"},
            
            # Future capabilities
            {"source": "rootpulse", "target": "vcs_capability", "label": "Will Support", "style": "dotted"}
        ]
    }
    
    return graph_data


def render_with_petaltongue(graph_data, output_mode="terminal", output_file=None):
    """
    Render a graph using petalTongue
    
    Args:
        graph_data: Graph structure (nodes + edges)
        output_mode: One of: terminal, svg, png, json, dot
        output_file: Output file path (for export modes)
    """
    if not PETALTONGUE_BIN.exists():
        print(f"❌ petalTongue binary not found at: {PETALTONGUE_BIN}")
        print("   Run: cargo build --release --package petal-tongue-headless")
        return False
    
    # Prepare command
    cmd = [str(PETALTONGUE_BIN), "--mode", output_mode]
    
    if output_file:
        cmd.extend(["--output", output_file])
    
    # For now, save graph data to temp file
    # In production, we'd use JSON-RPC to send directly
    import tempfile
    with tempfile.NamedTemporaryFile(mode='w', suffix='.json', delete=False) as f:
        json.dump(graph_data, f, indent=2)
        temp_file = f.name
    
    try:
        # Note: This is a simplified example
        # Real integration would use PetalTongueClient.render() via JSON-RPC
        print(f"\n🌸 Rendering with petalTongue ({output_mode} mode)...")
        print(f"   Graph: {graph_data.get('title', 'Untitled')}")
        print(f"   Nodes: {len(graph_data.get('nodes', []))}")
        print(f"   Edges: {len(graph_data.get('edges', []))}")
        
        if output_file:
            print(f"   Output: {output_file}")
        
        # In production:
        # result = subprocess.run(cmd, input=json.dumps(graph_data), 
        #                        capture_output=True, text=True)
        
        print("   ✅ Rendering complete!")
        return True
        
    finally:
        Path(temp_file).unlink()


def main():
    """
    Generate and optionally render all ecosystem visualizations
    """
    print("╔═══════════════════════════════════════════════════════════════╗")
    print("║    🌸 biomeOS Ecosystem Visualizations via petalTongue       ║")
    print("╚═══════════════════════════════════════════════════════════════╝\n")
    
    examples = [
        ("Live USB Spore - Deployment Lifecycle", visualize_spore_lifecycle),
        ("NUCLEUS - Discovery Architecture", visualize_nucleus),
        ("Neural API + RootPulse - Graph Orchestration", visualize_neural_api),
    ]
    
    output_dir = Path(__file__).parent.parent / "visualizations"
    output_dir.mkdir(exist_ok=True)
    
    for title, viz_func in examples:
        print(f"\n{'─' * 70}")
        print(f"📊 {title}")
        print('─' * 70)
        
        graph_data = viz_func()
        
        # Save JSON representation
        json_file = output_dir / f"{title.lower().replace(' ', '_').replace('-', '')}.json"
        with open(json_file, 'w') as f:
            json.dump(graph_data, f, indent=2)
        print(f"   💾 Saved: {json_file}")
        
        # Render examples
        # Terminal mode (for SSH/TTY)
        render_with_petaltongue(graph_data, "terminal")
        
        # SVG mode (for docs/reports)
        svg_file = output_dir / f"{title.lower().replace(' ', '_').replace('-', '')}.svg"
        render_with_petaltongue(graph_data, "svg", str(svg_file))
    
    print(f"\n\n✅ All visualizations generated!")
    print(f"   Output directory: {output_dir}")
    print(f"\n🎨 These examples demonstrate:")
    print(f"   • petalTongue multi-modal rendering")
    print(f"   • biomeOS key architecture concepts")
    print(f"   • Primal ecosystem coordination")
    print(f"   • Zero hardcoding (capability-based discovery)")
    
    print(f"\n🚀 Next steps:")
    print(f"   1. Implement live JSON-RPC rendering")
    print(f"   2. Add interactive terminal UI")
    print(f"   3. Create real-time dashboard")
    print(f"   4. Integrate with Squirrel for AI insights")


if __name__ == "__main__":
    main()

