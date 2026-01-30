#!/usr/bin/env python3
"""
🎊 NUCLEUS AI Coordination Demo
================================

Demonstrates Squirrel-like AI coordination between:
- Local AI (Toadstool + GPU via barraCUDA)
- Large AI (Anthropic Claude, OpenAI GPT-4)
- Tower security (BearDog + Songbird)

Architecture:
  User Request
      ↓
  AI Coordinator (this script - Squirrel pattern)
      ↓
      ├──→ Toadstool (local, fast, private)
      ├──→ Anthropic Claude (cloud, powerful)
      └──→ OpenAI GPT-4 (cloud, alternative)

Date: January 30, 2026
Status: NUCLEUS Tower + Node Validated
"""

import json
import socket
import sys
import os
from pathlib import Path
try:
    import tomli
except ImportError:
    tomli = None

# Add color support
class Colors:
    HEADER = '\033[95m'
    BLUE = '\033[94m'
    CYAN = '\033[96m'
    GREEN = '\033[92m'
    YELLOW = '\033[93m'
    RED = '\033[91m'
    ENDC = '\033[0m'
    BOLD = '\033[1m'

def load_api_keys():
    """Load API keys from testing-secrets"""
    secrets_path = Path.home() / "Development/ecoPrimals/testing-secrets/api-keys.toml"
    if not secrets_path.exists():
        print(f"{Colors.RED}❌ API keys not found at: {secrets_path}{Colors.ENDC}")
        return None
    
    # Parse the mixed-format file
    api_keys = {"ai_providers": {}}
    
    with open(secrets_path, "r") as f:
        lines = f.readlines()
        
    in_section = None
    for line in lines:
        line = line.strip()
        
        # Skip comments and empty lines
        if not line or line.startswith("#"):
            continue
        
        # Section headers
        if line.startswith("["):
            in_section = line.strip("[]")
            if in_section not in api_keys:
                api_keys[in_section] = {}
            continue
        
        # Key-value pairs (TOML style)
        if "=" in line and in_section:
            key, value = line.split("=", 1)
            key = key.strip()
            value = value.strip().strip('"')
            api_keys[in_section][key] = value
    
    return api_keys

def query_toadstool(prompt: str) -> dict:
    """Query Toadstool for local compute capabilities"""
    socket_path = f"/run/user/{os.getuid()}/biomeos/toadstool.jsonrpc.sock"
    
    if not os.path.exists(socket_path):
        return {"error": f"Toadstool socket not found: {socket_path}", "available": False}
    
    try:
        sock = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
        sock.settimeout(3.0)
        sock.connect(socket_path)
        
        # Query capabilities
        request = {
            "jsonrpc": "2.0",
            "method": "health",
            "id": 1
        }
        
        sock.sendall(json.dumps(request).encode() + b'\n')
        response = sock.recv(4096).decode()
        sock.close()
        
        result = json.loads(response)
        return {
            "available": True,
            "status": result.get("result", {}).get("status"),
            "capabilities": ["local_compute", "gpu", "fast", "private"],
            "method": "local_inference"
        }
    except Exception as e:
        return {"error": str(e), "available": False}

def query_anthropic(prompt: str, api_key: str) -> dict:
    """Query Anthropic Claude (simulated - would use httpx in production)"""
    try:
        import httpx
        
        headers = {
            "x-api-key": api_key,
            "anthropic-version": "2023-06-01",
            "content-type": "application/json"
        }
        
        data = {
            "model": "claude-3-sonnet-20240229",
            "max_tokens": 1024,
            "messages": [{"role": "user", "content": prompt}]
        }
        
        with httpx.Client(timeout=30.0) as client:
            response = client.post(
                "https://api.anthropic.com/v1/messages",
                headers=headers,
                json=data
            )
            
            if response.status_code == 200:
                result = response.json()
                return {
                    "available": True,
                    "provider": "anthropic_claude",
                    "model": "claude-3-sonnet",
                    "response": result.get("content", [{}])[0].get("text", ""),
                    "tokens": result.get("usage", {})
                }
            else:
                return {"error": f"HTTP {response.status_code}", "available": False}
    except ImportError:
        return {
            "available": False,
            "error": "httpx not installed (pip install httpx)",
            "simulated": True,
            "provider": "anthropic_claude",
            "model": "claude-3-sonnet",
            "response": "[Simulated] Claude would process this request in production"
        }
    except Exception as e:
        return {"error": str(e), "available": False}

def query_openai(prompt: str, api_key: str) -> dict:
    """Query OpenAI GPT-4 (simulated - would use httpx in production)"""
    try:
        import httpx
        
        headers = {
            "Authorization": f"Bearer {api_key}",
            "Content-Type": "application/json"
        }
        
        data = {
            "model": "gpt-4",
            "messages": [{"role": "user", "content": prompt}],
            "max_tokens": 1024
        }
        
        with httpx.Client(timeout=30.0) as client:
            response = client.post(
                "https://api.openai.com/v1/chat/completions",
                headers=headers,
                json=data
            )
            
            if response.status_code == 200:
                result = response.json()
                return {
                    "available": True,
                    "provider": "openai_gpt4",
                    "model": "gpt-4",
                    "response": result.get("choices", [{}])[0].get("message", {}).get("content", ""),
                    "tokens": result.get("usage", {})
                }
            else:
                return {"error": f"HTTP {response.status_code}", "available": False}
    except ImportError:
        return {
            "available": False,
            "error": "httpx not installed (pip install httpx)",
            "simulated": True,
            "provider": "openai_gpt4",
            "model": "gpt-4",
            "response": "[Simulated] GPT-4 would process this request in production"
        }
    except Exception as e:
        return {"error": str(e), "available": False}

def ai_coordinator(task: str, complexity: str = "auto"):
    """
    Squirrel-pattern AI coordinator
    
    Routes tasks based on complexity:
    - simple: Toadstool (local, fast, private)
    - complex: Claude or GPT-4 (cloud, powerful)
    """
    
    print(f"\n{Colors.HEADER}{Colors.BOLD}╔═══════════════════════════════════════════════════════╗{Colors.ENDC}")
    print(f"{Colors.HEADER}{Colors.BOLD}║  🐿️  NUCLEUS AI Coordinator (Squirrel Pattern)      ║{Colors.ENDC}")
    print(f"{Colors.HEADER}{Colors.BOLD}╚═══════════════════════════════════════════════════════╝{Colors.ENDC}\n")
    
    print(f"{Colors.CYAN}📝 Task:{Colors.ENDC} {task}\n")
    
    # Load API keys
    print(f"{Colors.BLUE}🔑 Loading API keys...{Colors.ENDC}")
    api_keys = load_api_keys()
    if not api_keys:
        print(f"{Colors.RED}❌ Failed to load API keys{Colors.ENDC}")
        return
    
    anthropic_key = api_keys.get("ai_providers", {}).get("anthropic_api_key", "")
    openai_key = api_keys.get("ai_providers", {}).get("openai_api_key", "")
    print(f"{Colors.GREEN}✅ API keys loaded{Colors.ENDC}\n")
    
    # Check available providers
    print(f"{Colors.BLUE}🔍 Discovering AI providers...{Colors.ENDC}")
    print(f"{Colors.HEADER}═══════════════════════════════════════════════════════{Colors.ENDC}\n")
    
    # 1. Check Toadstool (Local AI)
    print(f"{Colors.YELLOW}1. Toadstool (Local Compute):{Colors.ENDC}")
    toadstool_status = query_toadstool(task)
    if toadstool_status.get("available"):
        print(f"   {Colors.GREEN}✅ Available{Colors.ENDC}")
        print(f"   Status: {toadstool_status.get('status')}")
        print(f"   Capabilities: {', '.join(toadstool_status.get('capabilities', []))}")
        print(f"   Advantages: Fast, Private, No API costs")
    else:
        print(f"   {Colors.RED}❌ Unavailable{Colors.ENDC}")
        print(f"   Error: {toadstool_status.get('error')}")
    print()
    
    # 2. Check Anthropic Claude
    print(f"{Colors.YELLOW}2. Anthropic Claude (Large AI):{Colors.ENDC}")
    if anthropic_key:
        print(f"   {Colors.GREEN}✅ API Key configured{Colors.ENDC}")
        print(f"   Model: claude-3-sonnet-20240229")
        print(f"   Advantages: Powerful, Context-aware, Latest")
    else:
        print(f"   {Colors.RED}❌ No API key{Colors.ENDC}")
    print()
    
    # 3. Check OpenAI GPT-4
    print(f"{Colors.YELLOW}3. OpenAI GPT-4 (Large AI):{Colors.ENDC}")
    if openai_key:
        print(f"   {Colors.GREEN}✅ API Key configured{Colors.ENDC}")
        print(f"   Model: gpt-4")
        print(f"   Advantages: Well-known, Reliable, Powerful")
    else:
        print(f"   {Colors.RED}❌ No API key{Colors.ENDC}")
    print()
    
    # Decision logic (Squirrel pattern)
    print(f"{Colors.HEADER}═══════════════════════════════════════════════════════{Colors.ENDC}")
    print(f"{Colors.BLUE}{Colors.BOLD}🧠 AI Coordinator Decision:{Colors.ENDC}\n")
    
    # Analyze task complexity
    if complexity == "auto":
        simple_keywords = ["hello", "ping", "test", "status", "health"]
        is_simple = any(kw in task.lower() for kw in simple_keywords)
        complexity = "simple" if is_simple else "complex"
    
    print(f"   Task Complexity: {Colors.CYAN}{complexity.upper()}{Colors.ENDC}")
    
    if complexity == "simple" and toadstool_status.get("available"):
        print(f"   {Colors.GREEN}✅ Routing to: Toadstool (Local Compute){Colors.ENDC}")
        print(f"   Reason: Simple task, keep it local and fast")
        provider = "toadstool"
    elif anthropic_key:
        print(f"   {Colors.GREEN}✅ Routing to: Anthropic Claude{Colors.ENDC}")
        print(f"   Reason: Complex task requires large AI model")
        provider = "anthropic"
    elif openai_key:
        print(f"   {Colors.GREEN}✅ Routing to: OpenAI GPT-4{Colors.ENDC}")
        print(f"   Reason: Complex task, Claude unavailable")
        provider = "openai"
    else:
        print(f"   {Colors.RED}❌ No AI providers available{Colors.ENDC}")
        return
    
    print()
    print(f"{Colors.HEADER}═══════════════════════════════════════════════════════{Colors.ENDC}")
    print(f"{Colors.BLUE}{Colors.BOLD}🚀 Executing query...{Colors.ENDC}\n")
    
    # Execute query
    if provider == "toadstool":
        result = toadstool_status
        print(f"{Colors.GREEN}✅ Toadstool Health Check: {result.get('status')}{Colors.ENDC}")
        print(f"\nCapabilities: {', '.join(result.get('capabilities', []))}")
        print(f"\n{Colors.CYAN}Note:{Colors.ENDC} Toadstool is ready for local AI inference!")
        print(f"For actual AI queries, Toadstool would use barraCUDA GPU framework.")
    elif provider == "anthropic":
        print(f"{Colors.YELLOW}Querying Anthropic Claude...{Colors.ENDC}")
        result = query_anthropic(task, anthropic_key)
        if result.get("available"):
            print(f"{Colors.GREEN}✅ Response received{Colors.ENDC}\n")
            print(f"Response: {result.get('response')}")
            if result.get("tokens"):
                print(f"\nTokens: {result.get('tokens')}")
        elif result.get("simulated"):
            print(f"{Colors.YELLOW}⚠️  Running in simulation mode{Colors.ENDC}")
            print(f"Reason: {result.get('error')}\n")
            print(f"Response: {result.get('response')}")
        else:
            print(f"{Colors.RED}❌ Query failed: {result.get('error')}{Colors.ENDC}")
    elif provider == "openai":
        print(f"{Colors.YELLOW}Querying OpenAI GPT-4...{Colors.ENDC}")
        result = query_openai(task, openai_key)
        if result.get("available"):
            print(f"{Colors.GREEN}✅ Response received{Colors.ENDC}\n")
            print(f"Response: {result.get('response')}")
            if result.get("tokens"):
                print(f"\nTokens: {result.get('tokens')}")
        elif result.get("simulated"):
            print(f"{Colors.YELLOW}⚠️  Running in simulation mode{Colors.ENDC}")
            print(f"Reason: {result.get('error')}\n")
            print(f"Response: {result.get('response')}")
        else:
            print(f"{Colors.RED}❌ Query failed: {result.get('error')}{Colors.ENDC}")
    
    print(f"\n{Colors.HEADER}═══════════════════════════════════════════════════════{Colors.ENDC}")
    print(f"{Colors.GREEN}{Colors.BOLD}🎊 AI Coordination Complete!{Colors.ENDC}\n")

def main():
    """Main entry point"""
    
    if len(sys.argv) < 2:
        print(f"{Colors.HEADER}{Colors.BOLD}NUCLEUS AI Coordinator Demo{Colors.ENDC}\n")
        print("Usage:")
        print(f"  {sys.argv[0]} '<task>' [complexity]\n")
        print("Examples:")
        print(f"  {sys.argv[0]} 'Hello, test local compute' simple")
        print(f"  {sys.argv[0]} 'Explain quantum computing' complex")
        print(f"  {sys.argv[0]} 'What is biomeOS?' auto")
        print()
        sys.exit(1)
    
    task = sys.argv[1]
    complexity = sys.argv[2] if len(sys.argv) > 2 else "auto"
    
    ai_coordinator(task, complexity)

if __name__ == "__main__":
    main()
