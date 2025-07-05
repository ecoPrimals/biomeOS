# biome.yaml Specification

**Version:** 1.0.0 | **Status:** Draft | **Date:** January 2025

---

## Overview

The `biome.yaml` file is the **genome of a biomeOS instance** - it defines the complete configuration, capabilities, and orchestration of all five Primals within a digital organism. This specification aligns with Toadstool's existing `BiomeManifest` structure while extending it for full biomeOS integration.

## File Structure

```yaml
# biome.yaml - The Digital Organism Genome
apiVersion: biomeOS/v1
kind: Biome
metadata:
  name: my-biome
  version: "1.0.0"
  description: "A digital organism for AI research"
  specialization: research  # development, enterprise, edge, scientific, research
  created: "2025-01-15T10:30:00Z"
  owner: "research-team"
  tags:
    - ai-research
    - gpu-compute
    - large-storage

# MYCORRHIZA Energy Flow Management
mycorrhiza:
  system_state: "closed"  # closed | private_open | commercial_open
  
  # Personal sovereignty - always available in closed systems
  personal_ai:
    enabled: true
    local_models:
      - llama.cpp
      - whisper.cpp
    api_keys:
      - provider: anthropic
        key_ref: claude_personal_key
      - provider: openai
        key_ref: gpt4_personal_key
      - provider: google
        key_ref: gemini_personal_key
  
  # Trust-based external access (private_open state)
  trusted_externals:
    enabled: false  # Set to true in private_open state
    grants: []      # Crypto keys granted on good faith
    
  # Commercial integrations (commercial_open state)
  commercial_access:
    enabled: false  # Set to true in commercial_open state
    licensed_providers: []  # AWS, GCP, Azure require payment
    
  # Security enforcement
  enforcement:
    deep_packet_inspection: true
    api_signature_detection: true
    behavioral_analysis: true
    threat_response: "block_and_preserve"  # block | warn | preserve

# Primal Orchestration Configuration
primals:
  # 🐕 BearDog - Security (Always First)
  beardog:
    enabled: true
    priority: 1
    startup_timeout: 30s
    config:
      security_level: high
      compliance: [gdpr, hipaa]
      hsm_integration: true
      
  # 🎼 Songbird - Service Mesh (Second)  
  songbird:
    enabled: true
    priority: 2
    startup_timeout: 45s
    depends_on: [beardog]
    config:
      discovery_backend: consul
      load_balancing: health_based
      federation_enabled: true
      
  # 🏰 NestGate - Storage (Third)
  nestgate:
    enabled: true
    priority: 3
    startup_timeout: 60s
    depends_on: [beardog, songbird]
    config:
      zfs_pool: "nestpool"
      tiered_storage: true
      protocols: [nfs, smb, s3]
      
  # 🍄 Toadstool - Runtime (Fourth)
  toadstool:
    enabled: true
    priority: 4
    startup_timeout: 30s
    depends_on: [beardog, songbird, nestgate]
    config:
      runtimes: [container, wasm, native, gpu]
      resource_limits:
        cpu: "0-15"
        memory: "64Gi"
        gpu: "0-3"
        
  # 🐿️ Squirrel - MCP Platform (Fifth)
  squirrel:
    enabled: true
    priority: 5
    startup_timeout: 45s
    depends_on: [beardog, songbird, toadstool]
    config:
      ai_providers: [openai, anthropic, gemini]
      plugin_sandboxing: strict
      mcp_transports: [stdio, websocket, sse]

# Service Definitions
services:
  # AI Research Services
  jupyter-lab:
    primal: toadstool
    runtime: container
    image: "jupyter/tensorflow-notebook:latest"
    resources:
      cpu: 4
      memory: "16Gi"
      gpu: 1
    volumes:
      - name: research-data
        mount: /home/jovyan/data
        size: "1Ti"
        tier: hot
    ports:
      - 8888:8888
    environment:
      JUPYTER_TOKEN: "${secrets.jupyter_token}"
      
  # AI Model Storage
  model-registry:
    primal: nestgate
    protocol: s3
    bucket: ml-models
    tier: warm
    retention: 365d
    versioning: true
    
  # Research Assistant Agent
  research-agent:
    primal: squirrel
    type: mcp_agent
    provider: anthropic
    model: claude-3-sonnet
    capabilities:
      - code_analysis
      - data_processing
      - research_assistance
    sandbox: strict
    memory_limit: "4Gi"

# Resource Management
resources:
  # Compute Resources
  compute:
    nodes:
      - name: primary
        cpu_cores: 16
        memory: "128Gi"
        gpu:
          - type: nvidia-a100
            count: 4
            memory: "40Gi"
        storage:
          local: "2Ti"
          
  # Storage Configuration
  storage:
    pools:
      - name: nestpool
        type: zfs
        devices:
          - /dev/nvme0n1  # Hot tier - NVMe
          - /dev/sda      # Warm tier - SSD
          - /dev/sdb      # Cold tier - HDD
        tiers:
          hot:
            devices: [/dev/nvme0n1]
            size: "2Ti"
            compression: lz4
          warm:
            devices: [/dev/sda]
            size: "8Ti"
            compression: gzip
          cold:
            devices: [/dev/sdb]
            size: "32Ti"
            compression: zstd
            
    volumes:
      - name: research-data
        pool: nestpool
        size: "1Ti"
        tier: hot
        snapshots: true
        backup: daily
        
      - name: model-storage
        pool: nestpool
        size: "10Ti"
        tier: warm
        deduplication: true
        
      - name: archive-data
        pool: nestpool
        size: "20Ti"
        tier: cold
        compression: zstd

# Security Configuration
security:
  # Authentication & Authorization
  authentication:
    provider: beardog
    methods: [jwt, mutual_tls]
    token_lifetime: 24h
    refresh_enabled: true
    
  # Service-to-Service Security
  service_mesh:
    mtls_enabled: true
    cipher_suites: [ECDHE-ECDSA-AES256-GCM-SHA384]
    cert_rotation: 7d
    
  # Secrets Management
  secrets:
    provider: beardog_hsm
    rotation_policy: 30d
    encryption: aes256
    
  # Compliance & Audit
  compliance:
    standards: [gdpr, hipaa]
    audit_retention: 7y
    log_encryption: true
    
  # Access Control
  rbac:
    roles:
      - name: researcher
        permissions:
          - jupyter:read,write
          - data:read,write
          - models:read
      - name: admin
        permissions:
          - "*:*"
    users:
      - name: alice
        roles: [researcher]
      - name: bob
        roles: [admin]

# Networking Configuration
networking:
  # Service Discovery
  discovery:
    provider: songbird
    backend: consul
    health_checks: true
    
  # Load Balancing
  load_balancing:
    algorithm: health_based
    health_check_interval: 30s
    failure_threshold: 3
    
  # Federation (Multi-Biome)
  federation:
    enabled: true
    peers:
      - name: production-biome
        endpoint: "https://prod.example.com"
        trust_domain: "prod.biome.local"
        
  # Network Policies
  policies:
    default_deny: true
    ingress:
      - from: [jupyter-lab]
        to: [model-registry]
        ports: [443]
    egress:
      - from: [research-agent]
        to: [external]
        domains: ["api.anthropic.com"]

# Monitoring & Observability
observability:
  # Metrics Collection
  metrics:
    provider: prometheus
    retention: 30d
    scrape_interval: 15s
    
  # Logging
  logging:
    provider: loki
    retention: 90d
    log_level: info
    structured: true
    
  # Tracing
  tracing:
    provider: jaeger
    sampling_rate: 0.1
    retention: 7d
    
  # Alerting
  alerting:
    provider: alertmanager
    channels:
      - type: slack
        webhook: "${secrets.slack_webhook}"
      - type: email
        recipients: ["admin@example.com"]

# AI Agent Configuration
agents:
  # Research Assistant
  research-assistant:
    provider: anthropic
    model: claude-3-sonnet
    runtime: squirrel
    capabilities:
      - name: code_analysis
        tools: [ast_parser, complexity_analyzer]
      - name: data_processing
        tools: [pandas_toolkit, numpy_ops]
      - name: research_assistance
        tools: [web_search, paper_analyzer]
    resources:
      memory: "4Gi"
      cpu: 2
      timeout: 300s
    sandbox:
      type: strict
      network_access: limited
      file_access: /workspace
      
  # Data Science Agent
  data-scientist:
    provider: openai
    model: gpt-4
    runtime: squirrel
    capabilities:
      - name: statistical_analysis
        tools: [scipy_stats, sklearn_toolkit]
      - name: visualization
        tools: [matplotlib, plotly]
    resources:
      memory: "8Gi"
      cpu: 4
      gpu: 1

# Deployment Configuration
deployment:
  # Bootstrap Sequence
  bootstrap:
    timeout: 300s
    retry_attempts: 3
    health_check_interval: 10s
    
  # Rolling Updates
  updates:
    strategy: rolling
    max_unavailable: 1
    max_surge: 1
    
  # Backup & Recovery
  backup:
    enabled: true
    schedule: "0 2 * * *"  # Daily at 2 AM
    retention: 30d
    destinations:
      - type: s3
        bucket: biome-backups
        encryption: true

# Environment-Specific Overrides
environments:
  development:
    security:
      authentication:
        token_lifetime: 7d
    resources:
      compute:
        nodes:
          - cpu_cores: 8
            memory: "32Gi"
            
  production:
    security:
      compliance:
        standards: [sox, pci_dss]
    observability:
      metrics:
        retention: 365d
      logging:
        retention: 2y

# Templates & Specializations
templates:
  # Quick Start Templates
  ai_research:
    description: "AI research environment with GPU compute"
    includes:
      - jupyter-lab
      - model-registry
      - research-agent
    resources:
      gpu_required: true
      storage_tier: hot
      
  secure_enterprise:
    description: "Enterprise environment with enhanced security"
    security:
      compliance: [sox, gdpr, hipaa]
      encryption: fips_140_2
    observability:
      audit_level: detailed
      
  edge_computing:
    description: "Lightweight edge deployment"
    primals:
      toadstool:
        config:
          runtimes: [container, wasm]
      squirrel:
        enabled: false
    resources:
      compute:
        memory_limit: "16Gi"
        cpu_limit: 4

# Validation Rules
validation:
  required_primals: [beardog, songbird]
  security_minimum: medium
  resource_limits:
    max_memory: "1Ti"
    max_storage: "100Ti"
  compliance_checks:
    - encryption_at_rest
    - access_logging
    - backup_verification 